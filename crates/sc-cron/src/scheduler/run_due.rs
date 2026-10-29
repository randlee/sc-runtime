//! Persistence and recovery helpers for externally driven due runs.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use super::*;
use crate::CronError;
#[cfg(feature = "monty")]
use crate::Generation;
use crate::HookTarget;
use crate::clock::sleep_until_monotonic;
use crate::coord::NameInFlightGuard;
#[cfg(feature = "monty")]
use crate::coord::RunCoord;
use crate::worker::{resolve, run_target};

#[cfg(feature = "monty")]
const CODE_DUE_JOIN_FAILED: &str = "DUE_JOIN_FAILED";
#[cfg(feature = "monty")]
const DUE_JOIN_RECOVERY: &str =
    "Retry the schedule; if it persists, inspect the Monty due task and hook source.";
#[cfg(feature = "monty")]
const CODE_MONTY_RESOLVE: &str = "MONTY_RESOLVE";
#[cfg(feature = "monty")]
const MONTY_RESOLVE_RECOVERY: &str =
    "Run this row on its Rust or Python target, or set HookTarget::Monty.";
#[cfg(feature = "monty")]
const CODE_MONTY_TICK: &str = "MONTY_TICK";
#[cfg(feature = "monty")]
const MONTY_TICK_RECOVERY: &str =
    "Return normally from cron_tick; disable and delete belong in cron_stop.";

#[cfg(feature = "monty")]
fn monty_failed(
    code: &'static str,
    detail: impl std::fmt::Display,
    recovery: &'static str,
) -> RunOutcome {
    RunOutcome::Failed(format!("{code}: {detail}; {recovery}"))
}

const CODE_RUN_TASK: &str = "RUN_TASK_FAILED";
const RUN_TASK_RECOVERY: &str = "Retry the run; if it persists, inspect the job task.";

fn run_task_failed(error: &impl std::fmt::Display) -> RunOutcome {
    RunOutcome::Failed(format!(
        "{CODE_RUN_TASK}: job panicked or was cancelled: {error}; {RUN_TASK_RECOVERY}"
    ))
}

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

type Launched = (Schedule, Option<DateTime<Utc>>, DueTask);

pub(super) async fn drive<S, C, J>(
    scheduler: &Scheduler<S, C, J>,
    job: Arc<dyn Job>,
) -> Result<Vec<DueRun>, CronError>
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    let core = Arc::clone(&scheduler.core);
    let rows = super::blocking(move || core.store.list_active()).await?;
    let now = scheduler.core.clock.now();
    let mut runs = Vec::new();
    for row in rows {
        if let Some(launched) = launch_row(scheduler, &job, row, now).await {
            runs.push(launched);
        }
    }
    collect_runs(scheduler, runs).await
}

async fn launch_row<S, C, J>(
    scheduler: &Scheduler<S, C, J>,
    job: &Arc<dyn Job>,
    row: Schedule,
    now: DateTime<Utc>,
) -> Option<Launched>
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    if !row.enabled || row.next_run_at.is_none_or(|at| at.as_datetime() > now) {
        return None;
    }
    if consumed_last_occurrence(&row) {
        if !last_outcome_refused(&row) {
            end_recorded_final(scheduler, &row).await;
        }
        return None;
    }
    let monty = matches!(row.spec.hook, HookTarget::Monty { .. });
    if monty && scheduler.coord.name_in_flight(&row.spec.name) {
        return None;
    }
    let name_guard = if monty {
        match NameInFlightGuard::try_insert(&scheduler.coord.names, &row.spec.name) {
            Some(guard) => Some(guard),
            None => return None,
        }
    } else {
        None
    };
    let next = row.spec.expr.next_run(row.jitter, now);
    if !store_next_before_launch(scheduler, &row, next).await {
        return None;
    }
    let monotonic = scheduler.core.clock.monotonic_now();
    let deadline = monotonic
        .checked_add(row.spec.timeout.get())
        .unwrap_or(monotonic);
    let clock = Arc::clone(&scheduler.core.clock);
    tracing::info!(subsystem = "sc_cron", action = "launched", schedule = %row.id,
        name = %row.spec.name, "run started");
    #[cfg(feature = "monty")]
    if monty {
        return Some(spawn_monty_due(
            scheduler, row, next, deadline, clock, name_guard,
        ));
    }
    drop(name_guard);
    Some(spawn_abortable_due(job, row, next, deadline, clock))
}

