//! SIGINT, SIGTERM, and the fixture's private shutdown trigger.

use tokio::sync::oneshot;

use crate::error::RuntimeError;

/// How `run` waits before it stops accepting connections.
pub(crate) struct RunControl {
    pub(crate) ready: Option<oneshot::Sender<sc_transport::Endpoint>>,
    pub(crate) shutdown: Option<oneshot::Receiver<()>>,
    /// Production listens for process signals. The in-process fixture does not,
    /// because a process signal would stop every fixture in the test binary.
    /// The shutdown sequence after the trigger is the same either way.
    pub(crate) listen_for_signals: bool,
}

impl RunControl {
    pub(crate) fn production() -> Self {
        Self {
            ready: None,
            shutdown: None,
            listen_for_signals: true,
        }
    }
}

/// Installed process-signal listeners. Absent when the fixture owns shutdown.
pub(crate) struct Signals {
    #[cfg(unix)]
    interrupt: tokio::signal::unix::Signal,
    #[cfg(unix)]
    terminate: tokio::signal::unix::Signal,
}

/// Installs process signal handlers when `listen` is set.
///
/// Signal streams are registered before lock and bind so start-up is not a
/// race (FTQ-003). Windows installs Ctrl-C here; the future is created again
/// in [`wait`] once serving starts.
///
/// # Errors
///
/// Returns [`RuntimeError::ServeFailed`] when a handler cannot be installed.
pub(crate) fn install(listen: bool) -> Result<Option<Signals>, RuntimeError> {
    if !listen {
        return Ok(None);
    }
    #[cfg(unix)]
    {
        let interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .map_err(|err| RuntimeError::serve_failed(err.to_string()))?;
        let terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .map_err(|err| RuntimeError::serve_failed(err.to_string()))?;
        Ok(Some(Signals {
            interrupt,
            terminate,
        }))
    }
    #[cfg(windows)]
    {
        // Install the Ctrl-C handler before lock and bind. The future is
        // created again in `wait` once serving starts.
        drop(tokio::signal::ctrl_c());
        Ok(Some(Signals {}))
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
        let _ = signals;
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                let _ = result;
            }
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
