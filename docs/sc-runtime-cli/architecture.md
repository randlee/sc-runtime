# `sc-runtime-cli` Architecture

This document records the architecture of the `sc-runtime-cli` crate.

## Role

`sc-runtime-cli` applies the sc-ai-cli framework as reusable infrastructure.
Machine contract is primary; human output is secondary. Every aspect of the
crate's design — envelope shape, error structure, type sharing, exit codes,
and the command router — follows from that principle.

This crate does not define domain commands. It defines the infrastructure layer
that domain commands are built on, and provides a set of built-in lifecycle
commands (`start`, `stop`, `restart`, `status`, `health`) that every sc-runtime
tool gets for free.

## sc-ai-cli Compliance

`sc-runtime-cli` is built to the sc-ai-cli contract defined in
`synaptic-canvas/packages/sc-ai-cli/skills/creating-ai-clis`. The key
compliance points:

- Every command exposes `--json` producing a stable `CommandEnvelope<T>`.
- Human output is always a rendering of the same data as `--json`. Human output
  is never richer than `--json` for any machine-significant field.
- `CliError` carries `kind`, `code`, `message`, `cause`, `details`, and
  `suggested_action` — the full sc-ai-cli error contract.
- `CommandId` is a stable dotted identifier (`runtime.status`,
  `runtime.health`). It is stable across patch and minor releases.
- The MCP-ready seam is present from day one: the same request/response types
  used by the CLI are used by daemon RPC and by the stdio MCP transport.
  There is no reshaping at any transport boundary.

## `CommandEnvelope<T>`

`CommandEnvelope<T>` is the top-level JSON wrapper for every command response.
Its shape is identical across all command families and all transports.

```rust
pub struct CommandEnvelope<T: Serialize> {
    pub ok: bool,
    pub command: CommandId,
    pub data: Option<T>,
    pub error: Option<CliError>,
    pub diagnostics: Vec<Diagnostic>,
}
```

`T` is the command-specific success payload. When `ok` is `true`, `data` is
`Some`. When `ok` is `false`, `error` is `Some`. `diagnostics` carries
non-fatal observations regardless of outcome.

### Stability Guarantee

The top-level field names and types of `CommandEnvelope` are stable. Adding
new fields to `T` is non-breaking. Removing or renaming top-level fields is a
semver-breaking change, subject to `sc-lint-version` enforcement.

### `Diagnostic`

`Diagnostic` is a non-fatal warning or informational message attached to any
`CommandEnvelope`. It is defined here because it is part of the stable
`CommandEnvelope` shape shared across all transports.

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,           // stable dotted identifier, e.g. "runtime.db.slow_query"
    pub message: String,        // human-readable description
    pub detail: Option<String>, // optional extended detail
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Deprecated,
}
```

Key invariants:

- `diagnostics` in `CommandEnvelope` is always serialized as a JSON array.
  It is never omitted and never `null`. An empty response carries
  `"diagnostics": []`.
- A successful response (`ok: true`) may carry diagnostics — for example,
  deprecation warnings on a command whose flags are scheduled for removal.
- `Diagnostic.code` follows the same dotted convention as `CommandId` but
  is **not** a `CommandId`. It identifies the condition, not the command that
  triggered it. Codes use a `{layer}.{subsystem}.{name}` form, e.g.
  `runtime.db.slow_query` or `runtime.plugin.deprecated_api`.
- `DiagnosticSeverity` is a closed enum. New severities are semver-breaking.

## `CliError`

```rust
pub struct CliError {
    pub kind: CliErrorKind,
    pub code: ErrorCode,
    pub message: String,
    pub cause: Option<String>,
    pub details: Option<serde_json::Value>,
    pub suggested_action: Option<String>,
}
```

`CliErrorKind` is an exhaustive enum — not an open trait — so that exhaustive
matching is possible in machine consumers (RBP-003):

```
usage | config | capability | backend_failure | backend_protocol | internal
```

`code` follows the `CLI.{KIND}.{NAME}` convention (e.g., `CLI.USAGE_ERROR`,
`CLI.CONFIG.MISSING_KEY`). Codes are stable across minor releases.

`suggested_action` is populated on every variant where an operator can take a
corrective action. It is `None` only on `internal` errors where no user action
helps.

## `CommandId`

`CommandId` is a newtype over `Cow<'static, str>` (RBP-004, RBP-009):

