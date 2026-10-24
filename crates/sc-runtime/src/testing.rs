//! In-process daemon fixture. Present only with the `test-util` feature.
//!
//! [`DaemonFixture`] calls [`crate::Daemon::builder`] and [`crate::DaemonBuilder::run`].
//! It does not spawn a daemon process and it does not send a process signal.
//! Endpoint text stays with `sc-transport`. This module does not choose an
//! address or a port.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::Duration;

use rmcp::transport::streamable_http_server::StreamableHttpService;
use sc_transport::{AppName, Client, Endpoint};
use serde::Deserialize;
use tokio::sync::oneshot;
use utoipa_axum::router::OpenApiRouter;

use crate::Ready;
use crate::builder::DaemonBuilder;
use crate::config::DaemonConfig;
use crate::error::RuntimeError;
use crate::shutdown::RunControl;

/// A daemon started through the production builder on a private instance root.
///
/// Drop stops the daemon and waits until `run` returns. [`shutdown`](Self::shutdown)
/// does the same and returns the `run` result.
pub struct DaemonFixture {
    client: Client,
    endpoint: Endpoint,
    instance_root: PathBuf,
    shutdown_tx: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<Result<(), RuntimeError>>>,
    _temp: tempfile::TempDir,
}

impl DaemonFixture {
    /// Starts a daemon. `config.transport.instance_root` is replaced with a
    /// new temporary directory. Endpoint fields on `config` are left as the
    /// caller set them and are resolved by `sc-transport`.
    ///
    /// # Errors
    ///
    /// Returns the [`RuntimeError`] from `run` when the daemon fails before it
    /// is accepting connections.
    pub async fn start<S, FStores, Fut, FRoutes>(
        app: &AppName,
        config: DaemonConfig,
        stores: FStores,
        routes: FRoutes,
    ) -> Result<Self, RuntimeError>
    where
        S: Clone + Send + Sync + 'static,
        FStores: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<S, RuntimeError>> + Send + 'static,
        FRoutes: FnOnce(S) -> OpenApiRouter + Send + 'static,
    {
        let (config, temp) = prepare_config(config)?;
        let builder = crate::Daemon::builder(app, &config)
            .stores(stores)
            .routes(routes);
        spawn(builder, temp).await
    }

    /// Starts a daemon that also mounts an rmcp service.
    ///
    /// # Errors
    ///
    /// Returns the [`RuntimeError`] from `run` when startup fails.
    pub async fn start_with_mcp<S, FStores, Fut, FRoutes, Svc, Mgr, FMcp>(
        app: &AppName,
        config: DaemonConfig,
        stores: FStores,
        routes: FRoutes,
        mcp: FMcp,
    ) -> Result<Self, RuntimeError>
    where
        S: Clone + Send + Sync + 'static,
        FStores: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<S, RuntimeError>> + Send + 'static,
        FRoutes: FnOnce(S) -> OpenApiRouter + Send + 'static,
        Svc: rmcp::ServerHandler + Send + 'static,
        Mgr: rmcp::transport::streamable_http_server::SessionManager,
        FMcp: FnOnce(S) -> StreamableHttpService<Svc, Mgr> + Send + 'static,
    {
        let (config, temp) = prepare_config(config)?;
        let builder = crate::Daemon::builder(app, &config)
            .stores(stores)
            .routes(routes)
            .mcp(mcp);
        spawn(builder, temp).await
    }

    /// Connected client for this daemon's bound endpoint.
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Bound endpoint. Its [`Display`](Endpoint) text is the `--endpoint` form.
    #[must_use]
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Endpoint text from [`Endpoint`]'s `Display` implementation.
    ///
    /// This is the value accepted by `--endpoint` and the endpoint environment variable.
    #[must_use]
    pub fn endpoint_str(&self) -> String {
        self.endpoint.to_string()
    }

    /// Instance root that holds `daemon.lock`.
    #[must_use]
    pub fn instance_root(&self) -> &Path {
        &self.instance_root
    }

