# `sc-runtime-cli` Requirements

These requirements define the `sc-runtime-cli` crate surface and constraints.

## Purpose

`sc-runtime-cli` is the sc-ai-cli infrastructure layer. It owns the machine
contract, command routing, exit code policy, and the type definitions shared
across CLI, RPC, and MCP transports.

## Authoritative Requirement Sources

- [../../docs/prd/sc-runtime-prd.md](../../docs/prd/sc-runtime-prd.md) —
  Sections 6.2, 9 (RBP-001 through RBP-010), 11 (FR-01, FR-02, FR-11), 12 (NF-05)
- [architecture.md](./architecture.md)
- sc-ai-cli skill reference at `synaptic-canvas/packages/sc-ai-cli/skills/creating-ai-clis`

## Scope Rules

The following belong in `sc-runtime-cli`:

- `CommandEnvelope<T>`, `CliError`, `CliErrorKind`, `CommandId`
- `CommandTransport` trait (transport adapter for router injection)
- Built-in lifecycle commands: `start`, `stop`, `restart`, `status`, `health`
- Command registry and routing logic
- Exit code assignment and process exit

The following do not belong in `sc-runtime-cli`:

- Daemon lifecycle, PID files, signal handling (belong in `sc-runtime-daemon`)
- stdio MCP JSON-RPC framing (belongs in `sc-runtime-mcp-stdio`)
- Storage types or backend initialization (belong in `sc-runtime-db*`)
- Domain-specific commands (belong in consumer application code)
- Logger construction (belongs in application code)

## Functional Requirements

### FR-CLI-01 — Every Command Must Support `--json` Producing Stable `CommandEnvelope<T>`

Every command registered in the CLI must accept a `--json` flag. When
`--json` is present, output must be a single JSON object conforming to
`CommandEnvelope<T>` printed to stdout. The shape must be identical whether
the command executed directly or via IPC to the daemon.

**Rationale:** FR-01. Machine contract is primary. MCP clients and CI scripts
depend on `--json` for programmatic consumption.

### FR-CLI-02 — `CliError` Must Carry Kind, Code, Message, Cause, Details, and `suggested_action`

`CliError` must have all six fields populated or explicitly `None` where
optional. No error variant may omit `kind` or `code`. `suggested_action` must
be populated on every variant where an operator corrective action exists.

**Rationale:** RBP-001. The CLI error contract must be machine-readable and
actionable. Opaque string errors are forbidden.

### FR-CLI-03 — `CommandId` Must Be a Stable Dotted Identifier

`CommandId` values must follow the `{tool}.{command}` convention. Built-in
commands use the `runtime.*` namespace. Consumer commands use a tool-owned
namespace. `CommandId` values must not change across patch or minor releases.
Changing a `CommandId` is a semver-breaking change.

**Rationale:** MCP clients, CI pipelines, and test fixtures key on
`CommandId` values for routing and assertion. Instability breaks them silently.

### FR-CLI-04 — Command Router: IPC When Daemon Reachable, Direct When Not

When the daemon feature is active and a daemon socket or named pipe is
reachable, the router must forward the request over IPC and return the
deserialized `CommandEnvelope<T>` to the caller. When the daemon is not
reachable (connection refused, socket absent) or the daemon feature is not
active, the router must execute the command directly. This must be transparent
to the caller — the caller receives `CommandEnvelope<T>` regardless of routing
path.

**Rationale:** FR-11. Operators must be able to run `my-tool status` and get a
result whether or not the daemon is currently running.

### FR-CLI-05 — Same Request/Response Types Used by CLI, RPC, and HTTP

`CommandEnvelope<T>` and `CliError` must be the same Rust types used across
the CLI path, the daemon RPC path, and the MCP transports. No transport-specific
DTOs may be introduced. No reshaping may occur at any transport boundary.

**Rationale:** FR-02, ADR-01. Reshaping creates a maintenance burden and
introduces subtle contract drift between transports that is difficult to detect.

### FR-CLI-06 — Human Output Must Not Be Richer Than `--json` for Machine-Significant Data

Human-readable output (when `--json` is not specified) must be derived from
the same `CommandEnvelope<T>` that `--json` would emit. Any field that a
machine consumer would care about must appear in `--json`. Human output may
format, color, or elide fields for readability, but must not surface
machine-significant data that is absent from `--json`.

**Rationale:** sc-ai-cli contract. If human output is richer, machine consumers
are forced to parse human-formatted text rather than structured JSON.

### FR-CLI-07 — Exit Codes Are CLI-Owned

The CLI layer must assign process exit codes according to the defined policy:

