//! Active ownership regressions; an idle worker cannot establish these joins.

use super::*;
use sc_command::observability::{RecordingCanonicalLog, SharedCanonicalLog};
use sc_observability_types::ServiceName;

fn logger() -> Arc<RecordingCanonicalLog> {
    Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("lifecycle").expect("service"),
    ))
}

#[tokio::test]
async fn shutdown_timeout_joins_active_store_before_releasing_real_logger() {
    let root = tempfile::tempdir().expect("root");
    let log = logger();
    let lifetime = Arc::downgrade(&log);
    let inner = SqliteScheduleStore::open_with_logger(
        root.path().join("cron.db"),
        ScheduleLimits::default(),
        log.clone(),
    )
    .expect("real store");
    let store = Arc::new(RecordingStore::from_inner(inner));
    let clock = Arc::new(ManualClock::new(at("2026-10-19T00:00:00Z")));
    let scheduler = Scheduler::new_with_logger(
        store.clone(),
        clock,
        Arc::new(SeqJitter::default()),
        log.clone(),
    );
    let open = store.gate_next_list();
    let worker = scheduler.spawn_worker(Arc::new(TestJob::default()));
    store.wait_listing(1).await;
    drop(scheduler);
    drop(store);
    drop(log);
    let mut shutdown = Box::pin(worker.shutdown(tokio::time::Instant::now()));
    let early = tokio::time::timeout(Duration::from_millis(25), &mut shutdown).await;
    assert!(
        early.is_err(),
        "shutdown must join the active blocking store after its deadline"
    );
    assert!(
        lifetime.upgrade().is_some(),
        "real SQLite store still owns injected logger while gated"
    );
    open.send(()).expect("open gate");
    assert!(
        !tokio::time::timeout(WAIT, shutdown)
            .await
            .expect("physical store join")
    );
    assert!(
        lifetime.upgrade().is_none(),
        "no injected logger Arc survives shutdown return"
    );
}

/// A child destructor held in flight after cancellation, retaining the actual logger.
struct ChildDrop {
    entered: Option<tokio::sync::oneshot::Sender<()>>,
    release: std::sync::mpsc::Receiver<()>,
    _logger: SharedCanonicalLog,
}

impl Drop for ChildDrop {
    fn drop(&mut self) {
        if let Some(entered) = self.entered.take() {
            let _ = entered.send(());
        }
        let _ = self.release.recv_timeout(WAIT);
    }
}

struct GatedChild {
    entered: watch::Sender<bool>,
    guard: Mutex<Option<ChildDrop>>,
}

#[async_trait]
impl Job for GatedChild {
    async fn should_stop(&self, _: &Value) -> StopDecision {
        StopDecision::Continue
    }

    async fn run(&self, _: &Value, _: Instant) -> RunOutcome {
        let _guard = self.guard.lock().expect("guard").take().expect("one child");
        self.entered.send_replace(true);
        std::future::pending().await
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_timeout_joins_cancelled_child_before_releasing_logger() {
    let log = logger();
    let lifetime = Arc::downgrade(&log);
    let (dropping, dropped) = tokio::sync::oneshot::channel();
    let (open, release) = std::sync::mpsc::channel();
    let job = Arc::new(GatedChild {
        entered: watch::Sender::new(false),
        guard: Mutex::new(Some(ChildDrop {
            entered: Some(dropping),
            release,
            _logger: log.clone(),
        })),
    });
    let store = Arc::new(RecordingStore::new());
    let clock = Arc::new(ManualClock::new(at("2026-10-19T00:00:00Z")));
    let scheduler = Scheduler::new_with_logger(
        store,
        clock.clone(),
        Arc::new(SeqJitter::default()),
        log.clone(),
    );
    scheduler
        .start(spec("active-child", "0 * * * * *", 20))
        .expect("schedule");
    let parks = clock.parks();
    let worker = scheduler.spawn_worker(job.clone());
    clock.wait_parks_above(parks).await;
    clock.advance(chrono::Duration::seconds(60));
    wait_on(&job.entered, "active child", |entered| *entered).await;
    drop(job);
    drop(scheduler);
    drop(log);
    let mut shutdown = tokio::spawn(worker.shutdown(tokio::time::Instant::now()));
    tokio::time::timeout(WAIT, dropped)
        .await
        .expect("cancellation bound")
        .expect("child dropping");
    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut shutdown)
            .await
            .is_err(),
        "shutdown cannot finish before child destruction joins"
    );
    assert!(
        lifetime.upgrade().is_some(),
        "active child still owns logger"
    );
    open.send(()).expect("release child destruction");
    assert!(
        !tokio::time::timeout(WAIT, shutdown)
            .await
            .expect("child joined")
            .expect("shutdown task")
    );
    assert!(
        lifetime.upgrade().is_none(),
        "no injected logger Arc survives child join"
    );
}
