//! One Axum router: project routes, OpenAPI, liveness, and optional `/mcp`.
//!
//! `GET /health` is liveness. The handler reports that this process is serving
//! HTTP. Store and dependency readiness stay on a project route such as
//! `GET /ready`. That path is left unreserved, and `/health` stays liveness.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tracing::Instrument;

use axum::Extension;
use axum::Json;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use sc_command::{Envelope, ErrorKind, OpError};
use serde::Serialize;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::error::RuntimeError;

const HEALTH: &str = "/health";
const OPENAPI: &str = "/openapi.json";
const METRICS: &str = "/metrics";
const MCP: &str = "/mcp";
const REQUEST_ID: &str = "x-request-id";

/// Per-router request metrics and request-id sequence (not process-global).
#[derive(Clone, Default)]
pub(crate) struct RequestMetrics {
    inner: Arc<RequestMetricsInner>,
}

#[derive(Default)]
struct RequestMetricsInner {
    requests: AtomicU64,
    server_errors: AtomicU64,
    next_request_id: AtomicU64,
}

impl RequestMetrics {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(RequestMetricsInner {
                next_request_id: AtomicU64::new(1),
                ..RequestMetricsInner::default()
            }),
        }
    }
}

/// Maximum time one accepted request may spend in a handler.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Largest request body the daemon's extractors accept.
///
/// This is axum's extractor default of 2 MiB, stated on the runtime so the
/// limit is an application choice. [`with_body_limit`] applies it before the
/// server starts. A project route that streams a larger body layers
/// [`axum::extract::DefaultBodyLimit::disable`] or
/// [`axum::extract::DefaultBodyLimit::max`] on that route.
pub(crate) const DEFAULT_REQUEST_BODY_LIMIT: usize = 2_097_152;

const REQUEST_TIMEOUT_BODY: &str = "the handler exceeded its time limit";
const DRAIN_TIMEOUT_BODY: &str = "shutdown finished waiting for this request";

/// Stable code when a handler exceeds [`REQUEST_TIMEOUT`].
const CODE_REQUEST_TIMEOUT: &str = "RUNTIME.REQUEST_TIMEOUT";

/// Stable code when shutdown stops waiting for an in-flight handler.
const CODE_DRAIN_TIMEOUT: &str = "RUNTIME.DRAIN_TIMEOUT";

/// Wire `suggested_action` for [`CODE_REQUEST_TIMEOUT`].
const REQUEST_TIMEOUT_RECOVERY: &str =
    "retry the request; if it persists, shorten the handler or raise the request time limit";

/// Wire `suggested_action` for [`CODE_DRAIN_TIMEOUT`].
const DRAIN_TIMEOUT_RECOVERY: &str = "retry after the daemon is serving again";

/// Body of the liveness probe `GET /health`.
///
/// `status` is `"ok"` while this process is serving. It does not describe
/// store or dependency readiness. The HTTP response is [`Envelope`]'s server
/// conversion.
#[derive(Debug, Serialize)]
struct HealthBody {
    status: &'static str,
}

/// Liveness handler for `GET /health`.
///
/// Succeeds whenever the process can run the handler. A project that needs
/// dependency-aware readiness registers its own route, such as `GET /ready`,
/// on the project router. This handler does not call stores.
async fn health() -> Envelope<HealthBody> {
    Envelope::success(HealthBody { status: "ok" })
}

#[derive(Debug, Serialize)]
struct MetricsBody {
    requests: u64,
    server_errors: u64,
}

async fn metrics(Extension(metrics): Extension<RequestMetrics>) -> Json<MetricsBody> {
    Json(MetricsBody {
        requests: metrics.inner.requests.load(Ordering::Relaxed),
        server_errors: metrics.inner.server_errors.load(Ordering::Relaxed),
    })
}

