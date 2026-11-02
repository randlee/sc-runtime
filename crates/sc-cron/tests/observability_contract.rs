mod common;

use std::sync::Arc;
use std::time::Duration;

use sc_command::observability::{RecordingCanonicalLog, RejectingCanonicalLog, SharedCanonicalLog};
use sc_cron::{
    PythonHook, ScheduleLimits, ScheduleSpec, Scheduler, SeqJitter, SqliteScheduleStore,
    SystemClock, SystemJitter,
};
use sc_observability_types::ServiceName;

#[test]
fn generic_providers_and_shared_coord_keep_original_identity() {
    let store = Arc::new(common::RecordingStore::new());
    let jitter = Arc::new(SeqJitter::new([Duration::from_millis(123)]));
    let recorder = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("injected").unwrap(),
    ));
    let logger: SharedCanonicalLog = recorder.clone();
    let scheduler = Scheduler::new_with_logger(
        Arc::clone(&store),
        Arc::new(SystemClock),
        Arc::clone(&jitter),
        Arc::clone(&logger),
    );
    assert!(Arc::ptr_eq(scheduler.store(), &store));
    let shared = scheduler.share();
    assert!(Arc::ptr_eq(shared.store(), &store));
    assert_eq!(Arc::strong_count(&logger), 3); // caller, recorder, one shared RunCoord
    let row = shared
        .start(
            ScheduleSpec::new(
                "sentinel",
                "* * * * *",
                Duration::from_secs(1),
                serde_json::json!({}),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(row.jitter, Duration::from_millis(123));
    assert_eq!(jitter.calls().len(), 1);
    let events = recorder.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].target.as_str(), "cron.scheduler");
    assert_eq!(events[0].action.as_str(), "logger.attached");
    assert_eq!(events[0].service.as_str(), "injected");
}

#[test]
fn built_in_open_installs_same_handle_once_and_legacy_defaults_stay_fixed() {
    let root = tempfile::tempdir().unwrap();
    let recorder = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("builtins").unwrap(),
    ));
    let logger: SharedCanonicalLog = recorder.clone();
    let scheduler = Scheduler::open_with_logger(
        root.path().join("cron.db"),
        ScheduleLimits::default(),
        Arc::new(SystemClock),
        Arc::clone(&logger),
    )
    .unwrap();
    // recorder + caller + store + jitter + RunCoord all share the same allocation.
    assert_eq!(Arc::strong_count(&logger), 5);
    assert_eq!(recorder.events().len(), 1);
    assert!(format!("{:?}", scheduler.store()).contains("builtins"));
    drop(scheduler);
    assert_eq!(Arc::strong_count(&logger), 2);
    let store = SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap();
    assert!(format!("{store:?}").contains("sc-cron"));
    assert!(format!("{SystemJitter:?}").contains("sc-cron"));
    let legacy = Scheduler::new(
        Arc::new(store),
        Arc::new(SystemClock),
        Arc::new(SystemJitter),
    );
    assert!(format!("{legacy:?}").contains("sc-cron"));
}

#[test]
fn rejection_is_best_effort_for_scheduler_and_python_constructor() {
    let reject = Arc::new(RejectingCanonicalLog::new(
        ServiceName::new("reject").unwrap(),
    ));
    let scheduler = Scheduler::new_with_logger(
        Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap()),
        Arc::new(SystemClock),
        Arc::new(SystemJitter),
        reject.clone(),
    );
    assert_eq!(reject.attempts(), 1);
    let row = scheduler
        .start(
            ScheduleSpec::new(
                "still-works",
                "* * * * *",
                Duration::from_secs(1),
                serde_json::json!({}),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(row.spec.name.as_str(), "still-works");
    let hook = PythonHook::new_with_logger("hooks/test.py", reject).unwrap();
    assert_eq!(hook, PythonHook::new("hooks/test.py").unwrap());
}

#[tokio::test]
async fn worker_retains_shared_logger_without_reattachment() {
    let recorder = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("worker-identity").unwrap(),
    ));
    let logger: SharedCanonicalLog = recorder.clone();
    let scheduler = Scheduler::new_with_logger(
        Arc::new(common::RecordingStore::new()),
        Arc::new(SystemClock),
        Arc::new(SeqJitter::new([])),
        Arc::clone(&logger),
    );
    let shared = scheduler.share();
    let worker = shared.spawn_worker(Arc::new(common::ScriptedJob::new()));
    assert!(format!("{worker:?}").contains("worker-identity"));
    assert_eq!(Arc::strong_count(&logger), 4); // caller, recorder, RunCoord, WorkerHandle
    drop(scheduler);
    drop(shared);
    assert_eq!(recorder.events().len(), 1);
    assert!(
        worker
            .shutdown(tokio::time::Instant::now() + Duration::from_secs(2))
            .await
    );
    assert_eq!(Arc::strong_count(&logger), 2);
}

#[test]
fn manifest_and_owned_sources_do_not_construct_full_runtime() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("sc-observability-types.workspace = true"));
    // Resolved normal package identities are enforced for both libraries by
    // the shared observability-dependencies check in the existing CI test path.
    for source in [
        include_str!("../src/coord.rs"),
        include_str!("../src/scheduler.rs"),
        include_str!("../src/hook.rs"),
        include_str!("../src/process.rs"),
    ] {
        assert!(!source.contains("enqueue_blocking"));
        assert!(!source.contains(".log("));
        assert!(!source.contains("LoggerBuilder"));
    }
}
