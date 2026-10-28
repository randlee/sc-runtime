//! Deadline wait for the spawned `render_template` compose child.
//!
//! [`crate::process::supervise`] already kills a process group at an expiry.
//! It is an async future. This call runs on the blocking pool, so it cannot
//! drive that future without parking the runtime (ADR-RUN-0011). The wait
//! uses `process-wrap`'s synchronous process group (Unix) or job object
//! (Windows), the same wrappers `supervise` applies, and parks between
//! `try_wait` calls so the deadline can `start_kill` the group. After the
//! leader exits, stdout and stderr are still read only until that deadline.
//! A descendant that keeps a pipe open does not extend the wait: the group
//! or job is killed and the leader is reaped.

use std::io::ErrorKind;
use std::process::{ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use process_wrap::std::JobObject;
#[cfg(unix)]
use process_wrap::std::ProcessGroup;
use process_wrap::std::{ChildWrapper, CommandWrap};

use crate::monty_compose_common::{ComposeHostError, stderr_text};
use crate::monty_host::host_budget_exhausted;

/// How often the compose wait checks the child between parks.
const COMPOSE_POLL: Duration = Duration::from_millis(10);

type GroupChild = Box<dyn ChildWrapper>;

/// Waits on the spawned compose child and kills that child's group at `host_deadline`.
pub(crate) fn run_compose_child(
    program: &str,
    root: &std::path::Path,
    host_deadline: Option<Instant>,
) -> Result<String, ComposeHostError> {
    let mut child = spawn_compose(program, root).map_err(|error| spawn_error(program, error))?;
    let stdout = match child.stdout().take() {
        Some(pipe) => pipe,
        None => {
            return Err(stop_child(
                program,
                &mut child,
                "compose child stdout pipe was missing",
            ));
        }
    };
    let stderr = match child.stderr().take() {
        Some(pipe) => pipe,
        None => {
            return Err(stop_child(
                program,
                &mut child,
                "compose child stderr pipe was missing",
            ));
        }
    };
    let cap = crate::hook::MAX_HOOK_OUTPUT_BYTES;
    let stdout = match spawn_reader(stdout, cap) {
        Ok(reader) => reader,
        Err(source) => {
            let _ = child.start_kill();
            let _ = child.wait();
            return Err(not_runnable(program, source));
        }
    };
    let stderr = match spawn_reader(stderr, cap) {
        Ok(reader) => reader,
        Err(source) => {
            let _ = child.start_kill();
            let _ = child.wait();
            let _ = join_reader(stdout);
            return Err(not_runnable(program, source));
        }
    };
    let mut watch = match watch_child(child) {
        Ok(watch) => watch,
        Err(source) => {
            let _ = join_reader(stdout);
            let _ = join_reader(stderr);
            return Err(not_runnable(program, source));
        }
    };
    match wait_for_exit(&watch, host_deadline) {
        Waited::Exited(status) => {
            finish_watched(program, &mut watch, status, stdout, stderr, host_deadline)
        }
        Waited::Expired => {
            expire_child(&mut watch, stdout, stderr);
            Err(ComposeHostError::HostTimedOut {
                program: program.to_owned(),
            })
        }
        Waited::Lost(source) => {
            expire_child(&mut watch, stdout, stderr);
            Err(not_runnable(program, source))
        }
    }
}

fn spawn_compose(program: &str, root: &std::path::Path) -> std::io::Result<GroupChild> {
    let mut command = CommandWrap::with_new(program, |command| {
        command
            .current_dir(root)
            .arg("render")
            .arg("--root")
            .arg(root)
            .args(["--mode", "file", "--file", "t.j2", "--var-file", "v.json"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
    });
    #[cfg(unix)]
    command.wrap(ProcessGroup::leader());
    #[cfg(windows)]
    command.wrap(JobObject);
    command.spawn()
}

struct PipeRead {
    receiver: mpsc::Receiver<std::io::Result<crate::process::Capped>>,
    handle: thread::JoinHandle<()>,
}

fn spawn_reader(
    reader: impl std::io::Read + Send + 'static,
    cap: usize,
) -> std::io::Result<PipeRead> {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::Builder::new()
        .name("sc-cron-compose-read".to_owned())
        .spawn(move || {
            let _ = sender.send(crate::process::read_pipe_capped(reader, cap));
        })?;
    Ok(PipeRead { receiver, handle })
}

fn join_reader(pipe: PipeRead) -> std::io::Result<crate::process::Capped> {
    match pipe.receiver.recv_timeout(crate::process::REAP_AFTER_KILL) {
        Ok(drained) => {
            let _ = pipe.handle.join();
            drained
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            tracing::warn!("compose pipe was not drained before the reap bound");
            Ok(crate::process::Capped::default())
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(std::io::Error::other(
            "compose pipe reader exited without sending output",
        )),
    }
}

struct ChildWatch {
    child: Arc<Mutex<GroupChild>>,
    stop: Arc<AtomicBool>,
    receiver: mpsc::Receiver<std::io::Result<ExitStatus>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Drop for ChildWatch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
        }
    }
}

fn watch_child(child: GroupChild) -> std::io::Result<ChildWatch> {
    let child = Arc::new(Mutex::new(child));
    let stop = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel();
    let child_thread = Arc::clone(&child);
    let stop_thread = Arc::clone(&stop);
    let spawned = thread::Builder::new()
        .name("sc-cron-compose-wait".to_owned())
        .spawn(move || {
            let status = poll_child(child_thread, &stop_thread);
            let _ = sender.send(status);
        });
    let handle = match spawned {
        Ok(handle) => handle,
        Err(error) => {
            let _ = kill_and_reap(&child);
            return Err(error);
        }
    };
    Ok(ChildWatch {
        child,
        stop,
        receiver,
        handle: Some(handle),
    })
}

fn poll_child(child: Arc<Mutex<GroupChild>>, stop: &AtomicBool) -> std::io::Result<ExitStatus> {
    loop {
        if stop.load(Ordering::Acquire) {
            return kill_and_reap(&child);
        }
        let polled = {
            let mut guard = child.lock().unwrap_or_else(|error| error.into_inner());
            guard.try_wait()
        };
        match polled {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => thread::park_timeout(COMPOSE_POLL),
            Err(error) => return Err(error),
        }
    }
}

fn kill_and_reap(child: &Mutex<GroupChild>) -> std::io::Result<ExitStatus> {
    let mut guard = child.lock().unwrap_or_else(|error| error.into_inner());
    let _ = guard.start_kill();
    guard.wait()
}

enum Waited {
    Exited(std::io::Result<ExitStatus>),
    Expired,
    Lost(std::io::Error),
}

fn wait_for_exit(watch: &ChildWatch, host_deadline: Option<Instant>) -> Waited {
    let Some(deadline) = host_deadline else {
        return match watch.receiver.recv() {
            Ok(status) => Waited::Exited(status),
            Err(_) => Waited::Lost(std::io::Error::other(
                "compose wait ended before the child exited",
            )),
        };
    };
    match watch
        .receiver
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
    {
        Ok(status) => Waited::Exited(status),
        Err(mpsc::RecvTimeoutError::Timeout) => Waited::Expired,
        Err(mpsc::RecvTimeoutError::Disconnected) => Waited::Lost(std::io::Error::other(
            "compose wait ended before the child exited",
        )),
    }
}

enum Take {
    Ready(crate::process::Capped),
    Expired(PipeRead),
    ReadFailed(std::io::Error),
}

/// Reads one pipe until it closes, or until `deadline`.
///
/// A zero remaining budget does not read. The caller kills the group when
/// this returns [`Take::Expired`].
fn take_pipe(pipe: PipeRead, deadline: Instant) -> Take {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Take::Expired(pipe);
    }
    match pipe.receiver.recv_timeout(remaining) {
        Ok(Ok(drained)) => {
            let _ = pipe.handle.join();
            Take::Ready(drained)
        }
        Ok(Err(source)) => {
            let _ = pipe.handle.join();
            Take::ReadFailed(source)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => Take::Expired(pipe),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            let _ = pipe.handle.join();
            Take::ReadFailed(std::io::Error::other(
                "compose pipe reader exited without sending output",
            ))
        }
    }
}

/// Kills the compose group or job and reaps the leader after a pipe outlasted
/// the host deadline. Reader threads are joined only up to
/// [`crate::process::REAP_AFTER_KILL`], once the kill can close the pipes.
fn budget_expired(
    program: &str,
    watch: &mut ChildWatch,
    pending: Vec<PipeRead>,
) -> ComposeHostError {
    let _ = kill_and_reap(&watch.child);
    for pipe in pending {
        let _ = join_reader(pipe);
    }
    ComposeHostError::HostTimedOut {
        program: program.to_owned(),
    }
}

fn drain_outputs(
    program: &str,
    watch: &mut ChildWatch,
    stdout: PipeRead,
    stderr: PipeRead,
    host_deadline: Option<Instant>,
) -> Result<(crate::process::Capped, crate::process::Capped), ComposeHostError> {
    let Some(deadline) = host_deadline else {
        let _ = kill_and_reap(&watch.child);
        let stdout = join_reader(stdout).map_err(|source| not_runnable(program, source))?;
        let stderr = join_reader(stderr).map_err(|source| not_runnable(program, source))?;
        return Ok((stdout, stderr));
    };
    let drained = match take_pipe(stdout, deadline) {
        Take::Ready(drained) => drained,
        Take::Expired(stdout) => {
            return Err(budget_expired(program, watch, vec![stdout, stderr]));
        }
        Take::ReadFailed(source) => return Err(not_runnable(program, source)),
    };
    match take_pipe(stderr, deadline) {
        Take::Ready(stderr_drained) => {
            let _ = kill_and_reap(&watch.child);
            Ok((drained, stderr_drained))
        }
        Take::Expired(stderr) => Err(budget_expired(program, watch, vec![stderr])),
        Take::ReadFailed(source) => Err(not_runnable(program, source)),
    }
}

fn finish_watched(
    program: &str,
    watch: &mut ChildWatch,
    status: std::io::Result<ExitStatus>,
    stdout: PipeRead,
    stderr: PipeRead,
    host_deadline: Option<Instant>,
) -> Result<String, ComposeHostError> {
    join_watch(watch);
    let (drained, stderr_drained) = drain_outputs(program, watch, stdout, stderr, host_deadline)?;
    let status = status.map_err(|source| not_runnable(program, source))?;
    if host_budget_exhausted(host_deadline) {
        return Err(ComposeHostError::HostTimedOut {
            program: program.to_owned(),
        });
    }
    if !status.success() {
        return Err(ComposeHostError::NonZeroExit {
            status: status.code(),
            stderr: stderr_text(&stderr_drained.bytes),
        });
    }
    if drained.truncated {
        return Err(ComposeHostError::OutputTooLarge {
            program: program.to_owned(),
        });
    }
    String::from_utf8(drained.bytes).map_err(|_| ComposeHostError::OutputNotUtf8 {
        program: program.to_owned(),
    })
}

fn expire_child(watch: &mut ChildWatch, stdout: PipeRead, stderr: PipeRead) {
    watch.stop.store(true, Ordering::Release);
    if let Some(handle) = watch.handle.as_ref() {
        handle.thread().unpark();
    }
    match watch.receiver.recv_timeout(crate::process::REAP_AFTER_KILL) {
        Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => join_watch(watch),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            tracing::warn!("compose child did not exit after the host kill");
        }
    }
    let _ = join_reader(stdout);
    let _ = join_reader(stderr);
}

