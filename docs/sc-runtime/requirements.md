# `sc-runtime` Requirements

**ID Range:** REQ-RT-0001 through REQ-RT-0007; NFR-RT-0001 through NFR-RT-0004  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for `sc-runtime`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md) ("decision N" refers to its
decision log). Entries follow the shared SC requirement and ADR templates:
the obligation, then **Why** and **Verified by**. Every id is binding and is
never reused.

## Purpose

`sc-runtime` is the assembled daemon: a bootstrap that builds the parts and
hands them to the project. It is the single assembly point (SC scaffold decision record 013) reduced
to assembly only. It is a dependency of the daemon binary and of tests, never
of the CLI.

## ID Ranges

| Range | Area | Scope |
|---|---|---|
| REQ-RT-0001 through REQ-RT-0007 | Requirements | - |
| NFR-RT-0001 through NFR-RT-0004 | Non-functional requirements | - |

---

## REQ-RT-0001: The builder and its fixed order

**Status:** Active  

### Requirement Statement

`Daemon::builder(app, &daemon_config)` accepts a stores closure, a routes
closure and an optional MCP closure. `run()` performs, in this order: take
the already-loaded config; resolve the instance root and take the
`daemon.lock` singleton; call the stores closure; merge the project's router,
the OpenAPI document route, a health route, and the MCP service if one was
given; bind the listener through `sc-transport` and serve.

### Rationale

This is the hand-assembly every project used to write differently.
The order matters: the lock comes before stores so a second daemon never
opens the database, and binding comes last so nothing accepts a request
before it can be served.

### Success Criteria

A test that records step order through the closures; tests
that a failure at each step stops the later steps.

---

## REQ-RT-0002: Hard singleton

**Status:** Active  

### Requirement Statement

The singleton is an OS exclusive lock on `<instance-root>/daemon.lock`, held
for the life of the process. A second daemon on the same instance root fails
with a typed error before it opens any store or binds any listener.

### Rationale

The daemon is the only process allowed to open the local database
(decision 2). An OS lock is released by the kernel when the process dies, so
there is no stale-PID problem.

### Success Criteria

A test starting two daemons on one instance root and
asserting the second's error and that its stores closure never ran.

---

## REQ-RT-0003: `Stores` is opaque

**Status:** Active  

### Requirement Statement

The project's `Stores` value is a generic parameter. The crate never inspects
it; it only passes it to the project's routes and MCP closures.

### Rationale

This is how "which route uses which store" stays entirely in project
code. Opening pools and running migrations happens inside the store crates,
which are project-owned template code.

### Success Criteria

The crate compiles with no bound on `Stores` beyond what
passing it to the closures needs, and has no sqlx dependency.

---

## REQ-RT-0004: What gets mounted

**Status:** Active  

### Requirement Statement

The routes closure returns a `utoipa_axum::OpenApiRouter`; the daemon serves
the resulting `openapi.json` and a health route alongside it. The MCP closure
returns an rmcp Streamable HTTP service, mounted with
`nest_service("/mcp", ...)` on the same router and listener.

### Rationale

One registration yields both router and spec, and MCP shares the
binary and the port ([REQ-RUN-0205](../requirements.md), [ADR-RUN-0202](../architecture.md)).

### Success Criteria

Fixture tests fetching `openapi.json`, the health route and
`/mcp`; a test with no MCP closure showing `/mcp` absent.

---

## REQ-RT-0005: Graceful shutdown

**Status:** Active  

### Requirement Statement

The daemon shuts down gracefully on SIGINT and SIGTERM (Ctrl-C on Windows):
in-flight requests finish, the listener closes, the lock is released and a
UDS socket file is removed.

### Rationale

Daemons are restarted by people, by service managers and by tests;
a restart must not drop requests or leave files that block the next start.

### Success Criteria

A test that signals a daemon with a request in flight and
asserts the response completes and the socket file is gone.

---

## REQ-RT-0006: `DaemonFixture`

**Status:** Active  

### Requirement Statement

`sc_runtime::testing::DaemonFixture` starts a daemon on a tempdir instance
root, exposes a connected `sc_transport::Client`, and stops the daemon on
drop. Fixtures run in parallel within one test binary.

### Rationale

Every generated project tests its operations against a real daemon;
without an isolated fixture those tests would share a socket and a database
and could not run in parallel ([NFR-RUN-0005](../requirements.md)). It is the example test in the
template ([REQ-RUN-0302](../requirements.md)).

### Success Criteria

A test running many fixtures concurrently.

---

## REQ-RT-0007: A typed result for `main`

**Status:** Active  

### Requirement Statement

`run()` returns `Result<(), RuntimeError>`; `RuntimeError` identifies the
failing step (lock held, stores failed, bind failed, serve failed) and
carries the underlying error.

### Rationale

The generated `main.rs` maps the result to an exit code and a
message; "daemon already running" must be distinguishable from "cannot bind".

### Success Criteria

A test per variant.

---

## NFR-RT-0001: Assembly only

**Status:** Active  

### Requirement Statement

No command registry, no macro, and no type that wraps an axum, rmcp or sqlx
type.

### Rationale

[ADR-RUN-0001](../architecture.md). `sc-runtime` is a bootstrap, not a framework layer.

### Success Criteria

`arch-qa`; the public surface in the boundary manifest.

---

## NFR-RT-0002: No sqlx, no observability, no config loading

**Status:** Active  

### Requirement Statement

The crate does not depend on sqlx or on `sc-observability`, and does not load
configuration itself; it receives config already loaded.

### Rationale

Stores are project code ([ADR-RUN-0301](../architecture.md)); observability is initialised by
the project ([ADR-RUN-0004](../architecture.md)); keeping config loading outside lets the project report
config errors before anything else starts.

### Success Criteria

`Cargo.toml`; `forbidden_edges`.

---

## NFR-RT-0003: The only assembler

**Status:** Active  

### Requirement Statement

It is the only crate in this workspace that depends on the other three, and
it enables their `server` features.

### Rationale

[ADR-RUN-0003](../architecture.md).

### Success Criteria

The boundary manifests.

---

## NFR-RT-0004: Errors are values

**Status:** Active  

### Requirement Statement

No public function panics.

### Rationale

[ADR-RUN-0006](../architecture.md).

### Success Criteria

Source review; [REQ-RT-0007](requirements.md) tests.
