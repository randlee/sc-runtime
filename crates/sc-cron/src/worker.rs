//! The worker task that ticks every active schedule.
//!
//! The worker keeps one slot per active schedule: its stored row (whose
//! expression is already parsed), its next run time and at most one run in
//! flight. Each loop pass it (1) reloads the rows if signalled or a retry is
//! due, (2) times out runs past their deadline, (3) launches the runs that
//! are due, then sleeps on the clock until the next run time, the nearest
//! run deadline, a reload signal, a run finishing or shutdown. The worker is
//! the only caller of [`Clock::sleep_until`], so a manual clock's parks tell
//! a test that the worker has settled. Shutdown is checked between slots,
//! so a stop request is honoured before the next store call.
//!
//! Each run is its own tokio task: stop check, then the job, under one
//! deadline on the clock's monotonic timeline. A timed-out run is aborted at
//! its deadline by the worker loop, so the next tick is never delayed by it,
//! and a wall-clock jump cannot cut it short. A tick that comes due while
//! the previous run is still in flight is skipped and logged. Every store
//! write names `(id, generation)`; a fenced write makes the worker reload.
//!
//! A run is the row's [`HookTarget`]: a Rust job or `python3 <module>`
//! (REQ-CRON-0013). The worker reads that hook once per launch, before the
//! stop check; a Python path that cannot be built is recorded
//! [`RunOutcome::Failed`] and the schedule stays active and enabled. The stop
//! check answers continue, disable or delete (REQ-CRON-0005,
//! ADR-CRON-0007). A disabled schedule has no slot, so it never fires
//! (REQ-CRON-0014).
//!
//! Nothing signals this worker when another process starts, restarts,
//! enables, disables or stops a schedule, and a schedule it holds no slot for
//! (new, disabled, ended) wakes nothing. So the worker reloads at least once
//! per [`DEFAULT_RELOAD_POLL`] (or the interval set with
//! `Scheduler::with_reload_poll`) on the injected clock, whatever the rows
//! are: a change made elsewhere is noticed at most one interval after it is
//! stored (REQ-CRON-0014). A reload is one `list_active` query that keeps
//! every unchanged slot, so the poll is cheap.

use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use tokio::sync::{oneshot, watch};
use tokio::task::{JoinError, JoinHandle};

use crate::clock::wall_after;
use crate::coord::{MontyFeedGuard, RunCoord};
use crate::hook::{ResolvedTarget, Step, stop_step, tick_outcome};
use crate::{
    Clock, CronError, HookCall, HookEvent, HookTarget, Job, MissedTickPolicy, PythonHook,
    RunOutcome, Schedule, ScheduleStore, StopDecision, StoreError, StoreErrorKind, Timestamp,
};

mod worker_runs;

pub(crate) use worker_runs::{Event, event_record, terminal_event};
use worker_runs::{WorkerFailure, await_aborted, finished, panic_message};

pub(crate) type RunCorrelation = Option<sc_observability_types::CorrelationId>;

/// Logical occurrence identity within this scheduler/store domain.
pub(crate) fn run_correlation(schedule: &Schedule, due: Option<DateTime<Utc>>) -> RunCorrelation {
    let due = due?;
    sc_observability_types::CorrelationId::new(format!(
        "cron-s{}-g{}-t{}-n{}",
        schedule.id,
        schedule.generation,
        due.timestamp(),
        due.timestamp_subsec_nanos()
    ))
    .ok()
}

/// The longest a worker goes between reloads of the active schedules.
///
/// A schedule started, restarted, enabled or stopped by another process sends
/// this worker no signal, so the reload is the only way it learns of that
/// (REQ-CRON-0014). 30 s keeps the delay before such a change takes effect
/// short against the usual tick (a minute or more) and costs one
/// `list_active` per 30 s. A shorter interval raises that cost; a longer one
/// raises the worst-case delay.
pub const DEFAULT_RELOAD_POLL: Duration = Duration::from_secs(30);

/// The shortest reload poll accepted; anything below is raised to it.
///
/// Below one second the poll would cost a reload per tick of a busy loop for
/// no gain, since the finest schedule resolution is one second.
pub(crate) const MIN_RELOAD_POLL: Duration = Duration::from_secs(1);

/// Stop reason recorded when an expression has no further occurrence.
pub(crate) const EXHAUSTED: &str = "no future occurrence";

/// What a run task reports back to the worker.
#[derive(Debug)]
pub(crate) enum Report {
    /// The stop check asked to end the schedule; the job did not run.
    Delete(String),
    /// The stop check asked to disable the schedule; the job did not run.
    Disable(String),
    /// The job ran with this outcome, or the run was skipped with one.
    Ran(RunOutcome),
}

