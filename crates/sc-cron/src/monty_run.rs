//! In-process Monty feeds with application-owned host dispatch.
//!
//! Host callbacks run on the blocking feed thread with the same row deadline.
//! Arbitrary synchronous Rust application code remains cooperative.

use crate::environment::EnvironmentSnapshot;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use monty::{MontyRepl, ReplProgress};
use monty_types::{
    CompileOptions, ExcType, ExtFunctionResult, MontyException, MontyObject, NamedValues,
    PrintWriter, ResourceLimits, ResourceTracker,
};
use serde_json::Value;

use crate::monty_compose_common::ComposeHostError;
use crate::monty_host::{HostDispatchError, MontyHostCall, MontyHostRegistry};
use crate::{CronError, Generation, RunOutcome, StopDecision};

const STOP_FN: &str = "cron_stop";
const TICK_FN: &str = "cron_tick";
const DENY_RECOVERY: &str = "Fix hook source or register the required application host function.";
const CODE_HOST_DENIED: &str = "HOST_DENIED";
const CODE_HOST_UNDEFINED: &str = "HOST_UNDEFINED";
#[cfg_attr(feature = "monty-test-handshake", allow(dead_code))]
const CODE_HOST_OS: &str = "HOST_OS";
const CODE_HOST_RAISED: &str = "HOST_RAISED";
const CODE_HOST_REPLY: &str = "HOST_REPLY";
const STOP_REPLY_RECOVERY: &str = "Return an object with string \"action\" (continue, disable, or delete) and a non-empty \"reason\" when disabling or deleting.";
const CODE_FEED_TASK: &str = "FEED_TASK_FAILED";
const FEED_TASK_RECOVERY: &str =
    "Retry the run; if it persists, inspect blocking-pool capacity and hook source.";

/// Which function one feed calls. The first feed of a generation also compiles
/// `source`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MontyPhase {
    Stop,
    Tick,
}

/// A repl whose source has already been fed for `generation`.
pub(crate) struct MontySession {
    repl: Box<MontyRepl>,
    generation: Generation,
    pub(crate) environment: Arc<EnvironmentSnapshot>,
    pub(crate) deadline: Instant,
}

/// What one feed did, plus the repl when a later feed of this generation may
/// reuse it.
pub(crate) struct FeedDone {
    pub session: Option<MontySession>,
    pub result: FeedResult,
}

/// The worker's decision input. `Continue` is only returned for a stop feed.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FeedResult {
    Continue,
    Disable(String),
    Delete(String),
    Outcome(RunOutcome),
}

/// Where a feed is when it fails. The label is display text, not a control flag
/// stored in a string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeedStage {
    Compile,
    Stop,
    Tick,
}

impl FeedStage {
    const fn label(self) -> &'static str {
        match self {
            Self::Compile => "compile",
            Self::Stop => "stop check",
            Self::Tick => "tick",
        }
    }

    const fn from_phase(phase: MontyPhase) -> Self {
        match phase {
            MontyPhase::Stop => Self::Stop,
            MontyPhase::Tick => Self::Tick,
        }
    }
}

/// Why a stop reply could not be applied.
#[derive(Clone, Debug, PartialEq, Eq)]
enum StopReplyError {
    NotObject,
    MissingAction,
    MissingReason { action: String },
    UnknownAction,
}

impl fmt::Display for StopReplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotObject => formatter.write_str("reply is not an object"),
            Self::MissingAction => formatter.write_str("reply has no string \"action\""),
            Self::MissingReason { action } => {
                write!(formatter, "action {action:?} needs a \"reason\"")
            }
            Self::UnknownAction => {
                formatter.write_str("unknown action; expected continue, disable or delete")
            }
        }
    }
}

