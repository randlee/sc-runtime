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
use futures::future::BoxFuture;
use tokio_util::sync::CancellationToken;

pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;

    fn init<'a>(&'a mut self, ctx: &'a PluginContext) -> BoxFuture<'a, Result<(), PluginError>>;
    fn run<'a>(&'a mut self, cancel: CancellationToken) -> BoxFuture<'a, Result<(), PluginError>>;
    fn shutdown<'a>(&'a mut self) -> BoxFuture<'a, Result<(), PluginError>>;
}
```

### Object Safety via `BoxFuture`

**Why `async fn` in traits is not object-safe.** As of Rust 1.94, async
functions in traits (AFIT/RPITIT) are stable, but using them with `dyn Trait`
still requires that every `async fn` return a concrete, statically-known future
type. Because each impl produces a distinct future type, there is no single
vtable entry that covers all implementations — the compiler rejects
`Box<dyn Plugin>` when the trait contains bare `async fn` methods.

**Why `BoxFuture<'a, _>` resolves this.** `BoxFuture<'a, T>` is a type alias
for `Pin<Box<dyn Future<Output = T> + Send + 'a>>`. Returning a heap-allocated,
type-erased future from each method means every implementation returns the same
concrete type (`Pin<Box<...>>`), which satisfies the vtable requirement and
makes `Box<dyn Plugin>` compile. The daemon plugin registry stores plugins as
`Box<dyn Plugin>` to support heterogeneous plugin collections without
monomorphization.

**How consumer plugins implement this trait.** Two patterns are valid:

1. **`async-trait` proc-macro** (recommended for readability): annotate the
   impl block with `#[async_trait::async_trait]`. The macro rewrites each
   `async fn` body to return `BoxFuture` automatically. The impl reads as
   normal `async fn`.

2. **Manual `BoxFuture` wrapping**: write each method to return
   `Box::pin(async move { ... })` explicitly. No proc-macro dependency
   required.

Both patterns produce identical vtable entries. The choice is a per-crate
style decision.

**RBP-008 compliance and CI enforcement.** This is the RBP-008 (Trait Object
Safety) compliance point. Object safety is verified at compile time by a CI
test that asserts `Box<dyn Plugin>` is constructible:

```rust
#[test]
fn plugin_trait_is_object_safe() {
    // This line does not need to run — it only needs to compile.
    let _: Box<dyn Plugin>;
}
```

If the `Plugin` trait definition ever regresses to bare `async fn` methods,
this test will fail to compile and block the PR.

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

`PluginContext` is the base capability bundle delivered to each plugin during
`init()`. It carries exactly the capabilities that belong to the core layer
and nothing else.

```rust
pub struct PluginContext {
    pub logger: Arc<Logger<Running>>,
    pub cancel: CancellationToken,
    pub home: ScRuntimeHome,
}
```

`PluginContext` is defined and owned entirely by `sc-runtime-core`. It has
exactly three fields and will never grow additional fields that require
daemon-layer types (wake channels, storage backends, etc.). This is a
deliberate ownership boundary: a struct owned by `sc-runtime-core` cannot
carry fields whose types are defined in `sc-runtime-daemon` without
introducing an illegal reverse dependency.

Daemon-layer context — a `WakeEvent` receiver and an optional
`StorageBackend` — is delivered through a separate mechanism. See
`sc-runtime-daemon` for the `DaemonPluginContext` type and the `DaemonAware`
supertrait that governs its injection.

### Layered Context Design

`PluginContext` is the base context. `sc-runtime-daemon` defines
`DaemonPluginContext`, which wraps `PluginContext` and extends it with
daemon-specific capabilities:

```
PluginContext          (sc-runtime-core)
  logger, cancel, home

DaemonPluginContext    (sc-runtime-daemon)
  base: PluginContext
  wake: broadcast::Receiver<WakeEvent>
  storage: Option<Arc<dyn StorageBackend>>
```

The `Plugin::init` signature takes `&PluginContext` — it receives only the
base context. Plugins that need daemon-specific capabilities implement the
optional `DaemonAware` supertrait, defined in `sc-runtime-daemon`. The daemon
calls `plugin.on_daemon_context(&daemon_ctx)` after `init()` completes for
any plugin that implements `DaemonAware`.

This layered approach means:
- Core plugins with no daemon dependencies remain in `sc-runtime-core` and
  receive only `&PluginContext`.
- Daemon-aware plugins depend on `sc-runtime-daemon` and opt in to
  `DaemonPluginContext` injection via `DaemonAware`.
- The ownership boundary is clean: `sc-runtime-core` never references types
  from `sc-runtime-daemon`.

### Logger Injection

`PluginContext.logger` is always an `Arc<Logger<Running>>` constructed and
injected by the application. `sc-runtime-core` never calls `LoggerBuilder` and
never constructs a `Logger` of its own. This is the ADR-07 injection contract:
tools own their log root, service name, sinks, and retention; sc-runtime owns
only the reference.

Plugins log exclusively through `ctx.logger`. They do not construct loggers
internally, and they do not use `println!`, `tracing`, or `log` directly for
structured output.

### Stable `ActionName` Values

sc-runtime emits log events using the following stable `ActionName` values.
These are stable as of 0.1.0. Removing or renaming an action name is a
breaking change — `sc-lint-version` will catch it via the public type
`ActionName`.

