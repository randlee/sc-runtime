//! One in-process Monty feed for a schedule name.
//!
//! The repl stays with the caller between feeds. This module never drops a
//! repl while `feed_run` or `feed_start` is on the stack. A feed-time-limit
//! error is the only Monty timeout. Time inside `render_template` is measured
//! with [`std::time::Instant`] and is not a feed-time-limit error. A source
//! that does not compile, or that lacks `cron_stop` or `cron_tick`, is
//! [`CronError::HookFailed`] and leaves no repl.
//!
//! Production feeds spawn `sc-compose` and register `render_template` and
//! `sc_compose_run`. Hook source cannot choose the program or another host
//! function.

use std::fmt;
use std::time::{Duration, Instant};

use monty::{MontyRepl, ReplProgress};
use monty_types::{
    CompileOptions, ExcType, ExtFunctionResult, MontyException, MontyObject, NamedValues,
    PrintWriter, ResourceLimits, ResourceTracker,
};
use serde_json::Value;

use crate::monty_host::{ComposeHostError, ComposeProgram};
use crate::{CronError, Generation, RunOutcome, StopDecision};

const STOP_FN: &str = "cron_stop";
const TICK_FN: &str = "cron_tick";
const DENY_RECOVERY: &str =
    "Fix hook source; only `render_template` and `sc_compose_run` are allowlisted.";
const CODE_HOST_DENIED: &str = "HOST_DENIED";
const CODE_HOST_UNDEFINED: &str = "HOST_UNDEFINED";
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

/// Why `render_template` arguments were rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderArgError {
    Arity,
    Template,
    Context,
}

impl fmt::Display for RenderArgError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Arity => "render_template expects a string template and one context value",
            Self::Template => "render_template template must be a string",
            Self::Context => "render_template context is not a JSON value",
        })
    }
}

/// Why `sc_compose_run` arguments were rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComposeRunArgError {
    Arity,
    Subcommand,
    Args,
}

impl fmt::Display for ComposeRunArgError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Arity => {
                "sc_compose_run expects a subcommand string and an optional list of argument strings"
            }
            Self::Subcommand => "sc_compose_run subcommand must be a string",
            Self::Args => "sc_compose_run args must be a list of strings",
        })
    }
}

/// Why a stop reply could not be applied.
#[derive(Clone, Debug, PartialEq, Eq)]
enum StopReplyError {
    NotObject,
    MissingAction,
    MissingReason { action: String },
    UnknownAction { action: String },
}

impl fmt::Display for StopReplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotObject => formatter.write_str("reply is not an object"),
            Self::MissingAction => formatter.write_str("reply has no string \"action\""),
            Self::MissingReason { action } => {
                write!(formatter, "action {action:?} needs a \"reason\"")
            }
            Self::UnknownAction { action } => write!(
                formatter,
                "unknown action {action:?}; expected continue, disable or delete"
            ),
        }
    }
}

/// Compose program and extra host names for one feed.
///
/// [`execute_feed`] does not take this value. [`crate::Scheduler`] cannot set
/// it. Tests pass it to [`execute_feed_with_compose_program`].
pub(crate) struct ComposeInjection<'a> {
    /// Program spawned for `render_template`.
    pub(crate) program: ComposeProgram<'a>,
    /// Host function names registered besides `render_template` (unit tests only).
    #[cfg(test)]
    pub(crate) extra_host_fns: &'a [&'a str],
}

impl ComposeInjection<'static> {
    /// Production injection: `sc-compose` and no extra host functions.
    #[must_use]
    pub(crate) const fn production() -> Self {
        Self {
            program: ComposeProgram::SC_COMPOSE,
            #[cfg(test)]
            extra_host_fns: &[],
        }
    }
}

