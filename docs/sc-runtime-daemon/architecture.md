# `sc-runtime-daemon` Architecture

**Role:** Plugin registry, daemon singleton lifecycle, cross-platform RPC.

---

## Overview

`sc-runtime-daemon` is the process-management and inter-process communication layer for sc-runtime. It owns three concerns that are tightly coupled by design: the plugin registry that orchestrates `Plugin` trait objects, the PID file singleton that enforces single-instance semantics, and the cross-platform RPC server that routes CLI commands to a running daemon.

No domain logic lives here. The daemon starts plugins, keeps them alive, routes requests to them, and shuts them down cleanly. Everything else belongs to consumers.

---

## DaemonPluginContext and DaemonAware

### DaemonPluginContext

`DaemonPluginContext` is defined in `sc-runtime-daemon`. It wraps the base
`PluginContext` from `sc-runtime-core` and adds the daemon-layer capabilities
that `sc-runtime-core` cannot carry without a reverse dependency:

```rust
pub struct DaemonPluginContext {
    pub base: PluginContext,
    pub wake: tokio::sync::broadcast::Receiver<WakeEvent>,
    pub storage: Option<Arc<dyn StorageBackend>>,
}

impl DaemonPluginContext {
    pub fn as_base(&self) -> &PluginContext {
        &self.base
    }
}
```

`DaemonPluginContext` implements `AsRef<PluginContext>` so it can be
dereferenced to the base context anywhere a `&PluginContext` is accepted.

### DaemonAware Supertrait

`DaemonAware` is an optional supertrait defined in `sc-runtime-daemon`. Plugins
that need wake events or storage access implement it:

```rust
pub trait DaemonAware: Plugin {
    fn on_daemon_context<'a>(
        &'a mut self,
        ctx: &'a DaemonPluginContext,
    ) -> BoxFuture<'a, Result<(), PluginError>>;
}
```

### Injection Sequence

The daemon plugin registry drives the following sequence for each registered
plugin:

1. **`plugin.init(&daemon_ctx.as_base())`** — called for every plugin.
   Receives the base `PluginContext` (logger, cancel, home).
2. **`plugin.on_daemon_context(&daemon_ctx)`** — called only for plugins that
   implement `DaemonAware`, immediately after `init()` succeeds. The plugin
   receives the full `DaemonPluginContext` (wake receiver, storage).
3. **`plugin.run(cancel)`** — called after all init + on_daemon_context calls
   have completed successfully.

Plugins that do not implement `DaemonAware` never receive a `DaemonPluginContext`.
This keeps core-only plugins free of any daemon dependency.

---

## Plugin Registry

### Lifecycle Orchestration

The plugin registry holds a `Vec<Box<dyn Plugin>>` and drives each plugin through three ordered phases:

```
init (sequential, forward order)
  ↓
run (concurrent, JoinSet)
  ↓
shutdown (sequential, reverse-init order)
```

**Init phase** — plugins are initialized in registration order. For each
plugin, `Plugin::init(&mut self, ctx: &PluginContext)` is awaited first. If
the plugin implements `DaemonAware`, `plugin.on_daemon_context(&daemon_ctx)`
is then awaited before moving to the next plugin. A failure in either call
aborts the startup sequence: already-initialized plugins receive `shutdown()`
in reverse order before the daemon exits. The error is returned as
`ScRuntimeError`.

**Run phase** — after all plugins have successfully initialized, `Plugin::run(&mut self, cancel: CancellationToken)` is spawned for each plugin as a separate tokio task inside a `JoinSet`. The plugins run concurrently. The `CancellationToken` passed to each `run` call is derived from the daemon's root token — signaling the root token propagates cancellation to all running plugins.

**Shutdown phase** — triggered when the root `CancellationToken` is cancelled (by SIGTERM, CTRL_C_EVENT, or programmatic request). The registry awaits the `JoinSet` to drain — each plugin's `run` is expected to observe its cancellation token and return. Once all tasks have exited, `shutdown()` is called on each plugin in reverse-init order. This reverse ordering allows plugins registered later (which may depend on earlier ones) to release resources before their dependencies do.

### Structured Concurrency

All plugin tasks are spawned into a single `JoinSet<Result<(), PluginError>>`. There are no detached `tokio::spawn` calls in the registry. This guarantees:

- Every spawned task is observable — the `JoinSet` collects all outcomes.
- No task outlives the registry's `run` scope.
- Cancellation propagation is complete — when the root token is cancelled and all tasks drain from the `JoinSet`, the daemon knows with certainty that every plugin has stopped.

