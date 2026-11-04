//! Sole version-1 wire DTOs, shared by CLI, HTTP and MCP projections.
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeMap, future::Future, pin::Pin, time::Instant};

/// Maximum aggregate UTF-8 JSON request size, independently of field limits.
pub const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;

/// Sole versioned cron request; parsing rejects duplicate and unknown fields.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
pub struct CronRequest {
    /// Exact wire contract version, currently "1".
    #[cfg_attr(feature = "schema", schemars(regex(pattern = "^1$")))]
    #[cfg_attr(feature = "schema", schema(pattern = "^1$"))]
    pub version: String,
    /// Tagged standard operation.
    #[serde(flatten)]
    pub operation: CronOperation,
}

impl<'de> Deserialize<'de> for CronRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            version: String,
            command: String,
            request: Box<serde_json::value::RawValue>,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.version != "1" {
            return Err(serde::de::Error::custom("unsupported cron request version"));
        }
        let operation = match wire.command.as_str() {
            "cron.start" => CronOperation::Start(
                serde_json::from_str(wire.request.get()).map_err(serde::de::Error::custom)?,
            ),
            "cron.end" => CronOperation::End(
                serde_json::from_str(wire.request.get()).map_err(serde::de::Error::custom)?,
            ),
            "cron.list" => CronOperation::List(
                serde_json::from_str(wire.request.get()).map_err(serde::de::Error::custom)?,
            ),
            _ => return Err(serde::de::Error::custom("unknown cron command")),
        };
        Ok(Self {
            version: wire.version,
            operation,
        })
    }
}

/// Shared object-safe asynchronous executor result.
pub type ExecutorFuture<'a> =
    Pin<Box<dyn Future<Output = sc_command::Envelope<CronResponse>> + Send + 'a>>;
/// Application-replaceable command execution and physical-work ownership port.
pub trait CronExecutor: Send + Sync {
    /// Executes one request against the caller's real absolute deadline.
    fn execute(&self, request: CronRequest, deadline: Instant) -> ExecutorFuture<'_>;
}

/// Standard schedule-management commands.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(tag = "command", content = "request", deny_unknown_fields)]
pub enum CronOperation {
    /// Create or restart a schedule.
    #[serde(rename = "cron.start")]
    Start(StartRequest),
    /// End one expected generation.
    #[serde(rename = "cron.end")]
    End(EndRequest),
    /// Read a bounded page of safe schedule views.
    #[serde(rename = "cron.list")]
    List(ListRequest),
}

/// Start an application-authorized target on a cron schedule.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct StartRequest {
    /// Validated core schedule name.
    pub name: String,
    /// Existing five-field grammar and supported aliases.
    pub cron: String,
    /// Positive bounded core run timeout in milliseconds.
    pub timeout_ms: u64,
    /// Existing authorized hook target.
    pub target: ExecutionTarget,
    /// Bounded opaque application input.
    pub payload: serde_json::Value,
    /// Persisted dotenv and variable operations.
    pub environment: EnvironmentPolicy,
}

/// End one explicitly fenced schedule generation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct EndRequest {
    /// Exact schedule name.
    pub name: String,
    /// Positive expected generation.
    pub generation: u32,
    /// Bounded terminal reason.
    pub reason: String,
}

/// Request a deterministic bounded ascending-id page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ListRequest {
    /// Active, ended or all rows.
    pub state: ScheduleState,
    /// Maximum rows, from 1 through 1000.
    pub limit: u32,
    /// Exclusive positive row id, or null.
    pub cursor: Option<u64>,
}

/// Current lifecycle selection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ScheduleState {
    /// Active schedule rows.
    Active,
    /// Ended schedule rows.
    Ended,
    /// All schedule rows.
    All,
}

/// Existing handler kinds; authorization remains application-owned.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExecutionTarget {
    /// Application-configured Rust Job dispatcher.
    Rust,
    /// Configured Python module path.
    Python { module: String },
    /// Explicitly enabled Monty hook source.
    Monty { source: String },
}

/// Safe target identity without source or environment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    /// Rust target.
    Rust,
    /// Python target.
    Python,
    /// Monty target.
    Monty,
}

/// Ordered dotenv inputs followed by final per-name operations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct EnvironmentPolicy {
    /// Ordered absolute dotenv sources.
    pub dotenv: Vec<DotenvSource>,
    /// Portable environment names and operations.
    pub variables: BTreeMap<String, EnvironmentOperation>,
}

/// A persisted dotenv path with explicit absence policy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct DotenvSource {
    /// Absolute normalized UTF-8 path.
    pub path: String,
    /// Whether a missing file fails execution.
    pub required: bool,
}

/// Persisted set/reference/clear intent, resolved only for an occurrence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum EnvironmentOperation {
    /// Explicit persisted plaintext.
    Set { value: String },
    /// Application-owned opaque locator.
    Reference { locator: String },
    /// Remove any previously supplied value.
    Clear,
}

/// Typed operation result in the canonical command Envelope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(
    tag = "result",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CronResponse {
    /// Started schedule identity and safe view.
    Started(ScheduleView),
    /// Ended schedule identity.
    Ended(EndResult),
    /// Bounded safe page.
    Listed(ListResult),
}

/// Safe current row view without payload, environment or source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ScheduleView {
    /// Positive persisted schedule id.
    pub id: u64,
    /// Positive generation fence.
    pub generation: u32,
    /// Validated schedule name.
    pub name: String,
    /// Persisted expression.
    pub cron: String,
    /// Handler kind only.
    pub target: TargetKind,
    /// Whether scheduling is enabled.
    pub enabled: bool,
    /// Current active or ended state.
    pub state: ScheduleState,
    /// Next Unix timestamp in milliseconds when applicable.
    pub next_run_ms: Option<i64>,
}

/// Ended row and generation identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct EndResult {
    /// Persisted schedule id.
    pub id: u64,
    /// Persisted generation.
    pub generation: u32,
}

/// Safe page and exclusive continuation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ListResult {
    /// Ascending-id rows.
    pub items: Vec<ScheduleView>,
    /// Exclusive next-page cursor, or null.
    pub next_cursor: Option<u64>,
}
