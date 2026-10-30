use std::sync::Arc;
use std::time::Duration;

use sc_cron::{
    ScheduleLimits, ScheduleSpec, ScheduleStore, Scheduler, SeqJitter, SqliteScheduleStore,
    SystemClock, SystemJitter, WorkerHandle,
};
use sc_observability_types::{
    CanonicalLog, Level, LogEvent, RecordingCanonicalLog, RejectingCanonicalLog, ServiceName,
    SharedCanonicalLog,
};

fn recorder() -> Arc<RecordingCanonicalLog> {
    Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("component-test").unwrap(),
    ))
}

fn row(event: &LogEvent, target: &str, action: &str, level: Level, outcome: &str, code: &str) {
    assert_eq!(event.target.as_str(), target);
    assert_eq!(event.action.as_str(), action);
    assert_eq!(event.level, level);
    assert_eq!(event.outcome.as_ref().unwrap().as_str(), outcome);
    let diagnostic = event.diagnostic.as_ref().unwrap();
    assert_eq!(diagnostic.code.as_str(), code);
    assert_eq!(diagnostic.timestamp, event.timestamp);
    assert!(!event.fields.contains_key("code"));
    assert_eq!(event.service.as_str(), "component-test");
    assert!(event.trace.is_none() && event.state_transition.is_none());
}

#[test]
fn entropy_failure_emits_once_and_keeps_zero_fallback_under_rejection() {
    let log = recorder();
    let jitter = SystemJitter::with_logger(log.clone());
    assert_eq!(
        jitter.jitter_without_entropy(Duration::from_secs(1)),
        Duration::ZERO
    );
    assert_eq!(
        jitter.jitter_without_entropy(Duration::ZERO),
        Duration::ZERO
    );
    let events = log.events();
    assert_eq!(events.len(), 1);
    row(
        &events[0],
        "cron.jitter",
        "source.unavailable",
        Level::Warn,
        "failed",
        "JITTER_SOURCE_UNAVAILABLE",
    );
    assert!(events[0].fields["error"].is_string());
    let reject = Arc::new(RejectingCanonicalLog::new(
        ServiceName::new("reject").unwrap(),
    ));
    assert_eq!(
        SystemJitter::with_logger(reject.clone()).jitter_without_entropy(Duration::from_secs(1)),
        Duration::ZERO
    );
    assert_eq!(reject.attempts(), 1);
}

fn corrupt_row(log: SharedCanonicalLog) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("cron.db");
    let store = Arc::new(
        SqliteScheduleStore::open_with_logger(&path, ScheduleLimits::default(), log.clone())
            .unwrap(),
    );
    let scheduler = Scheduler::new_with_logger(
        store.clone(),
        Arc::new(SystemClock),
        Arc::new(SeqJitter::new([])),
        log,
    );
    for name in ["bad", "good"] {
        scheduler
            .start(
                ScheduleSpec::new(
                    name,
                    "* * * * *",
                    Duration::from_secs(1),
                    serde_json::json!({}),
                )
                .unwrap(),
            )
            .unwrap();
    }
    rusqlite::Connection::open(path)
        .unwrap()
        .execute(
            "UPDATE sc_cron_schedule SET payload = 'not json' WHERE name = 'bad'",
            [],
        )
        .unwrap();
    for _ in 0..2 {
        let listed = store.list_active().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].spec.name.as_str(), "good");
    }
}

#[test]
fn corrupt_rows_emit_one_canonical_row_and_rejection_does_not_change_skipping() {
    let log = recorder();
    corrupt_row(log.clone());
    let events: Vec<_> = log
        .events()
        .into_iter()
        .filter(|event| event.action.as_str() == "row.corrupt")
        .collect();
    assert_eq!(events.len(), 1);
    row(
        &events[0],
        "cron.store",
        "row.corrupt",
        Level::Warn,
        "failed",
        "CORRUPT_SCHEDULE_ROW",
    );
    assert_eq!(events[0].fields["schedule_name"], "bad");
    assert!(events[0].fields["schedule_id"].is_u64());
    assert!(events[0].fields["error"].is_string());
    let reject = Arc::new(RejectingCanonicalLog::new(
        ServiceName::new("reject").unwrap(),
    ));
    corrupt_row(reject.clone());
    assert_eq!(reject.attempts(), 2); // one attachment plus one corrupt row
}

#[tokio::test]
async fn failed_kill_and_unreaped_child_emit_exact_rows_with_unchanged_result() {
    let log = recorder();
    assert!(!sc_cron::process::testing::failed_kill_and_reap(log.clone()).await);
    let events = log.events();
    assert_eq!(events.len(), 2);
    row(
        &events[0],
        "cron.process",
        "child.kill_failed",
        Level::Debug,
        "failed",
        "CHILD_KILL_FAILED",
    );
    assert!(events[0].fields["error"].is_string());
    assert_eq!(events[0].fields["pid"], 42);
    row(
        &events[1],
        "cron.process",
        "child.reap_timeout",
        Level::Warn,
        "timeout",
        "CHILD_REAP_TIMEOUT",
    );
    assert_eq!(events[1].fields["pid"], 42);
    assert_eq!(events[1].fields["deadline_remaining_ms"], 0);
    let reject = Arc::new(RejectingCanonicalLog::new(
        ServiceName::new("reject").unwrap(),
    ));
    assert!(!sc_cron::process::testing::failed_kill_and_reap(reject.clone()).await);
    assert_eq!(reject.attempts(), 2);
}

