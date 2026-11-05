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

/// Keep a Rust-owned job around the bridge, including Python import/startup.
#[cfg(windows)]
pub(super) fn output(command: &mut Command, timeout: Duration) -> Output {
    windows::output(command, timeout, windows::BRIDGE, || false, || {})
}

#[cfg(windows)]
#[path = "owned_child_windows.rs"]
mod windows;
