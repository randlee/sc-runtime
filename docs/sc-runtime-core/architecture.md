# `sc-runtime-core` Architecture

This document records the architecture of the `sc-runtime-core` crate.

## Role

`sc-runtime-core` is the foundation crate of the sc-runtime workspace. Every
other crate in the workspace depends on it directly or transitively. It depends
on nothing else in the workspace.

Its responsibilities are narrow and stable:

- define the `Plugin` trait and its supporting types
- define `PluginContext` — the capability bundle injected into every plugin
- define `ScRuntimeHome` — the resolved home directory for all runtime artifacts
- define `ScRuntimeError` — the workspace-wide structured error type
- wire the sc-observability injection point

All workspace-wide abstractions that must not carry upstream domain knowledge
live here.

## Boundary Rules

`sc-runtime-core` must have zero dependencies on any SC domain crate
(`atm-core`, `continuity`, `ci`, or any future tool crate). Dependency flows
strictly downward: domain crates depend on sc-runtime; sc-runtime never depends
on domain crates.

This boundary is defined in `boundaries/sc-runtime-core/` and enforced by the
`sc-lint-boundary` CI gate. A pull request that introduces a domain crate
dependency into `sc-runtime-core` will fail the CI lint step. See
[`boundaries/sc-runtime-core/`](../../boundaries/sc-runtime-core/) for the
governing TOML definitions.

## `Plugin` Trait Design

The `Plugin` trait is the primary extension point for sc-runtime consumers.
Domain plugins implement it; the daemon registry orchestrates implementations
at runtime.

```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
    async fn run(&mut self, cancel: CancellationToken) -> Result<(), PluginError>;
    async fn shutdown(&mut self) -> Result<(), PluginError>;
}
```

### Object Safety via `BoxFuture`

Rust's object safety rules forbid `async fn` in traits that are used as trait
objects (`dyn Plugin`). The daemon plugin registry stores plugins as
`Box<dyn Plugin>` to support heterogeneous plugin collections without
monomorphization.

To satisfy both constraints — real `async fn` ergonomics for implementors and
`dyn Plugin` for the registry — `sc-runtime-core` provides a blanket
`PluginExt` implementation that wraps each async method in
`BoxFuture<'_, Result<...>>`. Consumers implement `async fn` directly; the
trait object machinery is internal to the crate.

This is the RBP-008 (Trait Object Safety) compliance point: object safety is
verified at design time, not discovered at compile time.

### `PluginMetadata`

```rust
pub struct PluginMetadata {
    pub name: &'static str,
    pub version: &'static str,
}
```

Accessors on `PluginMetadata` are infallible — they return `&'static str`,
never `Result`. This is RBP-007 (Infallible) applied to metadata access.

### Lifecycle Contract

| Method | Called by | Precondition | Result on error |
|--------|-----------|--------------|-----------------|
| `metadata()` | any | any | — (infallible) |
| `init()` | daemon, before `run()` | context injected | daemon aborts startup |
| `run()` | daemon, after all inits | `init()` succeeded | daemon cancels all plugins |
| `shutdown()` | daemon, on SIGTERM/cancel | `run()` running or complete | logged, cleanup continues |

The daemon calls `shutdown()` on all plugins in reverse-init order after the
`CancellationToken` fires. Plugin panics during any phase are caught by the
daemon's `JoinSet` and converted to `PluginError::Panic` — they do not
propagate to the runtime supervisor.

## `PluginContext`

`PluginContext` is the capability bundle delivered to each plugin during
`init()`. It carries everything a plugin needs and nothing it should not have.

```rust
pub struct PluginContext {
    pub logger: Arc<Logger<Running>>,
    pub cancel: CancellationToken,
    pub home: ScRuntimeHome,
}
```

When the daemon layer is configured with a storage backend, it delivers an
enriched context variant that additionally carries `Arc<dyn StorageBackend>`.
The base `PluginContext` in `sc-runtime-core` does not carry storage — that
association is made by `sc-runtime-daemon` when constructing the context for
each plugin.

### Logger Injection

`PluginContext.logger` is always an `Arc<Logger<Running>>` constructed and
injected by the application. `sc-runtime-core` never calls `LoggerBuilder` and
never constructs a `Logger` of its own. This is the ADR-07 injection contract:
tools own their log root, service name, sinks, and retention; sc-runtime owns
only the reference.

Plugins log exclusively through `ctx.logger`. They do not construct loggers
internally, and they do not use `println!`, `tracing`, or `log` directly for
structured output.

### `CancellationToken`

