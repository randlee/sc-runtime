//! Non-blocking exclusive lock on `<instance-root>/daemon.lock`.

use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::path::Path;
use std::thread::JoinHandle;

use fd_lock::RwLock;

use crate::error::RuntimeError;

const LOCK_FILE: &str = "daemon.lock";

/// Holds `daemon.lock` until dropped. The lock file stays on disk.
pub(crate) struct InstanceLock {
    release: Option<std::sync::mpsc::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Creates the instance root when it is missing and try-locks `daemon.lock`.
///
/// Called only from startup, before the stores closure. A file at `root` is
/// [`RuntimeError::InstanceFailed`]. Lock contention is
/// [`RuntimeError::LockHeld`] and does not wait.
pub(crate) fn acquire(root: &Path) -> Result<InstanceLock, RuntimeError> {
    prepare_root(root)?;
    let path = root.join(LOCK_FILE);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .map_err(|err| RuntimeError::instance_failed(err.to_string()))?;
    let (result_tx, result_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();
    let thread = std::thread::spawn(move || hold(file, result_tx, release_rx));
    match result_rx.recv() {
        Ok(Ok(())) => Ok(InstanceLock {
            release: Some(release_tx),
            thread: Some(thread),
        }),
        Ok(Err(err)) => {
            drop(release_tx);
            let _ = thread.join();
            Err(err)
        }
        Err(err) => {
            drop(release_tx);
            let _ = thread.join();
            Err(RuntimeError::instance_failed(err.to_string()))
        }
    }
}

fn prepare_root(root: &Path) -> Result<(), RuntimeError> {
    if root.exists() && !root.is_dir() {
        return Err(RuntimeError::instance_failed(format!(
            "{} exists and is not a directory",
            root.display()
        )));
    }
    std::fs::create_dir_all(root).map_err(|err| RuntimeError::instance_failed(err.to_string()))
}

fn hold(
    file: File,
    result_tx: std::sync::mpsc::Sender<Result<(), RuntimeError>>,
    release_rx: std::sync::mpsc::Receiver<()>,
) {
    let mut lock = RwLock::new(file);
    let guard = loop {
        match lock.try_write() {
            Ok(guard) => break guard,
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                let _ = result_tx.send(Err(RuntimeError::lock_held(Some(err.to_string()))));
                return;
            }
            Err(err) => {
                let _ = result_tx.send(Err(RuntimeError::instance_failed(err.to_string())));
                return;
            }
        }
    };
    if result_tx.send(Ok(())).is_err() {
        drop(guard);
        return;
    }
    let _ = release_rx.recv();
    drop(guard);
}