| ActionName | When emitted |
|---|---|
| `sc_runtime.daemon.started` | Daemon finished init, all plugins running |
| `sc_runtime.daemon.stopping` | SIGTERM received, shutdown beginning |
| `sc_runtime.daemon.stopped` | All plugins shut down, process about to exit |
| `sc_runtime.plugin.init_started` | Plugin `init()` called |
| `sc_runtime.plugin.init_completed` | Plugin `init()` returned `Ok` |
| `sc_runtime.plugin.init_failed` | Plugin `init()` returned `Err` |
| `sc_runtime.plugin.panic` | Plugin panicked, `PluginError::Panic` created |
| `sc_runtime.rpc.request_received` | RPC server received a request |
| `sc_runtime.rpc.request_completed` | RPC server sent response |
| `sc_runtime.storage.migration_started` | `migrate()` called |
| `sc_runtime.storage.migration_completed` | All migrations applied |
| `sc_runtime.storage.health_degraded` | `StorageHealth::Degraded` detected |

### `CancellationToken`

`PluginContext.cancel` is a `tokio_util::sync::CancellationToken`. Plugins must
poll or select on this token in their `run()` loop. When the cancellation token
fires, `run()` must return within the `cancellation_ack_timeout` configured in
`DaemonConfig` (default: 10 seconds). Plugins that do not return within this
window will be forcibly dropped by the daemon's JoinSet.

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
| `futures` | 0.3 | `BoxFuture` type alias used in `Plugin` trait method signatures |

`tokio` itself is not a direct dependency of `sc-runtime-core`. The
`CancellationToken` type from `tokio-util` has no tokio runtime requirement
at the type level. This keeps `sc-runtime-core` free of async-runtime
assumptions and usable in sync contexts if needed.

## Supporting Types

### `ErrorCode`

`ErrorCode` is a `&'static str` newtype that carries a stable, machine-readable
error identifier. Codes use the `SC_RUNTIME.{SUBSYSTEM}.{NAME}` convention —
all uppercase, dot-separated — and never change within a published version.
Consumers may match on `ErrorCode` values programmatically.

```rust
pub struct ErrorCode(pub &'static str);

impl ErrorCode {
    pub const fn new(code: &'static str) -> Self { Self(code) }
    pub fn as_str(&self) -> &'static str { self.0 }
}
```

Predefined codes defined in `sc-runtime-core`:

| Code | Meaning |
|------|---------|
| `SC_RUNTIME.CORE.HOME_NOT_FOUND` | `SC_RUNTIME_HOME` resolves to a nonexistent path |
| `SC_RUNTIME.CORE.PLUGIN_INIT_FAILED` | A plugin's `init()` returned an error |
| `SC_RUNTIME.CORE.PLUGIN_RUN_FAILED` | A plugin's `run()` returned an error |
| `SC_RUNTIME.CORE.PLUGIN_SHUTDOWN_FAILED` | A plugin's `shutdown()` returned an error |
| `SC_RUNTIME.CORE.PLUGIN_PANIC` | A plugin task panicked |
| `SC_RUNTIME.PLUGIN.NAME_COLLISION` | Two plugins registered the same name |

Adding a new code is non-breaking. Removing or renaming a code is a
semver-breaking change subject to `sc-lint-version` enforcement.

### `Remediation`

`Remediation` is a structured suggestion for error recovery, surfaced to
operators by the CLI and MCP layers.

```rust
pub struct Remediation {
    pub suggested_action: String,  // human-readable: what the user/operator should do
    pub docs_url: Option<String>,  // optional link to relevant documentation
    pub command: Option<String>,   // optional CLI command to run, e.g. "sc-runtime diagnose"
}
```

`Remediation` is always `Some` on `ScRuntimeError` variants where an operator
corrective action exists. It is `None` only on internal errors where no user
action can help.

### `PluginError`

`PluginError` is the error type returned by all `Plugin` lifecycle methods
(`init`, `run`, `shutdown`). It is the only error type permitted in
`BoxFuture<'a, Result<(), PluginError>>` return positions on the `Plugin`
trait.

```rust
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin init failed: {reason}")]
    InitFailed {
        plugin_name: &'static str,
        reason: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        remediation: Option<Remediation>,
    },

    #[error("plugin run failed: {reason}")]
    RunFailed {
        plugin_name: &'static str,
        reason: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        remediation: Option<Remediation>,
    },

    #[error("plugin shutdown failed: {reason}")]
    ShutdownFailed {
        plugin_name: &'static str,
        reason: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("plugin panicked: {message}")]
    Panic {
        plugin_name: String,  // String, not &'static str — panic may corrupt statics
        message: String,      // captured from std::panic::catch_unwind
    },

    #[error("plugin cancelled")]
    Cancelled {
        plugin_name: &'static str,
    },
}
```

The `Panic` variant uses `String` (not `&'static str`) for `plugin_name`
because a panic may corrupt static memory — allocating a fresh `String` before
the panic handler runs avoids use-after-free.

The daemon's `JoinSet` catches plugin panics and converts them to
`PluginError::Panic`. They do not propagate to the runtime supervisor.
`PluginError` variants are exhaustive and stable; adding a new variant is a
semver-breaking change.

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
