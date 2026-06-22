# sc-runtime

**Product Requirements Document**
Version 1.0 · June 2026 · Synaptic Canvas

---

## 1. Overview

`sc-runtime` is a composable Rust infrastructure workspace that provides the standard building blocks for every Synaptic Canvas tool. Tools assemble exactly the layers they need — no layer forces another as a dependency.

The five independent capability layers are:

| Layer | Crate(s) | Requires |
|-------|----------|----------|
| CLI + JSON contract | `sc-runtime-cli` | always |
| stdio MCP server | `sc-runtime-mcp-stdio` | CLI |
| Storage | `sc-runtime-db-*` | nothing |
| Daemon + RPC | `sc-runtime-daemon` | CLI |
| HTTP / streaming MCP | `sc-runtime-web` | Daemon |

Every layer shares the same request/response types. CLI, RPC, and HTTP are transport variations on a single contract — no reshaping at any boundary.

---

## 2. Problem Statement

Every SC tool (atm-core, Continuity, ci, and all future tools) rebuilds the same infrastructure from scratch: command parsing, structured logging, storage, daemon lifecycle, and IPC. The result is:

- inconsistent CLI contracts that are not MCP-ready
- incompatible storage patterns across tools
- duplicated daemon implementations with different signal and PID behaviors
- no shared portability enforcement, leading to platform-specific assumptions
- agents lack consistent patterns to follow across the tool family

---

## 3. Goals

- Single dependency for any combination of: CLI, MCP (stdio or HTTP), storage, daemon, web server
- sc-ai-cli JSON-first contract as the foundation — machine contract is primary, human output is secondary
- MCP-ready seam from day one: same request/response types for CLI, daemon RPC, and HTTP — no reshaping
- Injectable sc-observability structured logging — app constructs and injects; sc-runtime never owns the logger
- StorageBackend trait abstracts over rusqlite, FrankenSQLite, and sqlx/postgres transparently from consumers
- Cross-platform: macOS, Linux, Windows — enforced by sc-lint portability gates
- sc-lint boundary, portability, runtime, and xwin gates in CI from Sprint 1
- No runtime panics in normal operation — illegal state transitions are compile errors (typestate)
- All rust-best-practices (RBP-001 through RBP-010) applied at the design stage

---

## 4. Non-Goals

- Not a framework — tools own their domain logic entirely
- No ATM-specific concepts — sc-runtime has no knowledge of agents, messages, or inbox/outbox
- No opinionated HTTP framework lock-in beyond the MVP axum implementation (consumer can replace)
- No built-in auth, rate limiting, or multi-tenancy — those belong in the application layer
- No FrankenSQLite native mode until it ships a stable public toggle
- No TCL, Python, or non-Rust runtime requirements

---

## 5. Architecture

### 5.1 Ecosystem Position

```
sc-runtime  (infrastructure: CLI, MCP, storage, daemon, web)
├── atm-core        (domain: agent messaging)
├── Continuity      (domain: CI/PR monitoring)
├── ci              (domain: git/gh policy enforcement)
└── [future tools]
```

sc-runtime has zero dependencies on any SC domain crate. Dependency flows strictly downward.

### 5.2 Composability Model

All five capability axes are independent. A tool takes exactly what it needs:

```
MCP axis:     none │ stdio │ http/streaming  (http requires Daemon)
Storage axis: none │ sqlite │ frankensqlite │ sqlx
Daemon axis:  no   │ yes
Web axis:     no   │ yes   (foreground without Daemon; plugin-supervised with Daemon)
CLI axis:     always
```

**Composability matrix** (representative valid configurations × storage axis):

| Configuration | CLI | stdio MCP | Daemon | Web | http MCP | Storage |
|---------------|:---:|:---------:|:------:|:---:|:--------:|:-------:|
| Minimal CLI | ✓ | | | | | any |
| CLI + stdio MCP | ✓ | ✓ | | | | any |
| CLI + web (foreground) | ✓ | | | ✓ | | any |
| CLI + stdio MCP + web | ✓ | ✓ | | ✓ | | any |
| CLI + daemon | ✓ | | ✓ | | | any |
| CLI + daemon + http MCP | ✓ | | ✓ | | ✓ | any |
| CLI + daemon + web | ✓ | | ✓ | ✓ | | any |
| CLI + daemon + web + http MCP | ✓ | | ✓ | ✓ | ✓ | any |

Every row is valid with any storage column (none, sqlite, frankensqlite, sqlx) or any combination thereof.

### 5.3 Layer Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Consumers                                │
│  CLI binary   stdio MCP   HTTP client   daemon plugin        │
└────────────────────┬─────────────────────────────────────────┘
                     │
┌────────────────────▼─────────────────────────────────────────┐
│              sc-runtime (facade + typestate builder)          │
└───┬──────────┬──────────┬──────────┬──────────┬─────────────┘
    │          │          │          │          │
