//! External SDK consumer: only sc-cron provides scheduling and canonical receiver types.
use sc_cron::observability::{CanonicalLog, EventError, LogEvent, ServiceName};
use sc_cron::{
    HookTarget, Job, ManualClock, RunOutcome, ScheduleLimits, ScheduleSpec, ScheduleStore,
    Scheduler, SeqJitter, ShutdownBudget, SqliteScheduleStore, StopDecision,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Receiver {
    service: ServiceName,
    actions: tokio::sync::watch::Sender<Vec<String>>,
}
impl CanonicalLog for Receiver {
    fn service_name(&self) -> &ServiceName {
        &self.service
    }
    fn emit(&self, event: LogEvent) -> Result<(), EventError> {
        self.actions
            .send_modify(|actions| actions.push(event.action.as_str().to_owned()));
        Ok(())
    }
}
struct RustJob;
#[async_trait::async_trait]
impl Job for RustJob {
    async fn run(&self, _: &serde_json::Value, _: Instant) -> RunOutcome {
        RunOutcome::Ok
    }
    async fn should_stop(&self, _: &serde_json::Value) -> StopDecision {
        StopDecision::Continue
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_receiver_observes_actual_target_execution_and_owned_shutdown() {
    let (events, mut observed) = tokio::sync::watch::channel(Vec::new());
    let receiver = Arc::new(Receiver {
        service: ServiceName::new("external-cron-consumer").unwrap(),
        actions: events,
    });
    let clock = Arc::new(ManualClock::new("2026-10-19T00:00:00Z".parse().unwrap()));
    let scheduler = Scheduler::new_with_logger(
        Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap()),
        Arc::clone(&clock),
        Arc::new(SeqJitter::default()),
        receiver,
    );
    #[cfg(feature = "monty")]
    let scheduler = {
        let registry = sc_cron::MontyHostRegistry::new()
            .register("application_answer", |call: &sc_cron::MontyHostCall| {
                assert_eq!(call.phase, sc_cron::HookEvent::Tick);
                Ok(serde_json::json!(42))
            })
            .unwrap();
        scheduler.with_monty_hosts(registry)
    };
    let directory = tempfile::tempdir().unwrap();
    let python = directory.path().join("consumer_tick.py");
    std::fs::write(&python, "import json,sys\nif sys.argv[1]=='cron.stop':\n print(json.dumps({'action':'continue'}))\nelse:\n data=json.load(sys.stdin)\n assert data['payload']['from']=='consumer'\n").unwrap();
    let mut hooks = vec![
        ("rust", HookTarget::Rust),
        ("python", HookTarget::python(&python).unwrap()),
    ];
    #[cfg(feature = "monty")]
    hooks.push(("monty", HookTarget::monty("def cron_stop(hook):\n    return {'action': 'continue'}\ndef cron_tick(hook):\n    if application_answer() != 42:\n        raise RuntimeError('custom host')\n").unwrap()));
    #[cfg(not(feature = "monty"))]
    let hooks = {
        hooks.shrink_to_fit();
        hooks
    };
    let mut rows = Vec::new();
    for (name, target) in &hooks {
        let spec = ScheduleSpec::with_hook(
            *name,
            "0 * * * * *",
            Duration::from_secs(10),
            serde_json::json!({"from":"consumer"}),
            target.clone(),
        )
        .unwrap();
        assert!(spec.environment.is_empty());
        rows.push(scheduler.start(spec).unwrap());
    }
    let worker = scheduler
        .spawn_worker_cooperative(Arc::new(RustJob))
        .unwrap();
    let control = worker.stop_control();
    clock.wait_parks_above(0).await;
    clock.advance_settled(chrono::Duration::seconds(60)).await;
    tokio::time::timeout(
        Duration::from_secs(15),
        observed.wait_for(|actions| {
            actions
                .iter()
                .filter(|action| action.as_str() == "run.finished")
                .count()
                == hooks.len()
        }),
    )
    .await
    .unwrap()
    .unwrap();
    let actions = observed.borrow().clone();
    assert!(actions.iter().any(|action| action == "logger.attached"));
    assert_eq!(
        actions
            .iter()
            .filter(|action| action.as_str() == "run.launched")
            .count(),
        hooks.len()
    );
    for row in rows {
        assert_eq!(
            scheduler.store().get(row.id).unwrap().unwrap().last_outcome,
            Some(RunOutcome::Ok)
        );
    }
    let now = Instant::now();
    control
        .request_stop(
            ShutdownBudget::new(now + Duration::from_secs(5), now + Duration::from_secs(10))
                .unwrap(),
        )
        .unwrap();
    worker.join_owned().await.unwrap();
}