async fn store_next_before_launch<S, C, J>(
    scheduler: &Scheduler<S, C, J>,
    row: &Schedule,
    next: Option<DateTime<Utc>>,
) -> bool
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    let Some(next_at) = next else {
        return true;
    };
    let core = Arc::clone(&scheduler.core);
    let (id, generation) = (row.id, row.generation);
    let stamp = Timestamp::from_datetime(next_at);
    match super::blocking(move || core.store.set_next(id, generation, stamp)).await {
        Ok(true) => true,
        Ok(false) => {
            tracing::warn!(subsystem = "sc_cron", action = "fenced",
                schedule = %row.id, name = %row.spec.name,
                "the schedule changed before launch; skipped");
            false
        }
        Err(error) => {
            tracing::error!(subsystem = "sc_cron", action = "set_next_failed",
                schedule = %row.id, code = error.code(), %error,
                "next run not stored: {error}");
            true
        }
    }
}

fn spawn_abortable_due<C: Clock + 'static>(
    job: &Arc<dyn Job>,
    row: Schedule,
    next: Option<DateTime<Utc>>,
    deadline: std::time::Instant,
    clock: Arc<C>,
) -> Launched {
    let target = resolve(job, &row);
    let schedule = row.clone();
    let handle = tokio::spawn(async move {
        let target = match target {
            Ok(target) => target,
            Err(outcome) => return Report::Ran(outcome),
        };
        let run = run_target(target, Arc::clone(&clock), &schedule, deadline);
        tokio::select! {
            report = run => report,
            () = sleep_until_monotonic(clock.as_ref(), deadline) => Report::Ran(RunOutcome::TimedOut),
        }
    });
    (row, next, DueTask::abortable(handle))
}

#[cfg(feature = "monty")]
fn spawn_monty_due<S, C, J>(
    scheduler: &Scheduler<S, C, J>,
    row: Schedule,
    next: Option<DateTime<Utc>>,
    deadline: std::time::Instant,
    clock: Arc<C>,
    name_guard: Option<NameInFlightGuard>,
) -> Launched
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    let coord = Arc::clone(&scheduler.coord);
    let core = Arc::clone(&scheduler.core);
    let compose_program = Arc::clone(&scheduler.compose_program);
    let schedule = row.clone();
    let feed_guard = crate::coord::MontyFeedGuard::enter(&scheduler.coord);
    let handle = tokio::spawn(async move {
        let _name = name_guard;
        let (report, feed_guard) = run_monty_due(
            coord,
            clock,
            schedule.clone(),
            deadline,
            feed_guard,
            compose_program,
        )
        .await;
        let recorded = record_due(core, &schedule, report, next).await;
        drop(feed_guard);
        recorded
    });
    (row, next, DueTask::monty(handle))
}

async fn collect_runs<S, C, J>(
    scheduler: &Scheduler<S, C, J>,
    runs: Vec<Launched>,
) -> Result<Vec<DueRun>, CronError>
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    let mut done = Vec::with_capacity(runs.len());
    for (row, next, mut task) in runs {
        match &mut task {
            DueTask::Abortable(slot) => {
                let Some(handle) = slot.as_mut() else {
                    return Err(CronError::HookFailed("due run has no task".to_owned()));
                };
                let report = handle
                    .await
                    .unwrap_or_else(|error| Report::Ran(run_task_failed(&error)));
                slot.take();
                done.push(scheduler.finish_due(&row, report, next).await);
            }
            #[cfg(feature = "monty")]
            DueTask::Monty(slot) => {
                let Some(handle) = slot.as_mut() else {
                    return Err(CronError::HookFailed("due run has no task".to_owned()));
                };
                let recorded = handle.await.unwrap_or_else(|error| DueRun {
                    name: row.spec.name.clone(),
                    outcome: monty_failed(
                        CODE_DUE_JOIN_FAILED,
                        format!("monty due task panicked or was cancelled: {error}"),
                        DUE_JOIN_RECOVERY,
                    ),
                });
                slot.take();
                done.push(recorded);
            }
        }
    }
    Ok(done)
}