┌───▼──┐  ┌───▼──┐  ┌────▼───┐  ┌───▼──┐  ┌───▼──────┐
│ cli  │  │ mcp  │  │daemon  │  │ web  │  │  db/*    │
│      │  │stdio │  │        │  │      │  │          │
└───┬──┘  └──────┘  └───┬────┘  └──────┘  └──────────┘
    │                   │
    └──────────┬────────┘
               │
┌──────────────▼───────────────────────────────────────────────┐
│                     sc-runtime-core                           │
│   Plugin trait · types · observability wiring · errors        │
└──────────────────────────────────────────────────────────────┘
```

### 5.4 Crate Structure

```
sc-runtime/
├── Cargo.toml                   # workspace root, resolver = "2"
├── rust-toolchain.toml          # edition 2024, rust-version = 1.94.1
├── deny.toml                    # cargo-deny: licenses, duplicates, advisories
├── Justfile                     # lint, test, ci, xwin profiles
├── boundaries/                  # sc-lint boundary TOML definitions
│   ├── planning.toml
│   ├── sc-runtime-core/
│   ├── sc-runtime-cli/
│   ├── sc-runtime-db/
│   ├── sc-runtime-daemon/
│   └── sc-runtime-web/
├── crates/
│   ├── sc-runtime-core/         # Plugin trait, types, observability wiring
│   ├── sc-runtime-cli/          # sc-ai-cli framework: clap, JSON contract, router
│   ├── sc-runtime-mcp-stdio/    # stdio MCP transport (no daemon dependency)
│   ├── sc-runtime-db/           # StorageBackend trait only — no implementation
│   ├── sc-runtime-db-sqlite/    # rusqlite implementation
│   ├── sc-runtime-db-fsqlite/   # FrankenSQLite implementation (future)
│   ├── sc-runtime-db-sqlx/      # sqlx implementation (sqlite + postgres)
│   ├── sc-runtime-daemon/       # daemon lifecycle, PID, signals, RPC
│   ├── sc-runtime-web/          # axum HTTP + streaming MCP (MVP)
│   └── sc-runtime/              # facade + typestate ScRuntime builder
└── docs/
    └── adr/                     # Architecture Decision Records
```

---

## 6. Layer Specifications

### 6.1 sc-runtime-core

**Role:** Foundation. All other crates depend on this; it depends on nothing in the workspace.

**Exports:**

```rust
// Plugin lifecycle — object-safe for dynamic dispatch
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
    async fn run(&mut self, cancel: CancellationToken) -> Result<(), PluginError>;
    async fn shutdown(&mut self) -> Result<(), PluginError>;
}

pub struct PluginMetadata {
    pub name: &'static str,
    pub version: &'static str,
}

// Capability context injected into every plugin
pub struct PluginContext {
    pub logger: Arc<Logger<Running>>,  // sc-observability — injected, not constructed here
    pub cancel: CancellationToken,
    pub home: ScRuntimeHome,           // SC_RUNTIME_HOME resolution
}

// Home directory resolution (mirrors ATM_HOME pattern)
pub struct ScRuntimeHome(PathBuf);
// env override: SC_RUNTIME_HOME → defaults to OS home dir
// test isolation: set SC_RUNTIME_HOME to a temp dir

// Structured error type for all sc-runtime failure paths
// RBP-001: error context + recovery — every variant carries stable code, cause, and remediation
pub struct ScRuntimeError {
    pub code: ErrorCode,
    pub message: &'static str,
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub remediation: Option<Remediation>,
}
```

**Dependencies:** `sc-observability`, `tokio-util` (CancellationToken), `thiserror`.

**Boundary rule:** sc-runtime-core must have zero SC domain crate dependencies. Enforced by `sc-lint-boundary` boundary definition and CI gate.

---

### 6.2 sc-runtime-cli

**Role:** sc-ai-cli framework applied as a reusable infrastructure layer. Machine contract is primary; human output is secondary.

**sc-ai-cli compliance** (per `synaptic-canvas/packages/sc-ai-cli/skills/creating-ai-clis`):

- Every command exposes `--json` producing a `CommandEnvelope<T>`
- Request and response structs derive `Serialize`/`Deserialize` — same types consumed by MCP and daemon RPC
- Error contract uses `CliError` with stable `kind`, `code`, `message`, `cause`, `details`, and `suggested_action`
- Command identity is a stable dotted identifier (e.g., `runtime.status`, `runtime.health`)
- Human output is a rendering of the JSON result — never richer than `--json`
- Exit codes are CLI-owned: `0` success, `1` internal, `2` usage, `3` config, `4` capability, `5` backend failure, `6` backend protocol

**Envelope types:**

```rust
// Stable top-level envelope — identical shape across all command families
pub struct CommandEnvelope<T: Serialize> {
    pub ok: bool,
    pub command: CommandId,   // stable dotted identifier: "runtime.status"
    pub data: Option<T>,
    pub error: Option<CliError>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct CliError {
    pub kind: CliErrorKind,   // usage | config | capability | backend_failure | backend_protocol | internal
    pub code: ErrorCode,      // stable string: "CLI.USAGE_ERROR"
    pub message: String,
    pub cause: Option<String>,
    pub details: Option<serde_json::Value>,
    pub suggested_action: Option<String>,
}
```

**Built-in commands** (sc-runtime-cli provides; consumers extend):

| Command | `--json` contract | Description |
|---------|-------------------|-------------|
| `start` | `CommandEnvelope<StartResult>` | Start daemon or run directly |
| `stop` | `CommandEnvelope<StopResult>` | Send SIGTERM to daemon |
| `restart` | `CommandEnvelope<RestartResult>` | Stop + start |
| `status` | `CommandEnvelope<StatusResult>` | Daemon state, PID, uptime |
| `health` | `CommandEnvelope<HealthResult>` | Plugin health checks |

**Command router:** if daemon socket/pipe is reachable → serialize request and forward over IPC; otherwise execute directly. Transparent to the caller. The router is a compile-time capability — when the daemon layer is not included, the router always executes directly.

**Simulator-backed testing:** per sc-ai-cli `designing-cli-simulators`, external integrations must be testable via a stateful simulator implementing the same adapter trait. The router's transport is an injected trait, enabling simulator-backed integration tests without a live daemon.

---

### 6.3 sc-runtime-mcp-stdio

**Role:** MCP server over stdin/stdout. No daemon, no socket, no background process.

- Reads JSON-RPC 2.0 from stdin line by line
- Dispatches to the same operation layer as the CLI (same request/response types, zero reshaping)
- Writes JSON-RPC 2.0 responses to stdout
- Runs in the foreground; process lifetime is MCP client lifetime
- No `sc-runtime-daemon` dependency — this is a hard boundary enforced by `sc-lint-boundary`

**MCP contract parity:** the same JSON fixtures used in CLI `--json` tests must pass against the stdio MCP path. Per sc-ai-cli skill: "test the same JSON fixtures against the CLI path and the MCP path with no contract reshaping between them."

---

### 6.4 sc-runtime-db

**Role:** StorageBackend trait definition only. Zero implementations. Any crate that needs to accept storage accepts `Arc<dyn StorageBackend>` without taking a dependency on any specific backend crate.

```rust
pub trait StorageBackend: Send + Sync {
    fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<u64, StorageError>;
    fn query(&self, sql: &str, params: &[SqlParam]) -> Result<Rows, StorageError>;
    fn transaction<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&dyn StorageBackend) -> Result<R, StorageError>;
    fn migrate(&self, migrations: &[Migration]) -> Result<(), StorageError>;
    fn health(&self) -> StorageHealth;
}

pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub up: &'static str,    // SQL DDL — standard subset compatible with all backends
    pub down: &'static str,
}
```

**RBP-003 (Sealed Trait):** `StorageBackend` is sealed within the sc-runtime workspace. External crates may hold `Arc<dyn StorageBackend>` but may not implement the trait. Backend implementations are sc-runtime-workspace-owned. This preserves the ability to evolve the trait without a breaking change surface.

**Test isolation:** every backend reads `SC_RUNTIME_DB` environment variable to override the database path/URL, enabling parallel test isolation without config changes.

---

### 6.5 sc-runtime-db-sqlite

**Role:** rusqlite implementation of `StorageBackend`.

- Opens SQLite with WAL journal mode on every connection
- Connection pool via `r2d2-sqlite`
- Runs schema migrations on first open; idempotent on re-run
- `SC_RUNTIME_DB` overrides the database file path
- Bundled SQLite (`rusqlite` bundled feature) — zero system dependency

```rust
pub struct SqliteBackend { /* ... */ }

