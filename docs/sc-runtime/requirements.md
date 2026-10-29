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

### Requirement Statement

The crate `sc-runtime` (`crates/sc-runtime`) MUST expose a builder that
assembles and runs a daemon. The names below are pinned. The shape and the
order are binding. `run` exists only after `stores` and `routes` (consuming
typestate, [ADR-RT-0001](architecture.md)).

| Call | Argument | Required |
|---|---|---|
| `Daemon::builder(app, &daemon_config)` | `app`: `&sc_transport::AppName`; `daemon_config`: a `&DaemonConfig` the project has already loaded. This crate does not define `sc_runtime::AppName` | yes |
| `.stores(closure)` | an async closure taking no arguments that opens the project's stores and returns the project's own `Stores` value or an error | yes |
| `.routes(closure)` | a closure that receives the `Stores` value and returns a `utoipa_axum::OpenApiRouter` | yes |
| `.mcp(closure)` | a closure that receives the `Stores` value and returns an rmcp Streamable HTTP service | no |
| `.logger(handle)` | a `sc_observability_types::SharedCanonicalLog`; optional in every builder state and replaces the fixed `sc-runtime` Noop | no |
| `.run().await` | none; returns `Result<(), RuntimeError>` | yes |

Intended use, in the `daemon/src/main.rs` of a generated project:

```rust
#[tokio::main]
async fn main() -> ExitCode {
    // config is loaded by the project with sc-config, before the builder
    let cfg: AppConfig = match sc_config::load("my-app") {
        Ok(cfg) => cfg,
        Err(e) => return report(e),
    };
    // observability is initialised by the project, not by sc-runtime
    let _obs = sc_observability::init(cfg.logging.clone());

    let app = match sc_transport::AppName::new("my-app") {
        Ok(app) => app,
        Err(e) => return report(e),
    };
    let result = sc_runtime::Daemon::builder(&app, &cfg.daemon)
        .stores(|| async { Stores::open(&cfg.stores).await })   // project-defined struct
        .routes(|stores| routes::router(stores))                // utoipa_axum::OpenApiRouter
        .mcp(|stores| mcp::service(stores))                     // optional; an rmcp service
        .run()
        .await;
    exit_code(result)
}
```

`DaemonConfig` is a public struct of `sc-runtime`. It MUST implement
`serde::Deserialize`. This follows from the usage above: the project loads it
as the `daemon` section of its own config (`cfg.daemon`) through `sc-config`,
which deserialises with serde.

`run()` MUST perform exactly these five steps, in this order:

1. Take the `DaemonConfig` passed to `builder`. `run()` MUST NOT read a config
   file or an environment variable to load configuration. Honouring the
   `SC_ENDPOINT` override through the `sc-transport` endpoint resolver is not
   loading configuration ([REQ-RT-0008](requirements.md)).
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

Endpoint text is resolved before the lock so an invalid endpoint does not
open stores or take the lock. Signal streams are registered before bind.
`run()` MUST NOT call the stores closure before the lock is held.
`run()` MUST NOT bind a listener before steps 1 to 4 have succeeded.
If a step fails, `run()` MUST return an `Err(RuntimeError)` and MUST NOT
execute any later step.
The builder MUST NOT offer hooks, callbacks or configuration that insert work
between steps or reorder them (decided in this document and recorded in
[ADR-RT-0001](architecture.md); the design lists the five steps and says the
crate is a bootstrap, not a framework layer).
`.logger(...)` only replaces the event handle carried by the builder. It is
available on `NeedStores`, `NeedRoutes`, and `Ready`, adds no typestate, and
does not add or reorder a `run()` step. The required transition remains
`NeedStores` -> `NeedRoutes` -> `Ready`.

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

### Rationale

Before this crate, every project hand-assembled lock, stores, router and
listener, each slightly differently. One builder makes start-up identical in
every project. The order is the safety property: the lock comes before the
stores so a second daemon never opens the local database, and binding comes
last so no request is accepted before it can be served.

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
6. Inspection of the public API of `crates/sc-runtime`: `DaemonBuilder` has no
   public method other than `stores`, `routes`, `mcp` and `run`.
7. A test deserialises a `DaemonConfig` from a JSON document with
   `serde_json` (a dev-dependency) and passes it to `Daemon::builder`; it
   compiles, which shows `DaemonConfig: serde::Deserialize`.

---

