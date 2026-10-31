//! The child-process supervisor: runs one child so that a kill reaches everything it started.
//!
//! This is the process half of the Python hook runner (REQ-CRON-0013,
//! ADR-CRON-0008), exported for any caller that runs a child process of its
//! own (REQ-CRON-0018, ADR-CRON-0009). The child is spawned as the leader of
//! a Unix process group, or inside a Windows job object, so that killing it
//! also kills any grandchild it started: a child that forks a worker cannot
//! outlive its expiry (REQ-CRON-0013). The group or job is killed
//!
//! - when the caller's `expiry` future completes before the child is done, or
//! - when the [`supervise`] future is dropped before the child is reaped, for
//!   instance because the run that awaited it was aborted at its deadline.
//!
//! The child is done when its pipes are closed and it has exited: a child
//! that starts a background process in its group and exits 0 is waited for
//! while that process still holds the pipes, or killed with it at `expiry`.
//! When the child exits on its own, the rest of its group or job is then
//! killed too (best effort), so a member that let go of the pipes does not
//! outlive the run either: no member of the group or job outlives a
//! supervised run. A descendant that leaves the group on its own (`setsid`
//! on Unix; Windows jobs are not left once joined) is out of reach, which is
//! a documented limit. After a kill at `expiry`, the reap of the group is
//! bounded by [`REAP_AFTER_KILL`]; one that does not finish (a member stuck
//! in the kernel) is logged and left, and the group is killed again when the
//! supervisor drops.
//!
//! Standard output and standard error are each kept up to a cap; the rest is
//! read and discarded, so a child that floods a pipe is never blocked on it
//! and never grows this process's memory. Input is written concurrently with
//! reading, so a large input to a child that does not read it cannot stall
//! the supervisor.
//!
//! The child's environment is the caller's choice ([`ChildEnv`]): inherited
//! (the default), or cleared except the variables named, plus those given.
//! Its working directory is the caller's; [`TempCwd`] is a fresh private
//! directory, removed when it drops.
//!
//! The group and job calls come from `process-wrap`, which keeps this crate
//! free of `unsafe`.
//!
//! # Examples
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use std::path::Path;
//! use std::time::Duration;
//!
//! use sc_cron::process::{ChildEnv, ChildSpec, TempCwd, supervise};
//!
//! let cwd = TempCwd::new()?;
//! let spec = ChildSpec {
//!     program: Path::new("/bin/sh"),
//!     args: vec!["-c".into(), "cat; echo done".into()],
//!     cwd: cwd.path(),
//!     input: b"hello\n",
//!     env: ChildEnv::Only { keep: vec!["PATH".into()], set: vec![] },
//! };
//! let expiry = async {
//!     let _ = tokio::time::timeout(Duration::from_secs(5), std::future::pending::<()>()).await;
//! };
//! let captured = supervise(&spec, 1024, expiry).await?;
//! assert_eq!(captured.stdout.bytes, b"hello\ndone\n");
//! # Ok(())
//! # }
//! ```

use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::{ExitStatus, Stdio};

use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::ChildStdin;

#[cfg(windows)]
use process_wrap::tokio::JobObject;
#[cfg(unix)]
use process_wrap::tokio::ProcessGroup;

/// How much of a pipe is read per call.
const READ_CHUNK: usize = 8 * 1024;

/// How long the reap of a killed group or job may take before it is logged
/// and left: a SIGKILL'd member exits at once unless it is stuck in the
/// kernel, so this only bounds that case.
pub const REAP_AFTER_KILL: std::time::Duration = std::time::Duration::from_secs(5);