fn accepted_request_id(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn request_id_from(request: &Request, metrics: &RequestMetrics) -> String {
    request
        .headers()
        .get(REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .filter(|value| accepted_request_id(value))
        .map(str::to_owned)
        .unwrap_or_else(|| {
            let tick = metrics
                .inner
                .next_request_id
                .fetch_add(1, Ordering::Relaxed);
            format!("req-{tick}")
        })
}

async fn stamp_request_id(
    Extension(metrics): Extension<RequestMetrics>,
    mut request: Request,
    next: Next,
) -> Response {
    let id = request_id_from(&request, &metrics);
    if let Ok(value) = axum::http::HeaderValue::from_str(&id) {
        request.headers_mut().insert(REQUEST_ID, value);
    }
    let span = tracing::info_span!("http_request", request_id = %id);
    let mut response = next.run(request).instrument(span).await;
    if let Ok(value) = axum::http::HeaderValue::from_str(&id) {
        response.headers_mut().insert(REQUEST_ID, value);
    }
    response
}

async fn count_request(
    Extension(metrics): Extension<RequestMetrics>,
    request: Request,
    next: Next,
) -> Response {
    let response = next.run(request).await;
    metrics.inner.requests.fetch_add(1, Ordering::Relaxed);
    if response.status().is_server_error() {
        metrics.inner.server_errors.fetch_add(1, Ordering::Relaxed);
    }
    response
}

/// Request-id stamping and metrics counting scoped to one router instance.
pub(crate) fn with_request_tracking(router: axum::Router, metrics: RequestMetrics) -> axum::Router {
    router
        .layer(axum::middleware::from_fn(count_request))
        .layer(axum::middleware::from_fn(stamp_request_id))
        .layer(Extension(metrics))
}

/// Merges the project router with the document route and the liveness route.
///
/// The OpenAPI document is the one `split_for_parts` returns. It is served at
/// `GET /openapi.json`. `GET /health` returns status 200 and a success
/// envelope while the process is serving. That route is liveness only.
/// Dependency readiness is a project route (for example `GET /ready`); this
/// function does not mount or reserve that path.
///
/// # Errors
///
/// Returns [`RuntimeError::RouteConflict`] when the project router already
/// defines a reserved path, so axum does not panic on the later `route`.
pub(crate) fn with_runtime_routes(
    openapi: OpenApiRouter,
    check_mcp: bool,
) -> Result<axum::Router, RuntimeError> {
    let (router, api) = openapi.split_for_parts();
    if let Some(path) = reserved_conflict(&router, &api, check_mcp) {
        return Err(RuntimeError::route_conflict(path));
    }
    let api = Arc::new(api);
    Ok(router
        .route(
            OPENAPI,
            get(move || {
                let api = Arc::clone(&api);
                async move { Json(api.as_ref().clone()) }
            }),
        )
        .route(HEALTH, get(health))
        .route(METRICS, get(metrics)))
}

/// Applies a handler timeout and a shutdown drain bound to the assembled router.
///
/// `shutdown` becomes true when the daemon starts graceful shutdown. A handler
/// still running then has `drain` left. A handler that exceeds `limit` from
/// the start of the request is the request-timeout case. Both failures use
/// [`ErrorKind::Unavailable`]. [`ErrorKind::http_status_code`] selects the
/// HTTP status, which is 503 for that kind. The codes are
/// `RUNTIME.REQUEST_TIMEOUT` and `RUNTIME.DRAIN_TIMEOUT`. Each envelope
/// sets `suggested_action` to the recovery for that code.
///
/// When `limit` elapses, the middleware returns that 503 and drops the handler
/// future. Drop cancels the future at its next `.await`. Cancellation is
/// cooperative: work that is not at an `.await`, or that the handler already
/// spawned onto another task, can keep running after the client has the
/// timeout response. This middleware does not detach the handler. A store
/// transaction that is still open at that `.await` rolls back when its future
/// is dropped.
pub(crate) fn with_request_timeout(
    router: axum::Router,
    limit: Duration,
    shutdown: tokio::sync::watch::Receiver<bool>,
    drain: Duration,
) -> axum::Router {
    router.layer(axum::middleware::from_fn(move |request, next: Next| {
        let mut shutdown = shutdown.clone();
        async move { enforce_deadlines(request, next, limit, &mut shutdown, drain).await }
    }))
}

async fn enforce_deadlines(
    request: Request,
    next: Next,
    limit: Duration,
    shutdown: &mut tokio::sync::watch::Receiver<bool>,
    drain: Duration,
) -> Response {
    let handler = next.run(request);
    tokio::pin!(handler);
    let deadline = tokio::time::Instant::now() + limit;
    match tokio::time::timeout_at(
        deadline,
        race_handler_or_shutdown(handler.as_mut(), shutdown),
    )
    .await
    {
        Ok(Some(response)) => response,
        // Dropping `handler` with this function cancels it. See [`with_request_timeout`].
        Err(_elapsed) => request_timeout_response(),
        Ok(None) => drain_handler(handler, deadline, drain).await,
    }
}

/// Sets the application request-body limit on `router`.
///
/// Extractors such as `Json`, `String`, and `Bytes` stop at
/// [`DEFAULT_REQUEST_BODY_LIMIT`]. A route that must stream past that limit
/// layers [`axum::extract::DefaultBodyLimit::disable`] or a higher
/// [`axum::extract::DefaultBodyLimit::max`] on itself.
pub(crate) fn with_body_limit(router: axum::Router) -> axum::Router {
    apply_body_limit(router, DEFAULT_REQUEST_BODY_LIMIT)
}

fn apply_body_limit(router: axum::Router, limit: usize) -> axum::Router {
    router.layer(axum::extract::DefaultBodyLimit::max(limit))
}

/// Returns the handler response, or `None` when shutdown has started.
///
/// The handler stays in the caller's [`Pin`] when shutdown wins, so the drain
/// bound can keep polling it. `timeout_at` on this future is the request limit.
async fn race_handler_or_shutdown(
    handler: Pin<&mut impl Future<Output = Response>>,
    shutdown: &mut tokio::sync::watch::Receiver<bool>,
) -> Option<Response> {
    tokio::select! {
        biased;
        response = handler => Some(response),
        () = shutdown_signal(shutdown) => None,
    }
}

async fn shutdown_signal(shutdown: &mut tokio::sync::watch::Receiver<bool>) {
    if *shutdown.borrow() {
        return;
    }
    loop {
        if shutdown.changed().await.is_err() || *shutdown.borrow() {
            return;
        }
    }
}

async fn drain_handler(
    handler: Pin<&mut impl Future<Output = Response>>,
    request_deadline: tokio::time::Instant,
    drain: Duration,
) -> Response {
    let drain_deadline = tokio::time::Instant::now() + drain;
    let request_first = request_deadline <= drain_deadline;
    let bound = if request_first {
        request_deadline
    } else {
        drain_deadline
    };
    match tokio::time::timeout_at(bound, handler).await {
        Ok(response) => response,
        Err(_elapsed) if request_first => request_timeout_response(),
        Err(_elapsed) => drain_timeout_response(),
    }
}

fn request_timeout_response() -> Response {
    deadline_response(
        CODE_REQUEST_TIMEOUT,
        REQUEST_TIMEOUT_BODY,
        REQUEST_TIMEOUT_RECOVERY,
    )
}

fn drain_timeout_response() -> Response {
    deadline_response(
        CODE_DRAIN_TIMEOUT,
        DRAIN_TIMEOUT_BODY,
        DRAIN_TIMEOUT_RECOVERY,
    )
}

/// Failure envelope whose status comes only from [`ErrorKind::http_status_code`].
///
/// `recovery` is the wire `suggested_action`.
fn deadline_response(
    code: &'static str,
    message: &'static str,
    recovery: &'static str,
) -> Response {
    Envelope::<()>::failure(
        OpError::from_code(ErrorKind::Unavailable, code, message).with_suggestion(recovery),
    )
    .into_response()
}

/// Mounts the rmcp service at `/mcp` as it was given.
pub(crate) fn with_mcp<S, M>(
    router: axum::Router,
    service: StreamableHttpService<S, M>,
) -> axum::Router
where
    S: rmcp::ServerHandler + Send + 'static,
    M: rmcp::transport::streamable_http_server::SessionManager,
{
    router.nest_service(MCP, service)
}

fn reserved_conflict(
    router: &axum::Router,
    spec: &OpenApi,
    check_mcp: bool,
) -> Option<&'static str> {
    reserved_paths(check_mcp)
        .find(|path| spec_has_path(spec, path) || router_debug_has_path(router, path))
}

fn reserved_paths(check_mcp: bool) -> impl Iterator<Item = &'static str> {
    [HEALTH, OPENAPI, METRICS]
        .into_iter()
        .chain(check_mcp.then_some(MCP))
}