/// Inputs for one blocking-pool feed shared by the worker and `run_due`.
pub(crate) struct FeedLaunch {
    /// Repl from the previous feed of this generation, if any.
    pub(crate) logger: sc_observability_types::SharedCanonicalLog,
    pub(crate) correlation: crate::worker::RunCorrelation,
    pub(crate) session: Option<MontySession>,
    /// Hook source.
    pub(crate) source: String,
    /// Stop or tick.
    pub(crate) phase: MontyPhase,
    /// Schedule generation the repl must match.
    pub(crate) generation: Generation,
    /// Schedule name placed in the hook JSON.
    pub(crate) name: crate::ScheduleName,
    /// Schedule payload placed in the hook JSON.
    pub(crate) payload: Value,
    /// Schedule timeout placed in the hook JSON.
    pub(crate) timeout: crate::RunTimeout,
    /// Monotonic deadline for this run.
    pub(crate) deadline: Instant,
    /// Immutable application host registry.
    pub(crate) hosts: std::sync::Arc<MontyHostRegistry>,
    pub(crate) environment: Arc<EnvironmentSnapshot>,
}

/// Builds the hook request and runs one feed on the blocking pool.
///
/// The worker and [`crate::Scheduler::run_due`] both call this so the request,
/// the blocking handoff, and a panicked task are handled once.
pub(crate) async fn drive_feed<C: crate::Clock + ?Sized>(
    clock: &C,
    launch: FeedLaunch,
) -> FeedDone {
    let FeedLaunch {
        logger,
        correlation,
        session,
        source,
        phase,
        generation,
        name,
        payload,
        timeout,
        deadline,
        hosts,
        environment,
    } = launch;
    let left = deadline.saturating_duration_since(clock.monotonic_now());
    let stamp = crate::Timestamp::from_datetime(crate::clock::wall_after(clock.now(), left));
    let event = match phase {
        MontyPhase::Stop => crate::HookEvent::Stop,
        MontyPhase::Tick => crate::HookEvent::Tick,
    };
    let call = crate::HookCall {
        event,
        schedule: &name,
        payload: &payload,
        timeout,
        deadline,
    };
    let hook = crate::hook::request(&call, stamp);
    // Project the injected clock's remaining row budget onto the host clock.
    // In production the original deadline is the tighter bound; manual clocks
    // must not extend a blocking child by the virtual time already advanced.
    let run_deadline = deadline.min(Instant::now() + left);
    match tokio::task::spawn_blocking(move || {
        execute_feed_with_logger(
            session,
            &source,
            phase,
            generation,
            left,
            &hook,
            &hosts,
            run_deadline,
            logger,
            environment,
            correlation,
        )
    })
    .await
    {
        Ok(mut done) => {
            if let Some(session) = &mut done.session {
                session.deadline = deadline;
            }
            done
        }
        Err(error) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
        Err(error) => FeedDone {
            session: None,
            result: FeedResult::Outcome(RunOutcome::Failed(format!(
                "{CODE_FEED_TASK}: monty feed task join failed: {error}; {FEED_TASK_RECOVERY}"
            ))),
        },
    }
}

/// Runs one feed without application hosts; used by interpreter-only tests.
#[cfg(all(test, feature = "monty-test-handshake"))]
pub(crate) fn execute_feed(
    session: Option<MontySession>,
    source: &str,
    phase: MontyPhase,
    generation: Generation,
    budget: Duration,
    hook: &Value,
    run_deadline: Instant,
) -> FeedDone {
    execute_feed_with_logger(
        session,
        source,
        phase,
        generation,
        budget,
        hook,
        &MontyHostRegistry::new(),
        run_deadline,
        crate::noop_logger(),
        Arc::new(EnvironmentSnapshot::default()),
        None,
    )
}

/// Runs one blocking feed while preserving the injected logger handle.
#[allow(
    clippy::too_many_arguments,
    reason = "one feed carries source, session, run context, registry and logger"
)]
pub(crate) fn execute_feed_with_logger(
    session: Option<MontySession>,
    source: &str,
    phase: MontyPhase,
    generation: Generation,
    budget: Duration,
    hook: &Value,
    hosts: &MontyHostRegistry,
    run_deadline: Instant,
    logger: sc_observability_types::SharedCanonicalLog,
    environment: Arc<EnvironmentSnapshot>,
    correlation: crate::worker::RunCorrelation,
) -> FeedDone {
    let (mut session, call_budget) = match open_session(
        session,
        source,
        generation,
        budget,
        Arc::clone(&environment),
        run_deadline,
        hosts,
        &logger,
    ) {
        Ok(ready) => ready,
        Err(done) => return done,
    };
    session
        .repl
        .tracker_mut()
        .set_max_feed_duration(feed_budget(call_budget));
    resume_started_feed(
        session,
        phase,
        hook,
        hosts,
        run_deadline,
        &logger,
        (&environment, &correlation),
    )
}