/// The environment a supervised child starts with.
///
/// `-I` on a Python interpreter only makes it ignore the `PYTHON*`
/// variables, the user site and the working directory on `sys.path`; it
/// does not clear the environment. A caller that must not leak its
/// environment to the child uses [`ChildEnv::Only`].
#[derive(Clone, Default, PartialEq, Eq)]
pub enum ChildEnv {
    /// The child inherits this process's environment.
    #[default]
    Inherit,
    /// The environment is cleared, except the variables named in `keep`
    /// (exact names, each kept when this process has it), then every pair in
    /// `set` is added. A caller that keeps a family such as `LC_*` lists the
    /// matching names of its own environment.
    Only {
        /// Variables kept from this process's environment, by exact name.
        keep: Vec<OsString>,
        /// Variables set for the child, after `keep`.
        set: Vec<(OsString, OsString)>,
    },
}

/// What to run: a program, its arguments, its working directory, its input
/// and its environment.
pub struct ChildSpec<'a> {
    /// The program; an absolute path, or a bare name looked up on `PATH`.
    pub program: &'a Path,
    /// Its arguments.
    pub args: Vec<OsString>,
    /// Its working directory.
    pub cwd: &'a Path,
    /// Written to its stdin while its output is read; stdin is then closed.
    pub input: &'a [u8],
    /// Its environment.
    pub env: ChildEnv,
}

impl std::fmt::Debug for ChildEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inherit => f.write_str("ChildEnv::Inherit"),
            Self::Only { keep, set } => f
                .debug_struct("ChildEnv::Only")
                .field("kept_names", keep)
                .field("set_names", &set.iter().map(|(n, _)| n).collect::<Vec<_>>())
                .finish(),
        }
    }
}
impl std::fmt::Debug for ChildSpec<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChildSpec")
            .field("argument_count", &self.args.len())
            .field("input_bytes", &self.input.len())
            .field("env", &self.env)
            .finish_non_exhaustive()
    }
}

/// Resolves a launch executable using the daemon PATH before child env clearing.
/// Relative components are anchored to the daemon cwd, independently of child cwd.
pub(crate) fn resolve_program(program: &Path) -> io::Result<std::path::PathBuf> {
    if program.is_absolute() {
        return Ok(program.to_path_buf());
    }
    let cwd = std::env::current_dir()?;
    if program.components().count() > 1 {
        return Ok(cwd.join(program));
    }
    let path = std::env::var_os("PATH").ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "daemon launch PATH is unavailable")
    })?;
    #[cfg(windows)]
    let extensions = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
    for dir in std::env::split_paths(&path) {
        let dir = if dir.is_absolute() {
            dir
        } else {
            cwd.join(dir)
        };
        let candidate = dir.join(program);
        if executable(&candidate) {
            return Ok(candidate);
        }
        #[cfg(windows)]
        for extension in extensions.split(';').filter(|s| !s.is_empty()) {
            let mut name = candidate.as_os_str().to_os_string();
            name.push(extension);
            let candidate = std::path::PathBuf::from(name);
            if executable(&candidate) {
                return Ok(candidate);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "configured executable not found on daemon PATH",
    ))
}
fn executable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// A fresh private working directory for a child, removed when dropped.
///
/// The directory is created with the platform's private-temporary-directory
/// rules (mode `0700` on Unix) under the system temporary directory.
#[derive(Debug)]
pub struct TempCwd(tempfile::TempDir);

impl TempCwd {
    /// Creates the directory.
    ///
    /// # Errors
    ///
    /// The directory could not be created.
    pub fn new() -> io::Result<Self> {
        let mut builder = tempfile::Builder::new();
        builder.prefix("sc-cron-child-");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            builder.permissions(std::fs::Permissions::from_mode(0o700));
        }
        builder.tempdir().map(Self)
    }

    /// The directory's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.0.path()
    }
}

/// Up to a cap of one pipe's output, and whether more arrived.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capped {
    /// The bytes kept, at most the cap.
    pub bytes: Vec<u8>,
    /// Whether more bytes arrived than were kept.
    pub truncated: bool,
}

impl Capped {
    fn push(&mut self, data: &[u8], cap: usize) {
        let room = cap.saturating_sub(self.bytes.len());
        let (kept, dropped) = data.split_at(data.len().min(room));
        self.bytes.extend_from_slice(kept);
        self.truncated |= !dropped.is_empty();
    }
}