```rust
pub struct CommandId(Cow<'static, str>);
```

`CommandId::new` accepts `impl Into<Cow<'static, str>>`. For the common case
of a `'static` string literal, no allocation occurs. Dynamic command
registration (e.g., plugin-provided commands) can use `String::into()`.

`CommandId` implements `Deref<Target = str>` (RBP-005), `Display`, `Serialize`,
and `Deserialize`. It is the canonical key used in the command router dispatch
table, in `CommandEnvelope.command`, and in RPC wire messages.

## Built-In Commands

`sc-runtime-cli` provides these built-in commands. Consumers extend the
registry with domain-specific commands via the callback passed to `.cli()` on
the builder.

| Command | `--json` type | Description |
|---------|--------------|-------------|
| `start` | `CommandEnvelope<StartResult>` | Start daemon or run directly |
| `stop` | `CommandEnvelope<StopResult>` | Send shutdown to daemon |
| `restart` | `CommandEnvelope<RestartResult>` | Stop then start |
| `status` | `CommandEnvelope<StatusResult>` | Daemon state, PID, uptime |
| `health` | `CommandEnvelope<HealthResult>` | Plugin-level health checks |

Each command is exposed at the clap subcommand level and accepts `--json` to
switch output format. Without `--json`, human-readable output is printed;
internally it is derived from the same `CommandEnvelope<T>` that `--json`
would emit.

## Command Router

The command router is responsible for deciding whether a command executes
directly in the current process or is forwarded to a running daemon over IPC.

### Routing Logic

```
if daemon socket or named pipe is reachable:
    serialize request → forward over IPC → deserialize response
else:
    execute command directly in current process
```

This routing is transparent to the caller. The caller receives a
`CommandEnvelope<T>` regardless of which path was taken.

When the `sc-runtime-daemon` layer is not included in the build, the router
always takes the direct path. The IPC path is a compile-time capability, not
a runtime configuration choice.

### Transport Adapter Trait

The router's transport is an injected trait:

```rust
pub trait CommandTransport: Send + Sync {
    fn send(
        &self,
        request: &CommandEnvelope<serde_json::Value>,
    ) -> Result<CommandEnvelope<serde_json::Value>, CliError>;
}
```

The production implementation connects to the daemon socket or named pipe.
Test implementations can inject a simulator that returns pre-configured
responses.

### Simulator-Backed Testing

Per the sc-ai-cli `designing-cli-simulators` skill, external integrations must
be testable via a stateful simulator implementing the same adapter trait. The
`CommandTransport` trait is the hook for this: integration tests inject a
simulator transport that mimics daemon behavior without requiring a running
daemon process. This enables deterministic, parallel, no-process-spawn CLI
integration tests.

## CommandRegistry

`CommandRegistry` is the mechanism by which consumers register application-specific commands. An instance is passed by mutable reference to the callback provided to `.cli()` on the builder:

```rust
// CommandRegistry — builder for registering application commands
// Passed to the .cli() builder method as a mutable reference
pub struct CommandRegistry { /* opaque */ }

impl CommandRegistry {
    // Register a command with its handler
    // name: stable dotted CommandId string, e.g. "tool.process"
    // handler: processes a CommandEnvelope request, returns a CommandEnvelope response
    pub fn register<Req, Resp>(
        &mut self,
        name: impl Into<CommandId>,
        handler: impl Fn(CommandEnvelope<Req>) -> Result<CommandEnvelope<Resp>, CliError>
            + Send + Sync + 'static,
    ) where
        Req: DeserializeOwned + Send + 'static,
        Resp: Serialize + Send + 'static;

    // Built-in commands (start, stop, restart, status, health) are registered
    // automatically — consumer cannot override them
}
```

The consumer's registration callback receives a `&mut CommandRegistry` and calls `register` for each domain command. Built-in lifecycle commands are pre-registered; any attempt to register a command with a `runtime.*` namespace is rejected at startup with a `CliErrorKind::usage` error.

