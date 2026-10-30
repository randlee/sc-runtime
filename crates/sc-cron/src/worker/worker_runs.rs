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
            log(
                &self.coord.logger,
                &self.slots[index].schedule,
                Event::RunTimedOut,
                "run deadline passed",
            );
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
            log(
                &self.coord.logger,
                &self.slots[index].schedule,
                Event::RunTimedOut,
                "shutdown deadline passed while a Monty feed is in flight",
            );
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
        let deadline = monotonic
            .checked_add(schedule.spec.timeout.get())
            .unwrap_or(monotonic);
        let hosts = Arc::clone(&self.hosts);
        let launching = phase == crate::monty_run::MontyPhase::Stop;
        let logger = Arc::clone(&self.coord.logger);
        let handle = tokio::spawn(async move {
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
                },
            )
            .await
        });
        let Some(slot) = self.slots.get_mut(index) else {
            drop(guard);
            return;
        };
        if launching {
            emit(
                &self.coord.logger,
                Event::RunLaunched,
                Some(&slot.schedule),
                serde_json::json!({"deadline_remaining_ms": deadline.saturating_duration_since(self.clock.monotonic_now()).as_millis() as u64}),
            );
        }
        slot.run = Some(InFlight {
            handle: Some(RunHandle::Monty(handle)),
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
                    emit(
                        &self.coord.logger,
                        Event::FeedJoinFailed,
                        Some(&schedule),
                        serde_json::json!({"error": message}),
                    );
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
                if let Some(slot) = self.slots.get(index) {
                    log(
                        &self.coord.logger,
                        &slot.schedule,
                        Event::ScheduleCancelled,
                        &error.to_string(),
                    );
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
impl Event {
    fn row(self) -> EventRow {
        use sc_observability_types::Level;
        match self {
            Self::TargetResolveFailed => (
                "cron.worker",
                "target.resolve_failed",
                Level::Warn,
                Some("failed"),
                Some("TARGET_RESOLVE_FAILED"),
            ),
            Self::ScheduleReloadFailed => (
                "cron.worker",
                "schedule.reload_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_RELOAD_FAILED"),
            ),
            Self::ScheduleCancelled => (
                "cron.worker",
                "schedule.cancelled",
                Level::Info,
                Some("stopped"),
                None,
            ),
            Self::RunTimedOut => (
                "cron.worker",
                "run.timed_out",
                Level::Warn,
                Some("timeout"),
                Some("RUN_TIMEOUT"),
            ),
            Self::RunSkippedInFlight => (
                "cron.worker",
                "run.skipped_in_flight",
                Level::Info,
                None,
                None,
            ),
            Self::RunMissedSkipped => {
                ("cron.worker", "run.missed_skipped", Level::Info, None, None)
            }
            Self::RunMissedReplayed => (
                "cron.worker",
                "run.missed_replayed",
                Level::Info,
                None,
                None,
            ),
            Self::RunLaunched => ("cron.worker", "run.launched", Level::Info, None, None),
            Self::RunFinished => ("cron.worker", "run.finished", Level::Info, None, None),
            Self::RunRecordFailed => (
                "cron.worker",
                "run.record_failed",
                Level::Error,
                Some("failed"),
                Some("RUN_RECORD_FAILED"),
            ),
            Self::ScheduleNextFailed => (
                "cron.worker",
                "schedule.next_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_NEXT_FAILED"),
            ),
            Self::ScheduleStopped => (
                "cron.worker",
                "schedule.stopped",
                Level::Info,
                Some("stopped"),
                None,
            ),
            Self::ScheduleStopFailed => (
                "cron.worker",
                "schedule.stop_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_STOP_FAILED"),
            ),
            Self::ScheduleDisabled => (
                "cron.worker",
                "schedule.disabled",
                Level::Info,
                Some("stopped"),
                None,
            ),
            Self::ScheduleDisableFailed => (
                "cron.worker",
                "schedule.disable_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_DISABLE_FAILED"),
            ),
            Self::RunFenced => (
                "cron.worker",
                "run.fenced",
                Level::Warn,
                Some("fenced"),
                Some("SCHEDULE_FENCED"),
            ),
            Self::DueLaunched => ("cron.scheduler", "due.launched", Level::Info, None, None),
            Self::DueFinished => ("cron.scheduler", "due.finished", Level::Info, None, None),
            Self::DueFenced => (
                "cron.scheduler",
                "due.fenced",
                Level::Warn,
                Some("fenced"),
                Some("SCHEDULE_FENCED"),
            ),
            Self::DuePartialWrite => (
                "cron.scheduler",
                "due.partial_write",
                Level::Warn,
                Some("partial"),
                Some("FOLLOW_UP_FENCED"),
            ),
            Self::DueFollowUpFailed => (
                "cron.scheduler",
                "due.follow_up_failed",
                Level::Error,
                Some("partial"),
                Some("FOLLOW_UP_WRITE_FAILED"),
            ),
            Self::DueRecordFailed => (
                "cron.scheduler",
                "due.record_failed",
                Level::Error,
                Some("failed"),
                Some("RUN_RECORD_FAILED"),
            ),
            Self::DueStopped => (
                "cron.scheduler",
                "due.stopped",
                Level::Info,
                Some("stopped"),
                None,
            ),
            Self::DueStopFailed => (
                "cron.scheduler",
                "due.stop_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_STOP_FAILED"),
            ),
            Self::DueNextFailed => (
                "cron.scheduler",
                "due.next_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_NEXT_FAILED"),
            ),
            Self::JobJoinFailed => (
                "cron.worker",
                "job.join_failed",
                Level::Error,
                Some("failed"),
                Some("JOB_JOIN_FAILED"),
            ),
            #[cfg(feature = "monty")]
            Self::DueJoinFailed => (
                "cron.scheduler",
                "due.join_failed",
                Level::Error,
                Some("failed"),
                Some("DUE_JOIN_FAILED"),
            ),
            Self::DueTaskFailed => (
                "cron.scheduler",
                "due.task_failed",
                Level::Error,
                Some("failed"),
                Some("RUN_TASK_FAILED"),
            ),
            #[cfg(feature = "monty")]
            Self::MontyResolveFailed => (
                "cron.scheduler",
                "monty.resolve_failed",
                Level::Error,
                Some("failed"),
                Some("MONTY_RESOLVE"),
            ),
            #[cfg(feature = "monty")]
            Self::MontyTickFailed => (
                "cron.scheduler",
                "monty.tick_failed",
                Level::Error,
                Some("failed"),
                Some("MONTY_TICK"),
            ),
            #[cfg(feature = "monty")]
            Self::FeedJoinFailed => (
                "cron.monty",
                "feed.join_failed",
                Level::Error,
                Some("failed"),
                Some("FEED_JOIN_FAILED"),
            ),
        }
    }
}