```
JoinSet
  ├── plugin_a::run(cancel_child_a)
  ├── plugin_b::run(cancel_child_b)
  └── plugin_c::run(cancel_child_c)
```

Each plugin receives a child cancellation token cloned from the daemon root. This allows targeted plugin cancellation without affecting others, while SIGTERM on the root propagates to all children simultaneously.

### Panic Isolation

Plugin panics are contained at the `JoinSet` boundary. A panic inside `Plugin::run` causes the corresponding tokio task to terminate with a `JoinError`. The registry catches this outcome when it polls the `JoinSet` and converts it to `PluginError::Panic { plugin_name, message }`. The error is logged via the injected `Logger<Running>` and the daemon continues operating with the remaining plugins.

Plugin panics never propagate to `ScRuntime::run()`. `ScRuntime::run()` returns `Result<(), ScRuntimeError>` — it never panics, never diverges. The `ScRuntimeError` variant for a plugin panic carries the plugin name and a captured backtrace (where available) as the cause field.

---

## PID File Singleton

### Write-on-Start

On daemon startup, before any plugin initialization, the daemon resolves the PID file path:

```
{SC_RUNTIME_HOME}/.sc/runtime/{tool}.pid
```

`SC_RUNTIME_HOME` is resolved via `ScRuntimeHome` from `sc-runtime-core`. The `.sc/runtime/` directory is created if absent.

The startup sequence:

1. Check if `{tool}.pid` exists.
2. If it does not exist, write the current PID and proceed.
3. If it exists, read the stored PID and perform a liveness check.
4. If the stored process is alive: exit immediately with `DAEMON.ALREADY_RUNNING`.
5. If the stored process is dead (stale PID): overwrite the PID file with the current PID and proceed.

### Stale Detection

Liveness checking is implemented in `sc-runtime-daemon` as a crate-private cross-platform function. The daemon does not attempt to parse or validate the previous daemon's state — it only asks "is this PID alive?". If the answer is no, the PID file is stale and is reclaimed.

#### PID Liveness Check

```rust
pub(crate) fn pid_is_alive(pid: u32) -> bool
```

**Unix** — sends signal 0 to the process (`kill(pid, 0)`):
- Returns `true` if the call succeeds (`errno == 0`): the process exists and we have permission to signal it.
- Returns `true` if the call fails with `EPERM`: the process exists but we lack permission — it is still alive.
- Returns `false` if the call fails with `ESRCH`: no such process — the PID is stale.

Uses `libc::kill` and `libc::ESRCH`/`libc::EPERM` for the errno check.

**Windows** — calls `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid)`:
- Returns `true` if the returned handle is non-null: the process exists.
- Returns `false` if the handle is null: no such process — the PID is stale.

Uses `windows-sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION}`.

**TOCTOU note** — both approaches have an inherent race: the checked process could exit between the liveness check and the PID file overwrite. This is acceptable. The PID file is the lock, not the liveness check. If two daemon instances race, the second one will fail to bind the socket/pipe anyway, producing a clean `DAEMON.ALREADY_RUNNING` error.

### Cleanup on Exit

The PID file is deleted as the last step before process exit, after all plugins have shut down. Deletion is performed unconditionally — whether shutdown was triggered by SIGTERM, CTRL_C_EVENT, or a fatal plugin error. If deletion fails (e.g., the file was already removed), the error is logged but does not affect the exit code.

---

## Signal Handling

Signal handling is strictly platform-gated. Every `#[cfg(unix)]` block has a `#[cfg(windows)]` companion. This is a PORT-010 compliance requirement enforced by `sc-lint-portability` in CI.

### Unix: SIGTERM and SIGUSR1

```rust
#[cfg(unix)]
{
    // SIGTERM → cancel the root CancellationToken
    let mut sigterm = signal(SignalKind::terminate())?;
    tokio::spawn(async move {
        sigterm.recv().await;
        root_cancel.cancel();
    });

    // SIGUSR1 → send wake notification to all plugins via channel
    let mut sigusr1 = signal(SignalKind::user_defined1())?;
    tokio::spawn(async move {
        loop {
            sigusr1.recv().await;
            wake_tx.send(WakeEvent).ok();
        }
    });
}
```

**SIGTERM** cancels the root `CancellationToken`. This propagates to all running plugin tasks and begins the shutdown sequence described in the plugin registry section above.

