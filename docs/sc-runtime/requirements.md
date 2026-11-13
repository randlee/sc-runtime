# `sc-runtime` Requirements

**ID Range:** REQ-RT-0001 through REQ-RT-0008; NFR-RT-0001 through NFR-RT-0004  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for `sc-runtime`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md). Entries follow the
shared SC requirement template: each item has the three sections
`Requirement Statement`, `Rationale` and `Success Criteria`, in that order.
Every id is binding and is never reused.

## Purpose

`sc-runtime` is the assembled daemon: a bootstrap that builds the parts and
hands them to the project. It is the single assembly point, reduced to
assembly only. It is a dependency of the daemon binary and of tests, never
of the CLI.

## ID Ranges

| Range | Area | Scope |
|---|---|---|
| REQ-RT-0001 through REQ-RT-0008 | Requirements | - |
| NFR-RT-0001 through NFR-RT-0004 | Non-functional requirements | - |

---

## REQ-RT-0001: `DaemonBuilder` and the fixed start-up order of `run()`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

The exact builder surface, five-step startup inventory and prohibition on inserting or reordering steps describe the Phase D deliverable. They do not prohibit a later owner-directed extension and create no work for it in Phase D. The current lock-before-stores, readiness-before-listen and failure-short-circuit requirements remain binding.

For Phase D, the names below are pinned and the shape and order are binding. `run` exists only after `stores` and `routes` (consuming typestate, [ADR-RT-0001](architecture.md)).

| Call | Argument | Required |
|---|---|---|
| `Daemon::builder(app, &daemon_config)` | `app`: `&sc_transport::AppName`; `daemon_config`: a `&DaemonConfig` the project has already loaded. This crate does not define `sc_runtime::AppName` | yes |
| `.stores(closure)` | an async closure taking no arguments that opens the project's stores and returns the project's own `Stores` value or an error | yes |
| `.routes(closure)` | a closure that receives the `Stores` value and returns a `utoipa_axum::OpenApiRouter` | yes |
| `.mcp(closure)` | a closure that receives the `Stores` value and returns an rmcp Streamable HTTP service | no |
| `.logger(handle)` | a `sc_command::observability::SharedCanonicalLog` (ADR-RUN-0017 initial bridge); optional in every builder state and replaces the fixed `sc-runtime` Noop | no |
| `.run().await` | none; returns `Result<(), RuntimeError>` | yes |
| `.run_with_cleanup(cleanup).await` | ready-state alternative terminal run with once-only asynchronous application cleanup | no |
| `.run_with_cleanup_and_bound(on_bound, cleanup).await` | Phase C ready-state alternative terminal run; reports the actual owned bound Endpoint, then preserves the same cleanup sequence | no |

Choose exactly one ready-state terminal run method. These alternatives add no
typestate, operation registry, library wrapper or new callback engine; all use
the same fixed startup/teardown runner. NFR-RT-0001 limits this exact surface.

The optional logger is a `SharedCanonicalLog` retained through `stores`,
`routes`, and `run` without adding a typestate. Repeated `.logger()` calls
replace and drop the prior handle; only explicit injection changes the fixed
`sc-runtime` identity. Library code never flushes or shuts down the logger.
Emission errors are best effort and cannot replace an HTTP or `RuntimeError`
result. `observability_contract` proves phase preservation, Arc ownership,
replacement, identity and unchanged failure behavior.

Intended use, in the `daemon/src/main.rs` of a generated project:

```rust
#[tokio::main]
async fn main() -> ExitCode {
    // config is loaded by the project with sc-config, before the builder
    let cfg: AppConfig = match sc_config::load("my-app") {
        Ok(cfg) => cfg,
        Err(e) => return report(e),
    };
    // observability is built and retained by the project, not by sc-runtime
    let observability = match daemon::observability::Observability::build(
        APP_NAME, log_root, &cfg.logging,
    ) {
        Ok(owner) => owner,
        Err(e) => return report_observability(e),
    };
    let logger = observability.logger();

    let app = match sc_transport::AppName::new("my-app") {
        Ok(app) => app,
        Err(e) => return report(e),
    };
    let result = sc_runtime::Daemon::builder(&app, &cfg.daemon)
        .logger(logger.clone())
        .stores(|| async { Stores::open(&cfg.stores).await })   // project-defined struct
        .routes(|stores| routes::router(stores))                // utoipa_axum::OpenApiRouter
        .mcp(|stores| mcp::service(stores))                     // optional; an rmcp service
        .run()
        .await;
    drop(logger);
    let shutdown = observability.shutdown();
    // project helper preserves a run error when shutdown also fails
    exit_code(prefer_run_error(result, shutdown))
}
```

`DaemonConfig` is a public struct of `sc-runtime`. This follows from the usage above: the project loads it as the `daemon` section of its own config (`cfg.daemon`) through `sc-config`, which deserialises with serde.

1. Take the `DaemonConfig` passed to `builder`. Honouring the `SC_ENDPOINT` override through the `sc-transport` endpoint resolver is not loading configuration ([REQ-RT-0008](requirements.md)).
2. Resolve the instance root (the per-user directory for `app`, or the one
   supplied explicitly) and take an OS exclusive lock on
   `<instance-root>/daemon.lock`.
3. Call the stores closure exactly once and await its result.
4. Build one `axum::Router` by merging: the router from the routes closure,
   a route serving the OpenAPI document produced by that `OpenApiRouter`, a
   health route, and, only if `.mcp(...)` was called, the rmcp service at
   `/mcp`. The relative order in which the routes closure and the MCP closure
   are called is not binding.
5. Bind through `sc-transport` (built with its `server` feature) and serve
   the merged router until SIGINT or SIGTERM. For a UDS endpoint, `bind`
   acquires and retains the endpoint-scoped lock required by REQ-TRN-0004
   before it inspects or replaces the socket. v0.1 binds exactly one listener.
   `TransportConfig` is `#[non_exhaustive]` and has no `additional_endpoints`
   field. It is one `axum::Router`, served on that listener.

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](../sc-transport/requirements.md)). How `run()`
obtains the instance root and the bind endpoint is
[REQ-RT-0008](requirements.md).

Endpoint text is resolved before the lock so an invalid endpoint does not open stores or take the lock. Signal streams are registered before bind. `.logger(...)` only replaces the event handle carried by the builder. It is available on `NeedStores`, `NeedRoutes`, and `Ready`, adds no typestate, and does not add or reorder a `run()` step. The required transition remains `NeedStores` -> `NeedRoutes` -> `Ready`.

v0.1 binds exactly one listener. `sc-transport` can bind either a Unix domain
socket or a TCP address ([REQ-TRN-0003](../sc-transport/requirements.md)).
`DaemonConfig` does not request a second listener.

`Daemon::builder(&sc_transport::AppName, &DaemonConfig)` returns a builder on
which `run` is not implemented. `.stores` takes `FnOnce() -> Fut` where `Fut`
is a future of `Result<S, RuntimeError>` and `S: Clone + Send + Sync + 'static`.
`.routes` takes `FnOnce(S) -> utoipa_axum::router::OpenApiRouter` after the
project has applied Axum state, so the router parameter is `()`. `.mcp` is
available only on that ready state, returns the same builder, and takes
`FnOnce(S) -> rmcp::transport::streamable_http_server::StreamableHttpService<Svc, Mgr>`.
The service is mounted as given. `.run().await` returns `Result<(), RuntimeError>`
only after `stores` and `routes`. `NeedStores`, `NeedRoutes`, and `Ready` are
`#[doc(hidden)]` typestate markers. They are not facade items.

`DaemonConfig` is `#[non_exhaustive]`. Field `transport` is
`sc_transport::TransportConfig`, flattened by serde, and carries `endpoint`,
`instance_root`, and `tcp_port`. Field `endpoint_flag` is `Option<String>`,
the `--endpoint` text the project's `main` already parsed. `SC_ENDPOINT` is
not a field. A project's CLI cannot name `DaemonConfig`, because the CLI does
not depend on `sc-runtime`. It uses `TransportConfig` from `sc-transport`
with default features. `DaemonConfig` embeds that type.

Definitions: A startup step is one of the five numbered steps (1 to 5) listed in this requirement statement. The startup lock is the OS exclusive lock that step 2 takes on `<instance-root>/daemon.lock`. `DaemonBuilder` is the builder, returned by `Daemon::builder`, that assembles and runs a daemon. The Phase D `run()` is the `run()` method as delivered in Phase D, and the Phase D `DaemonBuilder` is `DaemonBuilder` as delivered in Phase D. A startup extension point is any hook, callback or configuration option: a builder method, a closure argument or a `DaemonConfig` field.

**MUST statements:**

- The crate `sc-runtime` (`crates/sc-runtime`) MUST export the public type `DaemonBuilder`.
- The public struct `DaemonConfig` in `crates/sc-runtime` MUST implement `serde::Deserialize`.
- The Phase D `run()` MUST perform every startup step.

- The Phase D `run()` MUST perform the startup steps in ascending step-number order.
- If a startup step fails, `run()` MUST return `Err(RuntimeError)`.

**MUST NOT statements:**

- If a startup step fails, `run()` MUST NOT execute any later startup step.
- `run()` MUST NOT read a config file to load configuration.
- The Phase D `run()` MUST NOT perform any startup work other than the five startup steps.

- `run()` MUST NOT read an environment variable to load configuration.
- `run()` MUST NOT call the stores closure until step 2 has taken the startup lock.
- `run()` MUST NOT bind a listener until steps 1 to 4 have succeeded.
- The Phase D `DaemonBuilder` MUST NOT offer a startup extension point that inserts work between two startup steps.
- The Phase D `DaemonBuilder` MUST NOT offer a startup extension point that changes the order of the startup steps.

### Rationale

Before this crate, every project hand-assembled lock, stores, router and
listener, each slightly differently. One builder makes start-up identical in
every project. The order is the safety property: the lock comes before the
stores so a second daemon never opens the local database, and binding comes
last so no request is accepted before it can be served.
The ban on hooks, callbacks or configuration that insert or reorder steps was
decided in this document and recorded in [ADR-RT-0001](architecture.md): the
design lists the five steps and says the crate is a bootstrap, not a framework
layer.
### Success Criteria

1. A test in `crates/sc-runtime` builds a daemon whose stores, routes and MCP
   closures each append their name to a shared list, runs it on a tempdir
   instance root, and asserts the list has exactly three entries, that the
   first is `stores`, that the other two are `routes` and `mcp` in either
   order, and that `<instance-root>/daemon.lock` existed and was locked when
   the stores closure ran.
