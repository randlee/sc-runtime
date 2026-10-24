//! REST adapters. Each handler calls the service and returns its envelope.

use api_types::{CreateWidget, GetWidget, Widget};
use axum::Json;
use axum::extract::State;
use sc_command::Envelope;
use service::Stores;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// Project router. OpenAPI, health, and `/mcp` are mounted by sc-runtime.
pub fn router(stores: Stores) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(create_widget_route))
        .routes(routes!(get_widget_route))
        .with_state(stores)
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
    Envelope::from(service::create_widget(&stores, &body.name).await)
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
    Envelope::from(service::get_widget(&stores, &body.id).await)
}