impl SqliteBackend {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError>;
    pub fn in_memory() -> Result<Self, StorageError>;  // for tests
}
```

---

### 6.6 sc-runtime-db-fsqlite (future)

**Role:** FrankenSQLite implementation of `StorageBackend`.

Held until FrankenSQLite's native MVCC mode ships a stable `Connection` toggle. In compatibility mode FrankenSQLite is rusqlite-over-C-SQLite and provides no additional value over `sc-runtime-db-sqlite`. When native mode stabilizes, the 41× concurrent write throughput at 8 threads becomes meaningful for multi-plugin daemons sharing a single database.

The `StorageBackend` trait is designed so this backend can be added with zero consumer-facing API change.

---

### 6.7 sc-runtime-db-sqlx

**Role:** sqlx implementation of `StorageBackend` supporting SQLite and PostgreSQL via connection URL.

- Backend selected from URL scheme: `sqlite://path`, `postgres://host/db`
- Uses `sqlx::AnyPool` for runtime backend selection
- Migration runner uses `sqlx::migrate!`
- `SC_RUNTIME_DB` overrides the connection URL

This backend is the path to PostgreSQL. Consumers swap `SqliteBackend::open(path)` for `SqlxBackend::from_url("postgres://...")` in the builder — zero other changes.

---

### 6.8 sc-runtime-daemon

**Role:** Plugin registry, daemon singleton lifecycle, cross-platform RPC.

**Daemon lifecycle:**

```
start → write PID file → init all plugins → run all plugins concurrently
SIGTERM → propagate CancellationToken → await plugin shutdown() in order → delete PID file → exit
SIGUSR1 → send wake notification to all plugins via channel
```

**PID file singleton:** on start, check if PID file exists and process is alive. If alive, exit with `DAEMON.ALREADY_RUNNING`. If stale (process dead), replace. Liveness check is cross-platform (from sc-runtime-core `pid.rs`).

**Signal handling:**
- Unix: `SIGTERM` → graceful shutdown, `SIGUSR1` → wake
- Windows: `CTRL_C_EVENT` → graceful shutdown; no SIGUSR1 equivalent (no-op or named event)
- All signal handling gated with `#[cfg(unix)]` / `#[cfg(windows)]` companions — **PORT-010 compliant**

**RPC transport (identical protocol on all transports):**

| Platform | Transport | Path |
|----------|-----------|------|
| macOS / Linux | Unix domain socket | `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.sock` |
| Windows | Named pipe | `\\.\pipe\sc-{tool}` |
| Fallback | TCP loopback | port registered in `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.port` |

Wire format: newline-delimited JSON. Same `CommandEnvelope<T>` and `CliError` types as the CLI — no transport-specific DTOs. One request line per connection, one response line.

```json
// Request
{"version":1,"request_id":"uuid","command":"runtime.health","payload":{}}

// Response
{"version":1,"request_id":"uuid","ok":true,"command":"runtime.health","data":{"state":"running","plugins":[...]}}
```

**Plugin context enrichment:** `PluginContext` delivered to each plugin carries an `Arc<dyn StorageBackend>` if storage is configured, plus the sc-observability logger and cancellation token.

**Structured concurrency:** all plugin tasks run as region-owned tokio tasks under a `JoinSet`. No detached tasks. Cancellation is propagated via `CancellationToken` from `tokio-util`. Plugin panics are caught and converted to `PluginError::Panic` — they do not propagate to the runtime.

---

### 6.9 sc-runtime-web

**Role:** HTTP transport layer. MVP uses axum. The HTTP framework is not exposed in any public type — consumers wire routes, axum is a private implementation detail.

**Without daemon:** web server runs in the foreground. Process lifetime = server lifetime. Suitable for tools that are HTTP-first and managed externally (systemd, Docker, etc.).

**With daemon:** the web server runs as a `Plugin` inside the daemon's supervised region. It starts with the daemon, shuts down gracefully on SIGTERM.

**HTTP MCP (streaming):** Server-Sent Events (SSE) endpoint over HTTP. Same JSON-RPC 2.0 message format as stdio MCP. Requires daemon because SSE clients maintain persistent connections and the server must be running independently of the CLI.

**Route contract:** HTTP handlers are thin — they deserialize the request body into the same `CommandEnvelope` request type used by the CLI, call the operation layer, and serialize the response. No HTTP-specific DTOs. This is the MCP-compatible seam applied to HTTP transport.

```rust
// Consumer registers routes against sc-runtime's operation registry
// sc-runtime-web owns the axum server; the consumer owns the operations
pub trait HttpRouteRegistry: Send + Sync {
    fn register(&self, router: axum::Router) -> axum::Router;
}
```

