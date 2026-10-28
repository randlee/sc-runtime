//! One in-process Monty feed for a schedule name.
//!
//! The repl stays with the caller between feeds. This module never drops a
//! repl while `feed_run` or `feed_start` is on the stack. A feed-time-limit
//! error is the only Monty timeout. Time inside `render_template` is measured
//! with [`std::time::Instant`] and is not a feed-time-limit error. A source
//! that does not compile, or that lacks `cron_stop` or `cron_tick`, is
//! [`CronError::HookFailed`] and leaves no repl.

use std::time::{Duration, Instant};

use monty::{MontyRepl, ReplProgress};
use monty_types::{
    CompileOptions, ExcType, ExtFunctionResult, MontyException, MontyObject, NamedValues,
    PrintWriter, ResourceLimits, ResourceTracker,
};
use serde_json::Value;

use crate::{CronError, Generation, RunOutcome, StopDecision};

const STOP_FN: &str = "cron_stop";
const TICK_FN: &str = "cron_tick";
const DENY_RECOVERY: &str = "Fix hook source; only `render_template` is allowlisted.";
#[cfg(feature = "test-utils")]
const TEST_COMPOSE_PROGRAM: &str = "# sc-cron-test-compose-program:";
#[cfg(feature = "test-utils")]
const TEST_HOST_FN: &str = "# sc-cron-test-host-fn:";

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

/// Runs one Monty feed on the calling thread. Call this from the blocking pool.
///
/// The feed calls `cron_stop(__hook)` or `cron_tick(__hook)` through
/// `feed_start`. A hook may call host function `render_template`; any other
/// host function or `OsCall` fails the feed and drops the repl.
pub(crate) fn execute_feed(
    mut session: Option<MontySession>,
    source: &str,
    phase: MontyPhase,
    generation: Generation,
    budget: Duration,
    hook: &Value,
) -> FeedDone {
    if cfg!(feature = "test-utils") && source.contains("SC_CRON_TEST_PANIC") {
        panic!("monty feed");
    }
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
            let compile_started = std::time::Instant::now();
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
    let label = phase_label(phase);
    let compose_program = compose_program_name(source);
    let host_deadline = Instant::now().checked_add(call_budget);
    let MontySession { repl, generation } = session;
    let repl = *repl;
    let mut progress = match repl.feed_start(
        phase_snippet(phase),
        snippet_inputs(source, hook),
        PrintWriter::Disabled,
    ) {
        Ok(progress) => progress,
        Err(error) => return from_exception(error.error, label),
    };
    loop {
        progress = match progress {
            ReplProgress::FunctionCall(call) if call.function_name == "render_template" => {
                match resume_render(call, label, &compose_program, host_deadline) {
                    Ok(next) => next,
                    Err(done) => return done,
                }
            }
            ReplProgress::Complete { repl, value } => {
                return finish_phase(phase, generation, repl, &value);
            }
            ReplProgress::FunctionCall(call) => {
                return denied(label, &format!("host function {}", call.function_name));
            }
            ReplProgress::OsCall(_) => return denied(label, "os call"),
            ReplProgress::NameLookup(_) | ReplProgress::ResolveFutures(_) => {
                return denied(label, "host call");
            }
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
        return Err(from_exception(error, "compile"));
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

fn from_exception(error: MontyException, what: &str) -> FeedDone {
    if is_feed_time_limit(&error) {
        return FeedDone {
            session: None,
            result: FeedResult::Outcome(RunOutcome::TimedOut),
        };
    }
    if what == "compile" || error.exc_type() == ExcType::SyntaxError {
        return failed_hook(error.message().unwrap_or("source did not compile"));
    }
    let detail = error.message().unwrap_or("hook raised");
    failed(format!("{what}: {detail}"))
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

fn failed_hook(detail: impl std::fmt::Display) -> FeedDone {
    FeedDone {
        session: None,
        result: FeedResult::Outcome(RunOutcome::Failed(
            CronError::HookFailed(detail.to_string()).recorded(),
        )),
    }
}

fn parse_stop(value: &MontyObject) -> Result<StopDecision, String> {
    let pairs = value
        .as_ref()
        .pairs()
        .ok_or_else(|| "reply is not an object".to_owned())?;
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
    let action = action.ok_or_else(|| "reply has no string \"action\"".to_owned())?;
    let needs_reason = || {
        reason
            .clone()
            .filter(|reason| !reason.trim().is_empty())
            .ok_or_else(|| format!("action {action:?} needs a \"reason\""))
    };
    match action.as_str() {
        "continue" => Ok(StopDecision::Continue),
        "disable" => needs_reason().map(StopDecision::Disable),
        "delete" => needs_reason().map(StopDecision::Delete),
        other => Err(format!(
            "unknown action {other:?}; expected continue, disable or delete"
        )),
    }
}

fn phase_label(phase: MontyPhase) -> &'static str {
    match phase {
        MontyPhase::Stop => "stop check",
        MontyPhase::Tick => "tick",
    }
}

fn phase_snippet(phase: MontyPhase) -> &'static str {
    match phase {
        MontyPhase::Stop => "cron_stop(__hook)",
        MontyPhase::Tick => "cron_tick(__hook)",
    }
}

fn snippet_inputs(source: &str, hook: &Value) -> NamedValues {
    let mut inputs = NamedValues::new();
    inputs.push("__hook", crate::monty_host::json_to_monty(hook));
    inputs.push(
        "render_template",
        MontyObject::function("render_template", None),
    );
    #[cfg(feature = "test-utils")]
    bind_test_host_fn(source, &mut inputs);
    #[cfg(not(feature = "test-utils"))]
    let _ = source;
    inputs
}

fn compose_program_name(source: &str) -> String {
    #[cfg(feature = "test-utils")]
    if let Some(name) = test_directive(source, TEST_COMPOSE_PROGRAM) {
        return name;
    }
    #[cfg(not(feature = "test-utils"))]
    let _ = source;
    "sc-compose".to_owned()
}

#[cfg(feature = "test-utils")]
fn bind_test_host_fn(source: &str, inputs: &mut NamedValues) {
    let Some(name) = test_directive(source, TEST_HOST_FN) else {
        return;
    };
    if name == "render_template" || name == "__hook" || !monty_types::is_identifier(&name) {
        return;
    }
    inputs.push(name.clone(), MontyObject::function(name, None));
}

#[cfg(feature = "test-utils")]
fn test_directive(source: &str, prefix: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let value = line.trim().strip_prefix(prefix)?.trim();
        if value.is_empty() || value.split_whitespace().nth(1).is_some() {
            None
        } else {
            Some(value.to_owned())
        }
    })
}

