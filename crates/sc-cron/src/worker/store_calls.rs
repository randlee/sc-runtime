//! Worker store scopes and persistence reconciliation policy.
use super::*;

impl<S: ScheduleStore + 'static, C: Clock + 'static> Worker<S, C> {
    pub(super) async fn store_call<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&S) -> Result<T, CronError> + Send + 'static,
    ) -> Result<T, CronError> {
        let store = Arc::clone(&self.store);
        let scoped = if self.cooperative {
            let controller = self.controller.as_ref().ok_or_else(|| {
                crate::consumer_contract::unavailable("operation_controller_missing")
            })?;
            let root = crate::StoreOperationContext::root(Arc::clone(controller));
            let context = if self.stopping { root.terminal() } else { root };
            if let Err(error) = context.check() {
                if context.cleanup {
                    controller.record_cleanup(crate::TransactionOutcome {
                        admission: crate::CommitAdmission::NotAttempted,
                        persistence: crate::PersistenceOutcome::NoMutationAttempted,
                        completed_after_cutoff: true,
                    });
                }
                return Err(error);
            }
            let guard = controller.register(context.cleanup, context.external_deadline(), false)?;
            Some((context, guard))
        } else {
            None
        };
        if let Some((context, guard)) = scoped {
            let controller = Arc::clone(&context.state);
            return controller
                .run(guard, move || {
                    store.with_operation_context(&context, operation)
                })
                .await;
        }
        crate::scheduler::blocking(move || operation(store.as_ref())).await
    }

    pub(super) fn persistence_requires_reconciliation(&self, error: &CronError) -> bool {
        self.cooperative
            && matches!(error, CronError::Store(error) if error.kind() == crate::StoreErrorKind::Indeterminate || error.transaction_outcome().is_some_and(|outcome| matches!(outcome.persistence, crate::PersistenceOutcome::Committed | crate::PersistenceOutcome::Indeterminate)))
    }
}
