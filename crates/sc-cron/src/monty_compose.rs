//! One bounded blocking child driver for both opt-in Monty CLI adapters.
//!
//! This runs on the feed thread, without an async runtime bridge. The child
//! gets a process group/job object, explicit cwd, argv, and empty environment.

use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use process_wrap::std::JobObject;
#[cfg(unix)]
use process_wrap::std::ProcessGroup;
use process_wrap::std::{ChildWrapper, CommandWrap};
use sc_command::observability::SharedCanonicalLog;

use crate::monty_compose_common::ComposeHostError;

const POLL: Duration = Duration::from_millis(2);

/// Completed children return ordinary results even when their exit is nonzero.
#[derive(Debug)]
pub(crate) struct ComposeRunResult {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) exit_code: Option<i32>,
}

/// Owns cleanup on every return, including capture and thread-spawn failures.
struct ChildGuard(Box<dyn ChildWrapper>);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.start_kill();
    }
}

pub(crate) fn run_cli(
    program: &Path,
    cwd: Option<&Path>,
    args: &[String],
    deadline: Instant,
    logger: &SharedCanonicalLog,
    environment: &crate::environment::EnvironmentSnapshot,
    correlation: &crate::worker::RunCorrelation,
) -> Result<ComposeRunResult, ComposeHostError> {
    if Instant::now() >= deadline {
        return Err(ComposeHostError::TimedOut);
    }
    let mut child = spawn_cli(program, cwd, args, environment)?;
    let spawn_error = |source| ComposeHostError::Spawn {
        program: program.display().to_string(),
        source,
    };
    let captures = capture_pipes(&mut *child.0).map_err(|source| {
        reap_after_kill(
            &mut *child.0,
            Instant::now() + crate::process::REAP_AFTER_KILL,
            logger,
            correlation,
        );
        spawn_error(source)
    })?;
    let status = match wait_until(&mut *child.0, deadline) {
        Ok(status) => status,
        Err(source) => {
            reap_after_kill(
                &mut *child.0,
                Instant::now() + crate::process::REAP_AFTER_KILL,
                logger,
                correlation,
            );
            return Err(spawn_error(source));
        }
    };
    if status.is_none() {
        reap_after_kill(
            &mut *child.0,
            Instant::now() + crate::process::REAP_AFTER_KILL,
            logger,
            correlation,
        );
        return Err(ComposeHostError::TimedOut);
    }
    let output = collect_pipes(captures, deadline, logger, correlation).map_err(|source| {
        ComposeHostError::Output {
            program: program.display().to_string(),
            source,
        }
    });
    // Kill remaining members even when the leader exited successfully.
    reap_after_kill(
        &mut *child.0,
        Instant::now() + crate::process::REAP_AFTER_KILL,
        logger,
        correlation,
    );
    if Instant::now() >= deadline {
        return Err(ComposeHostError::TimedOut);
    }
    let (stdout, stderr) = output?;
    Ok(ComposeRunResult {
        stdout,
        stderr,
        exit_code: status.and_then(|status| status.code()),
    })
}

fn spawn_cli(
    program: &Path,
    cwd: Option<&Path>,
    args: &[String],
    environment: &crate::environment::EnvironmentSnapshot,
) -> Result<ChildGuard, ComposeHostError> {
    let spawn_error = |source| ComposeHostError::Spawn {
        program: program.display().to_string(),
        source,
    };
    let resolved = crate::process::resolve_program(program).map_err(spawn_error)?;
    let cwd = cwd.ok_or_else(|| {
        spawn_error(io::Error::other(
            "working directory unavailable at configuration",
        ))
    })?;
    let mut command = Command::new(resolved);
    command
        .current_dir(cwd)
        .args(args)
        .env_clear()
        .envs(environment.iter())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut command = CommandWrap::from(command);
    #[cfg(unix)]
    command.wrap(ProcessGroup::leader());
    #[cfg(windows)]
    command.wrap(JobObject);
    Ok(ChildGuard(command.spawn().map_err(spawn_error)?))
}