---

## Built-in Command Response Types

The five built-in commands each return a concrete result type as the `T` in `CommandEnvelope<T>`. These types are defined in `sc-runtime-cli` and re-exported from `sc-runtime`.

```rust
pub struct StartResult {
    pub pid: u32,
    pub socket_path: Option<String>,   // Unix socket path, if daemon mode
    pub pipe_name: Option<String>,     // Windows named pipe, if daemon mode
    pub tcp_port: Option<u16>,         // TCP fallback port, if used
    pub started_at: String,            // RFC 3339 timestamp
}

pub struct StopResult {
    pub stopped_pid: u32,
    pub exit_code: Option<i32>,        // None if forcibly killed
    pub stopped_at: String,            // RFC 3339 timestamp
}

pub struct RestartResult {
    pub stopped_pid: u32,
    pub new_pid: u32,
    pub restarted_at: String,          // RFC 3339 timestamp
}

pub struct StatusResult {
    pub running: bool,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub version: String,
    pub plugins: Vec<PluginStatus>,
}

pub struct PluginStatus {
    pub name: String,
    pub state: PluginState,
    pub uptime_seconds: Option<u64>,
}

pub enum PluginState {
    Initializing,
    Running,
    ShuttingDown,
    Failed { reason: String },
}

pub struct HealthResult {
    pub healthy: bool,                 // true only if ALL checks pass
    pub checks: Vec<HealthCheck>,
    pub storage: Vec<StorageHealthEntry>,  // one per configured backend
}

pub struct HealthCheck {
    pub name: String,
    pub healthy: bool,
    pub message: Option<String>,
}

pub struct StorageHealthEntry {
    pub backend: String,               // "sqlite", "sqlx-sqlite", "sqlx-postgres"
    pub health: StorageHealthSummary,  // Healthy / Degraded / Unavailable
    pub reason: Option<String>,
}

pub enum StorageHealthSummary { Healthy, Degraded, Unavailable }
```

**`HealthResult` aggregation rule:** `healthy` is `true` only when ALL plugin checks are healthy AND ALL storage backends report `Healthy`. A single `Degraded` backend makes the overall result unhealthy.

All five result types implement `Serialize` and `Deserialize`. They are stable types subject to the same semver rules as `CommandEnvelope`: adding fields is non-breaking; removing or renaming fields is semver-breaking.

---

## Command Router: Error and Timeout Behavior

This section specifies the exact behavior of the command router for all failure
and timeout cases. These cases were previously under-specified.

### Connect Timeout and Reachability

"Reachable" means a TCP/socket connect completes within **500ms**. File
existence of a socket path or pipe name is NOT sufficient — the connect attempt
must succeed within that window.

If the connect attempt fails or times out (no listener, connection refused,
or 500ms exceeded), the router falls back to direct execution. This is not an
error — it is the designed fallback. The caller receives a `CommandEnvelope<T>`
produced by the direct path.

This means: a socket/pipe path that exists but whose connect is refused (e.g.,
daemon starting up) is treated identically to "unreachable" — the router falls
back to direct execution.

### Mid-Read Timeout

**10 seconds** from connection established to first complete newline-delimited
JSON response. If this window is exceeded, the connection is dropped and the
call returns:

```rust
CliError {
    kind: CliErrorKind::BackendTimeout,
    code: "SC_RUNTIME.CLI.RPC_TIMEOUT",
    message: "Daemon did not respond within 10 seconds",
    suggested_action: Some("Check daemon health with `status` or restart with `restart`"),
    ..
}
```

This is **NOT** a fallback to direct execution. A mid-read timeout means the
daemon was reached and accepted the connection but is not delivering a response.
That is a different condition from "unreachable" — falling back to direct
execution could produce a result that conflicts with what the daemon is already
processing.

### Partial Read / Malformed JSON

If the full response is received (newline delimiter found) but the content is
not valid JSON, return:

```rust
CliError {
    kind: CliErrorKind::BackendProtocol,
    code: "SC_RUNTIME.CLI.RPC_MALFORMED_RESPONSE",
    message: "Daemon returned a response that is not valid JSON",
    ..
}
```