/// Keeps a repl only when its generation matches, otherwise compiles `source`.
#[expect(
    clippy::too_many_arguments,
    reason = "session open carries the occurrence environment and host context"
)]
fn open_session(
    mut session: Option<MontySession>,
    source: &str,
    generation: Generation,
    budget: Duration,
    environment: Arc<EnvironmentSnapshot>,
    deadline: Instant,
    hosts: &MontyHostRegistry,
    logger: &sc_observability_types::SharedCanonicalLog,
) -> Result<(MontySession, Duration), FeedDone> {
    if session
        .as_ref()
        .is_some_and(|current| current.generation != generation)
    {
        session = None;
    }
    match session {
        Some(mut session) => {
            session.environment = environment;
            session.deadline = deadline;
            Ok((session, budget))
        }
        None => {
            let compile_started = Instant::now();
            let compiled = compile(
                source,
                generation,
                feed_budget(budget),
                environment,
                deadline,
                hosts,
                logger,
            )?;
            Ok((compiled, budget.saturating_sub(compile_started.elapsed())))
        }
    }
}

/// Starts a hook feed and uses the same suspension driver as initialization.
fn resume_started_feed(
    session: MontySession,
    phase: MontyPhase,
    hook: &Value,
    hosts: &MontyHostRegistry,
    run_deadline: Instant,
    logger: &sc_observability_types::SharedCanonicalLog,
    context: (&EnvironmentSnapshot, &crate::worker::RunCorrelation),
) -> FeedDone {
    let (environment, correlation) = context;
    let stage = FeedStage::from_phase(phase);
    let MontySession {
        repl,
        generation,
        environment: snapshot,
        deadline,
    } = session;
    let progress = match repl.feed_start(
        phase_snippet(phase),
        snippet_inputs(hook, hosts),
        PrintWriter::Disabled,
    ) {
        Ok(progress) => progress,
        Err(error) => return from_exception(error.error, stage, environment),
    };
    match drive_progress(
        progress,
        stage,
        phase,
        hook,
        hosts,
        run_deadline,
        logger,
        &snapshot,
        correlation,
    ) {
        Ok((repl, value)) => finish_phase(phase, generation, repl, &value, snapshot, deadline),
        Err(done) => done,
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "resumable driver shares row, phase, host and snapshot context"
)]
fn drive_progress(
    mut progress: ReplProgress,
    stage: FeedStage,
    phase: MontyPhase,
    hook: &Value,
    hosts: &MontyHostRegistry,
    deadline: Instant,
    logger: &sc_observability_types::SharedCanonicalLog,
    environment: &Arc<EnvironmentSnapshot>,
    correlation: &crate::worker::RunCorrelation,
) -> Result<(MontyRepl, MontyObject), FeedDone> {
    loop {
        if Instant::now() >= deadline {
            return Err(timed_out());
        }
        progress = match progress {
            ReplProgress::Complete { repl, value } => return Ok((repl, value)),
            ReplProgress::FunctionCall(call) => {
                if stage == FeedStage::Compile {
                    return Err(denied(
                        stage,
                        "application host calls are denied during initialization",
                    ));
                }
                resume_host(
                    call,
                    stage,
                    phase,
                    hook,
                    hosts,
                    deadline,
                    logger,
                    environment,
                    correlation,
                )?
            }
            ReplProgress::OsCall(call) => resume_os(call, stage, deadline, environment)?,
            ReplProgress::NameLookup(lookup) => {
                return Err(undefined_name(stage, &environment.redact(&lookup.name)));
            }
            ReplProgress::ResolveFutures(_) => return Err(denied(stage, "unresolved futures")),
        };
    }
}