## REQ-RT-0002: Hard singleton through an OS lock on `daemon.lock`

**Status:** Active  

### Requirement Statement

`sc-runtime` MUST enforce that at most one daemon runs per instance root.
`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](../sc-transport/requirements.md)). Tests supply a
tempdir as the explicit path.

- The mechanism MUST be an OS exclusive file lock, taken with the `fd-lock`
  crate, on the file `<instance-root>/daemon.lock`.
- The lock MUST be taken by `run()` before the stores closure is called and
  before any listener is bound.
- The lock MUST be held until the process exits or `run()` returns.
- The attempt to lock MUST NOT block waiting for the holder: it is a
  non-blocking try-lock (decided in this document; the design states the
  exclusive lock but not whether the attempt waits). If another process holds
  the lock, `run()` MUST return a `RuntimeError` whose variant means "lock
  held" (illustrative name `RuntimeError::LockHeld`;
  [REQ-RT-0007](requirements.md) defines the variants).
- In that case the stores closure MUST NOT be called and a listener MUST NOT
  be bound.
- The crate MUST NOT use a PID file or probe a port to detect another daemon.

A failure to open or lock `daemon.lock` that is not lock contention (for
example permission denied, or the instance root is not a directory) MUST make
`run()` return `Err(RuntimeError::InstanceFailed)` without calling the stores
closure. That variant is not `LockHeld` ([REQ-RT-0007](requirements.md)).

`run()` creates the instance root when the path is absent. It rejects an
existing file as `InstanceFailed` and then calls `create_dir_all` before it
opens `daemon.lock`. Whether `sc_transport::instance_root` itself creates a
directory stays with [REQ-TRN-0002](../sc-transport/requirements.md). This
crate creates the directory it is about to lock.

### Rationale

The daemon is the only process allowed to open the local database; two
daemons on one SQLite file would mean two writers. An OS lock is released by
the kernel whenever the process ends, including a crash, so there is no
stale-PID file to clean up and no race between checking and claiming.

### Success Criteria

1. A test starts one daemon on a tempdir instance root, then calls `run()` for
   a second daemon on the same instance root, and asserts the second returns
   the "lock held" `RuntimeError` variant without blocking (the call returns
   within the test timeout while the first daemon is still running).
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

### Requirement Statement

`Stores` is a plain struct defined by the generated project (for example one
holding a SQLite store and a Postgres store). In `sc-runtime` it MUST be a
generic type parameter of the builder.

- `sc-runtime` MUST obtain the `Stores` value only from the project's stores
  closure.
- `sc-runtime` MUST pass that value to the project's routes closure and, if
  present, the project's MCP closure, and MUST NOT do anything else with it.
- `sc-runtime` MUST NOT read a field of, call a method on, or require a trait
  describing the contents of `Stores`.
- The only bounds on the `Stores` parameter MUST be those needed to move it
  into the closures and across the async runtime (for example `Send`,
  `'static`, `Clone`). The exact bounds are pinned with the closure
  signatures by the contract sprint.
- `sc-runtime` MUST NOT open a database pool or run a migration. Those happen
  inside the project's stores closure, in the project-owned store crates.

`sc-runtime` does not depend on `sqlx`. That rule and its two checks (the
`cargo tree` command and the boundary manifest entry) are stated once, in
[NFR-RT-0002](requirements.md).

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

### Requirement Statement

In step 4 of `run()` (after the lock is held and the stores closure has
returned, before anything is bound), `sc-runtime` MUST build one
`axum::Router` containing the following. It is one `axum::Router`, served
unchanged on every listener `run()` binds.

