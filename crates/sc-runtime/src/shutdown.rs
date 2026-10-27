//! SIGINT, SIGTERM, and the fixture's private shutdown trigger.

use std::time::Duration;

use tokio::sync::oneshot;

use crate::error::RuntimeError;

/// How long in-flight requests may finish after shutdown starts.
pub(crate) const DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

/// Extra time after [`DRAIN_TIMEOUT`] for connections to close once handlers return.
pub(crate) const DRAIN_BACKSTOP: Duration = Duration::from_secs(1);

/// How `run` waits before it stops accepting connections.
pub(crate) struct RunControl {
    pub(crate) ready: Option<oneshot::Sender<sc_transport::Endpoint>>,
    pub(crate) shutdown: Option<oneshot::Receiver<()>>,
    /// Production listens for process signals. The in-process fixture does not,
    /// because a process signal would stop every fixture in the test binary.
    /// The shutdown sequence after the trigger is the same either way.
    pub(crate) listen_for_signals: bool,
    /// Bound on in-flight work after the shutdown trigger.
    pub(crate) drain: Duration,
}

impl RunControl {
    pub(crate) fn production() -> Self {
        Self {
            ready: None,
            shutdown: None,
            listen_for_signals: true,
            drain: DRAIN_TIMEOUT,
        }
    }

    /// Private trigger, no process signals, production drain bound.
    #[cfg(feature = "test-util")]
    pub(crate) fn detached(
        ready: Option<oneshot::Sender<sc_transport::Endpoint>>,
        shutdown: Option<oneshot::Receiver<()>>,
    ) -> Self {
        Self {
            ready,
            shutdown,
            listen_for_signals: false,
            drain: DRAIN_TIMEOUT,
        }
    }
}

/// Installed process-signal listeners. Absent when the fixture owns shutdown.
pub(crate) struct Signals {
    #[cfg(unix)]
    interrupt: tokio::signal::unix::Signal,
    #[cfg(unix)]
    terminate: tokio::signal::unix::Signal,
    /// Same listener polled during install. A later `ctrl_c()` call would
    /// miss a Ctrl-C that arrived while the daemon was locking or binding.
    #[cfg(windows)]
    ctrl_c: tokio::signal::windows::CtrlC,
    /// Set when the install poll already observed Ctrl-C.
    #[cfg(windows)]
    seen: bool,
}

/// Installs process signal handlers when `listen` is set.
///
/// The caller awaits this before the instance lock and before bind. Unix
/// registers SIGINT and SIGTERM when [`tokio::signal::unix::signal`] returns.
/// On Windows, [`tokio::signal::windows::ctrl_c`] installs the console handler
/// when it is called, and the listener does nothing until it is polled. This
/// function polls that listener once before returning, so a Ctrl-C during
/// lock, stores, or bind is observed by the same listener [`wait`] uses.
///
/// # Errors
///
/// Returns [`RuntimeError::ServeFailed`] when a handler cannot be installed.
pub(crate) async fn install(listen: bool) -> Result<Option<Signals>, RuntimeError> {
    if !listen {
        return Ok(None);
    }
    #[cfg(unix)]
    {
        let interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .map_err(RuntimeError::serve_failed_io)?;
        let terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .map_err(RuntimeError::serve_failed_io)?;
        Ok(Some(Signals {
            interrupt,
            terminate,
        }))
    }
    #[cfg(windows)]
    {
        let mut ctrl_c = tokio::signal::windows::ctrl_c().map_err(RuntimeError::serve_failed_io)?;
        let seen = std::future::poll_fn(|cx| match ctrl_c.poll_recv(cx) {
            std::task::Poll::Ready(_) => std::task::Poll::Ready(true),
            std::task::Poll::Pending => std::task::Poll::Ready(false),
        })
        .await;
        Ok(Some(Signals { ctrl_c, seen }))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = listen;
        Ok(None)
    }
}

/// Waits for Ctrl-C, SIGTERM on Unix, or the private shutdown receiver.
pub(crate) async fn wait(signals: Option<Signals>, cancel: Option<oneshot::Receiver<()>>) {
    let Some(signals) = signals else {
        recv_cancel(cancel).await;
        return;
    };
    #[cfg(unix)]
    {
        let mut signals = signals;
        tokio::select! {
            _ = signals.interrupt.recv() => {}
            _ = signals.terminate.recv() => {}
            _ = recv_cancel(cancel) => {}
        }
    }
    #[cfg(windows)]
    {
        if signals.seen {
            return;
        }
        let mut ctrl_c = signals.ctrl_c;
        tokio::select! {
            _ = ctrl_c.recv() => {}
            _ = recv_cancel(cancel) => {}
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = signals;
        recv_cancel(cancel).await;
    }
}

async fn recv_cancel(cancel: Option<oneshot::Receiver<()>>) {
    if let Some(cancel) = cancel {
        let _ = cancel.await;
    } else {
        std::future::pending::<()>().await;
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn windows_ctrl_c_is_registered_and_polled() {
        let signals = install(true).await.expect("install");
        assert!(signals.is_some());
    }
}
