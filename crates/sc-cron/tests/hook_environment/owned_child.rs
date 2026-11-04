//! Deadline and cleanup for re-executed integration-test children.
#[cfg(not(windows))]
use process_wrap::std::{ChildWrapper, CommandWrap};
#[cfg(not(windows))]
use std::io::{Read, Seek, SeekFrom};
#[cfg(not(windows))]
use std::process::Stdio;
use std::process::{Command, Output};
use std::time::Duration;
#[cfg(not(windows))]
use std::time::Instant;

#[cfg(not(windows))]
struct OwnedChild(Box<dyn ChildWrapper>);

#[cfg(not(windows))]
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.start_kill();
        let _ = self.0.wait();
    }
}

/// Run an owned child with a wall-clock deadline and bounded output collection.
#[cfg(not(windows))]
pub(super) fn output(command: &mut Command, timeout: Duration) -> Output {
    let mut stdout = tempfile::tempfile().expect("stdout file");
    let mut stderr = tempfile::tempfile().expect("stderr file");
    command.stdout(Stdio::from(stdout.try_clone().expect("stdout clone")));
    command.stderr(Stdio::from(stderr.try_clone().expect("stderr clone")));
    let deadline = Instant::now() + timeout;
    let mut wrapped = CommandWrap::from(std::mem::replace(command, Command::new("")));
    #[cfg(unix)]
    wrapped.wrap(process_wrap::std::ProcessGroup::leader());
    let mut child = OwnedChild(wrapped.spawn().expect("spawn child"));
    let status = loop {
        if let Some(status) = child.0.try_wait().expect("poll child") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "owned child exceeded {timeout:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    fn read(file: &mut std::fs::File) -> Vec<u8> {
        file.seek(SeekFrom::Start(0)).expect("seek output");
        let mut bytes = Vec::new();
        file.take(1024 * 1024)
            .read_to_end(&mut bytes)
            .expect("read output");
        bytes
    }
    Output {
        status,
        stdout: read(&mut stdout),
        stderr: read(&mut stderr),
    }
}

/// The std JobObject wrapper consumes completion notifications during polling,
/// so a later blocking wait can wait forever. Reuse the repository's tree owner:
/// it terminates the job, observes ActiveProcesses == 0, and reaps its launcher.
#[cfg(windows)]
pub(super) fn output(command: &mut Command, timeout: Duration) -> Output {
    let request = tempfile::NamedTempFile::new().expect("owned command request");
    let text = |value: &std::ffi::OsStr| value.to_str().expect("UTF-8 test command").to_owned();
    let config = serde_json::json!({
        "args": std::iter::once(command.get_program()).chain(command.get_args()).map(text).collect::<Vec<_>>(),
        "cwd": command.get_current_dir().map(|path| text(path.as_os_str())),
        "env": command.get_envs().map(|(key, value)| (text(key), value.map(text))).collect::<Vec<_>>(),
        "timeout": timeout.as_secs_f64(),
    });
    std::fs::write(
        request.path(),
        serde_json::to_vec(&config).expect("request JSON"),
    )
    .expect("write request");
    let mut output = Command::new(super::python_program())
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/hook_environment/owned_child.py"
        ))
        .arg(request.path())
        .output()
        .expect("bounded Python owned runner");
    output.stdout.truncate(1024 * 1024);
    output.stderr.truncate(1024 * 1024);
    output
}