| Mounted item | Source | Condition |
|---|---|---|
| The project's REST routes | the `utoipa_axum::OpenApiRouter` returned by the routes closure | always |
| The OpenAPI document as JSON (`openapi.json`) | the `utoipa` OpenAPI document produced by the same `OpenApiRouter` (utoipa-axum's `split_for_parts`) | always |
| A health route | supplied by `sc-runtime` | always |
| The MCP endpoint at `/mcp` | the rmcp Streamable HTTP service (`StreamableHttpService`) returned by the MCP closure, mounted with `Router::nest_service("/mcp", service)` | only if `.mcp(...)` was called |

- The routes closure MUST return a `utoipa_axum::OpenApiRouter`; `sc-runtime`
  MUST NOT define its own router type.
- The OpenAPI document served MUST be the one generated from the project's
  route registrations; `sc-runtime` MUST NOT require the project to register
  the document route itself.
- `sc-runtime` MUST mount the project's router and the rmcp service as given,
  without wrapping them in its own types.
- `sc-runtime` MUST NOT add an MCP session store or any other session state
  of its own. Configuring the rmcp service in stateless mode is done by the
  generated project's `mcp::service`, and is required of the template by
  [REQ-RUN-0302](../requirements.md); `sc-runtime` mounts what it is given.
- If `.mcp(...)` was not called, `sc-runtime` MUST NOT mount anything at
  `/mcp`; with no project route or fallback there, axum answers HTTP 404.
- MCP MUST NOT get a listener, port or process of its own; it is reachable
  only as `/mcp` on the one router.

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

---

## REQ-RT-0005: Graceful shutdown on SIGINT and SIGTERM

**Status:** Active  

### Requirement Statement

While serving (step 5 of `run()`), `sc-runtime` MUST begin graceful shutdown
when the process receives SIGINT or SIGTERM. On Windows the trigger is Ctrl-C
(decided in this document; the design names only SIGINT and SIGTERM).

On shutdown `run()` MUST:

1. Stop accepting new connections.
2. Let requests already in flight run to completion and send their responses.
3. Stop serving on the listener while retaining its UDS endpoint ownership.
4. Before returning, if the endpoint is a Unix domain socket, remove the
   selected socket file while the listener still holds its endpoint lock,
   then release that endpoint lock (decided in this document and
   ADR-TRN-0006). Do not delete the endpoint-lock file.
5. Before returning, release the lock on `<instance-root>/daemon.lock`.
6. Return `Ok(())` (decided in this document, following from the
   `Result<(), RuntimeError>` return type in [REQ-RT-0007](requirements.md)).

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](../sc-transport/requirements.md)).

Steps 1 to 5 happen in that order. Endpoint cleanup must precede release of
endpoint ownership; the instance-root lock is released afterwards.

`run()` MUST NOT delete `daemon.lock`; releasing the lock is sufficient.

The same six-step sequence MUST run when shutdown is started by the
crate-private trigger that `testing::DaemonFixture` uses to stop its own
daemon ([REQ-RT-0006](requirements.md)). That trigger MUST NOT be public.

**OPEN:** the drain timeout, that is how long shutdown waits for in-flight
requests before giving up, and what `run()` returns if it is exceeded, is not
stated by the design.

**OPEN:** Windows signal handling beyond Ctrl-C (Ctrl-Break, console close,
service stop) and the API used to receive it are not stated by the design.

### Rationale

Daemons are restarted by people, by service managers and by tests. A restart
must not drop a request that was already accepted, and must not leave a
socket file behind that the next start or a client would trip over.

### Success Criteria

Criteria that start a daemon in a child process run only on an isolated
machine, never on a developer's host
([NFR-RUN-0010](../requirements.md)).

1. A `#[cfg(unix)]` test starts a daemon in a child process with a route that
   sleeps before replying, sends a request, sends SIGTERM to the child while
   the request is in flight, and asserts the client receives the complete
   successful response.
2. The same test asserts the child then exits, that a new connection attempt
   after the signal fails, and that the socket file no longer exists.
3. The same scenario with SIGINT passes the same assertions.
4. After the child exits, a new daemon on the same instance root starts
   successfully, proving the lock was released.
5. A test asserts `run()` returns `Ok(())` after a signal-initiated shutdown
   (the child's exit code is the success code).
6. Windows. Until the OPEN on the Windows signal API is decided: inspection of
   the signal-handling code in `crates/sc-runtime/src` confirms that, under
   `#[cfg(windows)]`, Ctrl-C starts the same shutdown sequence as SIGINT. Once
   it is decided: a `#[cfg(windows)]` test sends `CTRL_C_EVENT` to a child
   daemon while a request is in flight and asserts the client receives the
   complete successful response and the child exits with the success code.

---

## REQ-RT-0006: Test fixture `DaemonFixture` in the `testing` module

**Status:** Active  

### Requirement Statement

`sc-runtime` MUST expose `sc_runtime::testing::DaemonFixture` for use in the
tests of generated projects and of this workspace. The name is illustrative
until the contract sprint pins it.

- Starting a fixture MUST create a new temporary directory and use it as the
  daemon's instance root (the directory that holds `daemon.lock` and, on
  macOS and Linux, the socket file `daemon.sock`), supplied to the daemon as
  an explicit instance-root path.
