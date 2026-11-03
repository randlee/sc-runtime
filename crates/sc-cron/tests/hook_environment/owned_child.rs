//! Deadline and cleanup for re-executed integration-test children.
use process_wrap::std::{ChildWrapper, CommandWrap};
use std::io::{Read, Seek, SeekFrom};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

struct OwnedChild(Box<dyn ChildWrapper>);

impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.start_kill();
        let _ = self.0.wait();
    }
}

/// Run an owned child with a wall-clock deadline and bounded output collection.
pub(super) fn output(command: &mut Command, timeout: Duration) -> Output {
    let mut stdout = tempfile::tempfile().expect("stdout file");
    let mut stderr = tempfile::tempfile().expect("stderr file");
    command.stdout(Stdio::from(stdout.try_clone().expect("stdout clone")));
    command.stderr(Stdio::from(stderr.try_clone().expect("stderr clone")));
    let deadline = Instant::now() + timeout;
    let mut wrapped = CommandWrap::from(std::mem::replace(command, Command::new("")));
    #[cfg(unix)]
    wrapped.wrap(process_wrap::std::ProcessGroup::leader());
    #[cfg(windows)]
    wrapped.wrap(process_wrap::std::JobObject);
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
