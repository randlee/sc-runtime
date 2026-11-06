//! Platform owners for re-executed integration-test children.
use std::process::{Command, Output};
use std::time::Duration;

#[cfg(unix)]
#[path = "owned_child_unix.rs"]
mod unix;
#[cfg(windows)]
#[path = "owned_child_windows.rs"]
mod windows;

/// Run an owned child with a deadline, bounded capture and physical cleanup.
pub(super) fn output(command: &mut Command, timeout: Duration) -> Output {
    #[cfg(unix)]
    {
        unix::output(command, timeout, || false)
    }
    #[cfg(windows)]
    {
        windows::output(command, timeout, windows::BRIDGE, || false, || {})
    }
}
