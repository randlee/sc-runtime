//! Consuming builder. `run` is implemented only after `stores` and `routes`.

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use rmcp::transport::streamable_http_server::StreamableHttpService;
use sc_transport::{AppName, Endpoint};
use utoipa_axum::router::OpenApiRouter;

use crate::config::DaemonConfig;
use crate::error::{RuntimeError, TransportStep};
use crate::lock::{self, InstanceLock};
use crate::router;
use crate::shutdown::{self, RunControl};

/// Typestate before [`DaemonBuilder::stores`]. Not part of the facade.
#[doc(hidden)]
#[non_exhaustive]
pub struct NeedStores {
    _private: (),
}

/// Typestate before [`DaemonBuilder::routes`]. Not part of the facade.
#[doc(hidden)]
#[non_exhaustive]
pub struct NeedRoutes {
    _private: (),
}

/// Typestate after `routes`, when `mcp` and `run` are available. Not part of the facade.
#[doc(hidden)]
#[non_exhaustive]
pub struct Ready {
    _private: (),
}

/// Entry type. The only public function is [`Daemon::builder`].
///
/// # Examples
///
/// ```
/// use sc_runtime::{Daemon, DaemonConfig};
/// use sc_transport::AppName;
///
/// let app = AppName::new("my-app")?;
/// let _builder = Daemon::builder(&app, &DaemonConfig::default());
/// # Ok::<(), sc_transport::TransportError>(())
/// ```
#[derive(Debug)]
pub struct Daemon {
    _private: (),
}

/// Consuming builder for one daemon.
///
/// Call [`stores`](DaemonBuilder::stores), then [`routes`](DaemonBuilder::routes),
/// then optionally [`mcp`](DaemonBuilder::mcp), then [`run`](DaemonBuilder::run).
/// `run` is not available before `stores` and `routes`.
pub struct DaemonBuilder<Phase, S = (), StoresFn = (), RoutesFn = ()> {
    app: AppName,
    config: DaemonConfig,
    stores: StoresFn,
    routes: RoutesFn,
    /// Erased inside `mcp` so the public builder does not grow a public trait.
    mount: Option<Box<dyn FnOnce(axum::Router, S) -> axum::Router + Send>>,
    control: RunControl,
    _phase: PhantomData<fn() -> (Phase, S)>,
}

impl Daemon {
    /// Starts assembly with an application name and configuration the caller loaded.
    ///
    /// `app` is [`sc_transport::AppName`]. This crate does not construct or
    /// re-export that type. `config` is borrowed and cloned.
    #[must_use]
    pub fn builder(app: &AppName, config: &DaemonConfig) -> DaemonBuilder<NeedStores> {
        DaemonBuilder {
            app: app.clone(),
            config: config.clone(),
            stores: (),
            routes: (),
            mount: None,
            control: RunControl::production(),
            _phase: PhantomData,
        }
    }
}

impl<Phase, S, StoresFn, RoutesFn> DaemonBuilder<Phase, S, StoresFn, RoutesFn> {
    #[cfg(feature = "test-util")]
    pub(crate) fn with_control(mut self, control: RunControl) -> Self {
        self.control = control;
        self
    }
}

impl DaemonBuilder<NeedStores> {
    /// Stores the closure that opens the project's stores.
    ///
    /// The closure runs once, after the instance lock is held and before
    /// routes, MCP, or bind. Its error must be [`RuntimeError::StoresFailed`].
    #[must_use]
    pub fn stores<S, F, Fut>(self, stores: F) -> DaemonBuilder<NeedRoutes, S, F>
    where
        S: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<S, RuntimeError>> + Send + 'static,
    {
        DaemonBuilder {
            app: self.app,
            config: self.config,
            stores,
            routes: (),
            mount: None,
            control: self.control,
            _phase: PhantomData,
        }
    }
}

