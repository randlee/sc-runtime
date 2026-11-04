//! Sole version-1 wire DTOs, shared by CLI, HTTP and MCP projections.
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeMap, future::Future, pin::Pin, time::Instant};

/// Maximum aggregate UTF-8 JSON request size, independently of field limits.
pub const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;

/// Sole versioned cron request; parsing rejects duplicate and unknown fields.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CronRequest {
    /// Exact wire contract version, currently "1".
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
    #[serde(deserialize_with = "unique_variables")]
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

/// Persisted environment intent using the existing core wire representation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentOperation {
    /// Explicit persisted plaintext: `{ "set": "value" }`.
    Set(String),
    /// Application-owned opaque locator: `{ "reference": "locator" }`.
    Reference(String),
    /// Remove earlier values: `{ "clear": true }`; false fails validation.
    Clear(bool),
}

fn unique_variables<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, EnvironmentOperation>, D::Error> {
    struct Unique;
    impl<'de> serde::de::Visitor<'de> for Unique {
        type Value = BTreeMap<String, EnvironmentOperation>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("unique environment variable names")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut variables = BTreeMap::new();
            while let Some((name, operation)) = map.next_entry::<String, EnvironmentOperation>()? {
                if variables.insert(name, operation).is_some() {
                    return Err(serde::de::Error::custom("duplicate environment variable"));
                }
            }
            Ok(variables)
        }
    }
    deserializer.deserialize_map(Unique)
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

#[cfg(feature = "schema")]
fn request_schema(
    start: serde_json::Value,
    end: serde_json::Value,
    list: serde_json::Value,
) -> serde_json::Value {
    let variants = [
        ("cron.start", start),
        ("cron.end", end),
        ("cron.list", list),
    ]
    .into_iter()
    .map(|(command, request)| {
        serde_json::json!({
            "type":"object", "additionalProperties":false,
            "required":["version","command","request"],
            "properties":{
                "version":{"type":"string","enum":["1"]},
                "command":{"type":"string","enum":[command]},
                "request":request
            }
        })
    })
    .collect::<Vec<_>>();
    serde_json::json!({"oneOf":variants})
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for CronRequest {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "CronRequest".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        serde_json::from_value(request_schema(
            generator.subschema_for::<StartRequest>().to_value(),
            generator.subschema_for::<EndRequest>().to_value(),
            generator.subschema_for::<ListRequest>().to_value(),
        ))
        .expect("typed request schema is an object")
    }
}

#[cfg(feature = "schema")]
impl utoipa::PartialSchema for CronRequest {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        serde_json::from_value(request_schema(
            serde_json::json!({"$ref":"#/components/schemas/StartRequest"}),
            serde_json::json!({"$ref":"#/components/schemas/EndRequest"}),
            serde_json::json!({"$ref":"#/components/schemas/ListRequest"}),
        ))
        .expect("typed request schema is valid OpenAPI")
    }
}

#[cfg(feature = "schema")]
impl utoipa::ToSchema for CronRequest {
    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        schemas.push((
            "StartRequest".into(),
            <StartRequest as utoipa::PartialSchema>::schema(),
        ));
        schemas.push((
            "EndRequest".into(),
            <EndRequest as utoipa::PartialSchema>::schema(),
        ));
        schemas.push((
            "ListRequest".into(),
            <ListRequest as utoipa::PartialSchema>::schema(),
        ));
        <StartRequest as utoipa::ToSchema>::schemas(schemas);
        <EndRequest as utoipa::ToSchema>::schemas(schemas);
        <ListRequest as utoipa::ToSchema>::schemas(schemas);
    }
}
