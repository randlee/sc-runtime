# `sc-runtime-web` Requirements

---

## Scope

`sc-runtime-web` is responsible for:

- HTTP server lifecycle in foreground mode (without daemon)
- HTTP server lifecycle in daemon-supervised mode (as a `Plugin`)
- HTTP MCP SSE endpoint (daemon-supervised only)
- `HttpRouteRegistry` trait for consumer route registration
- Keeping axum as a private implementation detail

`sc-runtime-web` is not responsible for:

- Operation logic — consumers own their route handlers
- Authentication, rate limiting, or multi-tenancy
- CLI argument parsing (owned by `sc-runtime-cli`)
- Daemon lifecycle (owned by `sc-runtime-daemon`)
- TLS certificate management beyond accepting a `TlsConfig` from the consumer

---

## Functional Requirements

### Operating Modes

**FR-WEB-01** — `sc-runtime-web` must operate in foreground mode: the HTTP server runs directly inside `ScRuntime::run()`, and the process lifetime equals the server lifetime. Foreground mode is available without the daemon layer.

**FR-WEB-02** — `sc-runtime-web` must operate in daemon-supervised mode: the HTTP server implements the `Plugin` trait and runs inside the daemon's `JoinSet` as a supervised task. Supervised mode requires the daemon layer.

**FR-WEB-03** — In daemon-supervised mode, the web server `Plugin::run()` must observe the `CancellationToken` from `PluginContext` and begin a graceful drain — allowing in-flight requests to complete and stopping new connection acceptance — before returning from `Plugin::run()`.

### HTTP MCP SSE Endpoint

**FR-WEB-04** — The HTTP MCP SSE endpoint must only be available in daemon-supervised mode. Configuring `mcp_http()` without the daemon layer must be a compile error (enforced by the typestate builder in `sc-runtime`).

**FR-WEB-05** — The SSE endpoint must use JSON-RPC 2.0 message format. The message envelope must be identical to the stdio MCP transport — no HTTP-specific envelope shape.

**FR-WEB-06** — The same JSON request fixtures used in CLI `--json` tests and stdio MCP tests must produce identical responses against the HTTP MCP SSE endpoint. A fixture that produces a different response on the HTTP path is a contract violation.

### Route Contract

**FR-WEB-07** — HTTP route handlers must deserialize request bodies into the same `CommandEnvelope` request types used by the CLI and daemon RPC. There must be no HTTP-specific DTOs at the handler boundary.

**FR-WEB-08** — HTTP route handlers must serialize responses using the same `CommandEnvelope` types. The response shape seen by an HTTP client must be identical to the response shape seen by a CLI `--json` caller for the same operation.

**FR-WEB-09** — The `HttpRouteRegistry` trait must be the mechanism by which consumers register routes. `sc-runtime-web` must own the server binding and serve loop; the consumer must own the route handlers.

### Framework Isolation

**FR-WEB-10** — axum must not appear in any public type of `sc-runtime-web` other than the `axum::Router` parameter in `HttpRouteRegistry::register()`. `WebConfig`, `WebMode`, and all builder-facing types must be axum-free.

**FR-WEB-11** — Replacing axum with a different HTTP framework must require changes only within `sc-runtime-web`. No consumer-facing API change may be required to replace the HTTP framework.

---

## Non-Functional Requirements

**NF-WEB-01** — HTTP framework replaceability: axum is the MVP implementation. The constraint that axum is a private implementation detail (FR-WEB-10, FR-WEB-11) must be maintained for the lifetime of the `sc-runtime-web` 0.x series.

**NF-WEB-02** — `sc-runtime-web` must compile for macOS, Linux, and Windows. The `cargo xwin check` and `cargo xwin clippy` targets must pass in the `full` lint profile. Any platform-specific server-binding code must have `#[cfg(unix)]` / `#[cfg(windows)]` companions (PORT-010).

**NF-WEB-03** — The `sc-runtime-web` boundary definition in `boundaries/sc-runtime-web/` must pass `sc-lint-boundary` in CI.

**NF-WEB-04** — `unsafe_code = "forbid"` applies to `sc-runtime-web`. Any unsafe block requires a documented justification.
