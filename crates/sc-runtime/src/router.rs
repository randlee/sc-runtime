//! One Axum router: project routes, OpenAPI, liveness, and optional `/mcp`.
//!
//! `GET /health` is liveness. The handler reports that this process is serving
//! HTTP. Store and dependency readiness stay on a project route such as
//! `GET /ready`. That path is left unreserved, and `/health` stays liveness.

use std::sync::Arc;

use axum::Json;
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
        .route(HEALTH, get(health)))
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

    #[test]
    fn with_runtime_routes_rejects_reserved_health() {
        let openapi = OpenApiRouter::new().route(HEALTH, get(|| async { "nope" }));
        let err = with_runtime_routes(openapi, false).expect_err("overlap");
        assert_eq!(err.code(), "RUNTIME.ROUTE_CONFLICT");
    }
}