impl<S, FStores, Fut> DaemonBuilder<NeedRoutes, S, FStores>
where
    S: Clone + Send + Sync + 'static,
    FStores: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<S, RuntimeError>> + Send + 'static,
{
    /// Stores the closure that returns the project's [`OpenApiRouter`].
    ///
    /// The router is mounted as given. Apply Axum state inside the closure
    /// with `OpenApiRouter::with_state` so the value returned here is
    /// `OpenApiRouter<()>`.
    #[must_use]
    pub fn routes<FRoutes>(self, routes: FRoutes) -> DaemonBuilder<Ready, S, FStores, FRoutes>
    where
        FRoutes: FnOnce(S) -> OpenApiRouter + Send + 'static,
    {
        DaemonBuilder {
            app: self.app,
            config: self.config,
            stores: self.stores,
            routes,
            mount: None,
            control: self.control,
            _phase: PhantomData,
        }
    }
}

impl<S, FStores, Fut, FRoutes> DaemonBuilder<Ready, S, FStores, FRoutes>
where
    S: Clone + Send + Sync + 'static,
    FStores: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<S, RuntimeError>> + Send + 'static,
    FRoutes: FnOnce(S) -> OpenApiRouter + Send + 'static,
{
    /// Stores the closure that returns an rmcp [`StreamableHttpService`].
    ///
    /// The service is mounted at `/mcp` as given. This crate does not wrap it
    /// and does not add session state.
    #[must_use]
    pub fn mcp<Svc, Mgr, FMcp>(mut self, mcp: FMcp) -> Self
    where
        Svc: rmcp::ServerHandler + Send + 'static,
        Mgr: rmcp::transport::streamable_http_server::SessionManager,
        FMcp: FnOnce(S) -> StreamableHttpService<Svc, Mgr> + Send + 'static,
    {
        self.mount = Some(Box::new(move |router, stores| {
            router::with_mcp(router, mcp(stores))
        }));
        self
    }

    /// Locks, opens stores, mounts routes and optional MCP, binds one listener, and serves.
    ///
    /// `run` is available only after [`stores`](DaemonBuilder::stores) and
    /// [`routes`](DaemonBuilder::routes). [`mcp`](DaemonBuilder::mcp) is optional
    /// on that ready state.
    ///
    /// Endpoint text is resolved before the lock so an invalid endpoint does
    /// not open stores. Signal listeners are registered, and on Windows Ctrl-C
    /// is polled, before the lock and before bind. The order after a valid
    /// endpoint is signals, lock, stores, routes, optional MCP, then bind.
    /// Shutdown stops accepting, waits up to the drain bound for in-flight
    /// requests, then releases the listener and the instance lock.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] for lock contention, a stores failure, endpoint
    /// resolution, a reserved-route conflict, bind, or serve. A failed step
    /// does not run later steps. Shutdown on SIGINT or SIGTERM returns `Ok(())`
    /// after the listener is released. This function does not call
    /// `process::exit`.
    ///
    /// ```compile_fail
    /// use sc_runtime::{Daemon, DaemonConfig};
    /// use sc_transport::AppName;
    ///
    /// fn demo() -> Result<(), sc_transport::TransportError> {
    ///     let app = AppName::new("my-app")?;
    ///     let _ = Daemon::builder(&app, &DaemonConfig::default()).run();
    ///     Ok(())
    /// }
    /// ```
    pub async fn run(self) -> Result<(), RuntimeError> {
        let DaemonBuilder {
            app,
            config,
            stores,
            routes,
            mount,
            control,
            ..
        } = self;
        let check_mcp = mount.is_some();
        drive(
            &app,
            &config,
            control,
            stores,
            routes,
            check_mcp,
            |router, stores| match mount {
                Some(mount) => mount(router, stores),
                None => router,
            },
        )
        .await
    }
}

