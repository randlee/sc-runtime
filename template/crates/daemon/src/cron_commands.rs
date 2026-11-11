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

/// Explicit trusted targets authorized by the application before daemon startup.
/// Rust's built-in heartbeat works immediately. Python and Monty require registration.
#[derive(Default)]
pub struct RegisteredTargets {
    python: std::collections::BTreeSet<String>,
    monty: std::collections::BTreeSet<String>,
}

impl RegisteredTargets {
    /// Authorizes one configured Python script path, independent of its working directory.
    /// Prefer an absolute path to a repository script. SQLite stores this path, not a copy
    /// of the file: edits to that file affect subsequent runs. Relative paths resolve from
    /// the daemon's working directory when a hook starts, before entering private scratch.
    /// A consumer may register a temporary script, but owns keeping it present for every
    /// scheduled run and removing it only after work is physically joined.
    /// The core's fresh private working directory does not change the script's location.
    #[must_use]
    pub fn with_python(mut self, module: impl Into<String>) -> Self {
        self.python.insert(module.into());
        self
    }

    /// Authorizes one trusted Monty source string. SQLite stores the source snapshot,
    /// so editing an unrelated script file does not change this scheduled program.
    /// The daemon's monty feature must also be explicitly enabled.
    #[must_use]
    pub fn with_monty(mut self, source: impl Into<String>) -> Self {
        self.monty.insert(source.into());
        self
    }
}

impl sc_cron_commands::ApplicationTargetBindings for RegisteredTargets {
    fn resolve(&self, target: &sc_cron_commands::ExecutionTarget, payload: serde_json::Value)
        -> Result<sc_cron_commands::ResolvedTarget, sc_command::OpError> {
        use sc_cron_commands::ExecutionTarget;
        let hook = match target {
            ExecutionTarget::Rust => sc_cron::HookTarget::Rust,
            ExecutionTarget::Python { module } if self.python.contains(module) =>
                sc_cron::HookTarget::Python { module: module.into() },
            ExecutionTarget::Monty { source } if cfg!(feature = "monty") && self.monty.contains(source) =>
                sc_cron::HookTarget::Monty { source: source.clone() },
            _ => return Err(sc_command::OpError::from_code(
                sc_command::ErrorKind::InvalidInput, "CRON.TARGET_NOT_REGISTERED",
                "target is not registered by this application",
            ).with_suggestion("register the trusted target before starting the daemon")),
        };
        Ok(sc_cron_commands::ResolvedTarget { hook, payload })
    }
}

/// Small useful default Rust dispatcher. Replace it through CronServices::with_job.
/// `{"job":"heartbeat"}` emits a fixed event without logging opaque payload values.
pub struct HeartbeatJob;

#[async_trait::async_trait]
impl sc_cron::Job for HeartbeatJob {
    async fn run(&self, payload: &serde_json::Value, deadline: Instant) -> sc_cron::RunOutcome {
        if Instant::now() >= deadline {
            return sc_cron::RunOutcome::TimedOut;
        }
        if payload.get("job").and_then(serde_json::Value::as_str) != Some("heartbeat") {
            return sc_cron::RunOutcome::Refused("application job is not registered".into());
        }
        tracing::info!(target: "cron", "scheduled heartbeat");
        sc_cron::RunOutcome::Ok
    }

    async fn should_stop(&self, _: &serde_json::Value) -> sc_cron::StopDecision {
        sc_cron::StopDecision::Continue
    }
}