2. A test holds the lock on the instance root, calls `run()`, and asserts it
   returns `Err`, and that the stores, routes and MCP closures were never
   called and no listener was bound.
3. A test whose stores closure returns an error asserts `run()` returns `Err`,
   and that the routes and MCP closures were never called and no listener was
   bound (connecting to the endpoint fails).
4. A test that makes the bind fail (the TCP port is already in use) asserts
   `run()` returns `Err` after the stores closure ran, and that no request was
   served.
5. On Unix, a first daemon uses instance root `R1` and UDS override `S`. A
   second daemon uses distinct root `R2` and the same override `S`; its
   startup fails safely, the first daemon remains reachable through `S`, and
   the failed startup and cleanup do not remove `S`.
6. For the Phase D deliverable, inspection of the public API of `crates/sc-runtime`: `DaemonBuilder` has no
   public method other than `logger`, `stores`, `routes`, `mcp`, `run`,
   `run_with_cleanup` and `run_with_cleanup_and_bound`. The three terminal
   run alternatives are implemented only on Ready.
7. A test deserialises a `DaemonConfig` from a JSON document with
   `serde_json` (a dev-dependency) and passes it to `Daemon::builder`; it
   compiles, which shows `DaemonConfig: serde::Deserialize`.

---

## REQ-RT-0002: Hard singleton through an OS lock on `daemon.lock`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions:

- sc-runtime is the Rust crate in `crates/sc-runtime` that assembles application daemons.
- A daemon is one running application server process built with sc-runtime.
- The instance root is the per-application, per-user directory resolved by sc-transport, or a directory the caller supplies explicitly; its default location is undecided ([REQ-TRN-0002](../sc-transport/requirements.md)).
- The lock file is the file `daemon.lock` in the instance root.
- The instance lock is an operating-system exclusive file lock on the lock file.
- `fd-lock` is the third-party Rust crate for operating-system file locks.
- `run()` is the daemon builder's terminal method that starts and serves a daemon.
- `run_with_cleanup` is the daemon builder's terminal method that runs like `run()` and then awaits the cleanup callback before returning.
- The cleanup callback is the asynchronous application cleanup passed to `run_with_cleanup`.
- Cleanup completion is the moment the future returned by the cleanup callback resolves.
- Lock contention is the case where another daemon holds the instance lock when `run()` tries to take the instance lock.
- A lock-file failure is a failure to open or lock the lock file without lock contention, such as permission denied or an instance root that is an existing file.
- The lock-release point is the earlier of two events: `run()` returns, or the process exits.
- The stores closure is the project-supplied asynchronous `.stores(...)` closure that opens stores and returns the project's `Stores` value.
- The runtime logger is the `sc_command::observability::SharedCanonicalLog` handle held by the daemon builder.
- A listener is a Unix-domain-socket or TCP listener bound through sc-transport.
- Successful startup is completion of the startup sequence through listener binding.
- A startup error is an error that startup returns after `run_with_cleanup` holds the instance lock.
- `RuntimeError` is the public sc-runtime startup and serving error enum; [REQ-RT-0007](requirements.md) defines its variants `LockHeld` and `InstanceFailed`.
- A PID file is a file that stores a process id.

`run()` creates the instance root when the path is absent; whether sc-transport creates it stays with [REQ-TRN-0002](../sc-transport/requirements.md). Tests supply a temporary directory as the instance root.

**MUST statements:**

- For each instance root, at most one daemon started through `run()` MUST be running at any time.
- The lock code in `crates/sc-runtime/src/lock.rs` MUST take the instance lock as an exclusive (write) operating-system file lock.
- sc-runtime MUST take the instance lock through the `fd-lock` crate.
- sc-runtime MUST place the instance lock on `<instance-root>/daemon.lock`.
- `run()` MUST hold the instance lock when `run()` calls the stores closure.
- `run()` MUST hold the instance lock when `run()` binds each listener.
- `run()` MUST hold the instance lock without a gap from acquisition until the lock-release point.
- `run_with_cleanup` MUST hold the instance lock until cleanup completion.
- After successful startup, `run_with_cleanup` MUST hold the instance lock until cleanup completion.
- After a startup error, `run_with_cleanup` MUST hold the instance lock until cleanup completion.
- After the stores closure succeeds, `run_with_cleanup` MUST keep the returned `Stores` value alive until cleanup completion.
- `run_with_cleanup` MUST keep the runtime logger alive until cleanup completion.
- On lock contention, `run()` MUST return `RuntimeError::LockHeld`.
- When opening the lock file fails without lock contention, `run()` MUST return `RuntimeError::InstanceFailed`.
- When locking the lock file fails without lock contention, `run()` MUST return `RuntimeError::InstanceFailed`.
- The lock code in `crates/sc-runtime/src/lock.rs` MUST take the instance lock by calling the non-blocking `fd-lock` method `try_write`.

**MUST NOT statements:**

- On a lock-file failure, `run()` MUST NOT call the stores closure.
- The lock code in `crates/sc-runtime/src/lock.rs` MUST NOT call a blocking `fd-lock` lock method.
- On lock contention, `run()` MUST NOT call the stores closure.
- On lock contention, `run()` MUST NOT bind a listener.
- sc-runtime MUST NOT use a PID file to detect another daemon.
- sc-runtime MUST NOT probe a network port to detect another daemon.

### Rationale

The design requires an exclusive lock; this record additionally chooses a non-blocking acquisition attempt so a second daemon reports contention without waiting for the lock holder.

The daemon is the only process allowed to open the local database; two
daemons on one SQLite file would mean two writers. An OS lock is released by
the kernel whenever the process ends, including a crash, so there is no
stale-PID file to clean up and no race between checking and claiming.
### Success Criteria

The ordering and retention clauses are checked by inspecting the startup and cleanup runner: lock acquisition precedes both the stores-closure invocation and listener binding; the acquired lock, successful `Stores` value and logger handle remain owned until application cleanup completes. The non-blocking acquisition clause is checked by inspecting the fd-lock try-lock path, as criterion 1 specifies. The two-daemon test checks the observable startup refusal. These checks add no wall-clock timing assertion.

1. A test starts the first daemon with an explicit temporary instance root and waits for an explicit startup-complete signal confirming that its instance lock is held. While that daemon remains running and holds the lock, call `run()` for a second daemon with the same instance root and assert the lock-held `RuntimeError` result. Inspect the production acquisition path to verify that it uses fd-lock's non-blocking try-lock operation. A harness watchdog may terminate a hung test, but elapsed wall time is not a pass/fail assertion for non-blocking behavior.
2. The same test asserts the second daemon's stores closure was never called
   (a flag set by the closure stays unset) and the first daemon still answers
   a request.
3. A test stops the first daemon, then starts a new daemon on the same
   instance root, and asserts it starts successfully even though
   `daemon.lock` still exists on disk.
4. `crates/sc-runtime/Cargo.toml` lists `fd-lock` as a dependency, and the
   crate source contains no code that writes or reads a PID file.

---

## REQ-RT-0003: The project's `Stores` type is opaque to `sc-runtime`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: MCP means Model Context Protocol. A migration is database schema initialization or upgrade owned by the project's store code, rather than daemon assembly. A receiving closure is the project's routes closure or its MCP closure. Passing the `Stores` value includes moving or cloning it into a receiving closure.

`Stores` being opaque to this crate describes this release. `Storage<T>` is defined in the `sc-storage` crate (REQ-RUN-0002); this record does not restrict it.

`Stores` is a plain struct defined by the generated project (for example one holding a SQLite store and a Postgres store).

- The exact bounds are pinned with the closure signatures by the contract sprint.
- Those happen inside the project's stores closure, in the project-owned store crates.

`sc-runtime` does not depend on `sqlx`. That rule and its two checks (the
`cargo tree` command and the boundary manifest entry) are stated once, in
[NFR-RT-0002](requirements.md).

**MUST statements:**

- In `sc-runtime` it MUST be a generic type parameter of the builder.
- `sc-runtime` MUST obtain the `Stores` value only from the project's stores closure.
- `sc-runtime` MUST pass that value to the project's routes closure.
- When the project provides an MCP closure, `sc-runtime` MUST pass the `Stores` value to the project's MCP closure.
- The only bounds on the `Stores` parameter MUST be those needed to move it into the closures and across the async runtime (for example `Send`, `'static`, `Clone`).

**MUST NOT statements:**

- Code in `crates/sc-runtime/src` MUST NOT use the `Stores` value except to pass it to a receiving closure.
- `sc-runtime` MUST NOT read a field of `Stores`.
- `sc-runtime` MUST NOT call a method on `Stores`.
- `sc-runtime` MUST NOT require a trait describing the contents of `Stores`.
- `sc-runtime` MUST NOT open a database pool.
- `sc-runtime` MUST NOT run a migration.

### Rationale

Which route uses which store is a project decision. If the framework knew
what a store was, it would need sqlx, knowledge of each backend, and an
opinion about how many stores exist. Keeping `Stores` opaque keeps all of
that in project code, and lets a project with no database use the crate.
### Success Criteria

1. A test in `crates/sc-runtime` runs a daemon with `Stores = ()` and another
   with a test-local struct that implements no trait beyond the pinned bounds;
   both compile and serve a request.
2. A test asserts the value produced by the stores closure is the value the
   routes closure and the MCP closure receive (the closure returns a struct
   holding a unique token; both closures assert the token).
3. Inspection of `crates/sc-runtime/src`: no `where` clause or trait bound on
   the `Stores` parameter names a trait defined by `sc-runtime`, and no code
   accesses a field or method of a `Stores` value.

---

## REQ-RT-0004: Routes mounted on the daemon's single router

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: MCP means Model Context Protocol. HTTP means Hypertext Transfer Protocol. JSON means JavaScript Object Notation. ADR means architecture decision record. The daemon router is the one `axum::Router` that `run()` builds. The mount table is the table in this requirement headed Mounted item, Source, Condition. The routes closure is the application-supplied closure passed to `.routes(...)`. The MCP closure is the application-supplied closure passed to `.mcp(...)`. The rmcp service is the `StreamableHttpService` returned by the MCP closure. Session state is server-owned state tracking an MCP client session; a session store is persistence for that state. The daemon listener is the listener `run()` binds. The daemon process is the process in which `run()` runs. The rmcp service is mounted unchanged when the value mounted is the value the MCP closure returned, with none of its configuration modified.

It is one `axum::Router`, served unchanged on every listener `run()` binds.

