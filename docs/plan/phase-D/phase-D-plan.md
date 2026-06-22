# Phase D: Web + HTTP MCP

**Version target:** 0.1.0-beta.1
**Entry state:** Phase C complete (v0.1.0-alpha.3 tagged)
**Exit state:** axum web server in both modes; HTTP MCP SSE endpoint; full contract parity across all three transport paths

Related: [Project Plan](../project-plan.md) · [Phase C](../phase-C/phase-C-plan.md) · [PRD §14 Phase D](../../prd/sc-runtime-prd.md)

---

## Objectives

By the end of Phase D:

- A consumer can call `.web(WebConfig::default())` on `ScRuntimeBuilder<HasCli>` to run a foreground HTTP server — the process lifetime is the server lifetime, suitable for tools managed by systemd or Docker
- A consumer can call `.web(WebConfig::default())` on `ScRuntimeBuilder<HasDaemon>` to run the web server as a supervised `Plugin` inside the daemon's `JoinSet` — the server starts with the daemon and participates in graceful shutdown
- HTTP MCP is available only on `ScRuntimeBuilder<HasDaemon>`, enforced at compile time — calling `.mcp_http()` on a non-daemon builder is a compile error, not a runtime panic (FR-04, ADR-02)
- axum is a private implementation detail: no axum type appears in any public type exported by `sc-runtime-web` or `sc-runtime` (ADR-08)
- MCP contract parity: CLI `--json`, stdio MCP, and HTTP MCP all produce identical JSON for the same operation (FR-02)

All Phase A, B, and C exit criteria continue to pass.

---

## Crates Delivered

| Crate | Status at end of phase | Complete | Deferred to |
|-------|------------------------|----------|-------------|
| `sc-runtime-web` | v0.1.0-beta.1 | axum server (private), `HttpRouteRegistry` trait (public), `WebConfig`, foreground mode, daemon-supervised mode (implements `Plugin`), HTTP MCP SSE endpoint (`/mcp` path, JSON-RPC 2.0 over SSE) | – |
| `sc-runtime` | v0.1.0-beta.1 | `.web(WebConfig)` on `HasCli` (foreground) and `HasDaemon` (supervised), `.mcp_http(SocketAddr)` on `HasDaemon` only | – |
| `sc-runtime-example` | v0.1.0-beta.1 | Updated with foreground web demo; updated with daemon + supervised web + HTTP MCP demo; contract parity test across all three transport paths | – |

---

## axum Isolation Design

axum is not allowed to appear in any public type surface. The `HttpRouteRegistry` trait uses `axum::Router` as a parameter, which creates a tension: consumers need to register routes, but that requires knowing the axum router type.

Resolution: the `axum` crate is a re-export in `sc-runtime-web` for the purposes of route registration only. Consumers that call `.register()` take a direct `axum = "0.8"` dependency in their own crate. The key constraint is that `sc-runtime-web`'s public API does not expose any axum type in a struct field, return type, or enum variant — only in the `HttpRouteRegistry::register` method signature, where the consumer already has a direct axum dependency.

The `WebConfig` struct, `ScRuntimeBuilder` methods, and `ScRuntime::run()` return type carry no axum types. If axum is replaced in a future version, the change is confined to `sc-runtime-web` and the `HttpRouteRegistry` trait signature — no consumer code outside of route registration callbacks changes.

---

## HTTP MCP SSE Design

The HTTP MCP endpoint follows the same JSON-RPC 2.0 message format as stdio MCP. SSE is used as the response stream because MCP clients connect over HTTP and maintain a persistent connection for receiving server-initiated messages. The sequence for a single request is:

1. Client sends HTTP POST to `/mcp` with a JSON-RPC 2.0 request body
2. Server deserializes the request into the same `RpcRequest` type used by the daemon RPC
3. Server dispatches to the operation layer (identical to CLI and stdio paths)
4. Server streams the response as an SSE `data:` event
5. For operations that produce a single response, the SSE stream closes after one event
6. For future streaming operations, the SSE stream remains open; this phase implements single-response only

The SSE endpoint requires daemon because SSE clients maintain persistent connections that outlive any single CLI invocation (FR-04, ADR-05). Attempting to call `.mcp_http()` on `ScRuntimeBuilder<HasCli>` is rejected at compile time by the typestate system.

---

## Implementation Sequence

### Sprint D-1: axum Skeleton + HttpRouteRegistry + Foreground Mode

**Goal:** Web server runs in foreground mode and serves routes registered by consumers. axum confined to `sc-runtime-web`.

Tasks:
- Create `crates/sc-runtime-web/` with `axum = "0.8"` as a dependency
- Define `HttpRouteRegistry` trait:

  ```rust
  pub trait HttpRouteRegistry: Send + Sync {
      fn register(&self, router: axum::Router) -> axum::Router;
  }
  ```

- Define `WebConfig { bind: SocketAddr, registry: Option<Arc<dyn HttpRouteRegistry>> }` — `Default` impl binds to `127.0.0.1:3000`
- Implement `WebServer` (private type): holds the axum `Router`, binds to `WebConfig.bind`, serves via `axum::serve`
- Foreground mode: `WebServer::run_foreground(config, cancel: CancellationToken) -> Result<(), WebError>` — runs until cancellation token fires or server errors; shuts down via `axum::serve(...).with_graceful_shutdown()`
- Add `.web(WebConfig) -> Self` to `ScRuntimeBuilder<HasCli>` — stores config, triggers foreground mode in `ScRuntime::run()`
- Tests:
  - Server starts, HTTP GET to a registered route returns 200
  - Server shuts down cleanly when `CancellationToken` is cancelled
  - No axum type in public API: `static_assertions` test asserting that `WebConfig`, `WebError`, and `ScRuntimeBuilder<HasCli>` contain no axum types in their public surface
