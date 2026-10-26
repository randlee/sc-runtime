//! In-process daemon fixture. Present only with the `test-util` feature.
//!
//! [`DaemonFixture`] calls [`crate::Daemon::builder`] and [`crate::DaemonBuilder::run`].
//! It does not spawn a daemon process and it does not send a process signal.
//! Endpoint text stays with `sc-transport`. This module does not choose an
//! address or a port.
//!
//! Startup is an explicit serving handshake: the daemon thread signals the
//! caller only after `GET /health` succeeds on that same runtime. Drop and
//! [`shutdown`](DaemonFixture::shutdown) join the daemon thread until `run`
//! returns. They do not detach it. [`shutdown`](DaemonFixture::shutdown)
//! returns [`RuntimeError`] when `run` fails, the thread panics, or a
//! successful join misses its deadline.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use rmcp::transport::streamable_http_server::StreamableHttpService;
use sc_transport::{AppName, Client, Endpoint};
use serde::Deserialize;
use tokio::sync::oneshot;
use utoipa_axum::router::OpenApiRouter;

use crate::Ready;
use crate::builder::DaemonBuilder;
use crate::config::DaemonConfig;
use crate::error::{RuntimeError, TransportStep};
use crate::shutdown::RunControl;

/// Upper bound for the caller's wait until the daemon thread reports serving.
const STARTUP_BUDGET: Duration = Duration::from_secs(30);

/// Upper bound for the in-thread `GET /health` probe. Shorter than
/// [`STARTUP_BUDGET`] so a probe that never succeeds is reported before the
/// caller gives up.
const SERVING_PROBE: Duration = Duration::from_secs(15);

/// Upper bound for joining the daemon thread after shutdown is signaled.
const SHUTDOWN_BUDGET: Duration = Duration::from_secs(10);

type Serving = Result<Endpoint, RuntimeError>;

/// A daemon started through the production builder on a private instance root.
///
/// Drop stops the daemon and joins `run` until it returns. It does not panic
/// and it does not detach the daemon thread. [`shutdown`](Self::shutdown) does
/// the same and returns the `run` result.
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
    /// The returned fixture has already served `GET /health` from inside the
    /// daemon thread. The caller waits on that handshake, not on a sleep loop.
    ///
    /// # Errors
    ///
    /// Returns the [`RuntimeError`] from `run` when the daemon fails before it
    /// is accepting connections, or [`RuntimeError::ServeFailed`] when the
    /// serving handshake misses its deadline.
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
    /// Returns the [`RuntimeError`] from `run` when startup fails, including a
    /// missed serving deadline.
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
    /// The wait joins the daemon thread. A successful `run` that misses the
    /// shutdown deadline still finishes that join, then returns an error. The
    /// thread is not detached.
    ///
    /// # Errors
    ///
    /// Returns the error from `run`, a serve failure when the daemon thread
    /// panicked, or a serve failure when a successful `run` misses the shutdown
    /// deadline.
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
        match join_bounded_async(thread).await {
            Ok(result) => result,
            Err(fault) => Err(fault.into_error()),
        }
    }
}