| Mounted item | Source | Condition |
|---|---|---|
| The project's REST routes | the `utoipa_axum::OpenApiRouter` returned by the routes closure | always |
| The OpenAPI document as JSON (`openapi.json`) | the `utoipa` OpenAPI document produced by the same `OpenApiRouter` (utoipa-axum's `split_for_parts`) | always |
| A health route | supplied by `sc-runtime` | always |
| The MCP endpoint at `/mcp` | the rmcp Streamable HTTP service (`StreamableHttpService`) returned by the MCP closure, mounted with `Router::nest_service("/mcp", service)` | only if `.mcp(...)` was called |

- Configuring the rmcp service in stateless mode is done by the generated project's `mcp::service`, and is required of the template by [REQ-RUN-0302](../requirements.md); `sc-runtime` mounts what it is given.

v0.1 binds one listener ([REQ-RT-0001](requirements.md)).

The OpenAPI document route is `GET /openapi.json`. The body is the JSON
document from `OpenApiRouter::split_for_parts`.

`GET /health` returns status 200. The body is the `sc_command` success
envelope produced by the server-feature Axum `IntoResponse` for
`sc_command::Envelope::success`. The data is `{ "status": "ok" }`. That
conversion is the `sc_command` item `crates/sc-runtime/src` uses.

Those three paths (`GET /health`, `GET /openapi.json`, and `/mcp` when
`.mcp(...)` was called) are reserved. `run()` inspects the project's
`OpenApiRouter` after the routes closure returns and before it mounts the
reserved routes. Overlap returns `RuntimeError::RouteConflict` with code
`RUNTIME.ROUTE_CONFLICT` and does not bind. Axum is not allowed to panic on
the later `route` or `nest_service`. The stores closure has already run
because lock-then-stores-then-routes is the binding start-up order; the
check happens before reserved-route mount side effects and before bind.

MCP is reachable only as `/mcp` on the one router. With no project route or fallback at `/mcp`, axum answers HTTP 404.

**MUST statements:**

- `run()` MUST build exactly one daemon router.
- `run()` MUST build the daemon router after the instance lock on `daemon.lock` ([REQ-RT-0002](requirements.md)) is held.
- `run()` MUST build the daemon router after the stores closure has returned.
- `run()` MUST build the daemon router before the daemon listener is bound.
- The daemon router MUST contain every item of the mount table whose condition holds.
- The routes closure MUST return a `utoipa_axum::OpenApiRouter`.
- The OpenAPI document served MUST be the one generated from the project's route registrations.
- `sc-runtime` MUST mount the `OpenApiRouter` returned by the routes closure as given.
- `sc-runtime` MUST mount the rmcp service unchanged.

**MUST NOT statements:**

- `sc-runtime` MUST NOT wrap the `OpenApiRouter` returned by the routes closure in its own types.
- `sc-runtime` MUST NOT wrap the rmcp service in its own types.
- `sc-runtime` MUST NOT define its own router type.
- `sc-runtime` MUST NOT require the project to register the document route itself.
- `sc-runtime` MUST NOT add an MCP session store.
- `sc-runtime` MUST NOT add any other session state of its own.
- If `.mcp(...)` was not called, `sc-runtime` MUST NOT mount anything at `/mcp`.
- MCP MUST NOT get any listener other than the daemon listener.
- MCP MUST NOT get any TCP port other than the TCP port of the daemon listener.
- MCP MUST NOT get any process other than the daemon process.

### Rationale

One route registration yields both the router and the OpenAPI document, so
the document cannot drift from the routes. MCP shares the binary and the
port, so a project deploys and secures one endpoint. This item owns what
`sc-runtime` mounts; the product-level rule that one listener serves routes,
OpenAPI, health and `/mcp` is [REQ-RUN-0205](../requirements.md). The shape
(utoipa-axum plus rmcp in stateless mode on one axum router) is the Active
decision [ADR-RUN-0202](../architecture.md). The two facts still to be
verified by the spike sprint (that the library versions coexist, and that one
struct can serve as both the rmcp tool input and the OpenAPI schema) are held
separately in the Proposed [ADR-RUN-0203](../architecture.md); if one fails,
versions change and this shape does not.
### Success Criteria

1. A `DaemonFixture` test registers one project route, fetches `GET /openapi.json`,
   parses the body as JSON, and asserts the `paths` object contains that
   route's path.
2. A `DaemonFixture` test sends `GET /health` and asserts status 200 and the
   `sc_command` success envelope whose data is `{ "status": "ok" }`.
3. A `DaemonFixture` test built with an MCP closure sends an MCP `initialize`
   request over HTTP to `/mcp` on the endpoint the fixture exposes
   ([REQ-RT-0006](requirements.md)) and asserts a successful MCP response.
4. A `DaemonFixture` test built without an MCP closure requests `/mcp` and
   asserts HTTP 404.
5. One test calls a project route and performs criteria 1, 2, and 3 against
   the single endpoint of one fixture, showing that all of them are served by
   one router on one listener.
6. Inspection of `crates/sc-runtime/src` finds no MCP session manager, session
   store or session identifier handling.
7. Inspection of `run()` in `crates/sc-runtime/src/builder.rs` finds exactly
   one construction of the daemon router.
8. Inspection of `crates/sc-runtime/src` finds no listener bound, TCP port
   bound or process started for MCP other than the daemon listener, its TCP
   port and the daemon process.

---

## REQ-RT-0005: Graceful shutdown on SIGINT and SIGTERM

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions:

- `sc-runtime` is the Rust crate in `crates/sc-runtime` that assembles application daemons.
- `run()` is the `DaemonBuilder::run` method defined in `crates/sc-runtime/src/builder.rs`; `run()` returns `Result<(), RuntimeError>` ([REQ-RT-0007](requirements.md)).
- `run_with_cleanup` is the daemon builder's terminal method that runs like `run()` and then awaits the cleanup callback before returning.
- The cleanup callback is the infallible asynchronous application cleanup passed to `run_with_cleanup`.
- Cleanup completion is the moment the future returned by the cleanup callback resolves.
- Serving is step 5 of `run()` in the crate architecture: the period in which `run()` serves the router on every bound listener.
- The listener is the Unix-domain-socket or TCP listener that `run()` binds through `sc-transport`.
- The socket file is the Unix domain socket file at the endpoint path selected for the listener.
- The endpoint lock is the operating-system exclusive lock that the listener holds on the file named by appending `.lock` to the socket path ([ADR-TRN-0006](../sc-transport/architecture.md)).
- The endpoint-lock file is the file on which the listener holds the endpoint lock.
- The instance root is the per-application, per-user directory resolved by `sc-transport`, or an explicitly supplied path; its default location is undecided ([REQ-TRN-0002](../sc-transport/requirements.md)).
- `daemon.lock` is the file `<instance-root>/daemon.lock`.
- The instance lock is the operating-system exclusive lock that `run()` holds on `daemon.lock` ([REQ-RT-0002](requirements.md)).
- A shutdown request is SIGINT or SIGTERM received by the process, a Windows Ctrl-C or Ctrl-Break console event received by the process, or a firing of the crate-private trigger.
- The crate-private trigger is the internal shutdown notification, visible only inside `sc-runtime`, that `testing::DaemonFixture` uses to stop its own daemon without a process signal ([REQ-RT-0006](requirements.md)).
- An in-flight request is a request that the listener accepted before graceful shutdown began and whose response is not yet sent.
- Graceful shutdown is the sequence of steps that the MUST statements below impose on `run()` after a shutdown request.
- SIGINT is the Unix interrupt signal, and SIGTERM is the Unix termination signal.
- The request-handler timeout is the `REQUEST_TIMEOUT` constant in `crates/sc-runtime/src/router.rs`.
- The drain timeout is the `DRAIN_TIMEOUT` constant in `crates/sc-runtime/src/shutdown.rs`.
- The request-handler deadline of a request is the moment the request-handler timeout expires for that request's handler.
- The drain deadline is the moment the drain timeout expires after graceful shutdown begins.
- Shutdown cleanup is the listener, endpoint and instance-lock cleanup that graceful shutdown performs.
- The 503 error envelope is the existing HTTP 503 response whose JSON body carries a stable error code.
- The empty cleanup callback is a cleanup callback whose future resolves without doing any work.
- Ctrl-C is the Windows `CTRL_C_EVENT` console control event; Ctrl-Break is the Windows `CTRL_BREAK_EVENT` console control event.
- A handler future is the future that runs a route handler for one request.
- The stores closure is the project-supplied asynchronous `.stores(...)` closure that opens stores and returns the project's `Stores` value.
- A startup closure is the stores closure, routes closure or MCP closure passed to the daemon builder.
- A startup error is a `RuntimeError` that startup returns after `run_with_cleanup` holds the instance lock.
- A serve error is the `RuntimeError::ServeFailed` value that `run()` returns when serving the router ends with an error.

Statements that name Windows are Phase C targets. Phase B observability reports the timeouts and 503 error envelopes but does not change them.

**MUST statements:**

- While serving, `run()` MUST begin graceful shutdown when the process receives SIGINT.
- While serving, `run()` MUST begin graceful shutdown when the process receives SIGTERM.
- On Windows, while serving, `run()` MUST begin graceful shutdown when the process receives Ctrl-C.
- On Windows, while serving, `run()` MUST begin graceful shutdown when the process receives Ctrl-Break.
- When `run()` is compiled for Windows, the call that registers the Ctrl-C console handler MUST come before the call that takes the instance lock in the source of `run()`.
- When `run()` is compiled for Windows, the call that registers the Ctrl-Break console handler MUST come before the call that takes the instance lock in the source of `run()`.
- When the Windows process receives Ctrl-C after `run()` starts listening for Ctrl-C and before serving starts, `run()` MUST return `Ok(())`.
- When the Windows process receives Ctrl-Break after `run()` starts listening for Ctrl-Break and before serving starts, `run()` MUST return `Ok(())`.
- `run()` MUST begin graceful shutdown when the crate-private trigger fires.
- After graceful shutdown begins, the listener MUST refuse each new connection attempt.
- If the handler of an in-flight request completes before both its request-handler deadline and the drain deadline, the client MUST receive the complete response of the handler.
- While an in-flight request has no response, `run()` MUST keep serving the in-flight request on the listener.
- For a Unix domain socket endpoint, during graceful shutdown, the socket file MUST still exist when `run()` stops serving on the listener.
- The source of `run()` MUST place the call that removes the socket file before the call that releases the endpoint lock.
- For a Unix domain socket endpoint, the socket file MUST no longer exist when `run()` returns.
- After the call that stops serving on the listener, the source of `run_with_cleanup` MUST call the cleanup callback.
- The source of `run_with_cleanup` MUST place the call that removes the socket file before the call to the cleanup callback.
- The source of `run_with_cleanup` MUST place the call that releases the endpoint lock before the call to the cleanup callback.
- When a startup error occurs, the cleanup callback passed to `run_with_cleanup` MUST run.
- When a startup closure is not called, the source of `run_with_cleanup` MUST drop that startup closure before the call to the cleanup callback.
- While the cleanup callback of `run_with_cleanup` runs, `run()` of a second daemon on the same instance root MUST return `RuntimeError::LockHeld`.
- After cleanup completion, the source of `run_with_cleanup` MUST drop the `Stores` value that the stores closure returned.
- After the call that releases the endpoint lock, the source of `run()` MUST release the instance lock.
- After graceful shutdown completes, awaiting `run()` MUST produce `Ok(())`.
- After a startup error and cleanup completion, `run_with_cleanup` MUST return the startup error.
- After the cleanup completion that follows a serve error, `run_with_cleanup` MUST return `RuntimeError::ServeFailed`.
- In the source of `run()`, `run()` MUST call `run_with_cleanup` with the empty cleanup callback.
- The request-handler timeout MUST equal 30 seconds.
- The drain timeout MUST equal 30 seconds.
- If a request's request-handler deadline passes before its handler completes and before any drain deadline, `sc-runtime` MUST answer the request with the `RUNTIME.REQUEST_TIMEOUT` 503 error envelope.
- If the drain deadline passes before an in-flight request's handler completes and before that request's request-handler deadline, `run()` MUST answer the in-flight request with the 503 error envelope whose code is `RUNTIME.DRAIN_TIMEOUT`.
- After a request receives the 503 error envelope with code `RUNTIME.REQUEST_TIMEOUT` or `RUNTIME.DRAIN_TIMEOUT`, `sc-runtime` MUST drop the handler future of that request.
- When the drain deadline passes during graceful shutdown, `run()` MUST complete shutdown cleanup before returning `Ok(())`.
- When the drain timeout expires during graceful shutdown, `run()` MUST return `Ok(())`.
- For a Unix domain socket endpoint, after `run()` returns, the endpoint-lock file MUST still exist.
- Every call site of `run()` MUST await the future returned by `run()` to completion.

**MUST NOT statements:**

- The source of `sc-runtime` MUST NOT contain code that deletes `daemon.lock`.
- After `run()` returns, `run()` of a new daemon on the same instance root MUST NOT return `RuntimeError::LockHeld`.
- After a request receives the 503 error envelope with code `RUNTIME.REQUEST_TIMEOUT`, `sc-runtime` MUST NOT send another response to the request.
- The public API of `sc-runtime` MUST NOT expose the crate-private trigger.

### Rationale

Daemons are restarted by people, by service managers and by tests. A restart
must not drop a request that was already accepted, and must not leave a
socket file behind that the next start or a client would trip over.
Dropping the run future or panicking cannot perform asynchronous cleanup, so
callers await the run future. Releasing `daemon.lock` is sufficient; the
operating system frees the lock even when the file stays on disk.
### Success Criteria

Criteria that start a daemon in a child process run only on an isolated
machine, never on a developer's host
([NFR-RUN-0010](../requirements.md)).

1. A `#[cfg(unix)]` test starts a daemon in a child process whose handler
   acknowledges admission of the request and then awaits an explicit release
   signal. The test waits for the admission acknowledgement, sends SIGTERM to
   the child, observes that shutdown has stopped admitting new connections,
   releases the handler, and asserts the client receives the complete
   successful response and that cleanup completes. Ordering is driven by
   acknowledgements, not by sleeps; timeout checks use controlled time and
   explicit coordination rather than elapsed sleeps.
2. The same test asserts the child then exits, that a new connection attempt
   after the signal fails, and that the socket file no longer exists.
3. The same scenario with SIGINT passes the same assertions.
4. After the child exits, a new daemon on the same instance root starts
   successfully, proving the lock was released.
5. A test asserts `run()` returns `Ok(())` after a signal-initiated shutdown
   (the child's exit code is the success code).
6. Windows native isolated child tests send CTRL_C_EVENT and CTRL_BREAK_EVENT
   through real process-group console events, proving early registration,
   in-flight drain/cancel, cleanup, clean exit and restart. Keep existing
   private in-process fixture trigger private; do not signal the test host.
7. An in-process coordinated test proves a handler exceeding 30 seconds gets
   the 503 `RUNTIME.REQUEST_TIMEOUT` envelope, that its handler future is
   dropped, and that no later terminal response is emitted for that request.
8. A separate coordinated shutdown test proves an in-flight handler exceeding
   the 30-second drain gets the 503 `RUNTIME.DRAIN_TIMEOUT` envelope and its
   handler future is dropped, after
   which endpoint and instance-lock cleanup complete and `run()` returns
   `Ok(())`.
### Implementation Notes

OPEN: Windows console close and service stop remain outside this signal
contract. Ctrl-Break is selected for Phase C; implementation is planned.

---

## REQ-RT-0006: Test fixture `DaemonFixture` in the `testing` module

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface.
MCP means Model Context Protocol.
HTTP means Hypertext Transfer Protocol.
API means application programming interface.
OS means operating system.
TCP means Transmission Control Protocol.
A fixture is a sc_runtime::testing::DaemonFixture value controlling one application daemon.
A live fixture is a fixture whose daemon has started and has not completed shutdown.
The instance root is the fixture-specific temporary directory containing daemon.lock and, on macOS and Linux, daemon.sock.
The stores closure is the application-supplied asynchronous .stores(...) callback producing the application Stores value.
The routes closure is the application-supplied .routes(...) callback producing the application router.
The MCP closure is the optional application-supplied .mcp(...) callback producing the MCP service.
Endpoint is the sc-transport value identifying the bound Unix-domain-socket path or TCP socket address.
A real listener is an OS-bound Unix-domain-socket or TCP listener.
Client is the sc_transport::Client HTTP client type.
SC_ENDPOINT is the environment variable supplying an endpoint to the sc-transport resolver.
fixture in a code expression denotes the started DaemonFixture under test.
resolve_endpoint is the sc-transport function returning a parsed Endpoint; flag is the optional --endpoint value supplied to the function.
--endpoint is the command-line option supplying an endpoint to the sc-transport resolver.
A socket is a Unix-domain-socket endpoint.
A port is a TCP listener port.
A lock file is the instance-root daemon.lock file.
A database file stores the application database.
The shutdown trigger is the crate-private notification stopping only the fixture daemon.
The production shutdown runner is the common runtime path implementing the REQ-RT-0005 shutdown sequence for fixture stop and process signals.
The shutdown sequence is the production sequence in REQ-RT-0005: stop acceptance, drain requests, dispose the listener and selected socket in the specified endpoint-ownership order, await application cleanup while retaining successful stores and the instance lock, then release the instance lock.
Cleanup completion means resolution of the future returned by the application cleanup closure.
A started fixture is a DaemonFixture returned by a successful constructor call.
The current user is the operating-system user running the test.
A process signal is an operating-system signal delivered to a process.
A fixed path is a location reused instead of selecting a fresh temporary directory for each fixture.
Parallel fixture execution means overlapping live fixture lifetimes within one test binary under the default Rust test runner.

The testing module uses the non-default `test-util` feature ([NFR-RT-0003](requirements.md)); `tempfile` is optional and enabled only by `test-util`.

Constructors are `start(app, config, stores, routes)`, `start_with_mcp(app, config, stores, routes, mcp)` and additive `start_with_cleanup` with existing setup/routes inputs plus once-only asynchronous application cleanup. All are `DaemonFixture` methods. Accessors are `client()`, `endpoint()`, `endpoint_str()`, `instance_root()` and asynchronous `shutdown()`.

Endpoint text uses sc-transport conversion ([REQ-TRN-0001](../sc-transport/requirements.md)): `uds:<path>` or `tcp:<socket-addr>`. The fixture neither formats endpoint text nor invents endpoints. Windows callers supply distinct endpoints through `endpoint_flag`, `transport.endpoint` or `transport.tcp_port`. The macOS/Linux default is a Unix domain socket. The parallelism test uses two fixtures; capability has no fixture-count limit. `/tmp/...` is an example of a fixed path.

Cleanup uses production `run_with_cleanup` through the common runner; ordinary start supplies empty cleanup. Normal stop and startup failure preserve the primary runtime error. Cleanup entry has no unused setup closures. Successful stores, logger and acquired instance lock remain owned until cleanup completion. Endpoint release preserves production ownership order. Shutdown/drop physically join the fixture thread. Tests cover blocked cleanup, contention, startup failure, parallel fixtures, endpoint closure and startup-timeout no-detachment. No alternate lifecycle or public fixture shutdown trigger is introduced.

Port zero binds once with continuous listener ownership through actual-endpoint reporting. Reserve/drop/rebind and port-race retry/sleep/timeout workarounds are forbidden. Production real-signal child tests use `run_with_cleanup_and_bound(on_bound, cleanup)`: successful bind calls `on_bound` exactly once with the actual owned Endpoint; pre-bind failure calls no bound callback. The recipient confirms health before signaling. Ordinary `run` and `run_with_cleanup` delegate with a no-op bound callback. Existing startup/error/cleanup/ownership order remains binding. This extends [REQ-RT-0001](requirements.md) without changing the transport API.

**MUST statements:**

- The `sc-runtime` crate MUST export the public symbol `sc_runtime::testing::DaemonFixture`.
- Each call to a `DaemonFixture` constructor MUST create a new temporary directory.
- The fixture configuration code in `sc-runtime` MUST assign the new temporary-directory path to `config.transport.instance_root`.
- Fixture startup in `sc-runtime` MUST call the production `Daemon::builder()` path.
- Starting a fixture MUST acquire the OS lock on `<instance-root>/daemon.lock`.
- The production builder used by the fixture MUST receive the application's stores closure through `.stores(...)`.
- The production builder used by the fixture MUST receive the application's routes closure through `.routes(...)`.
- Fixture startup with an MCP closure MUST invoke the supplied MCP closure through the production builder's `.mcp(...)` callback.
- The fixture startup implementation in `sc-runtime` MUST call the `sc-transport` listener-binding API.
- The return type of `DaemonFixture::client()` MUST be `&sc_transport::Client`.
- The connection state of the client returned by `DaemonFixture::client()` MUST be connected to the fixture daemon endpoint.
- The string returned by `DaemonFixture::endpoint_str()` MUST equal the resolved Endpoint's `Display` value.
- `sc_transport::resolve_endpoint` MUST return `Ok(fixture.endpoint().clone())` for `--endpoint <value>` with `<value>` equal to `DaemonFixture::endpoint_str()`.
- The sc-transport resolver MUST return the fixture endpoint for `SC_ENDPOINT=<value>` with `<value>` equal to `DaemonFixture::endpoint_str()`.
- The path returned by `DaemonFixture::instance_root()` MUST equal the fixture's temporary instance-root path.
- The fixture stop implementation in `sc-runtime` MUST invoke the crate-private shutdown trigger.
- The fixture shutdown implementation in `sc-runtime` MUST call the production shutdown runner used by signal handling.
- The fixture daemon MUST be stopped when fixture drop completes.
- The fixture listener MUST be closed when fixture drop returns.
- The fixture listener MUST be closed when the asynchronous `shutdown()` future resolves.
- For a Unix-domain-socket fixture, the selected socket file MUST be absent when fixture drop returns.
- For a Unix-domain-socket fixture, the selected socket file MUST be absent when the asynchronous `shutdown()` future resolves.
- On macOS, `DaemonFixture` MUST support any number of concurrent live instances in one test binary under the default Rust test runner.
- On Linux, `DaemonFixture` MUST support any number of concurrent live instances in one test binary under the default Rust test runner.

**MUST NOT statements:**

- Within macOS/Linux parallel fixture execution, a fixture MUST NOT use another live fixture's socket.
- Within macOS/Linux parallel fixture execution, a fixture MUST NOT use another live fixture's port.
- Within macOS/Linux parallel fixture execution, a fixture MUST NOT use another live fixture's lock file.
- Within macOS/Linux parallel fixture execution, a fixture MUST NOT use another live fixture's database file.
- The public `DaemonBuilder` surface MUST NOT expose the fixture shutdown trigger.
- The fixture shutdown implementation in `sc-runtime` MUST NOT call a process-signal sending API.
- The fixture configuration code in `sc-runtime` MUST NOT assign a fixed instance-root path.
- The instance-root path returned by `DaemonFixture::instance_root()` MUST NOT be inside the current user's home directory.

### Rationale

Every generated project tests its operations against a real daemon. Without
an isolated fixture those tests would share one socket and one database and
could not run in parallel, and could collide with a developer's real daemon;
the repo-level rule that every test uses its own tempdir instance root and
runs in parallel is [NFR-RUN-0005](../requirements.md). The template's worked
example (`widget.create` and `widget.get`) ships with a test built on this
fixture ([REQ-RUN-0302](../requirements.md)). That test drives an MCP client
as well as REST, which is why the fixture must hand out its endpoint and not
only a connected client. It does not run the CLI binary; host-run tests never
do ([REQ-RUN-0312](../requirements.md)).
### Success Criteria

A criterion below that spawns a client child process on a developer's host
spawns a test helper, never the generated CLI binary
([REQ-RUN-0312](../requirements.md)).

1. A test starts a fixture, calls a project route through the fixture's
   client, and asserts the expected response.
2. A test starts several fixtures concurrently in one test binary, each writes
   a distinct value through its own daemon and reads it back, and each asserts
   it sees only its own value.
3. A test asserts two live fixtures report different instance-root paths and
   different endpoints, and a search of `crates/sc-runtime/src` finds no
   literal `/tmp` or home path.
4. A test stops a fixture (by drop, or by the explicit shutdown if one
   exists) and asserts, immediately and without polling, that connecting to
   its former endpoint fails and, for a Unix domain socket endpoint, that the
   socket file no longer exists.
5. A test spawns a child process with `SC_ENDPOINT` set to the fixture's
   endpoint string; the child resolves its endpoint through the
   `sc-transport` resolver, requests a project route, and the test asserts
   the child received that fixture's response.
6. A test stops one of two live fixtures and asserts the other still answers
   a request, showing the stop trigger is per fixture and not a process
   signal.
7. `cargo test -p sc-runtime` passes under the default parallel runner, and
   passes when two such runs execute at the same time.

## REQ-RT-0007: `run()` returns the typed error `RuntimeError`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions: `RuntimeError` is the error enum defined in `crates/sc-runtime/src/error.rs`.
`run()` is the `DaemonBuilder::run` method defined in `crates/sc-runtime/src/builder.rs`.
A variant is one case of the `RuntimeError` enum.
The variant table is the table below that names each variant and the failure for which `run()` returns that variant.
The runtime error inventory is the table in [the crate architecture](architecture.md#runtime-error-inventory) that gives each variant's failure mode, stable code, cause payload and recovery text.
A cause payload is the plain-text failure detail that a variant carries.
The stores closure is the application function passed to `DaemonBuilder::stores` that opens the application's stores.
A store error is an error value whose type the application defines outside `sc-runtime`.
An observability type is a type defined in the `sc-observability-types` crate.
A downstream test is an integration test in `crates/sc-runtime/tests/`; Cargo compiles each such test as a separate crate that sees only the public API of `sc-runtime`.
A per-variant match is a `match` expression on `RuntimeError` that has a separate arm for each of the seven variants.
A source file names a type when its Rust code refers to that type by path.
A trait-bound assertion is a function in a downstream test that compiles only when `RuntimeError` implements the named trait.
A public function signature is the parameter and return types of a `pub` function or method reachable from the `sc-runtime` crate root.
`Box<dyn Error>` is a heap-allocated `std::error::Error` trait object.
`sc_transport::AppName` is the validated application-name type defined in `sc-transport`.
The instance root is the per-application, per-user directory resolved by `sc-transport`, or an explicitly supplied path; its default location is undecided ([REQ-TRN-0002](../sc-transport/requirements.md)).
`daemon.lock` is the lock file `<instance-root>/daemon.lock`.
Lock contention is another process holding the lock on `daemon.lock`.

| Variant | Returned when |
|---|---|
| `LockHeld` | another process holds the lock on `<instance-root>/daemon.lock` |
| `StoresFailed` | the project's stores closure returned this value |
| `InstanceFailed` | the instance root or `daemon.lock` could not be prepared, and the lock is not held |
| `EndpointFailed` | endpoint resolution failed |
| `BindFailed` | `sc-transport` could not bind the listener |
| `ServeFailed` | serving the router ended with an error |
| `RouteConflict` | the project's router already defines `GET /health`, `GET /openapi.json`, or `/mcp` |

Out of scope: the project's `main` maps the result of `run()` to a process exit code. That `run()` does not panic is [NFR-RT-0004](requirements.md). The direct `sc-observability-types` dependency of `sc-runtime` is permitted for injected logging under REQ-RUN-0309.

**MUST statements:**

- `run()` MUST have the return type `Result<(), RuntimeError>`.
- `RuntimeError` MUST be declared `pub` in `sc-runtime`.
- The declaration of `RuntimeError` in `crates/sc-runtime/src/error.rs` MUST have no generic type parameters.
- The variants declared for `RuntimeError` in `crates/sc-runtime/src/error.rs` MUST be exactly the seven variants named in the variant table.
- The `cause` field of each variant MUST hold the cause payload that the runtime error inventory lists for that variant.
- A downstream test MUST compile a trait-bound assertion for `std::error::Error`.
- A downstream test MUST compile a trait-bound assertion for `std::fmt::Debug`.
- A downstream test MUST compile a trait-bound assertion for `std::fmt::Display`.
- A downstream test MUST compile a per-variant match.
- When `run()` fails, `run()` MUST return `Err` holding the variant that the variant table names for the failure.
- When the instance root cannot be resolved, `run()` MUST return `Err(RuntimeError::InstanceFailed)`.
- When opening `daemon.lock` fails for a reason other than lock contention, `run()` MUST return `Err(RuntimeError::InstanceFailed)`.
- When the endpoint cannot be parsed, `run()` MUST return `Err(RuntimeError::EndpointFailed)`.
- When the endpoint cannot be resolved, `run()` MUST return `Err(RuntimeError::EndpointFailed)`.
- In `crates/sc-runtime/src/error.rs`, the `code` field of `RuntimeError::StoresFailed` MUST be declared with type `&'static str`.
- In `crates/sc-runtime/src/error.rs`, the `message` field of `RuntimeError::StoresFailed` MUST be declared with type `String`.
- In `crates/sc-runtime/src/error.rs`, the `remediation` field of `RuntimeError::StoresFailed` MUST be declared with type `String`.
- In `crates/sc-runtime/src/error.rs`, the `cause` field of `RuntimeError::StoresFailed` MUST be declared with type `Option<String>`.
- The signature of `DaemonBuilder::stores` in `crates/sc-runtime/src/builder.rs` MUST require a stores closure whose future outputs `Result<S, RuntimeError>`.

**MUST NOT statements:**

- `run()` MUST NOT call `std::process::exit`.
- A public function signature in `sc-runtime` MUST NOT contain `Box<dyn Error>`.
- The source files under `crates/sc-runtime/src/` MUST NOT name the type of a store error.
- When a downstream test calls `RuntimeError::source()` on a `RuntimeError::StoresFailed` value, `RuntimeError::source()` MUST NOT return `Some`.
- A field of `RuntimeError::StoresFailed` MUST NOT have an observability type.
- `RuntimeError` MUST NOT have an `AppName` variant.
- The runtime error inventory MUST NOT contain an `APP.INVALID_NAME` row.
- `run()` MUST NOT construct `sc_transport::AppName`.
- `run()` MUST NOT invoke the `unreachable!` macro.

### Rationale

The generated `main.rs` turns the result into an exit code and a message for
a person or an agent. "A daemon is already running" calls for a different
action from "the port is taken" or "the database would not open", so the
caller needs a variant to branch on and not a string to parse. The cause
payload lets the caller print the failure detail.
### Success Criteria

1. A test holds the lock, calls `run()`, and matches the result against the
   `LockHeld` variant.
2. A test whose stores closure returns an error matches the result against
   `StoresFailed`, reads `code` as `&'static str`, reads `message` and
   `remediation` as `String`, and reads the optional `cause` text. The test
   does not call `source()` and does not downcast.
3. A test that makes the bind fail (TCP port already in use) matches the
   result against `BindFailed`.
4. A test covers `ServeFailed`, or, if no deterministic trigger exists, an
   inspection confirms the error from `axum::serve` is mapped to `ServeFailed`
   and to no other variant.
5. A test passes an explicit instance root that is a regular file, not a
   directory, calls `run()`, and asserts `InstanceFailed`, that the error is
   not `LockHeld`, and that the stores closure was never called.
6. A test supplies an endpoint override that cannot be parsed as an endpoint,
   calls `run()`, and asserts `EndpointFailed` and that no listener was
   bound.
7. A compile-time assertion in the tests checks
   `RuntimeError: std::error::Error + Send + Sync + 'static`.

---

## REQ-RT-0008: `run()` resolves instance root and endpoint via `sc-transport`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. UDS means Unix domain socket. TCP means Transmission Control Protocol. ADR means architecture decision record. The builder is `DaemonBuilder`, the type that `Daemon::builder` returns. The instance-root function and the endpoint resolver are the two public `sc-transport` functions described below. The endpoint sources are the `--endpoint` flag value, the `SC_ENDPOINT` environment variable, the configured endpoint and the platform default. The configured endpoint is `DaemonConfig.transport.endpoint`. The explicit instance root is `DaemonConfig.transport.instance_root` when it is set. Code is feature-gated when a `#[cfg(...)]` or `#[cfg_attr(...)]` attribute that names a Cargo feature guards it. The daemon is the process that calls `run()`. To bind an endpoint is to listen on it for connections. To parse the command line is to read `std::env::args` or `std::env::args_os`, directly or through a parser such as `clap`. Endpoint logic is computing a socket path, choosing a default address or port, parsing an endpoint string, or applying a precedence between endpoint sources. Calling an `sc-transport` function is not endpoint logic.

This item says where the daemon's lock directory and listening endpoint come
from. `sc-transport` (`crates/sc-transport`) exposes two public functions that
a daemon and a CLI both use: an instance-root function, which returns the
per-user directory for an application name or passes through an explicitly
supplied path ([REQ-TRN-0002](../sc-transport/requirements.md)), and an
endpoint resolver, which returns a Unix domain socket path or a TCP socket
address by taking the first of: the `--endpoint` flag value, the `SC_ENDPOINT`
environment variable, the configured endpoint, the platform default
([REQ-TRN-0001](../sc-transport/requirements.md)). Their sketch names are
`instance_root(app)` and `resolve_endpoint(...)`; the names are illustrative
until the contract sprint pins them.

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](../sc-transport/requirements.md)).

- In the daemon, the component that calls both functions is `run()` of `sc-runtime`.
- `endpoint_flag` carries the flag. `transport` embeds `TransportConfig`, whose `endpoint` and `instance_root` carry the other two.
- The project's `main` parses `--endpoint` and places the value in `DaemonConfig` before calling `Daemon::builder` (decided in this document; it follows from the builder taking only `app` and `&DaemonConfig`).
- Honouring `SC_ENDPOINT` through the resolver is not "loading configuration" in the sense of [REQ-RT-0001](requirements.md) step 1 and [NFR-RT-0002](requirements.md).
- `DaemonConfig` embeds it, and a project's CLI, which cannot name `DaemonConfig` because it does not depend on `sc-runtime`, uses it directly. The field split is recorded in [REQ-RT-0001](requirements.md). The `TransportConfig` field list is the one [REQ-TRN-0001](../sc-transport/requirements.md) froze.

Consequence, not a separate obligation: with an explicit instance root `R`
and no endpoint override, on macOS and Linux the lock file is
`R/daemon.lock` and the socket is `R/daemon.sock`, because both come from the
one instance root.

This item is decided in this document. The design states only that the
daemon binds "through sc-transport" and that `sc-transport` owns endpoint
resolution (instance root, `--endpoint`, `SC_ENDPOINT`); it does not name the
caller or the carrier.

`DaemonConfig.transport` is `sc_transport::TransportConfig` (`endpoint`,
`instance_root`, `tcp_port`). `DaemonConfig.endpoint_flag` is `Option<String>`.
`run` calls `sc_transport::resolve_endpoint_from_env` and passes the flag.
That function reads `SC_ENDPOINT`. The name `SC_ENDPOINT` does not appear in
`sc-runtime`. In-process parallel tests do not set the process environment.
A child process may set `SC_ENDPOINT` when it exercises the resolver.

**MUST statements:**

- `run()` MUST obtain the instance root only by calling the `sc-transport` instance-root function.
- `run()`'s call to the `sc-transport` instance-root function MUST pass the `app` name given to the builder.
- `run()`'s call to the `sc-transport` instance-root function MUST pass the explicit instance root when one is set.
- `run()` MUST obtain the bind endpoint only by calling the `sc-transport` endpoint resolver.
- `run()`'s call to the `sc-transport` endpoint resolver MUST pass the instance root returned by `run()`'s call to the `sc-transport` instance-root function.
- `run()` MUST read the `--endpoint` flag value only from the `DaemonConfig` passed to `Daemon::builder`.
- `run()` MUST read the configured endpoint only from the `DaemonConfig` passed to `Daemon::builder`.
- `run()` MUST read the explicit instance root only from the `DaemonConfig` passed to `Daemon::builder`.
- When the `SC_ENDPOINT` environment variable is set and `DaemonConfig.endpoint_flag` is `None`, the daemon MUST bind the endpoint that `SC_ENDPOINT` names.
- `DaemonConfig.transport` MUST have the type `sc_transport::TransportConfig`.
- `crates/sc-transport/src/config.rs` MUST implement `serde::Deserialize` for `TransportConfig`.

**MUST NOT statements:**

- The generated `daemon/src/main.rs` MUST NOT need to call the instance-root function.
- The generated `daemon/src/main.rs` MUST NOT need to call the endpoint resolver.
- `sc-runtime` MUST NOT contain endpoint logic.
- `sc-runtime` MUST NOT compute a socket path.
- `sc-runtime` MUST NOT choose a default address.
- `sc-runtime` MUST NOT choose a default port.
- `sc-runtime` MUST NOT parse an endpoint string.
- `sc-runtime` MUST NOT apply a precedence between endpoint sources.
- The Phase D `DaemonBuilder` MUST NOT have a method that supplies the `--endpoint` flag value (`DaemonConfig.endpoint_flag`).
- The Phase D `DaemonBuilder` MUST NOT have a method that supplies the configured endpoint (`DaemonConfig.transport.endpoint`).
- The Phase D `DaemonBuilder` MUST NOT have a method that supplies the explicit instance root (`DaemonConfig.transport.instance_root`).
- `sc-runtime` MUST NOT parse the command line.
- The `serde::Deserialize` implementation for `TransportConfig` MUST NOT be feature-gated.

### Rationale

The lock lives in the instance root and, by default, so does the socket. If
`sc-runtime` computed either on its own, the lock directory and the socket
directory could diverge, and a daemon could listen where no CLI looks. One
resolver in one crate, called by both binaries with the same inputs, is what
makes the CLI find the daemon. The daemon honours `SC_ENDPOINT` because a CLI
started with the same variable resolves to that endpoint, and the two would
otherwise never meet. The builder keeps a fixed assembly order, so the
endpoint inputs use a named carrier, and `DaemonConfig` is the only value the project
hands to `run()`. The decision that one resolver is shared by daemon and CLI
is [ADR-TRN-0002](../sc-transport/architecture.md); the template-side
counterpart, that the generated `cli` and `daemon` each accept a global
`--endpoint` and resolve only through `sc-transport`, is
[REQ-RUN-0310](../requirements.md).
### Success Criteria

Criteria that start a daemon in a child process run only on an isolated
machine, never on a developer's host
([NFR-RUN-0010](../requirements.md)).

1. A `#[cfg(unix)]` test starts a daemon with an explicit instance root `R`
   (a tempdir) and no endpoint override, and asserts that while it serves,
   `R/daemon.lock` exists and is locked and `R/daemon.sock` exists and
   accepts a connection.
2. A test starts a daemon whose `DaemonConfig` carries an explicit instance
   root that is a fresh tempdir and an endpoint override (a UDS path in a
   second tempdir on Unix; a free loopback TCP port otherwise). A client that calls the `sc-transport` resolver with the same
   inputs and connects to the result receives a response from that daemon,
   and on Unix no socket file exists at the default path
   `<instance-root>/daemon.sock`.
3. On Unix, two daemon startups use distinct fresh instance roots but the
   same UDS endpoint override. The second fails before replacing the endpoint;
   the first remains reachable, and cleanup from the failed startup cannot
   remove the first daemon's socket.
4. A test starts a daemon in a child process with an explicit instance root
   that is a fresh tempdir, with `SC_ENDPOINT` set, and with no flag value or
   configured endpoint; a client resolving with the same `SC_ENDPOINT` value
   reaches it, and `daemon.lock` exists inside that tempdir.
5. `grep -rn "daemon\.sock\|127\.0\.0\.1" crates/sc-runtime/src` prints no
   line outside `#[cfg(test)]` code. The same holds for `SC_ENDPOINT`, because
   `run` calls `resolve_endpoint_from_env` and does not read the variable.
6. Inspection of `crates/sc-runtime/src`: outside `#[cfg(test)]` code, the
   only source of an instance-root path is the `sc-transport` instance-root
   function, the only source of an endpoint value is the `sc-transport`
   resolver, and there is no use of `std::env::args` or `std::env::args_os`.

---

# Requirement: `sc-runtime` is assembly only, with a closed Phase D facade

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

---

## NFR-RT-0001: `sc-runtime` is assembly only, with a closed Phase D facade

**Status:** Active  

### Requirement Statement

Definitions:
`axum`, `rmcp` and `sqlx` are third-party Rust library crates: `axum` serves HTTP routes, `rmcp` serves Model Context Protocol (MCP) requests and `sqlx` runs SQL queries.
A public item is a module, struct, enum, trait or free function that other crates can name through a public path of `sc-runtime` with `test-util` enabled, whether or not it is marked `#[doc(hidden)]`, or a public method of `Daemon` or `DaemonBuilder`. The typestate markers `NeedStores`, `NeedRoutes` and `Ready` are not public items.
The public-surface table is the table in this record that gives the kind, count and name of each public item of the Phase D deliverable.
A free function is a function declared outside any `impl` block.
The allowed free-function list is `mcp_request_body` and `testing::instrument_router`.
A public data type is a struct or enum declared `pub` in `crates/sc-runtime/src`.
A field typed from a named crate is a struct or enum field whose declared type is a type declared in the named crate.
`test-util` is the non-default Cargo feature of `sc-runtime` that enables the `testing` module.
The shared command path is the one shared write path and one shared read path of PRD §3, supplied by `sc-command`, to which a project supplies `Tcli`, `Tsql` and the transform.
A command or operation registry is any structure in which a project registers operations for the framework to dispatch.

Scope: every rule in this record describes the Phase D deliverable of the `sc-runtime` crate (`crates/sc-runtime`) only. The rules create no work for, and do not prohibit, later consumer options or public API additions. The rules do not restrict `sc-command`, the shared command path, or `sc-storage`, whose `Storage<T>` trait PRD-STO-1 and PRD-STO-2 require.

`sc-runtime` is a bootstrap, not a framework layer. Phase D adds no command or operation registry to `sc-runtime` and builds no procedural or declarative macro in `sc-runtime`, private or exported. Phase D supplies the shared command path and default plain conversion from `sc-command` and builds no generator. Code generation for the command pattern, including macro-based command generation and a later consumer-side generator, is undecided (PRD §9): not built in Phase D and not forbidden later. Supplying the shared command path from `sc-command` does not expand `sc-runtime` beyond assembly.

- The project passes in an ordinary `utoipa_axum::OpenApiRouter` and an ordinary rmcp service, and `sc-runtime` mounts both as given.
- `Daemon::builder` takes `&sc_transport::AppName`. `sc-runtime` does not define or re-export `sc_runtime::AppName`.
- The `testing` module, `DaemonFixture` and `instrument_router` are compiled only with `test-util`.
- The `#[doc(hidden)]` typestate markers `NeedStores`, `NeedRoutes` and `Ready` appear in the builder signature and are not public items.
- The generated `daemon` and `service` crates take `sc-command` directly, as fixed by [REQ-RUN-0301](../requirements.md); `sc-runtime` does not re-export `sc-command`.

| Kind of public item | How many | Name |
|---|---|---|
| Entry type whose only public function is the constructor that returns the builder | one | `Daemon` (`Daemon::builder`) |
| Builder type with exactly seven public methods: three project closure setters, one optional all-phase logger setter, and three alternative ready-state terminal run methods | one | `DaemonBuilder` (`stores`, `routes`, `mcp`, `logger`, `run`, `run_with_cleanup`, `run_with_cleanup_and_bound`) |
| Configuration struct implementing `serde::Deserialize` | one | `DaemonConfig` |
| Error enum | one | `RuntimeError` |
| Public module for tests, containing the fixture and router instrumentation helper, compiled only with `test-util` | one | `testing` (`testing::DaemonFixture`, `testing::instrument_router`) |
| Read-only original MCP request body accessor | one | `mcp_request_body(&axum::http::request::Parts) -> Option<&[u8]>` |

**MUST statements:**

- `cargo doc -p sc-runtime --no-deps --features test-util` MUST document every item that the public-surface table names.

**MUST NOT statements:**

- `crates/sc-runtime/src` MUST NOT declare a public data type that has a field typed from the `axum` crate.
- `crates/sc-runtime/src` MUST NOT declare a public data type that has a field typed from the `rmcp` crate.
- `crates/sc-runtime/src` MUST NOT declare a public data type that has a field typed from the `sqlx` crate.
- `crates/sc-runtime/src` MUST NOT declare a public item that the public-surface table does not name.
- `sc-runtime` MUST NOT have a public free function whose name is absent from the allowed free-function list.
- `crates/sc-runtime/src` MUST NOT contain a `pub use` declaration that re-exports an item defined in another crate.

### Rationale

The repo-level decision [ADR-RUN-0001](../architecture.md) is that the
framework instantiates parts and the project wires them: operations are
ordinary async service functions, and REST, MCP and CLI are produced by the
standard derives of axum, rmcp and clap. A registry or wrapper type in
this crate would put the framework back between the project and those
libraries, and would be code every project has to learn.

### Success Criteria

1. Phase D scope check: `crates/sc-runtime/Cargo.toml` has no
   `proc-macro = true`; inspection of the crate source finds no procedural
   or declarative macro defined or exported by `sc-runtime`. This checks the
   Phase D deliverable and imposes no restriction on later consumer-side generation.
2. The `[public] facade` list of `boundaries/sc-runtime/sc-runtime.toml` is
   `Daemon, DaemonBuilder, DaemonConfig, RuntimeError, mcp_request_body, testing`. It does not
   list `AppName`. It contains exactly one entry type, one builder type, one
   configuration struct, one error enum, the one read-only raw-body accessor and one test module, and nothing else.
   `just lint` (which runs `sc-lint lint sc-boundary`) is clean. A default
   build does not expose `testing`. With `test-util`, the module exposes
   `DaemonFixture` and `instrument_router`; the latter accepts and returns an
   ordinary `axum::Router`, and accepts `sc_command::observability::SharedCanonicalLog`,
   a request limit, a shutdown watch receiver and a drain limit.
3. Inspection of the crate's rustdoc index (`cargo doc -p sc-runtime
   --no-deps`, and again with `--features test-util`): the builder type has
   exactly seven public methods (`stores`, `routes`, `mcp`, `logger`, `run`,
   `run_with_cleanup`, `run_with_cleanup_and_bound`), the entry
   type has exactly one public function (`builder`), and the index does not
   list `sc_runtime::AppName`. `testing` appears only when `test-util` is on;
   its helper composes the existing timeout and tracking/logger middleware.
   The terminal run methods exist only in Ready; logger remains same-phase.
   Cleanup and bound receipt reuse the fixed runner; no additional public
   setter, registry, wrapper, typestate or callback mechanism is part of the Phase D facade.
4. Phase D scope check: the `arch-qa` architecture review confirms that the
   delivered assembly facade adds no library-hiding wrapper type or operation
   registry. This check does not restrict later consumer options or API additions.

## NFR-RT-0002: No sqlx, no `sc-observability`, no config loading

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions:

- The dependency tree of `sc-runtime` is every package that `cargo tree -p sc-runtime -e normal --prefix none` prints.
- `sc-observability` is the logging runtime crate; `sc-observability-otlp` is its OpenTelemetry export crate; `sc-observability-types` is the types-only crate of the same family.
- Runtime source is the Rust code under `crates/sc-runtime/src`, excluding `#[cfg(test)]` code.
- Initialising logging means constructing a logger or a log sink.
- Loading configuration means building a configuration value by reading a config file, by reading an environment variable or by calling `sc_config::load`.
- A config file is a file whose contents supply configuration values to the daemon.

Context: `sc-runtime` receives a `&DaemonConfig` that the project's `main.rs` has already loaded. `run` calls `sc_transport::resolve_endpoint_from_env`, which reads the `SC_ENDPOINT` endpoint override in `sc-transport`; [REQ-RT-0008](requirements.md) requires this call, and it is not loading configuration by runtime source. `sc-observability-types` is a direct dependency used only for the injected canonical logger contract. `sqlx`, `sc-observability` and `sc-observability-otlp` are not workspace members, and sc-lint 0.8.0 rejects a forbidden edge whose target is absent, so `forbidden_edges` does not name them and `cargo tree` checks them. The repo-level owner of the forbidden edges of all workspace library crates is [REQ-RUN-0005](../requirements.md).

**MUST statements:**

- The output of `cargo tree -p sc-runtime -e normal --depth 1 --prefix none` MUST contain a line beginning `sc-observability-types `.

**MUST NOT statements:**

- The dependency tree of `sc-runtime` MUST NOT contain `sqlx`.
- The dependency tree of `sc-runtime` MUST NOT contain a crate whose name begins `sqlx-`.
- The dependency tree of `sc-runtime` MUST NOT contain `sc-observability`.
- The dependency tree of `sc-runtime` MUST NOT contain `sc-observability-otlp`.
- Runtime source MUST NOT initialise logging.
- Runtime source MUST NOT contain a `pub use` item whose path has the first segment `sc_observability`.
- Runtime source MUST NOT contain a `pub use` item whose path has the first segment `sc_observability_otlp`.
- Runtime source MUST NOT load configuration.
- Runtime source MUST NOT read a config file.
- Runtime source MUST NOT call `sc_config::load`.
- Runtime source MUST NOT read an environment variable.

### Rationale

Stores are project-owned code, one crate per database backend, so sqlx
belongs there ([ADR-RUN-0301](../architecture.md)). Observability is
initialised directly by the project's `main.rs` with plain values, so that
neither this crate nor `sc-config` becomes unusable without it
([ADR-RUN-0004](../architecture.md)). Keeping config loading in `main.rs`
lets the project report a config error before anything else starts.
### Success Criteria

1. `cargo tree -p sc-runtime -e normal --prefix none` prints no line
   beginning with `sqlx ` (the name then a space), no line beginning with
   `sqlx-`, no line beginning with `sc-observability ` and no line beginning
   with `sc-observability-otlp `. A line beginning with
   `sc-observability-types ` is allowed.
2. The boundary manifest under `boundaries/sc-runtime/` lists
   `sc-runtime -> sc-config` under `[dependencies] forbidden_edges`. It does
   not list `sqlx`, `sc-observability`, `sc-observability-otlp`, or
   `sc-observability-types`, because sc-lint 0.8.0 rejects a forbidden edge
   whose target is not a workspace member. `just lint` (which runs
   `sc-lint lint sc-boundary`) is clean. Criterion 1 is the `cargo tree` check
   for `sqlx` and the forbidden logging-runtime crates.
3. A search of `crates/sc-runtime/src` (excluding `#[cfg(test)]` code) finds
   no call to `sc_config::load`, no read of a config file, and no read of
   `SC_ENDPOINT`. `run` calls `sc_transport::resolve_endpoint_from_env`, which
   reads that variable.

---

## NFR-RT-0003: Allowed dependencies of the `sc-runtime` crate

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

The public package set is the nine crates of REQ-RUN-0002. This item owns only the list of third-party dependencies `sc-runtime` is allowed.

Definitions: The Allowed dependencies table is the table below; each row names one crate that `sc-runtime` may list under `[dependencies]` in `crates/sc-runtime/Cargo.toml`. `sc_config::load` is the function of the `sc-config` crate that loads a project's configuration ([REQ-CFG-0001](../sc-config/requirements.md)). A configuration file is any file whose contents supply configuration values to the daemon, including `config/default.json` and `config/local.json`, which `sc_config::load` reads. Reading a file means obtaining its contents at run time. The source files under `crates/sc-runtime/src` are the `.rs` files in that directory and its subdirectories. The direct dependency set of `sc-runtime` is the set of crate names that `cargo tree -p sc-runtime -e normal --depth 1 --prefix none` prints, one per line before the version, without `sc-runtime` itself.

Allowed dependencies table:

| Dependency | Why it is needed |
|---|---|
| `sc-transport` with `features = ["server"]` | listener binding, the endpoint resolver and the instance-root function |
| `sc-command` with `features = ["server"]` | the axum response and rmcp tool-result conversions of the response envelope, including `GET /health`; temporary canonical port under ADR-RUN-0017 |
| `sc-observability-types` | canonical request lifecycle values; no sink construction or shutdown |
| `axum` | the merged `axum::Router` and serving |
| `tokio` | the async runtime and signal handling |
| `fd-lock` | the exclusive lock on `daemon.lock` |
| `utoipa-axum` | the routes closure returns `utoipa_axum::OpenApiRouter` |
| `utoipa` | the OpenAPI document is a `utoipa` type |
| `serde` | `DaemonConfig` MUST implement `Deserialize`, because the project loads it as `cfg.daemon` through `sc-config` |
| `rmcp` | `.mcp(...)` accepts the concrete `StreamableHttpService` and mounts it as given. This crate does not wrap that type and does not re-export it |
| `tracing` | the request-id span on the assembled router, including `/health`, `/openapi.json`, `/metrics`, and optional `/mcp` |

- The edge `sc-runtime -> sc-config` is absent.
- `rmcp` is a direct dependency. `.mcp(...)` accepts the concrete rmcp
  service type and mounts it as given. `sc-runtime` does not wrap that type
  and does not re-export it ([ADR-RUN-0001](../architecture.md) point 4).
- `testing::DaemonFixture` sits behind the non-default cargo feature
  `test-util`. A production daemon build does not enable it and does not link
  the fixture. The temporary-directory crate is `tempfile`, an optional
  dependency of `test-util` only. It is not a normal dependency.
- `sc-runtime` is a dependency of a project's daemon binary and tests. That a
  project's CLI does not depend on it, and the `cargo tree -p cli` check for
  that, are owned by [NFR-RUN-0001](../requirements.md). Sprint a-6 runs that
  command.

REQ-RUN-0005 owns the workspace-internal and forbidden edges of all workspace library crates. `sc-runtime` is the assembly crate; its only workspace dependencies are `sc-transport` and `sc-command`. `sc-cron` depends on `sc-storage` and `sc-monty` (PRD-CRON-2, PRD-MONTY-2). [NFR-RT-0002](requirements.md) owns the ban on configuration loading, `sqlx` and the `sc-observability` runtime crates; the statements below repeat those bans for this crate's dependency list.

**MUST statements:**

- The direct dependency set of `sc-runtime` MUST equal the set of crates named in the Allowed dependencies table.

**MUST NOT statements:**

- The output of `cargo tree -p sc-config -e normal --prefix none` MUST NOT contain a line beginning `sc-runtime `.
- The output of `cargo tree -p sc-config -e normal --prefix none` MUST NOT contain a line beginning `sc-transport `.
- The output of `cargo tree -p sc-config -e normal --prefix none` MUST NOT contain a line beginning `sc-command `.
- The output of `cargo tree -p sc-transport -e normal --prefix none` MUST NOT contain a line beginning `sc-runtime `.
- The output of `cargo tree -p sc-transport -e normal --prefix none` MUST NOT contain a line beginning `sc-config `.
- The output of `cargo tree -p sc-transport -e normal --prefix none` MUST NOT contain a line beginning `sc-command `.
- The output of `cargo tree -p sc-command -e normal --prefix none` MUST NOT contain a line beginning `sc-runtime `.
- The output of `cargo tree -p sc-command -e normal --prefix none` MUST NOT contain a line beginning `sc-config `.
- The output of `cargo tree -p sc-command -e normal --prefix none` MUST NOT contain a line beginning `sc-transport `.
- The output of `cargo tree -p sc-runtime -e normal --prefix none` MUST NOT contain a line beginning `sc-config `.
- Source files under `crates/sc-runtime/src` MUST NOT call `sc_config::load`.
- Source files under `crates/sc-runtime/src` MUST NOT read a configuration file.
- The output of `cargo tree -p sc-runtime -e normal --prefix none` MUST NOT contain a line beginning `sqlx `.
- The output of `cargo tree -p sc-runtime -e normal --prefix none` MUST NOT contain a line beginning `sc-observability `.
- The output of `cargo tree -p sc-runtime -e normal --prefix none` MUST NOT contain a line beginning `sc-observability-otlp `.

### Rationale

The CLI must not link Axum, rmcp or sqlx, yet daemon and CLI both use
`sc-transport` and `sc-command`. The repo-level decision
[ADR-RUN-0003](../architecture.md) resolves this by putting server-side code
behind a `server` feature that is off by default; `sc-runtime` turns it on
for the daemon side. Whether a generated `daemon` crate also names
`sc-command` with `server` directly is open in
[REQ-RUN-0301](../requirements.md). A closed dependency list that matches
what the API actually names lets the boundary linter accept a conforming
crate; the list in the design (the three crates, `axum`, `tokio`, `fd-lock`)
would reject one.
### Success Criteria

1. `crates/sc-runtime/Cargo.toml` declares `sc-transport` with
   `features = ["server"]` and `sc-command` with `features = ["server"]`.
2. Every entry under `[dependencies]` in `crates/sc-runtime/Cargo.toml` is
   one of: `sc-transport`, `sc-command`, `sc-observability-types`, `axum`, `tokio`, `fd-lock`,
   `utoipa-axum`, `utoipa`, `serde`, `rmcp`, `tracing`, or optional `tempfile` enabled
   only by `test-util`. A search of `crates/sc-runtime/src` finds at least one
   `sc_command` use. The criterion fails when that use is absent.
   `cargo tree -p sc-runtime -e normal --prefix none` prints no `tempfile` or
   `tempdir` line. `cargo tree -p sc-runtime -e features --prefix none` shows
   `tempfile` only when `--features test-util` is set.
3. `sc-config` is not declared. A search of `crates/sc-runtime/src` finds no
   `sc_config` item.
4. The boundary manifest under `boundaries/sc-runtime/` records the same
   allowed dependencies under `[dependencies] allowed_dependencies`, and
   `just lint` (which runs `sc-lint-boundary`) is clean.

---

## NFR-RT-0004: No public function in `sc-runtime` panics

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions:

- The public package set is `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`, `sc-runtime-db-sqlite`, `sc-storage` and `sc-monty` ([REQ-RUN-0002](../requirements.md)).
- A failure outcome of a `pub` function or `pub` method is an error that code of its own crate detects, for example invalid caller input, a held lock, a missing directory, a port in use or a stores failure.
- A typed error enum is the error enum that one crate returns from its `pub` functions and `pub` methods that have a failure outcome.
- The own result type of a crate is `Result<T, E>`, or an alias of it, in which `T` is any success type and `E` is the typed error enum of that crate.
- `RuntimeError` is the typed error enum of `sc-runtime` ([REQ-RT-0007](requirements.md)).
- Test code is code under `#[cfg(test)]` or in `tests/`.
- Covered code is every function body, other than test code, under `crates/<crate>/src` of a crate in the public package set that is a `pub` function of that set or that such a function calls directly or indirectly.
- A recorded exception is a signature or construct that [NFR-RUN-0009](../requirements.md) names as an exception to its error rule or panic rule.
- Calling `unwrap` or `expect` is any method call with that name, whatever the receiver type.
- A panicking index expression is a `[]` index on a slice, array, `Vec` or map that panics when the index or key is absent.

Each statement below excludes a recorded exception, and nothing else; an explanatory comment is not a recorded exception. NFR-RUN-0009 owns the repo-level rule; `sc-runtime` adds no strictness beyond it.

Applied to `sc-runtime`: no public function or method of `sc-runtime` panics on caller input or on environment state (lock held, missing directory, port in use, stores failure, signal delivery). A panic raised inside a project closure belongs to the project; `sc-runtime` need not catch it.

**MUST statements:**

- If a `pub` function or `pub` method of `sc-runtime` has a failure outcome, its declared return type MUST be `Result<T, RuntimeError>`.
- If a `pub` function or `pub` method of a crate in the public package set has a failure outcome, its declared return type MUST be the own result type of that crate.

**MUST NOT statements:**

- Covered code MUST NOT call `unwrap`.
- Covered code MUST NOT call `expect`.
- Covered code MUST NOT invoke `panic!`.
- Covered code MUST NOT invoke `unreachable!`.
- Covered code MUST NOT invoke `todo!`.
- Covered code MUST NOT invoke `unimplemented!`.
- Covered code MUST NOT contain a panicking index expression.

### Rationale

These crates run inside long-lived daemons and under CLIs driven by agents; a
panic there is an outage or a failure no caller can parse. The repo-level
decision [ADR-RUN-0006](../architecture.md) is that every library API returns
`Result<T, E>` with one typed error enum per crate.
### Success Criteria

1. `grep -rn "unwrap()\|\.expect(\|panic!\|unreachable!\|todo!\|unimplemented!"
   crates/sc-runtime/src` prints no line outside `#[cfg(test)]` code, or each
   remaining match is shown by review to be unreachable from a public
   function. Review of the same code finds no `[]` indexing of a slice or map
   that can panic on a path reachable from a public function.
2. Four tests in `crates/sc-runtime`, each of which calls `run()` in a bad
   environment and asserts that the call returns an `Err(RuntimeError)` value
   and that the test thread did not panic: (a) the lock on the instance root
   is already held; (b) the stores closure returns an error; (c) the TCP port
   to bind is already in use; (d) the explicit instance root is a regular
   file, not a directory.
3. Inspection of the public API: every fallible public function or method of
   `sc-runtime`, including the constructor of `testing::DaemonFixture`,
   returns `Result<_, RuntimeError>`, and no public signature names
   `anyhow::Error` or `Box<dyn Error>`.

Phase C raw-input clarification: `mcp_request_body` only reads privately retained
original JSON POST bytes from ordinary RMCP HTTP Parts. The attachment retains
the service's configured byte cap, original headers, bytes and absolute deadline.
The application owns canonical DTO validation and arbitrary JSON payload semantics;
runtime adds no global duplicate-key rule or command registry. Requests outside
that path return None. This accessor is the only free function that NFR-RT-0001 allows in a build without `test-util`. Normal serde_json is unnecessary and remains outside NFR-RT-0003's
exact dependency budget; JSON parsers used only in runtime tests are dev edges.
