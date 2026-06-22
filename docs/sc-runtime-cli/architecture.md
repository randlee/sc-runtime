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