---

### 6.10 sc-runtime (facade + typestate builder)

**Role:** Single entry point. Typestate builder prevents illegal configuration at compile time.

```rust
// Typestate marker types
pub struct NoCli;
pub struct HasCli;
pub struct HasDaemon;  // implies HasCli

// Builder
pub struct ScRuntimeBuilder<State> { /* ... */ _state: PhantomData<State> }

impl ScRuntimeBuilder<NoCli> {
    pub fn new() -> Self;
    pub fn name(self, name: &'static str) -> Self;
}

impl ScRuntimeBuilder<NoCli> {
    // CLI is required — calling .cli() transitions state
    pub fn cli(self, register: impl FnOnce(&mut CommandRegistry)) -> ScRuntimeBuilder<HasCli>;
}

impl ScRuntimeBuilder<HasCli> {
    pub fn mcp_stdio(self) -> Self;                          // no daemon needed
    pub fn db(self, backend: Arc<dyn StorageBackend>) -> Self;
    pub fn web(self, config: WebConfig) -> Self;             // foreground HTTP
    pub fn logger(self, logger: Logger<Running>) -> Self;    // inject sc-observability logger
    pub fn plugin(self, plugin: impl Plugin + 'static) -> Self;

    // Transitions to HasDaemon — only path to http MCP and supervised web
    pub fn daemon(self) -> ScRuntimeBuilder<HasDaemon>;

    pub fn build(self) -> ScRuntime;  // foreground: no daemon
}

impl ScRuntimeBuilder<HasDaemon> {
    pub fn db(self, backend: Arc<dyn StorageBackend>) -> Self;
    pub fn web(self, config: WebConfig) -> Self;             // supervised by daemon
    pub fn mcp_http(self, addr: SocketAddr) -> Self;         // ONLY available with daemon
    pub fn logger(self, logger: Logger<Running>) -> Self;
    pub fn plugin(self, plugin: impl Plugin + 'static) -> Self;
    pub fn build(self) -> ScRuntime;
}
```

**No runtime panics:** the typestate builder makes `mcp_http` without `daemon` a compile error. Every other invalid configuration is similarly encoded. `ScRuntime::run()` returns `Result<(), ScRuntimeError>` — it never panics.

**Consumer patterns:**

```rust
// Pattern 1: Minimal CLI
ScRuntime::builder()
    .name("my-tool")
    .cli(commands::register)
    .logger(logger)
    .build()
    .run()?;

// Pattern 2: CLI + stdio MCP + sqlite
ScRuntime::builder()
    .name("my-tool")
    .cli(commands::register)
    .mcp_stdio()
    .db(Arc::new(SqliteBackend::open("my-tool.db")?))
    .logger(logger)
    .build()
    .run()?;

// Pattern 3: Full stack — daemon + HTTP + postgres + http MCP
ScRuntime::builder()
    .name("my-tool")
    .cli(commands::register)
    .db(Arc::new(SqlxBackend::from_url("postgres://localhost/my-tool")?))
    .logger(logger)
    .daemon()
    .plugin(MyDomainPlugin::new())
    .web(WebConfig::default())
    .mcp_http("127.0.0.1:3000".parse()?)
    .build()
    .run()?;
```

---

## 7. sc-observability Integration

sc-observability (`sc-observability = "1.2.0"`) is the only allowed structured logging dependency. Direct use of `tracing`, `log`, or `println!` for structured output is forbidden in sc-runtime crates.

**Injection model:** sc-runtime never constructs a `Logger<Running>`. The application constructs it via `LoggerBuilder` and injects it via `.logger(logger)` on the builder. This is a hard requirement — tools must be able to configure their own log root, service name, sinks, and retention policy.

```rust
// Application code — not sc-runtime internals
let logger = LoggerBuilder::new(LoggerConfig::default_for(
    ServiceName::new("my-tool")?,
    PathBuf::from("/var/log/sc/my-tool"),
))?
.build();  // Logger<Running>

ScRuntime::builder()
    .name("my-tool")
    .logger(logger)        // injected
    // ...
```

**PluginContext carries the logger as `Arc<Logger<Running>>`:** plugins log through the injected logger; they do not construct their own.

**Log events from sc-runtime** use stable `ActionName` values so consumers can filter sc-runtime infrastructure events from domain events. Prefix convention: `sc_runtime.*`.

**ADR-008 compliance** (from sc-lint): sc-observability logging boundary policy applies — `sc-runtime-core` owns the logger reference; backend crates receive it through context injection, not through a global.

---

## 8. sc-lint Integration

sc-runtime is a primary consumer and proving ground for the sc-lint linter suite. All linters run in CI from Sprint 1.

### Installation

```bash
brew install randlee/tap/sc-lint   # macOS/Linux via Homebrew
```

### Lint Profiles

| Profile | When | Includes xwin |
|---------|------|:-------------:|
| `fast` | local dev (pre-commit) | no |
| `full` | local pre-push | yes (if installed) |
| `ci` | CI lint gate | no |
| top-level `ci` | CI lint + tests | no |

### Linter Coverage

**`sc-lint-boundary`** — enforces the sc-runtime layered dependency rules via TOML boundary definitions in `boundaries/`. Key rules for sc-runtime:
- `sc-runtime-core` has zero SC domain crate dependencies
- `sc-runtime-mcp-stdio` has no dependency on `sc-runtime-daemon`
- `sc-runtime-db` (trait crate) has no dependency on any db implementation crate
- `sc-runtime-web` depends on `sc-runtime-daemon` only when daemon feature is active

**`sc-lint-portability`** — enforces cross-platform code. Rules directly relevant to sc-runtime:
- `PORT-001`–`PORT-005`: path, separator, and env literal checks
- `PORT-008`: flags `HOME`, `USER`, `XDG_*` lookups ungated — use `ScRuntimeHome` instead
- `PORT-009`: flags `Command::new("sh")` / `Command::new("bash")` — no shell assumptions
- `PORT-010`: every `#[cfg(unix)]` branch must have a Windows companion or portable fallback — enforces the Unix socket / named pipe transport pair