fn resume_os(
    call: monty::ReplOsCall,
    stage: FeedStage,
    deadline: Instant,
    environment: &EnvironmentSnapshot,
) -> Result<ReplProgress, FeedDone> {
    #[cfg(not(feature = "monty-test-handshake"))]
    let _ = deadline;
    let result = match &call.function_call {
        monty_types::OsFunctionCall::Getenv(args) => Some(ExtFunctionResult::Return(
            environment.get(&args.key).map_or_else(
                || args.default.clone(),
                |value| MontyObject::string(value.to_owned()),
            ),
        )),
        monty_types::OsFunctionCall::GetEnviron => Some(ExtFunctionResult::Return(
            MontyObject::dict(environment.iter().map(|(name, value)| {
                (
                    MontyObject::string(name.to_owned()),
                    MontyObject::string(value.to_owned()),
                )
            })),
        )),
        _ => None,
    };
    if let Some(result) = result {
        call.resume(result, PrintWriter::Disabled)
            .map_err(|error| from_exception(error.error, stage, environment))
    } else {
        #[cfg(feature = "monty-test-handshake")]
        {
            if stage == FeedStage::Compile || !crate::monty_test_os::os_call_allowed_in_tests(&call)
            {
                let operation: &'static str = call.function_call.into();
                return Err(denied_os(stage, operation));
            }
            crate::monty_test_os::resume_test_os(call, Some(deadline))
        }
        #[cfg(not(feature = "monty-test-handshake"))]
        {
            let operation: &'static str = call.function_call.into();
            Err(denied_os(stage, operation))
        }
    }
}

fn feed_budget(budget: Duration) -> Duration {
    if budget.is_zero() {
        Duration::from_nanos(1)
    } else {
        budget
    }
}

fn compile(
    source: &str,
    generation: Generation,
    budget: Duration,
    environment: Arc<EnvironmentSnapshot>,
    deadline: Instant,
    hosts: &MontyHostRegistry,
    logger: &sc_observability_types::SharedCanonicalLog,
) -> Result<MontySession, FeedDone> {
    let limits = ResourceLimits::default().max_feed_duration(budget);
    let repl = MontyRepl::new(
        "sc-cron-hook",
        ResourceTracker::new(limits),
        CompileOptions::default(),
    );
    let hook = serde_json::json!({});
    let progress = repl
        .feed_start(source, snippet_inputs(&hook, hosts), PrintWriter::Disabled)
        .map_err(|error| from_exception(error.error, FeedStage::Compile, &environment))?;
    let (repl, _) = drive_progress(
        progress,
        FeedStage::Compile,
        MontyPhase::Stop,
        &hook,
        hosts,
        deadline,
        logger,
        &environment,
        &None,
    )?;
    if !repl.has_function(STOP_FN) || !repl.has_function(TICK_FN) {
        return Err(failed_hook(format!(
            "source must define {STOP_FN} and {TICK_FN}"
        )));
    }
    Ok(MontySession {
        repl: Box::new(repl),
        generation,
        environment,
        deadline,
    })
}

fn from_exception(
    error: MontyException,
    stage: FeedStage,
    environment: &EnvironmentSnapshot,
) -> FeedDone {
    if is_feed_time_limit(&error) {
        return timed_out();
    }
    if stage == FeedStage::Compile || error.exc_type() == ExcType::SyntaxError {
        return failed_hook(
            environment.redact(error.message().unwrap_or("source did not compile")),
        );
    }
    let detail = environment.redact(error.message().unwrap_or("hook raised"));
    failed(format!(
        "{CODE_HOST_RAISED}: {label}: {detail}",
        label = stage.label()
    ))
}

fn is_feed_time_limit(error: &MontyException) -> bool {
    error.exc_type() == ExcType::TimeoutError
        && error
            .message()
            .is_some_and(|message| message.contains("feed time limit"))
}

fn failed(detail: impl Into<String>) -> FeedDone {
    FeedDone {
        session: None,
        result: FeedResult::Outcome(RunOutcome::Failed(detail.into())),
    }
}

fn failed_hook(detail: impl fmt::Display) -> FeedDone {
    FeedDone {
        session: None,
        result: FeedResult::Outcome(RunOutcome::Failed(
            CronError::HookFailed(detail.to_string()).recorded(),
        )),
    }
}

