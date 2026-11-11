//! Ordinary-feature consumer: no framework worker or test-utils.
use sc_cron::{CronError, CronOperations, ScheduleQuery, ScheduleSpec, StoreOperationContext};
use std::time::{Duration, Instant};

struct ApplicationOperations;
impl CronOperations for ApplicationOperations {
    fn start(
        &self,
        context: StoreOperationContext,
        _: ScheduleSpec,
    ) -> sc_cron::CronOperationFuture<'_, sc_cron::Schedule> {
        Box::pin(async move {
            context.check()?;
            Err(CronError::InvalidSpec("application policy".into()))
        })
    }
    fn end(
        &self,
        context: StoreOperationContext,
        _: sc_cron::EndSchedule,
    ) -> sc_cron::CronOperationFuture<'_, sc_cron::EndedSchedule> {
        Box::pin(async move {
            context.check()?;
            Err(CronError::InvalidSpec("application policy".into()))
        })
    }
    fn query(
        &self,
        context: StoreOperationContext,
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let schema = rmcp::handler::server::common::schema_for_input::<sc_cron_commands::CronRequest>()
        .expect("canonical request accepted at actual RMCP input-schema boundary");
    let tool = rmcp::model::Tool::new("cron", "standard cron operations", schema);
    assert_eq!(tool.input_schema["type"], "object");
    assert_eq!(tool.input_schema["oneOf"].as_array().unwrap().len(), 3);
    let control = sc_cron::WorkerStopControl::new().expect("ordinary factory");
    assert!(control.finish_application_shutdown().is_err());
    let context = control.operation_context();
    assert!(context.with_deadline(Instant::now()).check().is_err());
    let earlier = Instant::now() + Duration::from_secs(60);
    let child = context
        .with_deadline(earlier)
        .with_deadline(earlier + Duration::from_secs(60));
    assert_eq!(child.deadline(), Some(earlier));
    let provider: &dyn CronOperations = &ApplicationOperations;
    assert!(
        provider
            .query(child.clone(), ScheduleQuery::default())
            .await
            .expect("custom query")
            .items
            .is_empty()
    );

    // The provider has no outstanding work. Ask for stop from an ordinary
    // thread, retaining its join until is_finished proves physical completion.
    let stop = control.clone();
    let owner = std::thread::spawn(move || {
        assert!(tokio::runtime::Handle::try_current().is_err());
        assert!(sc_cron::WorkerStopControl::new().is_err());
        let grace = Instant::now() + Duration::from_secs(3600);
        stop.request_stop(sc_cron::ShutdownBudget::new(grace, grace).expect("checked budget"))
    });
    let guard = Instant::now() + Duration::from_secs(5);
    while !owner.is_finished() {
        assert!(
            Instant::now() < guard,
            "ordinary stop thread did not complete"
        );
        tokio::task::yield_now().await;
    }
    owner
        .join()
        .expect("finished thread")
        .expect("ordinary-thread stop");
    assert!(context.stop_requested());
    assert!(context.check().is_ok(), "future G has not expired");
    control
        .finish_application_shutdown()
        .expect("application joined its work");
    assert_eq!(
        *control.subscribe_status().borrow(),
        sc_cron::ShutdownStatus::Joined
    );
    for scope in [context, child, control.operation_context()] {
        let error = provider
            .query(scope, ScheduleQuery::default())
            .await
            .expect_err("acknowledgement revokes all scopes");
        assert!(error.recovery().contains("reconcile"));
        assert!(!error.recovery().contains("database path"));
    }
    assert!(sc_cron::ShutdownBudget::new(earlier, earlier - Duration::from_secs(1)).is_err());
    for reason in [
        "operation_context_unsupported",
        "schedule_query_unsupported",
    ] {
        let error = CronError::from(sc_cron::StoreError::new(
            sc_cron::StoreErrorKind::Unavailable,
            reason,
        ));
        assert!(error.recovery().contains("provider"));
        assert!(!error.recovery().contains("database path"));
    }
    println!("ordinary application-owned context contract passed");
}