- The fixture MUST start the daemon through the same `Daemon::builder()` path
  as production: real `daemon.lock`, the project's stores, routes and optional
  MCP closures, a real listener bound through `sc-transport`.
- The fixture MUST expose an `sc_transport::Client` already connected to that
  daemon's endpoint (decided in this document).
- The fixture MUST expose its resolved endpoint, including in the string form
  accepted by the `--endpoint` flag and the `SC_ENDPOINT` environment
  variable, so that a test can point a plain HTTP or MCP client at the
  daemon, and so that an isolated-machine test can point the CLI binary at it
  (decided in this document). The string is produced
  by the endpoint-to-string conversion that `sc-transport` provides
  ([REQ-TRN-0001](../sc-transport/requirements.md)); `sc-runtime` does not
  format it.
- The fixture MUST expose its instance-root path (decided in this document).
- The fixture MUST stop its daemon through a crate-private trigger that runs
  the same shutdown sequence a signal starts (stop accepting, drain in-flight
  requests, close the listener, remove the socket file, release the lock:
  [REQ-RT-0005](requirements.md)). There MUST NOT be a public builder method
  for it, and the fixture MUST NOT send a process signal, because a signal
  would stop every fixture in the test binary (decided in this document).
- Dropping the fixture MUST stop the daemon. The operation a test uses to stop
  it (drop, or an explicit async `shutdown()` if the contract sprint adds one)
  MUST return only after the listener is closed and, for a Unix domain socket
  endpoint, the socket file is removed (decided in this document).
- On platforms whose default endpoint is a Unix domain socket (macOS and
  Linux), any number of fixtures MUST be able to run at the same time inside
  one test binary under the default parallel test runner, with no shared
  socket, port, lock file or database file. On Windows the caller supplies a
  distinct endpoint for each fixture. The fixture does not choose one.
- The fixture MUST NOT use a fixed path such as `/tmp/...` or a path under the
  user's home directory.

`testing::DaemonFixture` sits behind the non-default cargo feature `test-util`
([NFR-RT-0003](requirements.md)). The temporary directory is `tempfile`, an
optional dependency of that feature only.

`DaemonFixture::start(app, config, stores, routes)` and
`DaemonFixture::start_with_mcp(app, config, stores, routes, mcp)` call
production `Daemon::builder`. The fixture always replaces
`config.transport.instance_root` with a fresh temporary directory. Accessors
are `client()`, `endpoint()`, `endpoint_str()`, `instance_root()`, and
`shutdown()`. `endpoint_str()` is `Endpoint`'s `Display` (`uds:<path>` or
`tcp:<socket-addr>`), which `sc-transport` owns
([REQ-TRN-0001](../sc-transport/requirements.md)). The fixture does not format
that string and does not invent an endpoint. On Windows the caller supplies
`endpoint_flag`, `transport.endpoint`, or `transport.tcp_port` before
`start`. The parallelism test runs two fixtures.

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

---

## REQ-RT-0007: `run()` returns the typed error `RuntimeError`

**Status:** Active  

### Requirement Statement

`run()` MUST return `Result<(), RuntimeError>`. `RuntimeError` is a public
non-generic enum in `sc-runtime`. Public signatures contain no `Box<dyn Error>`.
The variants are pinned. The inventory (failure mode, stable code, cause
payload, recovery text) is in [the crate architecture](architecture.md#runtime-error-inventory).

| Variant | Returned when |
|---|---|
| `LockHeld` | another process holds the lock on `<instance-root>/daemon.lock` |
| `StoresFailed` | the project's stores closure returned this value |
| `InstanceFailed` | the instance root or `daemon.lock` could not be prepared, and the lock is not held |
| `EndpointFailed` | endpoint resolution failed |
| `BindFailed` | `sc-transport` could not bind the listener |
| `ServeFailed` | serving the router ended with an error |
| `RouteConflict` | the project's router already defines `GET /health`, `GET /openapi.json`, or `/mcp` |

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](../sc-transport/requirements.md)).

- Each variant MUST carry the underlying error so the caller can print it.
- `RuntimeError` MUST implement `std::error::Error`, `Debug` and `Display`.
- A caller MUST be able to distinguish the variants with a `match`; in
  particular "daemon already running" (`LockHeld`) MUST be distinguishable
  from "cannot bind" (`BindFailed`).
