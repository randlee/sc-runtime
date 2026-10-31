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
use crate::worker::{
    Event, RunCorrelation, event_record, resolve, run_correlation, run_target_with_environment,
    terminal_event,
};

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
    logger: sc_command::observability::SharedCanonicalLog,
    row: &Schedule,
    report: Report,
    next: Option<DateTime<Utc>>,
    correlation: RunCorrelation,
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
    let operation = match &report {
        Report::Disable(_) => "disable",
        Report::Delete(_) => "stop",
        Report::Ran(_) => "record_run",
    };
    let result = blocking(move || persist_report(core, id, generation, at, report, next)).await;
    for event in recording_events(
        logger.service_name().clone(),
        row,
        &recorded,
        result,
        operation,
        correlation,
    ) {
        let _ = logger.emit(event);
    }
    DueRun {
        name: row.spec.name.clone(),
        outcome: recorded,
    }
}

fn persist_report<S: ScheduleStore, C: Clock, J: JitterSource>(
    core: Arc<Core<S, C, J>>,
    id: crate::ScheduleId,
    generation: crate::Generation,
    at: Timestamp,
    report: Report,
    next: Option<DateTime<Utc>>,
) -> Result<Finish, CronError> {
    let write = |wrote| {
        if wrote {
            Finish::Recorded
        } else {
            Finish::Fenced
        }
    };
    match report {
        Report::Disable(reason) => core.store.disable(id, generation, at, &reason).map(write),
        Report::Delete(reason) => {
            core.signal_reload();
            core.store.stop(id, generation, at, &reason).map(write)
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
    }
}

/// Pure event construction preserves terminal-before-follow-up ordering.
fn recording_events(
    service: sc_observability_types::ServiceName,
    row: &Schedule,
    recorded: &RunOutcome,
    result: Result<Finish, CronError>,
    operation: &str,
    correlation: RunCorrelation,
) -> Vec<sc_observability_types::LogEvent> {
    let terminal = matches!(
        result,
        Ok(Finish::Recorded | Finish::Partial | Finish::FollowUp(_))
    );
    let follow_up = match result {
        Ok(Finish::Recorded) => None,
        Ok(Finish::Fenced) => Some((
            Event::DueFenced,
            serde_json::json!({"operation": operation}),
        )),
        Ok(Finish::Partial) => Some((
            Event::DuePartialWrite,
            serde_json::json!({"outcome": recorded.to_string()}),
        )),
        Ok(Finish::FollowUp(error)) => Some((
            Event::DueFollowUpFailed,
            serde_json::json!({"outcome": recorded.to_string(), "error": error.to_string()}),
        )),
        Err(error) => Some((
            Event::DueRecordFailed,
            serde_json::json!({"error": error.to_string()}),
        )),
    };
    let mut events = Vec::with_capacity(2);
    if terminal
        && let Some(event) = terminal_event(service.clone(), Event::DueFinished, row, recorded)
    {
        events.push(event);
    }
    if let Some((kind, fields)) = follow_up
        && let Some(event) = event_record(service, kind, Some(row), fields)
    {
        events.push(event);
    }
    for event in &mut events {
        event.correlation_id = correlation.clone();
    }
    events
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
        Ok(true) => {
            if let Some(event) = event_record(
                scheduler.coord.logger.service_name().clone(),
                Event::DueStopped,
                Some(row),
                serde_json::json!({}),
            ) {
                let _ = scheduler.coord.logger.emit(event);
            }
        }
        Ok(false) => {
            if let Some(event) = event_record(
                scheduler.coord.logger.service_name().clone(),
                Event::DueFenced,
                Some(row),
                serde_json::json!({"operation": "stop"}),
            ) {
                let _ = scheduler.coord.logger.emit(event);
            }
        }
        Err(error) => {
            if let Some(event) = event_record(
                scheduler.coord.logger.service_name().clone(),
                Event::DueStopFailed,
                Some(row),
                serde_json::json!({"error": error.to_string()}),
            ) {
                let _ = scheduler.coord.logger.emit(event);
            }
        }
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

type Launched = (Schedule, Option<DateTime<Utc>>, DueTask, RunCorrelation);

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
    let correlation = run_correlation(&row, row.next_run_at.map(|at| at.as_datetime()));
    let next = row.spec.expr.next_run(row.jitter, now);
    if !store_next_before_launch(scheduler, &row, next).await {
        return None;
    }
    let monotonic = scheduler.core.clock.monotonic_now();
    let deadline = monotonic
        .checked_add(row.spec.timeout.get())
        .unwrap_or(monotonic);
    let clock = Arc::clone(&scheduler.core.clock);
    if let Some(mut event) = event_record(
        scheduler.coord.logger.service_name().clone(),
        Event::DueLaunched,
        Some(&row),
        serde_json::json!({"deadline_remaining_ms": deadline.saturating_duration_since(scheduler.core.clock.monotonic_now()).as_millis() as u64}),
    ) {
        event.correlation_id = correlation.clone();
        let _ = scheduler.coord.logger.emit(event);
    };
    #[cfg(feature = "monty")]
    if monty {
        return Some(spawn_monty_due(
            scheduler,
            row,
            next,
            deadline,
            clock,
            name_guard,
            correlation,
        ));
    }
    drop(name_guard);
    Some(spawn_abortable_due(
        job,
        row,
        next,
        deadline,
        clock,
        Arc::clone(&scheduler.coord.logger),
        (Arc::clone(&scheduler.environment), correlation),
    ))
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
            if let Some(event) = event_record(
                scheduler.coord.logger.service_name().clone(),
                Event::DueFenced,
                Some(row),
                serde_json::json!({"operation": "set_next"}),
            ) {
                let _ = scheduler.coord.logger.emit(event);
            };
            false
        }
        Err(error) => {
            if let Some(event) = event_record(
                scheduler.coord.logger.service_name().clone(),
                Event::DueNextFailed,
                Some(row),
                serde_json::json!({"error": error.to_string()}),
            ) {
                let _ = scheduler.coord.logger.emit(event);
            };
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
    logger: sc_command::observability::SharedCanonicalLog,
    execution: (Arc<crate::HookEnvironmentHost>, RunCorrelation),
) -> Launched {
    let (environment, correlation) = execution;
    let target = resolve(job, &row, &logger, correlation.clone());
    let schedule = row.clone();
    let handle = tokio::spawn(async move {
        let target = match target {
            Ok(target) => target,
            Err(outcome) => return Report::Ran(outcome),
        };
        let run = run_target_with_environment(
            target,
            Arc::clone(&clock),
            &schedule,
            deadline,
            &environment,
        );
        tokio::select! {
            report = run => report,
            () = sleep_until_monotonic(clock.as_ref(), deadline) => Report::Ran(RunOutcome::TimedOut),
        }
    });
    (row, next, DueTask::abortable(handle), correlation)
}

#[cfg(feature = "monty")]
fn spawn_monty_due<S, C, J>(
    scheduler: &Scheduler<S, C, J>,
    row: Schedule,
    next: Option<DateTime<Utc>>,
    deadline: std::time::Instant,
    clock: Arc<C>,
    name_guard: Option<NameInFlightGuard>,
    correlation: RunCorrelation,
) -> Launched
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    let coord = Arc::clone(&scheduler.coord);
    let core = Arc::clone(&scheduler.core);
    let hosts = Arc::clone(&scheduler.hosts);
    let environment_host = Arc::clone(&scheduler.environment);
    let schedule = row.clone();
    let feed_guard = crate::coord::MontyFeedGuard::enter(&scheduler.coord);
    let logger = Arc::clone(&coord.logger);
    let run_correlation = correlation.clone();
    let handle = tokio::spawn(async move {
        let _name = name_guard;
        let (report, feed_guard) = run_monty_due(
            coord,
            clock,
            schedule.clone(),
            deadline,
            feed_guard,
            hosts,
            (environment_host, run_correlation.clone()),
        )
        .await;
        let recorded = record_due(core, logger, &schedule, report, next, run_correlation).await;
        drop(feed_guard);
        recorded
    });
    (row, next, DueTask::monty(handle), correlation)
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
    for (row, next, mut task, correlation) in runs {
        match &mut task {
            DueTask::Abortable(slot) => {
                let Some(handle) = slot.as_mut() else {
                    return Err(CronError::HookFailed("due run has no task".to_owned()));
                };
                let report = handle.await.unwrap_or_else(|error| {
                    let detail = if matches!(row.spec.hook, HookTarget::Rust) {
                        error.to_string()
                    } else {
                        "hook or environment provider task panicked or was cancelled".to_owned()
                    };
                    if let Some(mut event) = event_record(
                        scheduler.coord.logger.service_name().clone(),
                        Event::DueTaskFailed,
                        Some(&row),
                        serde_json::json!({"error": detail}),
                    ) {
                        event.correlation_id = correlation.clone();
                        let _ = scheduler.coord.logger.emit(event);
                    };
                    Report::Ran(run_task_failed(&detail))
                });
                slot.take();
                done.push(
                    scheduler
                        .finish_due(&row, report, next, correlation.clone())
                        .await,
                );
            }
            #[cfg(feature = "monty")]
            DueTask::Monty(slot) => {
                let Some(handle) = slot.as_mut() else {
                    return Err(CronError::HookFailed("due run has no task".to_owned()));
                };
                let recorded = match handle.await {
                    Ok(recorded) => recorded,
                    Err(_) => {
                        let detail = "monty due task panicked or was cancelled";
                        if let Some(mut event) = event_record(
                            scheduler.coord.logger.service_name().clone(),
                            Event::DueJoinFailed,
                            Some(&row),
                            serde_json::json!({"error": detail}),
                        ) {
                            event.correlation_id = correlation.clone();
                            let _ = scheduler.coord.logger.emit(event);
                        };
                        let outcome = monty_failed(CODE_DUE_JOIN_FAILED, detail, DUE_JOIN_RECOVERY);
                        scheduler
                            .finish_due(&row, Report::Ran(outcome), next, correlation.clone())
                            .await
                    }
                };
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
fn monty_due_source(schedule: &Schedule) -> Result<String, Report> {
    let HookTarget::Monty { source } = &schedule.spec.hook else {
        return Err(Report::Ran(monty_failed(
            CODE_MONTY_RESOLVE,
            "monty path used for another hook",
            MONTY_RESOLVE_RECOVERY,
        )));
    };
    Ok(source.clone())
}

#[cfg(feature = "monty")]
async fn run_monty_due<C: Clock + 'static>(
    coord: Arc<RunCoord>,
    clock: Arc<C>,
    schedule: Schedule,
    deadline: std::time::Instant,
    stop_guard: crate::coord::MontyFeedGuard,
    hosts: Arc<crate::MontyHostRegistry>,
    execution: (Arc<crate::HookEnvironmentHost>, RunCorrelation),
) -> (Report, crate::coord::MontyFeedGuard) {
    let (environment_host, correlation) = execution;
    use crate::monty_run::{FeedResult, MontyPhase};
    let source = match monty_due_source(&schedule) {
        Ok(source) => source,
        Err(report) => {
            if let Some(mut event) = event_record(
                coord.logger.service_name().clone(),
                Event::MontyResolveFailed,
                Some(&schedule),
                serde_json::json!({"error": "monty path used for another hook"}),
            ) {
                event.correlation_id = correlation.clone();
                let _ = coord.logger.emit(event);
            }
            return (report, stop_guard);
        }
    };
    let environment = match environment_host
        .snapshot(clock.as_ref(), &schedule, deadline)
        .await
    {
        Ok(environment) => environment,
        Err(outcome) => return (Report::Ran(outcome), stop_guard),
    };
    let stop = feed_once(
        coord.take_repl(&schedule.spec.name),
        source.clone(),
        MontyPhase::Stop,
        schedule.generation,
        clock.as_ref(),
        &schedule,
        deadline,
        Arc::clone(&hosts),
        Arc::clone(&environment),
        Arc::clone(&coord.logger),
        correlation.clone(),
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
                (stop, source),
                hosts,
                environment,
                correlation,
            )
            .await
        }
    }
}

#[cfg(feature = "monty")]
#[expect(
    clippy::too_many_arguments,
    reason = "tick retains the stop snapshot and original deadline"
)]
async fn run_monty_tick_after_stop<C: Clock + 'static>(
    coord: Arc<RunCoord>,
    clock: Arc<C>,
    schedule: Schedule,
    deadline: std::time::Instant,
    stopped: (crate::monty_run::FeedDone, String),
    hosts: Arc<crate::MontyHostRegistry>,
    environment: Arc<crate::environment::EnvironmentSnapshot>,
    correlation: RunCorrelation,
) -> (Report, crate::coord::MontyFeedGuard) {
    use crate::monty_run::MontyPhase;
    let (stop, source) = stopped;
    let tick_guard = crate::coord::MontyFeedGuard::enter(&coord);
    let tick = feed_once(
        stop.session,
        source,
        MontyPhase::Tick,
        schedule.generation,
        clock.as_ref(),
        &schedule,
        deadline,
        hosts,
        environment,
        Arc::clone(&coord.logger),
        correlation.clone(),
    )
    .await;
    let report = tick_report(&coord, &schedule, tick, correlation);
    (report, tick_guard)
}

