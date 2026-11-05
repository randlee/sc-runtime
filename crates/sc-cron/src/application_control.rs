//! Application-owned context control; no framework worker or native work registry.
use crate::{CronError, ShutdownBudget, ShutdownStatus, StoreOperationContext};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

/// Cloneable control for application-owned native operation providers.
///
/// The provider owns admission, native work, cleanup and physical joins.
#[derive(Clone, Debug)]
pub struct WorkerStopControl {
    context: StoreOperationContext,
    runtime: tokio::runtime::Handle,
    status: watch::Sender<ShutdownStatus>,
}
impl WorkerStopControl {
    /// Creates live control inside the Tokio runtime that will own notifications.
    ///
    /// # Errors
    /// Returns a typed unavailable error outside that runtime.
    pub fn new() -> Result<Self, CronError> {
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| crate::consumer_contract::unavailable("worker_runtime_unavailable"))?;
        Ok(Self {
            context: StoreOperationContext {
                state: Arc::new(Mutex::new(Default::default())),
                deadline: None,
            },
            runtime,
            status: watch::channel(ShutdownStatus::Running).0,
        })
    }
    /// Returns the shared scope without creating or starting a worker.
    #[must_use]
    pub fn operation_context(&self) -> StoreOperationContext {
        self.context.clone()
    }
    /// Subscribes to live stop/expiry and acknowledged application completion.
    #[must_use]
    pub fn subscribe_status(&self) -> watch::Receiver<ShutdownStatus> {
        self.status.subscribe()
    }
    fn publish(&self, status: ShutdownStatus) {
        fn rank(s: ShutdownStatus) -> u8 {
            match s {
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
    /// Publishes stop intent and shortens absolute deadlines, from any thread.
    ///
    /// The application must close its own intake and physically join its own
    /// work before calling [`Self::finish_application_shutdown`].
    ///
    /// # Errors
    /// Rejects cleanup before grace. The creating runtime must remain alive.
    pub fn request_stop(&self, budget: ShutdownBudget) -> Result<(), CronError> {
        ShutdownBudget::new(budget.grace, budget.cleanup)?;
        let now = std::time::Instant::now();
        {
            let mut state = self
                .context
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let budget = if let Some(old) = state.budget {
                state.cancelled = true;
                ShutdownBudget {
                    grace: old.grace.min(budget.grace).min(now),
                    cleanup: old.cleanup.min(budget.cleanup),
                }
            } else {
                budget
            };
            state.stop_requested = true;
            state.cutoff = Some(budget.grace);
            state.budget = Some(budget);
        }
        self.publish(ShutdownStatus::Draining);
        if self.context.deadline().is_some_and(|at| now >= at) {
            self.publish(ShutdownStatus::DeadlineExceeded);
        }
        let owner = self.clone();
        self.runtime.spawn(async move {
            let mut status = owner.subscribe_status();
            loop {
                if *status.borrow() == ShutdownStatus::Joined {
                    return;
                }
                let budget = owner
                    .context
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .budget;
                let Some(budget) = budget else {
                    return;
                };
                let now = std::time::Instant::now();
                if now >= budget.grace {
                    owner.publish(ShutdownStatus::DeadlineExceeded);
                }
                let cutoff = if now < budget.grace {
                    budget.grace
                } else if now < budget.cleanup {
                    budget.cleanup
                } else {
                    return;
                };
                tokio::select! {
                    _=tokio::time::timeout_at(cutoff.into(),std::future::pending::<()>())=>{},
                    _=status.changed()=>{}
                }
            }
        });
        Ok(())
    }
    /// Cancels ordinary application work without extending an existing cleanup cutoff.
    pub fn cancel_now(&self) {
        let now = std::time::Instant::now();
        let _ = self.request_stop(ShutdownBudget {
            grace: now,
            cleanup: now,
        });
        self.context
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .cancelled = true;
    }
    /// Acknowledges that the application has closed intake and physically joined
    /// every custom operation and cleanup. This control cannot verify untracked
    /// provider work and owns no framework worker or native registration.
    ///
    /// # Errors
    /// Refuses acknowledgement before stop is requested.
    pub fn finish_application_shutdown(&self) -> Result<(), CronError> {
        if !self.context.stop_requested() {
            return Err(crate::consumer_contract::unavailable(
                "operation_completion_owner_required",
            ));
        }
        self.cancel_now();
        self.publish(ShutdownStatus::Joined);
        Ok(())
    }
}
