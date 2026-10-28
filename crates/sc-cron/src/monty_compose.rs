//! Deadline wait for the spawned `render_template` compose child.
//!
//! [`crate::process::supervise`] already kills a process group at an expiry.
//! It is an async future. This call runs on the blocking pool, so it cannot
//! drive that future without parking the runtime (ADR-RUN-0011). The wait
//! uses `process-wrap`'s synchronous process group (Unix) or job object
//! (Windows), the same wrappers `supervise` applies, and parks between
//! `try_wait` calls so the deadline can `start_kill` the group.

use std::io::{ErrorKind, Read};
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

use crate::monty_host::{ComposeHostError, host_budget_exhausted};

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
    receiver: mpsc::Receiver<Drained>,
    handle: thread::JoinHandle<()>,
}

fn spawn_reader(reader: impl Read + Send + 'static, cap: usize) -> std::io::Result<PipeRead> {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::Builder::new()
        .name("sc-cron-compose-read".to_owned())
        .spawn(move || {
            let _ = sender.send(drain_capped(reader, cap));
        })?;
    Ok(PipeRead { receiver, handle })
}

fn join_reader(pipe: PipeRead) -> Drained {
    match pipe.receiver.recv_timeout(crate::process::REAP_AFTER_KILL) {
        Ok(drained) => {
            let _ = pipe.handle.join();
            drained
        }
        Err(_) => {
            tracing::warn!("compose pipe was not drained before the reap bound");
            Drained {
                bytes: Vec::new(),
                over_cap: false,
            }
        }
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

fn finish_watched(
    program: &str,
    watch: &mut ChildWatch,
    status: std::io::Result<ExitStatus>,
    stdout: PipeRead,
    stderr: PipeRead,
    host_deadline: Option<Instant>,
) -> Result<String, ComposeHostError> {
    join_watch(watch);
    let _ = kill_and_reap(&watch.child);
    let drained = join_reader(stdout);
    let stderr_drained = join_reader(stderr);
    let status = status.map_err(|source| not_runnable(program, source))?;
    if host_budget_exhausted(host_deadline) {
        return Err(ComposeHostError::HostTimedOut {
            program: program.to_owned(),
        });
    }
    if !status.success() {
        return Err(ComposeHostError::NonZeroExit {
            status: status.code(),
            stderr: crate::monty_host::stderr_text(&stderr_drained.bytes),
        });
    }
    if drained.over_cap {
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

struct Drained {
    bytes: Vec<u8>,
    over_cap: bool,
}

fn drain_capped(mut reader: impl Read, cap: usize) -> Drained {
    let mut kept = Vec::new();
    let mut over_cap = false;
    let mut chunk = [0u8; 4096];
    loop {
        let read = match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        let Some(incoming) = chunk.get(..read) else {
            break;
        };
        let room = cap.saturating_sub(kept.len());
        let (head, rest) = incoming.split_at(incoming.len().min(room));
        kept.extend_from_slice(head);
        if !rest.is_empty() {
            over_cap = true;
        }
    }
    Drained {
        bytes: kept,
        over_cap,
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