/// Reads one pipe synchronously, keeping at most `cap` bytes.
///
/// Retries `ErrorKind::Interrupted`. Any other read error is returned so a
/// zero exit cannot be treated as a successful truncated render.
///
/// The Monty compose wait is the only caller. `supervise` reads pipes on the
/// runtime, so this stays out of builds that do not enable `monty`.
#[cfg(feature = "monty")]
pub(crate) fn read_pipe_capped(
    mut reader: impl std::io::Read,
    cap: usize,
) -> std::io::Result<Capped> {
    let mut capped = Capped::default();
    let mut chunk = [0u8; 4096];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => capped.push(&chunk[..n], cap),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(capped)
}

/// What a supervised child left behind.
#[derive(Debug)]
pub struct Captured {
    /// The exit status, or `None` when the child was killed at `expiry`.
    pub status: Option<ExitStatus>,
    /// Its stdout, up to the cap.
    pub stdout: Capped,
    /// Its stderr, up to the cap.
    pub stderr: Capped,
}

/// Kills the child's group or job on drop unless the child was reaped.
struct Tree {
    child: Box<dyn ChildWrapper>,
    reaped: bool,
}

impl Drop for Tree {
    fn drop(&mut self) {
        if !self.reaped {
            // Already gone is fine; a kill that fails here cannot be retried.
            let _ = self.child.start_kill();
        }
    }
}

/// Spawns `spec` in its own group or job and waits for it, up to `expiry`.
///
/// Returns the child's status and its output, each pipe kept up to `cap`
/// bytes, once it and its output pipes are done; or, if `expiry` completes
/// first, kills the whole group or job, reaps it, and returns the output read
/// so far with no status. Dropping the returned future before it completes
/// kills the group or job too (REQ-CRON-0018).
///
/// # Errors
///
/// The spawn failed, or waiting for the child failed (the group is killed
/// before the error is returned).
pub async fn supervise(
    spec: &ChildSpec<'_>,
    cap: usize,
    expiry: impl Future<Output = ()>,
) -> Result<Captured, crate::CronError> {
    supervise_with_logger(spec, cap, expiry, crate::noop_logger()).await
}

/// Supervises a child using a shared best-effort canonical logger.
///
/// # Errors
/// Returns `CronError::HookFailed` for spawn, wait, or reap-timer start failures.
pub async fn supervise_with_logger<F: Future<Output = ()>>(
    spec: &ChildSpec<'_>,
    cap: usize,
    expiry: F,
    logger: sc_observability_types::SharedCanonicalLog,
) -> Result<Captured, crate::CronError> {
    supervise_inner(spec, cap, expiry, logger)
        .await
        .map_err(|error| crate::CronError::HookFailed(error.to_string()))
}

