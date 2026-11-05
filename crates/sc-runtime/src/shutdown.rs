//! SIGINT, SIGTERM, Windows Ctrl-C/Ctrl-Break, and the fixture's private shutdown trigger.

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
    pub(crate) on_bound: Option<Box<dyn FnOnce(sc_transport::Endpoint) + Send>>,
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
            on_bound: None,
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
            on_bound: None,
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
    /// Same listeners polled during install. A later `ctrl_c()` call would
    /// miss a Ctrl-C that arrived while the daemon was locking or binding.
    #[cfg(windows)]
    ctrl_c: tokio::signal::windows::CtrlC,
    #[cfg(windows)]
    ctrl_break: tokio::signal::windows::CtrlBreak,
    /// Set when either install poll already observed a console event.
    #[cfg(windows)]
    seen: bool,
}

/// Installs process signal handlers when `listen` is set.
///
/// The caller awaits this before the instance lock and before bind. Unix
/// registers SIGINT and SIGTERM when [`tokio::signal::unix::signal`] returns.
/// On Windows, Ctrl-C and Ctrl-Break install their console handlers
/// when it is called, and the listener does nothing until it is polled. This
/// function polls both listeners once before returning, so an event during
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
        let mut ctrl_break =
            tokio::signal::windows::ctrl_break().map_err(RuntimeError::serve_failed_io)?;
        let seen = std::future::poll_fn(|cx| {
            // Poll both listeners even if the first has already observed an
            // event: both must be registered before lock/store/bind begins.
            let interrupt = ctrl_c.poll_recv(cx).is_ready();
            let breaking = ctrl_break.poll_recv(cx).is_ready();
            std::task::Poll::Ready(interrupt || breaking)
        })
        .await;
        Ok(Some(Signals {
            ctrl_c,
            ctrl_break,
            seen,
        }))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = listen;
        Ok(None)
    }
}

/// Waits for Unix signals, Windows Ctrl-C/Ctrl-Break, or the private trigger.
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
        let mut ctrl_break = signals.ctrl_break;
        tokio::select! {
            _ = ctrl_c.recv() => {}
            _ = ctrl_break.recv() => {}
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
