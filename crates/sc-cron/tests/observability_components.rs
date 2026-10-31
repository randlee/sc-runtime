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

#[tokio::test]
async fn reap_timer_start_failure_returns_without_waiting_or_claiming_elapsed_deadline() {
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
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            sc_cron::process::testing::failed_reap_timer(logger),
        )
        .await
        .expect("timer-start failure must not await an unreapable child");
        let error = result.unwrap_err();
        assert_eq!(error.code(), "HOOK_FAILED");
        assert!(error.to_string().contains("reap timer could not start"));
        if rejecting {
            assert_eq!(rejected.attempts(), 1);
        } else {
            let events = recording.events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].action.as_str(), "child.kill_failed");
            assert!(
                !events
                    .iter()
                    .any(|event| event.action.as_str() == "child.reap_timeout")
            );
        }
    }
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
struct PipeHolder {
    child: std::process::Child,
    release: Arc<std::sync::Mutex<std::process::ChildStdin>>,
    cancel: std::sync::mpsc::Sender<()>,
    watchdog: Option<std::thread::JoinHandle<()>>,
}

#[cfg(all(feature = "monty", unix))]
fn kill_holder_group(pid: u32) {
    let _ = std::process::Command::new("/bin/kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

#[cfg(all(feature = "monty", unix))]
impl PipeHolder {
    fn start(socket: &std::path::Path) -> Self {
        use std::io::BufRead;
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};

        // The test owns this child directly, so every path can kill and reap
        // it. The production compose child transfers its stdout descriptor
        // only after the listener is ready, then waits for a holder ACK.
        let script = r#"import array, os, socket, sys
listener = socket.socket(socket.AF_UNIX)
listener.bind(sys.argv[1])
listener.listen(1)
print('ready', flush=True)
connection, _ = listener.accept()
_, ancillary, _, _ = connection.recvmsg(1, socket.CMSG_SPACE(array.array('i').itemsize))
fds = array.array('i')
fds.frombytes(ancillary[0][2][:fds.itemsize])
connection.sendall(b'held')
sys.stdin.buffer.read(1)
for fd in fds:
    os.close(fd)
"#;
        let mut child = Command::new("python3")
            .args(["-c", script])
            .arg(socket)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .process_group(0)
            .spawn()
            .unwrap();
        let pid = child.id();
        let (cancel, cancelled) = std::sync::mpsc::channel();
        let watchdog = std::thread::spawn(move || {
            if cancelled.recv_timeout(Duration::from_secs(25)).is_err() {
                kill_holder_group(pid);
            }
        });
        let release = Arc::new(std::sync::Mutex::new(child.stdin.take().unwrap()));
        let stdout = child.stdout.take().unwrap();
        let guard = Self {
            child,
            release,
            cancel,
            watchdog: Some(watchdog),
        };
        let (ready, readiness) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            let mut line = String::new();
            let result = std::io::BufReader::new(stdout).read_line(&mut line);
            let _ = ready.send((result, line));
        });
        let (result, line) = readiness.recv_timeout(Duration::from_secs(5)).unwrap();
        result.unwrap();
        assert_eq!(line.trim(), "ready");
        reader.join().unwrap();
        guard
    }
}

#[cfg(all(feature = "monty", unix))]
impl Drop for PipeHolder {
    fn drop(&mut self) {
        // Unconditional cleanup also handles panic, failed assertions, expiry,
        // and failure before the pipe deadline can be observed.
        let _ = self.cancel.send(());
        kill_holder_group(self.child.id());
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(watchdog) = self.watchdog.take() {
            let _ = watchdog.join();
        }
    }
}

#[cfg(all(feature = "monty", unix))]
struct PipeReleaseLog {
    inner: SharedCanonicalLog,
    release: Arc<std::sync::Mutex<std::process::ChildStdin>>,
    deadlines: std::sync::atomic::AtomicUsize,
}

#[cfg(all(feature = "monty", unix))]
impl CanonicalLog for PipeReleaseLog {
    fn service_name(&self) -> &ServiceName {
        self.inner.service_name()
    }