pub(crate) fn emit(
    logger: &sc_observability_types::SharedCanonicalLog,
    event: Event,
    schedule: Option<&Schedule>,
    fields: serde_json::Value,
) {
    let (target, action, level, outcome, code) = event.row();
    emit_row(
        logger,
        (target, action, level, outcome, code),
        schedule,
        fields,
    );
}

type EventRow = (
    &'static str,
    &'static str,
    sc_observability_types::Level,
    Option<&'static str>,
    Option<&'static str>,
);
fn emit_row(
    logger: &sc_observability_types::SharedCanonicalLog,
    row: EventRow,
    schedule: Option<&Schedule>,
    fields: serde_json::Value,
) {
    use sc_observability_types::{
        ActionName, ErrorCode, ErrorContext, LogEvent, OBSERVATION_SCHEMA_VERSION, OutcomeLabel,
        ProcessIdentity, Remediation, TargetCategory, Timestamp,
    };
    let (target, action, level, outcome, code) = row;
    let (Ok(target), Ok(action)) = (TargetCategory::new(target), ActionName::new(action)) else {
        return;
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
    let _ = logger.emit(LogEvent {
        version: (*OBSERVATION_SCHEMA_VERSION).clone(),
        timestamp,
        level,
        service: logger.service_name().clone(),
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
    });
}

pub(crate) fn terminal(
    logger: &sc_observability_types::SharedCanonicalLog,
    event: Event,
    schedule: &Schedule,
    outcome: &RunOutcome,
) {
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
    emit_row(
        logger,
        (target, action, level, Some(label), code),
        Some(schedule),
        fields,
    );
}

#[cfg(test)]
mod event_contract_tests {
    use super::*;
    use sc_observability_types::{Level, RecordingCanonicalLog, ServiceName, SharedCanonicalLog};

    #[test]
    fn fixed_origin_rows_use_exact_taxonomy_and_best_effort_envelope() {
        let recorder = Arc::new(RecordingCanonicalLog::new(
            ServiceName::new("worker-test").unwrap(),
        ));
        let logger: SharedCanonicalLog = recorder.clone();
        let rows = [
            (
                Event::TargetResolveFailed,
                "cron.worker",
                "target.resolve_failed",
                Level::Warn,
                Some("failed"),
                Some("TARGET_RESOLVE_FAILED"),
            ),
            (
                Event::ScheduleReloadFailed,
                "cron.worker",
                "schedule.reload_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_RELOAD_FAILED"),
            ),
            (
                Event::ScheduleCancelled,
                "cron.worker",
                "schedule.cancelled",
                Level::Info,
                Some("stopped"),
                None,
            ),
            (
                Event::RunTimedOut,
                "cron.worker",
                "run.timed_out",
                Level::Warn,
                Some("timeout"),
                Some("RUN_TIMEOUT"),
            ),
            (
                Event::RunSkippedInFlight,
                "cron.worker",
                "run.skipped_in_flight",
                Level::Info,
                None,
                None,
            ),
            (
                Event::RunMissedSkipped,
                "cron.worker",
                "run.missed_skipped",
                Level::Info,
                None,
                None,
            ),
            (
                Event::RunMissedReplayed,
                "cron.worker",
                "run.missed_replayed",
                Level::Info,
                None,
                None,
            ),
            (
                Event::RunLaunched,
                "cron.worker",
                "run.launched",
                Level::Info,
                None,
                None,
            ),
            (
                Event::RunRecordFailed,
                "cron.worker",
                "run.record_failed",
                Level::Error,
                Some("failed"),
                Some("RUN_RECORD_FAILED"),
            ),
            (
                Event::ScheduleNextFailed,
                "cron.worker",
                "schedule.next_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_NEXT_FAILED"),
            ),
            (
                Event::ScheduleStopped,
                "cron.worker",
                "schedule.stopped",
                Level::Info,
                Some("stopped"),
                None,
            ),
            (
                Event::ScheduleStopFailed,
                "cron.worker",
                "schedule.stop_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_STOP_FAILED"),
            ),
            (
                Event::ScheduleDisabled,
                "cron.worker",
                "schedule.disabled",
                Level::Info,
                Some("stopped"),
                None,
            ),
            (
                Event::ScheduleDisableFailed,
                "cron.worker",
                "schedule.disable_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_DISABLE_FAILED"),
            ),
            (
                Event::RunFenced,
                "cron.worker",
                "run.fenced",
                Level::Warn,
                Some("fenced"),
                Some("SCHEDULE_FENCED"),
            ),
            (
                Event::DueLaunched,
                "cron.scheduler",
                "due.launched",
                Level::Info,
                None,
                None,
            ),
            (
                Event::DueFenced,
                "cron.scheduler",
                "due.fenced",
                Level::Warn,
                Some("fenced"),
                Some("SCHEDULE_FENCED"),
            ),
            (
                Event::DuePartialWrite,
                "cron.scheduler",
                "due.partial_write",
                Level::Warn,
                Some("partial"),
                Some("FOLLOW_UP_FENCED"),
            ),
            (
                Event::DueFollowUpFailed,
                "cron.scheduler",
                "due.follow_up_failed",
                Level::Error,
                Some("partial"),
                Some("FOLLOW_UP_WRITE_FAILED"),
            ),
            (
                Event::DueRecordFailed,
                "cron.scheduler",
                "due.record_failed",
                Level::Error,
                Some("failed"),
                Some("RUN_RECORD_FAILED"),
            ),
            (
                Event::DueStopped,
                "cron.scheduler",
                "due.stopped",
                Level::Info,
                Some("stopped"),
                None,
            ),
            (
                Event::DueStopFailed,
                "cron.scheduler",
                "due.stop_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_STOP_FAILED"),
            ),
            (
                Event::DueNextFailed,
                "cron.scheduler",
                "due.next_failed",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_NEXT_FAILED"),
            ),
            (
                Event::JobJoinFailed,
                "cron.worker",
                "job.join_failed",
                Level::Error,
                Some("failed"),
                Some("JOB_JOIN_FAILED"),
            ),
            #[cfg(feature = "monty")]
            (
                Event::DueJoinFailed,
                "cron.scheduler",
                "due.join_failed",
                Level::Error,
                Some("failed"),
                Some("DUE_JOIN_FAILED"),
            ),
            (
                Event::DueTaskFailed,
                "cron.scheduler",
                "due.task_failed",
                Level::Error,
                Some("failed"),
                Some("RUN_TASK_FAILED"),
            ),
            #[cfg(feature = "monty")]
            (
                Event::MontyResolveFailed,
                "cron.scheduler",
                "monty.resolve_failed",
                Level::Error,
                Some("failed"),
                Some("MONTY_RESOLVE"),
            ),
            #[cfg(feature = "monty")]
            (
                Event::MontyTickFailed,
                "cron.scheduler",
                "monty.tick_failed",
                Level::Error,
                Some("failed"),
                Some("MONTY_TICK"),
            ),
            #[cfg(feature = "monty")]
            (
                Event::FeedJoinFailed,
                "cron.monty",
                "feed.join_failed",
                Level::Error,
                Some("failed"),
                Some("FEED_JOIN_FAILED"),
            ),
        ];
        for (event, target, action, level, outcome, code) in rows {
            recorder.clear();
            emit(&logger, event, None, serde_json::json!({"error": "opaque"}));
            let events = recorder.events();
            assert_eq!(events.len(), 1);
            let actual = &events[0];
            assert_eq!(
                (actual.target.as_str(), actual.action.as_str(), actual.level),
                (target, action, level)
            );
            assert_eq!(actual.outcome.as_ref().map(|v| v.as_str()), outcome);
            assert_eq!(actual.diagnostic.as_ref().map(|v| v.code.as_str()), code);
            assert_eq!(actual.service.as_str(), "worker-test");
            assert!(actual.trace.is_none() && actual.state_transition.is_none());
            assert!(!actual.fields.contains_key("code"));
            if let Some(diagnostic) = &actual.diagnostic {
                assert_eq!(diagnostic.timestamp, actual.timestamp);
            }
        }
    }

    #[test]
    fn python_resolved_target_carries_the_same_arc_through_run_target() {
        let recorder = Arc::new(RecordingCanonicalLog::new(
            ServiceName::new("python").unwrap(),
        ));
        let logger: SharedCanonicalLog = recorder;
        let store = Arc::new(
            crate::SqliteScheduleStore::open_in_memory(crate::ScheduleLimits::default()).unwrap(),
        );
        let clock = Arc::new(crate::SystemClock);
        let scheduler = crate::Scheduler::new_with_logger(
            store,
            clock.clone(),
            Arc::new(crate::SystemJitter),
            logger.clone(),
        );
        let mut spec = crate::ScheduleSpec::new(
            "python",
            "* * * * *",
            std::time::Duration::from_secs(1),
            serde_json::json!({}),
        )
        .unwrap();
        spec.hook = crate::HookTarget::python("/tmp/b15-hook.py").unwrap();
        let row = scheduler.start(spec).unwrap();
        let job: Arc<dyn crate::Job> = Arc::new(Idle);
        let before = Arc::strong_count(&logger);
        let target = resolve(&job, &row, &logger).unwrap();
        assert_eq!(Arc::strong_count(&logger), before + 1);
        let future = run_target(target, clock, &row, std::time::Instant::now());
        assert_eq!(Arc::strong_count(&logger), before + 1);
        drop(future);
        assert_eq!(Arc::strong_count(&logger), before);
    }
    #[cfg(feature = "monty")]
    #[tokio::test]
    async fn feed_join_panic_emits_origin_then_one_recorded_terminal_and_drops_guard() {
        let recorder = Arc::new(RecordingCanonicalLog::new(
            ServiceName::new("feed").unwrap(),
        ));
        let store = Arc::new(
            crate::SqliteScheduleStore::open_in_memory(crate::ScheduleLimits::default()).unwrap(),
        );
        let clock = Arc::new(crate::SystemClock);
        let scheduler = crate::Scheduler::new_with_logger(
            store.clone(),
            clock.clone(),
            Arc::new(crate::SystemJitter),
            recorder.clone(),
        );
        let row = scheduler
            .start(
                crate::ScheduleSpec::new(
                    "feed-panic",
                    "* * * * *",
                    std::time::Duration::from_secs(1),
                    serde_json::json!({}),
                )
                .unwrap(),
            )
            .unwrap();
        let coord = Arc::new(RunCoord::with_logger(recorder.clone()));
        let (_, shutdown) = tokio::sync::oneshot::channel();
        let mut worker = Worker::new(
            store.clone(),
            clock,
            Arc::new(Idle),
            MissedTickPolicy::Coalesce,
            coord.clone(),
            std::time::Duration::from_secs(30),
            shutdown,
            Arc::new(crate::MontyHostRegistry::new()),
        );
        worker.slots.push(Slot {
            schedule: row.clone(),
            next: None,
            run: None,
            queued: Queued::None,
            last_skipped: None,
        });
        let guard = MontyFeedGuard::enter(&coord);
        let result = tokio::spawn(async {
            panic!("feed panic");
            #[allow(unreachable_code)]
            crate::monty_run::FeedDone {
                session: None,
                result: crate::monty_run::FeedResult::Continue,
            }
        })
        .await;
        recorder.clear();
        assert!(
            worker
                .monty_feed_from_join(0, result, Some(guard))
                .await
                .is_none()
        );
        assert_eq!(coord.feeds.load(std::sync::atomic::Ordering::Acquire), 0);
        let events = recorder.events();
        assert_eq!(
            events.iter().map(|e| e.action.as_str()).collect::<Vec<_>>(),
            ["feed.join_failed", "run.finished"]
        );
        assert_eq!(
            events[0].diagnostic.as_ref().unwrap().code.as_str(),
            "FEED_JOIN_FAILED"
        );
        assert_eq!(
            events[1].diagnostic.as_ref().unwrap().code.as_str(),
            "JOB_FAILED"
        );
        let persisted = store.get(row.id).unwrap().unwrap().last_outcome.unwrap();
        assert_eq!(
            persisted.failure_parts().unwrap().code,
            Some("FEED_JOIN_FAILED")
        );
    }
    struct Idle;
    #[async_trait::async_trait]
    impl crate::Job for Idle {
        async fn should_stop(&self, _: &serde_json::Value) -> StopDecision {
            StopDecision::Continue
        }
        async fn run(&self, _: &serde_json::Value, _: std::time::Instant) -> RunOutcome {
            RunOutcome::Ok
        }
    }
}