/// One run [`Scheduler::run_due`] started.
///
/// Rust and Python abort when the future is dropped. A Monty feed is forgotten
/// so the interpreter runs to its next checkpoint and the spawned task records
/// the outcome.
enum DueTask {
    Abortable(Option<tokio::task::JoinHandle<Report>>),
    #[cfg(feature = "monty")]
    Monty(Option<tokio::task::JoinHandle<DueRun>>),
}

impl DueTask {
    fn abortable(handle: tokio::task::JoinHandle<Report>) -> Self {
        Self::Abortable(Some(handle))
    }

    #[cfg(feature = "monty")]
    fn monty(handle: tokio::task::JoinHandle<DueRun>) -> Self {
        Self::Monty(Some(handle))
    }
}

impl Drop for DueTask {
    fn drop(&mut self) {
        match self {
            Self::Abortable(handle) => {
                if let Some(handle) = handle.take() {
                    handle.abort();
                }
            }
            #[cfg(feature = "monty")]
            Self::Monty(handle) => {
                handle.take();
            }
        }
    }
}

#[cfg(feature = "monty")]
async fn run_monty_due<C: Clock + 'static>(
    coord: Arc<RunCoord>,
    clock: Arc<C>,
    schedule: Schedule,
    deadline: std::time::Instant,
    stop_guard: crate::coord::MontyFeedGuard,
    compose_program: Arc<std::path::PathBuf>,
) -> (Report, crate::coord::MontyFeedGuard) {
    use crate::monty_run::{FeedResult, MontyPhase};
    let HookTarget::Monty { source } = schedule.spec.hook.clone() else {
        return (
            Report::Ran(monty_failed(
                CODE_MONTY_RESOLVE,
                "monty path used for another hook",
                MONTY_RESOLVE_RECOVERY,
            )),
            stop_guard,
        );
    };
    let generation = schedule.generation;
    let session = coord.take_repl(&schedule.spec.name);
    let stop = feed_once(
        session,
        source.clone(),
        MontyPhase::Stop,
        generation,
        clock.as_ref(),
        &schedule,
        deadline,
        Arc::clone(&compose_program),
    )
    .await;
    match stop.result {
        FeedResult::Disable(reason) => {
            drop(stop.session);
            (Report::Disable(reason), stop_guard)
        }
        FeedResult::Delete(reason) => {
            drop(stop.session);
            (Report::Delete(reason), stop_guard)
        }
        FeedResult::Outcome(outcome) => {
            if matches!(outcome, RunOutcome::Ok)
                && let Some(session) = stop.session
            {
                coord.store_repl(schedule.spec.name.clone(), session);
            }
            (Report::Ran(outcome), stop_guard)
        }
        FeedResult::Continue => {
            run_monty_tick_after_stop(
                coord,
                clock,
                schedule,
                deadline,
                stop,
                source,
                compose_program,
            )
            .await
        }
    }
}

