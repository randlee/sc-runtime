//! Shared admission and physical completion ownership for cooperative work.
use crate::{CronError, ShutdownBudget, ShutdownStatus};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use tokio::sync::{Notify, watch};

struct State {
    budget: Option<ShutdownBudget>,
    cancelled: bool,
    started: bool,
    capacity: u32,
    commands: u32,
    active: usize,
    interrupts: Vec<(Arc<rusqlite::InterruptHandle>, bool)>,
    cleanup: Option<crate::TransactionOutcome>,
    ordinary_completed: Option<Instant>,
    cleanup_completed: Option<Instant>,
    worker_finished: bool,
}

pub(crate) struct Controller {
    state: Arc<Mutex<State>>,
    pub(crate) changed: Arc<Notify>,
    tasks: Mutex<Vec<tokio::task::JoinHandle<()>>>,
    status: watch::Sender<ShutdownStatus>,
}
impl std::fmt::Debug for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Controller")
            .field("active", &self.lock().active)
            .finish_non_exhaustive()
    }
}
impl Controller {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                budget: None,
                cancelled: false,
                started: false,
                capacity: 64,
                commands: 0,
                active: 0,
                interrupts: Vec::new(),
                cleanup: None,
                ordinary_completed: None,
                cleanup_completed: None,
                worker_finished: false,
            })),
            changed: Arc::new(Notify::new()),
            tasks: Mutex::new(Vec::new()),
            status: watch::channel(ShutdownStatus::Running).0,
        }
    }
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    pub(crate) fn configure(&self, capacity: u32) -> Result<(), CronError> {
        let mut state = self.lock();
        if !(1..=65536).contains(&capacity) || state.started {
            return Err(CronError::InvalidSpec(
                "command capacity must be 1..=65536 and configured before first operation".into(),
            ));
        }
        state.capacity = capacity;
        Ok(())
    }
    pub(crate) fn mark_started(&self) {
        self.lock().started = true;
    }
    pub(crate) fn stopping(&self) -> bool {
        self.lock().budget.is_some()
    }
    pub(crate) fn cancelled(&self, cleanup: bool) -> bool {
        let state = self.lock();
        !cleanup && state.cancelled
    }
    pub(crate) fn cutoff(&self, cleanup: bool) -> Option<Instant> {
        self.lock()
            .budget
            .map(|b| if cleanup { b.cleanup } else { b.grace })
    }
    pub(crate) fn budget(&self) -> Option<ShutdownBudget> {
        self.lock().budget
    }
    pub(crate) fn subscribe(&self) -> watch::Receiver<ShutdownStatus> {
        self.status.subscribe()
    }
    pub(crate) fn status(&self, status: ShutdownStatus) {
        fn rank(status: ShutdownStatus) -> u8 {
            match status {
                ShutdownStatus::Running => 0,
                ShutdownStatus::Draining => 1,
                ShutdownStatus::CleaningUp => 2,
                ShutdownStatus::DeadlineExceeded => 3,
                ShutdownStatus::Joined => 4,
            }
        }
        self.status.send_if_modified(|current| {
            if rank(status) > rank(*current) {
                *current = status;
                true
            } else {
                false
            }
        });
    }
    pub(crate) fn stop(self: &Arc<Self>, budget: ShutdownBudget) -> Result<(), CronError> {
        ShutdownBudget::new(budget.grace, budget.cleanup)?;
        {
            let mut state = self.lock();
            if let Some(old) = state.budget {
                state.cancelled = true;
                state.budget = Some(ShutdownBudget {
                    grace: old.grace.min(budget.grace).min(Instant::now()),
                    cleanup: old.cleanup.min(budget.cleanup),
                });
            } else {
                state.budget = Some(budget);
            }
            self.status(ShutdownStatus::Draining);
        }
        self.changed.notify_waiters();
        self.interrupt_if_expired();
        let owner = Arc::clone(self);
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                let mut lifecycle = owner.subscribe();
                loop {
                    if *lifecycle.borrow() == ShutdownStatus::Joined {
                        return;
                    }
                    let Some(budget) = owner.budget() else {
                        return;
                    };
                    let now = Instant::now();
                    owner.interrupt_if_expired();
                    let cutoff = if now < budget.grace {
                        budget.grace
                    } else if now < budget.cleanup {
                        budget.cleanup
                    } else {
                        return;
                    };
                    tokio::select! {
                        () = tokio::time::sleep_until(cutoff.into()) => {},
                        _ = lifecycle.changed() => {}
                    }
                }
            });
        }
        Ok(())
    }
    fn interrupt_if_expired(&self) {
        let handles = {
            let state = self.lock();
            if state.worker_finished && state.active == 0 {
                return;
            }
            if !state.cancelled && state.budget.is_none_or(|b| Instant::now() < b.grace) {
                return;
            }
            state
                .interrupts
                .iter()
                .filter(|(_, cleanup)| {
                    if *cleanup {
                        state.budget.is_some_and(|b| Instant::now() >= b.cleanup)
                    } else {
                        state.cancelled || state.budget.is_some_and(|b| Instant::now() >= b.grace)
                    }
                })
                .map(|(handle, _)| Arc::clone(handle))
                .collect::<Vec<_>>()
        };
        self.status(ShutdownStatus::DeadlineExceeded);
        for handle in handles {
            handle.interrupt();
        }
        self.changed.notify_waiters();
    }
    pub(crate) fn cancel(self: &Arc<Self>) {
        let now = Instant::now();
        let _ = self.stop(ShutdownBudget {
            grace: now,
            cleanup: now,
        });
        self.lock().cancelled = true;
        self.interrupt_if_expired();
    }
    pub(crate) fn register(
        self: &Arc<Self>,
        cleanup: bool,
        external: Option<Instant>,
        command: bool,
    ) -> Result<PhysicalGuard, CronError> {
        let mut state = self.lock();
        if external.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(crate::StoreError::new(
                crate::StoreErrorKind::DeadlineExceeded,
                "operation_deadline_exceeded",
            )
            .into());
        }
        if state.budget.is_some() && !cleanup {
            return Err(refusal("operation_stopping"));
        }
        if cleanup
            && state
                .budget
                .is_some_and(|budget| Instant::now() >= budget.cleanup)
        {
            return Err(crate::StoreError::new(
                crate::StoreErrorKind::DeadlineExceeded,
                "operation_deadline_exceeded",
            )
            .into());
        }
        if command && state.commands >= state.capacity {
            return Err(refusal("command_capacity_exhausted"));
        }
        state.started = true;
        state.active += 1;
        if command {
            state.commands += 1;
        }
        Ok(PhysicalGuard {
            state: Arc::clone(&self.state),
            changed: Arc::clone(&self.changed),
            command,
            cleanup,
        })
    }
    /// Serializes final admission with stop/shortening, without holding across native COMMIT.
    pub(crate) fn admit_commit(&self, cleanup: bool, external: Option<Instant>) -> bool {
        let state = self.lock();
        let cutoff = state
            .budget
            .map(|b| if cleanup { b.cleanup } else { b.grace });
        if !cleanup && state.cancelled {
            return false;
        }
        if cutoff
            .into_iter()
            .chain(external)
            .any(|cutoff| Instant::now() >= cutoff)
        {
            return false;
        }
        // Local deadlines and foreign parents are checked before taking this controller lock.
        true
    }
    pub(crate) fn admit_launch(&self) -> bool {
        self.lock().budget.is_none()
    }
    pub(crate) fn record_cleanup(&self, outcome: crate::TransactionOutcome) {
        let mut state = self.lock();
        if outcome.persistence != crate::PersistenceOutcome::NoMutationAttempted
            || outcome.completed_after_cutoff
        {
            let mut observed = outcome;
            if let Some(previous) = state.cleanup {
                observed.completed_after_cutoff |= previous.completed_after_cutoff;
                if previous.persistence == crate::PersistenceOutcome::Indeterminate
                    || observed.persistence == crate::PersistenceOutcome::NoMutationAttempted
                {
                    observed.admission = previous.admission;
                    observed.persistence = previous.persistence;
                }
            }
            state.cleanup = Some(observed);
        }
    }
    pub(crate) fn note_ordinary_completion(&self) {
        self.lock().ordinary_completed = Some(Instant::now());
    }
    pub(crate) fn worker_finished(&self) {
        self.lock().worker_finished = true;
        self.changed.notify_waiters();
    }
    pub(crate) fn completion_times(&self) -> (Option<Instant>, Option<Instant>) {
        let state = self.lock();
        (state.ordinary_completed, state.cleanup_completed)
    }
    pub(crate) fn cleanup_outcome(&self) -> Option<crate::TransactionOutcome> {
        self.lock().cleanup
    }
    pub(crate) fn add_interrupt(&self, handle: Arc<rusqlite::InterruptHandle>, cleanup: bool) {
        self.lock().interrupts.push((handle, cleanup));
        self.interrupt_if_expired();
    }
    pub(crate) fn remove_interrupt(&self, handle: &Arc<rusqlite::InterruptHandle>) {
        self.lock()
            .interrupts
            .retain(|(item, _)| !Arc::ptr_eq(item, handle));
    }
    #[cfg(test)]
    pub(crate) fn counts(&self) -> (usize, u32) {
        let state = self.lock();
        (state.active, state.commands)
    }
    pub(crate) async fn run<T: Send + 'static>(
        &self,
        guard: PhysicalGuard,
        operation: impl FnOnce() -> Result<T, CronError> + Send + 'static,
    ) -> Result<T, CronError> {
        let (send, receive) = tokio::sync::oneshot::channel();
        let task = tokio::task::spawn_blocking(operation);
        let owner = tokio::spawn(async move {
            let result = task
                .await
                .map_err(|error| {
                    crate::StoreError::new(crate::StoreErrorKind::Other, "store_task_failed")
                        .with_source(error)
                        .into()
                })
                .and_then(|result| result);
            let _ = send.send(result);
            drop(guard);
        });
        {
            let mut tasks = self
                .tasks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            tasks.retain(|task| !task.is_finished());
            tasks.push(owner);
        }
        receive.await.map_err(|_| refusal("store_owner_failed"))?
    }
    pub(crate) async fn join(&self) {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let tasks = std::mem::take(
                &mut *self
                    .tasks
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
            for task in tasks {
                let _ = task.await;
            }
            if self.lock().active == 0 {
                return;
            }
            notified.await;
        }
    }
}

pub(crate) struct PhysicalGuard {
    state: Arc<Mutex<State>>,
    changed: Arc<Notify>,
    command: bool,
    cleanup: bool,
}
impl Drop for PhysicalGuard {
    fn drop(&mut self) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.active -= 1;
            if self.cleanup {
                state.cleanup_completed = Some(Instant::now());
            } else {
                state.ordinary_completed = Some(Instant::now());
            }
            if self.command {
                state.commands -= 1;
            }
        }
        self.changed.notify_waiters();
    }
}

fn refusal(reason: &'static str) -> CronError {
    crate::StoreError::new(crate::StoreErrorKind::Unavailable, reason)
        .with_transaction_outcome(crate::TransactionOutcome {
            admission: crate::CommitAdmission::NotAttempted,
            persistence: crate::PersistenceOutcome::NoMutationAttempted,
            completed_after_cutoff: false,
        })
        .into()
}