**`sc-lint-runtime`** — std runtime/concurrency rules:
- `SCB-RUNTIME-001`, `SCB-RUNTIME-002`: std concurrency correctness — applies to plugin registry, PID file, and RPC dispatch

**`sc-lint-tokio`** *(reserved, no implementation yet)* — when Tokio-specific rules ship, sc-runtime is a primary target: plugin task supervision, cancellation propagation, and `spawn_blocking` usage in storage backends.

**`sc-lint-version`** *(Phase C)* — interface versioning via `cargo-semver-checks`. sc-runtime's public API is the highest-value target: every consumer breaks on a semver violation.

**xwin preflight (`sc-lint check xwin` / `sc-lint clippy xwin`)** — runs `cargo xwin check` and `cargo xwin clippy` to verify Windows cross-compilation from macOS/Linux. Included in the `full` profile. Catches platform-specific compilation failures before Windows CI. Critical for the named-pipe transport and any `#[cfg(windows)]` code.

**`sc-lint-attributes`** — `#[sc_lint(allow = "...")]` used to document intentional exceptions with justification. Example: platform-specific RPC transport initialization annotated with `#[sc_lint(allow = "PORT-010", reason = "paired unix/windows impls in adjacent cfg blocks")]`.

### Justfile Targets

```
just lint        → sc-lint lint full
just lint fast   → sc-lint lint fast
just lint ci     → sc-lint lint ci
just ci          → sc-lint ci
```

---

## 9. Rust Best Practices Compliance

All ten practices from `rust-best-practices` (RBP-001 through RBP-010) are applied at design stage:

| Id | Practice | Application in sc-runtime |
|----|----------|---------------------------|
| `RBP-001` | Error Context + Recovery | `ScRuntimeError` carries stable code, cause, and `Remediation` on every variant. `CliError` carries `suggested_action`. No opaque `String` errors anywhere. |
| `RBP-002` | Typestate | `ScRuntimeBuilder<State>` — `mcp_http` and supervised `web` only available on `HasDaemon`. `Logger<Running>` typestate from sc-observability used directly. |
| `RBP-003` | Sealed Trait | `StorageBackend` is sealed — workspace-internal implementations only. `Plugin` is intentionally open (consumer extension point). `CliError.kind` variants are an exhaustive enum, not an open trait. |
| `RBP-004` | Newtype / Zero-Cost | `ScRuntimeHome`, `CommandId`, `PluginName`, `SocketPath`, `MigrationVersion` — all newtypes over primitives. No bare `String` for semantic identifiers. |
| `RBP-005` | Deref Coercion | `ScRuntimeHome` implements `Deref<Target = Path>`. `CommandId` implements `Deref<Target = str>`. |
| `RBP-006` | Interior Mutability | `Arc<Mutex<...>>` in plugin registry justified by multi-task concurrent access. All `Mutex` usages documented with their invariant. `RwLock` preferred over `Mutex` for read-heavy state. |
| `RBP-007` | Infallible | `PluginMetadata` accessors are infallible — return `&'static str`, no `Result`. |
| `RBP-008` | Trait Object Safety | `Plugin` trait is object-safe by design — async methods use `BoxFuture` return types. `StorageBackend` is object-safe. Verified at design time, not discovered at compile time. |
| `RBP-009` | `Cow` | `CommandId::new` accepts `impl Into<Cow<'static, str>>` — avoids allocation when using `'static` literals (the common case). |
| `RBP-010` | `PhantomData` / Capability Token | `ScRuntimeBuilder<State>` uses `PhantomData<State>`. `Logger<Running>` typestate consumed from sc-observability. `PluginContext` uses a capability-token pattern to prevent plugins from re-initializing the runtime. |

---

## 10. Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1 (full) | Async runtime |
| `tokio-util` | 0.7 | `CancellationToken` |
| `sc-observability` | 1.2.0 | Structured JSONL logging |
| `sc-observability-types` | 1.2.0 | Log types shared across crates |
| `clap` | 4 (derive) | CLI argument parsing |
| `serde` / `serde_json` | 1 | Request/response serialization |
| `thiserror` | 2 | Error type derivation |
| `uuid` | 1 | RPC `request_id` generation |
| `rusqlite` | 0.31 (bundled) | SQLite — `sc-runtime-db-sqlite` only |
| `r2d2-sqlite` | latest | Connection pool — `sc-runtime-db-sqlite` only |
| `sqlx` | 0.8 | sqlx backend — `sc-runtime-db-sqlx` only |
| `axum` | 0.8 | HTTP server — `sc-runtime-web` only |
| `libc` | 0.2 | POSIX signals, pid liveness (unix) |
| `sc-lint-attributes` | 0.3 | `#[sc_lint(...)]` annotations (dev) |

---

## 11. Functional Requirements

| ID | Requirement |
|----|-------------|
| FR-01 | Every command exposes `--json` producing a stable `CommandEnvelope<T>`. Human output is a rendering of the same data. |
| FR-02 | Request and response types are identical across CLI, daemon RPC, and HTTP transport. No reshaping at any transport boundary. |
| FR-03 | stdio MCP server operates without a daemon. Process starts, serves JSON-RPC over stdin/stdout, exits. |
| FR-04 | HTTP MCP and supervised web require daemon. `mcp_http()` on a non-daemon builder is a compile error. |
| FR-05 | `StorageBackend` is a sealed trait. All implementations live in sc-runtime workspace crates. Consumer selects backend at construction time. |
| FR-06 | `SC_RUNTIME_DB` environment variable overrides the database path/URL on all storage backends for test isolation. |
| FR-07 | Daemon enforces singleton via PID file. Second instance detects running daemon and exits with `DAEMON.ALREADY_RUNNING`. Stale PID files (dead process) are reclaimed automatically. |
| FR-08 | SIGTERM triggers graceful shutdown: `CancellationToken` propagated to all plugins; `shutdown()` called in reverse-init order. |
| FR-09 | SIGUSR1 sends wake notification to all plugins without terminating the daemon. Windows: equivalent named event. |
| FR-10 | RPC protocol is identical on all transports (Unix socket, named pipe, TCP loopback): newline-delimited JSON, version field, `request_id` for correlation. |
| FR-11 | CLI command router: if daemon socket/pipe is reachable → route via IPC; otherwise execute directly. Transparent to caller. |
| FR-12 | `PluginContext` provides: injected `Logger<Running>`, `Arc<dyn StorageBackend>` (if configured), `CancellationToken`. |
| FR-13 | All sc-lint gates (boundary, portability, runtime, xwin) pass in CI from Sprint 1. |
| FR-14 | sc-runtime never panics in normal operation. All error paths return `Result`. |
| FR-15 | sc-observability logger is injected by the consumer — sc-runtime never constructs a logger internally. |
| FR-16 | `ScRuntime::run()` returns `Result<(), ScRuntimeError>` — the entry point is fallible, not `!`. |

