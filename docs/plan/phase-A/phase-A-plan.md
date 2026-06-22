# Phase A: Foundation

**Version target:** 0.1.0-alpha.1
**Entry state:** Empty workspace
**Exit state:** Working CLI + stdio MCP that tool consumers can build against

Related: [Project Plan](../project-plan.md) · [PRD §14 Phase A](../../prd/sc-runtime-prd.md)

---

## Objectives

By the end of Phase A, a consumer can write a Rust binary that depends on `sc-runtime` and gets:

- A CLI entrypoint where every command exposes `--json` producing a stable `CommandEnvelope<T>`, satisfying FR-01
- A stdio MCP server over stdin/stdout, sharing the same request/response types as the CLI with zero reshaping, satisfying FR-02 and FR-03
- Structured logging via an injected `Logger<Running>` from sc-observability, satisfying FR-15
- `ScRuntime::run()` returning `Result<(), ScRuntimeError>` — never panicking in normal operation, satisfying FR-14 and FR-16
- All sc-lint gates (boundary, portability, runtime) passing in CI, satisfying FR-13

Phase A does not include storage, daemon, RPC, web, or the HTTP MCP endpoint. Those capabilities are added in Phases B through D. The facade builder exposes only the `NoCli→HasCli` typestate transition in this phase.

---

## Crates Delivered

| Crate | Status at end of phase | Complete | Deferred to |
|-------|------------------------|----------|-------------|
| `sc-runtime-core` | v0.1.0-alpha.1 | `Plugin` trait (object-safe via `BoxFuture`), `PluginMetadata`, `PluginContext`, `ScRuntimeError` (RBP-001 compliant), `ScRuntimeHome` (SC_RUNTIME_HOME env override), `CancellationToken` re-export | – |
| `sc-runtime-cli` | v0.1.0-alpha.1 | `CommandEnvelope<T>`, `CliError`, `CliErrorKind`, `CommandId`, 5 built-in commands (`start`, `stop`, `restart`, `status`, `health`), exit codes, `--json` flag, transport adapter trait (simulator-backed testing), command router stub (direct path only; IPC path added Phase C) | Daemon IPC router path (Phase C) |
| `sc-runtime-mcp-stdio` | v0.1.0-alpha.1 | JSON-RPC 2.0 over stdin/stdout, operation dispatch via `sc-runtime-cli` operation layer, fixture-parity tests | – |
| `sc-runtime` | v0.1.0-alpha.1 | `ScRuntimeBuilder<NoCli>`, `.name()`, `ScRuntimeBuilder<NoCli>::cli()→HasCli`, `.mcp_stdio()`, `.logger()`, `.build()`, `.run()` | `.daemon()`, `.db()`, `.web()`, `.mcp_http()` |
| `sc-runtime-example` | v0.1.0-alpha.1 | Minimal consumer binary exercising CLI `--json` and stdio MCP paths with the same command fixtures | Extended each subsequent phase |

---

## What Is NOT in Phase A

- Storage (`StorageBackend`, `SqliteBackend`, `SqlxBackend`) — Phase B
- Daemon lifecycle, PID file, signal handling — Phase C
- Daemon RPC server and the IPC path in the CLI router — Phase C
- Web server and HTTP MCP SSE endpoint — Phase D
- sc-lint-version gate (`cargo-semver-checks`) — Phase E
- xwin CI step — Phase E (portability lints still active; xwin compilation check deferred)

---

## Workspace Setup Tasks

The following workspace-level files are created in Sprint A-1 and are not changed by subsequent phases except to add content:

**`Cargo.toml` (workspace root)**
- `resolver = "2"`, `edition = "2024"`, `rust-version = "1.94.1"`
- All 10 sub-crates listed in `[workspace.members]` including stub entries for Phase B–D crates
- `[workspace.lints.rust]`: `unsafe_code = "forbid"`
- `[workspace.lints.clippy]`: `pedantic = "deny"`, `nursery = "deny"`

**`.cargo/config.toml`**
- `[build] rustflags = ["-D", "unknown-lints"]` — deny unknown lint names workspace-wide

**`rust-toolchain.toml`**
- `channel = "1.94.1"`, `components = ["clippy", "rustfmt"]`

**`deny.toml`**
- `[licenses]`: allow list covering MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0
- `[advisories]`: `db-path = "~/.cargo/advisory-db"`, `vulnerability = "deny"`
- `[bans]`: `duplicates = "warn"` — escalated to deny before Phase E publish

**`Justfile`**
- `lint` → `sc-lint lint full`
- `lint fast` → `sc-lint lint fast`
- `lint ci` → `sc-lint lint ci`
- `ci` → `sc-lint ci`
- `test` → `cargo test --workspace`
- `check` → `cargo check --workspace`
- `clippy` → `cargo clippy --workspace --all-targets`