| Code | Condition |
|------|-----------|
| 0 | Success (`ok: true`) |
| 1 | Internal error |
| 2 | Usage error |
| 3 | Configuration error |
| 4 | Capability error |
| 5 | Backend failure |
| 6 | Backend protocol error |

No other layer may call `std::process::exit` or otherwise set the exit code
for CLI invocations. Exit code assignment is derived deterministically from
`CliError.kind`.

**Rationale:** Machine consumers (CI scripts, shell wrappers) rely on exit
codes for control flow. Inconsistent exit codes from different layers break
automated pipelines.

### FR-CLI-08 — MCP JSON Fixture Parity

The same JSON request fixtures used in CLI `--json` tests must produce
equivalent responses when exercised against the stdio MCP path. The
`sc-runtime-cli` operation layer must not contain CLI-specific behavior that
diverges from MCP behavior. Contract tests must exercise both paths with
identical fixtures.

**Rationale:** FR-02, ADR-06. Fixture parity is the enforcement mechanism for
the no-reshaping rule.

### FR-CLI-09 — Transport Adapter Trait Enables Simulator-Backed Testing

The `CommandTransport` trait must be injectable. Integration tests must be
able to provide a simulator implementation that returns pre-configured
responses without spawning a daemon process. The simulator must implement the
same `CommandTransport` trait as the production IPC transport.

**Rationale:** sc-ai-cli `designing-cli-simulators` skill. External process
dependencies in tests introduce non-determinism and slow test execution.

### FR-CLI-10 — `diagnostics` Must Always Be Serialized, Never Omitted

The `diagnostics` field of `CommandEnvelope` must always be present in
serialized JSON output. It must serialize as a JSON array (`[]` when empty).
It must never be omitted via `#[serde(skip_serializing_if)]` or any other
mechanism. A consumer that always expects the field must not have to guard
against its absence.

A successful response (`ok: true`) may carry non-empty diagnostics — for
example, a deprecation warning on a command flag scheduled for removal in
the next major version.

**Rationale:** FR-CLI-01, FR-CLI-05. MCP consumers and CI scripts that
parse `CommandEnvelope` must be able to read `diagnostics` unconditionally.
Conditional serialization forces every consumer to handle two schema shapes.

### FR-CLI-11 — `DiagnosticSeverity` Is Closed to Three Values

`DiagnosticSeverity` must be an exhaustive enum with exactly three variants:
`Info`, `Warning`, `Deprecated`. No catch-all or open-ended variant is
permitted. Serialization must use `snake_case` (`"info"`, `"warning"`,
`"deprecated"`).

Adding a new severity variant is a semver-breaking change. Removing or
renaming a variant is also semver-breaking.

**Rationale:** RBP-003 (Exhaustive Enums). Machine consumers that match on
`DiagnosticSeverity` to decide alerting thresholds must be able to do so
exhaustively. An open-ended variant undermines that guarantee.

## Non-Functional Requirements

### NF-CLI-01 — `unsafe_code = "forbid"`

The `sc-runtime-cli` crate manifest must set `#![forbid(unsafe_code)]`.

### NF-CLI-02 — No Direct `sc-runtime-daemon` Dependency

`sc-runtime-cli` must not have a direct compile-time dependency on
`sc-runtime-daemon`. The IPC transport is injected by the daemon layer when
present. This is enforced by the boundary definition in
`boundaries/sc-runtime-cli/`.

## Dependency Boundary Rules

| Category | Rule |
|----------|------|
| Permitted workspace dependencies | `sc-runtime-core` only |
| Permitted external dependencies | `clap`, `serde`, `serde_json`, `thiserror`, `futures` |
| Forbidden | `sc-runtime-daemon`, `sc-runtime-web`, `sc-runtime-db`, `sc-runtime-db-*`; any SC domain crate |
| Boundary file | `boundaries/sc-runtime-cli/Boundary.toml` |

Note: IPC transport is injected via the `CommandTransport` trait — `sc-runtime-cli`
carries no direct socket or pipe dependency. Daemon connectivity is wired in by
the consumer, not by this crate.

## Related Docs

- [architecture.md](./architecture.md)
- [../sc-runtime-core/requirements.md](../sc-runtime-core/requirements.md)
- [../sc-runtime-mcp-stdio/requirements.md](../sc-runtime-mcp-stdio/requirements.md)
- [../../boundaries/sc-runtime-cli/](../../boundaries/sc-runtime-cli/)
- [../../docs/prd/sc-runtime-prd.md](../../docs/prd/sc-runtime-prd.md)
