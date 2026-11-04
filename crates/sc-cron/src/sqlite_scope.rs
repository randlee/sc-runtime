//! SQLite-specific live scope, callbacks, and known-baseline restoration.
use crate::{
    CommitAdmission, CronError, PersistenceOutcome, StoreError, StoreErrorKind,
    StoreOperationContext, TransactionOutcome,
};
use rusqlite::Connection;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use std::time::Duration;

#[derive(Clone)]
struct Scope {
    context: StoreOperationContext,
    outcome: Arc<Mutex<TransactionOutcome>>,
}
thread_local! { static CURRENT: RefCell<Option<Scope>> = const { RefCell::new(None) }; }
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        CURRENT.with(|current| {
            current.replace(None);
        });
    }
}
fn current() -> Option<Scope> {
    CURRENT.with(|current| current.borrow().clone())
}
pub(super) fn check() -> Result<(), CronError> {
    current().map_or(Ok(()), |scope| scope.context.check())
}
fn update(scope: &Scope, f: impl FnOnce(&mut TransactionOutcome)) {
    f(&mut scope
        .outcome
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner));
}
pub(super) fn attempted() -> Result<(), CronError> {
    check()?;
    if let Some(scope) = current() {
        update(&scope, |outcome| {
            outcome.persistence = PersistenceOutcome::Indeterminate
        });
    }
    Ok(())
}
pub(super) fn committed() {
    if let Some(scope) = current() {
        update(&scope, |outcome| {
            outcome.persistence = PersistenceOutcome::Committed
        });
    }
}
/// Legacy baseline has a finite five-second busy wait; scoped callbacks use live state.
pub(super) fn baseline_busy(attempt: i32) -> bool {
    if attempt >= 5000 {
        return false;
    }
    std::thread::park_timeout(Duration::from_millis(1));
    true
}
fn scoped_busy(attempt: i32) -> bool {
    #[cfg(test)]
    observe(TestStage::Busy);
    if attempt >= 5000 || check().is_err() {
        return false;
    }
    std::thread::park_timeout(Duration::from_millis(1));
    check().is_ok()
}

pub(super) fn with_provider<S, T>(
    context: &StoreOperationContext,
    provider: &S,
    operation: impl FnOnce(&S) -> Result<T, CronError>,
) -> Result<T, CronError> {
    with_scope(context, || operation(provider))
}

pub(super) fn with_scope<T>(
    context: &StoreOperationContext,
    operation: impl FnOnce() -> Result<T, CronError>,
) -> Result<T, CronError> {
    context.check()?;
    if current().is_some() {
        return Err(crate::consumer_contract::unavailable(
            "operation_context_nested",
        ));
    }
    let scope = Scope {
        context: context.clone(),
        outcome: Arc::new(Mutex::new(TransactionOutcome {
            admission: CommitAdmission::NotAttempted,
            persistence: PersistenceOutcome::NoMutationAttempted,
            completed_after_cutoff: false,
        })),
    };
    CURRENT.with(|current| current.replace(Some(scope.clone())));
    let _reset = Reset;
    let result = operation();
    let check = context.check();
    let mut outcome = *scope
        .outcome
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    outcome.completed_after_cutoff = context
        .deadline()
        .is_some_and(|cutoff| std::time::Instant::now() >= cutoff);
    if outcome.admission == CommitAdmission::Admitted
        && outcome.persistence == PersistenceOutcome::NoMutationAttempted
    {
        outcome.persistence = PersistenceOutcome::Indeterminate;
    }
    if context.cleanup {
        context.state.record_cleanup(outcome);
    }
    let failure = match check {
        Err(error) => Some(error),
        Ok(()) => result.as_ref().err().cloned(),
    };
    if let Some(error) = failure {
        let (kind, reason) = match error {
            CronError::Store(ref error) => (
                error.kind(),
                match error.kind() {
                    StoreErrorKind::Cancelled => "operation_cancelled",
                    StoreErrorKind::DeadlineExceeded => "operation_deadline_exceeded",
                    StoreErrorKind::Indeterminate => "operation_persistence_indeterminate",
                    StoreErrorKind::Busy => "operation_store_busy",
                    StoreErrorKind::Corrupt => "operation_store_corrupt",
                    _ => "operation_store_failed",
                },
            ),
            _ => return result,
        };
        let kind = if outcome.persistence == PersistenceOutcome::Indeterminate {
            StoreErrorKind::Indeterminate
        } else if outcome.completed_after_cutoff
            && outcome.persistence == PersistenceOutcome::Committed
        {
            StoreErrorKind::DeadlineExceeded
        } else {
            kind
        };
        let reason = if kind == StoreErrorKind::DeadlineExceeded {
            "operation_deadline_exceeded"
        } else {
            reason
        };
        return Err(StoreError::new(kind, reason)
            .with_transaction_outcome(outcome)
            .into());
    }
    result
}