**`boundaries/` directory**
Written in Sprint A-1, enforced immediately:
- `boundaries/sc-runtime-core/Boundary.toml` — zero SC domain crate dependencies
- `boundaries/sc-runtime-cli/Boundary.toml` — only `sc-runtime-core`
- `boundaries/sc-runtime-mcp-stdio/Boundary.toml` — only `sc-runtime-core` and `sc-runtime-cli`; explicitly excludes `sc-runtime-daemon` (NF-02)
- `boundaries/sc-runtime-db/Boundary.toml` — stub, no crate code yet; reserves the trait-only rule
- `boundaries/sc-runtime-daemon/Boundary.toml` — stub for Phase C
- `boundaries/sc-runtime-web/Boundary.toml` — stub for Phase D

**`.github/workflows/ci.yml`**
Written in Sprint A-1:
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `sc-lint lint ci`

---

## Implementation Sequence

### Sprint A-1: Workspace Scaffold

**Goal:** Workspace compiles. All stub crates present. sc-lint boundary gates active. No implementation logic yet.

Tasks:
- Create `Cargo.toml` workspace root with all members, lints, and resolver configuration
- Create `rust-toolchain.toml`, `deny.toml`, `.cargo/config.toml`
- Scaffold all 10 crate directories with minimal `lib.rs` files that re-export nothing but compile
- Write all boundary TOML files in `boundaries/`
- Write `Justfile` with all targets
- Write `.github/workflows/ci.yml` skeleton
- Run `sc-lint lint fast` — all boundary and portability gates must pass on empty stubs
- Commit: "feat: workspace scaffold with sc-lint gates active"

Deliverable: `cargo check --workspace` passes; `sc-lint lint fast` passes; CI is green on empty stubs.

### Sprint A-2: sc-runtime-core

**Goal:** Core types fully implemented and tested. This is the foundation every other crate depends on.

Tasks:
- Implement `Plugin` trait: `metadata()`, `init()`, `run()`, `shutdown()` — object-safe via `BoxFuture` return types (RBP-008)
- Implement `PluginMetadata { name: &'static str, version: &'static str }`
- Implement `PluginContext { logger: Arc<Logger<Running>>, cancel: CancellationToken, home: ScRuntimeHome }` — `Logger<Running>` injected, never constructed inside `sc-runtime-core`
- Implement `ScRuntimeHome(PathBuf)`: resolves `SC_RUNTIME_HOME` env var, falls back to OS home dir via `dirs` crate; implements `Deref<Target = Path>` (RBP-005)
- Implement `ScRuntimeError { code: ErrorCode, message: &'static str, cause: Option<Box<dyn Error + Send + Sync>>, remediation: Option<Remediation> }` — RBP-001 compliant; no opaque String errors
- Implement `ErrorCode` and `Remediation` newtypes (RBP-004)
- Re-export `CancellationToken` from `tokio-util` so consumers import it from `sc-runtime-core`
- Unit tests: `ScRuntimeHome` resolves `SC_RUNTIME_HOME` override; `ScRuntimeError` carries cause and remediation; `Plugin` trait objects compile via `Box<dyn Plugin>`
- Verify `sc-lint lint fast` still passes

Deliverable: `sc-runtime-core` fully implemented, all tests pass, sc-lint clean.

### Sprint A-3: sc-runtime-cli

**Goal:** sc-ai-cli JSON-first contract implemented as a reusable crate. All five built-in commands present. `--json` flag works. Transport adapter trait enables simulator-backed tests.

Tasks:
- Implement `CommandEnvelope<T: Serialize> { ok: bool, command: CommandId, data: Option<T>, error: Option<CliError>, diagnostics: Vec<Diagnostic> }`
- Implement `CliError { kind: CliErrorKind, code: ErrorCode, message: String, cause: Option<String>, details: Option<serde_json::Value>, suggested_action: Option<String> }` — no opaque errors (RBP-001)
- Implement `CliErrorKind` enum: `Usage | Config | Capability | BackendFailure | BackendProtocol | Internal`
- Implement `CommandId(Cow<'static, str>)` newtype with `Deref<Target = str>` and `Into<Cow<'static, str>>` (RBP-004, RBP-005, RBP-009)
- Implement 5 built-in commands (`start`, `stop`, `restart`, `status`, `health`) with their `--json` output types: `StartResult`, `StopResult`, `RestartResult`, `StatusResult`, `HealthResult`
- Wire `clap` derive on each command; all commands accept `--json` flag
- Exit codes: `0` success, `1` internal, `2` usage, `3` config, `4` capability, `5` backend failure, `6` backend protocol
- Define `CommandRouterTransport` trait (injected adapter; direct path implementation for now; IPC path added Phase C)
- Unit tests: each command serializes to `CommandEnvelope` under `--json`; `CliError` fields all populated; exit code mapping matches spec
- Contract tests: each built-in command's `--json` output round-trips through `serde_json::from_str::<CommandEnvelope<T>>`

