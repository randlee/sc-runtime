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
    let context =
        sc_cron::operation_context_probe(std::time::Instant::now() + Duration::from_secs(60));
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
    use cron_consumer_proof::CooperativeForwarder;
    struct Job;
    #[async_trait::async_trait]
    impl sc_cron::Job for Job {
        async fn run(&self, _: &serde_json::Value, _: std::time::Instant) -> sc_cron::RunOutcome {
            sc_cron::RunOutcome::Ok
        }
        async fn should_stop(&self, _: &serde_json::Value) -> sc_cron::StopDecision {
            sc_cron::StopDecision::Continue
        }
    }
    let store = Arc::new(CooperativeForwarder(
        sc_cron::SqliteScheduleStore::open_in_memory(sc_cron::ScheduleLimits::default()).unwrap(),
    ));
    let scheduler = Scheduler::new(
        Arc::clone(&store),
        Arc::new(SystemClock),
        Arc::new(SystemJitter),
    );
    let worker = scheduler.spawn_worker_cooperative(Arc::new(Job)).unwrap();
    let control = worker.stop_control();
    let context = control.operation_context();
    let (entered, wait_entered) = std::sync::mpsc::channel();
    let (release, wait_release) = std::sync::mpsc::channel();
    // A direct scoped store call remains explicitly caller-owned; join it here.
    let physical = {
        let store = Arc::clone(&store);
        std::thread::spawn(move || {
            store.with_operation_context(&context, |store| {
                entered.send(()).unwrap();
                wait_release.recv().unwrap();
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
    wait_entered.recv_timeout(Duration::from_secs(5)).unwrap();
    control.cancel_now();
    release.send(()).unwrap();
    assert!(physical.join().unwrap().is_err());
    worker.join_owned().await.unwrap();
    assert!(store.list_active().unwrap().is_empty());
}