/// Inputs for one blocking-pool feed shared by the worker and `run_due`.
pub(crate) struct FeedLaunch {
    /// Repl from the previous feed of this generation, if any.
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
        session,
        source,
        phase,
        generation,
        name,
        payload,
        timeout,
        deadline,
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
    let run_deadline = deadline;
    match tokio::task::spawn_blocking(move || {
        execute_feed(
            session,
            &source,
            phase,
            generation,
            left,
            &hook,
            run_deadline,
        )
    })
    .await
    {
        Ok(done) => done,
        Err(error) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
        Err(error) => FeedDone {
            session: None,
            result: FeedResult::Outcome(RunOutcome::Failed(format!(
                "{CODE_FEED_TASK}: monty feed task join failed: {error}; {FEED_TASK_RECOVERY}"
            ))),
        },
    }
}

/// Runs one Monty feed on the calling thread. Call this from the blocking pool.
///
/// The feed calls `cron_stop(__hook)` or `cron_tick(__hook)` through
/// `feed_start`. Host functions are `render_template` and `sc_compose_run`.
/// Any other host function or `OsCall` fails the feed and drops the repl.
/// Hook source does not choose the program.
pub(crate) fn execute_feed(
    session: Option<MontySession>,
    source: &str,
    phase: MontyPhase,
    generation: Generation,
    budget: Duration,
    hook: &Value,
    run_deadline: Instant,
) -> FeedDone {
    execute_feed_with_compose_program(
        session,
        source,
        phase,
        generation,
        budget,
        hook,
        ComposeInjection::production(),
        run_deadline,
    )
}

/// Runs one feed with a test-selected compose program and extra host names.
///
/// [`crate::Scheduler`] calls [`execute_feed`], which always passes
/// [`ComposeInjection::production`]. This function is crate-private so an
/// embedder cannot select the program. Extra host names are registered only
/// for this call; production registers `render_template` alone. Names that
/// are not identifiers, or that are `render_template` or `__hook`, are ignored.
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_feed_with_compose_program(
    mut session: Option<MontySession>,
    source: &str,
    phase: MontyPhase,
    generation: Generation,
    budget: Duration,
    hook: &Value,
    injection: ComposeInjection<'_>,
    run_deadline: Instant,
) -> FeedDone {
    if session
        .as_ref()
        .is_some_and(|current| current.generation != generation)
    {
        session = None;
    }
    let mut call_budget = budget;
    let mut session = match session {
        Some(session) => session,
        None => {
            let compile_started = Instant::now();
            let compiled = match compile(source, generation, feed_budget(budget)) {
                Ok(session) => session,
                Err(done) => return done,
            };
            call_budget = budget.saturating_sub(compile_started.elapsed());
            compiled
        }
    };
    session
        .repl
        .tracker_mut()
        .set_max_feed_duration(feed_budget(call_budget));
    let stage = FeedStage::from_phase(phase);
    let host_deadline = Some(run_deadline);
    let compose_program = injection.program;
    #[cfg(test)]
    let extra_host_fns = injection.extra_host_fns;
    let MontySession { repl, generation } = session;
    let repl = *repl;
    let mut progress = match repl.feed_start(
        phase_snippet(phase),
        snippet_inputs(
            hook,
            #[cfg(test)]
            extra_host_fns,
        ),
        PrintWriter::Disabled,
    ) {
        Ok(progress) => progress,
        Err(error) => return from_exception(error.error, stage),
    };
    loop {
        progress = match progress {
            ReplProgress::FunctionCall(call) if call.function_name == "render_template" => {
                match resume_render(call, stage, compose_program, host_deadline) {
                    Ok(next) => next,
                    Err(done) => return done,
                }
            }
            ReplProgress::FunctionCall(call) if call.function_name == "sc_compose_run" => {
                match resume_compose_run(call, stage, compose_program, host_deadline) {
                    Ok(next) => next,
                    Err(done) => return done,
                }
            }
            ReplProgress::Complete { repl, value } => {
                return finish_phase(phase, generation, repl, &value);
            }
            ReplProgress::FunctionCall(call) => {
                return denied(stage, &format!("host function {}", call.function_name));
            }
            ReplProgress::OsCall(call) => {
                let operation: &'static str = call.function_call.into();
                return denied_os(stage, operation);
            }
            ReplProgress::NameLookup(lookup) => return undefined_name(stage, &lookup.name),
            ReplProgress::ResolveFutures(_) => return denied(stage, "unresolved futures"),
        };
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
) -> Result<MontySession, FeedDone> {
    let limits = ResourceLimits::default().max_feed_duration(budget);
    let mut repl = MontyRepl::new(
        "sc-cron-hook",
        ResourceTracker::new(limits),
        CompileOptions::default(),
    );
    if let Err(error) = repl.feed_run(source, NamedValues::new(), PrintWriter::Disabled) {
        return Err(from_exception(error, FeedStage::Compile));
    }
    if !repl.has_function(STOP_FN) || !repl.has_function(TICK_FN) {
        return Err(failed_hook(format!(
            "source must define {STOP_FN} and {TICK_FN}"
        )));
    }
    Ok(MontySession {
        repl: Box::new(repl),
        generation,
    })
}

fn from_exception(error: MontyException, stage: FeedStage) -> FeedDone {
    if is_feed_time_limit(&error) {
        return timed_out();
    }
    if stage == FeedStage::Compile || error.exc_type() == ExcType::SyntaxError {
        return failed_hook(error.message().unwrap_or("source did not compile"));
    }
    let detail = error.message().unwrap_or("hook raised");
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
        _ => Err(StopReplyError::UnknownAction { action }),
    }
}

