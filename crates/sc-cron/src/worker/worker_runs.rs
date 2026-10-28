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
                &self.slots[index].schedule,
                "timed_out",
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
                &self.slots[index].schedule,
                "timed_out",
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
        let handle = tokio::spawn(async move {
            crate::monty_run::drive_feed(
                clock.as_ref(),
                crate::monty_run::FeedLaunch {
                    session,
                    source,
                    phase,
                    generation,
                    name: schedule.spec.name.clone(),
                    payload: schedule.spec.payload.clone(),
                    timeout: schedule.spec.timeout,
                    deadline,
                },
            )
            .await
        });
        let Some(slot) = self.slots.get_mut(index) else {
            drop(guard);
            return;
        };
        log(&slot.schedule, "launched", "monty feed started");
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
                    log(&slot.schedule, "cancelled", &error.to_string());
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