- PORT-010: verify no platform-specific binding code without companions; TCP bind is portable

Deliverable: Foreground web server working. axum isolation verified.

### Sprint D-2: Daemon-Supervised Mode

**Goal:** Web server runs as a `Plugin` inside the daemon's `JoinSet`. Participates in graceful shutdown sequence.

Tasks:
- Implement `WebServerPlugin` (in `sc-runtime-web/src/plugin.rs`): implements `Plugin` trait
  - `init()`: validates config, prepares the axum router with registered routes
  - `run(cancel)`: calls `WebServer::run_foreground()` with the provided cancellation token — the daemon's cancellation propagates naturally
  - `shutdown()`: signals graceful shutdown; waits for active connections to drain (configurable timeout, default 10 seconds)
  - `metadata()`: returns `PluginMetadata { name: "sc-runtime-web", version: env!("CARGO_PKG_VERSION") }`
- Add `.web(WebConfig) -> Self` to `ScRuntimeBuilder<HasDaemon>`: registers `WebServerPlugin` via the daemon's plugin registry instead of running foreground
- Integration test: daemon starts with `WebServerPlugin`; test sends HTTP request while daemon is running; daemon receives SIGTERM simulation; test verifies `WebServerPlugin::shutdown()` is called before process exits and active connections drain
- Verify `sc-lint-boundary`: `sc-runtime-web` depends on `sc-runtime-daemon` for `Plugin` trait only — explicit in `Boundary.toml`

Deliverable: Supervised mode working. Web server participates in daemon shutdown sequence.

### Sprint D-3: HTTP MCP SSE Endpoint

**Goal:** JSON-RPC 2.0 over SSE at `/mcp`. Requires daemon. Same operation dispatch as CLI and stdio paths.

Tasks:
- Add `/mcp` route to the axum router inside `WebServerPlugin` — this route is always registered when `WebServerPlugin` is used; it is not consumer-configurable
- SSE handler: receives HTTP POST with JSON-RPC 2.0 body; deserializes to `RpcRequest`; dispatches to the shared operation layer; serializes response as `CommandEnvelope`; returns via SSE `data:` event stream using `axum::response::Sse`
- Connection lifecycle: each SSE connection runs as a `JoinSet` task inside `WebServerPlugin`; connections are cancelled when the plugin receives its cancellation token
- Add `.mcp_http(addr: SocketAddr) -> Self` to `ScRuntimeBuilder<HasDaemon>` only — configures the `WebServerPlugin` to start the SSE endpoint on the specified address; calling this on `ScRuntimeBuilder<HasCli>` is a compile error enforced by typestate (FR-04)
- Compile-fail test: a test binary that calls `.mcp_http()` on a `HasCli` builder must fail to compile — verified via `trybuild` or equivalent compile-fail test harness
- SSE connection management: track active SSE connections; on `shutdown()`, close all SSE streams before returning; give active streams up to 5 seconds to close naturally before forcing closure

Deliverable: HTTP MCP SSE endpoint working. `.mcp_http()` compile error on `HasCli` verified.

### Sprint D-4: Contract Parity Tests + Facade Finalization

**Goal:** Three transport paths (CLI `--json`, stdio MCP, HTTP MCP) produce identical JSON for representative operations. Phase D facade complete.

Tasks:
- Define 5 representative operation fixtures (one per built-in command: `start`, `stop`, `restart`, `status`, `health`) with known request inputs and expected `CommandEnvelope` outputs
- Contract parity test suite:
  - Run each fixture against the CLI `--json` path
  - Run each fixture against the stdio MCP path (send JSON-RPC 2.0 request, receive response)
  - Run each fixture against the HTTP MCP SSE path (POST to `/mcp`, receive SSE event)
  - Assert that the `data` and `error` fields of the `CommandEnvelope` are byte-identical across all three paths for each fixture
- axum public API surface test: `grep` the compiled symbol list of `sc-runtime-web` and `sc-runtime` for any `axum::` prefixed type — must return zero matches in public items
- Update `sc-runtime-example`: add a demo showing the full stack (`HasDaemon` + storage + supervised web + HTTP MCP) and a demo showing foreground web (no daemon); both examples included in `--help` output
- Update `boundaries/sc-runtime-web/Boundary.toml` to finalize the `sc-runtime-daemon` feature-gated dependency rule
- Run `cargo test --workspace`; run `sc-lint lint fast`

Deliverable: Phase D complete. Tag `v0.1.0-beta.1`.

---

## Exit Criteria

All of the following must be true before Phase E begins:

- `cargo test --workspace` passes, including all web and HTTP MCP tests
- `sc-lint lint fast` passes: boundary rules clean for `sc-runtime-web`; PORT-010 satisfied; no new `#[cfg(unix)]` without Windows companions
- Foreground mode: web server serves registered routes; shuts down cleanly when `CancellationToken` fires (simulated SIGTERM in test); open connections drain within the configured timeout
- Supervised mode: `WebServerPlugin` initializes, runs, and shuts down as part of the daemon plugin sequence; shutdown is called in reverse-init order relative to other plugins; integration test confirms ordering
- `.mcp_http()` on `ScRuntimeBuilder<HasCli>` is a compile error — verified by a `trybuild` compile-fail test; the error message is human-readable, not just a type mismatch
- Contract parity: 5 representative operation fixtures produce byte-identical `data` and `error` field content across CLI `--json`, stdio MCP, and HTTP MCP SSE paths — verified by the parity test suite
- axum appears in zero public types in `sc-runtime-web` and `sc-runtime` — verified by API surface test
- All Phase A, B, and C exit criteria still pass
- Git tag `v0.1.0-beta.1` applied to the passing commit