`PluginContext.cancel` is a `tokio_util::sync::CancellationToken`. Plugins must
poll or select on this token in their `run()` loop. When the token fires,
`run()` must return in a bounded time. The daemon does not forcibly terminate a
plugin that ignores cancellation — the plugin is responsible for honoring it.

## `ScRuntimeHome`

`ScRuntimeHome` is a newtype over `PathBuf` that encapsulates the resolved home
directory for all sc-runtime artifacts (PID files, sockets, database files).

```rust
pub struct ScRuntimeHome(PathBuf);
```

### Resolution Order

1. `SC_RUNTIME_HOME` environment variable, if set and non-empty
2. OS-appropriate home directory (`$HOME` on Unix, `USERPROFILE` on Windows)
   with `.sc/runtime` appended

### Test Isolation

Tests set `SC_RUNTIME_HOME` to a temporary directory. Because `ScRuntimeHome`
reads the variable at construction time, each test process gets an isolated
artifact root without any global state mutation. Parallel test execution is
safe.

### Trait Implementations

`ScRuntimeHome` implements `Deref<Target = Path>` (RBP-005), so it can be
passed anywhere a `&Path` is expected without an explicit `.as_path()` call.
It also implements `AsRef<Path>` and `Display` for ergonomic use in format
strings and file-open calls.

## `ScRuntimeError`

`ScRuntimeError` is the workspace-wide structured error type. It complies with
RBP-001 (Error Context + Recovery): every variant carries a stable error code,
a human-readable message, an optional cause chain, and an optional
`Remediation`.

```rust
pub struct ScRuntimeError {
    pub code: ErrorCode,
    pub message: &'static str,
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub remediation: Option<Remediation>,
}
```

### Stable Error Codes

`ErrorCode` is a newtype over a `&'static str` with a dotted-namespace
convention: `SC_RUNTIME.{SUBSYSTEM}.{NAME}`. Examples:

| Code | Meaning |
|------|---------|
| `SC_RUNTIME.CORE.HOME_RESOLUTION` | `SC_RUNTIME_HOME` set to an invalid path |
| `SC_RUNTIME.DAEMON.ALREADY_RUNNING` | Second daemon instance detected live PID |
| `SC_RUNTIME.PLUGIN.INIT_FAILED` | Plugin `init()` returned an error |
| `SC_RUNTIME.PLUGIN.PANIC` | Plugin task panicked |

Codes are stable across patch and minor releases. Removing or renaming a code
is a semver-breaking change subject to `sc-lint-version` enforcement.

### `Remediation`

`Remediation` carries a `suggested_action: String` that tooling (CLI output,
MCP error responses) surfaces to the operator. The remediation is always
present on variants that represent operator-recoverable conditions. It is
`None` only on internal errors where no user action can help.

### Derivation

`ScRuntimeError` uses `thiserror` for `Display` and `Error` derivation.
The `cause` field implements the `source()` chain, enabling standard error
introspection by the CLI and MCP layers.

## Dependency List

`sc-runtime-core` has exactly three runtime dependencies outside of `std`:

| Crate | Version | Purpose |
|-------|---------|---------|
| `sc-observability` | 1.2.0 | `Logger<Running>` type used in `PluginContext` |
| `tokio-util` | 0.7 | `CancellationToken` used in `PluginContext` and `Plugin::run()` |
| `thiserror` | 2 | `ScRuntimeError` and `PluginError` derivation |

`tokio` itself is not a direct dependency of `sc-runtime-core`. The
`CancellationToken` type from `tokio-util` has no tokio runtime requirement
at the type level. This keeps `sc-runtime-core` free of async-runtime
assumptions and usable in sync contexts if needed.

## Newtype Inventory

The following newtypes are defined in `sc-runtime-core` (RBP-004):

| Newtype | Wraps | Purpose |
|---------|-------|---------|
| `ScRuntimeHome` | `PathBuf` | Resolved home directory with env override |
| `ErrorCode` | `&'static str` | Stable dotted error code identifier |
| `PluginName` | `Cow<'static, str>` | Plugin identity, allocation-free for static names |

`PluginName::new` accepts `impl Into<Cow<'static, str>>` (RBP-009), avoiding
allocation when the common case of a `'static` string literal is used.

## Related Docs

- [requirements.md](./requirements.md)
- [../sc-runtime-cli/architecture.md](../sc-runtime-cli/architecture.md)
- [../../boundaries/sc-runtime-core/](../../boundaries/sc-runtime-core/)
- [../../docs/prd/sc-runtime-prd.md](../../docs/prd/sc-runtime-prd.md) — Section 6.1, 9
