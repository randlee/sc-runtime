use sc_cron::{CronError, ScheduleQuery, ScheduleSpec};
use std::time::Duration;
#[tokio::main(flavor = "current_thread")]
async fn main() {
    struct ApplicationOperations;
    impl sc_cron::CronOperations for ApplicationOperations {
        fn start(
            &self,
            context: sc_cron::StoreOperationContext,
            _: ScheduleSpec,
        ) -> sc_cron::CronOperationFuture<'_, sc_cron::Schedule> {
            Box::pin(async move {
                context.check()?;
                Err(CronError::InvalidSpec("application policy".into()))
            })
        }
        fn end(
            &self,
            context: sc_cron::StoreOperationContext,
            _: sc_cron::EndSchedule,
        ) -> sc_cron::CronOperationFuture<'_, sc_cron::EndedSchedule> {
            Box::pin(async move {
                context.check()?;
                Err(CronError::InvalidSpec("application policy".into()))
            })
        }
        fn query(
            &self,
            context: sc_cron::StoreOperationContext,
            _: ScheduleQuery,
        ) -> sc_cron::CronOperationFuture<'_, sc_cron::SchedulePage> {
            Box::pin(async move {
                context.check()?;
                Ok(sc_cron::SchedulePage {
                    items: vec![],
                    next_after_id: None,
                })
            })
        }
    }
    let control = sc_cron::WorkerStopControl::new().unwrap();
    let now = std::time::Instant::now();
    let context = control
        .operation_context()
        .with_deadline(now + Duration::from_secs(5));
    let child = context.with_deadline(now + Duration::from_secs(10));
    assert_eq!(child.deadline(), context.deadline());
    let operations: &dyn sc_cron::CronOperations = &ApplicationOperations;
    assert!(
        operations
            .query(child.clone(), ScheduleQuery::default())
            .await
            .unwrap()
            .items
            .is_empty()
    );
    control.cancel_now();
    assert!(
        operations
            .query(child, ScheduleQuery::default())
            .await
            .is_err()
    );
    control.finish_application_shutdown().unwrap();
    assert_eq!(
        *control.subscribe_status().borrow(),
        sc_cron::ShutdownStatus::Joined
    );
}
