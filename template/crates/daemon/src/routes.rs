//! REST adapters. Each handler calls the service and returns its envelope.
//!
//! sc-runtime mounts `/health`. That route reports process liveness only. It
//! does not check the widget database. `GET /ready` is this project's
//! readiness route: it probes store connectivity and is not liveness.

use std::sync::atomic::{AtomicU64, Ordering};

use api_types::{CreateWidget, GetWidget, Widget};
#[cfg(feature = "general-db")]
use api_types::{DeleteRow, DeleteWidget, DeletedRow};
use axum::Json;
use axum::extract::{Request, State};
use axum::middleware::{self, Next};
use axum::response::Response;
use sc_command::Envelope;
use service::Stores;
use tracing::Instrument;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// Routes without state. Widget routes are always registered. General delete
/// routes are registered when the `general-db` feature is on.
pub fn openapi_router() -> OpenApiRouter<Stores> {
    let router = OpenApiRouter::new()
        .routes(routes!(ready_route))
        .routes(routes!(create_widget_route))
        .routes(routes!(get_widget_route));
    #[cfg(feature = "general-db")]
    let router = router
        .routes(routes!(delete_widget_route))
        .routes(routes!(delete_row_route));
    router
}

/// Project router. OpenAPI and `/health` are mounted by sc-runtime.
///
/// `GET /ready` stays on this router so it can read [`Stores`].
pub fn router(stores: Stores) -> OpenApiRouter {
    openapi_router()
        .route_layer(middleware::from_fn(request_id))
        .with_state(stores)
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
struct ReadyBody {
    status: &'static str,
}

#[utoipa::path(
    get,
    path = "/ready",
    responses((status = 200, description = "Stores are reachable", body = ReadyBody))
)]
async fn ready_route(State(stores): State<Stores>) -> Envelope<ReadyBody> {
    Envelope::from(service::ready(&stores).await.map(|()| ReadyBody { status: "ready" }))
}

#[utoipa::path(
    post,
    path = "/ops/widget.create",
    request_body = CreateWidget,
    responses((status = 200, description = "Created widget", body = Widget))
)]
async fn create_widget_route(
    State(stores): State<Stores>,
    Json(body): Json<CreateWidget>,
) -> Envelope<Widget> {
    Envelope::from(service::create_widget(&stores, body).await)
}

#[utoipa::path(
    post,
    path = "/ops/widget.get",
    request_body = GetWidget,
    responses((status = 200, description = "Fetched widget", body = Widget))
)]
async fn get_widget_route(
    State(stores): State<Stores>,
    Json(body): Json<GetWidget>,
) -> Envelope<Widget> {
    Envelope::from(service::get_widget(&stores, body).await)
}

#[cfg(feature = "general-db")]
#[utoipa::path(
    post,
    path = "/ops/widget.delete",
    request_body = DeleteWidget,
    responses((status = 200, description = "Deleted widget", body = DeletedRow))
)]
async fn delete_widget_route(
    State(stores): State<Stores>,
    Json(body): Json<DeleteWidget>,
) -> Envelope<DeletedRow> {
    Envelope::from(service::delete_widget(&stores, body).await)
}

#[cfg(feature = "general-db")]
#[utoipa::path(
    post,
    path = "/ops/delete.row",
    request_body = DeleteRow,
    responses((status = 200, description = "Deleted row", body = DeletedRow))
)]
async fn delete_row_route(
    State(stores): State<Stores>,
    Json(body): Json<DeleteRow>,
) -> Envelope<DeletedRow> {
    Envelope::from(service::delete_row(&stores, &body.table, &body.id).await)
}

async fn request_id(request: Request, next: Next) -> Response {
    let request_id = request_id_value(request.headers());
    let span = tracing::info_span!("request", request_id = %request_id);
    let mut response = next.run(request).instrument(span).await;
    if let Ok(value) = axum::http::HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn request_id_value(headers: &axum::http::HeaderMap) -> String {
    if let Some(value) = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        && (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return value.to_owned();
    }
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let tick = NEXT.fetch_add(1, Ordering::Relaxed);
    format!("req-{tick}")
}

#[cfg(test)]
mod tests {
    use api_types::{DELETE_ROW_PATH, WIDGET_CREATE_PATH, WIDGET_DELETE_PATH, WIDGET_GET_PATH};

    use super::*;

    #[test]
    fn openapi_paths_match_api_types() {
        let router = openapi_router();
        let spec = router.get_openapi();
        let paths: Vec<&str> = spec.paths.paths.keys().map(String::as_str).collect();
        assert!(
            paths.contains(&WIDGET_CREATE_PATH),
            "create path missing from {paths:?}"
        );
        assert!(
            paths.contains(&WIDGET_GET_PATH),
            "get path missing from {paths:?}"
        );
        assert!(paths.contains(&"/ready"), "ready path missing from {paths:?}");
        #[cfg(feature = "general-db")]
        {
            assert!(
                paths.contains(&WIDGET_DELETE_PATH),
                "widget delete path missing from {paths:?}"
            );
            assert!(
                paths.contains(&DELETE_ROW_PATH),
                "delete row path missing from {paths:?}"
            );
        }
    }

    #[cfg(not(feature = "general-db"))]
    #[test]
    fn openapi_paths_omit_general_delete() {
        let router = openapi_router();
        let spec = router.get_openapi();
        let paths: Vec<&str> = spec.paths.paths.keys().map(String::as_str).collect();
        assert!(
            !paths.contains(&WIDGET_DELETE_PATH),
            "widget delete still registered: {paths:?}"
        );
        assert!(
            !paths.contains(&DELETE_ROW_PATH),
            "delete row still registered: {paths:?}"
        );
        assert!(paths.contains(&WIDGET_CREATE_PATH), "{paths:?}");
        assert!(paths.contains(&WIDGET_GET_PATH), "{paths:?}");
        assert!(paths.contains(&"/ready"), "{paths:?}");
    }
}