    /// Stops the daemon and waits until the listener has closed.
    ///
    /// # Errors
    ///
    /// Returns the error from `run`, or an instance error when the daemon
    /// thread panicked.
    pub async fn shutdown(mut self) -> Result<(), RuntimeError> {
        self.signal_stop();
        self.join().await
    }

    fn signal_stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    async fn join(&mut self) -> Result<(), RuntimeError> {
        let Some(thread) = self.thread.take() else {
            return Ok(());
        };
        let joined = tokio::task::spawn_blocking(move || thread.join()).await;
        match joined {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(RuntimeError::serve_failed("daemon thread panicked")),
            Err(err) => Err(RuntimeError::serve_failed(err.to_string())),
        }
    }
}

impl Drop for DaemonFixture {
    fn drop(&mut self) {
        self.signal_stop();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn prepare_config(
    mut config: DaemonConfig,
) -> Result<(DaemonConfig, tempfile::TempDir), RuntimeError> {
    let dir = tempfile::tempdir().map_err(|err| RuntimeError::instance_failed(err.to_string()))?;
    config.transport.instance_root = Some(dir.path().to_path_buf());
    Ok((config, dir))
}

async fn spawn<S, FStores, Fut, FRoutes>(
    builder: DaemonBuilder<Ready, S, FStores, FRoutes>,
    temp: tempfile::TempDir,
) -> Result<DaemonFixture, RuntimeError>
where
    S: Clone + Send + Sync + 'static,
    FStores: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<S, RuntimeError>> + Send + 'static,
    FRoutes: FnOnce(S) -> OpenApiRouter + Send + 'static,
{
    let root = temp.path().to_path_buf();
    let (ready_tx, ready_rx) = oneshot::channel();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let builder = builder.with_control(RunControl {
        ready: Some(ready_tx),
        shutdown: Some(shutdown_rx),
        listen_for_signals: false,
    });
    let thread = std::thread::spawn(move || drive_on_thread(builder));
    let endpoint = match ready_rx.await {
        Ok(endpoint) => endpoint,
        Err(_) => return Err(join_startup(thread)),
    };
    let client = match Client::new(endpoint.clone()) {
        Ok(client) => client,
        Err(err) => {
            drop(shutdown_tx);
            let _ = join_startup(thread);
            return Err(RuntimeError::bind_failed(err.to_string()));
        }
    };
    if let Err(err) = wait_until_serving(&client).await {
        drop(shutdown_tx);
        let _ = join_startup(thread);
        return Err(err);
    }
    Ok(DaemonFixture {
        client,
        endpoint,
        instance_root: root,
        shutdown_tx: Some(shutdown_tx),
        thread: Some(thread),
        _temp: temp,
    })
}

fn join_startup(thread: JoinHandle<Result<(), RuntimeError>>) -> RuntimeError {
    match thread.join() {
        Ok(Err(err)) => err,
        Ok(Ok(())) => RuntimeError::serve_failed("daemon exited before it was ready"),
        Err(_) => RuntimeError::serve_failed("daemon thread panicked"),
    }
}

#[derive(Debug, Deserialize)]
struct ReadyProbe {
    ok: bool,
}

async fn wait_until_serving(client: &Client) -> Result<(), RuntimeError> {
    for _ in 0..50 {
        if let Ok(body) = client.get::<ReadyProbe>("/health").await {
            if body.ok {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Err(RuntimeError::serve_failed("daemon did not answer /health"))
}

fn drive_on_thread<S, FStores, Fut, FRoutes>(
    builder: DaemonBuilder<Ready, S, FStores, FRoutes>,
) -> Result<(), RuntimeError>
where
    S: Clone + Send + Sync + 'static,
    FStores: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<S, RuntimeError>> + Send + 'static,
    FRoutes: FnOnce(S) -> OpenApiRouter + Send + 'static,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| RuntimeError::serve_failed(err.to_string()))?;
    runtime.block_on(builder.run())
}