**SIGUSR1** sends a `WakeEvent` on a `tokio::sync::broadcast` channel. The
daemon's `DaemonPluginContext` carries a `broadcast::Receiver<WakeEvent>` for
each plugin (cloned from the broadcast sender at init time). Only plugins that
implement `DaemonAware` receive this field via `on_daemon_context()`; they
select on `ctx.wake` in their `run()` loop. SIGUSR1 does not terminate the
daemon.

Signal handlers are implemented using `tokio::signal::unix::signal` from the tokio async runtime. The `libc` crate is used for signal constant definitions and for `kill(2)` in `pid_is_alive` (the crate-private Unix liveness check).

#### WakeEvent Channel Specification

Channel type: `tokio::sync::broadcast::Sender<WakeEvent>` / `broadcast::Receiver<WakeEvent>`. The daemon holds a single `broadcast::Sender<WakeEvent>`. Each `DaemonPluginContext` carries a `broadcast::Receiver<WakeEvent>` cloned from the sender at plugin registration time.

```rust
pub enum WakeEvent {
    /// Sent on SIGUSR1 (Unix) or SC_RUNTIME_WAKE named event (Windows).
    /// Plugins should use this to re-read config, flush state, or perform
    /// any periodic work they would otherwise do on a timer.
    UserSignal,
}
```

Channel capacity: `broadcast` channel with capacity 16. Lagging receivers (plugins that have not polled) will miss events — this is intentional. `WakeEvent` is edge-triggered, not level-triggered. A plugin that misses a `WakeEvent` due to lag must remain correct; it simply defers any periodic work to the next event or its own internal timer.

### Windows: CTRL_C_EVENT and Named Wake Event

```rust
#[cfg(windows)]
{
    // CTRL_C_EVENT → cancel the root CancellationToken
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        root_cancel.cancel();
    });
    // Named Windows Event → send WakeEvent to plugins
    // Named event: Global\SC_RUNTIME_{TOOL_NAME_UPPER}_WAKE
}
```

`CTRL_C_EVENT` maps to graceful shutdown via `tokio::signal::ctrl_c()`. This mirrors SIGTERM semantics: the root `CancellationToken` is cancelled, plugins drain, and shutdown proceeds in reverse order.

**Windows wake equivalent**: On Windows, the daemon creates a named Windows Event object at `Global\SC_RUNTIME_{TOOL_NAME_UPPER}_WAKE` (where `TOOL_NAME_UPPER` is the tool name uppercased, e.g. `Global\SC_RUNTIME_MYTOOL_WAKE`). External processes can call `SetEvent()` on this named event to trigger wake delivery. The daemon polls this named event using `tokio::task::spawn_blocking` with `WaitForSingleObject(handle, 0)` in a loop, sending `WakeEvent::UserSignal` on the broadcast channel when the event is signaled.

If the named event cannot be created (e.g., insufficient permissions to create a `Global\` object), the daemon logs a warning at startup and the wake feature is silently disabled for that session. The broadcast channel still exists and plugins remain correct — they simply never receive `WakeEvent` on that platform/session.

---

## Daemon Configuration

The drain timeout and cancellation acknowledgement timeout are configurable via `DaemonConfig`:

```rust
pub struct DaemonConfig {
    /// Maximum time to wait for all plugins to complete shutdown() before forcibly terminating.
    /// After this duration, any plugins that have not returned from shutdown() are dropped.
    /// Default: 30 seconds.
    pub shutdown_drain_timeout: Duration,

    /// Maximum time a plugin's run() may take to acknowledge CancellationToken cancellation
    /// before it is considered hung and forcibly cancelled.
    /// Default: 10 seconds.
    pub cancellation_ack_timeout: Duration,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            shutdown_drain_timeout: Duration::from_secs(30),
            cancellation_ack_timeout: Duration::from_secs(10),
        }
    }
}
```

`DaemonConfig` is supplied via an optional builder method on `ScRuntimeBuilder<HasCli>` before calling `.daemon()`:

```rust
builder
    .daemon_config(DaemonConfig {
        shutdown_drain_timeout: Duration::from_secs(60),
        ..DaemonConfig::default()
    })
    .daemon()
```

If `.daemon_config()` is not called, `DaemonConfig::default()` is used.

**Environment variable override**: `SC_RUNTIME_SHUTDOWN_TIMEOUT_SECS` (integer seconds) overrides `shutdown_drain_timeout` at runtime. If set to a valid positive integer, it takes precedence over the value in `DaemonConfig`. If set to an invalid value, the daemon logs a warning and falls back to the configured or default value.

---

## RPC Transport

The RPC layer provides a local IPC server so that CLI invocations can route to a running daemon. The protocol is identical on all transports.

### Transport Selection

| Platform | Transport | Path |
|----------|-----------|------|
| macOS / Linux | Unix domain socket | `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.sock` |
| Windows | Named pipe | `\\.\pipe\sc-{tool}` |
| Fallback (all) | TCP loopback | port registered in `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.port` |

Transport selection is compile-time gated:

```rust
#[cfg(unix)]
fn bind_rpc_server(tool: &str, home: &ScRuntimeHome) -> Result<RpcListener, ScRuntimeError> {
    // Unix domain socket at {home}/.sc/runtime/{tool}.sock
}