---

## 12. Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| NF-01 | `sc-runtime-core` has zero SC domain crate dependencies. Enforced by `sc-lint-boundary` CI gate. |
| NF-02 | `sc-runtime-mcp-stdio` has no dependency on `sc-runtime-daemon`. Enforced by boundary definition. |
| NF-03 | Cross-platform: macOS, Linux, Windows. Every `#[cfg(unix)]` block has a Windows companion or portable fallback (PORT-010 compliant). |
| NF-04 | rusqlite bundled feature in `sc-runtime-db-sqlite`. Zero system SQLite dependency. |
| NF-05 | `unsafe_code = "forbid"` as a workspace lint. No unsafe code without a documented, justified override. |
| NF-06 | Pedantic + nursery Clippy lints at deny level (matches sc-lint workspace policy). |
| NF-07 | `cargo deny` checks for license compatibility, duplicate dependencies, and security advisories. |
| NF-08 | Edition 2024, rust-version = 1.94.1 (matches sc-lint and sc-observability). |
| NF-09 | Starting version 0.1.0 across all workspace crates. |
| NF-10 | sc-lint `full` profile (including xwin) must pass before any release tag. |

---

## 13. Architecture Decision Records

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-01 | sc-ai-cli JSON-first contract as the CLI foundation | Machine contract is the primary product surface; MCP-ready seam from day one eliminates future reshaping work. |
| ADR-02 | Typestate builder — no runtime panics | Illegal configurations (http MCP without daemon) are compile errors. Consistent with RBP-002 and the NO runtime panic policy. |
| ADR-03 | StorageBackend sealed trait — workspace-owned impls | Preserves ability to evolve the trait without a public semver break. Consumers hold `Arc<dyn StorageBackend>`, not a concrete type. |
| ADR-04 | Five separate db crates (trait + one per impl) | Consumers take exactly one backend with no dead code. FrankenSQLite and sqlx added without touching consumer code. |
| ADR-05 | stdio MCP has no daemon dependency | A CLI + stdio MCP tool has no background process, no PID file, no socket — simpler deployment and no lifecycle management overhead. |
| ADR-06 | Identical wire format on all RPC transports | Eliminates transport-conditional business logic. Same test fixtures work against Unix socket, named pipe, and TCP loopback paths. |
| ADR-07 | sc-observability injection, not construction | Tools own their log root, service name, sinks, and retention. sc-runtime never makes opinionated decisions about where logs go. |
| ADR-08 | axum for web MVP, not locked in | axum is private implementation detail. `HttpRouteRegistry` trait abstracts route registration. Replacing axum is a single-crate change. |
| ADR-09 | FrankenSQLite deferred to native mode stability | Compatibility mode is rusqlite-over-C-SQLite — no additional value over `sc-runtime-db-sqlite`. The `StorageBackend` trait is the hook for when native mode ships. |
| ADR-10 | sc-lint from Homebrew, not cargo install | Homebrew install is the sc-lint distribution path. Using `cargo install` would require building from source and conflict with workspace toolchain constraints. |
| ADR-11 | xwin in `full` profile only, not `ci` | Real Windows CI (GitHub Actions `windows-latest`) remains authoritative for Windows correctness. xwin is a fast local pre-push signal, not a replacement. |
| ADR-12 | Standalone repo from day one | sc-runtime lives in its own repo and workspace from Sprint 1 — never inside a consumer repo. This keeps sc-lint boundary enforcement real (same-workspace members bypass cross-boundary detection), makes the semver surface explicit from the start, and avoids repo surgery later. Consumer repos co-develop via `[patch.crates-io]` pointing to the local sc-runtime clone. |
| ADR-13 | App-types crate per consumer | Each SC tool defines a `{tool}-types` crate with minimal dependencies (`sc-runtime-core`, `serde` only). This crate owns all tool-specific request/response structs, CLI command shapes, and error codes. The same types are used by the CLI binary, daemon RPC, and any MCP wrapper — zero reshaping across transports. |
| ADR-14 | sc-lint-version is highest-priority semver gate | sc-runtime is a shared infrastructure crate; a silent breaking change breaks every consumer simultaneously. `sc-lint check interfaces` (Phase C) must be a required release gate before any crates.io publish. |

---

## 14. Development Plan

Four phases. Each phase has a gate before the next begins.

| Phase | Title | Sprints | Gate |
|-------|-------|---------|------|
| A | CLI Foundation | 1–2 | CLI + stdio MCP working end-to-end. sc-lint gates pass. |
| B | Storage | 3–4 | Both storage backends (sqlite, sqlx) working. Migration runner tested. |
| C | Daemon + RPC | 5–6 | Full daemon lifecycle + RPC. CLI router end-to-end. |
| D | Web + Consumers | 7–8 | Web layer. atm-core migrated. Continuity onboarded. |

---

### Phase A — CLI Foundation

#### Sprint 1 — Workspace, Core, sc-observability Wiring

**Goals:**
- sc-runtime workspace compiles with all crate stubs
- `sc-runtime-core`: Plugin trait, PluginMetadata, PluginContext, ScRuntimeHome, ScRuntimeError
- sc-observability injection pattern defined and tested
- sc-lint boundary definitions in `boundaries/` — CI gate active from this sprint