#[cfg(feature = "monty")]
fn tick_report(
    coord: &RunCoord,
    schedule: &Schedule,
    tick: crate::monty_run::FeedDone,
    correlation: RunCorrelation,
) -> Report {
    use crate::monty_run::FeedResult;
    match tick.result {
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
            if let Some(mut event) = event_record(
                coord.logger.service_name().clone(),
                Event::MontyTickFailed,
                Some(schedule),
                serde_json::json!({"error": "tick returned a stop action"}),
            ) {
                event.correlation_id = correlation.clone();
                let _ = coord.logger.emit(event);
            };
            Report::Ran(monty_failed(
                CODE_MONTY_TICK,
                "tick returned a stop action",
                MONTY_TICK_RECOVERY,
            ))
        }
    }
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
    hosts: Arc<crate::MontyHostRegistry>,
    environment: Arc<crate::environment::EnvironmentSnapshot>,
    logger: sc_command::observability::SharedCanonicalLog,
    correlation: RunCorrelation,
) -> crate::monty_run::FeedDone {
    crate::monty_run::drive_feed(
        clock,
        crate::monty_run::FeedLaunch {
            logger,
            correlation,
            session,
            source,
            phase,
            generation,
            name: schedule.spec.name.clone(),
            payload: schedule.spec.payload.clone(),
            timeout: schedule.spec.timeout,
            deadline,
            hosts,
            environment,
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
    #[tokio::test]
    async fn joined_due_tasks_emit_their_origin_before_terminal_recording() {
        use sc_command::observability::RecordingCanonicalLog;
        use sc_observability_types::ServiceName;
        use std::sync::Arc;
        let log = Arc::new(RecordingCanonicalLog::new(
            ServiceName::new("joined").unwrap(),
        ));
        let scheduler = crate::Scheduler::new_with_logger(
            Arc::new(
                crate::SqliteScheduleStore::open_in_memory(crate::ScheduleLimits::default())
                    .unwrap(),
            ),
            Arc::new(crate::SystemClock),
            Arc::new(crate::SystemJitter),
            log.clone(),
        );
        let row = scheduler
            .start(
                crate::ScheduleSpec::new(
                    "native",
                    "* * * * *",
                    std::time::Duration::from_secs(1),
                    serde_json::json!({}),
                )
                .unwrap(),
            )
            .unwrap();
        let task = tokio::spawn(async {
            panic!("native join");
            #[allow(unreachable_code)]
            super::Report::Ran(RunOutcome::Ok)
        });
        let runs = super::collect_runs(
            &scheduler,
            vec![(row.clone(), None, super::DueTask::abortable(task), None)],
        )
        .await
        .unwrap();
        assert_eq!(
            runs[0].outcome.failure_parts().unwrap().code,
            Some("RUN_TASK_FAILED")
        );
        let events = log.events();
        let origin = events
            .iter()
            .position(|e| e.action.as_str() == "due.task_failed")
            .unwrap();
        let terminal = events
            .iter()
            .position(|e| e.action.as_str() == "due.finished")
            .unwrap();
        assert!(origin < terminal);
        assert_eq!(
            events[terminal].diagnostic.as_ref().unwrap().code.as_str(),
            "JOB_FAILED"
        );
        #[cfg(feature = "monty")]
        {
            log.clear();
            async fn panic_due() -> super::DueRun {
                panic!("monty join")
            }
            let task = tokio::spawn(panic_due());
            let row = scheduler
                .start(
                    crate::ScheduleSpec::new(
                        "monty-joined",
                        "* * * * *",
                        std::time::Duration::from_secs(1),
                        serde_json::json!({}),
                    )
                    .unwrap(),
                )
                .unwrap();
            let runs = super::collect_runs(
                &scheduler,
                vec![(row.clone(), None, super::DueTask::monty(task), None)],
            )
            .await
            .unwrap();
            assert_eq!(
                runs[0].outcome.failure_parts().unwrap().code,
                Some("DUE_JOIN_FAILED")
            );
            let events = log.events();
            assert_eq!(
                events
                    .iter()
                    .map(|event| event.action.as_str())
                    .collect::<Vec<_>>(),
                ["due.join_failed", "due.finished"]
            );
            assert_eq!(
                events[0].diagnostic.as_ref().unwrap().code.as_str(),
                "DUE_JOIN_FAILED"
            );
            assert_eq!(
                events[1].diagnostic.as_ref().unwrap().code.as_str(),
                "JOB_FAILED"
            );
            use crate::ScheduleStore;
            assert_eq!(
                scheduler
                    .core
                    .store
                    .get(row.id)
                    .unwrap()
                    .unwrap()
                    .last_outcome,
                Some(runs[0].outcome.clone())
            );
        }
    }

    #[cfg(feature = "monty")]
    #[tokio::test]
    async fn invalid_monty_origins_use_fixed_typed_codes() {
        use sc_command::observability::RecordingCanonicalLog;
        use sc_observability_types::ServiceName;
        use std::sync::Arc;
        let log = Arc::new(RecordingCanonicalLog::new(
            ServiceName::new("monty-origin").unwrap(),
        ));
        let clock = Arc::new(crate::SystemClock);
        let scheduler = crate::Scheduler::new_with_logger(
            Arc::new(
                crate::SqliteScheduleStore::open_in_memory(crate::ScheduleLimits::default())
                    .unwrap(),
            ),
            clock.clone(),
            Arc::new(crate::SystemJitter),
            log.clone(),
        );
        let row = scheduler
            .start(
                crate::ScheduleSpec::new(
                    "wrong-target",
                    "* * * * *",
                    std::time::Duration::from_secs(1),
                    serde_json::json!({}),
                )
                .unwrap(),
            )
            .unwrap();
        let coord = Arc::clone(&scheduler.coord);
        let guard = crate::coord::MontyFeedGuard::enter(&coord);
        let (report, guard) = super::run_monty_due(
            coord.clone(),
            clock,
            row.clone(),
            std::time::Instant::now(),
            guard,
            Arc::clone(&scheduler.hosts),
            (Arc::clone(&scheduler.environment), None),
        )
        .await;
        assert!(matches!(report, super::Report::Ran(RunOutcome::Failed(_))));
        drop(guard);
        let report = super::tick_report(
            &coord,
            &row,
            crate::monty_run::FeedDone {
                session: None,
                result: crate::monty_run::FeedResult::Continue,
            },
            None,
        );
        assert!(matches!(report, super::Report::Ran(RunOutcome::Failed(_))));
        let codes: Vec<_> = log
            .events()
            .iter()
            .filter_map(|e| e.diagnostic.as_ref().map(|d| d.code.as_str().to_owned()))
            .collect();
        assert_eq!(codes, ["MONTY_RESOLVE", "MONTY_TICK"]);
    }
}