#[cfg(windows)]
fn bind_rpc_server(tool: &str, _home: &ScRuntimeHome) -> Result<RpcListener, ScRuntimeError> {
    // Named pipe at \\.\pipe\sc-{tool}
}
```

The TCP fallback is available on all platforms and activates when neither Unix socket nor named pipe binding succeeds. The bound port number is written to `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.port` for the CLI client to discover.

### Wire Format

The wire format is newline-delimited JSON. Each connection carries exactly one request and one response.

**Request envelope:**

```json
{
  "version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "command": "runtime.health",
  "payload": {}
}
```

**Response envelope:**

```json
{
  "version": 1,
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "ok": true,
  "command": "runtime.health",
  "data": { "state": "running", "plugins": [...] }
}
```

The `CommandEnvelope<T>` and `CliError` types from `sc-runtime-cli` are used directly — there are no transport-specific DTOs. The `request_id` field (a UUID v4 string) allows the CLI client to correlate responses when issuing concurrent requests.

The `version` field is an integer. Version 1 is the current protocol. The daemon rejects requests with a higher `version` than it understands and returns a `DAEMON.PROTOCOL_VERSION` error so the client can present a meaningful message.

### Message Size Limits

Maximum RPC message size (request or response): **512 KiB** (524,288 bytes).
Messages exceeding this limit are rejected with error code
`DAEMON.MESSAGE_TOO_LARGE` in a `CommandEnvelope` response. This applies to
all three transports (Unix socket, named pipe, TCP loopback).

Per-connection read buffer: **64 KiB**. The daemon grows its line buffer up to
the 512 KiB limit per connection. Connections are capped at **50 concurrent
RPC connections** per daemon instance (separate from SSE sessions, which are
in the web layer).

### Request Dispatch

Each accepted connection is handled in a spawned task inside the daemon's `JoinSet`. The request is deserialized, dispatched to the plugin registry's command router, and the response is serialized and written back before the connection closes. Command dispatch follows the same operation layer as direct CLI execution — the transport is transparent.

---

## TCP Fallback Port Protocol

This section specifies the full sub-protocol for TCP fallback port allocation,
file management, stale detection, and client-side discovery.

### Port Allocation

The daemon binds to `127.0.0.1:0` (OS-assigned ephemeral port) and reads the
assigned port number from the bound socket after binding. The OS selects a port
from its ephemeral range (typically 49152–65535 on Linux/macOS, 1024–65535 on
Windows). No fixed port range is used — binding to `:0` avoids port conflicts
entirely. The port changes on each daemon restart; the port file is the single
source of truth.

### Port File

After binding, the daemon writes the decimal port number as a UTF-8 string to:

```
{SC_RUNTIME_HOME}/.sc/runtime/{tool}.port
```

The write is **atomic**: the port number is written to
`{SC_RUNTIME_HOME}/.sc/runtime/{tool}.port.tmp` first, then renamed to
`{tool}.port`. This prevents the CLI client from reading a partial write.

### Port File Cleanup

The port file is deleted in the **same shutdown step as the PID file** — both
are removed before the process exits, after all plugin `shutdown()` calls
complete. Deletion failure is logged but does not affect the exit code.

If the process crashes (port file not cleaned up by the shutdown sequence),
the port file is considered stale. Stale detection rule: read the PID file
first. If the PID stored in `{tool}.pid` refers to a dead process, the port
file is stale regardless of its age or content. The client must not attempt to
connect to a port from a stale port file.

### Client-Side Discovery

The CLI client discovers the TCP fallback port as follows:

1. Read `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.port`.
2. If the file does not exist, TCP fallback is not attempted.
3. If the file exists, check `{tool}.pid` — if the PID is dead, the port file
   is stale; TCP fallback is not attempted.
4. Parse the port number from the port file.
5. Attempt `127.0.0.1:{port}` with the same **500ms connect timeout** as the
   Unix socket / named pipe connect.
6. If the connect fails or times out, the daemon is considered unreachable;
   fall back to direct execution.
7. If the connect succeeds, proceed with the standard NDJSON request/response
   exchange (same mid-read timeout and error handling as the primary transport).

### Shutdown Sequence Integration

Port file deletion is added to the shutdown sequence between RPC listener close
and PID file deletion:

```
6. RPC server listener closed — no new connections accepted
6a. Port file deleted (same step as PID file, both removed together)
7. PID file deleted
8. ScRuntime::run() returns Ok(())
```

---


## Shutdown Sequence

The complete shutdown sequence from SIGTERM receipt to process exit:

```
1. SIGTERM received (Unix) or CTRL_C_EVENT received (Windows)
2. Root CancellationToken cancelled
3. All plugin run() tasks observe cancellation and return
4. JoinSet drains — all plugin tasks complete
5. shutdown() called on each plugin in reverse-init order
6. RPC server listener closed — no new connections accepted
7. PID file deleted
8. ScRuntime::run() returns Ok(())
9. Process exits with code 0
```

If any plugin's `shutdown()` returns an error, it is logged and shutdown continues to the next plugin. A single plugin failure does not abort the shutdown sequence. The first error encountered is collected and returned from `ScRuntime::run()` as a `ScRuntimeError` with the plugin name in the cause chain.

If the `JoinSet` drain times out (configurable, default 30 seconds), the remaining tasks are cancelled forcibly and shutdown proceeds. This prevents a misbehaving plugin from blocking process exit indefinitely.

---

## Health Aggregation Rules

The `health` built-in command aggregates results from all plugins and all configured storage backends into a single `HealthResult`.

### Plugin Health Contributions

- Plugins that implement `DaemonAware` may expose a `health_check() -> HealthCheck` method. The daemon calls this method on each `DaemonAware` plugin and includes the result in the aggregate.
- Plugins that do not implement `DaemonAware` contribute a synthesized `HealthCheck { name: plugin_name, healthy: true, message: None }` — they are assumed healthy as long as they are running (i.e., their `run()` task has not exited).

### Storage Backend Health Contributions

The daemon calls `StorageBackend::health()` on every configured storage backend. `health()` is infallible — it returns `StorageHealth`, not `Result<StorageHealth, _>`. If a backend cannot determine its health (e.g., no recent successful query), it must return `StorageHealth::Degraded` with an appropriate reason string. A backend must never panic in `health()`.

### Aggregation Rule

```
HealthResult::healthy = true
  if and only if:
    - all plugin HealthCheck::healthy values are true, AND
    - all StorageBackend::health() results are StorageHealth::Healthy
