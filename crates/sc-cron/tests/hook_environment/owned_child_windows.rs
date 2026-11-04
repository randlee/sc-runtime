//! Windows bridge ownership; all waits inspect the std leader, never job ports.
use process_wrap::std::{ChildWrapper, CommandWrap, JobObject};
use std::io::{Read, Seek, SeekFrom};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::time::{Duration, Instant};

pub(super) const BRIDGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/hook_environment/owned_child.py"
);
// Same fixed cleanup budget as the existing Python owner; not target time.
const CLEANUP: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(10);

struct Bridge {
    child: Box<dyn ChildWrapper>,
    cleaned: bool,
}
impl Bridge {
    fn stop(&mut self) -> std::io::Result<()> {
        if self.cleaned {
            return Ok(());
        }
        self.cleaned = true;
        self.child.start_kill()?;
        let deadline = Instant::now() + CLEANUP;
        loop {
            // JobObject::try_wait consumes completion notifications. Only the
            // underlying std child is polled; no second job-port wait exists.
            if self.child.inner_mut().try_wait()?.is_some() {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(std::io::Error::other(
                    "bridge leader did not exit after job termination",
                ));
            }
            std::thread::sleep(POLL);
        }
    }
}
impl Drop for Bridge {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn read(file: &mut std::fs::File) -> Vec<u8> {
    file.seek(SeekFrom::Start(0)).expect("seek bridge output");
    let mut bytes = Vec::new();
    file.take(1024 * 1024)
        .read_to_end(&mut bytes)
        .expect("read bridge output");
    bytes
}

/// Private seams inject expiry and inspect completion while ownership is held.
/// The real caller deadline and owning cleanup remain active in every case.
pub(super) fn output(
    command: &mut Command,
    timeout: Duration,
    bridge: impl AsRef<Path>,
    expire: impl Fn() -> bool,
    completed: impl FnOnce(),
) -> Output {
    let deadline = Instant::now() + timeout;
    let request = tempfile::NamedTempFile::new().expect("owned command request");
    let units = |value: &std::ffi::OsStr| value.encode_wide().collect::<Vec<_>>();
    let config = serde_json::json!({
        "args_utf16": std::iter::once(command.get_program()).chain(command.get_args()).map(units).collect::<Vec<_>>(),
        "cwd_utf16": command.get_current_dir().map(|path| units(path.as_os_str())),
        "env_utf16": command.get_envs().map(|(key, value)| (units(key), value.map(units))).collect::<Vec<_>>(),
        "timeout": timeout.as_secs_f64(),
    });
    std::fs::write(
        request.path(),
        serde_json::to_vec(&config).expect("request JSON"),
    )
    .expect("write request");
    let mut stdout = tempfile::tempfile().expect("bridge stdout");
    let mut stderr = tempfile::tempfile().expect("bridge stderr");
    let mut launcher = Command::new(super::super::python_program());
    launcher
        .arg(bridge.as_ref())
        .arg(request.path())
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone().expect("stdout clone")))
        .stderr(Stdio::from(stderr.try_clone().expect("stderr clone")));
    let mut wrapped = CommandWrap::from(launcher);
    wrapped.wrap(JobObject);
    let mut owned = Bridge {
        child: wrapped.spawn().expect("spawn owned bridge"),
        cleaned: false,
    };
    let mut expired = false;
    let mut status = loop {
        if let Some(status) = owned
            .child
            .inner_mut()
            .try_wait()
            .expect("poll bridge leader")
        {
            break status;
        }
        if Instant::now() >= deadline || expire() {
            expired = true;
            break ExitStatus::from_raw(124);
        }
        std::thread::sleep(POLL);
    };
    // A terminal bridge is inspected before emergency cleanup can conceal a
    // leaked target. Expiry instead requires cleanup before any observation.
    let first_cleanup = if expired { owned.stop() } else { Ok(()) };
    completed();
    let cleanup = first_cleanup.and_then(|()| owned.stop());
    let mut output = Output {
        status,
        stdout: read(&mut stdout),
        stderr: read(&mut stderr),
    };
    if expired {
        output
            .stderr
            .extend_from_slice(b"owned bridge exceeded its deadline\n");
    }
    if let Err(error) = cleanup {
        status = ExitStatus::from_raw(1);
        output.status = status;
        output
            .stderr
            .extend_from_slice(format!("owned bridge cleanup failed: {error}\n").as_bytes());
    }
    output
}

#[path = "owned_child_windows_tests.rs"]
mod tests;
