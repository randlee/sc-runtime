{% if cron_commands %}
//! Injectable HTTP projections; the application owns selected root registration.
//! Mount on sc-runtime's router so its absolute Instant extension is available.
use std::{sync::Arc, time::Instant};
use axum::{Extension, Json, extract::{State, rejection::JsonRejection}};
use sc_command::Envelope;
use sc_cron_commands::{CronExecutor, CronRequest, CronResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

type Executor = Arc<dyn CronExecutor>;

/// Builds all three generic cron routes around an application-owned executor.
/// This does not construct a scheduler or register commands in production.
pub fn router(executor: Executor) -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(start)).routes(routes!(end))
        .routes(routes!(list)).with_state(executor)
}

async fn dispatch(executor: Executor, deadline: Instant, verb: &str,
                  body: Result<Json<CronRequest>, JsonRejection>) -> Envelope<CronResponse> {
    match body {
        Ok(Json(request)) => service::cron_commands::execute(executor.as_ref(), verb, request, deadline).await,
        Err(_) => service::cron_commands::invalid_request(),
    }
}

#[utoipa::path(post, path = "/ops/cron.start", request_body = CronRequest,
    responses((status = 200, body = Envelope<CronResponse>), (status = 400, body = Envelope<CronResponse>)))]
async fn start(State(executor): State<Executor>, Extension(deadline): Extension<Instant>,
               body: Result<Json<CronRequest>, JsonRejection>) -> Envelope<CronResponse> {
    dispatch(executor, deadline, "cron.start", body).await
}

#[utoipa::path(post, path = "/ops/cron.end", request_body = CronRequest,
    responses((status = 200, body = Envelope<CronResponse>), (status = 400, body = Envelope<CronResponse>)))]
async fn end(State(executor): State<Executor>, Extension(deadline): Extension<Instant>,
             body: Result<Json<CronRequest>, JsonRejection>) -> Envelope<CronResponse> {
    dispatch(executor, deadline, "cron.end", body).await
}

#[utoipa::path(post, path = "/ops/cron.list", request_body = CronRequest,
    responses((status = 200, body = Envelope<CronResponse>), (status = 400, body = Envelope<CronResponse>)))]
async fn list(State(executor): State<Executor>, Extension(deadline): Extension<Instant>,
              body: Result<Json<CronRequest>, JsonRejection>) -> Envelope<CronResponse> {
    dispatch(executor, deadline, "cron.list", body).await
}
{% endif %}