**Tasks:**
- Create Cargo.toml workspace root, `rust-toolchain.toml` (edition 2024, 1.94.1), `deny.toml`
- Scaffold all 10 sub-crate directories with stub `lib.rs` files
- Implement `sc-runtime-core`: Plugin trait (object-safe via BoxFuture), PluginContext, ScRuntimeHome (SC_RUNTIME_HOME env override), ScRuntimeError with RBP-001 compliance
- Write boundary TOML definitions for all layers in `boundaries/`
- Wire `sc-observability` as injected dependency in PluginContext — no logger construction in sc-runtime
- Add `Justfile` with `lint`, `lint fast`, `lint ci`, `ci` targets mapping to sc-lint profiles
- Unit tests: ScRuntimeHome resolution with SC_RUNTIME_HOME override
- Verify sc-lint boundary, portability, runtime gates all pass

**Deliverable:** Workspace compiles. Core types and Plugin trait defined. sc-lint gates green.

---

#### Sprint 2 — CLI Layer + stdio MCP

**Goals:**
- `sc-runtime-cli`: clap, CommandEnvelope, CliError, built-in commands, command router stub
- `sc-runtime-mcp-stdio`: stdio JSON-RPC 2.0 transport
- Typestate builder stub (NoCli → HasCli transition only)
- End-to-end: `my-tool status --json` produces valid `CommandEnvelope<StatusResult>`

**Tasks:**
- Implement CommandEnvelope<T>, CliError, CommandId, CliErrorKind per sc-ai-cli contract
- Implement built-in commands: start, stop, restart, status, health with `--json` on all
- Define transport adapter trait for command router (enables simulator-backed testing)
- Implement `sc-runtime-mcp-stdio`: stdin JSON-RPC reader, operation dispatch, stdout writer
- Implement ScRuntimeBuilder<NoCli> → ScRuntimeBuilder<HasCli> typestate transition
- sc-runtime facade: `.cli()`, `.mcp_stdio()`, `.logger()`, `.build()`, `.run()`
- Contract tests: same JSON fixtures exercised against CLI `--json` path and stdio MCP path
- Port-010 check: no `#[cfg(unix)]` without Windows companion
- Verify sc-lint full profile passes (including xwin if available)

**Deliverable:** CLI + stdio MCP working end-to-end. sc-ai-cli contract validated. Phase A gate passed.

---

### Phase B — Storage

#### Sprint 3 — StorageBackend Trait + rusqlite

**Goals:**
- `sc-runtime-db`: sealed StorageBackend trait, Migration, SqlParam, Rows
- `sc-runtime-db-sqlite`: rusqlite + WAL + r2d2 connection pool + migration runner
- Builder: `.db(backend)` wires storage into PluginContext

**Tasks:**
- Define StorageBackend trait (object-safe, sealed per RBP-003)
- Implement `sc-runtime-db-sqlite`: WAL open, migration runner, r2d2 pool, SC_RUNTIME_DB override
- Integration tests: migrations run on fresh DB, idempotent on re-run, two concurrent readers in WAL mode
- Add db boundary definition: sc-runtime-db has no impl dependency, sc-runtime-db-sqlite implements StorageBackend
- Wire storage into builder: `.db()` on HasCli and HasDaemon states
- Verify storage is absent from PluginContext when not configured (no Option<> footgun — use a separate PluginContextWithStorage newtype or sealed access)

**Deliverable:** sqlite storage working. Migration runner tested. Storage accessible in PluginContext.

---

#### Sprint 4 — sqlx Backend

**Goals:**
- `sc-runtime-db-sqlx`: sqlx AnyPool, sqlite + postgres, SC_RUNTIME_DB URL override
- Consumer can swap `SqliteBackend` for `SqlxBackend::from_url("postgres://...")` with zero other changes

**Tasks:**
- Implement `sc-runtime-db-sqlx` implementing StorageBackend via sqlx AnyPool
- URL scheme routing: `sqlite://` → SQLite pool, `postgres://` → Postgres pool
- Migration runner via sqlx migrate
- Integration tests: sqlite path parity with rusqlite backend (same migration fixtures)
- Integration tests: postgres path (requires test DB — can use testcontainers or skip in fast profile)
- Verify boundary rule: sc-runtime-db-sqlx does not appear in sc-runtime-db-sqlite dependency tree

**Deliverable:** Both storage backends working. sqlx/postgres path validated. Phase B gate passed.

---

### Phase C — Daemon + RPC

#### Sprint 5 — Daemon Lifecycle

**Goals:**
- `sc-runtime-daemon`: Plugin registry, PID file singleton, signal handling, structured concurrency
- Builder: `HasCli → HasDaemon` typestate transition via `.daemon()`

**Tasks:**
- Plugin registry: init/run/shutdown orchestration, JoinSet structured concurrency, panic isolation
- PID file: write on start, stale detection via sc-runtime-core pid liveness, cleanup on exit
- Signal handling: `#[cfg(unix)]` SIGTERM/SIGUSR1 + `#[cfg(windows)]` CTRL_C_EVENT companions (PORT-010)
- CancellationToken propagation from signal handlers to all plugin tasks
- Implement `HasCli → HasDaemon` typestate transition; `mcp_http` and supervised `web` only on HasDaemon
- Integration tests: start daemon, SIGTERM received, all plugin `shutdown()` called in order
- Integration tests: stale PID from dead process detected and reclaimed
- Verify SCB-RUNTIME lint rules pass on registry concurrency code

**Deliverable:** Daemon lifecycle fully functional. Plugin orchestration, PID singleton, signals tested.

---

#### Sprint 6 — RPC + CLI Router

**Goals:**
- `sc-runtime-daemon`: Unix socket server (macOS/Linux), named pipe server (Windows), TCP fallback
- Command router in `sc-runtime-cli` completes: IPC path + direct path
- End-to-end: `my-tool status` routes via socket when daemon is running, direct when not