fn join_watch(watch: &mut ChildWatch) {
    if let Some(handle) = watch.handle.take() {
        let _ = handle.join();
    }
}

fn stop_child(program: &str, child: &mut GroupChild, detail: &str) -> ComposeHostError {
    let _ = child.start_kill();
    let _ = child.wait();
    not_runnable(program, std::io::Error::other(detail))
}

fn not_runnable(program: &str, source: std::io::Error) -> ComposeHostError {
    ComposeHostError::ProgramNotRunnable {
        program: program.to_owned(),
        source,
    }
}

fn spawn_error(program: &str, source: std::io::Error) -> ComposeHostError {
    if source.kind() == ErrorKind::NotFound {
        ComposeHostError::ProgramNotFound {
            program: program.to_owned(),
        }
    } else {
        ComposeHostError::ProgramNotRunnable {
            program: program.to_owned(),
            source,
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::symlink;
    use std::path::Path;
    use std::process::Command;
    use std::time::{Duration, Instant};

    use super::run_compose_child;
    use crate::monty_compose_common::ComposeHostError;

    #[test]
    fn a_grandchild_holding_a_pipe_cannot_stall_past_the_host_deadline() {
        let dir = tempfile::TempDir::new().expect("dir");
        let link = dir.path().join("hold-stub");
        let fixture =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/compose_hold_pipe.sh");
        symlink(fixture, &link).expect("symlink");
        let program = link.to_str().expect("utf8 path");
        let started = Instant::now();
        let deadline = started + Duration::from_millis(1500);
        let error = run_compose_child(program, dir.path(), Some(deadline)).expect_err("held pipe");
        let elapsed = started.elapsed();
        assert!(
            matches!(error, ComposeHostError::HostTimedOut { .. }),
            "{error}"
        );
        assert!(
            (Duration::from_millis(1000)..Duration::from_secs(8)).contains(&elapsed),
            "pipe drain ran for {elapsed:?}"
        );
        assert!(
            dir.path().join("child.pid").is_file(),
            "grandchild did not start"
        );
        assert_dead(dir.path(), "leader.pid");
        assert_dead(dir.path(), "child.pid");
    }

    fn assert_dead(dir: &Path, name: &str) {
        let text = std::fs::read_to_string(dir.join(name)).expect(name);
        let pid: u32 = text.trim().parse().expect("pid");
        let alive = Command::new("/bin/kill")
            .args(["-0", &pid.to_string()])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        assert!(!alive, "{name} {pid} still alive");
    }
}
