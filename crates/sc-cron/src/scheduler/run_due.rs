//! Persistence and recovery helpers for externally driven due runs.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use super::*;
use crate::CronError;
#[cfg(feature = "monty")]
use crate::Generation;
use crate::HookTarget;
use crate::clock::sleep_until_monotonic;
#[cfg(feature = "monty")]
use crate::clock::wall_after;
use crate::coord::NameInFlightGuard;
#[cfg(feature = "monty")]
use crate::coord::RunCoord;
#[cfg(feature = "monty")]
use crate::hook::{HookCall, HookEvent, request};
use crate::worker::{resolve, run_target};

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
    let schedule = row.clone();
    let feed_guard = crate::coord::MontyFeedGuard::enter(&scheduler.coord.feeds);
    let handle = tokio::spawn(async move {
        let _name = name_guard;
        let (report, feed_guard) =
            run_monty_due(coord, clock, schedule.clone(), deadline, feed_guard).await;
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
                let report = handle.await.unwrap_or_else(|error| {
                    Report::Ran(RunOutcome::Failed(format!(
                        "job panicked or was cancelled: {error}"
                    )))
                });
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
                    outcome: RunOutcome::Failed(format!("job panicked or was cancelled: {error}")),
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
) -> (Report, crate::coord::MontyFeedGuard) {
    use crate::monty_run::{FeedResult, MontyPhase};
    let HookTarget::Monty { source } = schedule.spec.hook.clone() else {
        return (
            Report::Ran(RunOutcome::Failed(
                "resolve: monty path used for another hook".into(),
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
            run_monty_tick_after_stop(coord, clock, schedule, deadline, stop, source).await
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
) -> (Report, crate::coord::MontyFeedGuard) {
    use crate::monty_run::{FeedResult, MontyPhase};
    let tick_guard = crate::coord::MontyFeedGuard::enter(&coord.feeds);
    let tick = feed_once(
        stop.session,
        source,
        MontyPhase::Tick,
        schedule.generation,
        clock.as_ref(),
        &schedule,
        deadline,
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
        FeedResult::Continue => {
            Report::Ran(RunOutcome::Failed("tick returned a stop action".into()))
        }
    };
    (report, tick_guard)
}

#[cfg(feature = "monty")]
async fn feed_once<C: Clock + ?Sized>(
    session: Option<crate::monty_run::MontySession>,
    source: String,
    phase: crate::monty_run::MontyPhase,
    generation: Generation,
    clock: &C,
    schedule: &Schedule,
    deadline: std::time::Instant,
) -> crate::monty_run::FeedDone {
    let left = deadline.saturating_duration_since(clock.monotonic_now());
    let stamp = Timestamp::from_datetime(wall_after(clock.now(), left));
    let event = match phase {
        crate::monty_run::MontyPhase::Stop => HookEvent::Stop,
        crate::monty_run::MontyPhase::Tick => HookEvent::Tick,
    };
    let call = HookCall {
        event,
        schedule: &schedule.spec.name,
        payload: &schedule.spec.payload,
        timeout: schedule.spec.timeout,
        deadline,
    };
    let hook = request(&call, stamp);
    match tokio::task::spawn_blocking(move || {
        crate::monty_run::execute_feed(session, &source, phase, generation, left, &hook)
    })
    .await
    {
        Ok(done) => done,
        Err(error) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
        Err(error) => crate::monty_run::FeedDone {
            session: None,
            result: crate::monty_run::FeedResult::Outcome(RunOutcome::Failed(format!(
                "feed task failed: {error}"
            ))),
        },
    }
}
