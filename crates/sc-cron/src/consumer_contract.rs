//! Native consumer operation contracts and shared cancellation scope.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use crate::{
    CronError, Generation, Schedule, ScheduleId, ScheduleName, ScheduleSpec, StoreError,
    StoreErrorKind,
};

/// Shared live operation state; child deadlines never extend the root scope.
#[derive(Clone, Debug)]
pub struct StoreOperationContext {
    pub(crate) state: Arc<crate::operation::Controller>,
    deadline: Option<Instant>,
    pub(crate) cleanup: bool,
    parents: Vec<StoreOperationContext>,
}

impl StoreOperationContext {
    /// Returns the earliest local or shared operation cutoff.
    #[must_use]
    pub fn deadline(&self) -> Option<Instant> {
        let shared = self.state.cutoff(self.cleanup);
        self.parents
            .iter()
            .filter_map(Self::deadline)
            .chain(self.deadline)
            .chain(shared)
            .min()
    }

    /// Reports the current shared stop request.
    #[must_use]
    pub fn stop_requested(&self) -> bool {
        self.state.stopping()
    }

    /// Checks live cancellation and the real monotonic deadline.
    ///
    /// # Errors
    /// Returns a value-free unavailable error after cancellation or expiry.
    pub fn check(&self) -> Result<(), CronError> {
        if self.state.cancelled(self.cleanup) {
            return Err(StoreError::new(StoreErrorKind::Cancelled, "operation_cancelled").into());
        }
        for parent in &self.parents {
            parent.check()?;
        }
        if self
            .deadline()
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(StoreError::new(
                StoreErrorKind::DeadlineExceeded,
                "operation_deadline_exceeded",
            )
            .into());
        }
        Ok(())
    }

    /// Derives a child retaining the same shared cancellation state.
    #[must_use]
    pub fn with_deadline(&self, deadline: Instant) -> Self {
        Self {
            state: Arc::clone(&self.state),
            cleanup: self.cleanup,
            parents: self.parents.clone(),
            deadline: Some(
                self.deadline
                    .map_or(deadline, |current| current.min(deadline)),
            ),
        }
    }
    pub(crate) fn external_deadline(&self) -> Option<Instant> {
        self.parents
            .iter()
            .filter_map(Self::deadline)
            .chain(self.deadline)
            .min()
    }

    pub(crate) fn root(state: Arc<crate::operation::Controller>) -> Self {
        Self {
            state,
            deadline: None,
            cleanup: false,
            parents: Vec::new(),
        }
    }

    pub(crate) fn bind(&self, state: Arc<crate::operation::Controller>) -> Self {
        let mut bound = Self::root(state);
        if !Arc::ptr_eq(&bound.state, &self.state) {
            bound.parents.push(self.clone());
        }
        bound.deadline = self.deadline;
        bound
    }

    pub(crate) fn terminal(&self) -> Self {
        Self {
            cleanup: true,
            deadline: None,
            parents: Vec::new(),
            ..self.clone()
        }
    }
}

/// Creates a real worker-free scope for adapter integration tests.
#[cfg(feature = "test-utils")]
#[must_use]
pub fn operation_context_probe(deadline: Instant) -> StoreOperationContext {
    StoreOperationContext::root(Arc::new(crate::operation::Controller::new()))
        .with_deadline(deadline)
}

pub(crate) fn unavailable(reason: &'static str) -> CronError {
    StoreError::new(StoreErrorKind::Unavailable, reason)
        .with_transaction_outcome(TransactionOutcome {
            admission: CommitAdmission::NotAttempted,
            persistence: PersistenceOutcome::NoMutationAttempted,
            completed_after_cutoff: false,
        })
        .into()
}

/// Cooperative store capability; legacy providers remain caller-owned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreOperationSupport {
    /// Legacy methods provide no shared cancellation or commit admission.
    Legacy,
    /// Provider propagates the live context and owns physical completion.
    Cooperative,
}

/// Absolute ordinary and terminal-cleanup cutoffs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShutdownBudget {
    /// Ordinary operation commit-admission cutoff G.
    pub(crate) grace: Instant,
    /// Restricted cleanup commit-admission cutoff K, at least G.
    pub(crate) cleanup: Instant,
}

impl ShutdownBudget {
    /// Returns the ordinary-operation commit cutoff G.
    #[must_use]
    pub const fn grace(&self) -> Instant {
        self.grace
    }
    /// Returns the terminal-cleanup commit cutoff K.
    #[must_use]
    pub const fn cleanup(&self) -> Instant {
        self.cleanup
    }
    /// Checks that cleanup does not precede ordinary drain.
    ///
    /// # Errors
    /// Returns invalid-spec when K precedes G.
    pub fn new(grace: Instant, cleanup: Instant) -> Result<Self, CronError> {
        if cleanup < grace {
            return Err(CronError::InvalidSpec(
                "cleanup cutoff precedes grace cutoff".into(),
            ));
        }
        Ok(Self { grace, cleanup })
    }
}

/// Final transaction hook admission decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitAdmission {
    /// No final commit decision was attempted.
    NotAttempted,
    /// The live controller refused the commit.
    Vetoed,
    /// The live controller admitted the commit before its cutoff.
    Admitted,
}

/// Observed physical transaction persistence, independently of admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceOutcome {
    /// No mutation was attempted.
    NoMutationAttempted,
    /// The transaction is known committed, possibly late.
    Committed,
    /// Rollback is known complete.
    RolledBack,
    /// Available evidence cannot establish commit or rollback.
    Indeterminate,
}