Do not retry. Do not fall back to direct execution.

### Retry Policy

**Zero retries.** The router never retries a failed or timed-out RPC call.
Idempotency is the caller's responsibility. Retrying commands like `start`
without caller awareness could cause duplicate-start errors that are difficult
to diagnose.

### Daemon Shutdown Transition

If the daemon is in shutdown transition (connection accepted but no response
delivered before the connection closes), this manifests as either a mid-read
timeout or a connection drop before a complete response is received. Both cases
are treated as `BackendTimeout` or `BackendProtocol` errors respectively —
they are NOT silent fallbacks to direct execution.

### Summary Table

| Condition | Router Action | Error Kind |
|-----------|---------------|------------|
| Socket/pipe unreachable, connect refused, or 500ms timeout | Fall back to direct execution | — (not an error) |
| Socket/pipe file exists but connect refused | Fall back to direct execution | — (same as unreachable) |
| Connect succeeds, no response within 10s | Return error, drop connection | `BackendTimeout` / `SC_RUNTIME.CLI.RPC_TIMEOUT` |
| Connect succeeds, response is malformed JSON | Return error, no retry | `BackendProtocol` / `SC_RUNTIME.CLI.RPC_MALFORMED_RESPONSE` |
| Daemon in shutdown (connection dropped mid-read) | Return error | `BackendTimeout` or `BackendProtocol` |

---


## IPC Path vs Direct Execution Path

| Aspect | IPC path | Direct path |
|--------|----------|-------------|
| Requires daemon | yes | no |
| Request type | `CommandEnvelope<T>` serialized to NDJSON | Rust function call |
| Response type | `CommandEnvelope<T>` deserialized from NDJSON | `Result<CommandEnvelope<T>, CliError>` |
| Availability | when daemon feature active and socket reachable | always |

Both paths produce the same `CommandEnvelope<T>` to the caller. The CLI layer
never exposes which path was taken.

## Exit Code Policy

Exit codes are owned by the CLI layer. No other layer assigns process exit
codes.

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Internal error (sc-runtime bug or panic) |
| 2 | Usage error (bad arguments, unknown command) |
| 3 | Configuration error (missing or invalid config) |
| 4 | Capability error (feature not available in this build) |
| 5 | Backend failure (daemon, storage, or plugin error) |
| 6 | Backend protocol error (malformed RPC response) |

Exit codes are stable. Adding new codes requires a semver minor bump.
Reassigning an existing code is semver-breaking.

The exit code is derived from `CliError.kind` when `ok` is `false`:
`usage → 2`, `config → 3`, `capability → 4`, `backend_failure → 5`,
`backend_protocol → 6`, `internal → 1`.

## Type Sharing Across Transports

`CommandEnvelope<T>` and `CliError` are the same Rust types used by:

- the CLI binary (`--json` output)
- the daemon RPC layer (NDJSON over Unix socket, named pipe, or TCP)
- the stdio MCP transport (`sc-runtime-mcp-stdio`)
- the HTTP MCP transport (`sc-runtime-web`)

There is no reshaping at any boundary. No transport-specific DTOs exist.
This is the MCP-ready seam from ADR-01: the machine contract is defined once
and the transports are wrappers.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `sc-runtime-core` | `PluginContext`, `ScRuntimeError`, `ScRuntimeHome` |
| `clap` 4 (derive) | CLI argument parsing and subcommand dispatch |
| `serde` / `serde_json` | `CommandEnvelope<T>` and `CliError` serialization |
| `thiserror` | `CliError` derivation |

`sc-runtime-cli` does not depend on `sc-runtime-daemon`. The IPC transport
implementation is injected at the point where the daemon layer wires the
router.

## Related Docs

- [requirements.md](./requirements.md)
- [../sc-runtime-core/architecture.md](../sc-runtime-core/architecture.md)
- [../sc-runtime-mcp-stdio/architecture.md](../sc-runtime-mcp-stdio/architecture.md)
- [../../boundaries/sc-runtime-cli/](../../boundaries/sc-runtime-cli/)
- [../../docs/prd/sc-runtime-prd.md](../../docs/prd/sc-runtime-prd.md) — Section 6.2, 9, 11
