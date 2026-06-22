# `sc-runtime-web` Architecture

**Role:** HTTP transport layer. MVP uses axum. The HTTP framework is not exposed in any public type.

---

## Overview

`sc-runtime-web` provides the HTTP transport for sc-runtime tools. It operates in one of two modes depending on whether the daemon layer is present. In either mode, the consumer registers routes against the operation layer and `sc-runtime-web` owns the HTTP server. axum is the MVP implementation and is a private dependency — the HTTP framework does not appear in any public type.

---

## Operating Modes

### Foreground Mode (without daemon)

When the daemon layer is not configured, `sc-runtime-web` runs the web server directly inside `ScRuntime::run()`. The process lifetime equals the server lifetime. The server starts when `run()` is called and the process exits when the server stops.

This mode is appropriate for tools that are HTTP-first and managed externally — by systemd, Docker, or a process supervisor — where there is no need for a background daemon. stdio MCP and foreground web can be combined; the foreground web server is the process.

```
ScRuntime::run()
  └── axum server (foreground)
        └── registered routes
```

The consumer selects foreground mode by calling `.web(config)` on a `ScRuntimeBuilder<HasCli>` without calling `.daemon()`.

### Daemon-Supervised Mode (with daemon)

When the daemon layer is configured, `sc-runtime-web` implements the `Plugin` trait and runs inside the daemon's `JoinSet` as a supervised task. The web server starts with the daemon's init phase, receives the shared `CancellationToken`, and shuts down gracefully when the daemon's shutdown sequence begins.

```
daemon JoinSet
  ├── plugin_a::run(cancel)
  ├── sc-runtime-web Plugin::run(cancel)   ← axum server inside a Plugin
  └── plugin_c::run(cancel)
```

The `Plugin::run` implementation starts the axum server and selects on the `CancellationToken`. When the token is cancelled, the server begins a graceful drain — in-flight requests are allowed to complete and the listener stops accepting new connections before `Plugin::run` returns.

The consumer selects supervised mode by calling `.web(config)` on a `ScRuntimeBuilder<HasDaemon>`.

---

## HTTP MCP (SSE)

The HTTP MCP endpoint delivers JSON-RPC 2.0 messages over Server-Sent Events (SSE). This is the streaming MCP transport.

**Requires daemon.** SSE clients maintain persistent connections. The server must be running independently of any CLI invocation. This requirement is enforced at compile time: `mcp_http()` is only available on `ScRuntimeBuilder<HasDaemon>`. Calling it on a non-daemon builder is a compile error.

**Same message format as stdio MCP.** The JSON-RPC 2.0 message envelope is identical to the stdio transport. The same JSON fixtures used in stdio MCP tests must pass against the HTTP MCP endpoint. There is no HTTP-specific envelope shape.

**SSE endpoint contract:**

- `GET /mcp/sse` — establishes the SSE stream; the server sends JSON-RPC 2.0 messages as SSE events
- `POST /mcp/message` — client sends JSON-RPC 2.0 requests; the server processes and delivers the response via the SSE stream

The request body at `POST /mcp/message` is deserialized into the same `CommandEnvelope` request types used by the CLI. The response is serialized using the same types and delivered as an SSE event on the client's open stream.

---

## HttpRouteRegistry Trait

The `HttpRouteRegistry` trait is the seam between consumer operations and the axum router. The consumer implements this trait to register routes; `sc-runtime-web` calls it to construct the server.

```rust
pub trait HttpRouteRegistry: Send + Sync {
    fn register(&self, router: axum::Router) -> axum::Router;
}
```

The consumer receives the axum `Router`, adds their routes, and returns the extended router. `sc-runtime-web` owns the server binding, TLS (if configured), and the serve loop. The consumer owns the route handlers and the operations they invoke.

**axum is private.** The `axum::Router` parameter in `HttpRouteRegistry::register` is the only place axum appears at the trait boundary, and this trait is not public API — it is passed through the builder as `impl HttpRouteRegistry`. Consumers that do not implement their own route registry interact only with `WebConfig` and the builder methods. The public types `WebConfig`, `WebMode`, and the builder methods are axum-free.

Replacing axum is a single-crate change: update `sc-runtime-web`'s dependencies and the internal server binding code. The `HttpRouteRegistry` trait signature changes only if the replacement framework's router type differs, and that change is isolated to the one trait definition.

---

## Route Contract

HTTP handlers in `sc-runtime-web` are thin transport adapters. They have one job: deserialize the request body into the operation layer's input type, call the operation, and serialize the result.

```
HTTP request body
  → deserialize into CommandEnvelope request type
  → operation layer (same function as CLI handler)
  → serialize CommandEnvelope response
  → HTTP response body
```

There are no HTTP-specific DTOs. The request and response types are exactly the types defined in the consumer's `{tool}-types` crate and used by the CLI and daemon RPC. This is the MCP-compatible seam applied to HTTP: every transport variation is a serialization boundary, not a type boundary.

**MCP contract parity** is enforced by the test suite. The same JSON request fixtures that pass against the CLI `--json` path and the stdio MCP path must produce identical responses against the HTTP MCP path. A fixture that fails on one path and passes on another is a contract violation, not an HTTP issue.

---

## Framework Isolation

axum is not exposed in any public type of `sc-runtime-web`. This is a hard architectural constraint, not a soft preference.

| Surface | axum visible? |
|---------|:-------------:|
| `WebConfig` | no |
| `WebMode` | no |
| `HttpRouteRegistry` trait | yes — `axum::Router` in `register()` signature |
| Builder `.web()` / `.mcp_http()` | no |
| `Plugin` impl for supervised mode | no (internal) |
| Any re-export from `sc-runtime` facade | no |

The `HttpRouteRegistry` trait is the intentional exception. Consumers that register routes must name the router type. All other public surfaces are axum-free.

Future work: if a consumer wants full axum independence at the route-registration level (e.g., they want to build routes with a different abstraction), a route-description DSL can be introduced as an alternative to `HttpRouteRegistry`. This is a non-breaking addition — the existing `HttpRouteRegistry` remains.

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `sc-runtime-core` | `Plugin` trait, `PluginContext`, `PluginError`, `ScRuntimeError` |
| `sc-runtime-cli` | `CommandEnvelope`, `CliError` — wire format types shared with HTTP handlers |
| `sc-runtime-daemon` | `Plugin` registration, `JoinSet` supervision — required for supervised mode only |
| `axum` (0.8) | HTTP server, routing, SSE — private implementation detail |
| `tokio` (full) | Async runtime, server task execution |
| `serde` / `serde_json` | Request/response serialization for HTTP handlers |
| `thiserror` | Error type derivation |

`sc-runtime-daemon` is a conditional dependency. When `sc-runtime-web` is used in foreground mode only, the daemon crate is not required. The supervised mode and HTTP MCP endpoint require the daemon layer — this is enforced by the typestate builder in `sc-runtime`.