impl Drop for DaemonFixture {
    fn drop(&mut self) {
        self.signal_stop();
        let Some(thread) = self.thread.take() else {
            return;
        };
        // `JoinHandle`'s destructor detaches. Join on this thread instead, and
        // do not panic: the listener is closed only after `run` returns.
        join_quietly(thread);
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
    let (serving_tx, serving_rx) = oneshot::channel();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let thread = std::thread::spawn(move || drive_on_thread(builder, shutdown_rx, serving_tx));
    let endpoint = match tokio::time::timeout(STARTUP_BUDGET, serving_rx).await {
        Ok(Ok(Ok(endpoint))) => endpoint,
        Ok(Ok(Err(err))) => {
            let _ = stop_and_join(shutdown_tx, thread).await;
            return Err(err);
        }
        Ok(Err(_)) => {
            return Err(match stop_and_join(shutdown_tx, thread).await {
                Err(err) => err,
                Ok(()) => RuntimeError::serve_failed("daemon exited before it was ready"),
            });
        }
        Err(_) => {
            let _ = stop_and_join(shutdown_tx, thread).await;
            return Err(RuntimeError::serve_failed(format!(
                "daemon did not become ready within {STARTUP_BUDGET:?}"
            )));
        }
    };
    let client = match Client::new(endpoint.clone()) {
        Ok(client) => client,
        Err(err) => {
            let _ = stop_and_join(shutdown_tx, thread).await;
            return Err(RuntimeError::from_transport(TransportStep::Bind, err));
        }
    };
    Ok(DaemonFixture {
        client,
        endpoint,
        instance_root: root,
        shutdown_tx: Some(shutdown_tx),
        thread: Some(thread),
        _temp: temp,
    })
}

async fn stop_and_join(
    shutdown_tx: oneshot::Sender<()>,
    thread: JoinHandle<Result<(), RuntimeError>>,
) -> Result<(), RuntimeError> {
    drop(shutdown_tx);
    match join_bounded_async(thread).await {
        Ok(result) => result,
        Err(fault) => Err(fault.into_error()),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum JoinFault {
    Panicked,
    TimedOut,
}

impl JoinFault {
    fn into_error(self) -> RuntimeError {
        let cause = match self {
            Self::Panicked => "daemon thread panicked".to_owned(),
            Self::TimedOut => {
                format!("daemon thread did not exit within {SHUTDOWN_BUDGET:?} after shutdown")
            }
        };
        RuntimeError::serve_failed(cause)
    }
}

/// Owns a daemon thread until [`join_quietly`] has run.
///
/// Dropping a [`JoinHandle`] detaches the thread. This guard joins instead.
struct JoinGuard {
    thread: Option<JoinHandle<Result<(), RuntimeError>>>,
}

impl JoinGuard {
    fn new(thread: JoinHandle<Result<(), RuntimeError>>) -> Self {
        Self {
            thread: Some(thread),
        }
    }

    /// Joins the thread. A successful `run` that exceeds `budget` is
    /// [`JoinFault::TimedOut`] after the thread has stopped.
    fn join_within(mut self, budget: Duration) -> Result<Result<(), RuntimeError>, JoinFault> {
        let Some(thread) = self.thread.take() else {
            return Ok(Ok(()));
        };
        let started = Instant::now();
        let outcome = thread.join();
        let late = started.elapsed() >= budget;
        match outcome {
            Err(_) => Err(JoinFault::Panicked),
            Ok(Err(err)) => Ok(Err(err)),
            Ok(Ok(())) if late => Err(JoinFault::TimedOut),
            Ok(Ok(())) => Ok(Ok(())),
        }
    }
}

impl Drop for JoinGuard {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            join_quietly(thread);
        }
    }
}

fn join_quietly(thread: JoinHandle<Result<(), RuntimeError>>) {
    let _ = thread.join();
}

fn join_bounded(
    thread: JoinHandle<Result<(), RuntimeError>>,
    budget: Duration,
) -> Result<Result<(), RuntimeError>, JoinFault> {
    JoinGuard::new(thread).join_within(budget)
}

async fn join_bounded_async(
    thread: JoinHandle<Result<(), RuntimeError>>,
) -> Result<Result<(), RuntimeError>, JoinFault> {
    match tokio::task::spawn_blocking(move || join_bounded(thread, SHUTDOWN_BUDGET)).await {
        Ok(result) => result,
        Err(_) => Err(JoinFault::Panicked),
    }
}

#[derive(Debug, Deserialize)]
struct ReadyProbe {
    ok: bool,
}

enum ProbeFailure {
    /// The serve task is still running.
    StillRunning(RuntimeError),
    /// `serve` was polled to completion while waiting for `/health`.
    ServeFinished(RuntimeError),
}

fn drive_on_thread<S, FStores, Fut, FRoutes>(
    builder: DaemonBuilder<Ready, S, FStores, FRoutes>,
    shutdown_rx: oneshot::Receiver<()>,
    serving_tx: oneshot::Sender<Serving>,
) -> Result<(), RuntimeError>
where
    S: Clone + Send + Sync + 'static,
    FStores: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<S, RuntimeError>> + Send + 'static,
    FRoutes: FnOnce(S) -> OpenApiRouter + Send + 'static,
{
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => {
            let err = RuntimeError::serve_failed(err.to_string());
            let _ = serving_tx.send(Err(err.clone()));
            return Err(err);
        }
    };
    runtime.block_on(run_until_stopped(builder, shutdown_rx, serving_tx)) // allow: NFR-RUN-0002 fixture thread
}

async fn run_until_stopped<S, FStores, Fut, FRoutes>(
    builder: DaemonBuilder<Ready, S, FStores, FRoutes>,
    shutdown_rx: oneshot::Receiver<()>,
    serving_tx: oneshot::Sender<Serving>,
) -> Result<(), RuntimeError>
where
    S: Clone + Send + Sync + 'static,
    FStores: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<S, RuntimeError>> + Send + 'static,
    FRoutes: FnOnce(S) -> OpenApiRouter + Send + 'static,
{
    let (bound_tx, bound_rx) = oneshot::channel();
    let builder = builder.with_control(RunControl::detached(Some(bound_tx), Some(shutdown_rx)));
    let mut serve = tokio::spawn(builder.run());
    let endpoint = match bound_rx.await {
        Ok(endpoint) => endpoint,
        Err(_) => {
            let err = exited_early(serve.await);
            let _ = serving_tx.send(Err(err.clone()));
            return Err(err);
        }
    };
    let client = match Client::new(endpoint.clone()) {
        Ok(client) => client,
        Err(err) => {
            let err = RuntimeError::from_transport(TransportStep::Bind, err);
            let _ = serving_tx.send(Err(err.clone()));
            let _ = serve.await;
            return Err(err);
        }
    };
    match confirm_serving(&client, &mut serve).await {
        Ok(()) => {
            let _ = serving_tx.send(Ok(endpoint));
            task_result(serve.await)
        }
        Err(ProbeFailure::StillRunning(err)) => {
            let _ = serving_tx.send(Err(err.clone()));
            let _ = serve.await;
            Err(err)
        }
        Err(ProbeFailure::ServeFinished(err)) => {
            let _ = serving_tx.send(Err(err.clone()));
            Err(err)
        }
    }
}