fn phase_snippet(phase: MontyPhase) -> &'static str {
    match phase {
        MontyPhase::Stop => "cron_stop(__hook)",
        MontyPhase::Tick => "cron_tick(__hook)",
    }
}

fn snippet_inputs(hook: &Value, #[cfg(test)] extra_host_fns: &[&str]) -> NamedValues {
    let mut inputs = NamedValues::new();
    inputs.push("__hook", crate::monty_host::json_to_monty(hook));
    inputs.push(
        "render_template",
        MontyObject::function("render_template", None),
    );
    inputs.push(
        "sc_compose_run",
        MontyObject::function("sc_compose_run", None),
    );
    #[cfg(test)]
    for name in extra_host_fns {
        if reserved_host_binding(name) || !crate::is_identifier(name) {
            continue;
        }
        inputs.push(
            (*name).to_owned(),
            MontyObject::function((*name).to_owned(), None),
        );
    }
    inputs
}

#[cfg_attr(not(test), allow(dead_code))]
fn reserved_host_binding(name: &str) -> bool {
    matches!(name, "render_template" | "sc_compose_run" | "__hook")
}

fn resume_compose_run(
    call: monty::ReplFunctionCall,
    stage: FeedStage,
    compose_program: ComposeProgram<'_>,
    host_deadline: Option<Instant>,
) -> Result<ReplProgress, FeedDone> {
    let (subcommand, args) = match compose_run_arguments(&call) {
        Ok(ready) => ready,
        Err(why) => return Err(denied(stage, &why.to_string())),
    };
    if crate::monty_host::host_budget_exhausted(host_deadline) {
        return Err(timed_out());
    }
    let result = match crate::monty_host::sc_compose_run(
        &subcommand,
        &args,
        compose_program,
        host_deadline,
    ) {
        Ok(result) => result,
        Err(error) => return Err(after_compose(&error, stage, host_deadline)),
    };
    if crate::monty_host::host_budget_exhausted(host_deadline) {
        return Err(timed_out());
    }
    let value = crate::monty_host::compose_run_to_monty(&result);
    match call.resume(ExtFunctionResult::Return(value), PrintWriter::Disabled) {
        Ok(next) => Ok(next),
        Err(error) => Err(from_exception(error.error, stage)),
    }
}