/// The join a slot holds. Monty joins are not aborted when the slot drops.
enum RunHandle {
    Abortable(JoinHandle<Report>),
    #[cfg(feature = "monty")]
    Monty(JoinHandle<crate::monty_run::FeedDone>),
}

/// One run in flight.
struct InFlight {
    correlation: RunCorrelation,
    handle: Option<RunHandle>,
    /// On the clock's monotonic timeline. Ignored while `is_monty` is set:
    /// that run is not woken or expired from this instant.
    deadline: Instant,
    guard: Option<MontyFeedGuard>,
}

impl InFlight {
    fn is_monty(&self) -> bool {
        #[cfg(feature = "monty")]
        {
            matches!(self.handle, Some(RunHandle::Monty(_)))
        }
        #[cfg(not(feature = "monty"))]
        {
            false
        }
    }

    fn abort_if_cooperative(&mut self) {
        if let Some(RunHandle::Abortable(handle)) = self.handle.as_ref() {
            handle.abort();
        }
    }
}

/// Dropping a Rust or Python run cancels it. A Monty run is detached instead,
/// so a dropped slot does not abort the interpreter. The feed guard still
/// drops, which is what lets shutdown proceed after a panic.
impl Drop for InFlight {
    fn drop(&mut self) {
        if let Some(RunHandle::Abortable(handle)) = self.handle.take() {
            handle.abort();
        }
    }
}

/// A row change that arrived while a Monty feed was still inside this slot.
#[cfg_attr(not(feature = "monty"), allow(dead_code))]
#[derive(Clone, Debug)]
enum Queued {
    None,
    Replace(Box<Schedule>),
    Disable,
    Delete,
}

/// One active schedule as the worker tracks it.
struct Slot {
    schedule: Schedule,
    next: Option<DateTime<Utc>>,
    run: Option<InFlight>,
    queued: Queued,
    /// Logging-only deduplication while run_due retains this name.
    last_skipped: Option<DateTime<Utc>>,
}

/// Backoff for a failed reload, or for a failed stop or disable write: 1 s
/// doubling per failure, capped at 60 s.
#[derive(Debug, Default)]
struct ReloadRetry {
    failures: u32,
    due: Option<DateTime<Utc>>,
}

const RELOAD_RETRY_MAX_SECS: i64 = 60;

/// Largest shift used for the backoff: 2^6 = 64 s is already past the cap.
const RELOAD_RETRY_MAX_SHIFT: u32 = 6;

impl ReloadRetry {
    fn delay(failures: u32) -> chrono::Duration {
        let secs = (1i64 << failures.min(RELOAD_RETRY_MAX_SHIFT)).min(RELOAD_RETRY_MAX_SECS);
        chrono::Duration::seconds(secs)
    }

    fn failed(&mut self, now: DateTime<Utc>) {
        self.due = Some(now + Self::delay(self.failures));
        self.failures = self.failures.saturating_add(1);
    }

    fn succeeded(&mut self) {
        *self = Self::default();
    }

    fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.due.is_some_and(|due| due <= now)
    }
}

pub(crate) struct Worker<S, C> {
    store: Arc<S>,
    clock: Arc<C>,
    job: Arc<dyn Job>,
    policy: MissedTickPolicy,
    coord: Arc<RunCoord>,
    environment: Arc<crate::HookEnvironmentHost>,
    #[cfg(feature = "monty")]
    hosts: Arc<crate::MontyHostRegistry>,
    shutdown: oneshot::Receiver<Option<tokio::time::Instant>>,
    /// Tokio instant from [`crate::WorkerHandle::shutdown`]. `None` when the
    /// handle was dropped, which stops the worker without a handle deadline.
    handle_deadline: Option<tokio::time::Instant>,
    stopping: bool,
    slots: Vec<Slot>,
    reload_wanted: bool,
    retry: ReloadRetry,
    /// Backoff for the writes a stop check or an exhausted schedule needs.
    /// Unlike `retry`, a successful reload does not reset it: only a write
    /// that reached the store does, so a store that keeps failing that write
    /// is retried at 1 s, 2 s, ... 60 s, not every second.
    write_retry: ReloadRetry,
    /// Exhausted schedules whose ending write failed. Their last occurrence
    /// has run, but the stored next run is still that occurrence, so a slot
    /// rebuilt from the row would run it again: no slot is built for these,
    /// and the write is retried until the store answers.
    pending_ends: Vec<(Schedule, RunCorrelation)>,
    /// The most the worker waits between reloads.
    reload_poll: chrono::Duration,
    /// When the next periodic reload is due; `None` until the first reload
    /// succeeds, and while a failed reload is being retried by the backoff.
    poll_due: Option<DateTime<Utc>>,
}

