//! One Axum router: project routes, OpenAPI, health, and optional `/mcp`.

use axum::Json;
use axum::routing::get;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use sc_command::Envelope;
use serde::Serialize;
use utoipa_axum::router::OpenApiRouter;

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
pub(crate) fn with_runtime_routes(openapi: OpenApiRouter) -> axum::Router {
    let (router, api) = openapi.split_for_parts();
    router
        .route(
            "/openapi.json",
            get(move || {
                let api = api.clone();
                async move { Json(api) }
            }),
        )
        .route("/health", get(health))
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
    router.nest_service("/mcp", service)
}