async fn supervise_inner(
    spec: &ChildSpec<'_>,
    cap: usize,
    expiry: impl Future<Output = ()>,
    logger: sc_observability_types::SharedCanonicalLog,
) -> io::Result<Captured> {
    let mut command = CommandWrap::with_new(spec.program, |command| {
        command
            .args(&spec.args)
            .current_dir(spec.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let ChildEnv::Only { keep, set } = &spec.env {
            command.env_clear();
            for name in keep {
                if let Some(value) = std::env::var_os(name) {
                    command.env(name, value);
                }
            }
            command.envs(set.iter().map(|(k, v)| (k, v)));
        }
    });
    command.wrap(KillOnDrop);
    #[cfg(unix)]
    command.wrap(ProcessGroup::leader());
    #[cfg(windows)]
    command.wrap(JobObject);
    let mut tree = Tree {
        child: command.spawn()?,
        reaped: false,
    };
    let stdin = tree.child.stdin().take();
    let stdout = tree.child.stdout().take();
    let stderr = tree.child.stderr().take();
    let (mut out, mut err) = (Capped::default(), Capped::default());

    let finished = {
        let io = async {
            let (status, (), (), ()) = tokio::join!(
                tree.child.wait(),
                feed(stdin, spec.input),
                drain(stdout, &mut out, cap),
                drain(stderr, &mut err, cap),
            );
            status
        };
        tokio::pin!(io);
        tokio::select! {
            biased;
            status = &mut io => Some(status),
            () = expiry => None,
        }
    };
    let status = if let Some(status) = finished {
        let status = status?;
        tree.reaped = true;
        // The leader is gone and the pipes are closed, but a member of its
        // group or job may be left, having let go of the pipes: end it, so
        // nothing outlives the run. An empty group is fine (nothing to kill).
        if let Err(error) = tree.child.start_kill() {
            kill_failed(&logger, tree.child.id(), &error);
        }
        Some(status)
    } else {
        reap_killed(&mut tree, &logger).await?;
        None
    };
    Ok(Captured {
        status,
        stdout: out,
        stderr: err,
    })
}

/// After a kill at expiry, reap the group for at most [`REAP_AFTER_KILL`].
///
/// The bound is a thread timer, not tokio's: a paused or manual tokio clock
/// must not cut the reap short. A group that is not reaped in time is logged
/// and left; drop kills it again. Timer-start failure returns immediately with
/// an error instead of entering an unbounded wait; drop retries the kill.
async fn reap_killed(
    tree: &mut Tree,
    logger: &sc_observability_types::SharedCanonicalLog,
) -> io::Result<()> {
    reap_killed_with_timer(tree, logger, start_reap_timer).await
}

fn start_reap_timer(bound: tokio::sync::oneshot::Sender<()>) -> io::Result<()> {
    std::thread::Builder::new()
        .name("sc-cron-reap-bound".to_owned())
        .spawn(move || {
            std::thread::park_timeout(REAP_AFTER_KILL);
            let _ = bound.send(());
        })
        .map(|_| ())
}

async fn reap_killed_with_timer(
    tree: &mut Tree,
    logger: &sc_observability_types::SharedCanonicalLog,
    start_timer: impl FnOnce(tokio::sync::oneshot::Sender<()>) -> io::Result<()>,
) -> io::Result<()> {
    try_kill(tree, logger);
    let (bound, bounded) = tokio::sync::oneshot::channel::<()>();
    // Fail closed immediately when no real-time bound can be established.
    // The caller drops Tree on this error, retrying group cleanup. Do not
    // report child.reap_timeout: no reap deadline has actually elapsed.
    start_timer(bound).map_err(|error| {
        io::Error::new(error.kind(), format!("reap timer could not start: {error}"))
    })?;
    await_reap(tree, logger, async {
        let _ = bounded.await;
    })
    .await;
    Ok(())
}

async fn await_reap(
    tree: &mut Tree,
    logger: &sc_observability_types::SharedCanonicalLog,
    bounded: impl Future<Output = ()>,
) {
    tokio::select! {
        biased;
        _ = tree.child.wait() => tree.reaped = true,
        () = bounded => {
            let mut fields = serde_json::Map::new();
            fields.insert("deadline_remaining_ms".into(), 0.into());
            if let Some(pid) = tree.child.id() { fields.insert("pid".into(), pid.into()); }
            if let Some(event) = crate::component_event(logger.service_name().clone(), "cron.process", "child.reap_timeout",
                sc_observability_types::Level::Warn, "timeout", Some("CHILD_REAP_TIMEOUT"), fields) { let _ = logger.emit(event); }
        }
    }
}

fn try_kill(tree: &mut Tree, logger: &sc_observability_types::SharedCanonicalLog) {
    if let Err(error) = tree.child.start_kill() {
        kill_failed(logger, tree.child.id(), &error);
    }
}

fn kill_failed(
    logger: &sc_observability_types::SharedCanonicalLog,
    pid: Option<u32>,
    error: &io::Error,
) {
    let mut fields = serde_json::Map::new();
    fields.insert("error".into(), error.to_string().into());
    if let Some(pid) = pid {
        fields.insert("pid".into(), pid.into());
    }
    if let Some(event) = crate::component_event(
        logger.service_name().clone(),
        "cron.process",
        "child.kill_failed",
        sc_observability_types::Level::Debug,
        "failed",
        Some("CHILD_KILL_FAILED"),
        fields,
    ) {
        let _ = logger.emit(event);
    }
}

/// Deterministic process faults for component-contract tests only.
#[cfg(feature = "test-utils")]
#[doc(hidden)]
pub mod testing {
    use super::*;

    #[derive(Debug)]
    struct Unreapable;
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
        fn id(&self) -> Option<u32> {
            Some(42)
        }
        fn start_kill(&mut self) -> io::Result<()> {
            Err(io::Error::other("injected kill failure"))
        }
        fn wait(
            &mut self,
        ) -> std::pin::Pin<Box<dyn Future<Output = io::Result<ExitStatus>> + Send + '_>> {
            Box::pin(std::future::pending())
        }
    }

    /// Injects timer creation failure with a child whose wait never completes.
    pub async fn failed_reap_timer(
        logger: sc_observability_types::SharedCanonicalLog,
    ) -> Result<(), crate::CronError> {
        let mut tree = Tree {
            child: Box::new(Unreapable),
            reaped: false,
        };
        reap_killed_with_timer(&mut tree, &logger, |_| {
            Err(io::Error::other("injected timer-start failure"))
        })
        .await
        .map_err(|error| crate::CronError::HookFailed(error.to_string()))
    }

    /// Returns whether an unreapable fake child was reaped after an immediate bound.
    pub async fn failed_kill_and_reap(logger: sc_observability_types::SharedCanonicalLog) -> bool {
        let mut tree = Tree {
            child: Box::new(Unreapable),
            reaped: false,
        };
        try_kill(&mut tree, &logger);
        await_reap(&mut tree, &logger, std::future::ready(())).await;
        tree.reaped
    }
}