async fn drive<S, FStores, Fut, FRoutes, Apply>(
    app: &AppName,
    config: &DaemonConfig,
    mut control: RunControl,
    stores_fn: FStores,
    routes_fn: FRoutes,
    check_mcp: bool,
    apply_mcp: Apply,
) -> Result<(), RuntimeError>
where
    S: Clone + Send + 'static,
    FStores: FnOnce() -> Fut,
    Fut: Future<Output = Result<S, RuntimeError>>,
    FRoutes: FnOnce(S) -> OpenApiRouter,
    Apply: FnOnce(axum::Router, S) -> axum::Router,
{
    let root = resolve_root(app, config)?;
    let endpoint = resolve_endpoint(app, config, &root)?;
    let signals = shutdown::install(control.listen_for_signals).await?;
    let root_for_lock = root.clone();
    let lock = tokio::task::spawn_blocking(move || lock::acquire(&root_for_lock))
        .await
        .map_err(|err| {
            RuntimeError::instance_failed(format!("instance lock task failed: {err}"))
        })??;
    let stores = match stores_fn().await {
        Ok(stores) => stores,
        Err(err) => {
            drop(lock);
            return Err(err);
        }
    };
    let openapi = routes_fn(stores.clone());
    let router = match router::with_runtime_routes(openapi, check_mcp) {
        Ok(router) => apply_mcp(router, stores),
        Err(err) => {
            drop(lock);
            return Err(err);
        }
    };
    serve(router, &endpoint, &mut control, lock, signals).await
}

async fn serve(
    router: axum::Router,
    endpoint: &Endpoint,
    control: &mut RunControl,
    lock: InstanceLock,
    signals: Option<shutdown::Signals>,
) -> Result<(), RuntimeError> {
    let listener = match sc_transport::bind(endpoint).await {
        Ok(listener) => listener,
        Err(err) => {
            drop(lock);
            return Err(RuntimeError::from_transport(TransportStep::Bind, err));
        }
    };
    let bound = match listener.local_endpoint() {
        Ok(bound) => bound,
        Err(err) => {
            drop(listener);
            drop(lock);
            return Err(RuntimeError::from_transport(TransportStep::Bind, err));
        }
    };
    if let Some(ready) = control.ready.take() {
        let _ = ready.send(bound);
    }
    let drain = control.drain;
    let cancel = control.shutdown.take();
    let (drain_tx, drain_rx) = tokio::sync::watch::channel(false);
    let metrics = router::RequestMetrics::new();
    let router = router::with_request_tracking(
        router::with_body_limit(router::with_request_timeout(
            router,
            router::REQUEST_TIMEOUT,
            drain_rx,
            drain,
        )),
        metrics,
    );
    let (gate_tx, gate_rx) = tokio::sync::oneshot::channel::<()>();
    let served =
        axum::serve(listener, router.into_make_service()).with_graceful_shutdown(async move {
            let _ = gate_rx.await;
        });
    let mut served = Box::pin(served.into_future());

    let early = tokio::select! {
        result = &mut served => Some(result),
        () = shutdown::wait(signals, cancel) => None,
    };
    if let Some(result) = early {
        return finish_serve(served, lock, result);
    }

    let _ = drain_tx.send(true);
    let _ = gate_tx.send(());
    let waited =
        tokio::time::timeout(drain.saturating_add(shutdown::DRAIN_BACKSTOP), &mut served).await;
    let result = match waited {
        Ok(result) => result,
        Err(_elapsed) => Ok(()),
    };
    finish_serve(served, lock, result)
}

/// Drops the server (and its listener) before the instance lock.
fn finish_serve<F>(
    served: std::pin::Pin<Box<F>>,
    lock: InstanceLock,
    result: Result<(), std::io::Error>,
) -> Result<(), RuntimeError> {
    drop(served);
    drop(lock);
    result.map_err(RuntimeError::serve_failed_io)
}

fn resolve_root(app: &AppName, config: &DaemonConfig) -> Result<PathBuf, RuntimeError> {
    if let Some(root) = &config.transport.instance_root {
        return Ok(root.clone());
    }
    sc_transport::instance_root(app)
        .map_err(|err| RuntimeError::from_transport(TransportStep::Instance, err))
}

fn resolve_endpoint(
    app: &AppName,
    config: &DaemonConfig,
    root: &Path,
) -> Result<Endpoint, RuntimeError> {
    let mut transport = config.transport.clone();
    transport.instance_root = Some(root.to_path_buf());
    let flag = config.flag_os();
    sc_transport::resolve_endpoint_from_env(app, flag.as_deref(), &transport)
        .map_err(|err| RuntimeError::from_transport(TransportStep::Endpoint, err))
}