- `run()` MUST NOT call `std::process::exit`; mapping the result to an exit
  code is done by the project's `main`. That `run()` does not panic is
  [NFR-RT-0004](requirements.md).
- Every failure of `run()` MUST be returned as an `Err(RuntimeError)`. A
  failure in resolving the instance root or in opening `daemon.lock` is
  `InstanceFailed`, not `LockHeld`. A failure to parse or resolve the endpoint
  is `EndpointFailed`, not `BindFailed`.
- `StoresFailed` is non-generic. Its `code` is `&'static str`. Its `message`,
  `remediation`, and optional `cause` are `String`. The application maps its
  store error into those fields inside the stores closure. `sc-runtime` does
  not name the foreign error type, does not implement `source()` for a
  downcast, and does not declare a direct dependency on
  `sc-observability-types`. There is no `AppName` variant and no
  `APP.INVALID_NAME` row. `run` does not construct `sc_transport::AppName`
  and does not use `unreachable!`.

### Rationale

The generated `main.rs` turns the result into an exit code and a message for
a person or an agent. "A daemon is already running" calls for a different
action from "the port is taken" or "the database would not open", so the
caller needs a variant to branch on and not a string to parse.

### Success Criteria

1. A test holds the lock, calls `run()`, and matches the result against the
   `LockHeld` variant.
2. A test whose stores closure returns an error matches the result against
   `StoresFailed` and reads `code` as `&'static str` and `message`,
   `remediation`, and `cause` as `String`. The test does not call `source()`
   and does not downcast.
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

### Requirement Statement

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

- In the daemon, the component that calls both functions is `run()` of
  `sc-runtime`. The generated `daemon/src/main.rs` MUST NOT need to call
  either.
- `run()` MUST obtain the instance root only by calling the `sc-transport`
  instance-root function, passing the `app` name given to `Daemon::builder`
  and the explicit instance root if one was supplied.
- `run()` MUST obtain the bind endpoint only by calling the `sc-transport`
  endpoint resolver, passing the same instance root it locks.
- `sc-runtime` MUST NOT contain endpoint logic of its own: it MUST NOT compute
  a socket path, choose a default address or port, parse an endpoint string,
  or apply a precedence between sources.
- The `--endpoint` flag value, the configured endpoint and the explicit
  instance root MUST reach `run()` inside the `DaemonConfig` passed to
  `Daemon::builder`. The builder MUST NOT gain a method for them.
  `endpoint_flag` carries the flag. `transport` embeds `TransportConfig`,
  whose `endpoint` and `instance_root` carry the other two.
- `sc-runtime` MUST NOT parse the command line. The project's `main` parses
  `--endpoint` and places the value in `DaemonConfig` before calling
  `Daemon::builder` (decided in this document; it follows from the builder
  taking only `app` and `&DaemonConfig`).
- The daemon MUST honour `SC_ENDPOINT`, because a CLI started with the same
  variable resolves to that endpoint and the two would otherwise never meet.
  Honouring `SC_ENDPOINT` through the resolver is not "loading configuration"
  in the sense of [REQ-RT-0001](requirements.md) step 1 and
  [NFR-RT-0002](requirements.md).
- The endpoint and instance-root portion of configuration MUST be a
  `serde::Deserialize` type exported by `sc-transport` with default features.
  `DaemonConfig` embeds it, and a project's CLI, which cannot name
  `DaemonConfig` because it does not depend on `sc-runtime`, uses it
  directly. The field split is recorded in [REQ-RT-0001](requirements.md).
  The `TransportConfig` field list is the one [REQ-TRN-0001](../sc-transport/requirements.md)
  froze.

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

### Rationale

The lock lives in the instance root and, by default, so does the socket. If
`sc-runtime` computed either on its own, the lock directory and the socket
directory could diverge, and a daemon could listen where no CLI looks. One
resolver in one crate, called by both binaries with the same inputs, is what
makes the CLI find the daemon. The builder is limited to five methods, so the
inputs need a named carrier, and `DaemonConfig` is the only value the project
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

## NFR-RT-0001: `sc-runtime` is assembly only, no registry, macro or wrapper

**Status:** Active  

### Requirement Statement

`sc-runtime` is a bootstrap, not a framework layer.

- The crate MUST NOT contain a command or operation registry (any structure
  in which a project registers operations for the framework to dispatch).