fn parse_stop(value: &MontyObject) -> Result<StopDecision, StopReplyError> {
    let pairs = value.as_ref().pairs().ok_or(StopReplyError::NotObject)?;
    let mut action = None;
    let mut reason = None;
    for (key, item) in pairs {
        let Some(key) = key.as_str() else {
            continue;
        };
        match key {
            "action" => action = item.as_str().map(str::to_owned),
            "reason" => reason = item.as_str().map(str::to_owned),
            _ => {}
        }
    }
    let action = action.ok_or(StopReplyError::MissingAction)?;
    let needs_reason = || {
        reason
            .clone()
            .filter(|reason| !reason.trim().is_empty())
            .ok_or_else(|| StopReplyError::MissingReason {
                action: action.clone(),
            })
    };
    match action.as_str() {
        "continue" => Ok(StopDecision::Continue),
        "disable" => needs_reason().map(StopDecision::Disable),
        "delete" => needs_reason().map(StopDecision::Delete),
        _ => Err(StopReplyError::UnknownAction),
    }
}

fn phase_snippet(phase: MontyPhase) -> &'static str {
    match phase {
        MontyPhase::Stop => "cron_stop(__hook)",
        MontyPhase::Tick => "cron_tick(__hook)",
    }
}

fn snippet_inputs(hook: &Value, hosts: &MontyHostRegistry) -> NamedValues {
    let mut inputs = NamedValues::new();
    inputs.push("__hook", crate::monty_host::json_to_monty(hook));
    for name in hosts.names() {
        inputs.push(
            name.to_owned(),
            MontyObject::function(name.to_owned(), None),
        );
    }
    inputs
}

fn host_call(
    call: &monty::ReplFunctionCall,
    phase: MontyPhase,
    hook: &Value,
    deadline: Instant,
    environment: Arc<EnvironmentSnapshot>,
) -> Result<MontyHostCall, &'static str> {
    let args = call
        .args
        .args()
        .map(crate::monty_host::monty_value_to_json)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "host positional argument is not JSON; integers must be between -9223372036854775808 and 18446744073709551615")?;
    let mut kwargs = serde_json::Map::new();
    for (key, value) in call.args.kwargs() {
        let key = key.as_str().ok_or("host keyword name is not a string")?;
        kwargs.insert(
            key.to_owned(),
            crate::monty_host::monty_value_to_json(value)
                .map_err(|_| "host keyword argument is not JSON; integers must be between -9223372036854775808 and 18446744073709551615")?,
        );
    }
    let schedule = crate::ScheduleName::new(
        hook.get("schedule")
            .and_then(Value::as_str)
            .unwrap_or("monty-hook"),
    )
    .map_err(|_| "invalid schedule context")?;
    Ok(MontyHostCall {
        name: crate::MontyHostName::new(call.function_name.clone())
            .map_err(|_| "invalid host name")?,
        args,
        kwargs,
        schedule,
        phase: match phase {
            MontyPhase::Stop => crate::HookEvent::Stop,
            MontyPhase::Tick => crate::HookEvent::Tick,
        },
        deadline,
        environment,
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "dispatch retains row, interpreter and logger context"
)]
fn resume_host(
    call: monty::ReplFunctionCall,
    stage: FeedStage,
    phase: MontyPhase,
    hook: &Value,
    hosts: &MontyHostRegistry,
    deadline: Instant,
    logger: &sc_observability_types::SharedCanonicalLog,
    environment: &Arc<EnvironmentSnapshot>,
    correlation: &crate::worker::RunCorrelation,
) -> Result<ReplProgress, FeedDone> {
    if Instant::now() >= deadline {
        return Err(timed_out());
    }
    let context = host_call(&call, phase, hook, deadline, Arc::clone(environment))
        .map_err(|why| denied(stage, why))?;
    let result = hosts.invoke(&context, logger, environment, correlation);
    if Instant::now() >= deadline {
        return Err(timed_out());
    }
    let value = match result {
        Ok(value) => value,
        Err(HostDispatchError::Undefined) => {
            return Err(undefined_name(
                stage,
                &environment.redact(context.name.as_str()),
            ));
        }
        Err(HostDispatchError::Denied(why)) => return Err(denied(stage, why)),
        Err(HostDispatchError::Handler(error)) => {
            return Err(failed(format!(
                "HOST_CALL_FAILED: {}: {}; Fix the registered application handler.",
                stage.label(),
                environment.redact(&error.to_string())
            )));
        }
        Err(HostDispatchError::Child(ComposeHostError::TimedOut)) => return Err(timed_out()),
        Err(HostDispatchError::Child(error)) => {
            return Err(compose_failed(stage, &error, environment));
        }
    };
    call.resume(
        ExtFunctionResult::Return(crate::monty_host::json_to_monty(&value)),
        PrintWriter::Disabled,
    )
    .map_err(|error| from_exception(error.error, stage, environment))
}

