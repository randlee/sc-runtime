# `sc-runtime` Architecture

**Role:** Facade crate and typestate `ScRuntime` builder. Single consumer entry point.

---

## Overview

`sc-runtime` is the surface consumers depend on. It re-exports the public API of every sub-crate and provides the `ScRuntimeBuilder<State>` typestate builder that assembles a configured `ScRuntime` instance. Consumers add one dependency — `sc-runtime` — and get access to the entire workspace.

No business logic lives in this crate. Every capability (CLI, storage, daemon, web) is implemented in the sub-crates; this crate wires them together and enforces legal configurations at compile time.

---

## Typestate Builder Design (RBP-002)

The builder uses Rust's typestate pattern to make illegal configurations into compile errors. The type parameter `State` encodes what has been configured so far. Methods that require a prerequisite are only available on the builder state that guarantees the prerequisite is satisfied.

### State Types

```rust
pub struct NoCli;      // initial state — nothing configured
pub struct HasCli;     // CLI layer has been configured (logger already held)
pub struct HasDaemon;  // daemon layer has been configured (implies HasCli)
```

These are zero-sized marker structs. They carry no runtime data. Their only purpose is to constrain which methods are available at each step.

### Logger as a Positional Argument

The logger is a required argument to `ScRuntime::builder(name, logger)`, not
an optional builder method. This makes omitting the logger a compile error
rather than a runtime panic or silent no-op, which would violate FR-RT-08 (no
panics) and FR-RT-06 (logger required before `run()`).

```rust
pub fn builder(name: &'static str, logger: Logger<Running>) -> ScRuntimeBuilder<NoCli>
```

There is no `.logger()` method on the builder. The logger is received at
construction time and stored in `BuilderInner` immediately. Every builder
state from `NoCli` onward holds a valid, fully-initialized logger.

This avoids introducing additional typestate variants (`HasLogger`,
`HasCliAndLogger`, `HasDaemonAndLogger`) that would make the state transition
graph significantly more complex for a single required field. The positional
argument is the simpler enforcement.

### State Transitions

The builder is created with `ScRuntime::builder(name, logger)`. The logger is
a required positional argument — it cannot be omitted. From that point:

```
ScRuntime::builder("tool", logger)  → ScRuntimeBuilder<NoCli>   (logger already held)

ScRuntimeBuilder<NoCli>
  .cli(register)      → ScRuntimeBuilder<HasCli>      (transition: NoCli → HasCli)

ScRuntimeBuilder<HasCli>
  .mcp_stdio()        → ScRuntimeBuilder<HasCli>      (no state change)
  .db(backend)        → ScRuntimeBuilder<HasCli>      (no state change)
  .web(config)        → ScRuntimeBuilder<HasCli>      (foreground web, no state change)
  .plugin(plugin)     → ScRuntimeBuilder<HasCli>      (no state change)
  .daemon()           → ScRuntimeBuilder<HasDaemon>   (transition: HasCli → HasDaemon)
  .build()            → ScRuntime                     (foreground: no daemon)

ScRuntimeBuilder<HasDaemon>
  .db(backend)        → ScRuntimeBuilder<HasDaemon>   (no state change)
  .web(config)        → ScRuntimeBuilder<HasDaemon>   (supervised web, no state change)
  .mcp_http(addr)     → ScRuntimeBuilder<HasDaemon>   (ONLY on HasDaemon)
  .plugin(plugin)     → ScRuntimeBuilder<HasDaemon>   (no state change)
  .build()            → ScRuntime                     (daemon-managed)
```

There is no `.logger()` method anywhere on the builder. The logger is provided
once, up front, and is never optional.

`mcp_http()` does not exist on `ScRuntimeBuilder<HasCli>`. Calling it on a non-daemon builder is a compile error — the method is not in scope. This is the central enforcement: HTTP MCP requires persistent connections, which requires a running daemon. The type system enforces this with no runtime check.

### `.web()` Semantics: Foreground vs Supervised

`.web(config: WebConfig)` is available on **both** `HasCli` and `HasDaemon`, but its runtime behaviour differs:

| State | Mode | Behaviour |
|-------|------|-----------|
| `HasCli` | **Foreground** | `ScRuntime::run()` blocks on the web server directly. Process lifetime equals server lifetime — when the server exits, the process exits. No daemon process, no plugin JoinSet. |
| `HasDaemon` | **Supervised** | The web server runs as a `Plugin` inside the daemon JoinSet. It receives a `CancellationToken`, participates in the standard init/run/shutdown lifecycle, and is restartable in principle if the daemon's supervisor policy supports it. |

`.mcp_http()` is only available on `HasDaemon`. It requires persistent SSE connections, which in turn require a daemon process that outlives any single request. Calling `.mcp_http()` on `HasCli` is a compile error.

### PhantomData Marker (RBP-010)

```rust
pub struct ScRuntimeBuilder<State> {
    inner: BuilderInner,
    _state: PhantomData<State>,
}
```

`PhantomData<State>` marks that the builder is parameterized by `State` without storing a `State` value. The state is purely a compile-time annotation. At runtime, `ScRuntimeBuilder<NoCli>` and `ScRuntimeBuilder<HasDaemon>` have identical memory layout — only the compile-time type differs.

Builder methods that do not change state consume `self` and return `Self` with the same type parameter. Methods that change state consume `self: ScRuntimeBuilder<S>` and return `ScRuntimeBuilder<T>` where `T` is the new state.

---

## Builder Method Surface

```rust
impl ScRuntime {
    pub fn builder(name: &'static str, logger: Logger<Running>) -> ScRuntimeBuilder<NoCli>;
}

impl ScRuntimeBuilder<NoCli> {
    pub fn cli(self, register: impl FnOnce(&mut CommandRegistry)) -> ScRuntimeBuilder<HasCli>;
}

impl ScRuntimeBuilder<HasCli> {
    pub fn mcp_stdio(self) -> Self;
    pub fn db(self, backend: Arc<dyn StorageBackend>) -> Self;
    pub fn web(self, config: WebConfig) -> Self;
    pub fn plugin(self, plugin: impl Plugin + 'static) -> Self;
    pub fn daemon(self) -> ScRuntimeBuilder<HasDaemon>;
    pub fn build(self) -> ScRuntime;
}

impl ScRuntimeBuilder<HasDaemon> {
    pub fn db(self, backend: Arc<dyn StorageBackend>) -> Self;
    pub fn web(self, config: WebConfig) -> Self;
    pub fn mcp_http(self, addr: SocketAddr) -> Self;
    pub fn plugin(self, plugin: impl Plugin + 'static) -> Self;
    pub fn build(self) -> ScRuntime;
}
```

**`ScRuntime::builder(name, logger)`** — the `Logger<Running>` is a required
positional argument. The `Running` typestate from sc-observability guarantees
the logger is fully initialized before injection. `sc-runtime` does not
construct a logger. It is the consumer's responsibility to call `LoggerBuilder`
and pass the result here. There is no `.logger()` method on the builder —
omitting the logger is a compile error, not a runtime panic or silent no-op.

**`.db(backend: Arc<dyn StorageBackend>)`** — accepts a trait object, not a concrete backend type. The consumer selects the implementation (`SqliteBackend`, `SqlxBackend`) and wraps it in `Arc`. The builder holds `Arc<dyn StorageBackend>`. This satisfies RBP-003: the `StorageBackend` trait is sealed and consumers hold trait objects, not concrete types.

**`.plugin(plugin: impl Plugin + 'static)`** — registers a custom plugin. Available on both `HasCli` (foreground execution) and `HasDaemon` (supervised execution). The plugin is moved into the builder.

---

## ScRuntime::run()

```rust
impl ScRuntime {
    pub fn run(self) -> Result<(), ScRuntimeError>;
}
```

`ScRuntime::run()` is the process entry point. It starts all configured layers, blocks until shutdown, and returns. It never panics and never diverges (`!`). All error paths return `Err(ScRuntimeError)`.

In foreground mode (no daemon), `run()` directly executes the CLI command and returns.

In daemon mode, `run()` starts the daemon, blocks until SIGTERM / CTRL_C_EVENT triggers shutdown, and returns after the shutdown sequence completes.