/// Resolves the stored hook of one launch of `schedule`.
///
/// An error is the outcome to record, [`RunOutcome::Failed`], with the
/// schedule left active and enabled; it is logged as `resolve_failed`.
pub(crate) fn resolve(
    job: &Arc<dyn Job>,
    schedule: &Schedule,
    logger: &sc_observability_types::SharedCanonicalLog,
    correlation: RunCorrelation,
) -> Result<ResolvedTarget, RunOutcome> {
    match &schedule.spec.hook {
        HookTarget::Rust => Ok(ResolvedTarget::Rust(Arc::clone(job))),
        HookTarget::Python { module } => {
            match PythonHook::new_with_logger(module, Arc::clone(logger)) {
                Ok(hook) => Ok(ResolvedTarget::Python(
                    hook.with_correlation(correlation.clone()),
                )),
                Err(error) => {
                    if let Some(mut event) = event_record(
                        logger.service_name().clone(),
                        Event::TargetResolveFailed,
                        Some(schedule),
                        serde_json::json!({"target": module.display().to_string(), "error": error.recorded()}),
                    ) {
                        event.correlation_id = correlation.clone();
                        let _ = logger.emit(event);
                    };
                    Err(WorkerFailure::TargetResolve.outcome(error.recorded()))
                }
            }
        }
        HookTarget::Monty { .. } => {
            let error = CronError::HookFailed(
                "Monty hooks require sc-cron's optional `monty` feature".to_owned(),
            );
            if let Some(mut event) = event_record(
                logger.service_name().clone(),
                Event::TargetResolveFailed,
                Some(schedule),
                serde_json::json!({"target": "monty", "error": error.recorded()}),
            ) {
                event.correlation_id = correlation.clone();
                let _ = logger.emit(event);
            };
            Err(WorkerFailure::TargetResolve.outcome(error.recorded()))
        }
    }
}

/// One run of `target` for `schedule`: the stop check, then the run, under
/// `deadline` on `clock`'s monotonic timeline (REQ-CRON-0005, REQ-CRON-0013).
pub(crate) async fn run_target_with_environment<C: Clock + ?Sized>(
    target: ResolvedTarget,
    clock: Arc<C>,
    schedule: &Schedule,
    deadline: Instant,
    host: &crate::HookEnvironmentHost,
) -> Report {
    if let Err(error) = schedule.spec.validate_environment() {
        return Report::Ran(RunOutcome::Failed(error.recorded()));
    }
    let payload = &schedule.spec.payload;
    match target {
        ResolvedTarget::Rust(job) => match job.should_stop(payload).await {
            StopDecision::Continue => Report::Ran(job.run(payload, deadline).await),
            StopDecision::Disable(reason) => Report::Disable(reason),
            StopDecision::Delete(reason) => Report::Delete(reason),
        },
        ResolvedTarget::Python(hook) => {
            let environment = match host.snapshot(clock.as_ref(), schedule, deadline).await {
                Ok(environment) => environment,
                Err(outcome) => return Report::Ran(outcome),
            };
            let hook = hook.with_env(environment.child_env());
            // resolve installed RunCoord's identical logger Arc. The resolved
            // hook carries it through this future into supervise_with_logger.
            let call = |event| HookCall {
                event,
                schedule: &schedule.spec.name,
                payload,
                timeout: schedule.spec.timeout,
                deadline,
            };
            match stop_step(&hook, clock.as_ref(), &call(HookEvent::Stop), &environment).await {
                Step::Run => Report::Ran(
                    tick_outcome(&hook, clock.as_ref(), &call(HookEvent::Tick), &environment).await,
                ),
                Step::Skip(outcome) => Report::Ran(outcome),
                Step::Disable(reason) => Report::Disable(reason),
                Step::Delete(reason) => Report::Delete(reason),
            }
        }
    }
}

#[cfg(test)]
pub(crate) async fn run_target<C: Clock + ?Sized>(
    target: ResolvedTarget,
    clock: Arc<C>,
    schedule: &Schedule,
    deadline: Instant,
) -> Report {
    run_target_with_environment(
        target,
        clock,
        schedule,
        deadline,
        &crate::HookEnvironmentHost::new(),
    )
    .await
}

