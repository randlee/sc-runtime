# Phase C: Daemon + RPC

**Version target:** 0.1.0-alpha.3
**Entry state:** Phase B complete (v0.1.0-alpha.2 tagged)
**Exit state:** Full daemon lifecycle with cross-platform RPC; CLI router end-to-end

Related: [Project Plan](../project-plan.md) · [Phase B](../phase-B/phase-B-plan.md) · [PRD §14 Phase C](../../prd/sc-runtime-prd.md)

---

## Objectives

By the end of Phase C, a consumer can call `.daemon()` on the builder and get:

- A singleton daemon enforced by PID file, with automatic stale PID reclamation (FR-07)
- A structured plugin registry that runs plugins concurrently under a `JoinSet`, isolates panics, and shuts down in reverse-init order on SIGTERM (FR-08)
- Cross-platform signal handling: `SIGTERM` and `SIGUSR1` on Unix; `CTRL_C_EVENT` and named event on Windows; every `#[cfg(unix)]` block paired with a `#[cfg(windows)]` companion (PORT-010, NF-03, FR-09)
- A cross-platform RPC server accepting `CommandEnvelope` requests over Unix socket (macOS/Linux), named pipe (Windows), or TCP loopback fallback, responding with the same `CommandEnvelope` types used by the CLI (FR-10)
- A completed CLI command router: when a daemon socket or pipe is reachable, the router forwards requests over IPC; when it is not, it executes directly — transparent to the caller (FR-11)

All Phase A and Phase B exit criteria continue to pass.

---

## Crates Delivered

| Crate | Status at end of phase | Complete | Deferred to |
|-------|------------------------|----------|-------------|
| `sc-runtime-daemon` | v0.1.0-alpha.3 | Plugin registry (init/run/shutdown, `JoinSet`, panic isolation, reverse-shutdown drain), PID file singleton with stale detection, SIGTERM/SIGUSR1 (Unix) + CTRL_C_EVENT (Windows), RPC server (Unix socket + named pipe + TCP loopback), RPC client | – |
| `sc-runtime` | v0.1.0-alpha.3 | `.daemon()` typestate transition `HasCli→HasDaemon`, `.plugin()` on `HasDaemon`, signal type re-exports | `.mcp_http()`, `.web()` supervised mode (Phase D) |
| `sc-runtime-cli` | v0.1.0-alpha.3 | Command router IPC path complete: detects socket/pipe, serializes `CommandEnvelope` request, receives response, falls back to direct execution on connection refused | – |
| `sc-runtime-example` | v0.1.0-alpha.3 | Updated to exercise daemon start/stop, RPC round-trip, plugin registration, graceful shutdown with artificial delay | – |

---

## Cross-Platform Design Constraints

Every piece of daemon and RPC code that is platform-specific must satisfy PORT-010: every `#[cfg(unix)]` block must have a `#[cfg(windows)]` companion or a portable fallback. There are no exceptions. Where the Windows implementation is not yet verified on a real Windows host, the `#[sc_lint(allow = "PORT-010", reason = "paired unix/windows impls in adjacent cfg blocks")]` attribute documents the pairing explicitly.

The three-transport RPC design exists precisely to make this tractable:

| Transport | Platform | Path |
|-----------|----------|------|
| Unix domain socket | macOS, Linux | `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.sock` |
| Named pipe | Windows | `\\.\pipe\sc-{tool}` |
| TCP loopback | All (fallback) | Port registered in `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.port` |

The wire format is identical on all three: newline-delimited JSON, one request line per connection, one response line. The `CommandEnvelope<T>` and `CliError` types are the transport — no transport-specific DTOs are introduced (FR-10, ADR-06).

---

## Plugin Registry Design Notes

The plugin registry is the central coordination point for the daemon. Its invariants:

- **Init order is preserved.** Plugins are initialized in registration order. Each plugin's `init()` completes before the next plugin's `init()` begins.
- **Run is concurrent.** After all plugins are initialized, each plugin's `run()` is spawned as a `JoinSet`-owned tokio task. Tasks are region-owned — no detached tasks.
- **Cancellation is via token.** The daemon's `CancellationToken` is passed to each `run()` call. Plugins must cooperate with cancellation; the runtime cannot forcibly terminate a plugin that ignores the token.
- **Panic isolation.** If a plugin's `run()` task panics, the panic is caught by the `JoinSet`, converted to `PluginError::Panic { name, message }`, and logged. Other plugins continue running.
- **Shutdown is reverse-init order.** On SIGTERM (or equivalent), the daemon waits for the `CancellationToken` to propagate, then calls `shutdown()` on each plugin in reverse registration order. A configurable drain timeout (default: 30 seconds) applies per plugin; plugins that exceed it are logged as forcibly dropped.
- **Interior mutability with documented invariant.** The registry uses `Arc<Mutex<Vec<Box<dyn Plugin>>>>` for the plugin list. The `Mutex` is held only during registration and shutdown phases, not during concurrent run. `RwLock` is preferred for state read during health checks (RBP-006).

