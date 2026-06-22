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
pub struct HasCli;     // CLI layer has been configured
pub struct HasDaemon;  // daemon layer has been configured (implies HasCli)
```

These are zero-sized marker structs. They carry no runtime data. Their only purpose is to constrain which methods are available at each step.

### State Transitions

```
ScRuntimeBuilder<NoCli>
  .name("tool")       → ScRuntimeBuilder<NoCli>      (no state change)
  .cli(register)      → ScRuntimeBuilder<HasCli>      (transition: NoCli → HasCli)

ScRuntimeBuilder<HasCli>
  .mcp_stdio()        → ScRuntimeBuilder<HasCli>      (no state change)
  .db(backend)        → ScRuntimeBuilder<HasCli>      (no state change)
  .web(config)        → ScRuntimeBuilder<HasCli>      (foreground web, no state change)
  .logger(logger)     → ScRuntimeBuilder<HasCli>      (no state change)
  .plugin(plugin)     → ScRuntimeBuilder<HasCli>      (no state change)
  .daemon()           → ScRuntimeBuilder<HasDaemon>   (transition: HasCli → HasDaemon)
  .build()            → ScRuntime                     (foreground: no daemon)

ScRuntimeBuilder<HasDaemon>
  .db(backend)        → ScRuntimeBuilder<HasDaemon>   (no state change)
  .web(config)        → ScRuntimeBuilder<HasDaemon>   (supervised web, no state change)
  .mcp_http(addr)     → ScRuntimeBuilder<HasDaemon>   (ONLY on HasDaemon)
  .logger(logger)     → ScRuntimeBuilder<HasDaemon>   (no state change)
  .plugin(plugin)     → ScRuntimeBuilder<HasDaemon>   (no state change)
  .build()            → ScRuntime                     (daemon-managed)
```

`mcp_http()` does not exist on `ScRuntimeBuilder<HasCli>`. Calling it on a non-daemon builder is a compile error — the method is not in scope. This is the central enforcement: HTTP MCP requires persistent connections, which requires a running daemon. The type system enforces this with no runtime check.

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
impl ScRuntimeBuilder<NoCli> {
    pub fn new() -> Self;
    pub fn name(self, name: &'static str) -> Self;
    pub fn cli(self, register: impl FnOnce(&mut CommandRegistry)) -> ScRuntimeBuilder<HasCli>;
}

impl ScRuntimeBuilder<HasCli> {
    pub fn mcp_stdio(self) -> Self;
    pub fn db(self, backend: Arc<dyn StorageBackend>) -> Self;
    pub fn web(self, config: WebConfig) -> Self;
    pub fn logger(self, logger: Logger<Running>) -> Self;
    pub fn plugin(self, plugin: impl Plugin + 'static) -> Self;
    pub fn daemon(self) -> ScRuntimeBuilder<HasDaemon>;
    pub fn build(self) -> ScRuntime;
}

impl ScRuntimeBuilder<HasDaemon> {
    pub fn db(self, backend: Arc<dyn StorageBackend>) -> Self;
    pub fn web(self, config: WebConfig) -> Self;
    pub fn mcp_http(self, addr: SocketAddr) -> Self;
    pub fn logger(self, logger: Logger<Running>) -> Self;
    pub fn plugin(self, plugin: impl Plugin + 'static) -> Self;
    pub fn build(self) -> ScRuntime;
}
```

**`.logger(logger: Logger<Running>)`** — accepts a `Logger<Running>` from `sc-observability`. The `Running` typestate from sc-observability guarantees the logger is fully initialized before injection. `sc-runtime` does not construct a logger. It is the consumer's responsibility to call `LoggerBuilder` and inject the result.

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

    ScRuntime::builder()
        .name("my-tool")
        .cli(commands::register)
        .logger(logger)
        .build()
        .run()
}
```

### Pattern 2: CLI + stdio MCP + sqlite

A tool that serves both CLI and MCP clients over stdio, backed by a local SQLite database. No daemon, no background process.

```rust
fn main() -> Result<(), ScRuntimeError> {
    let logger = /* ... */;

    ScRuntime::builder()
        .name("my-tool")
        .cli(commands::register)
        .mcp_stdio()
        .db(Arc::new(SqliteBackend::open("my-tool.db")?))
        .logger(logger)
        .build()
        .run()
}
```

### Pattern 3: Full stack — daemon + HTTP MCP + postgres + web

A long-running daemon with plugins, HTTP API, HTTP MCP, and PostgreSQL storage.

```rust
fn main() -> Result<(), ScRuntimeError> {
    let logger = /* ... */;

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
