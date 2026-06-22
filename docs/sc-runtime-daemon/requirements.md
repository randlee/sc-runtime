# `sc-runtime-daemon` Requirements

---

## Scope

`sc-runtime-daemon` is responsible for:

- Plugin registry: init, run, and shutdown orchestration
- Daemon singleton enforcement via PID file
- Cross-platform signal handling (SIGTERM, SIGUSR1, CTRL_C_EVENT)
- Cross-platform RPC transport: Unix socket, named pipe, TCP fallback
- Structured concurrency for all plugin tasks

`sc-runtime-daemon` is not responsible for:

- Domain logic of any kind
- CLI argument parsing (owned by `sc-runtime-cli`)
- Storage backend management (owned by `sc-runtime-db-*`)
- HTTP transport (owned by `sc-runtime-web`)
- Log construction (injected by the consumer via `sc-runtime` builder)

---

## Functional Requirements

### Singleton Enforcement

**FR-DAEMON-01** — The daemon must enforce single-instance semantics via a PID file at `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.pid`. A second instance attempting to start must detect the running daemon and exit with error code `DAEMON.ALREADY_RUNNING` without initializing any plugins.

**FR-DAEMON-02** — A stale PID file (the stored PID refers to a dead process) must be reclaimed automatically on startup. The daemon must overwrite the stale file and proceed with normal initialization. No user intervention is required.

**FR-DAEMON-03** — The PID file must be deleted as the last step before process exit, after all plugin `shutdown()` calls complete. Deletion failure must be logged but must not affect the process exit code.

### Signal Handling

**FR-DAEMON-04** — On Unix, receipt of `SIGTERM` must trigger graceful shutdown: the root `CancellationToken` is cancelled, all plugin `run()` tasks are allowed to drain, `shutdown()` is called in reverse-init order, the PID file is deleted, and the process exits with code 0.

**FR-DAEMON-05** — On Unix, receipt of `SIGUSR1` must send a `WakeEvent` to all plugins via the wake channel without terminating the daemon. The daemon must continue running after delivering the wake event.

**FR-DAEMON-06** — On Windows, `CTRL_C_EVENT` must map to graceful shutdown with identical semantics to SIGTERM on Unix (FR-DAEMON-04). `CTRL_C_EVENT` must cancel the root `CancellationToken` and trigger the full shutdown sequence.

**FR-DAEMON-07** — There is no SIGUSR1 equivalent on Windows. The wake channel must exist on all platforms, but on Windows it must never be signaled by an OS event. Plugins must treat wake events as optional — their absence must be correct behavior.

**FR-DAEMON-08** — Every `#[cfg(unix)]` signal-handling block must have a `#[cfg(windows)]` companion block. PORT-010 must pass in CI.

### RPC Transport

**FR-DAEMON-09** — The daemon must start an RPC server on the platform-appropriate transport:
- macOS / Linux: Unix domain socket at `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.sock`
- Windows: Named pipe at `\\.\pipe\sc-{tool}`
- Fallback (all platforms): TCP loopback, port registered in `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.port`

**FR-DAEMON-10** — The RPC wire protocol must be newline-delimited JSON on all transports. The same `CommandEnvelope<T>` types used by the CLI must be used on all transports. There must be no transport-specific DTOs.

**FR-DAEMON-11** — Every RPC request must carry a `version` integer field and a `request_id` UUID string field. The daemon must reject requests with a `version` higher than it understands and return a `DAEMON.PROTOCOL_VERSION` error. The `request_id` must be echoed in the response.

**FR-DAEMON-12** — The RPC server must close its listener and stop accepting new connections before the PID file is deleted during shutdown.

### Plugin Lifecycle

**FR-DAEMON-13** — Plugins must be initialized in registration order. A failure in any plugin's `init()` must abort the startup sequence. Already-initialized plugins must receive `shutdown()` in reverse order before the daemon exits.

**FR-DAEMON-14** — All plugin `run()` tasks must execute concurrently under a single `JoinSet`. There must be no detached `tokio::spawn` calls in the plugin registry.

**FR-DAEMON-15** — Plugin `shutdown()` calls must be made in reverse-init order after all `run()` tasks have drained from the `JoinSet`.

**FR-DAEMON-16** — A plugin panic inside `run()` must be caught at the `JoinSet` boundary and converted to `PluginError::Panic`. Plugin panics must not propagate to `ScRuntime::run()`. The daemon must continue operating with remaining plugins after a plugin panic.

**FR-DAEMON-17** — `ScRuntime::run()` must return `Result<(), ScRuntimeError>`. It must never panic and must never have return type `!`.

**FR-DAEMON-18** — If the `JoinSet` drain during shutdown exceeds the configurable drain timeout (default 30 seconds), the remaining tasks must be forcibly cancelled and shutdown must proceed. The daemon must not block indefinitely on a misbehaving plugin.

---

## Non-Functional Requirements

**NF-DAEMON-01** — Every `#[cfg(unix)]` block in `sc-runtime-daemon` must have a `#[cfg(windows)]` companion block or a portable fallback. This is enforced by the `PORT-010` lint rule in `sc-lint-portability` and must pass in CI.

**NF-DAEMON-02** — `SCB-RUNTIME-001` and `SCB-RUNTIME-002` lint rules from `sc-lint-runtime` must pass on all plugin registry concurrency code. `std::sync::Mutex` must not be held across `.await` points.

**NF-DAEMON-03** — The `sc-runtime-daemon` boundary definition in `boundaries/sc-runtime-daemon/` must pass `sc-lint-boundary` in CI. `sc-runtime-daemon` must not be a dependency of `sc-runtime-mcp-stdio`.

**NF-DAEMON-04** — `sc-runtime-daemon` must compile for macOS, Linux, and Windows. The `cargo xwin check` and `cargo xwin clippy` targets must pass in the `full` lint profile.

**NF-DAEMON-05** — `unsafe_code = "forbid"` applies to `sc-runtime-daemon`. Any unsafe block requires a documented justification and a reviewer-approved `#[sc_lint(allow = "...")]` annotation.
