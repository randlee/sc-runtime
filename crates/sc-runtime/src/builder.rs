//! Consuming builder. `run` is implemented only after `stores` and `routes`.

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sc_command::observability::{NoopCanonicalLog, SharedCanonicalLog};

use rmcp::transport::streamable_http_server::StreamableHttpService;
use sc_transport::{AppName, Endpoint};
use utoipa_axum::router::OpenApiRouter;

use crate::config::DaemonConfig;
use crate::error::{RuntimeError, TransportStep};
use crate::lock;
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
    pub(crate) logger: SharedCanonicalLog,
    _phase: PhantomData<fn() -> (Phase, S)>,
}

impl<P, S, F, R> std::fmt::Debug for DaemonBuilder<P, S, F, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DaemonBuilder")
            .field("service", self.logger.service_name())
            .finish_non_exhaustive()
    }
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
            logger: Arc::new(NoopCanonicalLog::named_or_default("sc-runtime")),
            _phase: PhantomData,
        }
    }
}

impl<Phase, S, StoresFn, RoutesFn> DaemonBuilder<Phase, S, StoresFn, RoutesFn> {
    /// Replaces the canonical logger without changing the builder state.
    ///
    /// Logging is optional and best effort. The application owns the logger's
    /// construction, flushing and shutdown; a legacy builder uses `sc-runtime`.
    #[must_use]
    pub fn logger(mut self, logger: SharedCanonicalLog) -> Self {
        self.logger = logger;
        self
    }

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
            logger: self.logger,
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
            logger: self.logger,
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
        self.run_with_cleanup(|| async {}).await
    }

    /// Runs the daemon, then awaits application cleanup while retaining its instance lock.
    ///
    /// Cleanup runs once after serving stops, including on startup errors. The
    /// unused stores closure is dropped before cleanup on errors preceding
    /// stores, so a receiver may safely await ownership sent by that closure.
    /// Successful stores, the runtime logger, and any acquired instance lock
    /// remain owned until cleanup finishes. This does not change startup order
    /// or introduce a dependency on application
    /// background tasks. The application owns cleanup and logger shutdown.
    ///
    /// Await this future to completion: dropping it cannot await cleanup, and
    /// dropping or panicking inside cleanup releases the instance lock normally.
    ///
    /// # Errors
    ///
    /// Returns the same startup and serving errors as [`Self::run`], after
    /// cleanup. Cleanup returns no runtime error; applications may retain their
    /// own cleanup result separately without replacing the primary error.
    pub async fn run_with_cleanup<FCleanup, Cleanup>(
        self,
        cleanup: FCleanup,
    ) -> Result<(), RuntimeError>
    where
        FCleanup: FnOnce() -> Cleanup,
        Cleanup: Future<Output = ()>,
    {
        let DaemonBuilder {
            app,
            config,
            stores,
            routes,
            mount,
            control,
            logger,
            ..
        } = self;
        let check_mcp = mount.is_some();
        drive(
            &app,
            &config,
            control,
            logger,
            stores,
            routes,
            check_mcp,
            |router, stores| match mount {
                Some(mount) => mount(router, stores),
                None => router,
            },
            cleanup,
        )
        .await
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "assembly retains the existing closures and control plus shared logger"
)]
async fn drive<S, FStores, Fut, FRoutes, Apply, FCleanup, Cleanup>(
    app: &AppName,
    config: &DaemonConfig,
    mut control: RunControl,
    logger: SharedCanonicalLog,
    stores_fn: FStores,
    routes_fn: FRoutes,
    check_mcp: bool,
    apply_mcp: Apply,
    cleanup: FCleanup,
) -> Result<(), RuntimeError>
where
    S: Clone + Send + 'static,
    FStores: FnOnce() -> Fut,
    Fut: Future<Output = Result<S, RuntimeError>>,
    FRoutes: FnOnce(S) -> OpenApiRouter,
    Apply: FnOnce(axum::Router, S) -> axum::Router,
    FCleanup: FnOnce() -> Cleanup,
    Cleanup: Future<Output = ()>,
{
    let mut instance = None;
    let mut retained_stores = None;
    let serve_logger = Arc::clone(&logger);
    // This inner future owns startup closures, including unused senders, and
    // is dropped before cleanup is called even when an early step fails.
    let retained_instance = &mut instance;
    let stores_for_cleanup = &mut retained_stores;
    let result = async move {
        let root = resolve_root(app, config)?;
        let endpoint = resolve_endpoint(app, config, &root)?;
        let signals = shutdown::install(control.listen_for_signals).await?;
        let root_for_lock = root.clone();
        *retained_instance = Some(
            tokio::task::spawn_blocking(move || lock::acquire(&root_for_lock))
                .await
                .map_err(|err| {
                    RuntimeError::instance_failed(format!("instance lock task failed: {err}"))
                })??,
        );
        let stores = stores_fn().await?;
        *stores_for_cleanup = Some(stores.clone());
        let openapi = routes_fn(stores.clone());
        let router = match router::with_runtime_routes(openapi, check_mcp) {
            Ok(router) => apply_mcp(router, stores),
            Err(err) => return Err(err),
        };
        serve(router, &endpoint, &mut control, signals, serve_logger).await
    }
    .await;
    cleanup().await;
    drop(retained_stores);
    drop(instance);
    drop(logger);
    result
}

async fn serve(
    router: axum::Router,
    endpoint: &Endpoint,
    control: &mut RunControl,
    signals: Option<shutdown::Signals>,
    logger: SharedCanonicalLog,
) -> Result<(), RuntimeError> {
    let listener = match sc_transport::bind(endpoint).await {
        Ok(listener) => listener,
        Err(err) => {
            return Err(RuntimeError::from_transport(TransportStep::Bind, err));
        }
    };
    let bound = match listener.local_endpoint() {
        Ok(bound) => bound,
        Err(err) => {
            drop(listener);
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
        logger,
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
        return finish_serve(served, result);
    }

    let _ = drain_tx.send(true);
    let _ = gate_tx.send(());
    let waited =
        tokio::time::timeout(drain.saturating_add(shutdown::DRAIN_BACKSTOP), &mut served).await;
    let result = match waited {
        Ok(result) => result,
        Err(_elapsed) => Ok(()),
    };
    finish_serve(served, result)
}

/// Drops the server and listener before application cleanup and instance release.
fn finish_serve<F>(
    served: std::pin::Pin<Box<F>>,
    result: Result<(), std::io::Error>,
) -> Result<(), RuntimeError> {
    drop(served);
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