pub(super) struct Guard<'a> {
    connection: MutexGuard<'a, Connection>,
    scope: Option<Scope>,
    interrupt: Option<Arc<rusqlite::InterruptHandle>>,
    quarantined: &'a std::sync::atomic::AtomicBool,
}
impl Deref for Guard<'_> {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        &self.connection
    }
}
impl DerefMut for Guard<'_> {
    fn deref_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if let Some(scope) = &self.scope {
            let uncertain = scope
                .outcome
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .persistence
                == PersistenceOutcome::Indeterminate;
            if !self.connection.is_autocommit() || uncertain {
                self.quarantined
                    .store(true, std::sync::atomic::Ordering::Release);
                return;
            }
            let _ = self.connection.progress_handler(0, None::<fn() -> bool>);
            let _ = self.connection.commit_hook(None::<fn() -> bool>);
            let _ = self.connection.rollback_hook(None::<fn()>);
            let _ = self.connection.busy_handler(Some(baseline_busy));
            if let Some(handle) = &self.interrupt {
                scope.context.state.remove_interrupt(handle);
            }
        }
    }
}
pub(super) fn lock<'a>(
    connection: &'a Mutex<Connection>,
    quarantined: &'a std::sync::atomic::AtomicBool,
) -> Result<Guard<'a>, CronError> {
    if quarantined.load(std::sync::atomic::Ordering::Acquire) {
        return Err(StoreError::new(
            StoreErrorKind::Indeterminate,
            "operation_connection_quarantined",
        )
        .into());
    }
    let scope = current();
    let connection = if scope.is_some() {
        loop {
            check()?;
            if quarantined.load(std::sync::atomic::Ordering::Acquire) {
                return Err(StoreError::new(
                    StoreErrorKind::Indeterminate,
                    "operation_connection_quarantined",
                )
                .into());
            }
            match connection.try_lock() {
                Ok(guard) => break guard,
                Err(TryLockError::Poisoned(error)) => break error.into_inner(),
                Err(TryLockError::WouldBlock) => {
                    #[cfg(test)]
                    observe(TestStage::LockWait);
                    std::thread::park_timeout(Duration::from_millis(1))
                }
            }
        }
    } else {
        connection
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    };
    if quarantined.load(std::sync::atomic::Ordering::Acquire) {
        return Err(StoreError::new(
            StoreErrorKind::Indeterminate,
            "operation_connection_quarantined",
        )
        .into());
    }
    let mut guard = Guard {
        connection,
        scope: scope.clone(),
        interrupt: None,
        quarantined,
    };
    if let Some(scope) = scope {
        install_callbacks(&mut guard, scope)?;
    }
    Ok(guard)
}
fn install_callbacks(guard: &mut Guard<'_>, scope: Scope) -> Result<(), CronError> {
    check()?;
    let handle = Arc::new(guard.get_interrupt_handle());
    scope
        .context
        .state
        .add_interrupt(Arc::clone(&handle), scope.context.cleanup);
    guard.interrupt = Some(handle);
    guard
        .busy_handler(Some(scoped_busy))
        .map_err(callback_error)?;
    let progress = scope.context.clone();
    guard
        .progress_handler(
            64,
            Some(move || {
                #[cfg(test)]
                observe(TestStage::Progress);
                progress.check().is_err()
            }),
        )
        .map_err(callback_error)?;
    let commit = scope.clone();
    guard
        .commit_hook(Some(move || {
            #[cfg(test)]
            test_gate(false);
            let admitted = commit.context.check().is_ok()
                && commit
                    .context
                    .state
                    .admit_commit(commit.context.cleanup, commit.context.external_deadline());
            update(&commit, |outcome| {
                outcome.admission = if admitted {
                    CommitAdmission::Admitted
                } else {
                    CommitAdmission::Vetoed
                }
            });
            #[cfg(test)]
            test_gate(true);
            !admitted
        }))
        .map_err(callback_error)?;
    guard
        .rollback_hook(Some(move || {
            update(&scope, |outcome| {
                outcome.persistence = PersistenceOutcome::RolledBack
            })
        }))
        .map_err(callback_error)?;
    Ok(())
}

fn callback_error(_error: rusqlite::Error) -> CronError {
    StoreError::new(StoreErrorKind::Unavailable, "operation_callback_failed").into()
}

#[cfg(test)]
type CommitGate = Option<Box<dyn Fn(bool)>>;
#[cfg(test)]
thread_local! { static COMMIT_GATE: RefCell<CommitGate> = const { RefCell::new(None) }; }
#[cfg(test)]
pub(super) fn set_commit_gate(gate: impl Fn(bool) + 'static) {
    COMMIT_GATE.with(|current| current.replace(Some(Box::new(gate))));
}
#[cfg(test)]
fn test_gate(after_admission: bool) {
    COMMIT_GATE.with(|gate| {
        if let Some(gate) = gate.borrow().as_ref() {
            gate(after_admission);
        }
    });
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq)]
pub(super) enum TestStage {
    LockWait,
    Busy,
    Progress,
    Decode,
}
#[cfg(test)]
type ObserveGate = Option<Box<dyn Fn(TestStage)>>;
#[cfg(test)]
thread_local! { static OBSERVE_GATE: RefCell<ObserveGate> = const { RefCell::new(None) }; }
#[cfg(test)]
pub(super) fn set_observer(gate: impl Fn(TestStage) + 'static) {
    OBSERVE_GATE.with(|current| current.replace(Some(Box::new(gate))));
}
#[cfg(test)]
pub(super) fn observe(stage: TestStage) {
    OBSERVE_GATE.with(|gate| {
        if let Some(gate) = gate.borrow().as_ref() {
            gate(stage);
        }
    });
}

#[cfg(test)]
pub(super) fn lose_result() {
    if let Some(scope) = current() {
        update(&scope, |outcome| {
            outcome.persistence = PersistenceOutcome::Indeterminate
        });
    }
}

#[cfg(test)]
pub(super) fn has_committed() -> bool {
    current().is_some_and(|scope| {
        scope
            .outcome
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .persistence
            == PersistenceOutcome::Committed
    })
}