fn compose_run_arguments(
    call: &monty::ReplFunctionCall,
) -> Result<(String, Vec<String>), ComposeRunArgError> {
    let positional_len = call.args.args().len();
    if positional_len == 0 || positional_len > 2 || call.args.kwargs().len() != 0 {
        return Err(ComposeRunArgError::Arity);
    }
    let subcommand = call
        .args
        .arg(0)
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or(ComposeRunArgError::Subcommand)?;
    let args = if positional_len == 1 {
        Vec::new()
    } else {
        let tail = call.args.arg(1).ok_or(ComposeRunArgError::Arity)?;
        let json = crate::monty_host::monty_value_to_json(tail)
            .map_err(|_| ComposeRunArgError::Args)?;
        let Some(items) = json.as_array() else {
            return Err(ComposeRunArgError::Args);
        };
        let mut argv = Vec::with_capacity(items.len());
        for item in items {
            let Some(text) = item.as_str() else {
                return Err(ComposeRunArgError::Args);
            };
            argv.push(text.to_owned());
        }
        argv
    };
    Ok((subcommand, args))
}

fn resume_render(
    call: monty::ReplFunctionCall,
    stage: FeedStage,
    compose_program: ComposeProgram<'_>,
    host_deadline: Option<Instant>,
) -> Result<ReplProgress, FeedDone> {
    let (template, context) = match render_arguments(&call) {
        Ok(ready) => ready,
        Err(why) => return Err(denied(stage, &why.to_string())),
    };
    if crate::monty_host::host_budget_exhausted(host_deadline) {
        return Err(timed_out());
    }
    let rendered = match crate::monty_host::render_template(
        &template,
        &context,
        compose_program,
        host_deadline,
    ) {
        Ok(rendered) => rendered,
        Err(error) => return Err(after_compose(&error, stage, host_deadline)),
    };
    if crate::monty_host::host_budget_exhausted(host_deadline) {
        return Err(timed_out());
    }
    match call.resume(
        ExtFunctionResult::Return(MontyObject::string(rendered)),
        PrintWriter::Disabled,
    ) {
        Ok(next) => Ok(next),
        Err(error) => Err(from_exception(error.error, stage)),
    }
}

fn after_compose(
    error: &ComposeHostError,
    stage: FeedStage,
    host_deadline: Option<Instant>,
) -> FeedDone {
    if matches!(error, ComposeHostError::HostTimedOut { .. }) {
        return timed_out();
    }
    if error.child_exited() && crate::monty_host::host_budget_exhausted(host_deadline) {
        return timed_out();
    }
    compose_failed(stage, error)
}

fn render_arguments(call: &monty::ReplFunctionCall) -> Result<(String, Value), RenderArgError> {
    if call.args.args().len() != 2 || call.args.kwargs().len() != 0 {
        return Err(RenderArgError::Arity);
    }
    let template = call
        .args
        .arg(0)
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or(RenderArgError::Template)?;
    let context = call.args.arg(1).ok_or(RenderArgError::Arity)?;
    crate::monty_host::monty_value_to_json(context)
        .map(|context| (template, context))
        .map_err(|_| RenderArgError::Context)
}

fn finish_phase(
    phase: MontyPhase,
    generation: Generation,
    repl: MontyRepl,
    value: &MontyObject,
) -> FeedDone {
    match phase {
        MontyPhase::Stop => match parse_stop(value) {
            Ok(StopDecision::Continue) => kept(repl, generation, FeedResult::Continue),
            Ok(StopDecision::Disable(reason)) => {
                kept(repl, generation, FeedResult::Disable(reason))
            }
            Ok(StopDecision::Delete(reason)) => kept(repl, generation, FeedResult::Delete(reason)),
            Err(why) => {
                drop(repl);
                failed(format!(
                    "stop check: {CODE_HOST_REPLY}: {why}; {STOP_REPLY_RECOVERY}"
                ))
            }
        },
        MontyPhase::Tick => kept(repl, generation, FeedResult::Outcome(RunOutcome::Ok)),
    }
}