async fn confirm_serving(
    client: &Client,
    serve: &mut tokio::task::JoinHandle<Result<(), RuntimeError>>,
) -> Result<(), ProbeFailure> {
    let probe = async {
        loop {
            tokio::select! {
                biased;
                result = &mut *serve => {
                    return Err(ProbeFailure::ServeFinished(exited_early(result)));
                }
                probe = client.get::<ReadyProbe>("/health") => {
                    match probe {
                        Ok(body) if body.ok => return Ok(()),
                        Ok(_) => {
                            return Err(ProbeFailure::StillRunning(RuntimeError::serve_failed(
                                "daemon /health reported not ok",
                            )));
                        }
                        Err(_) => tokio::task::yield_now().await,
                    }
                }
            }
        }
    };
    match tokio::time::timeout(SERVING_PROBE, probe).await {
        Ok(result) => result,
        Err(_) => Err(ProbeFailure::StillRunning(RuntimeError::serve_failed(
            format!("daemon did not answer /health within {SERVING_PROBE:?}"),
        ))),
    }
}

fn task_result(
    result: Result<Result<(), RuntimeError>, tokio::task::JoinError>,
) -> Result<(), RuntimeError> {
    match result {
        Ok(result) => result,
        Err(err) if err.is_panic() => Err(RuntimeError::serve_failed("daemon task panicked")),
        Err(err) => Err(RuntimeError::serve_failed(err.to_string())),
    }
}

fn exited_early(result: Result<Result<(), RuntimeError>, tokio::task::JoinError>) -> RuntimeError {
    match task_result(result) {
        Ok(()) => RuntimeError::serve_failed("daemon exited before it was ready"),
        Err(err) => err,
    }
}

#[cfg(test)]
mod join_tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, mpsc};
    use std::thread::JoinHandle;
    use std::time::Duration;

    use super::{JoinFault, JoinGuard, RuntimeError, join_bounded};

    #[test]
    fn late_success_is_a_timeout_after_the_thread_stops() {
        let (finished, release, thread) = held_thread(Ok(()));
        let joiner = std::thread::spawn(move || join_bounded(thread, Duration::ZERO));
        assert!(
            !joiner_finished_while_held(&joiner),
            "join must not return before the daemon thread stops"
        );
        release.send(()).expect("release held thread");
        let result = joiner.join().expect("join the joiner");
        assert!(finished.load(Ordering::SeqCst));
        assert!(matches!(result, Err(JoinFault::TimedOut)));
    }

    #[test]
    fn late_run_error_is_returned_after_the_thread_stops() {
        let error = RuntimeError::serve_failed("run failed");
        let (finished, release, thread) = held_thread(Err(error.clone()));
        let joiner = std::thread::spawn(move || join_bounded(thread, Duration::ZERO));
        assert!(!joiner_finished_while_held(&joiner));
        release.send(()).expect("release held thread");
        let result = joiner.join().expect("join the joiner");
        assert!(finished.load(Ordering::SeqCst));
        assert_eq!(result, Ok(Err(error)));
    }

    #[test]
    fn dropping_the_join_guard_waits_until_the_thread_stops() {
        let (finished, release, thread) = held_thread(Ok(()));
        let dropper = std::thread::spawn(move || drop(JoinGuard::new(thread)));
        assert!(
            !joiner_finished_while_held(&dropper),
            "drop must not detach the daemon thread"
        );
        release.send(()).expect("release held thread");
        dropper.join().expect("join the dropper");
        assert!(finished.load(Ordering::SeqCst));
    }

    #[test]
    fn join_within_budget_returns_the_thread_result() {
        let thread = std::thread::spawn(|| Ok(()));
        let result = join_bounded(thread, Duration::from_secs(10));
        assert_eq!(result, Ok(Ok(())));
    }

    #[test]
    fn a_panicked_thread_is_a_join_fault() {
        let thread = std::thread::spawn(|| -> Result<(), RuntimeError> {
            panic!("daemon thread failed");
        });
        let result = join_bounded(thread, Duration::from_secs(10));
        assert!(matches!(result, Err(JoinFault::Panicked)));
    }

    fn held_thread(
        result: Result<(), RuntimeError>,
    ) -> (
        Arc<AtomicBool>,
        mpsc::Sender<()>,
        JoinHandle<Result<(), RuntimeError>>,
    ) {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let finished = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&finished);
        let thread = std::thread::spawn(move || {
            entered_tx
                .send(())
                .expect("signal that the thread is waiting");
            release_rx
                .recv()
                .expect("wait until the test releases the thread");
            flag.store(true, Ordering::SeqCst);
            result
        });
        entered_rx
            .recv()
            .expect("daemon thread is waiting for release");
        (finished, release_tx, thread)
    }

    fn joiner_finished_while_held<T>(joiner: &JoinHandle<T>) -> bool {
        std::thread::sleep(Duration::from_millis(50));
        joiner.is_finished()
    }
}
