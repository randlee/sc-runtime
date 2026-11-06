//! Standard validated command dispatch over application-owned native operations.
use std::sync::Arc;
use std::time::Instant;

use sc_command::{Envelope, OpError};
use sc_cron::{CronOperations, StoreOperationContext};

use crate::{
    ApplicationTargetBindings, CronExecutor, CronOperation, CronRequest, CronResponse, EndResult,
    ExecutorFuture, ListResult, ScheduleState, ScheduleView, TargetKind,
};

mod errors;
mod inputs;

/// Standard adapter; native operations retain physical-work and overload ownership.
///
/// This adapter adds neither a timeout wrapper nor another scheduling registry.
/// Applications authorize targets through `B` and may replace either port.
pub struct StandardCronExecutor<O: CronOperations + ?Sized, B: ApplicationTargetBindings + ?Sized> {
    operations: Arc<O>,
    bindings: Arc<B>,
    context: StoreOperationContext,
}

impl<O: CronOperations + ?Sized, B: ApplicationTargetBindings + ?Sized> StandardCronExecutor<O, B> {
    /// Binds native operations, target authorization, and a shared live root scope.
    #[must_use]
    pub fn new(operations: Arc<O>, bindings: Arc<B>, context: StoreOperationContext) -> Self {
        Self {
            operations,
            bindings,
            context,
        }
    }

    async fn dispatch(
        &self,
        request: CronRequest,
        deadline: Instant,
    ) -> Result<CronResponse, OpError> {
        inputs::request(&request)?;
        let context = self.context.with_deadline(deadline);
        context.check().map_err(errors::native)?;
        match request.operation {
            CronOperation::Start(start) => {
                let spec = inputs::start(start, self.bindings.as_ref())?;
                context.check().map_err(errors::native)?;
                let row = self
                    .operations
                    .start(context, spec)
                    .await
                    .map_err(errors::native)?;
                Ok(CronResponse::Started(view(row)?))
            }
            CronOperation::End(end) => {
                let end = inputs::end(end)?;
                context.check().map_err(errors::native)?;
                let ended = self
                    .operations
                    .end(context, end)
                    .await
                    .map_err(errors::native)?;
                Ok(CronResponse::Ended(EndResult {
                    id: ended.id.get(),
                    generation: ended.generation.get(),
                }))
            }
            CronOperation::List(list) => {
                let query = inputs::query(list)?;
                context.check().map_err(errors::native)?;
                let page = self
                    .operations
                    .query(context, query)
                    .await
                    .map_err(errors::native)?;
                Ok(CronResponse::Listed(ListResult {
                    items: page.items.into_iter().map(view).collect::<Result<_, _>>()?,
                    next_cursor: page.next_after_id.map(sc_cron::ScheduleId::get),
                }))
            }
        }
    }
}

impl<O: CronOperations + ?Sized, B: ApplicationTargetBindings + ?Sized> CronExecutor
    for StandardCronExecutor<O, B>
{
    fn execute(&self, request: CronRequest, deadline: Instant) -> ExecutorFuture<'_> {
        Box::pin(async move { Envelope::from(self.dispatch(request, deadline).await) })
    }
}

fn view(row: sc_cron::Schedule) -> Result<ScheduleView, OpError> {
    let target = match row.spec.hook {
        sc_cron::HookTarget::Rust => TargetKind::Rust,
        sc_cron::HookTarget::Python { .. } => TargetKind::Python,
        sc_cron::HookTarget::Monty { .. } => TargetKind::Monty,
        _ => return Err(errors::internal()),
    };
    let state = if row.is_active() {
        ScheduleState::Active
    } else {
        ScheduleState::Ended
    };
    Ok(ScheduleView {
        id: row.id.get(),
        generation: row.generation.get(),
        name: row.spec.name.to_string(),
        cron: row.spec.expr.as_str().to_owned(),
        target,
        enabled: row.enabled,
        state,
        next_run_ms: row
            .next_run_at
            .map(|at| at.as_datetime().timestamp_millis()),
    })
}
