//! One in-process Monty feed for a schedule name.
//!
//! The repl stays with the caller between feeds. This module never drops a
//! repl while `feed_run` or `call_function` is on the stack. A feed-time-limit
//! error is the only timeout; other exceptions fail the run. A source that
//! does not compile, or that lacks `cron_stop` or `cron_tick`, is
//! [`CronError::HookFailed`] and leaves no repl.
//!
//! Host extension functions come from [`crate::monty_host`]. That map is empty
//! until the b-10 bridge registers `render_template`.

use std::time::Duration;

use monty::MontyRepl;
use monty_types::{
    CallArgs, CompileOptions, ExcType, MontyException, MontyObject, PrintWriter, ResourceLimits,
    ResourceTracker,
};
use serde_json::Value;

use crate::{CronError, Generation, RunOutcome, StopDecision};

const STOP_FN: &str = "cron_stop";
const TICK_FN: &str = "cron_tick";

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
    let mut args = CallArgs::new();
    args.push_arg(json_to_monty(hook));
    let function = match phase {
        MontyPhase::Stop => STOP_FN,
        MontyPhase::Tick => TICK_FN,
    };
    let called = session
        .repl
        .call_function(function, args, PrintWriter::Disabled);
    match phase {
        MontyPhase::Stop => match called {
            Ok(value) => match parse_stop(&value) {
                Ok(StopDecision::Continue) => FeedDone {
                    session: Some(session),
                    result: FeedResult::Continue,
                },
                Ok(StopDecision::Disable(reason)) => FeedDone {
                    session: Some(session),
                    result: FeedResult::Disable(reason),
                },
                Ok(StopDecision::Delete(reason)) => FeedDone {
                    session: Some(session),
                    result: FeedResult::Delete(reason),
                },
                Err(why) => failed(format!("stop check: {why}")),
            },
            Err(error) => from_exception(error, "stop check"),
        },
        MontyPhase::Tick => match called {
            Ok(_) => FeedDone {
                session: Some(session),
                result: FeedResult::Outcome(RunOutcome::Ok),
            },
            Err(error) => from_exception(error, "tick"),
        },
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
    tracing::trace!(
        planned = ?crate::monty_host::planned_allowlist(),
        "monty host extensions are not registered yet"
    );
    if let Err(error) = repl.feed_run(
        source,
        crate::monty_host::extensions(),
        PrintWriter::Disabled,
    ) {
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

fn json_to_monty(value: &Value) -> MontyObject {
    match value {
        Value::Null => MontyObject::none(),
        Value::Bool(item) => MontyObject::bool(*item),
        Value::Number(number) => number.as_i64().map_or_else(
            || MontyObject::float(number.as_f64().unwrap_or(0.0)),
            MontyObject::int,
        ),
        Value::String(text) => MontyObject::string(text.clone()),
        Value::Array(items) => MontyObject::list(items.iter().map(json_to_monty)),
        Value::Object(map) => MontyObject::dict(
            map.iter()
                .map(|(key, item)| (MontyObject::string(key.clone()), json_to_monty(item))),
        ),
    }
}