fn spec_has_path(spec: &OpenApi, reserved: &str) -> bool {
    spec.paths
        .paths
        .keys()
        .any(|path| path_conflicts(path, reserved))
}

fn path_conflicts(path: &str, reserved: &str) -> bool {
    let path = path.trim_end_matches('/');
    let reserved = reserved.trim_end_matches('/');
    path == reserved
        || path
            .strip_prefix(reserved)
            .is_some_and(|tail| tail.starts_with('/'))
}

fn router_debug_has_path(router: &axum::Router, reserved: &str) -> bool {
    let debug = format!("{router:?}");
    debug.contains(&format!("\"{reserved}\"")) || debug.contains(&format!("\"{reserved}/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use std::net::{Ipv4Addr, SocketAddr};

    async fn serve_test_router(router: axum::Router) -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let handle = tokio::spawn(async move {
            let _ = ready_tx.send(());
            let _ = axum::serve(listener, router).await;
        });
        tokio::time::timeout(Duration::from_secs(2), ready_rx)
            .await
            .expect("server ready timeout")
            .expect("server ready signal");
        (addr, handle)
    }

    #[test]
    fn debug_detects_route_overlap() {
        let router = axum::Router::new().route(HEALTH, get(|| async { "nope" }));
        assert!(router_debug_has_path(&router, HEALTH));
        assert!(!router_debug_has_path(&router, OPENAPI));
    }

    #[tokio::test]
    async fn slow_handler_returns_request_timeout() {
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let router = with_request_timeout(
            axum::Router::new()
                .route("/fast", get(|| async { "ok" }))
                .route("/slow", get(std::future::pending::<&'static str>)),
            Duration::from_millis(200),
            shutdown_rx,
            Duration::from_secs(30),
        );
        let (addr, _handle) = serve_test_router(router).await;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("client");
        let fast = client
            .get(format!("http://{addr}/fast"))
            .send()
            .await
            .expect("fast");
        assert_eq!(fast.status(), reqwest::StatusCode::OK);
        let slow = client
            .get(format!("http://{addr}/slow"))
            .send()
            .await
            .expect("slow");
        let status = slow.status();
        let body: serde_json::Value = slow.json().await.expect("envelope");
        assert_unavailable_envelope(
            &status,
            &body,
            CODE_REQUEST_TIMEOUT,
            REQUEST_TIMEOUT_BODY,
            REQUEST_TIMEOUT_RECOVERY,
        );
        drop(shutdown_tx);
    }

    #[tokio::test]
    async fn shutdown_during_handler_returns_drain_envelope() {
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(true);
        let router = with_request_timeout(
            axum::Router::new()
                .route("/fast", get(|| async { "ok" }))
                .route("/slow", get(std::future::pending::<&'static str>)),
            Duration::from_secs(30),
            shutdown_rx,
            Duration::from_millis(100),
        );
        let (addr, _handle) = serve_test_router(router).await;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("client");
        let slow = client
            .get(format!("http://{addr}/slow"))
            .send()
            .await
            .expect("slow");
        let status = slow.status();
        let body: serde_json::Value = slow.json().await.expect("envelope");
        assert_unavailable_envelope(
            &status,
            &body,
            CODE_DRAIN_TIMEOUT,
            DRAIN_TIMEOUT_BODY,
            DRAIN_TIMEOUT_RECOVERY,
        );
    }

    fn assert_unavailable_envelope(
        status: &reqwest::StatusCode,
        body: &serde_json::Value,
        code: &str,
        message: &str,
        recovery: &str,
    ) {
        assert_eq!(status.as_u16(), ErrorKind::Unavailable.http_status_code());
        assert_eq!(body["ok"], false);
        assert_eq!(body["error"]["kind"], "unavailable");
        assert_eq!(body["error"]["code"], code);
        assert_eq!(body["error"]["message"], message);
        assert_eq!(body["error"]["suggested_action"], recovery);
    }

    #[tokio::test]
    async fn body_over_the_route_limit_is_rejected() {
        let router = apply_body_limit(
            axum::Router::new().route(
                "/echo",
                axum::routing::post(|body: String| async move { body }),
            ),
            32,
        );
        let (addr, _handle) = serve_test_router(router).await;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("client");
        let url = format!("http://{addr}/echo");
        let oversized = client
            .post(&url)
            .body("x".repeat(33))
            .send()
            .await
            .expect("oversized");
        assert_eq!(oversized.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[test]
    fn with_runtime_routes_rejects_reserved_health() {
        let openapi = OpenApiRouter::new().route(HEALTH, get(|| async { "nope" }));
        let err = with_runtime_routes(openapi, false).expect_err("overlap");
        assert_eq!(err.code(), "RUNTIME.ROUTE_CONFLICT");
    }

    #[tokio::test]
    async fn health_reuses_request_id_and_metrics_count() {
        let request_metrics = RequestMetrics::new();
        let router = with_request_tracking(
            axum::Router::new()
                .route(HEALTH, get(health))
                .route(METRICS, get(metrics)),
            request_metrics,
        );
        let (addr, _handle) = serve_test_router(router).await;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("client");
        let health_response = client
            .get(format!("http://{addr}{HEALTH}"))
            .header(REQUEST_ID, "cli-1")
            .send()
            .await
            .expect("health");
        assert_eq!(
            health_response
                .headers()
                .get(REQUEST_ID)
                .and_then(|value| value.to_str().ok()),
            Some("cli-1")
        );
        let metrics = client
            .get(format!("http://{addr}{METRICS}"))
            .send()
            .await
            .expect("metrics");
        let body: serde_json::Value = metrics.json().await.expect("metrics json");
        let requests = body["requests"].as_u64().expect("requests");
        assert_eq!(requests, 1, "{body}");
    }
}