Deliverable: sc-runtime-cli fully implemented. All built-in commands work. sc-lint clean.

### Sprint A-4: sc-runtime-mcp-stdio

**Goal:** Stdio MCP transport reads JSON-RPC 2.0 from stdin and dispatches to the same operation layer as the CLI. Same fixtures pass both paths.

Tasks:
- Implement stdin reader: reads lines from stdin, parses each as `serde_json::Value`, validates JSON-RPC 2.0 envelope (`jsonrpc: "2.0"`, `method`, optional `id`, optional `params`)
- Implement operation dispatch: maps MCP `method` to the equivalent `CommandId` and dispatches through the same operation layer as `sc-runtime-cli`
- Implement stdout writer: serializes JSON-RPC 2.0 response and writes one line to stdout; flushes after each write
- MCP error mapping: `CliError` → JSON-RPC error object; error codes from the MCP spec
- Process lifetime: MCP server runs until stdin closes; exits cleanly on EOF
- Fixture-parity tests: define 5 JSON fixture pairs (request + expected response) and run them against both the CLI `--json` path and the stdio MCP path, asserting byte-identical output on the data and error fields (FR-02)
- Verify `sc-lint-boundary` still passes: `sc-runtime-mcp-stdio` must not have a path to `sc-runtime-daemon` in its dependency graph (NF-02)

Deliverable: stdio MCP server works end-to-end. Fixture-parity tests pass. Boundary clean.

### Sprint A-5: sc-runtime Facade (Phase A Slice) + Integration Test

**Goal:** Consumer-facing `sc-runtime` crate exposes the Phase A builder slice. Integration test proves the full consumer pattern works.

Tasks:
- Implement `ScRuntimeBuilder<NoCli>` with `.name()` and typestate-transitioning `.cli()→ScRuntimeBuilder<HasCli>` (RBP-002, RBP-010)
- Implement `ScRuntimeBuilder<HasCli>` with `.mcp_stdio()`, `.logger()`, `.build()→ScRuntime`
- Implement `ScRuntime::run() -> Result<(), ScRuntimeError>`: runs CLI dispatch or stdio MCP dispatch depending on builder configuration; never panics (FR-14, FR-16)
- Stub out deferred methods with compile-error messages: attempting to call `.daemon()`, `.db()`, `.web()`, `.mcp_http()` on the Phase A builder produces a clear diagnostic
- Implement `sc-runtime-example` crate in the workspace: binary that uses `ScRuntimeBuilder<HasCli>`, registers a custom `ping` command, and exercises both the `--json` path and stdio MCP path with identical fixtures
- Integration test in `sc-runtime-example`: spawns the binary with `--json` flag and with a simulated MCP stdin, asserts both produce the same `CommandEnvelope` output
- Verify `cargo test --workspace` passes, `sc-lint lint fast` passes, `sc-runtime-example` binary compiles and tests pass

Deliverable: Phase A complete. Tag `v0.1.0-alpha.1`.

---

## Exit Criteria

All of the following must be true before Phase B begins:

- `cargo test --workspace` passes with no failures and no ignored tests that cover Phase A scope
- `sc-lint lint fast` passes: boundary, portability (PORT-001 through PORT-010), and runtime (SCB-RUNTIME-001, SCB-RUNTIME-002) gates all green
- `sc-lint check xwin` is not required in this phase; however, any `#[cfg(unix)]` in Phase A code must have a Windows companion or portable fallback per PORT-010 — this should be zero occurrences in Phase A since daemon transport is not yet present
- Integration test: the `sc-runtime-example` binary processes a `CommandEnvelope` via `--json` and via stdio MCP input, producing identical `data` and `error` fields in both outputs
- All five built-in commands (`start`, `stop`, `restart`, `status`, `health`) serialize correctly under `--json` and are dispatched correctly by the stdio MCP server
- `sc-runtime-mcp-stdio` boundary check passes: no path to `sc-runtime-daemon` in the dependency graph
- `ScRuntime::run()` returns `Result<(), ScRuntimeError>` and does not call `unwrap()`, `expect()`, or `panic!()` in any reachable code path
- Git tag `v0.1.0-alpha.1` applied to the passing commit