#[cfg(feature = "monty")]
async fn run_monty_tick_after_stop<C: Clock + 'static>(
    coord: Arc<RunCoord>,
    clock: Arc<C>,
    schedule: Schedule,
    deadline: std::time::Instant,
    stop: crate::monty_run::FeedDone,
    source: String,
    compose_program: Arc<std::path::PathBuf>,
) -> (Report, crate::coord::MontyFeedGuard) {
    use crate::monty_run::{FeedResult, MontyPhase};
    let tick_guard = crate::coord::MontyFeedGuard::enter(&coord);
    let tick = feed_once(
        stop.session,
        source,
        MontyPhase::Tick,
        schedule.generation,
        clock.as_ref(),
        &schedule,
        deadline,
        compose_program,
    )
    .await;
    let report = match tick.result {
        FeedResult::Outcome(outcome) => {
            if matches!(outcome, RunOutcome::Ok)
                && let Some(session) = tick.session
            {
                coord.store_repl(schedule.spec.name.clone(), session);
            }
            Report::Ran(outcome)
        }
        FeedResult::Disable(reason) => Report::Disable(reason),
        FeedResult::Delete(reason) => Report::Delete(reason),
        FeedResult::Continue => Report::Ran(monty_failed(
            CODE_MONTY_TICK,
            "tick returned a stop action",
            MONTY_TICK_RECOVERY,
        )),
    };
    (report, tick_guard)
}

#[cfg(feature = "monty")]
#[allow(clippy::too_many_arguments)]
async fn feed_once<C: Clock + ?Sized>(
    session: Option<crate::monty_run::MontySession>,
    source: String,
    phase: crate::monty_run::MontyPhase,
    generation: Generation,
    clock: &C,
    schedule: &Schedule,
    deadline: std::time::Instant,
    compose_program: Arc<std::path::PathBuf>,
) -> crate::monty_run::FeedDone {
    crate::monty_run::drive_feed(
        clock,
        crate::monty_run::FeedLaunch {
            session,
            source,
            phase,
            generation,
            name: schedule.spec.name.clone(),
            payload: schedule.spec.payload.clone(),
            timeout: schedule.spec.timeout,
            deadline,
            compose_program,
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::{CODE_RUN_TASK, RUN_TASK_RECOVERY, run_task_failed};
    use crate::RunOutcome;

    #[test]
    fn a_joined_task_failure_names_a_code_and_recovery() {
        let RunOutcome::Failed(detail) = run_task_failed(&"boom") else {
            panic!("expected Failed");
        };
        assert_eq!(
            detail,
            format!("{CODE_RUN_TASK}: job panicked or was cancelled: boom; {RUN_TASK_RECOVERY}")
        );
    }

    #[cfg(feature = "monty")]
    #[test]
    fn monty_due_failures_name_a_code_and_recovery() {
        use super::{
            CODE_DUE_JOIN_FAILED, CODE_MONTY_RESOLVE, CODE_MONTY_TICK, DUE_JOIN_RECOVERY,
            MONTY_RESOLVE_RECOVERY, MONTY_TICK_RECOVERY, monty_failed,
        };

        let tick = monty_failed(
            CODE_MONTY_TICK,
            "tick returned a stop action",
            MONTY_TICK_RECOVERY,
        );
        let RunOutcome::Failed(detail) = tick else {
            panic!("expected Failed");
        };
        assert_eq!(
            detail,
            format!("{CODE_MONTY_TICK}: tick returned a stop action; {MONTY_TICK_RECOVERY}")
        );

        let joined = monty_failed(
            CODE_DUE_JOIN_FAILED,
            "monty due task panicked or was cancelled: boom",
            DUE_JOIN_RECOVERY,
        );
        let RunOutcome::Failed(detail) = joined else {
            panic!("expected Failed");
        };
        assert!(
            detail.starts_with(&format!("{CODE_DUE_JOIN_FAILED}: ")),
            "{detail}"
        );
        assert!(detail.ends_with(DUE_JOIN_RECOVERY), "{detail}");

        let resolved = monty_failed(
            CODE_MONTY_RESOLVE,
            "monty path used for another hook",
            MONTY_RESOLVE_RECOVERY,
        );
        let RunOutcome::Failed(detail) = resolved else {
            panic!("expected Failed");
        };
        assert!(
            detail.starts_with(&format!("{CODE_MONTY_RESOLVE}: ")),
            "{detail}"
        );
        assert!(detail.ends_with(MONTY_RESOLVE_RECOVERY), "{detail}");
    }
}
