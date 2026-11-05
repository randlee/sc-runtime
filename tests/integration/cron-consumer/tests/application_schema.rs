mod support;
use cron_consumer_proof::ApplicationStore;
use sc_cron::{
    CronError, Generation, ScheduleQuery, ScheduleSelection, ScheduleSpec, ScheduleStore,
    Scheduler, StoreOperationSupport, SystemClock, SystemJitter,
};
use std::sync::Arc;
use std::time::Duration;
fn spec() -> ScheduleSpec {
    ScheduleSpec::new(
        "application",
        "* * * * *",
        Duration::from_secs(1),
        serde_json::json!({}),
    )
    .unwrap()
}
#[tokio::test]
async fn extra_field_survives_query_end_restart_and_reopen_with_generation_fencing() {
    if support::supervised(
        "extra_field_survives_query_end_restart_and_reopen_with_generation_fencing",
    ) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let schedules = dir.path().join("cron.db");
    let app = dir.path().join("application.db");
    let store = Arc::new(ApplicationStore::open(&schedules, &app, "tenant-42").unwrap());
    assert_eq!(store.operation_support(), StoreOperationSupport::Legacy);
    let scheduler = Scheduler::new(
        Arc::clone(&store),
        Arc::new(SystemClock),
        Arc::new(SystemJitter),
    );
    let row = scheduler.start(spec()).unwrap();
    assert_eq!(store.tenant(row.id, row.generation).unwrap(), "tenant-42");
    assert_eq!(
        store.query(&ScheduleQuery::default()).unwrap().items.len(),
        1
    );
    scheduler
        .stop(row.id, row.generation, "application ended")
        .unwrap();
    assert_eq!(
        store
            .query(&ScheduleQuery {
                selection: ScheduleSelection::Ended,
                ..Default::default()
            })
            .unwrap()
            .items[0]
            .id,
        row.id
    );
    let replacement = scheduler.start(spec()).unwrap();
    assert_eq!(replacement.id, row.id);
    assert_ne!(replacement.generation, Generation::FIRST);
    assert!(matches!(
        scheduler.stop(row.id, row.generation, "stale"),
        Err(CronError::Fenced { .. })
    ));
    let context = sc_cron::WorkerStopControl::new()
        .unwrap()
        .operation_context()
        .with_deadline(std::time::Instant::now() + Duration::from_secs(60));
    let refused = scheduler
        .start_with_context(
            ScheduleSpec::new(
                "unsupported",
                "* * * * *",
                Duration::from_secs(1),
                serde_json::json!({}),
            )
            .unwrap(),
            context,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(refused, CronError::Store(ref error) if error.context() == "operation_context_unsupported")
    );
    assert_eq!(
        store.query(&ScheduleQuery::default()).unwrap().items.len(),
        1
    );
    drop(scheduler);
    drop(store);
    let reopened = ApplicationStore::open(&schedules, &app, "unused-next-tenant").unwrap();
    assert_eq!(
        reopened
            .tenant(replacement.id, replacement.generation)
            .unwrap(),
        "tenant-42"
    );
    assert_eq!(
        reopened.get(replacement.id).unwrap().unwrap().generation,
        replacement.generation
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn declared_cooperative_forwarder_retains_live_scope_after_admission() {
    if support::supervised("declared_cooperative_forwarder_retains_live_scope_after_admission") {
        return;
    }
    use cron_consumer_proof::CooperativeForwarder;
    let store = Arc::new(CooperativeForwarder(
        sc_cron::SqliteScheduleStore::open_in_memory(sc_cron::ScheduleLimits::default()).unwrap(),
    ));
    let control = sc_cron::WorkerStopControl::new().unwrap();
    let context = control.operation_context();
    let (entered, wait_entered) = std::sync::mpsc::channel();
    let (release, wait_release) = std::sync::mpsc::channel();
    // A direct scoped store call remains explicitly caller-owned; join it here.
    let physical = {
        let store = Arc::clone(&store);
        std::thread::spawn(move || {
            store.with_operation_context(&context, |store| {
                entered.send(()).unwrap();
                let _ = wait_release.recv_timeout(Duration::from_secs(5));
                let now = sc_cron::Timestamp::from_datetime(chrono::Utc::now());
                store.create(
                    &sc_cron::NewSchedule {
                        spec: spec(),
                        jitter: Duration::ZERO,
                        next_run_at: now,
                    },
                    now,
                )
            })
        })
    };
    let ready = wait_entered.recv_timeout(Duration::from_secs(5));
    control.cancel_now();
    let _ = release.send(());
    let cutoff = std::time::Instant::now() + Duration::from_secs(5);
    while !physical.is_finished() {
        assert!(
            std::time::Instant::now() < cutoff,
            "physical custom-provider cleanup failed"
        );
        std::thread::park_timeout(Duration::from_millis(1));
    }
    let result = physical.join().unwrap();
    ready.unwrap();
    assert!(result.is_err());
    assert!(store.list_active().unwrap().is_empty());
}

#[tokio::test]
async fn application_operations_use_normal_worker_free_control_and_earliest_deadline() {
    if support::supervised(
        "application_operations_use_normal_worker_free_control_and_earliest_deadline",
    ) {
        return;
    }
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
}
