//! Temporary composition proof against exact registry-selected identities.
use sc_cron_commands::{CronRequest, CronResponse};
use rmcp::transport::streamable_http_server::{StreamableHttpService, session::local::LocalSessionManager};

#[derive(Clone)]
struct Server;
impl rmcp::ServerHandler for Server {}

fn main() {
    let _parameters: Option<rmcp::handler::server::wrapper::Parameters<CronRequest>> = None;
    let _request = schemars::schema_for!(CronRequest);
    let _response = schemars::schema_for!(sc_command::Envelope<CronResponse>);
    let _openapi_request = <CronRequest as utoipa::PartialSchema>::schema();
    let _openapi_response = <sc_command::Envelope<CronResponse> as utoipa::PartialSchema>::schema();
    let service = StreamableHttpService::new(|| Ok(Server), std::sync::Arc::new(LocalSessionManager::default()), Default::default());
    let (router, _api) = utoipa_axum::router::OpenApiRouter::<()>::new().split_for_parts();
    let _composed = router.nest_service("/mcp", service);
    let _logger: sc_command::observability::SharedCanonicalLog = std::sync::Arc::new(sc_command::observability::NoopCanonicalLog::named_or_default("phase-c-probe"));
    println!("COMPOSITION_PASS");
}
