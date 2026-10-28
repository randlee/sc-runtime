//! Blocking wait for the `render_template` compose child.
//!
//! The caller is the Monty feed's blocking-pool thread. This function stalls
//! on [`std::process::Child::wait`] until that child exits. It does not stop
//! the child when the row budget elapses. The feed records `TimedOut` only
//! after this wait returns and the host [`std::time::Instant`] is past the
//! row budget.

use std::io::{ErrorKind, Read};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::monty_compose_common::{ComposeHostError, stderr_text};

/// Spawns `program` and waits until it exits.
///
/// Stdout and stderr are read on side threads, each kept up to the hook
/// output cap, so a child that fills a pipe cannot stall [`Child::wait`](std::process::Child::wait).
///
/// # Errors
///
/// [`ComposeHostError::ProgramNotFound`] when the program cannot be found,
/// [`ComposeHostError::ProgramNotRunnable`] when it cannot be started or
/// waited on, [`ComposeHostError::OutputNotUtf8`] when stdout is not UTF-8,
/// [`ComposeHostError::OutputTooLarge`] when stdout exceeds the cap, and
/// [`ComposeHostError::NonZeroExit`] when the child exits non-zero.
pub(crate) fn run_compose_child(
    program: &str,
    root: &std::path::Path,
) -> Result<String, ComposeHostError> {
    let mut child = Command::new(program)
        .current_dir(root)
        .arg("render")
        .arg("--root")
        .arg(root)
        .args(["--mode", "file", "--file", "t.j2", "--var-file", "v.json"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| spawn_error(program, error))?;
    let cap = crate::hook::MAX_HOOK_OUTPUT_BYTES;
    let stdout = match spawn_reader(child.stdout.take(), cap) {
        Ok(reader) => reader,
        Err(source) => {
            let _ = child.wait();
            return Err(not_runnable(program, source));
        }
    };
    let stderr = match spawn_reader(child.stderr.take(), cap) {
        Ok(reader) => reader,
        Err(source) => {
            let _ = child.wait();
            drop(stdout);
            return Err(not_runnable(program, source));
        }
    };
    let status = child
        .wait()
        .map_err(|source| not_runnable(program, source))?;
    finish_compose(program, status, stdout, stderr)
}

struct Drained {
    bytes: Vec<u8>,
    over_cap: bool,
}

fn finish_compose(
    program: &str,
    status: ExitStatus,
    stdout: mpsc::Receiver<Drained>,
    stderr: mpsc::Receiver<Drained>,
) -> Result<String, ComposeHostError> {
    let drained = recv_drained(program, stdout)?;
    let err = recv_drained(program, stderr)?;
    if !status.success() {
        return Err(ComposeHostError::NonZeroExit {
            status: status.code(),
            stderr: stderr_text(&err.bytes),
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

fn recv_drained(
    program: &str,
    receiver: mpsc::Receiver<Drained>,
) -> Result<Drained, ComposeHostError> {
    receiver.recv().map_err(|_| {
        not_runnable(
            program,
            std::io::Error::other("compose output reader ended before sending"),
        )
    })
}

fn drain_capped(mut reader: impl Read, cap: usize) -> Drained {
    let mut kept = Vec::new();
    let mut over_cap = false;
    let mut chunk = [0_u8; 8 * 1024];
    loop {
        let read = match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => read,
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

fn spawn_reader(
    reader: Option<impl Read + Send + 'static>,
    cap: usize,
) -> std::io::Result<mpsc::Receiver<Drained>> {
    let (sender, receiver) = mpsc::channel();
    let Some(reader) = reader else {
        let _ = sender.send(Drained {
            bytes: Vec::new(),
            over_cap: false,
        });
        return Ok(receiver);
    };
    thread::Builder::new()
        .name("sc-cron-compose-read".to_owned())
        .spawn(move || {
            let _ = sender.send(drain_capped(reader, cap));
        })?;
    Ok(receiver)
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
        not_runnable(program, source)
    }
}
