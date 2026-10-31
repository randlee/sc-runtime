//! In-flight worker run management, including the optional Monty feed lifecycle.

use std::future::poll_fn;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::Poll;
use std::time::Instant;

use tokio::task::JoinError;

use super::*;
use crate::clock::sleep_until_monotonic;

#[cfg(feature = "monty")]
const CODE_FEED_JOIN_FAILED: &str = "FEED_JOIN_FAILED";
#[cfg(feature = "monty")]
const FEED_JOIN_RECOVERY: &str = "Fix the panic in the hook or host and retry the schedule.";

impl<S, C> Worker<S, C>
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
{
    /// Joins every in-flight run at once.
    ///
    /// Rust and Python still abort at their own monotonic deadline when no
    /// Monty feed is outstanding. A handle deadline aborts those siblings,
    /// and records `TimedOut`, only while a Monty feed is still outstanding.
    /// Monty joins are not aborted.
    pub(super) async fn drain(&mut self) {
        let mut siblings_aborted = false;
        loop {
            if !self.slots.iter().any(|slot| slot.run.is_some()) {
                return;
            }
            let monty_busy = self.coord.feeds.load(Ordering::Acquire) > 0;
            let nearest = if monty_busy {
                None
            } else {
                self.nearest_abortable_deadline()
            };
            let shutdown_at = self
                .handle_deadline
                .filter(|_| monty_busy && !siblings_aborted);
            let clock = Arc::clone(&self.clock);
            tokio::select! {
                biased;
                (index, finished, guard) = finished(&mut self.slots) => {
                    self.on_finished(index, finished, guard).await;
                }
                () = sleep_until_monotonic(clock.as_ref(), nearest.unwrap_or(Instant::now())), if nearest.is_some() => {
                    self.abort_due_abortable().await;
                }
                () = async {
                    if let Some(deadline) = shutdown_at {
                        wait_shutdown_deadline(deadline).await;
                    }
                }, if shutdown_at.is_some() => {
                    self.coord.note_shutdown_deadline();
                    siblings_aborted = true;
                    self.abort_all_abortable().await;
                }
            }
        }
    }

    fn nearest_abortable_deadline(&self) -> Option<Instant> {
        self.slots
            .iter()
            .filter_map(|slot| {
                slot.run
                    .as_ref()
                    .filter(|run| !run.is_monty())
                    .map(|run| run.deadline)
            })
            .min()
    }

    async fn abort_due_abortable(&mut self) {
        let monotonic = self.clock.monotonic_now();
        let mut index = 0;
        while index < self.slots.len() {
            let due = self.slots.get(index).is_some_and(|slot| {
                slot.run
                    .as_ref()
                    .is_some_and(|run| !run.is_monty() && run.deadline <= monotonic)
            });
            if !due {
                index += 1;
                continue;
            }
            let Some(run) = self.slots[index].run.take() else {
                index += 1;
                continue;
            };
            if let Some(event) = detail_event(
                self.coord.logger.service_name().clone(),
                &self.slots[index].schedule,
                Event::RunTimedOut,
                "run deadline passed",
            ) {
                let _ = self.coord.logger.emit(event);
            };
            let result = await_aborted(run).await;
            if !self.complete(index, result).await {
                index += 1;
            }
        }
    }

    async fn abort_all_abortable(&mut self) {
        let mut index = 0;
        while index < self.slots.len() {
            let abortable = self
                .slots
                .get(index)
                .is_some_and(|slot| slot.run.as_ref().is_some_and(|run| !run.is_monty()));
            if !abortable {
                index += 1;
                continue;
            }
            let Some(run) = self.slots[index].run.take() else {
                index += 1;
                continue;
            };
            if let Some(event) = detail_event(
                self.coord.logger.service_name().clone(),
                &self.slots[index].schedule,
                Event::RunTimedOut,
                "shutdown deadline passed while a Monty feed is in flight",
            ) {
                let _ = self.coord.logger.emit(event);
            };
            let result = await_aborted(run).await;
            if !self.complete(index, result).await {
                index += 1;
            }
        }
    }

    pub(super) async fn on_finished(
        &mut self,
        index: usize,
        finished: Finished,
        guard: Option<MontyFeedGuard>,
    ) {
        if let Some(slot) = self.slots.get_mut(index) {
            slot.run = None;
        }
        match finished {
            Finished::Report(result) => {
                drop(guard);
                self.complete(index, result).await;
            }
            #[cfg(feature = "monty")]
            Finished::Monty(result) => self.finish_monty(index, result, guard).await,
        }
    }

    #[cfg(feature = "monty")]
    pub(super) fn launch_monty(&mut self, index: usize) {
        let name = self
            .slots
            .get(index)
            .map(|slot| slot.schedule.spec.name.clone());
        let session = name.as_ref().and_then(|name| self.coord.take_repl(name));
        let guard = MontyFeedGuard::enter(&self.coord);
        self.spawn_feed(index, crate::monty_run::MontyPhase::Stop, session, guard);
    }

    #[cfg(feature = "monty")]
    fn spawn_feed(
        &mut self,
        index: usize,
        phase: crate::monty_run::MontyPhase,
        session: Option<crate::monty_run::MontySession>,
        guard: MontyFeedGuard,
    ) {
        let Some(slot) = self.slots.get(index) else {
            drop(guard);
            return;
        };
        let HookTarget::Monty { source } = &slot.schedule.spec.hook else {
            drop(guard);
            return;
        };
        let source = source.clone();
        let generation = slot.schedule.generation;
        let schedule = slot.schedule.clone();
        let clock = Arc::clone(&self.clock);
        let monotonic = self.clock.monotonic_now();
        let deadline = if phase == crate::monty_run::MontyPhase::Tick {
            session.as_ref().map_or(monotonic, |s| s.deadline)
        } else {
            monotonic
                .checked_add(schedule.spec.timeout.get())
                .unwrap_or(monotonic)
        };
        let snapshot = if phase == crate::monty_run::MontyPhase::Tick {
            session.as_ref().map(|s| Arc::clone(&s.environment))
        } else {
            None
        };
        let environment_host = Arc::clone(&self.environment);
        let hosts = Arc::clone(&self.hosts);
        let launching = phase == crate::monty_run::MontyPhase::Stop;
        let logger = Arc::clone(&self.coord.logger);
        let handle = tokio::spawn(async move {
            let environment = match snapshot {
                Some(environment) => environment,
                None => match environment_host
                    .snapshot(clock.as_ref(), &schedule, deadline)
                    .await
                {
                    Ok(environment) => environment,
                    Err(outcome) => {
                        return crate::monty_run::FeedDone {
                            session: None,
                            result: crate::monty_run::FeedResult::Outcome(outcome),
                        };
                    }
                },
            };
            crate::monty_run::drive_feed(
                clock.as_ref(),
                crate::monty_run::FeedLaunch {
                    logger,
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
        });
        self.install_feed(index, RunHandle::Monty(handle), deadline, guard, launching);
    }

    #[cfg(feature = "monty")]
    fn install_feed(
        &mut self,
        index: usize,
        handle: RunHandle,
        deadline: Instant,
        guard: MontyFeedGuard,
        launching: bool,
    ) {
        let Some(slot) = self.slots.get_mut(index) else {
            drop(guard);
            return;
        };
        if launching
            && let Some(event) = event_record(
                self.coord.logger.service_name().clone(),
                Event::RunLaunched,
                Some(&slot.schedule),
                serde_json::json!({"deadline_remaining_ms": deadline.saturating_duration_since(self.clock.monotonic_now()).as_millis() as u64}),
            )
        {
            let _ = self.coord.logger.emit(event);
        }
        slot.run = Some(InFlight {
            handle: Some(handle),
            deadline,
            guard: Some(guard),
        });
    }

    #[cfg(feature = "monty")]
    async fn finish_monty(
        &mut self,
        index: usize,
        result: Result<crate::monty_run::FeedDone, JoinError>,
        guard: Option<MontyFeedGuard>,
    ) {
        let Some((done, guard)) = self.monty_feed_from_join(index, result, guard).await else {
            return;
        };
        self.apply_monty_feed_done(index, done, guard).await;
    }

    #[cfg(feature = "monty")]
    async fn monty_feed_from_join(
        &mut self,
        index: usize,
        result: Result<crate::monty_run::FeedDone, JoinError>,
        guard: Option<MontyFeedGuard>,
    ) -> Option<(crate::monty_run::FeedDone, Option<MontyFeedGuard>)> {
        match result {
            Ok(done) => Some((done, guard)),
            Err(error) if error.is_panic() => {
                let message = panic_message(error);
                if let Some(slot) = self.slots.get(index) {
                    let schedule = slot.schedule.clone();
                    if let Some(event) = event_record(
                        self.coord.logger.service_name().clone(),
                        Event::FeedJoinFailed,
                        Some(&schedule),
                        serde_json::json!({"error": message}),
                    ) {
                        let _ = self.coord.logger.emit(event);
                    };
                    self.record(
                        &schedule,
                        RunOutcome::Failed(format!(
                            "{CODE_FEED_JOIN_FAILED}: monty feed task panicked: {message}; {FEED_JOIN_RECOVERY}"
                        )),
                    )
                    .await;
                }
                self.forget_repl(index);
                drop(guard);
                None
            }
            Err(error) => {
                if let Some(slot) = self.slots.get(index)
                    && let Some(event) = detail_event(
                        self.coord.logger.service_name().clone(),
                        &slot.schedule,
                        Event::ScheduleCancelled,
                        &error.to_string(),
                    )
                {
                    let _ = self.coord.logger.emit(event);
                }
                drop(guard);
                None
            }
        }
    }

    #[cfg(feature = "monty")]
    async fn apply_monty_feed_done(
        &mut self,
        index: usize,
        done: crate::monty_run::FeedDone,
        guard: Option<MontyFeedGuard>,
    ) {
        use crate::monty_run::{FeedResult, MontyPhase};
        let Some(slot) = self.slots.get(index) else {
            drop(done.session);
            drop(guard);
            return;
        };
        let schedule = slot.schedule.clone();
        let queued = slot.queued.clone();
        let interrupt = self.stopping || !matches!(queued, Queued::None);
        match done.result {
            FeedResult::Disable(reason) => {
                drop(done.session);
                self.disable(&schedule, &reason).await;
                drop(guard);
                self.remove_or_settle_monty_slot(index, &queued).await;
            }
            FeedResult::Delete(reason) => {
                drop(done.session);
                if !self.end(&schedule, &reason).await {
                    self.rebuild_later();
                }
                drop(guard);
                self.remove_or_settle_monty_slot(index, &queued).await;
            }
            FeedResult::Continue if interrupt => {
                drop(done.session);
                drop(guard);
                self.settle_queued(index).await;
            }
            FeedResult::Continue => {
                let tick_guard = MontyFeedGuard::enter(&self.coord);
                drop(guard);
                self.spawn_feed(index, MontyPhase::Tick, done.session, tick_guard);
            }
            FeedResult::Outcome(outcome) => {
                let keep = matches!(outcome, RunOutcome::Ok) && !interrupt;
                self.record(&schedule, outcome).await;
                if keep {
                    if let Some(session) = done.session {
                        self.coord.store_repl(schedule.spec.name.clone(), session);
                    }
                } else {
                    self.coord.drop_repl(&schedule.spec.name);
                    drop(done.session);
                }
                drop(guard);
                if interrupt {
                    self.settle_queued(index).await;
                }
            }
        }
    }

    #[cfg(feature = "monty")]
    async fn remove_or_settle_monty_slot(&mut self, index: usize, queued: &Queued) {
        if matches!(queued, Queued::Replace(_)) {
            self.settle_queued(index).await;
        } else {
            self.slots.remove(index);
        }
    }

    #[cfg(feature = "monty")]
    fn forget_repl(&mut self, index: usize) {
        if let Some(slot) = self.slots.get(index) {
            self.coord.drop_repl(&slot.schedule.spec.name);
        }
    }

    #[cfg(feature = "monty")]
    async fn settle_queued(&mut self, index: usize) {
        let Some(slot) = self.slots.get_mut(index) else {
            return;
        };
        let queued = std::mem::replace(&mut slot.queued, Queued::None);
        match queued {
            Queued::Replace(schedule) => {
                let schedule = *schedule;
                self.slots.remove(index);
                if !self.stop_requested()
                    && let Some(slot) = self.new_slot(schedule).await
                {
                    self.slots.push(slot);
                }
            }
            Queued::None | Queued::Disable | Queued::Delete => {
                self.slots.remove(index);
            }
        }
    }
}

pub(super) enum Finished {
    Report(Result<Report, JoinError>),
    #[cfg(feature = "monty")]
    Monty(Result<crate::monty_run::FeedDone, JoinError>),
}

/// Aborts a cooperative run and reads `TimedOut` from that cancellation.
pub(super) async fn await_aborted(mut run: InFlight) -> Result<Report, JoinError> {
    run.abort_if_cooperative();
    match run.handle.take() {
        Some(RunHandle::Abortable(mut handle)) => match (&mut handle).await {
            Err(error) if error.is_cancelled() => Ok(Report::Ran(RunOutcome::TimedOut)),
            other => other,
        },
        _ => Ok(Report::Ran(RunOutcome::TimedOut)),
    }
}

/// Resolves when any slot's run finishes. The feed guard is taken out so the
/// caller can drop it after the outcome write.
pub(super) async fn finished(slots: &mut [Slot]) -> (usize, Finished, Option<MontyFeedGuard>) {
    poll_fn(|context| {
        for (index, slot) in slots.iter_mut().enumerate() {
            let Some(run) = slot.run.as_mut() else {
                continue;
            };
            let ready = match run.handle.as_mut() {
                Some(RunHandle::Abortable(handle)) => match Pin::new(handle).poll(context) {
                    Poll::Ready(result) => Some(Finished::Report(result)),
                    Poll::Pending => None,
                },
                #[cfg(feature = "monty")]
                Some(RunHandle::Monty(handle)) => match Pin::new(handle).poll(context) {
                    Poll::Ready(result) => Some(Finished::Monty(result)),
                    Poll::Pending => None,
                },
                None => None,
            };
            if let Some(finished) = ready {
                let guard = run.guard.take();
                return Poll::Ready((index, finished, guard));
            }
        }
        Poll::Pending
    })
    .await
}

/// Waits until a handle deadline from [`WorkerHandle::shutdown`]. Uses the
/// tokio clock so manual test clocks do not stall real shutdown timeouts.
async fn wait_shutdown_deadline(deadline: tokio::time::Instant) {
    use std::time::Duration;
    use tokio::time::MissedTickBehavior;
    let mut tick = tokio::time::interval(Duration::from_millis(5));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    while tokio::time::Instant::now() < deadline {
        tick.tick().await;
    }
}

pub(super) fn panic_message(error: JoinError) -> String {
    let payload = error.into_panic();
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-string panic".to_owned())
}

/// Typed worker-origin failures keep the persisted consumer format in one place.
pub(super) enum WorkerFailure {
    TargetResolve,
    JobJoin,
}
impl WorkerFailure {
    pub(super) fn outcome(self, detail: impl std::fmt::Display) -> RunOutcome {
        let (code, recovery) = match self {
            Self::TargetResolve => (
                "TARGET_RESOLVE_FAILED",
                "fix hook target configuration and retry",
            ),
            Self::JobJoin => ("JOB_JOIN_FAILED", "fix the job panic and retry"),
        };
        RunOutcome::Failed(format!("{code}: {detail}; {recovery}"))
    }
}

/// Fixed event origins: application prose is never used to choose a diagnostic.
#[derive(Clone, Copy)]
pub(crate) enum Event {
    TargetResolveFailed,
    ScheduleReloadFailed,
    ScheduleCancelled,
    RunTimedOut,
    RunSkippedInFlight,
    RunMissedSkipped,
    RunMissedReplayed,
    RunLaunched,
    RunFinished,
    RunRecordFailed,
    ScheduleNextFailed,
    ScheduleStopped,
    ScheduleStopFailed,
    ScheduleDisabled,
    ScheduleDisableFailed,
    RunFenced,
    DueLaunched,
    DueFinished,
    DueFenced,
    DuePartialWrite,
    DueFollowUpFailed,
    DueRecordFailed,
    DueStopped,
    DueStopFailed,
    DueNextFailed,
    JobJoinFailed,
    #[cfg(feature = "monty")]
    DueJoinFailed,
    DueTaskFailed,
    #[cfg(feature = "monty")]
    MontyResolveFailed,
    #[cfg(feature = "monty")]
    MontyTickFailed,
    #[cfg(feature = "monty")]
    FeedJoinFailed,
}
#[path = "event_rows.rs"]
mod event_rows;
impl Event {
    fn row(self) -> EventRow {
        match self {
            Self::TargetResolveFailed => event_rows::TARGET_RESOLVE_FAILED,
            Self::ScheduleReloadFailed => event_rows::SCHEDULE_RELOAD_FAILED,
            Self::ScheduleCancelled => event_rows::SCHEDULE_CANCELLED,
            Self::RunTimedOut => event_rows::RUN_TIMED_OUT,
            Self::RunSkippedInFlight => event_rows::RUN_SKIPPED_IN_FLIGHT,
            Self::RunMissedSkipped => event_rows::RUN_MISSED_SKIPPED,
            Self::RunMissedReplayed => event_rows::RUN_MISSED_REPLAYED,
            Self::RunLaunched => event_rows::RUN_LAUNCHED,
            Self::RunFinished => event_rows::RUN_FINISHED,
            Self::RunRecordFailed => event_rows::RUN_RECORD_FAILED,
            Self::ScheduleNextFailed => event_rows::SCHEDULE_NEXT_FAILED,
            Self::ScheduleStopped => event_rows::SCHEDULE_STOPPED,
            Self::ScheduleStopFailed => event_rows::SCHEDULE_STOP_FAILED,
            Self::ScheduleDisabled => event_rows::SCHEDULE_DISABLED,
            Self::ScheduleDisableFailed => event_rows::SCHEDULE_DISABLE_FAILED,
            Self::RunFenced => event_rows::RUN_FENCED,
            Self::DueLaunched => event_rows::DUE_LAUNCHED,
            Self::DueFinished => event_rows::DUE_FINISHED,
            Self::DueFenced => event_rows::DUE_FENCED,
            Self::DuePartialWrite => event_rows::DUE_PARTIAL_WRITE,
            Self::DueFollowUpFailed => event_rows::DUE_FOLLOW_UP_FAILED,
            Self::DueRecordFailed => event_rows::DUE_RECORD_FAILED,
            Self::DueStopped => event_rows::DUE_STOPPED,
            Self::DueStopFailed => event_rows::DUE_STOP_FAILED,
            Self::DueNextFailed => event_rows::DUE_NEXT_FAILED,
            Self::JobJoinFailed => event_rows::JOB_JOIN_FAILED,
            #[cfg(feature = "monty")]
            Self::DueJoinFailed => event_rows::DUE_JOIN_FAILED,
            Self::DueTaskFailed => event_rows::DUE_TASK_FAILED,
            #[cfg(feature = "monty")]
            Self::MontyResolveFailed => event_rows::MONTY_RESOLVE_FAILED,
            #[cfg(feature = "monty")]
            Self::MontyTickFailed => event_rows::MONTY_TICK_FAILED,
            #[cfg(feature = "monty")]
            Self::FeedJoinFailed => event_rows::FEED_JOIN_FAILED,
        }
    }
}

/// Constructs the fixed origin row without owning or invoking a logger.
pub(crate) fn event_record(
    service: sc_observability_types::ServiceName,
    event: Event,
    schedule: Option<&Schedule>,
    fields: serde_json::Value,
) -> Option<sc_observability_types::LogEvent> {
    let (target, action, level, outcome, code) = event.row();
    event_row(
        service,
        (target, action, level, outcome, code),
        schedule,
        fields,
    )
}

type EventRow = (
    &'static str,
    &'static str,
    sc_observability_types::Level,
    Option<&'static str>,
    Option<&'static str>,
);
fn event_row(
    service: sc_observability_types::ServiceName,
    row: EventRow,
    schedule: Option<&Schedule>,
    fields: serde_json::Value,
) -> Option<sc_observability_types::LogEvent> {
    use sc_observability_types::{
        ActionName, ErrorCode, ErrorContext, LogEvent, OBSERVATION_SCHEMA_VERSION, OutcomeLabel,
        ProcessIdentity, Remediation, TargetCategory, Timestamp,
    };
    let (target, action, level, outcome, code) = row;
    let (Ok(target), Ok(action)) = (TargetCategory::new(target), ActionName::new(action)) else {
        return None;
    };
    let mut fields = fields.as_object().cloned().unwrap_or_default();
    if let Some(schedule) = schedule {
        fields.insert("schedule".into(), schedule.id.to_string().into());
    }
    let timestamp = Timestamp::now_utc();
    let diagnostic = code.map(|code| {
        let context = ErrorContext::new(
            ErrorCode::new_static(code),
            action.as_str(),
            Remediation::recoverable(
                "Resolve the job or store failure and retry.",
                [] as [&str; 0],
            ),
        );
        let mut diagnostic = context.diagnostic().clone();
        diagnostic.timestamp = timestamp;
        diagnostic
    });
    Some(LogEvent {
        version: (*OBSERVATION_SCHEMA_VERSION).clone(),
        timestamp,
        level,
        service,
        target,
        action,
        message: None,
        identity: ProcessIdentity::default(),
        trace: None,
        request_id: None,
        correlation_id: None,
        outcome: outcome.and_then(|label| OutcomeLabel::new(label).ok()),
        diagnostic,
        state_transition: None,
        fields,
    })
}

/// Constructs a terminal row without changing persistence or emitting it.
pub(crate) fn terminal_event(
    service: sc_observability_types::ServiceName,
    event: Event,
    schedule: &Schedule,
    outcome: &RunOutcome,
) -> Option<sc_observability_types::LogEvent> {
    use sc_observability_types::Level;
    let (level, label, code, fields) = match outcome {
        RunOutcome::Ok => (Level::Info, "success", None, serde_json::json!({})),
        RunOutcome::Refused(reason) => (
            Level::Info,
            "refused",
            None,
            serde_json::json!({"reason": reason}),
        ),
        RunOutcome::Failed(error) => (
            Level::Error,
            "failed",
            Some("JOB_FAILED"),
            serde_json::json!({"error": error}),
        ),
        RunOutcome::TimedOut => (
            Level::Warn,
            "timeout",
            Some("RUN_TIMEOUT"),
            serde_json::json!({"deadline_remaining_ms": 0}),
        ),
    };
    let (target, action, _, _, _) = event.row();
    event_row(
        service,
        (target, action, level, Some(label), code),
        Some(schedule),
        fields,
    )
}

#[cfg(test)]
#[path = "event_contract_tests.rs"]
mod event_contract_tests;
