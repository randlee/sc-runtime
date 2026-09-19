# `sc-runtime` Requirements

Crate-level requirements for `sc-runtime`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md) ("decision N" refers to its
decision log). Entries follow `.claude/skills/plan-hardening/req-adr-format.md`:
the obligation, then **Why** and **Verified by**. Every id is binding and is
never reused.

## Purpose

`sc-runtime` is the assembled daemon: a bootstrap that builds the parts and
hands them to the project. It is the single assembly point (SC scaffold decision record 013) reduced
to assembly only. It is a dependency of the daemon binary and of tests, never
of the CLI.

## Requirements

- `REQ-RUNTIME-001` **The builder and its fixed order.**
  `Daemon::builder(app, &daemon_config)` accepts a stores closure, a routes
  closure and an optional MCP closure. `run()` performs, in this order: take
  the already-loaded config; resolve the instance root and take the
  `daemon.lock` singleton; call the stores closure; merge the project's router,
  the OpenAPI document route, a health route, and the MCP service if one was
  given; bind the listener through `sc-transport` and serve.
  **Why:** this is the hand-assembly every project used to write differently.
  The order matters: the lock comes before stores so a second daemon never
  opens the database, and binding comes last so nothing accepts a request
  before it can be served.
  **Verified by:** a test that records step order through the closures; tests
  that a failure at each step stops the later steps.

- `REQ-RUNTIME-002` **Hard singleton.**
  The singleton is an OS exclusive lock on `<instance-root>/daemon.lock`, held
  for the life of the process. A second daemon on the same instance root fails
  with a typed error before it opens any store or binds any listener.
  **Why:** the daemon is the only process allowed to open the local database
  (decision 2). An OS lock is released by the kernel when the process dies, so
  there is no stale-PID problem.
  **Verified by:** a test starting two daemons on one instance root and
  asserting the second's error and that its stores closure never ran.

- `REQ-RUNTIME-003` **`Stores` is opaque.**
  The project's `Stores` value is a generic parameter. The crate never inspects
  it; it only passes it to the project's routes and MCP closures.
  **Why:** this is how "which route uses which store" stays entirely in project
  code. Opening pools and running migrations happens inside the store crates,
  which are project-owned template code.
  **Verified by:** the crate compiles with no bound on `Stores` beyond what
  passing it to the closures needs, and has no sqlx dependency.

- `REQ-RUNTIME-004` **What gets mounted.**
  The routes closure returns a `utoipa_axum::OpenApiRouter`; the daemon serves
  the resulting `openapi.json` and a health route alongside it. The MCP closure
  returns an rmcp Streamable HTTP service, mounted with
  `nest_service("/mcp", ...)` on the same router and listener.
  **Why:** one registration yields both router and spec, and MCP shares the
  binary and the port (`REQ-E2E-005`, ADR-006).
  **Verified by:** fixture tests fetching `openapi.json`, the health route and
  `/mcp`; a test with no MCP closure showing `/mcp` absent.

- `REQ-RUNTIME-005` **Graceful shutdown.**
  The daemon shuts down gracefully on SIGINT and SIGTERM (Ctrl-C on Windows):
  in-flight requests finish, the listener closes, the lock is released and a
  UDS socket file is removed.
  **Why:** daemons are restarted by people, by service managers and by tests;
  a restart must not drop requests or leave files that block the next start.
  **Verified by:** a test that signals a daemon with a request in flight and
  asserts the response completes and the socket file is gone.

- `REQ-RUNTIME-006` **`DaemonFixture`.**
  `sc_runtime::testing::DaemonFixture` starts a daemon on a tempdir instance
  root, exposes a connected `sc_transport::Client`, and stops the daemon on
  drop. Fixtures run in parallel within one test binary.
  **Why:** every generated project tests its operations against a real daemon;
  without an isolated fixture those tests would share a socket and a database
  and could not run in parallel (`NFR-REPO-005`). It is the example test in the
  template (`REQ-TEMPLATE-002`).
  **Verified by:** a test running many fixtures concurrently.

- `REQ-RUNTIME-007` **A typed result for `main`.**
  `run()` returns `Result<(), RuntimeError>`; `RuntimeError` identifies the
  failing step (lock held, stores failed, bind failed, serve failed) and
  carries the underlying error.
  **Why:** the generated `main.rs` maps the result to an exit code and a
  message; "daemon already running" must be distinguishable from "cannot bind".
  **Verified by:** a test per variant.

## Non-functional requirements

- `NFR-RUNTIME-001` **Assembly only.**
  No command registry, no macro, and no type that wraps an axum, rmcp or sqlx
  type.
  **Why:** ADR-001. `sc-runtime` is a bootstrap, not a framework layer.
  **Verified by:** `arch-qa`; the public surface in the boundary manifest.

- `NFR-RUNTIME-002` **No sqlx, no observability, no config loading.**
  The crate does not depend on sqlx or on `sc-observability`, and does not load
  configuration itself; it receives config already loaded.
  **Why:** stores are project code (ADR-007); observability is initialised by
  the project (ADR-005); keeping config loading outside lets the project report
  config errors before anything else starts.
  **Verified by:** `Cargo.toml`; `forbidden_edges`.

- `NFR-RUNTIME-003` **The only assembler.**
  It is the only crate in this workspace that depends on the other three, and
  it enables their `server` features.
  **Why:** ADR-004.
  **Verified by:** the boundary manifests.

- `NFR-RUNTIME-004` **Errors are values.**
  No public function panics.
  **Why:** ADR-011.
  **Verified by:** source review; `REQ-RUNTIME-007` tests.