    fn emit(&self, event: LogEvent) -> Result<(), sc_observability_types::v2::EventError> {
        use std::io::Write;
        let deadline = event.target.as_str() == "cron.monty.compose"
            && event.action.as_str() == "pipe.drain_timeout";
        let result = self.inner.emit(event);
        if deadline {
            self.deadlines
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            // Release even when the inner logger rejects the event.
            let _ = self.release.lock().unwrap().write_all(b"x");
        }
        result
    }
}

#[cfg(all(feature = "monty", unix))]
#[tokio::test]
async fn production_monty_pipe_deadline_uses_injected_logger_and_preserves_failure() {
    use sc_cron::{HookTarget, ManualClock, MontyHostRegistry, RunOutcome, ScComposeHostConfig};
    use std::os::unix::fs::PermissionsExt;

    let python = std::process::Command::new("python3")
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .unwrap();
    let interpreter = String::from_utf8(python.stdout).unwrap();
    let source = "def cron_stop(hook):\n    return {'action': 'continue'}\ndef cron_tick(hook):\n    sc_compose_run('render', [])\n";

    for rejecting in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let socket = root.path().join("pipe.sock");
        let holder = PipeHolder::start(&socket);
        let program = root.path().join("compose-pipe-holder.py");
        let socket_literal = serde_json::to_string(socket.to_str().unwrap()).unwrap();
        std::fs::write(&program, format!(r#"#!{interpreter}
import array, socket, sys
connection = socket.socket(socket.AF_UNIX)
connection.connect({socket_literal})
connection.sendmsg([b'x'], [(socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', [sys.stdout.fileno()]))])
assert connection.recv(4) == b'held'
print('rendered', flush=True)
"#, interpreter=interpreter.trim())).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        let recording = recorder();
        let rejected = Arc::new(RejectingCanonicalLog::new(
            ServiceName::new("reject").unwrap(),
        ));
        let inner: SharedCanonicalLog = if rejecting {
            rejected.clone()
        } else {
            recording.clone()
        };
        let observed = Arc::new(PipeReleaseLog {
            inner,
            release: holder.release.clone(),
            deadlines: std::sync::atomic::AtomicUsize::new(0),
        });
        let clock = Arc::new(ManualClock::new("2026-10-19T00:00:00Z".parse().unwrap()));
        let scheduler = Scheduler::new_with_logger(
            Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap()),
            clock.clone(),
            Arc::new(SeqJitter::new([])),
            observed.clone(),
        )
        .with_monty_hosts(
            MontyHostRegistry::new()
                .with_sc_compose(ScComposeHostConfig::new(&program).unwrap())
                .unwrap(),
        );
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
        let runs = tokio::time::timeout(Duration::from_secs(20), scheduler.run_due(Arc::new(Idle)))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(runs.len(), 1);
        assert!(
            matches!(&runs[0].outcome, RunOutcome::Failed(message) if message.contains("HOST_OUTPUT"))
        );
        assert_eq!(
            observed.deadlines.load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        if rejecting {
            assert_eq!(rejected.attempts(), 4); // attached + due.launched + pipe deadline + due.finished
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
            let all = recording.events();
            let launch = all
                .iter()
                .find(|event| event.action.as_str() == "due.launched")
                .unwrap();
            let finish = all
                .iter()
                .find(|event| event.action.as_str() == "due.finished")
                .unwrap();
            assert!(launch.correlation_id.is_some());
            assert_eq!(launch.correlation_id, events[0].correlation_id);
            assert_eq!(launch.correlation_id, finish.correlation_id);
            assert!(
                all.iter()
                    .find(|event| event.action.as_str() == "logger.attached")
                    .unwrap()
                    .correlation_id
                    .is_none()
            );
            for event in all {
                assert!(
                    !event.fields.contains_key("run_id")
                        && !event.fields.contains_key("correlation_id")
                );
            }
        }
    }
}