/// Writes `input` to the child and closes its stdin.
///
/// A child that exits or closes stdin without reading it breaks the pipe;
/// its exit status is the answer then, so the write error is not one.
async fn feed(stdin: Option<ChildStdin>, input: &[u8]) {
    let Some(mut stdin) = stdin else { return };
    let _ = stdin.write_all(input).await;
    let _ = stdin.shutdown().await;
}

/// Reads `pipe` to its end, keeping the first `cap` bytes and dropping the rest.
async fn drain<R: AsyncRead + Unpin>(pipe: Option<R>, sink: &mut Capped, cap: usize) {
    let Some(mut pipe) = pipe else { return };
    // On the heap: two of these futures are alive at once, and an array of
    // this size would put their state past what a task should carry.
    let mut chunk = vec![0_u8; READ_CHUNK];
    loop {
        match pipe.read(&mut chunk).await {
            Ok(0) | Err(_) => return,
            Ok(read) => sink.push(chunk.get(..read).unwrap_or_default(), cap),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capped_keeps_the_first_cap_bytes_and_flags_the_rest() {
        let mut capped = Capped::default();
        capped.push(b"abc", 4);
        assert_eq!(
            (capped.bytes.as_slice(), capped.truncated),
            (&b"abc"[..], false)
        );
        capped.push(b"defg", 4);
        assert_eq!(
            (capped.bytes.as_slice(), capped.truncated),
            (&b"abcd"[..], true)
        );
        capped.push(b"h", 4);
        assert_eq!(capped.bytes, b"abcd");
    }

    #[test]
    fn capped_at_exactly_the_cap_is_not_truncated() {
        let mut capped = Capped::default();
        capped.push(b"abcd", 4);
        assert_eq!(
            (capped.bytes.as_slice(), capped.truncated),
            (&b"abcd"[..], false)
        );
    }

    /// A shell script run as the child, in a directory of its own.
    #[cfg(unix)]
    mod unix {
        use std::fs;
        use std::io::Read;
        use std::os::unix::fs::PermissionsExt;
        use std::path::PathBuf;
        use std::process::Command;
        use std::sync::mpsc;
        use std::time::Duration;

        use super::*;

        /// Starts a background process in the child's group that holds the
        /// write end of the named pipe `life`, signals the named pipe `ready`
        /// and sleeps. Nothing forks after the signal, so every member of the
        /// group exists when the test hears it.
        const TREE: &str = r#"#!/bin/sh
cd "$1"
exec 3> life
( echo up > ready; exec sleep 3600 ) &
wait
"#;

        struct Tree {
            dir: tempfile::TempDir,
            script: PathBuf,
        }

        impl Tree {
            fn new() -> Self {
                let dir = tempfile::tempdir().expect("tempdir");
                let script = dir.path().join("tree.sh");
                fs::write(&script, TREE).expect("script");
                fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("chmod");
                for pipe in ["ready", "life"] {
                    let made = Command::new("mkfifo").arg(dir.path().join(pipe)).status();
                    assert!(made.expect("mkfifo").success());
                }
                Self { dir, script }
            }

            fn spec(&self) -> ChildSpec<'_> {
                // `/bin/sh` is the executed file. Executing the script itself
                // races on Linux: a just-written file returns ETXTBSY.
                ChildSpec {
                    program: Path::new("/bin/sh"),
                    args: vec![self.script.clone().into(), self.dir.path().into()],
                    cwd: self.dir.path(),
                    input: b"",
                    env: ChildEnv::Inherit,
                }
            }

            /// Reads `pipe` to its end on a thread of its own: the value
            /// arrives when every writer has closed it.
            fn read(&self, pipe: &str) -> mpsc::Receiver<()> {
                let path = self.dir.path().join(pipe);
                let (sender, receiver) = mpsc::channel();
                std::thread::spawn(move || {
                    let mut sink = Vec::new();
                    if let Ok(mut pipe) = fs::File::open(path) {
                        let _ = pipe.read_to_end(&mut sink);
                    }
                    let _ = sender.send(());
                });
                receiver
            }
        }

        async fn wait(receiver: mpsc::Receiver<()>, what: &str) {
            let heard =
                tokio::task::spawn_blocking(move || receiver.recv_timeout(Duration::from_secs(10)))
                    .await
                    .expect("join");
            assert!(heard.is_ok(), "timed out waiting for {what}");
        }

        #[tokio::test]
        async fn expiry_kills_the_child_and_everything_it_started() {
            let tree = Tree::new();
            let (ready, life) = (tree.read("ready"), tree.read("life"));
            let (expire, expired) = tokio::sync::oneshot::channel::<()>();
            let spec = tree.spec();
            let run = supervise(&spec, 1024, async {
                let _ = expired.await;
            });
            let drive = async {
                wait(ready, "the tree to start").await;
                expire.send(()).expect("supervisor is waiting");
                wait(life, "the whole tree to die").await;
            };
            let (captured, ()) = tokio::join!(run, drive);
            assert!(
                captured.expect("supervise").status.is_none(),
                "killed, no status"
            );
        }

        #[tokio::test]
        async fn dropping_the_supervisor_kills_the_child_and_everything_it_started() {
            let tree = Tree::new();
            let (ready, life) = (tree.read("ready"), tree.read("life"));
            let spec = tree.spec();
            let run = supervise(&spec, 1024, std::future::pending());
            tokio::select! {
                _ = run => panic!("the tree never ends by itself"),
                () = wait(ready, "the tree to start") => {}
            }
            // `run` is dropped; the group dies with it.
            wait(life, "the whole tree to die").await;
        }

        /// A child that starts a background member of its group and exits 0
        /// is not done until that member ends: at `expiry` the whole group is
        /// killed, so nothing it started outlives the run (REQ-CRON-0018).
        /// Planted negative: without the background member the child's exit
        /// ends the run with its status.
        #[tokio::test]
        async fn a_member_left_behind_by_an_exited_child_is_killed_with_the_group() {
            let tree = Tree::new();
            let (ready, life) = (tree.read("ready"), tree.read("life"));
            let script = tree.dir.path().join("leaves.sh");
            fs::write(
                &script,
                "#!/bin/sh\ncd \"$1\"\nexec 3> life\n( echo up > ready; exec sleep 3600 ) &\nexit 0\n",
            )
            .expect("script");
            fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("chmod");
            let spec = ChildSpec {
                program: Path::new("/bin/sh"),
                args: vec![script.into(), tree.dir.path().into()],
                cwd: tree.dir.path(),
                input: b"",
                env: ChildEnv::Inherit,
            };
            let (expire, expired) = tokio::sync::oneshot::channel::<()>();
            let run = supervise(&spec, 1024, async {
                let _ = expired.await;
            });
            let drive = async {
                wait(ready, "the background member to start").await;
                expire.send(()).expect("supervisor is waiting");
                wait(life, "the member to die with the group").await;
            };
            let (captured, ()) = tokio::join!(run, drive);
            assert!(
                captured.expect("supervise").status.is_none(),
                "killed at expiry"
            );
            let alone = ChildSpec {
                program: Path::new("/bin/sh"),
                args: vec!["-c".into(), "exit 0".into()],
                cwd: tree.dir.path(),
                input: b"",
                env: ChildEnv::Inherit,
            };
            let done = supervise(&alone, 64, std::future::pending())
                .await
                .expect("supervise");
            assert_eq!(done.status.and_then(|s| s.code()), Some(0));
        }

        /// A child that starts a member of its group which lets go of the
        /// pipes and exits 0 ends the run at once, and the member is killed
        /// with the rest of the group before `supervise` returns. The member
        /// holds the write end of the named pipe `life`, so the pipe reaches
        /// its end only when the member is gone.
        /// Planted negative: without the kill after the child's exit the
        /// member sleeps on and the test times out.
        #[tokio::test]
        async fn a_member_that_let_go_of_the_pipes_is_killed_when_the_child_exits() {
            let tree = Tree::new();
            let life = tree.read("life");
            let script = tree.dir.path().join("detached.sh");
            fs::write(
                &script,
                "#!/bin/sh\ncd \"$1\"\nexec 3> life\n( exec 0</dev/null >/dev/null 2>&1; exec sleep 3600 ) &\nexit 0\n",
            )
            .expect("script");
            fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("chmod");
            let spec = ChildSpec {
                program: Path::new("/bin/sh"),
                args: vec![script.into(), tree.dir.path().into()],
                cwd: tree.dir.path(),
                input: b"",
                env: ChildEnv::Inherit,
            };
            let captured = supervise(&spec, 1024, std::future::pending())
                .await
                .expect("supervise");
            assert_eq!(captured.status.and_then(|s| s.code()), Some(0));
            wait(life, "the detached member to die with the group").await;
        }

        #[tokio::test]
        async fn a_child_that_exits_reports_its_status_and_output() {
            let dir = tempfile::tempdir().expect("tempdir");
            let spec = ChildSpec {
                program: Path::new("/bin/sh"),
                args: vec!["-c".into(), "cat; echo out; echo err >&2; exit 7".into()],
                cwd: dir.path(),
                input: b"in\n",
                env: ChildEnv::Inherit,
            };
            let captured = supervise(&spec, 1024, std::future::pending())
                .await
                .expect("supervise");
            assert_eq!(captured.status.and_then(|status| status.code()), Some(7));
            assert_eq!(captured.stdout.bytes, b"in\nout\n");
            assert_eq!(captured.stderr.bytes, b"err\n");
            assert!(!captured.stdout.truncated && !captured.stderr.truncated);
        }
    }
}