fn finish_phase(
    phase: MontyPhase,
    generation: Generation,
    repl: MontyRepl,
    value: &MontyObject,
    environment: Arc<EnvironmentSnapshot>,
    deadline: Instant,
) -> FeedDone {
    match phase {
        MontyPhase::Stop => match parse_stop(value) {
            Ok(StopDecision::Continue) => kept(
                repl,
                generation,
                FeedResult::Continue,
                environment,
                deadline,
            ),
            Ok(StopDecision::Disable(reason)) => kept(
                repl,
                generation,
                FeedResult::Disable(environment.redact(&reason)),
                environment,
                deadline,
            ),
            Ok(StopDecision::Delete(reason)) => kept(
                repl,
                generation,
                FeedResult::Delete(environment.redact(&reason)),
                environment,
                deadline,
            ),
            Err(why) => {
                drop(repl);
                let why = environment.redact(&why.to_string());
                failed(format!(
                    "stop check: {CODE_HOST_REPLY}: {why}; {STOP_REPLY_RECOVERY}"
                ))
            }
        },
        MontyPhase::Tick => kept(
            repl,
            generation,
            FeedResult::Outcome(RunOutcome::Ok),
            environment,
            deadline,
        ),
    }
}

fn kept(
    repl: MontyRepl,
    generation: Generation,
    result: FeedResult,
    environment: Arc<EnvironmentSnapshot>,
    deadline: Instant,
) -> FeedDone {
    FeedDone {
        session: Some(MontySession {
            repl: Box::new(repl),
            generation,
            environment,
            deadline,
        }),
        result,
    }
}

fn denied(stage: FeedStage, why: &str) -> FeedDone {
    failed(format!(
        "{CODE_HOST_DENIED}: {label}: {why}; {DENY_RECOVERY}",
        label = stage.label()
    ))
}

fn undefined_name(stage: FeedStage, name: &str) -> FeedDone {
    failed(format!(
        "{CODE_HOST_UNDEFINED}: {label}: undefined name `{name}`; Fix hook source; `{name}` is not defined.",
        label = stage.label()
    ))
}

#[cfg_attr(feature = "monty-test-handshake", allow(dead_code))]
fn denied_os(stage: FeedStage, operation: &str) -> FeedDone {
    failed(format!(
        "{CODE_HOST_OS}: {label}: os call {operation}; {DENY_RECOVERY}",
        label = stage.label()
    ))
}

fn compose_failed(
    stage: FeedStage,
    error: &ComposeHostError,
    environment: &EnvironmentSnapshot,
) -> FeedDone {
    failed(format!(
        "{code}: {label}: {detail}; {recovery}",
        detail = environment.redact(&error.to_string()),
        code = error.code(),
        label = stage.label(),
        recovery = error.recovery()
    ))
}

fn timed_out() -> FeedDone {
    FeedDone {
        session: None,
        result: FeedResult::Outcome(RunOutcome::TimedOut),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn snippet_inputs_derive_only_configured_names() {
        let registry = crate::MontyHostRegistry::new()
            .register("jev_call", |_: &crate::MontyHostCall| {
                Ok(serde_json::Value::Null)
            })
            .expect("registry");
        let inputs = super::snippet_inputs(&serde_json::json!({}), &registry);
        let names: Vec<_> = inputs.iter().map(|(name, _)| name).collect();
        assert_eq!(names, ["__hook", "jev_call"]);
    }
}