```

A single `Degraded` or `Unavailable` storage backend makes the overall `HealthResult::healthy = false`. A single plugin health check with `healthy: false` makes the overall `HealthResult::healthy = false`. Individual `HealthCheck` and `StorageHealth` entries are always included in the response regardless of the aggregate result, so callers can identify the specific failing component.

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `sc-runtime-core` | `Plugin` trait, `PluginContext`, `PluginError`, `ScRuntimeHome`, `ScRuntimeError` |
| `sc-runtime-cli` | `CommandEnvelope`, `CliError`, `CommandId` — wire format types |
| `tokio` (full) | Async runtime, `JoinSet`, `signal`, Unix socket and named pipe listeners |
| `tokio-util` | `CancellationToken` |
| `libc` | POSIX signal constants, `kill(2)` for Unix liveness check (Unix only) |
| `windows-sys` | `OpenProcess`, `PROCESS_QUERY_LIMITED_INFORMATION` for Windows liveness check (Windows only) |
| `uuid` | `request_id` generation (v4) |
| `serde` / `serde_json` | Request/response serialization |
| `thiserror` | `PluginError` and `ScRuntimeError` derivation |

---

## Lint Compliance

| Rule | Application |
|------|-------------|
| `PORT-010` | Every `#[cfg(unix)]` signal and socket block has a `#[cfg(windows)]` named-pipe/ctrl-c companion |
| `SCB-RUNTIME-001` | Plugin registry uses `Arc<Mutex<...>>` with documented invariants for shared state |
| `SCB-RUNTIME-002` | No `std::sync::Mutex` held across `.await` points — tokio `Mutex` used where async hold is required |
| `sc-lint-boundary` | `sc-runtime-daemon` boundary definition in `boundaries/sc-runtime-daemon/` enforces no reverse dependencies |

Platform-specific transport initialization blocks carry `#[sc_lint(allow = "PORT-010", reason = "paired unix/windows impls in adjacent cfg blocks")]` where the linter cannot statically verify the pairing.
