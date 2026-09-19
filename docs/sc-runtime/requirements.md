# `sc-runtime` Requirements

Crate-level requirements. Repo-level requirements: [`../requirements.md`](../requirements.md).
Source: [`../sc-runtime-design.md`](../sc-runtime-design.md). Every id is binding and is never reused.

## Purpose

The assembled daemon: a bootstrap that builds the parts and hands them to the
project.

## Requirements

- `REQ-RUNTIME-001` `Daemon::builder(app, &daemon_config)` accepts a stores
  closure, a routes closure and an optional MCP closure, and `run()` performs,
  in this order: take the already-loaded config; resolve the instance root and
  take the `daemon.lock` singleton; call the stores closure; merge the
  project's router, the OpenAPI document route, a health route and the MCP
  service if one was given; bind the listener through `sc-transport` and
  serve.
- `REQ-RUNTIME-002` The singleton is an OS exclusive lock on
  `<instance-root>/daemon.lock`. A second daemon on the same instance root
  fails with a typed error before it opens any store or binds any listener.
- `REQ-RUNTIME-003` The project's `Stores` value is opaque: the crate is
  generic over it, never inspects it, and only passes it to the project's
  closures.
- `REQ-RUNTIME-004` The routes closure returns a `utoipa_axum::OpenApiRouter`;
  the daemon serves the resulting `openapi.json` and a health route. The MCP
  closure returns an rmcp Streamable HTTP service, mounted with
  `nest_service("/mcp", ...)` on the same router and listener.
- `REQ-RUNTIME-005` The daemon shuts down gracefully on SIGINT and SIGTERM
  (Ctrl-C on Windows): in-flight requests finish, the listener closes, the
  lock is released and a UDS socket file is removed.
- `REQ-RUNTIME-006` `sc_runtime::testing::DaemonFixture` starts a daemon on a
  tempdir instance root, exposes a connected `sc_transport::Client`, and stops
  the daemon on drop. Fixtures run in parallel within one test binary.
- `REQ-RUNTIME-007` `run()` returns a typed result the project maps to a
  process exit code; assembly failures identify the failing step.

## Non-functional requirements

- `NFR-RUNTIME-001` Assembly only: no command registry, no macro, and no type
  that wraps an axum, rmcp or sqlx type.
- `NFR-RUNTIME-002` The crate does not depend on sqlx or on
  `sc-observability`, and does not load configuration itself.
- `NFR-RUNTIME-003` It is the only crate in this workspace that depends on
  the other three, and it enables their `server` features.
- `NFR-RUNTIME-004` No public function panics.