- The crate MUST NOT define or export a procedural or declarative macro.
- The crate MUST NOT define a public type that wraps an `axum`, `rmcp` or
  `sqlx` type. The project passes in an ordinary `utoipa_axum::OpenApiRouter`
  and an ordinary rmcp service, and they are mounted as given.
- The public surface MUST be limited to the following kinds of item. The
  names are pinned. `Daemon::builder` takes `&sc_transport::AppName`. This
  crate does not define or re-export `sc_runtime::AppName`. The `testing`
  module and `DaemonFixture` sit behind the non-default `test-util` feature.
  `#[doc(hidden)]` typestate markers `NeedStores`, `NeedRoutes`, and `Ready`
  appear in the builder signature and are not facade items.

| Kind of public item | How many | Name |
|---|---|---|
| Entry type whose only public function is the constructor that returns the builder | one | `Daemon` (`Daemon::builder`) |
| Builder type with exactly five public methods: three that each take one project closure (stores, routes, optional MCP service), one optional all-phase logger setter, and one that runs the daemon | one | `DaemonBuilder` (`stores`, `routes`, `mcp`, `logger`, `run`) |
| Configuration struct implementing `serde::Deserialize` | one | `DaemonConfig` |
| Error enum | one | `RuntimeError` |
| Public module for tests, containing one fixture type, behind `test-util` | one | `testing` (`testing::DaemonFixture`) |

- The crate MUST NOT have any other public item: no public trait, no public
  free function, no public macro, and no `pub use` of an item from another
  crate.

**OPEN:** whether a generated project's `daemon` crate takes `sc-command` as
a direct dependency or through a re-export from `sc-runtime` is not decided
(owner: [REQ-RUN-0301](../requirements.md)); the `service` crate takes it
directly, because it may not depend on `sc-runtime`. If the re-export
is chosen, it MUST be added to the table above as a further kind of item;
until then no re-export is allowed.

### Rationale

The repo-level decision [ADR-RUN-0001](../architecture.md) is that the
framework instantiates parts and the project wires them: operations are
ordinary async service functions, and REST, MCP and CLI are produced by the
standard derives of axum, rmcp and clap. A registry, macro or wrapper type in
this crate would put the framework back between the project and those
libraries, and would be code every project has to learn.

### Success Criteria

1. `crates/sc-runtime/Cargo.toml` has no `proc-macro = true` and the crate
   source contains no `macro_rules!` exported with `#[macro_export]`.
2. The `[public] facade` list of `boundaries/sc-runtime/sc-runtime.toml` is
   `Daemon, DaemonBuilder, DaemonConfig, RuntimeError, testing`. It does not
   list `AppName`. It contains exactly one entry type, one builder type, one
   configuration struct, one error enum and one test module, and nothing else.
   `just lint` (which runs `sc-lint lint sc-boundary`) is clean. A default
   build does not expose `testing`.