fn resume_render(
    call: monty::ReplFunctionCall,
    label: &str,
    compose_program: &str,
    host_deadline: Option<Instant>,
) -> Result<ReplProgress, FeedDone> {
    let (template, context) = match render_arguments(&call) {
        Ok(ready) => ready,
        Err(why) => return Err(denied(label, why)),
    };
    let rendered = match crate::monty_host::render_template(&template, &context, compose_program) {
        Ok(rendered) => rendered,
        Err(error) => {
            let missing = matches!(
                error,
                crate::monty_host::ComposeHostError::ProgramNotFound { .. }
            );
            if !missing && crate::monty_host::host_budget_exhausted(host_deadline) {
                return Err(timed_out());
            }
            return Err(compose_failed(label, &error));
        }
    };
    if crate::monty_host::host_budget_exhausted(host_deadline) {
        return Err(timed_out());
    }
    match call.resume(
        ExtFunctionResult::Return(MontyObject::string(rendered)),
        PrintWriter::Disabled,
    ) {
        Ok(next) => Ok(next),
        Err(error) => Err(from_exception(error.error, label)),
    }
}

fn render_arguments(call: &monty::ReplFunctionCall) -> Result<(String, Value), &'static str> {
    if call.args.args().len() != 2 || call.args.kwargs().len() != 0 {
        return Err("render_template expects a string template and one context value");
    }
    let template = call
        .args
        .arg(0)
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or("render_template template must be a string")?;
    let context = call
        .args
        .arg(1)
        .ok_or("render_template expects a string template and one context value")?;
    crate::monty_host::monty_value_to_json(context)
        .map(|context| (template, context))
        .map_err(|_| "render_template context is not a JSON value")
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
                failed(format!("stop check: {why}"))
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

fn denied(label: &str, why: &str) -> FeedDone {
    failed(format!("{label}: {why}; {DENY_RECOVERY}"))
}

fn compose_failed(label: &str, error: &crate::monty_host::ComposeHostError) -> FeedDone {
    failed(format!("{label}: {error}; {}", error.recovery()))
}

fn timed_out() -> FeedDone {
    FeedDone {
        session: None,
        result: FeedResult::Outcome(RunOutcome::TimedOut),
    }
}
