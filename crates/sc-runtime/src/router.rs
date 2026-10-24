//! One Axum router: project routes, OpenAPI, health, and optional `/mcp`.

use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use sc_command::Envelope;
use serde::Serialize;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::error::RuntimeError;

const HEALTH: &str = "/health";
const OPENAPI: &str = "/openapi.json";
const MCP: &str = "/mcp";

/// Maximum time one accepted request may spend in a handler.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

const REQUEST_TIMEOUT_BODY: &str = "the handler exceeded its time limit";
const DRAIN_TIMEOUT_BODY: &str = "shutdown finished waiting for this request";

/// Body of `GET /health`. The HTTP response is [`Envelope`]'s server conversion.
#[derive(Debug, Serialize)]
struct HealthBody {
    status: &'static str,
}

async fn health() -> Envelope<HealthBody> {
    Envelope::success(HealthBody { status: "ok" })
}

/// Merges the project router with the document route and the health route.
///
/// The OpenAPI document is the one `split_for_parts` returns. It is served at
/// `GET /openapi.json`. `GET /health` returns status 200 and a success envelope.
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
        .route(HEALTH, get(health)))
}

/// Applies a handler timeout and a shutdown drain bound to the assembled router.
///
/// `shutdown` becomes true when the daemon starts graceful shutdown. Handlers
/// still running then have `drain` left before this layer returns
/// [`StatusCode::SERVICE_UNAVAILABLE`]. A handler that exceeds `limit` from
/// the start of the request gets [`StatusCode::REQUEST_TIMEOUT`].
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
    let sleep_limit = tokio::time::sleep(limit);
    tokio::pin!(sleep_limit);
    let shutdown_started = async {
        if *shutdown.borrow() {
            return;
        }
        loop {
            if shutdown.changed().await.is_err() || *shutdown.borrow() {
                return;
            }
        }
    };
    tokio::pin!(shutdown_started);

    tokio::select! {
        biased;
        response = &mut handler => return response,
        () = &mut sleep_limit => {
            return deadline_response(StatusCode::REQUEST_TIMEOUT, REQUEST_TIMEOUT_BODY);
        }
        () = &mut shutdown_started => {}
    }

    tokio::select! {
        biased;
        response = handler => response,
        () = tokio::time::sleep(drain) => {
            deadline_response(StatusCode::SERVICE_UNAVAILABLE, DRAIN_TIMEOUT_BODY)
        }
        () = sleep_limit => deadline_response(StatusCode::REQUEST_TIMEOUT, REQUEST_TIMEOUT_BODY),
    }
}

fn deadline_response(status: StatusCode, body: &'static str) -> Response {
    (status, body).into_response()
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
    [HEALTH, OPENAPI]
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
                .route(
                    "/slow",
                    get(|| async {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        "late"
                    }),
                ),
            Duration::from_millis(200),
            shutdown_rx,
            Duration::from_secs(30),
        );
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("client");
        let fast_url = format!("http://{addr}/fast");
        let mut fast = None;
        for _ in 0..50 {
            match client.get(&fast_url).send().await {
                Ok(response) => {
                    fast = Some(response);
                    break;
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
            }
        }
        let fast = fast.expect("fast");
        assert_eq!(fast.status(), reqwest::StatusCode::OK);
        let slow = client
            .get(format!("http://{addr}/slow"))
            .send()
            .await
            .expect("slow");
        assert_eq!(slow.status(), reqwest::StatusCode::REQUEST_TIMEOUT);
        assert_eq!(slow.text().await.expect("body"), REQUEST_TIMEOUT_BODY);
        drop(shutdown_tx);
    }

    #[test]
    fn with_runtime_routes_rejects_reserved_health() {
        let openapi = OpenApiRouter::new().route(HEALTH, get(|| async { "nope" }));
        let err = with_runtime_routes(openapi, false).expect_err("overlap");
        assert_eq!(err.code(), "RUNTIME.ROUTE_CONFLICT");
    }
}