3. Inspection of the crate's rustdoc index (`cargo doc -p sc-runtime
   --no-deps`, and again with `--features test-util`): the builder type has
   exactly five public methods (`stores`, `routes`, `mcp`, `logger`, `run`), the entry
   type has exactly one public function (`builder`), and the index does not
   list `sc_runtime::AppName`. `testing` appears only when `test-util` is on.
4. The `arch-qa` architecture review confirms no public type has a field or
   newtype payload of an `axum`, `rmcp` or `sqlx` type, and no registry
   structure exists.

---

## NFR-RT-0002: No sqlx, no `sc-observability`, no config loading

**Status:** Active  

### Requirement Statement

- `sc-runtime` MUST NOT depend on `sqlx` or any `sqlx-*` crate, directly or
  transitively.
- `sc-runtime` MUST NOT depend on the crate `sc-observability` or on its OTel
  export crate `sc-observability-otlp`, directly or transitively. It MUST NOT
  initialise logging, and MUST NOT re-export either crate.
- `sc-runtime` MUST NOT load configuration: it MUST NOT read a config file or
  call `sc_config::load`. It receives a `&DaemonConfig` that the project has
  already loaded.
- Honouring the `SC_ENDPOINT` endpoint override by way of the `sc-transport`
  endpoint resolver is not loading configuration and is required
  ([REQ-RT-0008](requirements.md)). No other environment variable may be
  read.

The only observability-related crate permitted in its dependency tree is
`sc-observability-types`. It is a direct dependency used only for the
injected canonical logger contract; the full runtime and OTel exporter remain
forbidden. The direct-depth cargo tree must contain the types crate and no
full-runtime line.

`sqlx`, `sc-observability`, and `sc-observability-otlp` are not workspace
members. sc-lint 0.8.0 rejects a forbidden edge whose target is absent, so
those names are not `forbidden_edges` entries. `cargo tree` enforces them.
The repo-level owner of the forbidden edges of all workspace library crates is
[REQ-RUN-0005](../requirements.md).

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

### Requirement Statement

The workspace library set includes `sc-config`, `sc-transport`, `sc-command`,
`sc-runtime`, `sc-runtime-db-sqlite`, and `sc-cron`. This item owns only the list of third-party
dependencies `sc-runtime` is allowed. The workspace-internal edges and the
forbidden edges of all workspace library crates are owned by
[REQ-RUN-0005](../requirements.md): `sc-runtime` is the only one of the four
that may depend on the others, and `sc-config`, `sc-transport` and
`sc-command` MUST NOT depend on `sc-runtime` or on each other.

The normal (non-dev, non-optional) dependencies of `sc-runtime` MUST be exactly
the following.

| Dependency | Why it is needed |
|---|---|
| `sc-transport` with `features = ["server"]` | listener binding, the endpoint resolver and the instance-root function |
| `sc-command` with `features = ["server"]` | the axum response and rmcp tool-result conversions of the response envelope, including `GET /health` |
| `sc-observability-types` | `SharedCanonicalLog` and canonical request lifecycle events; no sink construction or shutdown |
| `axum` | the merged `axum::Router` and serving |
| `tokio` | the async runtime and signal handling |
| `fd-lock` | the exclusive lock on `daemon.lock` |
| `utoipa-axum` and `utoipa` | the routes closure returns `utoipa_axum::OpenApiRouter`, and the OpenAPI document is a `utoipa` type |
| `serde` | `DaemonConfig` MUST implement `Deserialize`, because the project loads it as `cfg.daemon` through `sc-config` |
| `rmcp` | `.mcp(...)` accepts the concrete `StreamableHttpService` and mounts it as given. This crate does not wrap that type and does not re-export it |
| `tracing` | the request-id span on the assembled router, including `/health`, `/openapi.json`, `/metrics`, and optional `/mcp` |

- The edge `sc-runtime -> sc-config` is absent. `sc-runtime` MUST NOT depend
  on `sc-config` and MUST NOT load configuration
  ([NFR-RT-0002](requirements.md)).
- `rmcp` is a direct dependency. `.mcp(...)` accepts the concrete rmcp
  service type and mounts it as given. `sc-runtime` does not wrap that type
  and does not re-export it ([ADR-RUN-0001](../architecture.md) point 4).
- `testing::DaemonFixture` sits behind the non-default cargo feature
  `test-util`. A production daemon build does not enable it and does not link
  the fixture. The temporary-directory crate is `tempfile`, an optional
  dependency of `test-util` only. It is not a normal dependency.
- `sc-runtime` MUST NOT depend on `sqlx`, `sc-observability` or
  `sc-observability-otlp` ([NFR-RT-0002](requirements.md)).
- `sc-runtime` is a dependency of a project's daemon binary and tests. That a
  project's CLI does not depend on it, and the `cargo tree -p cli` check for
  that, are owned by [NFR-RUN-0001](../requirements.md). Sprint a-6 runs that
  command.

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

### Requirement Statement

The repo-level rule is owned by [NFR-RUN-0009](../requirements.md): in each
of the four library crates, code reachable from a public function MUST NOT
use `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`
or panicking `[]` indexing, and every fallible public operation returns
`Result<T, E>` with `E` the crate's own typed error enum. There is no
exception for a use that carries an explanatory comment. Test code is exempt.

Applied to `sc-runtime`:

- No public function or method of `sc-runtime` may panic on caller input or
  on the state of the environment (lock held, missing directory, port in use,
  stores failure, signal delivery).
- Every fallible public operation MUST return `Result<_, RuntimeError>`
  ([REQ-RT-0007](requirements.md)).
- Non-test code in `crates/sc-runtime/src` that is reachable from a public
  function MUST NOT use `unwrap`, `expect`, `panic!`, `unreachable!`,
  `todo!`, `unimplemented!` or panicking `[]` indexing.
- A panic raised inside a project closure is the project's; `sc-runtime` is
  not required to catch it.

`sc-runtime` adds no strictness beyond the repo-level rule.

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