type PipeReceiver = mpsc::Receiver<io::Result<crate::process::Capped>>;
fn capture_pipes(child: &mut dyn ChildWrapper) -> io::Result<(PipeReceiver, PipeReceiver)> {
    let stdout = child
        .stdout()
        .take()
        .ok_or_else(|| io::Error::other("stdout pipe missing"))?;
    let stderr = child
        .stderr()
        .take()
        .ok_or_else(|| io::Error::other("stderr pipe missing"))?;
    Ok((spawn_reader(stdout)?, spawn_reader(stderr)?))
}
fn spawn_reader(reader: impl io::Read + Send + 'static) -> io::Result<PipeReceiver> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("sc-cron-cli-read".into())
        .spawn(move || {
            let _ = sender.send(crate::process::read_pipe_capped(
                reader,
                crate::MAX_HOOK_OUTPUT_BYTES,
            ));
        })?;
    Ok(receiver)
}
fn collect_pipes(
    pipes: (PipeReceiver, PipeReceiver),
    deadline: Instant,
    logger: &SharedCanonicalLog,
    correlation: &crate::worker::RunCorrelation,
) -> io::Result<(String, String)> {
    // One absolute collection bound covers both streams and still respects the row.
    let bound = deadline.min(Instant::now() + crate::process::REAP_AFTER_KILL);
    let stdout = read_pipe(pipes.0, bound, logger, correlation)?;
    let stderr = read_pipe(pipes.1, bound, logger, correlation)?;
    Ok((stdout, stderr))
}
fn read_pipe(
    receiver: PipeReceiver,
    deadline: Instant,
    logger: &SharedCanonicalLog,
    correlation: &crate::worker::RunCorrelation,
) -> io::Result<String> {
    let captured = match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        Ok(result) => result?,
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            return Err(io::Error::other("CLI pipe reader disconnected"));
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            if let Some(mut event) = crate::component_event(
                logger.service_name().clone(),
                "cron.monty.compose",
                "pipe.drain_timeout",
                sc_observability_types::Level::Warn,
                "timeout",
                Some("COMPOSE_PIPE_DRAIN_TIMEOUT"),
                serde_json::json!({"deadline_remaining_ms": 0})
                    .as_object()
                    .cloned()
                    .unwrap_or_default(),
            ) {
                event.correlation_id = correlation.clone();
                let _ = logger.emit(event);
            }
            return Err(io::Error::other("CLI pipe drain deadline expired"));
        }
    };
    if captured.truncated {
        return Err(io::Error::other("CLI stream exceeded output cap"));
    }
    String::from_utf8(captured.bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
fn wait_until(child: &mut dyn ChildWrapper, deadline: Instant) -> io::Result<Option<ExitStatus>> {
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Ok(None);
        }
        thread::park_timeout(left.min(POLL));
    }
}
fn reap_after_kill(
    child: &mut dyn ChildWrapper,
    deadline: Instant,
    logger: &SharedCanonicalLog,
    correlation: &crate::worker::RunCorrelation,
) {
    let _ = child.start_kill();
    match wait_until(child, deadline) {
        Ok(None) => {
            if let Some(mut event) = crate::component_event(
                logger.service_name().clone(),
                "cron.monty.compose",
                "child.kill_timeout",
                sc_observability_types::Level::Warn,
                "timeout",
                Some("COMPOSE_CHILD_KILL_TIMEOUT"),
                serde_json::json!({"deadline_remaining_ms": 0, "pid": child.id()})
                    .as_object()
                    .cloned()
                    .unwrap_or_default(),
            ) {
                event.correlation_id = correlation.clone();
                let _ = logger.emit(event);
            }
        }
        Ok(Some(_)) => {}
        // An observation error is not evidence that the reap deadline elapsed.
        // Cleanup remains best effort and does not replace the caller's result.
        Err(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_command::observability::{RecordingCanonicalLog, RejectingCanonicalLog};
    use sc_observability_types::{Level, ServiceName};
    use std::sync::Arc;

    #[derive(Debug)]
    struct Unreapable {
        kills: usize,
    }
    impl ChildWrapper for Unreapable {
        fn inner(&self) -> &dyn ChildWrapper {
            self
        }
        fn inner_mut(&mut self) -> &mut dyn ChildWrapper {
            self
        }
        fn into_inner(self: Box<Self>) -> Box<dyn ChildWrapper> {
            self
        }
        fn id(&self) -> u32 {
            71
        }
        fn start_kill(&mut self) -> io::Result<()> {
            self.kills += 1;
            Ok(())
        }
        fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
            Ok(None)
        }
    }

    #[test]
    fn kill_timeout_event_and_rejection_preserve_cleanup() {
        let recording = Arc::new(RecordingCanonicalLog::new(
            ServiceName::new("driver-test").expect("service"),
        ));
        let logger: SharedCanonicalLog = recording.clone();
        let mut child = Unreapable { kills: 0 };
        reap_after_kill(&mut child, Instant::now(), &logger, &None);
        assert_eq!(child.kills, 1);
        let events = recording.events();
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.target.as_str(), "cron.monty.compose");
        assert_eq!(event.action.as_str(), "child.kill_timeout");
        assert_eq!(event.level, Level::Warn);
        assert_eq!(
            event.outcome.as_ref().map(|outcome| outcome.as_str()),
            Some("timeout")
        );
        assert_eq!(
            event
                .diagnostic
                .as_ref()
                .map(|diagnostic| diagnostic.code.as_str()),
            Some("COMPOSE_CHILD_KILL_TIMEOUT")
        );
        assert_eq!(event.fields["pid"], serde_json::json!(71));
        assert_eq!(event.fields["deadline_remaining_ms"], serde_json::json!(0));
        let rejecting = Arc::new(RejectingCanonicalLog::new(
            ServiceName::new("reject").expect("service"),
        ));
        let logger: SharedCanonicalLog = rejecting.clone();
        let mut child = Unreapable { kills: 0 };
        reap_after_kill(&mut child, Instant::now(), &logger, &None);
        assert_eq!(child.kills, 1);
        assert_eq!(rejecting.attempts(), 1);
    }
    #[derive(Debug)]
    struct WaitError(Unreapable);
    impl ChildWrapper for WaitError {
        fn inner(&self) -> &dyn ChildWrapper {
            self
        }
        fn inner_mut(&mut self) -> &mut dyn ChildWrapper {
            self
        }
        fn into_inner(self: Box<Self>) -> Box<dyn ChildWrapper> {
            self
        }
        fn id(&self) -> u32 {
            self.0.id()
        }
        fn start_kill(&mut self) -> io::Result<()> {
            self.0.start_kill()
        }
        fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
            Err(io::Error::other("injected immediate wait failure"))
        }
    }

    #[test]
    fn immediate_wait_error_does_not_claim_an_elapsed_reap_deadline() {
        let recording = Arc::new(RecordingCanonicalLog::new(
            ServiceName::new("wait-error").expect("service"),
        ));
        let logger: SharedCanonicalLog = recording.clone();
        let mut child = WaitError(Unreapable { kills: 0 });
        reap_after_kill(
            &mut child,
            Instant::now() + Duration::from_secs(60),
            &logger,
            &None,
        );
        assert_eq!(child.0.kills, 1);
        assert!(recording.events().is_empty());
        let rejecting = Arc::new(RejectingCanonicalLog::new(
            ServiceName::new("reject").expect("service"),
        ));
        let logger: SharedCanonicalLog = rejecting.clone();
        reap_after_kill(
            &mut child,
            Instant::now() + Duration::from_secs(60),
            &logger,
            &None,
        );
        assert_eq!(child.0.kills, 2);
        assert_eq!(rejecting.attempts(), 0);
    }

    #[cfg(unix)]
    struct TestChild(Box<dyn ChildWrapper>, Arc<std::sync::atomic::AtomicBool>);
    #[cfg(unix)]
    impl Drop for TestChild {
        fn drop(&mut self) {
            let _ = self.0.start_kill();
            let result = wait_until(
                &mut *self.0,
                Instant::now() + crate::process::REAP_AFTER_KILL,
            );
            self.1.store(
                matches!(result, Ok(Some(_))),
                std::sync::atomic::Ordering::Release,
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn readiness_failure_unwind_kills_and_reaps_the_owned_test_group() {
        let mut command = CommandWrap::from({
            let mut command = Command::new("/bin/sh");
            command.args(["-c", "exec /bin/sleep 20"]);
            command
        });
        command.wrap(ProcessGroup::leader());
        let reaped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let observed = Arc::clone(&reaped);
        let child = TestChild(command.spawn().expect("spawn test group"), observed);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _owned = child;
            panic!("injected readiness assertion failure");
        }));
        assert!(result.is_err());
        assert!(reaped.load(std::sync::atomic::Ordering::Acquire));
    }

    #[cfg(unix)]
    #[test]
    fn a_started_group_is_killed_when_its_row_deadline_expires() {
        use std::io::BufRead;
        let dir = tempfile::tempdir().expect("fixture root");
        let mut command = Command::new("/bin/sh");
        command
            .current_dir(dir.path())
            .env_clear()
            .args([
                "-c",
                "/bin/sleep 20 & printf 'ready\\n'; wait; printf 'finished' > finished",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .stdin(Stdio::null());
        let mut command = CommandWrap::from(command);
        command.wrap(ProcessGroup::leader());
        let mut child = TestChild(
            command.spawn().expect("spawn group"),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
        let stdout = child.0.stdout().take().expect("startup pipe");
        let (ready, observed) = mpsc::sync_channel(1);
        let reader = thread::spawn(move || {
            let mut line = String::new();
            let result = io::BufReader::new(stdout).read_line(&mut line);
            let _ = ready.send((result, line));
        });
        let (read, line) = observed
            .recv_timeout(Duration::from_secs(30))
            .expect("bounded startup handshake");
        read.expect("read handshake");
        assert_eq!(line, "ready\n");
        reader.join().expect("startup reader");
        // Startup is established independently of the expiration assertion.
        // These are the same wait/cleanup helpers used by run_cli; the test
        // never extends a production row deadline or assumes startup latency.
        let deadline = Instant::now() + Duration::from_millis(100);
        assert!(
            wait_until(&mut *child.0, deadline)
                .expect("wait child")
                .is_none()
        );
        let logger = crate::noop_logger();
        reap_after_kill(
            &mut *child.0,
            Instant::now() + crate::process::REAP_AFTER_KILL,
            &logger,
            &None,
        );
        assert!(child.0.try_wait().expect("reaped child").is_some());
        assert!(!dir.path().join("finished").exists());
    }
}
