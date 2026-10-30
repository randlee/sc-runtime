//! Wait for the spawned `render_template` compose child.
//!
//! The call runs on the blocking pool and stalls on [`std::process::Child::wait`]
//! until the child exits (REQ-CRON-0005). There is no kill at the row or host
//! deadline and no use of [`crate::process::supervise`]. The caller compares the
//! host [`std::time::Instant`] with the row budget after this wait returns.

use std::io::ErrorKind;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::monty_compose_common::{ComposeHostError, stderr_text};

/// Outcome of one allowlisted `sc-compose` subprocess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ComposeRunResult {
    /// Captured stdout (UTF-8 when the child exited zero).
    pub(crate) stdout: String,
    /// Capped stderr text.
    pub(crate) stderr: String,
    /// Process exit code, or `None` when ended by signal.
    pub(crate) exit_code: Option<i32>,
}

impl ComposeRunResult {
    #[must_use]
    pub(crate) const fn success(&self) -> bool {
        matches!(self.exit_code, Some(0))
    }
}

/// Waits on the spawned compose child until it exits.
#[allow(
    dead_code,
    reason = "legacy internal Noop entry point retained for compatibility"
)]
pub(crate) fn run_compose_child(
    program: &str,
    root: &std::path::Path,
) -> Result<String, ComposeHostError> {
    run_compose_child_with_logger(program, root, &crate::noop_logger())
}

pub(crate) fn run_compose_child_with_logger(
    program: &str,
    root: &std::path::Path,
    logger: &sc_observability_types::SharedCanonicalLog,
) -> Result<String, ComposeHostError> {
    let root_arg = root.to_string_lossy().into_owned();
    let args = vec![
        "render".to_owned(),
        "--root".to_owned(),
        root_arg,
        "--mode".to_owned(),
        "file".to_owned(),
        "--file".to_owned(),
        "t.j2".to_owned(),
        "--var-file".to_owned(),
        "v.json".to_owned(),
    ];
    let result = run_compose_argv_with_logger(program, Some(root), &args, logger)?;
    if result.success() {
        return Ok(result.stdout);
    }
    Err(ComposeHostError::NonZeroExit {
        status: result.exit_code,
        stderr: result.stderr,
    })
}

/// Spawns `program` with `args` and blocks in [`std::process::Child::wait`] until it exits.
///
/// Non-zero exit is returned in [`ComposeRunResult`]; spawn errors are
/// [`ComposeHostError`]. On success exit, non-UTF-8 stdout or stdout over the cap is an error.
#[allow(
    dead_code,
    reason = "legacy internal Noop entry point retained for compatibility"
)]
pub(crate) fn run_compose_argv(
    program: &str,
    cwd: Option<&std::path::Path>,
    args: &[String],
) -> Result<ComposeRunResult, ComposeHostError> {
    run_compose_argv_with_logger(program, cwd, args, &crate::noop_logger())
}

pub(crate) fn run_compose_argv_with_logger(
    program: &str,
    cwd: Option<&std::path::Path>,
    args: &[String],
    logger: &sc_observability_types::SharedCanonicalLog,
) -> Result<ComposeRunResult, ComposeHostError> {
    let mut command = Command::new(program);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| spawn_error(program, error))?;
    let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(not_runnable(
            program,
            std::io::Error::other("compose child pipe was missing"),
        ));
    };
    let cap = crate::hook::MAX_HOOK_OUTPUT_BYTES;
    let stdout = match spawn_reader(stdout, cap) {
        Ok(reader) => reader,
        Err(source) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(not_runnable(program, source));
        }
    };
    let stderr = match spawn_reader(stderr, cap) {
        Ok(reader) => reader,
        Err(source) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = join_reader(stdout, logger);
            return Err(not_runnable(program, source));
        }
    };
    let status = child.wait();
    let drained = join_reader(stdout, logger);
    let stderr_drained = join_reader(stderr, logger);
    let status = status.map_err(|source| not_runnable(program, source))?;
    let drained = drained.map_err(|source| not_runnable(program, source))?;
    let stderr_drained = stderr_drained.map_err(|source| not_runnable(program, source))?;
    let stderr = stderr_text(&stderr_drained.bytes);
    let exit_code = status.code();
    if status.success() {
        if drained.truncated {
            return Err(ComposeHostError::OutputTooLarge {
                program: program.to_owned(),
            });
        }
        let stdout =
            String::from_utf8(drained.bytes).map_err(|_| ComposeHostError::OutputNotUtf8 {
                program: program.to_owned(),
            })?;
        return Ok(ComposeRunResult {
            stdout,
            stderr,
            exit_code,
        });
    }
    Ok(ComposeRunResult {
        stdout: String::from_utf8_lossy(&drained.bytes).into_owned(),
        stderr,
        exit_code,
    })
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

/// Collects one pipe after the leader has exited, for at most
/// [`crate::process::REAP_AFTER_KILL`]. A descendant that keeps the pipe open
/// past that bound is an error, not an empty success.
fn join_reader(
    pipe: PipeRead,
    logger: &sc_observability_types::SharedCanonicalLog,
) -> std::io::Result<crate::process::Capped> {
    match pipe.receiver.recv_timeout(crate::process::REAP_AFTER_KILL) {
        Ok(drained) => {
            let _ = pipe.handle.join();
            drained
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            crate::emit_event(
                logger,
                "cron.monty.compose",
                "pipe.drain_timeout",
                sc_observability_types::Level::Warn,
                "timeout",
                Some("COMPOSE_PIPE_DRAIN_TIMEOUT"),
                serde_json::json!({"deadline_remaining_ms": 0})
                    .as_object()
                    .cloned()
                    .unwrap_or_default(),
            );
            Err(std::io::Error::other(
                "compose pipe stayed open after the child exited",
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(std::io::Error::other(
            "compose pipe reader exited without sending output",
        )),
    }
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

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::symlink;
    use std::path::Path;
    use std::time::{Duration, Instant};

    use super::run_compose_child;

    #[test]
    fn wait_blocks_until_the_child_exits_and_returns_its_stdout() {
        let dir = tempfile::TempDir::new().expect("dir");
        let link = dir.path().join("slow-stub");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/compose_slow.sh");
        symlink(fixture, &link).expect("symlink");
        let program = link.to_str().expect("utf8 path");
        let started = Instant::now();
        let rendered = run_compose_child(program, dir.path()).expect("child exit");
        assert_eq!(rendered, "rendered");
        assert!(
            started.elapsed() >= Duration::from_millis(1500),
            "returned before the child exited: {:?}",
            started.elapsed()
        );
    }
}