`SCB-RUNTIME-001` and `SCB-RUNTIME-002` from `sc-lint-runtime` apply directly to the registry concurrency code. The lint gate verifies correctness properties at the source level.

---

## Implementation Sequence

### Sprint C-1: PID File Singleton + Signal Handling

**Goal:** Daemon singleton enforcement and cross-platform signal handling. No plugin registry yet.

Tasks:
- Implement PID file logic in `sc-runtime-daemon/src/pid.rs`:
  - `PidFile::acquire(home: &ScRuntimeHome, tool: &str) -> Result<PidFile, DaemonError>`: writes PID to `{home}/.sc/runtime/{tool}.pid`; if file exists, reads the PID and checks liveness; if alive, returns `DaemonError::AlreadyRunning`; if stale (process dead), overwrites
  - PID liveness check: on Unix, `kill(pid, 0)` via `libc`; on Windows, `OpenProcess` + `GetExitCodeProcess` via `windows-sys`; must be paired `#[cfg(unix)]` / `#[cfg(windows)]` (PORT-010)
  - `PidFile::release()`: deletes the PID file on clean exit; called in `Drop` impl
- Implement signal handling in `sc-runtime-daemon/src/signals.rs`:
  - Unix: `tokio::signal::unix::signal(SignalKind::terminate())` for SIGTERM; `SignalKind::user_defined1()` for SIGUSR1
  - Windows: `tokio::signal::ctrl_c()` for CTRL_C_EVENT; named event stub for SIGUSR1 equivalent (no Windows equivalent — documented no-op with comment explaining Windows event object approach for future)
  - Signal handler sends to a `tokio::sync::watch::Sender<SignalEvent>` channel; daemon main loop selects on the receiver
- Tests:
  - PID acquire with no existing file: succeeds, file contains current PID
  - PID acquire with live PID: returns `DaemonError::AlreadyRunning` — tested by writing the current test process's own PID
  - PID acquire with stale PID (dead process): succeeds, overwrites file — tested by writing a PID known not to exist (e.g., `u32::MAX`)
  - Drop impl: `PidFile` dropped, file deleted
  - PORT-010 compliance: `sc-lint lint fast` passes on the new platform-specific code

Deliverable: PID singleton and signal infrastructure ready. sc-lint passes.

### Sprint C-2: Plugin Registry

**Goal:** Plugin registry fully implemented. Daemon can start, run plugins concurrently, and shut down cleanly.

Tasks:
- Implement `PluginRegistry` in `sc-runtime-daemon/src/registry.rs`:
  - `register(&mut self, plugin: Box<dyn Plugin>)` — appends to ordered vec; registration order preserved
  - `init_all(&mut self, ctx: &PluginContext) -> Result<(), RegistryError>` — sequential init, short-circuits on first error
  - `run_all(&mut self, cancel: CancellationToken) -> JoinSet<Result<(), PluginError>>` — spawns each plugin's `run()` as a named task in a `JoinSet`
  - `shutdown_all(&mut self) -> Result<(), RegistryError>` — calls `shutdown()` in reverse-init order; per-plugin 30s drain timeout
  - Panic isolation: `JoinSet` catch: `tokio::task::JoinError::is_panic()` → `PluginError::Panic`
- Integrate with signal handling: SIGTERM received → propagate `CancellationToken` → await `JoinSet` completion → call `shutdown_all()`
- Integration tests:
  - Three plugins: init called in order 1→2→3, shutdown called in order 3→2→1
  - Plugin that panics in `run()`: panic caught, converted to `PluginError::Panic`, other plugins unaffected
  - Plugin that ignores cancellation: drain timeout fires, plugin logged as forcibly dropped
  - SIGTERM simulation: send simulated signal event, assert graceful shutdown sequence completes
- Verify `sc-lint-runtime` SCB-RUNTIME-001 and SCB-RUNTIME-002 pass on the registry code

Deliverable: Plugin registry fully tested. Daemon lifecycle works end-to-end without RPC.

### Sprint C-3: RPC Transport

**Goal:** RPC server listens on all three transports. Client connects and round-trips a `CommandEnvelope`. Wire format identical across transports.

