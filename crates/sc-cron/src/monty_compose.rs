//! Deadline wait for the `render_template` compose child.
//!
//! The parent blocks on the child's exit. At the host deadline it kills the
//! process group (Unix) or the process tree (Windows) and reaps the leader.

use std::io::{ErrorKind, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::monty_host::{ComposeHostError, host_budget_exhausted};

/// Waits on the compose child until it exits or `host_deadline` is due.
pub(crate) fn run_compose_child(
    program: &str,
    root: &std::path::Path,
    host_deadline: Option<Instant>,
) -> Result<String, ComposeHostError> {
    let mut command = Command::new(program);
    command
        .current_dir(root)
        .arg("render")
        .arg("--root")
        .arg(root)
        .args(["--mode", "file", "--file", "t.j2", "--var-file", "v.json"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = spawn_grouped(&mut command).map_err(|error| spawn_error(program, error))?;
    let pid = child.id();
    let cap = crate::hook::MAX_HOOK_OUTPUT_BYTES;
    let stdout = read_capped(child.stdout.take(), cap);
    let stderr = read_capped(child.stderr.take(), cap);
    let (sender, receiver) = mpsc::channel();
    let waiter = thread::Builder::new()
        .name("sc-cron-compose-wait".to_owned())
        .spawn(move || {
            let status = child.wait();
            let _ = sender.send(status);
        })
        .map_err(|source| {
            kill_tree(pid);
            ComposeHostError::ProgramNotRunnable {
                program: program.to_owned(),
                source,
            }
        })?;
    match recv_exit(&receiver, host_deadline) {
        ExitWait::Exited(status) => {
            let _ = waiter.join();
            finish_compose(program, status, &stdout, &stderr, host_deadline)
        }
        ExitWait::Expired => {
            kill_tree(pid);
            reap_waiter(&receiver, waiter);
            let _ = stdout.recv_timeout(crate::process::REAP_AFTER_KILL);
            let _ = stderr.recv_timeout(crate::process::REAP_AFTER_KILL);
            Err(ComposeHostError::HostTimedOut {
                program: program.to_owned(),
            })
        }
        ExitWait::Disconnected => {
            let _ = waiter.join();
            Err(ComposeHostError::ProgramNotRunnable {
                program: program.to_owned(),
                source: std::io::Error::other("compose wait ended before the child exited"),
            })
        }
    }
}

enum ExitWait {
    Exited(std::io::Result<std::process::ExitStatus>),
    Expired,
    Disconnected,
}

fn recv_exit(
    receiver: &mpsc::Receiver<std::io::Result<std::process::ExitStatus>>,
    host_deadline: Option<Instant>,
) -> ExitWait {
    let Some(deadline) = host_deadline else {
        return match receiver.recv() {
            Ok(status) => ExitWait::Exited(status),
            Err(_) => ExitWait::Disconnected,
        };
    };
    match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        Ok(status) => ExitWait::Exited(status),
        Err(mpsc::RecvTimeoutError::Disconnected) => ExitWait::Disconnected,
        Err(mpsc::RecvTimeoutError::Timeout) => match receiver.try_recv() {
            Ok(status) => ExitWait::Exited(status),
            Err(_) => ExitWait::Expired,
        },
    }
}

fn finish_compose(
    program: &str,
    status: std::io::Result<std::process::ExitStatus>,
    stdout: &mpsc::Receiver<Drained>,
    stderr: &mpsc::Receiver<Drained>,
    host_deadline: Option<Instant>,
) -> Result<String, ComposeHostError> {
    let status = status.map_err(|source| ComposeHostError::ProgramNotRunnable {
        program: program.to_owned(),
        source,
    })?;
    if host_budget_exhausted(host_deadline) {
        let _ = stdout.recv_timeout(crate::process::REAP_AFTER_KILL);
        let _ = stderr.recv_timeout(crate::process::REAP_AFTER_KILL);
        return Err(ComposeHostError::HostTimedOut {
            program: program.to_owned(),
        });
    }
    let drained = stdout.recv().unwrap_or_else(|_| Drained {
        bytes: Vec::new(),
        over_cap: false,
    });
    let stderr = stderr.recv().unwrap_or_else(|_| Drained {
        bytes: Vec::new(),
        over_cap: false,
    });
    if !status.success() {
        return Err(ComposeHostError::NonZeroExit {
            status: status.code(),
            stderr: crate::monty_host::stderr_text(&stderr.bytes),
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
        if kept.len() >= cap {
            over_cap = true;
            continue;
        }
        let room = cap - kept.len();
        if incoming.len() > room {
            if let Some(prefix) = incoming.get(..room) {
                kept.extend_from_slice(prefix);
            }
            over_cap = true;
        } else {
            kept.extend_from_slice(incoming);
        }
    }
    Drained {
        bytes: kept,
        over_cap,
    }
}

fn read_capped(reader: Option<impl Read + Send + 'static>, cap: usize) -> mpsc::Receiver<Drained> {
    let (sender, receiver) = mpsc::channel();
    let Some(reader) = reader else {
        let _ = sender.send(Drained {
            bytes: Vec::new(),
            over_cap: false,
        });
        return receiver;
    };
    if thread::Builder::new()
        .name("sc-cron-compose-read".to_owned())
        .spawn(move || {
            let _ = sender.send(drain_capped(reader, cap));
        })
        .is_err()
    {
        return mpsc::channel().1;
    }
    receiver
}

fn reap_waiter(
    receiver: &mpsc::Receiver<std::io::Result<std::process::ExitStatus>>,
    waiter: thread::JoinHandle<()>,
) {
    match receiver.recv_timeout(crate::process::REAP_AFTER_KILL) {
        Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
            let _ = waiter.join();
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            tracing::warn!("compose child did not exit after the host kill");
        }
    }
}

fn spawn_grouped(command: &mut Command) -> std::io::Result<std::process::Child> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0).spawn()
    }
    #[cfg(not(unix))]
    {
        command.spawn()
    }
}

fn kill_tree(pid: u32) {
    if pid <= 1 {
        return;
    }
    let mut command = kill_command(pid);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Err(error) = command.status() {
        tracing::debug!(pid, %error, "failed to signal the compose child");
    }
}

#[cfg(unix)]
fn kill_command(pid: u32) -> Command {
    let mut command = Command::new("/bin/kill");
    command.args(["-s", "KILL", "--", &format!("-{pid}")]);
    command
}

#[cfg(not(unix))]
fn kill_command(pid: u32) -> Command {
    let mut command = Command::new("taskkill");
    command.args(["/F", "/T", "/PID", &pid.to_string()]);
    command
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
