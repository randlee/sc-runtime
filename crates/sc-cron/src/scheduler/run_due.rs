//! Persistence and recovery helpers for externally driven due runs.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use super::*;

pub(super) async fn record_due<S, C, J>(
    core: Arc<Core<S, C, J>>,
    row: &Schedule,
    report: Report,
    next: Option<DateTime<Utc>>,
) -> DueRun
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    let (id, generation) = (row.id, row.generation);
    let at = Timestamp::from_datetime(core.clock.now());
    let outcome = match &report {
        Report::Ran(outcome) => outcome.clone(),
        Report::Disable(reason) | Report::Delete(reason) => RunOutcome::Refused(reason.clone()),
    };
    let recorded = outcome.clone();
    let result = blocking(move || match report {
        Report::Disable(reason) => core
            .store
            .disable(id, generation, at, &reason)
            .map(|wrote| {
                if wrote {
                    Finish::Recorded
                } else {
                    Finish::Fenced
                }
            }),
        Report::Delete(reason) => {
            core.signal_reload();
            core.store.stop(id, generation, at, &reason).map(|wrote| {
                if wrote {
                    Finish::Recorded
                } else {
                    Finish::Fenced
                }
            })
        }
        Report::Ran(outcome) => {
            if !core.store.record_run(id, generation, at, &outcome)? {
                return Ok(Finish::Fenced);
            }
            let follow_up = match next {
                Some(next) => core
                    .store
                    .set_next(id, generation, Timestamp::from_datetime(next)),
                None => core.store.stop(id, generation, at, EXHAUSTED),
            };
            Ok(follow_up_finish(follow_up))
        }
    })
    .await;
    match result {
        Ok(Finish::Recorded) => tracing::info!(subsystem = "sc_cron", action = "recorded",
                schedule = %id, name = %row.spec.name, outcome = %recorded, "run recorded"),
        Ok(Finish::Fenced) => tracing::warn!(subsystem = "sc_cron", action = "fenced",
                schedule = %id, name = %row.spec.name,
                "the schedule changed during its run; nothing recorded"),
        Ok(Finish::Partial) => tracing::warn!(subsystem = "sc_cron", action = "partial_write",
                schedule = %id, name = %row.spec.name, outcome = %recorded,
                "run recorded; a follow-up write was refused"),
        Ok(Finish::FollowUp(error)) => tracing::error!(
                subsystem = "sc_cron", action = "partial_write",
                schedule = %id, name = %row.spec.name, outcome = %recorded,
                code = error.code(), %error,
                "run recorded; a follow-up write failed: {error}"),
        Err(error) => tracing::error!(subsystem = "sc_cron", action = "record_failed",
                schedule = %id, code = error.code(), %error, "run outcome not recorded: {error}"),
    }
    DueRun {
        name: row.spec.name.clone(),
        outcome: recorded,
    }
}

/// The stored next run is an occurrence already recorded, and the expression
/// has nothing after that record.
pub(super) fn consumed_last_occurrence(row: &Schedule) -> bool {
    match (row.next_run_at, row.last_run_at) {
        (Some(next), Some(last)) if next.as_datetime() <= last.as_datetime() => row
            .spec
            .expr
            .next_run(row.jitter, last.as_datetime())
            .is_none(),
        _ => false,
    }
}

/// A stop-check disable records [`RunOutcome::Refused`]. Re-enabling that row
/// must not run the occurrence again and must not end it.
pub(super) fn last_outcome_refused(row: &Schedule) -> bool {
    matches!(row.last_outcome, Some(RunOutcome::Refused(_)))
}

/// Ends a final run whose record committed and whose `stop` did not.
pub(super) async fn end_recorded_final<S, C, J>(scheduler: &Scheduler<S, C, J>, row: &Schedule)
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    let (id, generation) = (row.id, row.generation);
    let at = Timestamp::from_datetime(scheduler.core.clock.now());
    let core = Arc::clone(&scheduler.core);
    match blocking(move || core.store.stop(id, generation, at, EXHAUSTED)).await {
        Ok(true) => tracing::info!(subsystem = "sc_cron", action = "stopped",
            schedule = %id, name = %row.spec.name,
            "schedule ended; its last occurrence was already recorded"),
        Ok(false) => tracing::warn!(subsystem = "sc_cron", action = "fenced",
            schedule = %id, name = %row.spec.name,
            "the schedule changed before it could be ended"),
        Err(error) => tracing::error!(subsystem = "sc_cron", action = "stop_failed",
            schedule = %id, code = error.code(), %error, "schedule not ended: {error}"),
    }
}

/// How [`Scheduler::finish_due`] left the row.
enum Finish {
    Recorded,
    Fenced,
    /// The outcome was written; a later `set_next` or `stop` was refused.
    Partial,
    /// The outcome was written; a later `set_next` or `stop` failed.
    FollowUp(CronError),
}

/// Classifies a follow-up `set_next` or `stop` after `record_run` committed.
fn follow_up_finish(result: Result<bool, CronError>) -> Finish {
    match result {
        Ok(true) => Finish::Recorded,
        Ok(false) => Finish::Partial,
        Err(error) => Finish::FollowUp(error),
    }
}