Tasks:
- Implement RPC server in `sc-runtime-daemon/src/rpc/server.rs`:
  - Unix path: `tokio::net::UnixListener` bound to `{home}/.sc/runtime/{tool}.sock` — gated `#[cfg(unix)]`
  - Windows path: `tokio::net::windows::named_pipe::ServerOptions` bound to `\\.\pipe\sc-{tool}` — gated `#[cfg(windows)]`
  - TCP fallback: `tokio::net::TcpListener` on `127.0.0.1:0` (random port); port written to `{home}/.sc/runtime/{tool}.port`
  - Per-connection handler: reads one line from the socket, deserializes `RpcRequest { version: u32, request_id: Uuid, command: CommandId, payload: serde_json::Value }`, dispatches to command operation layer, serializes `CommandEnvelope` response, writes one line, closes connection
  - Concurrent connections: each connection spawned as a `JoinSet` task inside the RPC server's own supervised region
- Implement RPC client in `sc-runtime-daemon/src/rpc/client.rs`:
  - Probes for socket/pipe/port file in priority order; returns `Ok(None)` gracefully if none found (not an error — daemon not running)
  - Sends `RpcRequest`, receives `CommandEnvelope` response
  - `request_id` correlation: asserts response `request_id` matches request
- Complete CLI command router IPC path in `sc-runtime-cli`: inject `RpcClient` via transport adapter trait; when `connect()` returns `Some(client)`, serialize and forward; when `None`, execute directly (FR-11)
- Tests:
  - Server starts on all three transports; client round-trip on each
  - Concurrent requests: 10 simultaneous clients; `request_id` correlation verified for all 10
  - Client returns `Ok(None)` when no server running (connection refused)
  - PORT-010 check: sc-lint clean on the socket/pipe code

Deliverable: RPC transport working on all three paths. Wire format verified. CLI router complete.

### Sprint C-4: Facade `.daemon()` Typestate + Integration Test

**Goal:** Consumer-facing builder exposes the daemon transition. `sc-runtime-example` exercises the full daemon lifecycle end-to-end.

Tasks:
- Add `.daemon() -> ScRuntimeBuilder<HasDaemon>` to `ScRuntimeBuilder<HasCli>` (RBP-002, RBP-010): transitions typestate, moves all `HasCli` configuration into `HasDaemon` state
- Add `.plugin(impl Plugin + 'static) -> Self` to `ScRuntimeBuilder<HasDaemon>`
- Stub `.mcp_http()` and supervised `.web()` on `HasDaemon` with "not yet available — Phase D" compile diagnostics
- Re-export signal types from `sc-runtime`: `SignalEvent`, `DaemonError`, `PluginError` so consumers do not need a direct `sc-runtime-daemon` dependency
- Update `sc-runtime-example`: add a daemon configuration with two plugins (a logging plugin and a counter plugin); exercise start → register → run → shutdown sequence; send an RPC `status` request while daemon is running and assert the response
- Integration test: spawn the example daemon in a subprocess; send an RPC status request from the test process; assert `CommandEnvelope<StatusResult>` response; send simulated SIGTERM; assert daemon exits cleanly
- Run `cargo test --workspace`; run `sc-lint lint fast`; verify PORT-010 on all `#[cfg(unix)]` blocks

Deliverable: Phase C complete. Tag `v0.1.0-alpha.3`.

---

## Exit Criteria

All of the following must be true before Phase D begins:

- `cargo test --workspace` passes, including all daemon and RPC tests
- `sc-lint lint fast` passes: boundary rules clean, PORT-010 satisfied on all `#[cfg(unix)]` blocks in daemon code (every unix-specific block has a Windows companion), SCB-RUNTIME-001 and SCB-RUNTIME-002 pass on registry concurrency code
- Daemon singleton: second invocation of the example binary while the daemon is running returns `DAEMON.ALREADY_RUNNING` via `CommandEnvelope<StartResult>` with `ok: false` — verified by test
- Stale PID: writing a PID that belongs to a dead process, then calling `PidFile::acquire()`, succeeds and overwrites — verified by test
- Graceful shutdown: two plugins with artificial shutdown delays complete their `shutdown()` in reverse-init order before the process exits; total time is bounded by the drain timeout — verified by integration test
- RPC roundtrip: client round-trip verified on all three transport types (Unix socket, named pipe on Windows or skipped with `#[cfg(not(unix))]` documentation, TCP loopback always)
- PORT-010: `sc-lint lint fast` confirms every `#[cfg(unix)]` block in `sc-runtime-daemon` has a Windows companion or portable fallback
- Phase A and Phase B exit criteria still pass: all previous tests green, facade builder still compiles with Phase A and B configurations
- Git tag `v0.1.0-alpha.3` applied to the passing commit
