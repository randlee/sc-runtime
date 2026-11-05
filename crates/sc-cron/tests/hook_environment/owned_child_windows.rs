//! Windows bridge ownership; all waits inspect the std leader, never job ports.
use process_wrap::std::{ChildWrapper, CommandWrap, CommandWrapper, JobObject};
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use waitpid_any::WaitHandle;
use win32job::{ExtendedLimitInfo, Job};

pub(super) const BRIDGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/hook_environment/owned_child.py"
);
// Same fixed cleanup budget as the existing Python owner; not target time.
const CLEANUP: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(10);

// win32job 2.0.3 exposes supported safe job membership queries, including
// nested jobs: https://docs.rs/win32job/2.0.3/win32job/struct.Job.html .
// It is a target-Windows dev dependency only; no SDK edge or unsafe is added.
#[derive(Debug)]
struct Membership(Arc<Job>);
impl CommandWrapper for Membership {
    fn post_spawn(
        &mut self,
        _: &mut Command,
        child: &mut std::process::Child,
        core: &CommandWrap,
    ) -> std::io::Result<()> {
        assert!(
            core.has_wrap::<JobObject>(),
            "assignment requires JobObject suspended spawn"
        );
        // All post_spawn hooks run before JobObject::wrap_child resumes the
        // child. Its first instruction therefore already belongs to this job.
        if let Err(error) = self.0.assign_process(child.as_raw_handle() as isize) {
            child.kill()?;
            let deadline = Instant::now() + CLEANUP;
            while child.try_wait()?.is_none() {
                if Instant::now() >= deadline {
                    return Err(std::io::Error::other(
                        "failed to reap suspended child after job assignment failure",
                    ));
                }
                std::thread::sleep(POLL); // allow: bounded test-only suspended-child cleanup poll
            }
            return Err(std::io::Error::other(error));
        }
        Ok(())
    }
}

struct Bridge {
    child: Box<dyn ChildWrapper>,
    job: Arc<Job>,
    cleaned: bool,
    cleanup_deadline: Option<Instant>,
    members: BTreeMap<usize, Vec<WaitHandle>>,
}
impl Bridge {
    fn spawn(command: Command) -> std::io::Result<Self> {
        let mut limits = ExtendedLimitInfo::new();
        limits.limit_kill_on_job_close();
        let job = Arc::new(Job::create_with_limit_info(&limits).map_err(std::io::Error::other)?);
        let mut wrapped = CommandWrap::from(command);
        wrapped.wrap(Membership(Arc::clone(&job))).wrap(JobObject);
        Ok(Self {
            child: wrapped.spawn()?,
            job,
            cleaned: false,
            cleanup_deadline: None,
            members: BTreeMap::new(),
        })
    }

    fn stop(&mut self) -> std::io::Result<()> {
        self.stop_with(|job| job.query_process_id_list().map_err(std::io::Error::other))
    }

    // The private query seam permits deterministic unavailable/nonempty-proof
    // fault injection. The actual stop path always queries the retained job.
    fn retain_members(&mut self) -> std::io::Result<()> {
        for pid in self
            .job
            .query_process_id_list()
            .map_err(std::io::Error::other)?
        {
            if let Some(identities) = self.members.get_mut(&pid)
                && let Some(latest) = identities.last_mut()
                && latest.wait_timeout(Duration::ZERO)?.is_none()
            {
                continue;
            }
            let native_pid = i32::try_from(pid).map_err(std::io::Error::other)?;
            match WaitHandle::open(native_pid) {
                Ok(handle) => {
                    self.members.entry(pid).or_default().push(handle);
                }
                // ERROR_INVALID_PARAMETER means the process object no longer
                // exists. Access failures are never treated as proof of exit.
                Err(error)
                    if error.raw_os_error() == Some(87)
                        && !self
                            .job
                            .query_process_id_list()
                            .map_err(std::io::Error::other)?
                            .contains(&pid) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn signaled(members: &mut BTreeMap<usize, Vec<WaitHandle>>) -> std::io::Result<bool> {
        let mut complete = true;
        for member in members.values_mut().flatten() {
            complete &= member.wait_timeout(Duration::ZERO)?.is_some();
        }
        Ok(complete)
    }

    fn stop_with(
        &mut self,
        members: impl FnMut(&Job) -> std::io::Result<Vec<usize>>,
    ) -> std::io::Result<()> {
        self.stop_with_signals(members, Self::signaled)
    }

    fn stop_with_signals(
        &mut self,
        mut members: impl FnMut(&Job) -> std::io::Result<Vec<usize>>,
        mut signaled: impl FnMut(&mut BTreeMap<usize, Vec<WaitHandle>>) -> std::io::Result<bool>,
    ) -> std::io::Result<()> {
        if self.cleaned {
            return Ok(());
        }
        let deadline = *self
            .cleanup_deadline
            .get_or_insert_with(|| Instant::now() + CLEANUP);
        // Open while membership still identifies the process, before kill can
        // remove it from accounting. Retain identities until physical signals.
        self.retain_members()?;
        self.child.start_kill()?;
        loop {
            // Capture newly observed members too; an empty membership query by
            // itself does not mean every retained process object is signaled.
            self.retain_members()?;
            let reaped = self.child.inner_mut().try_wait()?.is_some();
            let empty = members(&self.job)?.is_empty();
            let exited = signaled(&mut self.members)?;
            if reaped && empty && exited {
                self.cleaned = true;
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(std::io::Error::other(
                    "bridge members did not physically signal exit before cleanup deadline",
                ));
            }
            std::thread::sleep(POLL); // allow: bounded test-only physical job completion poll
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
    let mut owned = Bridge::spawn(launcher).expect("spawn owned bridge");
    owned
        .retain_members()
        .expect("retain initial bridge identity");
    let mut expired = false;
    let mut status = loop {
        owned.retain_members().expect("retain live bridge members");
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
        std::thread::sleep(POLL); // allow: bounded test-only bridge deadline poll
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