impl<S, C> Worker<S, C>
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        store: Arc<S>,
        clock: Arc<C>,
        job: Arc<dyn Job>,
        policy: MissedTickPolicy,
        coord: Arc<RunCoord>,
        reload_poll: Duration,
        shutdown: oneshot::Receiver<Option<tokio::time::Instant>>,
        #[cfg(feature = "monty")] hosts: Arc<crate::MontyHostRegistry>,
    ) -> Self {
        Self {
            store,
            clock,
            job,
            policy,
            coord,
            environment: Arc::new(crate::HookEnvironmentHost::new()),
            #[cfg(feature = "monty")]
            hosts,
            shutdown,
            handle_deadline: None,
            stopping: false,
            slots: Vec::new(),
            reload_wanted: true,
            retry: ReloadRetry::default(),
            write_retry: ReloadRetry::default(),
            pending_ends: Vec::new(),
            reload_poll: chrono::Duration::from_std(reload_poll.max(MIN_RELOAD_POLL))
                .unwrap_or(chrono::Duration::MAX),
            poll_due: None,
        }
    }

    pub(crate) fn with_hook_environment(
        mut self,
        environment: Arc<crate::HookEnvironmentHost>,
    ) -> Self {
        self.environment = environment;
        self
    }

    /// Returns whether shutdown was requested, without waiting.
    fn note_stop(&mut self, deadline: Option<tokio::time::Instant>) {
        self.stopping = true;
        if deadline.is_some() {
            self.handle_deadline = deadline;
        }
    }

    fn stop_requested(&mut self) -> bool {
        if !self.stopping {
            match self.shutdown.try_recv() {
                Ok(deadline) => self.note_stop(deadline),
                Err(oneshot::error::TryRecvError::Closed) => self.stopping = true,
                Err(oneshot::error::TryRecvError::Empty) => {}
            }
        }
        self.stopping
    }

    pub(crate) async fn run(mut self, mut reload: watch::Receiver<u64>) {
        let mut reload_open = true;
        loop {
            if self.stop_requested() {
                break;
            }
            let due = self.clock.now();
            let write_due = self.write_retry.is_due(due);
            if write_due {
                self.write_retry.due = None;
                self.retry_ends().await;
            }
            if self.reload_wanted || write_due || self.retry.is_due(due) || self.poll_is_due(due) {
                self.reload().await;
            }
            self.expire().await;
            let now = self.clock.now();
            self.launch_due(now).await;
            if self.stop_requested() {
                break;
            }
            // A fenced write asks for a reload: do it now, not at the next wake.
            let wake = if self.reload_wanted { now } else { self.wake() };
            let clock = Arc::clone(&self.clock);
            tokio::select! {
                biased;
                message = &mut self.shutdown => {
                    self.note_stop(message.ok().flatten());
                    break;
                }
                changed = reload.changed(), if reload_open => {
                    // A dropped scheduler cannot signal again; keep ticking.
                    reload_open = changed.is_ok();
                    self.reload_wanted |= reload_open;
                }
                (index, finished, guard) = finished(&mut self.slots) => {
                    self.on_finished(index, finished, guard).await;
                }
                () = clock.sleep_until(wake) => {}
            }
        }
        self.drain().await;
    }

    /// The earliest wall-clock instant the worker has something to do.
    fn wake(&self) -> DateTime<Utc> {
        let (now, monotonic) = (self.clock.now(), self.clock.monotonic_now());
        let slots = self.slots.iter().flat_map(|slot| {
            let deadline = slot
                .run
                .as_ref()
                .filter(|run| !run.is_monty())
                .map(|run| wall_after(now, run.deadline.saturating_duration_since(monotonic)));
            let next = slot
                .next
                .into_iter()
                .filter(|_| !self.coord.name_in_flight(&slot.schedule.spec.name));
            next.chain(deadline)
        });
        slots
            .chain(self.retry.due)
            .chain(self.write_retry.due)
            .chain(self.poll_due)
            .min()
            .unwrap_or(DateTime::<Utc>::MAX_UTC)
    }

    fn poll_is_due(&self, now: DateTime<Utc>) -> bool {
        self.poll_due.is_some_and(|due| due <= now)
    }

    /// Asks for a reload, with backoff, after a store error on a stop check's
    /// `Delete` or `Disable` left a schedule without a slot, so it is not
    /// silently lost. The reload rebuilds the slot from the row, which is
    /// still active and enabled, and its next run was already advanced.
    ///
    /// Not for an exhausted schedule: its stored next run is the occurrence
    /// that has already run, so a rebuilt slot would run it again. Those go
    /// through [`Worker::end_exhausted`].
    fn rebuild_later(&mut self) {
        self.write_retry.failed(self.clock.now());
    }

    /// Called when a stop or disable write reached the store.
    fn write_answered(&mut self) {
        if self.pending_ends.is_empty() {
            self.write_retry.failures = 0;
        }
    }

    fn is_pending_end(&self, schedule: &Schedule) -> bool {
        self.pending_ends
            .iter()
            .any(|(ending, _)| ending.id == schedule.id && ending.generation == schedule.generation)
    }

    /// Ends a schedule whose last occurrence has run. If the store fails, the
    /// write is retried with backoff and the schedule gets no slot meanwhile,
    /// so it can never run again.
    async fn end_exhausted(&mut self, schedule: &Schedule, correlation: RunCorrelation) {
        if self.end(schedule, EXHAUSTED, correlation.clone()).await {
            return;
        }
        if !self.is_pending_end(schedule) {
            self.pending_ends.push((schedule.clone(), correlation));
        }
        self.write_retry.failed(self.clock.now());
    }

    /// Retries the ending writes that failed.
    async fn retry_ends(&mut self) {
        for (schedule, correlation) in std::mem::take(&mut self.pending_ends) {
            if self.stop_requested() {
                self.pending_ends.push((schedule, correlation));
            } else if !self.end(&schedule, EXHAUSTED, correlation.clone()).await {
                self.pending_ends.push((schedule, correlation));
                self.write_retry.failed(self.clock.now());
            }
        }
    }

    fn now_stamp(&self) -> Timestamp {
        Timestamp::from_datetime(self.clock.now())
    }

    async fn store_call<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&S) -> Result<T, CronError> + Send + 'static,
    ) -> Result<T, CronError> {
        let store = Arc::clone(&self.store);
        // Not cancelled: shutdown waits out the current statement. Lock waits
        // inside it are bounded by the store's SQLite busy timeout.
        tokio::task::spawn_blocking(move || operation(store.as_ref()))
            .await
            .map_err(|error| {
                CronError::Store(
                    StoreError::new(StoreErrorKind::Other, "store task failed").with_source(error),
                )
            })?
    }

    async fn reload(&mut self) {
        self.reload_wanted = false;
        // A successful reload arms the poll again; a failed one is retried by
        // the backoff, so a past deadline must not linger and spin the loop.
        self.poll_due = None;
        // The interval counts from before the query: a write made after the
        // rows were read is then at most one interval from the next reload.
        let started = self.clock.now();
        match self.store_call(ScheduleStore::list_active).await {
            Ok(rows) => {
                self.retry.succeeded();
                self.poll_due = started.checked_add_signed(self.reload_poll);
                self.merge(rows).await;
            }
            Err(error) => {
                if let Some(event) = event_record(
                    self.coord.logger.service_name().clone(),
                    Event::ScheduleReloadFailed,
                    None,
                    serde_json::json!({"error": error.to_string()}),
                ) {
                    let _ = self.coord.logger.emit(event);
                };
                self.retry.failed(self.clock.now());
            }
        }
    }

    /// Keeps the slots whose `(id, generation)` is unchanged, builds the rest.
    ///
    /// Once shutdown is requested no new slot is built (that needs store
    /// calls), but the keep-matching pass still runs to the end: an unchanged
    /// slot's in-flight run must reach `drain` and be recorded, not be
    /// cancelled by dropping its slot.
    async fn merge(&mut self, rows: Vec<Schedule>) {
        let mut previous = std::mem::take(&mut self.slots);
        for schedule in rows.into_iter().filter(|schedule| schedule.is_active()) {
            if let Some(index) = previous.iter().position(|slot| {
                slot.schedule.id == schedule.id && slot.schedule.generation == schedule.generation
            }) {
                let mut slot = previous.swap_remove(index);
                if schedule.enabled {
                    self.slots.push(slot);
                } else if slot.run.as_ref().is_some_and(InFlight::is_monty) {
                    slot.queued = Queued::Disable;
                    self.slots.push(slot);
                } else if slot.run.is_some()
                    && let Some(mut event) = detail_event(
                        self.coord.logger.service_name().clone(),
                        &slot.schedule,
                        Event::ScheduleCancelled,
                        "schedule disabled",
                    )
                {
                    event.correlation_id =
                        slot.run.as_ref().and_then(|run| run.correlation.clone());
                    let _ = self.coord.logger.emit(event);
                }
                continue;
            }
            if let Some(index) = previous.iter().position(|slot| {
                slot.schedule.spec.name == schedule.spec.name
                    && slot.run.as_ref().is_some_and(InFlight::is_monty)
            }) {
                let mut slot = previous.swap_remove(index);
                slot.queued = if schedule.enabled {
                    Queued::Replace(Box::new(schedule))
                } else {
                    Queued::Disable
                };
                self.slots.push(slot);
                continue;
            }
            if schedule.enabled
                && !self.stop_requested()
                && let Some(slot) = self.new_slot(schedule).await
            {
                self.slots.push(slot);
            }
        }
        for mut slot in previous {
            if slot.run.as_ref().is_some_and(InFlight::is_monty) {
                if matches!(slot.queued, Queued::None) {
                    slot.queued = Queued::Delete;
                }
                self.slots.push(slot);
            } else if slot.run.is_some()
                && let Some(mut event) = detail_event(
                    self.coord.logger.service_name().clone(),
                    &slot.schedule,
                    Event::ScheduleCancelled,
                    "schedule ended, disabled or restarted",
                )
            {
                event.correlation_id = slot.run.as_ref().and_then(|run| run.correlation.clone());
                let _ = self.coord.logger.emit(event);
            }
        }
    }

    async fn new_slot(&mut self, schedule: Schedule) -> Option<Slot> {
        if self.is_pending_end(&schedule) {
            // Its last occurrence has run; only the ending write is missing.
            return None;
        }
        let next = if let Some(next) = schedule.next_run_at {
            Some(next.as_datetime())
        } else {
            // The port allows an active row without a next run (a store
            // that never had one); compute it. A corrupt stored value
            // never reaches here: it fails decoding in the store.
            let next = schedule
                .spec
                .expr
                .next_run(schedule.jitter, self.clock.now());
            if let Some(next) = next {
                self.persist_next(&schedule, next).await;
            }
            next
        };
        if next.is_none() {
            self.end_exhausted(&schedule, None).await;
            return None;
        }
        Some(Slot {
            schedule,
            next,
            run: None,
            queued: Queued::None,
            last_skipped: None,
        })
    }

    /// Times out every run past its monotonic deadline; the run records `TimedOut`.
    async fn expire(&mut self) {
        let mut index = 0;
        while index < self.slots.len() {
            if self.stop_requested() {
                return;
            }
            let monotonic = self.clock.monotonic_now();
            let Some(slot) = self.slots.get_mut(index) else {
                return;
            };
            let expired = slot
                .run
                .as_ref()
                .is_some_and(|run| !run.is_monty() && run.deadline <= monotonic);
            let run = if expired { slot.run.take() } else { None };
            let Some(run) = run else {
                index += 1;
                continue;
            };
            let correlation = run.correlation.clone();
            if let Some(mut event) = detail_event(
                self.coord.logger.service_name().clone(),
                &slot.schedule,
                Event::RunTimedOut,
                "run deadline passed",
            ) {
                event.correlation_id = correlation.clone();
                let _ = self.coord.logger.emit(event);
            };
            let result = await_aborted(run).await;
            if !self.complete(index, result, correlation).await {
                index += 1;
            }
        }
    }

    /// Launches every due slot, per the missed-tick policy.
    async fn launch_due(&mut self, now: DateTime<Utc>) {
        let mut index = 0;
        while index < self.slots.len() {
            if self.stop_requested() {
                return;
            }
            let Some(slot) = self.slots.get_mut(index) else {
                return;
            };
            if self.coord.name_in_flight(&slot.schedule.spec.name) {
                if let Some(due) = slot.next.filter(|next| *next <= now)
                    && slot.last_skipped != Some(due)
                {
                    slot.last_skipped = Some(due);
                    if let Some(event) = detail_event(
                        self.coord.logger.service_name().clone(),
                        &slot.schedule,
                        Event::RunSkippedInFlight,
                        "run_due still holds this name",
                    ) {
                        let _ = self.coord.logger.emit(event);
                    };
                }
                index += 1;
                continue;
            }
            let Some(due) = slot.next.filter(|next| *next <= now) else {
                index += 1;
                continue;
            };
            let (expr, jitter) = (&slot.schedule.spec.expr, slot.schedule.jitter);
            let missed = expr.next_run(jitter, due).is_some_and(|after| after <= now);
            let next = expr.next_run(jitter, now);
            slot.next = next;
            let schedule = slot.schedule.clone();
            if let Some(next) = next
                && !self.persist_next(&schedule, next).await
            {
                // Fenced: the schedule changed under us; the reload decides.
                index += 1;
                continue;
            }
            self.launch_selected(index, missed, run_correlation(&schedule, Some(due)));
            let Some(slot) = self.slots.get(index) else {
                return;
            };
            if slot.next.is_none() && slot.run.is_none() {
                let slot = self.slots.remove(index);
                self.end_exhausted(&slot.schedule, None).await;
            } else {
                index += 1;
            }
        }
    }

    fn launch_selected(&mut self, index: usize, missed: bool, correlation: RunCorrelation) {
        let Some(slot) = self.slots.get(index) else {
            return;
        };
        if slot.run.is_some() {
            if let Some(event) = detail_event(
                self.coord.logger.service_name().clone(),
                &slot.schedule,
                Event::RunSkippedInFlight,
                "previous run still in flight",
            ) {
                let _ = self.coord.logger.emit(event);
            };
        } else if missed && self.policy == MissedTickPolicy::Skip {
            if let Some(event) = detail_event(
                self.coord.logger.service_name().clone(),
                &slot.schedule,
                Event::RunMissedSkipped,
                "missed occurrences dropped",
            ) {
                let _ = self.coord.logger.emit(event);
            };
        } else {
            if missed
                && let Some(event) = detail_event(
                    self.coord.logger.service_name().clone(),
                    &slot.schedule,
                    Event::RunMissedReplayed,
                    "missed occurrences run once",
                )
            {
                let _ = self.coord.logger.emit(event);
            }
            self.launch(index, correlation);
        }
    }

    fn launch(&mut self, index: usize, correlation: RunCorrelation) {
        #[cfg(feature = "monty")]
        if matches!(
            self.slots.get(index).map(|slot| &slot.schedule.spec.hook),
            Some(HookTarget::Monty { .. })
        ) {
            self.launch_monty(index, correlation);
            return;
        }
        let monotonic = self.clock.monotonic_now();
        let Some(slot) = self.slots.get_mut(index) else {
            return;
        };
        let timeout = slot.schedule.spec.timeout;
        // RunTimeout is at most 24 h, so this cannot overflow in practice.
        let deadline = monotonic.checked_add(timeout.get()).unwrap_or(monotonic);
        let handle = match resolve(
            &self.job,
            &slot.schedule,
            &self.coord.logger,
            correlation.clone(),
        ) {
            Ok(target) => {
                let clock = Arc::clone(&self.clock);
                let schedule = slot.schedule.clone();
                let environment = Arc::clone(&self.environment);
                tokio::spawn(async move {
                    run_target_with_environment(target, clock, &schedule, deadline, &environment)
                        .await
                })
            }
            Err(outcome) => tokio::spawn(async move { Report::Ran(outcome) }),
        };
        if let Some(mut event) = event_record(
            self.coord.logger.service_name().clone(),
            Event::RunLaunched,
            Some(&slot.schedule),
            serde_json::json!({"deadline_remaining_ms": deadline.saturating_duration_since(self.clock.monotonic_now()).as_millis() as u64}),
        ) {
            event.correlation_id = correlation.clone();
            let _ = self.coord.logger.emit(event);
        };
        slot.run = Some(InFlight {
            correlation,
            handle: Some(RunHandle::Abortable(handle)),
            deadline,
            guard: None,
        });
    }

    /// Records a finished run. Returns whether the slot was removed.
    async fn complete(
        &mut self,
        index: usize,
        result: Result<Report, JoinError>,
        correlation: RunCorrelation,
    ) -> bool {
        let Some(slot) = self.slots.get_mut(index) else {
            return false;
        };
        slot.run = None;
        let schedule = slot.schedule.clone();
        match result {
            Ok(Report::Delete(reason)) => {
                self.slots.remove(index);
                if !self.end(&schedule, &reason, correlation.clone()).await {
                    self.rebuild_later();
                }
                return true;
            }
            Ok(Report::Disable(reason)) => {
                self.slots.remove(index);
                self.disable(&schedule, &reason, correlation.clone()).await;
                return true;
            }
            Ok(Report::Ran(outcome)) => self.record(&schedule, outcome, correlation.clone()).await,
            Err(error) if error.is_panic() => {
                let message = panic_message(error);
                if let Some(mut event) = event_record(
                    self.coord.logger.service_name().clone(),
                    Event::JobJoinFailed,
                    Some(&schedule),
                    serde_json::json!({"error": message}),
                ) {
                    event.correlation_id = correlation.clone();
                    let _ = self.coord.logger.emit(event);
                };
                self.record(
                    &schedule,
                    WorkerFailure::JobJoin.outcome(message),
                    correlation.clone(),
                )
                .await;
            }
            Err(error) => {
                if let Some(mut event) = detail_event(
                    self.coord.logger.service_name().clone(),
                    &schedule,
                    Event::ScheduleCancelled,
                    &error.to_string(),
                ) {
                    event.correlation_id = correlation.clone();
                    let _ = self.coord.logger.emit(event);
                }
            }
        }
        if self
            .slots
            .get(index)
            .is_some_and(|slot| slot.next.is_none())
        {
            self.slots.remove(index);
            self.end_exhausted(&schedule, correlation).await;
            return true;
        }
        false
    }

    async fn record(
        &mut self,
        schedule: &Schedule,
        outcome: RunOutcome,
        correlation: RunCorrelation,
    ) {
        let (id, generation) = (schedule.id, schedule.generation);
        let at = self.now_stamp();
        let recorded = outcome.clone();
        let result = self
            .store_call(move |store| store.record_run(id, generation, at, &outcome))
            .await;
        match result {
            Ok(true) => {
                if let Some(mut event) = terminal_event(
                    self.coord.logger.service_name().clone(),
                    Event::RunFinished,
                    schedule,
                    &recorded,
                ) {
                    event.correlation_id = correlation.clone();
                    let _ = self.coord.logger.emit(event);
                }
            }
            Ok(false) => self.fenced(schedule, "record_run", correlation.clone()),
            Err(error) => {
                if let Some(mut event) = event_record(
                    self.coord.logger.service_name().clone(),
                    Event::RunRecordFailed,
                    Some(schedule),
                    serde_json::json!({"error": error.to_string()}),
                ) {
                    event.correlation_id = correlation.clone();
                    let _ = self.coord.logger.emit(event);
                }
            }
        }
    }

    /// Stores the next run time; returns `false` only when fenced.
    async fn persist_next(&mut self, schedule: &Schedule, next: DateTime<Utc>) -> bool {
        let (id, generation) = (schedule.id, schedule.generation);
        let next = Timestamp::from_datetime(next);
        match self
            .store_call(move |store| store.set_next(id, generation, next))
            .await
        {
            Ok(true) => true,
            Ok(false) => {
                self.fenced(schedule, "set_next", None);
                false
            }
            Err(error) => {
                // The in-memory next run still drives this process.
                if let Some(event) = event_record(
                    self.coord.logger.service_name().clone(),
                    Event::ScheduleNextFailed,
                    Some(schedule),
                    serde_json::json!({"error": error.to_string()}),
                ) {
                    let _ = self.coord.logger.emit(event);
                };
                true
            }
        }
    }

    /// Writes the end of a schedule; returns `false` only when the store
    /// failed, so the caller can retry. A fenced write is answered (`true`).
    async fn end(
        &mut self,
        schedule: &Schedule,
        reason: &str,
        correlation: RunCorrelation,
    ) -> bool {
        let (id, generation) = (schedule.id, schedule.generation);
        let at = self.now_stamp();
        let owned = reason.to_owned();
        match self
            .store_call(move |store| store.stop(id, generation, at, &owned))
            .await
        {
            Ok(true) => {
                if let Some(mut event) = detail_event(
                    self.coord.logger.service_name().clone(),
                    schedule,
                    Event::ScheduleStopped,
                    reason,
                ) {
                    event.correlation_id = correlation.clone();
                    let _ = self.coord.logger.emit(event);
                }
            }
            Ok(false) => self.fenced(schedule, "stop", correlation.clone()),
            Err(error) => {
                if let Some(mut event) = event_record(
                    self.coord.logger.service_name().clone(),
                    Event::ScheduleStopFailed,
                    Some(schedule),
                    serde_json::json!({"error": error.to_string()}),
                ) {
                    event.correlation_id = correlation.clone();
                    let _ = self.coord.logger.emit(event);
                };
                return false;
            }
        }
        self.write_answered();
        true
    }

    /// Disables the schedule on its stop check's say-so; fenced like every write.
    async fn disable(&mut self, schedule: &Schedule, reason: &str, correlation: RunCorrelation) {
        let (id, generation) = (schedule.id, schedule.generation);
        let at = self.now_stamp();
        let owned = reason.to_owned();
        match self
            .store_call(move |store| store.disable(id, generation, at, &owned))
            .await
        {
            Ok(true) => {
                if let Some(mut event) = detail_event(
                    self.coord.logger.service_name().clone(),
                    schedule,
                    Event::ScheduleDisabled,
                    reason,
                ) {
                    event.correlation_id = correlation.clone();
                    let _ = self.coord.logger.emit(event);
                };
                if let Some(mut event) = terminal_event(
                    self.coord.logger.service_name().clone(),
                    Event::RunFinished,
                    schedule,
                    &RunOutcome::Refused(reason.to_owned()),
                ) {
                    event.correlation_id = correlation.clone();
                    let _ = self.coord.logger.emit(event);
                };
                self.write_answered();
            }
            Ok(false) => self.fenced(schedule, "disable", correlation.clone()),
            Err(error) => {
                // The slot is gone but the row is still enabled: rebuild it.
                if let Some(mut event) = event_record(
                    self.coord.logger.service_name().clone(),
                    Event::ScheduleDisableFailed,
                    Some(schedule),
                    serde_json::json!({"error": error.to_string()}),
                ) {
                    event.correlation_id = correlation.clone();
                    let _ = self.coord.logger.emit(event);
                };
                self.rebuild_later();
            }
        }
    }

    fn fenced(&mut self, schedule: &Schedule, operation: &str, correlation: RunCorrelation) {
        if let Some(mut event) = event_record(
            self.coord.logger.service_name().clone(),
            Event::RunFenced,
            Some(schedule),
            serde_json::json!({"operation": operation}),
        ) {
            event.correlation_id = correlation.clone();
            let _ = self.coord.logger.emit(event);
        };
        self.reload_wanted = true;
    }
}

fn detail_event(
    service: sc_observability_types::ServiceName,
    schedule: &Schedule,
    event: Event,
    detail: &str,
) -> Option<sc_observability_types::LogEvent> {
    let fields = match event {
        Event::RunTimedOut => serde_json::json!({"deadline_remaining_ms": 0}),
        Event::RunMissedSkipped | Event::RunMissedReplayed => serde_json::json!({}),
        _ => serde_json::json!({"reason": detail}),
    };
    event_record(service, event, Some(schedule), fields)
}

#[cfg(test)]
mod tests;