fn kept(repl: MontyRepl, generation: Generation, result: FeedResult) -> FeedDone {
    FeedDone {
        session: Some(MontySession {
            repl: Box::new(repl),
            generation,
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

fn denied_os(stage: FeedStage, operation: &str) -> FeedDone {
    failed(format!(
        "{CODE_HOST_OS}: {label}: os call {operation}; {DENY_RECOVERY}",
        label = stage.label()
    ))
}

fn compose_failed(stage: FeedStage, error: &ComposeHostError) -> FeedDone {
    failed(format!(
        "{code}: {label}: {error}; {recovery}",
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
    use std::time::{Duration, Instant};

    use super::{FeedResult, FeedStage, after_compose};
    use crate::RunOutcome;
    use crate::monty_host::ComposeHostError;

    #[test]
    fn a_missing_program_stays_failed_when_the_host_deadline_has_passed() {
        let error = ComposeHostError::ProgramNotFound {
            program: "sc-cron-missing-compose-b10".to_owned(),
        };
        let past = Instant::now()
            .checked_sub(Duration::from_secs(5))
            .expect("past");
        let done = after_compose(&error, FeedStage::Tick, Some(past));
        assert!(done.session.is_none());
        match done.result {
            FeedResult::Outcome(RunOutcome::Failed(detail)) => {
                assert!(detail.contains("COMPOSE_NOT_FOUND"), "{detail}");
                assert!(detail.contains("sc-cron-missing-compose-b10"), "{detail}");
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn host_timed_out_maps_to_timed_out() {
        let error = ComposeHostError::HostTimedOut {
            program: "sc-compose".to_owned(),
        };
        let done = after_compose(&error, FeedStage::Tick, None);
        assert!(matches!(
            done.result,
            FeedResult::Outcome(RunOutcome::TimedOut)
        ));
    }

    #[test]
    fn a_compose_error_after_the_deadline_is_timed_out() {
        let error = ComposeHostError::NonZeroExit {
            status: Some(1),
            stderr: "late".to_owned(),
        };
        let past = Instant::now()
            .checked_sub(Duration::from_secs(5))
            .expect("past");
        let done = after_compose(&error, FeedStage::Tick, Some(past));
        assert!(done.session.is_none());
        assert!(matches!(
            done.result,
            FeedResult::Outcome(RunOutcome::TimedOut)
        ));
    }

    #[test]
    fn pre_exit_compose_errors_stay_failed_when_the_host_deadline_has_passed() {
        let past = Instant::now()
            .checked_sub(Duration::from_secs(5))
            .expect("past");
        let stage = FeedStage::Tick;

        let staged = ComposeHostError::StageFailed {
            program: "sc-compose".to_owned(),
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        };
        let done = after_compose(&staged, stage, Some(past));
        assert!(done.session.is_none());
        match &done.result {
            FeedResult::Outcome(RunOutcome::Failed(detail)) => {
                assert!(detail.contains("COMPOSE_SPAWN"), "{detail}");
            }
            other => panic!("expected Failed for staging error, got {other:?}"),
        }

        let not_runnable = ComposeHostError::ProgramNotRunnable {
            program: "sc-compose".to_owned(),
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        };
        let done = after_compose(&not_runnable, stage, Some(past));
        assert!(done.session.is_none());
        match done.result {
            FeedResult::Outcome(RunOutcome::Failed(detail)) => {
                assert!(detail.contains("COMPOSE_NOT_RUNNABLE"), "{detail}");
            }
            other => panic!("expected Failed for not-runnable error, got {other:?}"),
        }
    }

    #[test]
    fn snippet_inputs_keeps_only_crate_identifiers() {
        let inputs = super::snippet_inputs(
            &serde_json::json!({}),
            &["compose_file", "a-b", "2bad", "café"],
        );
        let names: Vec<&str> = inputs.iter().map(|(name, _)| name).collect();
        assert!(names.contains(&"compose_file"));
        assert!(names.contains(&"render_template"));
        assert!(names.contains(&"sc_compose_run"));
        assert!(names.contains(&"__hook"));
        assert!(!names.contains(&"a-b"));
        assert!(!names.contains(&"2bad"));
        assert!(!names.contains(&"café"));
    }
}