struct ObservedLog {
    inner: SharedCanonicalLog,
    emitted: tokio::sync::Notify,
}
impl CanonicalLog for ObservedLog {
    fn service_name(&self) -> &ServiceName {
        self.inner.service_name()
    }
    fn emit(&self, event: LogEvent) -> Result<(), sc_observability_types::v2::EventError> {
        let result = self.inner.emit(event);
        self.emitted.notify_one();
        result
    }
}

async fn shutdown_branch(
    log: SharedCanonicalLog,
    feeds: usize,
    observed: &tokio::sync::Notify,
) -> bool {
    let (worker, release) = WorkerHandle::shutdown_probe(log, feeds);
    if feeds == 0 {
        return worker.shutdown(tokio::time::Instant::now()).await;
    }
    let shutdown = tokio::spawn(worker.shutdown(tokio::time::Instant::now()));
    tokio::time::timeout(Duration::from_secs(2), observed.notified())
        .await
        .unwrap();
    let _ = release.send(());
    tokio::time::timeout(Duration::from_secs(2), shutdown)
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn each_shutdown_branch_emits_once_and_returns_false() {
    for (feeds, action, level, code) in [
        (
            0,
            "shutdown.worker_timeout",
            Level::Error,
            "WORKER_SHUTDOWN_TIMEOUT",
        ),
        (
            1,
            "shutdown.deadline_exceeded",
            Level::Warn,
            "SHUTDOWN_DEADLINE_EXCEEDED",
        ),
    ] {
        let log = recorder();
        let observed = Arc::new(ObservedLog {
            inner: log.clone(),
            emitted: tokio::sync::Notify::new(),
        });
        assert!(!shutdown_branch(observed.clone(), feeds, &observed.emitted).await);
        let events = log.events();
        assert_eq!(events.len(), 1);
        row(&events[0], "cron.scheduler", action, level, "timeout", code);
        assert_eq!(events[0].fields["deadline_remaining_ms"], 0);
        assert_eq!(events[0].fields["worker_count"], 1);
        let rejecting = Arc::new(RejectingCanonicalLog::new(
            ServiceName::new("reject").unwrap(),
        ));
        let observed = Arc::new(ObservedLog {
            inner: rejecting.clone(),
            emitted: tokio::sync::Notify::new(),
        });
        assert!(!shutdown_branch(observed.clone(), feeds, &observed.emitted).await);
        assert_eq!(rejecting.attempts(), 1);
    }
}

#[cfg(all(feature = "monty", unix))]
#[tokio::test]
async fn production_monty_pipe_deadline_uses_injected_logger_and_preserves_failure() {
    use sc_cron::{HookTarget, ManualClock, MontyHostConfig, RunOutcome};
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let program = root.path().join("compose-pipe-holder.py");
    // The leader exits, while one bounded descendant holds stdout past the
    // existing pipe-drain bound. No global environment or PATH is modified.
    std::fs::write(&program, "#!/usr/bin/env python3\nimport subprocess, sys\nsubprocess.Popen([sys.executable, '-c', 'import time; time.sleep(6)'], stdout=sys.stdout, stderr=subprocess.DEVNULL)\nprint('rendered', flush=True)\n").unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let source = "def cron_stop(hook):\n    return {'action': 'continue'}\ndef cron_tick(hook):\n    render_template('x', {})\n";

    for rejecting in [false, true] {
        let recording = recorder();
        let rejected = Arc::new(RejectingCanonicalLog::new(
            ServiceName::new("reject").unwrap(),
        ));
        let logger: SharedCanonicalLog = if rejecting {
            rejected.clone()
        } else {
            recording.clone()
        };
        let clock = Arc::new(ManualClock::new("2026-10-19T00:00:00Z".parse().unwrap()));
        let scheduler = Scheduler::new_with_logger(
            Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap()),
            clock.clone(),
            Arc::new(SeqJitter::new([])),
            logger,
        )
        .with_monty_host(MontyHostConfig::new(&program).unwrap());
        scheduler
            .start(
                ScheduleSpec::with_hook(
                    "pipe-test",
                    "* * * * *",
                    Duration::from_secs(20),
                    serde_json::json!({}),
                    HookTarget::monty(source).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        clock.advance(chrono::Duration::seconds(60));
        struct Idle;
        #[async_trait::async_trait]
        impl sc_cron::Job for Idle {
            async fn run(&self, _: &serde_json::Value, _: std::time::Instant) -> RunOutcome {
                RunOutcome::Ok
            }
            async fn should_stop(&self, _: &serde_json::Value) -> sc_cron::StopDecision {
                sc_cron::StopDecision::Continue
            }
        }
        let runs = tokio::time::timeout(Duration::from_secs(15), scheduler.run_due(Arc::new(Idle)))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(runs.len(), 1);
        assert!(
            matches!(&runs[0].outcome, RunOutcome::Failed(message) if message.contains("COMPOSE_NOT_RUNNABLE"))
        );
        if rejecting {
            assert_eq!(rejected.attempts(), 2); // attachment and pipe deadline
        } else {
            let events: Vec<_> = recording
                .events()
                .into_iter()
                .filter(|event| event.action.as_str() == "pipe.drain_timeout")
                .collect();
            assert_eq!(events.len(), 1);
            row(
                &events[0],
                "cron.monty.compose",
                "pipe.drain_timeout",
                Level::Warn,
                "timeout",
                "COMPOSE_PIPE_DRAIN_TIMEOUT",
            );
            assert_eq!(events[0].fields["deadline_remaining_ms"], 0);
        }
    }
}
