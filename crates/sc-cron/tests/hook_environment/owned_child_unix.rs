//! A pinned session replaces blocking process-group waits on every exit path.
use process_wrap::std::{ChildWrapper, CommandWrap, ProcessSession};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::{Duration, Instant};

const BRIDGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/hook_environment/owned_child_unix.py"
);
const CLEANUP: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(10);

struct Session {
    child: Box<dyn ChildWrapper>,
    cleanup_deadline: Option<Instant>,
    cleaned: bool,
}
impl Session {
    fn stop(&mut self) -> std::io::Result<()> {
        if self.cleaned {
            return Ok(());
        }
        let deadline = *self
            .cleanup_deadline
            .get_or_insert_with(|| Instant::now() + CLEANUP);
        if Instant::now() >= deadline {
            return Err(std::io::Error::other(
                "owned session cleanup deadline exhausted",
            ));
        }
        let mut inspector = Command::new(super::super::python_program())
            .args([BRIDGE, "stop", &self.child.id().to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let status = bounded_wait(&mut inspector, deadline)?;
        if !status.success() {
            return Err(std::io::Error::other(
                "owned session inspection or descendant cleanup failed",
            ));
        }
        // The inspector stops every live session member before killing the
        // still-pinned leader. Only now may Rust reap and release that identity.
        loop {
            if self.child.inner_mut().try_wait()?.is_some() {
                self.cleaned = true;
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(std::io::Error::other("owned session leader did not reap"));
            }
            std::thread::park_timeout(POLL);
        }
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if self.stop().is_err() {
            // Reported failures never become successful completion. A final
            // group kill remains available if session inspection itself fails.
            let _ = self.child.start_kill();
        }
    }
}

fn bounded_wait(child: &mut Child, deadline: Instant) -> std::io::Result<ExitStatus> {
    bounded_wait_with(child, deadline, || false)
}

fn bounded_wait_with(
    child: &mut Child,
    deadline: Instant,
    expire: impl Fn() -> bool,
) -> std::io::Result<ExitStatus> {
    // Reserve part of the existing cleanup budget for physical reaping after
    // a failed inspector; expiry never releases a still-owned helper early.
    let reserve = deadline
        .saturating_duration_since(Instant::now())
        .min(Duration::from_secs(1));
    let work_deadline = deadline - reserve;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= work_deadline || expire() {
            break;
        }
        std::thread::park_timeout(POLL);
    }
    child.kill()?;
    loop {
        if child.try_wait()?.is_some() {
            return Err(std::io::Error::other(
                "owned session inspector exceeded cleanup deadline and was reaped",
            ));
        }
        if Instant::now() >= deadline {
            return Err(std::io::Error::other(
                "owned session inspector could not be reaped",
            ));
        }
        std::thread::park_timeout(POLL);
    }
}

fn read(file: &mut std::fs::File) -> Vec<u8> {
    file.seek(SeekFrom::Start(0)).expect("seek output");
    let mut data = Vec::new();
    file.take(1024 * 1024)
        .read_to_end(&mut data)
        .expect("read output");
    data
}

pub(super) fn output(
    command: &mut Command,
    timeout: Duration,
    expire: impl Fn() -> bool,
) -> Output {
    let root = tempfile::tempdir().expect("owned session request");
    let request = root.path().join("request.json");
    let bytes = |value: &std::ffi::OsStr| value.as_bytes().to_vec();
    let config = serde_json::json!({
        "args": std::iter::once(command.get_program()).chain(command.get_args()).map(bytes).collect::<Vec<_>>(),
        "cwd": command.get_current_dir().map(|path| bytes(path.as_os_str())),
        "env": command.get_envs().map(|(key,value)| (bytes(key),value.map(bytes))).collect::<Vec<_>>(),
    });
    std::fs::write(&request, serde_json::to_vec(&config).expect("request JSON"))
        .expect("request file");
    let mut stdout = tempfile::tempfile().expect("stdout");
    let mut stderr = tempfile::tempfile().expect("stderr");
    let mut launcher = Command::new(super::super::python_program());
    launcher
        .args([BRIDGE, "launch"])
        .arg(&request)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone().expect("stdout clone")))
        .stderr(Stdio::from(stderr.try_clone().expect("stderr clone")));
    let deadline = Instant::now() + timeout;
    let mut wrapped = CommandWrap::from(launcher);
    wrapped.wrap(ProcessSession);
    let mut owned = Session {
        child: wrapped.spawn().expect("owned session"),
        cleanup_deadline: None,
        cleaned: false,
    };
    let status = loop {
        if root.path().join("ready").exists() && !root.path().join("start").exists() {
            std::fs::write(root.path().join("start"), b"start").expect("release launcher");
        }
        if root.path().join("status").exists() {
            let result: serde_json::Value = serde_json::from_slice(
                &std::fs::read(root.path().join("status")).expect("status file"),
            )
            .expect("status JSON");
            let code = i32::try_from(result["returncode"].as_i64().expect("returncode"))
                .expect("returncode range");
            break ExitStatus::from_raw(if code >= 0 { code << 8 } else { -code });
        }
        if Instant::now() >= deadline || expire() {
            break ExitStatus::from_raw(124 << 8);
        }
        std::thread::park_timeout(POLL);
    };
    let cleanup = owned.stop();
    let mut result = Output {
        status,
        stdout: read(&mut stdout),
        stderr: read(&mut stderr),
    };
    if let Err(error) = cleanup {
        result.status = ExitStatus::from_raw(1 << 8);
        result
            .stderr
            .extend_from_slice(format!("owned session cleanup failed: {error}\n").as_bytes());
    }
    result
}

#[cfg(test)]
#[path = "owned_child_unix_tests.rs"]
mod tests;