/// Honest persistence outcome after owned physical completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionOutcome {
    /// Final admission decision.
    pub admission: CommitAdmission,
    /// Observed physical persistence.
    pub persistence: PersistenceOutcome,
    /// Physical completion occurred after its applicable cutoff.
    pub completed_after_cutoff: bool,
}

/// Completed worker shutdown and terminal cleanup observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShutdownReport {
    /// Ordinary drain exceeded G.
    pub grace_exceeded: bool,
    /// Terminal cleanup exceeded K.
    pub cleanup_exceeded: bool,
    /// Physical terminal cleanup result.
    pub cleanup: TransactionOutcome,
}

/// Lifecycle state visible independently of the physical join owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShutdownStatus {
    /// Worker accepts ordinary work.
    Running,
    /// Ordinary intake is closed; admitted operations drain.
    Draining,
    /// Only restricted terminal cleanup may be admitted.
    CleaningUp,
    /// Physical work remains owned after a deadline.
    DeadlineExceeded,
    /// All owned physical work has completed.
    Joined,
}

/// Selection of current schedule rows, without a separate history table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleSelection {
    /// Rows that have not ended.
    Active,
    /// Ended rows only.
    Ended,
    /// Active and ended rows.
    All,
}

/// Ascending-id page query; limits must be within 1 through 1000.
#[derive(Clone, Debug)]
pub struct ScheduleQuery {
    /// Lifecycle selection.
    pub selection: ScheduleSelection,
    /// Optional exact schedule name.
    pub name: Option<ScheduleName>,
    /// Exclusive ascending-id cursor.
    pub after_id: Option<ScheduleId>,
    /// Maximum returned rows.
    pub limit: u32,
}

impl Default for ScheduleQuery {
    fn default() -> Self {
        Self {
            selection: ScheduleSelection::Active,
            name: None,
            after_id: None,
            limit: 100,
        }
    }
}

/// One bounded page and its exclusive continuation cursor.
#[derive(Clone, Debug)]
pub struct SchedulePage {
    /// Current schedule rows in ascending id order.
    pub items: Vec<Schedule>,
    /// Cursor for the next page, absent when exhausted.
    pub next_after_id: Option<ScheduleId>,
}

/// Native request to end the named generation.
#[derive(Clone, Debug)]
pub struct EndSchedule {
    /// Exact schedule name.
    pub name: ScheduleName,
    /// Expected generation fence.
    pub generation: Generation,
    /// Application-supplied terminal reason.
    pub reason: String,
}

/// Successfully ended schedule identity.
#[derive(Clone, Debug)]
pub struct EndedSchedule {
    /// Persisted row identity.
    pub id: ScheduleId,
    /// Persisted generation fence.
    pub generation: Generation,
}

/// Object-safe asynchronous native operation result.
pub type CronOperationFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CronError>> + Send + 'a>>;

/// Replaceable application scheduling operations, without adapter wire types.
pub trait CronOperations: Send + Sync {
    /// Starts one schedule within the live context.
    fn start(
        &self,
        context: StoreOperationContext,
        spec: ScheduleSpec,
    ) -> CronOperationFuture<'_, Schedule>;
    /// Ends one schedule generation within the live context.
    fn end(
        &self,
        context: StoreOperationContext,
        request: EndSchedule,
    ) -> CronOperationFuture<'_, EndedSchedule>;
    /// Queries one bounded schedule page within the live context.
    fn query(
        &self,
        context: StoreOperationContext,
        query: ScheduleQuery,
    ) -> CronOperationFuture<'_, SchedulePage>;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn child_deadline_never_extends_shared_scope() {
        let deadline = Instant::now() + Duration::from_secs(60);
        let parent = StoreOperationContext::root(Arc::new(crate::operation::Controller::new()))
            .with_deadline(deadline);
        assert_eq!(
            parent
                .with_deadline(deadline + Duration::from_secs(60))
                .deadline(),
            Some(deadline)
        );
        let earlier = deadline - Duration::from_secs(30);
        assert_eq!(parent.with_deadline(earlier).deadline(), Some(earlier));
        parent.state.cancel();
        assert!(parent.with_deadline(earlier).stop_requested());
        assert!(parent.check().is_err());
    }

    #[test]
    fn rusqlite_hooks_and_interrupt_are_available() {
        let connection = rusqlite::Connection::open_in_memory().expect("database");
        let _interrupt = connection.get_interrupt_handle();
        connection
            .busy_handler(Some(|_| false))
            .expect("busy handler");
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let progress_stop = Arc::clone(&stop);
        connection
            .progress_handler(
                1,
                Some(move || progress_stop.load(std::sync::atomic::Ordering::Acquire)),
            )
            .expect("install progress handler");
        let veto = Arc::clone(&stop);
        connection
            .commit_hook(Some(move || {
                veto.load(std::sync::atomic::Ordering::Acquire)
            }))
            .expect("install commit hook");
        connection
            .execute_batch("CREATE TABLE probe (id INTEGER)")
            .expect("initial commit");
        stop.store(true, std::sync::atomic::Ordering::Release);
        connection
            .progress_handler(0, None::<fn() -> bool>)
            .expect("remove progress handler");
        assert!(
            connection
                .execute("INSERT INTO probe VALUES (1)", [])
                .is_err()
        );
        connection
            .commit_hook(None::<fn() -> bool>)
            .expect("remove commit hook");
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM probe", [], |row| row.get(0))
            .expect("readback");
        assert_eq!(count, 0);
    }
}