---

## Consumer Patterns

### Pattern 1: Minimal CLI

A tool that parses arguments, executes a command, and exits. No background process, no storage, no HTTP.

```rust
fn main() -> Result<(), ScRuntimeError> {
    let logger = LoggerBuilder::new(LoggerConfig::default_for(
        ServiceName::new("my-tool")?,
        PathBuf::from("/var/log/sc/my-tool"),
    ))?.build();

    ScRuntime::builder("my-tool", logger)
        .cli(commands::register)
        .build()
        .run()
}
```

### Pattern 2: CLI + stdio MCP + sqlite

A tool that serves both CLI and MCP clients over stdio, backed by a local SQLite database. No daemon, no background process.

```rust
fn main() -> Result<(), ScRuntimeError> {
    let logger = LoggerBuilder::new(/* ... */)?.build();

    ScRuntime::builder("my-tool", logger)
        .cli(commands::register)
        .mcp_stdio()
        .db(Arc::new(SqliteBackend::open("my-tool.db")?))
        .build()
        .run()
}
```

### Pattern 3: Full stack — daemon + HTTP MCP + postgres + web

A long-running daemon with plugins, HTTP API, HTTP MCP, and PostgreSQL storage.

```rust
fn main() -> Result<(), ScRuntimeError> {
    let logger = LoggerBuilder::new(/* ... */)?.build();

    ScRuntime::builder("my-tool", logger)
        .cli(commands::register)
        .db(Arc::new(SqlxBackend::from_url("postgres://localhost/my-tool")?))
        .daemon()
        .plugin(MyDomainPlugin::new())
        .web(WebConfig::default())
        .mcp_http("127.0.0.1:3000".parse()?)
        .build()
        .run()
}
```

Note that `.mcp_http()` is called after `.daemon()`. The call chain before `.daemon()` returns `ScRuntimeBuilder<HasCli>`; calling `.daemon()` transitions to `ScRuntimeBuilder<HasDaemon>` where `.mcp_http()` is available. Attempting to call `.mcp_http()` before `.daemon()` is a compile error.

---

## Re-export Policy

`sc-runtime` re-exports the public API of each sub-crate so consumers can write `use sc_runtime::Plugin` rather than `use sc_runtime_core::Plugin`. The re-export surface is:

| Re-exported from | Items |
|-----------------|-------|
| `sc-runtime-core` | `Plugin`, `PluginMetadata`, `PluginContext`, `ScRuntimeHome`, `ScRuntimeError`, `ErrorCode`, `Remediation` |
| `sc-runtime-cli` | `CommandEnvelope`, `CliError`, `CliErrorKind`, `CommandId`, `CommandRegistry` |
| `sc-runtime-db` | `StorageBackend`, `Migration`, `StorageError`, `StorageHealth` |
| `sc-runtime-db-sqlite` | `SqliteBackend` (when `sqlite` feature active) |
| `sc-runtime-db-sqlx` | `SqlxBackend` (when `sqlx` feature active) |
| `sc-runtime-daemon` | `PluginError` (when `daemon` feature active) |
| `sc-runtime-web` | `WebConfig`, `HttpRouteRegistry` (when `web` feature active) |

axum types are never re-exported. `Logger<Running>` and `LoggerBuilder` are re-exported from `sc-observability` (consumers need them to construct the logger before injection).

---

## Dependencies

`sc-runtime` depends on all sub-crates, with the daemon and web sub-crates gated behind Cargo features:

| Feature | Activates |
|---------|-----------|
| *(default)* | `sc-runtime-core`, `sc-runtime-cli`, `sc-runtime-db` |
| `sqlite` | `sc-runtime-db-sqlite` |
| `sqlx` | `sc-runtime-db-sqlx` |
| `daemon` | `sc-runtime-daemon` |
| `web` | `sc-runtime-web` (foreground mode only) |
| `daemon`, `web` | `sc-runtime-web` with supervised mode and `mcp_http()` available |
| `mcp-stdio` | `sc-runtime-mcp-stdio` |

Feature flags are additive. No feature conflicts with another. Consumers declare exactly the features they need and take no compile-time cost for features they do not use.
