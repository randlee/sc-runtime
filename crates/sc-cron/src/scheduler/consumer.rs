//! Native cooperative command admission and independently controlled physical shutdown.
use super::*;

impl<S: ScheduleStore + 'static, C: Clock + 'static, J: JitterSource + 'static> Scheduler<S, C, J> {
    /// Sets the finite shared command intake bound before any work starts.
    ///
    /// # Errors
    /// Rejects values outside 1..=65536 or configuration after work starts.
    pub fn with_command_capacity(self, capacity: u32) -> Result<Self, CronError> {
        self.core.controller.configure(capacity)?;
        Ok(self)
    }

    /// Starts a schedule through bounded, physically owned cooperative admission.
    ///
    /// # Errors
    /// Rejects unsupported stores, stopped/expired scopes and full intake before submission.
    pub async fn start_with_context(
        &self,
        spec: ScheduleSpec,
        context: crate::StoreOperationContext,
    ) -> Result<Schedule, CronError> {
        let core = Arc::clone(&self.core);
        self.command(context, move || core.start(spec)).await
    }

    /// Ends the exact generation and returns its terminal row.
    ///
    /// # Errors
    /// Returns fencing, admission or provider errors without replaying the mutation.
    pub async fn stop_with_context(
        &self,
        id: ScheduleId,
        generation: Generation,
        reason: String,
        context: crate::StoreOperationContext,
    ) -> Result<Schedule, CronError> {
        let core = Arc::clone(&self.core);
        self.command(context, move || {
            core.stop(id, generation, &reason)?;
            core.store
                .get(id)?
                .ok_or(CronError::Fenced { id, generation })
        })
        .await
    }

    /// Queries active, ended or all rows with bounded id continuation.
    ///
    /// # Errors
    /// Returns invalid query, admission or provider errors.
    pub async fn query_with_context(
        &self,
        query: crate::ScheduleQuery,
        context: crate::StoreOperationContext,
    ) -> Result<crate::SchedulePage, CronError> {
        let core = Arc::clone(&self.core);
        self.command(context, move || core.store.query(&query))
            .await
    }

    async fn command<T: Send + 'static>(
        &self,
        context: crate::StoreOperationContext,
        operation: impl FnOnce() -> Result<T, CronError> + Send + 'static,
    ) -> Result<T, CronError> {
        if self.core.store.operation_support() != crate::StoreOperationSupport::Cooperative {
            return Err(crate::consumer_contract::unavailable(
                "operation_context_unsupported",
            ));
        }
        let context = context.bind(Arc::clone(&self.core.controller));
        let guard = self.core.controller.register(&context, true)?;
        let store = Arc::clone(&self.core.store);
        // The permit lives inside the physical closure, independent of its transport waiter.
        self.core
            .controller
            .run(guard, move || {
                store.with_operation_context(&context, |_| operation())
            })
            .await
    }

    /// Starts a worker using the store's declared cooperative operation contract.
    ///
    /// # Errors
    /// Refuses legacy stores before spawning tasks or loading rows.
    pub fn spawn_worker_cooperative(&self, job: Arc<dyn Job>) -> Result<WorkerHandle, CronError> {
        if self.core.store.operation_support() != crate::StoreOperationSupport::Cooperative {
            return Err(crate::consumer_contract::unavailable(
                "operation_context_unsupported",
            ));
        }
        if self.core.controller.stopping() {
            return Err(crate::consumer_contract::unavailable("operation_stopping"));
        }
        Ok(self.spawn_worker_mode(job, true))
    }
}

impl WorkerHandle {
    /// Returns stop control independent of the physical join owner.
    #[must_use]
    pub fn stop_control(&self) -> WorkerStopControl {
        WorkerStopControl {
            controller: Arc::clone(&self.controller),
        }
    }

    /// Physically joins the worker and every admitted command closure.
    ///
    /// # Errors
    /// Reports a worker panic; arbitrary native code may prolong physical completion.
    pub async fn join_owned(mut self) -> Result<crate::ShutdownReport, CronError> {
        let joined = match self.task.take() {
            Some(task) => task.await.map_err(|error| {
                StoreError::new(StoreErrorKind::Other, "worker_join_failed").with_source(error)
            }),
            None => Ok(()),
        };
        self.shutdown.take();
        self.controller.join().await;
        joined?;
        let now = std::time::Instant::now();
        let budget = self.controller.budget();
        self.controller.status(crate::ShutdownStatus::Joined);
        Ok(crate::ShutdownReport {
            grace_exceeded: budget.is_some_and(|b| now >= b.grace),
            cleanup_exceeded: budget.is_some_and(|b| now >= b.cleanup),
            cleanup: self
                .controller
                .cleanup_outcome()
                .unwrap_or(crate::TransactionOutcome {
                    admission: crate::CommitAdmission::NotAttempted,
                    persistence: crate::PersistenceOutcome::NoMutationAttempted,
                    completed_after_cutoff: budget.is_some_and(|b| now >= b.cleanup),
                }),
        })
    }
}

impl WorkerStopControl {
    /// Closes intake and shortens the live absolute shutdown cutoffs.
    ///
    /// # Errors
    /// Rejects cleanup cutoffs preceding grace.
    pub fn request_stop(&self, budget: crate::ShutdownBudget) -> Result<(), CronError> {
        self.controller.stop(budget)
    }
    /// Cancels ordinary work immediately without extending cleanup.
    pub fn cancel_now(&self) {
        self.controller.cancel();
    }
    /// Returns a live scope sharing this worker's controller.
    #[must_use]
    pub fn operation_context(&self) -> crate::StoreOperationContext {
        crate::StoreOperationContext::root(Arc::clone(&self.controller))
    }
    /// Subscribes to expiry independently of physical join.
    #[must_use]
    pub fn subscribe_status(&self) -> watch::Receiver<crate::ShutdownStatus> {
        self.controller.subscribe()
    }
}

impl<S: ScheduleStore + 'static, C: Clock + 'static, J: JitterSource + 'static>
    crate::CronOperations for Scheduler<S, C, J>
{
    fn start(
        &self,
        context: crate::StoreOperationContext,
        spec: ScheduleSpec,
    ) -> crate::CronOperationFuture<'_, Schedule> {
        Box::pin(self.start_with_context(spec, context))
    }
    fn end(
        &self,
        context: crate::StoreOperationContext,
        request: crate::EndSchedule,
    ) -> crate::CronOperationFuture<'_, crate::EndedSchedule> {
        Box::pin(async move {
            let core = Arc::clone(&self.core);
            self.command(context, move || {
                let page = core.store.query(&crate::ScheduleQuery {
                    name: Some(request.name.clone()),
                    selection: crate::ScheduleSelection::Active,
                    limit: 1,
                    ..Default::default()
                })?;
                let row = page
                    .items
                    .into_iter()
                    .next()
                    .ok_or(CronError::NotFound(request.name))?;
                core.stop(row.id, request.generation, &request.reason)?;
                Ok(crate::EndedSchedule {
                    id: row.id,
                    generation: request.generation,
                })
            })
            .await
        })
    }
    fn query(
        &self,
        context: crate::StoreOperationContext,
        query: crate::ScheduleQuery,
    ) -> crate::CronOperationFuture<'_, crate::SchedulePage> {
        Box::pin(self.query_with_context(query, context))
    }
}