**Tasks:**
- Implement RPC server: Unix socket listener (cfg unix), named pipe listener (cfg windows), TCP fallback
- Implement RPC client: connect to socket/pipe/TCP, send request, receive response, graceful Ok(None) on connection refused
- Wire request/response serialization: CommandEnvelope types over newline-delimited JSON
- Complete command router: detect socket/pipe → IPC, else direct execution
- Integration tests: server starts, client round-trip, request_id correlation on concurrent requests
- Integration tests: client returns Ok(None) when no server running (no error)
- End-to-end test: daemon started via builder, `status --json` routes via socket

**Deliverable:** Full daemon + RPC working. CLI router end-to-end. Phase C gate passed.

---

### Phase D — Web + Consumers

#### Sprint 7 — Web Layer

**Goals:**
- `sc-runtime-web`: axum HTTP server, foreground mode, daemon-supervised mode, HTTP MCP (SSE)
- Builder: `.web()` on HasCli (foreground) and HasDaemon (supervised); `.mcp_http()` only on HasDaemon

**Tasks:**
- Implement `sc-runtime-web`: axum router, `HttpRouteRegistry` trait for consumer route registration
- Foreground mode: web server runs directly in `ScRuntime::run()`
- Daemon-supervised mode: web server implements Plugin, runs inside daemon region
- HTTP MCP SSE endpoint: JSON-RPC 2.0 over Server-Sent Events
- Contract test: same JSON fixtures validated against CLI path, stdio MCP path, and HTTP MCP path
- PORT-010 check: any platform-specific axum binding code has companions

**Deliverable:** Web layer working in both modes. HTTP MCP endpoint tested.

---

#### Sprint 8 — Consumer Onboarding + Documentation

**Goals:**
- atm-core migrated to depend on sc-runtime
- Continuity onboarded as first real consumer
- ci onboarded
- README, consumer guide, and MIGRATION.md written
- sc-lint `full` profile (including xwin) passes for all onboarded repos

**Tasks:**
- Publish sc-runtime 0.1.0 to crates.io (all workspace crates)
- atm-core: add `[patch.crates-io]` for sc-runtime during migration; replace daemon lifecycle, PID, RPC, CLI infrastructure with sc-runtime equivalents; remove patch when 0.1.0 published
- Continuity: add `[patch.crates-io]`; replace daemon infrastructure with ScRuntime builder; `CiMonitorPlugin` and `PrWatchPlugin` implement Plugin trait; create `continuity-types` crate
- ci: add `[patch.crates-io]`; wire CLI passthrough and policy commands via sc-runtime-cli; create `ci-types` crate
- Write README: overview, composability matrix, consumer pattern, env vars
- Write MIGRATION.md: step-by-step guide from scratch-built infrastructure to sc-runtime
- Write CONSUMER.md: app-types crate pattern, `[patch.crates-io]` co-development workflow, when to pin crates.io version
- Run sc-lint full profile across sc-runtime, atm-core, Continuity, ci
- Verify all integration tests pass for all three consumers

**Deliverable:** sc-runtime 0.1.0 on crates.io. Three consumers migrated via import replacement (not repo surgery). Documentation complete.

---

## 15. Consumer Repository Pattern

### Repo Layout

sc-runtime is a standalone repo. Each SC tool is its own standalone repo. They never share a workspace.

```
github/
  sc-runtime/        ← own repo, own workspace, publishes to crates.io
  continuity/        ← own repo, [patch.crates-io] during migration sprint
  atm-core/          ← own repo, [patch.crates-io] during migration sprint
  ci/                ← own repo, [patch.crates-io] during migration sprint
```

### App-Types Crate Pattern

Every SC tool defines a `{tool}-types` crate with minimal dependencies:

```toml
# continuity-types/Cargo.toml
[dependencies]
sc-runtime-core = "0.1"   # Plugin trait, core types only
serde = { version = "1", features = ["derive"] }
```

This crate owns all tool-specific request/response structs, CLI command shapes, and error codes. The same types are used by the CLI binary, daemon RPC, and any MCP wrapper — zero reshaping across transports. It can be published to crates.io independently to support client SDKs or MCP wrappers.

```
continuity/
  crates/
    continuity-types/     # sc-runtime-core + serde only
    continuity-plugins/   # Plugin impls; depends on continuity-types
    continuity/           # binary; sc-runtime + continuity-types + continuity-plugins
```

### Co-Development Workflow

While sc-runtime and a consumer are being actively developed in parallel, the consumer workspace pins the local clone via `[patch.crates-io]`:

```toml
# continuity/Cargo.toml (workspace root)
[patch.crates-io]
sc-runtime = { path = "../sc-runtime/crates/sc-runtime" }
sc-runtime-core = { path = "../sc-runtime/crates/sc-runtime-core" }
sc-runtime-db = { path = "../sc-runtime/crates/sc-runtime-db" }
# add others as needed
```

When sc-runtime 0.1.0 publishes to crates.io, remove the `[patch]` block and update version pins. No structural changes to the consumer workspace.

### Semver Policy

sc-runtime is shared infrastructure — a breaking change in `sc-runtime-core::Plugin` or `StorageBackend` breaks every consumer simultaneously. `sc-lint check interfaces` (sc-lint-version, Phase C) is a required release gate before any crates.io publish. No crates.io publish without a clean interface version check.

---

## 16. Constraints and Limitations

**What sc-runtime does not provide:**
- Domain logic — tools own this entirely
- Agent messaging, ATM protocol, inbox/outbox — no knowledge of these
- HTTP auth, rate limiting, multi-tenancy — application layer concerns
- GUI or TUI — not in scope
- Stable FrankenSQLite native MVCC — deferred to when native mode ships
- Dynamic loadable plugins — all plugins compiled in; no dlopen

**What sc-runtime assumes:**
- The host OS provides a filesystem (for PID files and SQLite storage)
- The caller injects a valid `Logger<Running>` before calling `run()`
- Plugin implementations respect the `CancellationToken` — sc-runtime cannot forcibly terminate a plugin that ignores cancellation

---

*sc-runtime · v1.1 PRD · June 2026 · Synaptic Canvas*
