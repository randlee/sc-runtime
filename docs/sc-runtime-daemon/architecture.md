# `sc-runtime-daemon` Architecture

**Role:** Plugin registry, daemon singleton lifecycle, cross-platform RPC.

---

## Overview

`sc-runtime-daemon` is the process-management and inter-process communication layer for sc-runtime. It owns three concerns that are tightly coupled by design: the plugin registry that orchestrates `Plugin` trait objects, the PID file singleton that enforces single-instance semantics, and the cross-platform RPC server that routes CLI commands to a running daemon.

No domain logic lives here. The daemon starts plugins, keeps them alive, routes requests to them, and shuts them down cleanly. Everything else belongs to consumers.

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

**Init phase** — plugins are initialized in registration order. Each `Plugin::init(&mut self, ctx: &PluginContext)` call is awaited before the next begins. A failure in any plugin's `init` aborts the startup sequence: already-initialized plugins receive `shutdown()` in reverse order before the daemon exits. The error is returned as `ScRuntimeError`.

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

Liveness checking is implemented in `sc-runtime-core` as a cross-platform `pid::is_alive(pid: u32) -> bool`. On Unix this sends signal 0 (`kill(pid, 0)`) — no signal is delivered but the kernel validates the PID. On Windows this uses `OpenProcess` with `SYNCHRONIZE` access and checks the exit code. The daemon does not attempt to parse or validate the previous daemon's state — it only asks "is this PID alive?". If the answer is no, the PID file is stale and is reclaimed.

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

**SIGUSR1** sends a `WakeEvent` to an unbounded channel. Each plugin's `PluginContext` carries a clone of the wake receiver. Plugins that implement wake-aware behavior select on the channel. SIGUSR1 does not terminate the daemon.

Signal handlers are implemented using `tokio::signal::unix::signal` from the tokio async runtime. The `libc` crate is used for signal constant definitions and `kill(2)` in the liveness check.

### Windows: CTRL_C_EVENT

```rust
#[cfg(windows)]
{
    // CTRL_C_EVENT → cancel the root CancellationToken
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        root_cancel.cancel();
    });
    // No SIGUSR1 equivalent — wake channel exists but is never OS-signaled on Windows
}
```

`CTRL_C_EVENT` maps to graceful shutdown via `tokio::signal::ctrl_c()`. This mirrors SIGTERM semantics: the root `CancellationToken` is cancelled, plugins drain, and shutdown proceeds in reverse order.

There is no SIGUSR1 equivalent on Windows. The wake channel exists on all platforms but on Windows it is never signaled by an OS event. Plugins that rely on wake events must treat them as optional — always correct to not receive them.

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

### Request Dispatch

Each accepted connection is handled in a spawned task inside the daemon's `JoinSet`. The request is deserialized, dispatched to the plugin registry's command router, and the response is serialized and written back before the connection closes. Command dispatch follows the same operation layer as direct CLI execution — the transport is transparent.

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

## Dependencies

| Crate | Purpose |
|-------|---------|
| `sc-runtime-core` | `Plugin` trait, `PluginContext`, `PluginError`, `ScRuntimeHome`, `ScRuntimeError`, `pid::is_alive` |
| `sc-runtime-cli` | `CommandEnvelope`, `CliError`, `CommandId` — wire format types |
| `tokio` (full) | Async runtime, `JoinSet`, `signal`, Unix socket and named pipe listeners |
| `tokio-util` | `CancellationToken` |
| `libc` | POSIX signal constants, `kill(2)` for Unix liveness check |
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
