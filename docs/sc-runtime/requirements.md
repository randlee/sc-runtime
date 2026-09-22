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
assembles and runs a daemon. The names below are illustrative until the
contract sprint pins them; the shape and the order are binding.

| Call | Argument | Required |
|---|---|---|
| `Daemon::builder(app, &daemon_config)` | `app`: the application name as `&str` (for example `"my-app"`); `daemon_config`: a `&DaemonConfig` the project has already loaded | yes |
| `.stores(closure)` | an async closure taking no arguments that opens the project's stores and returns the project's own `Stores` value or an error | yes |
| `.routes(closure)` | a closure that receives the `Stores` value and returns a `utoipa_axum::OpenApiRouter` | yes |
| `.mcp(closure)` | a closure that receives the `Stores` value and returns an rmcp Streamable HTTP service | no |
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

    let result = sc_runtime::Daemon::builder("my-app", &cfg.daemon)
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
   before it inspects or replaces the socket. It is one `axum::Router`,
   served unchanged on every listener `run()` binds.

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](../sc-transport/requirements.md)). How `run()`
obtains the instance root and the bind endpoint is
[REQ-RT-0008](requirements.md).

`run()` MUST NOT call the stores closure before the lock is held.
`run()` MUST NOT bind a listener before steps 1 to 4 have succeeded.
If a step fails, `run()` MUST return an `Err(RuntimeError)` and MUST NOT
execute any later step.
The builder MUST NOT offer hooks, callbacks or configuration that insert work
between steps or reorder them (decided in this document and recorded in
[ADR-RT-0001](architecture.md); the design lists the five steps and says the
crate is a bootstrap, not a framework layer).

**OPEN:** whether `Daemon::builder()` supports two listeners in v0.1. A
daemon MAY bind a Unix domain socket listener and a TCP listener at once, and
`sc-transport` provides that capability
([REQ-TRN-0003](../sc-transport/requirements.md)). Whether `DaemonConfig` can
request both, so that step 5 binds two listeners, or v0.1 of `sc-runtime`
binds exactly one, is not decided. Once it is decided that two are supported:
a test MUST show one daemon answering the same route on its UDS listener and
on its TCP listener.

**OPEN:** the exact Rust signatures of `builder`, `stores`, `routes`, `mcp` and
`run` (how `Stores` is passed to the two closures, the error type the stores
closure returns, the router state type, the concrete rmcp service type) are
not stated by the design and are pinned by the contract sprint.

**OPEN:** the fields of `DaemonConfig` are not stated by the design. The design
states only that it is the `daemon` section of the project's config
(`&cfg.daemon`) and is loaded by the project before `builder` is called. Two
constraints bind whatever is chosen. First, `DaemonConfig` MUST carry the
`--endpoint` flag value, the configured endpoint and an explicit instance
root ([REQ-RT-0008](requirements.md)). Second, a project's CLI cannot name
`DaemonConfig`, because the CLI does not depend on `sc-runtime`; so the
endpoint and instance-root portion of configuration MUST be a
`serde::Deserialize` type exported by `sc-transport` with default features,
which `DaemonConfig` embeds and the CLI's config uses directly. The shape of
that type is the configuration-value OPEN of
[REQ-TRN-0001](../sc-transport/requirements.md).

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
`run()` return `Err(RuntimeError)` without calling the stores closure. Which
variant reports it is an OPEN of [REQ-RT-0007](requirements.md), which also
holds the test for it.

**OPEN:** whether `run()` creates the instance root directory when it does not
exist, or returns an error, is not stated by the design. The same question is
open for the `sc-transport` instance-root function
([REQ-TRN-0002](../sc-transport/requirements.md)).

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

Whether `run()` binds one listener or two in v0.1 is an OPEN of
[REQ-RT-0001](requirements.md).

**OPEN:** the path of the OpenAPI document route is not stated by the design
(it says only that the daemon serves `openapi.json`).

**OPEN:** the path, HTTP method, status code and response body of the health
route are not stated by the design.

**OPEN:** the behaviour when the project's router already defines the health
path, the OpenAPI document path or `/mcp` is not stated.

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

1. Once the path of the OpenAPI document route is decided (first OPEN above):
   a `DaemonFixture` test registers one project route, fetches the document
   from that path, parses the body as JSON, and asserts the `paths` object
   contains that route's path.
2. Once the path, method, status code and body of the health route are decided
   (second OPEN above): a `DaemonFixture` test sends that method to that path
   and asserts that status code and that body. Until then, inspection of
   `crates/sc-runtime/src` confirms that `run()` merges a health route
   supplied by `sc-runtime` itself into the router.
3. A `DaemonFixture` test built with an MCP closure sends an MCP `initialize`
   request over HTTP to `/mcp` on the endpoint the fixture exposes
   ([REQ-RT-0006](requirements.md)) and asserts a successful MCP response.
4. A `DaemonFixture` test built without an MCP closure requests `/mcp` and
   asserts HTTP 404.
5. One test calls a project route and performs criterion 3 (and criteria 1
   and 2 once their OPENs are decided) against the single endpoint of one
   fixture, showing that all of them are served by one router on one
   listener.
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
  socket, port, lock file or database file. On Windows the same property
  depends on the OPEN below.
- The fixture MUST NOT use a fixed path such as `/tmp/...` or a path under the
  user's home directory.

The crate that supplies the temporary directory, and whether the `testing`
module sits behind a cargo feature, are OPENs of
[NFR-RT-0003](requirements.md).

**OPEN:** the fixture's constructor signature (how a test supplies the stores,
routes and MCP closures and any `DaemonConfig` overrides) and the names of the
endpoint and instance-root accessors are not stated by the design.

**OPEN:** the endpoint a fixture uses on Windows, where the default transport
is TCP on `127.0.0.1` and a tempdir alone does not make the endpoint unique,
is not stated.

**OPEN:** how many concurrent fixtures the parallelism test must run is not
stated.

**OPEN:** the string syntax of an endpoint value is not decided
([REQ-TRN-0001](../sc-transport/requirements.md)); the fixture's string form
MUST be whatever that syntax turns out to be, produced by `sc-transport` and
not formatted by `sc-runtime`.

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
enum in `sc-runtime`. The four variants below are the minimum; the variant
names are illustrative until the contract sprint pins them.

| Variant (illustrative name) | Returned when |
|---|---|
| `LockHeld` | another process holds the lock on `<instance-root>/daemon.lock` |
| `StoresFailed` | the project's stores closure returned an error |
| `BindFailed` | `sc-transport` could not bind the listener |
| `ServeFailed` | serving the router ended with an error |

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
- Every failure of `run()`, including the two cases in the OPEN below, MUST be
  returned as an `Err(RuntimeError)`. A failure in resolving the instance root
  or in opening `daemon.lock` MUST NOT be reported as `LockHeld`, because the
  caller would tell the user a daemon is already running.

**OPEN:** which variant reports each of the following, and whether new
variants are added for them, is not decided: (a) a failure to resolve the
instance root (the `sc-transport` instance-root function returns a
`TransportError`), a missing or uncreatable instance root, or an I/O error
opening `daemon.lock` that is not lock contention (carried error:
`std::io::Error` or `TransportError`); (b) a failure of endpoint resolution,
for example an `--endpoint` or `SC_ENDPOINT` value that cannot be parsed
(carried error: `TransportError`). Options: two new variants; one new variant
for both; or (b) folded into `BindFailed`.

**OPEN:** how the project's stores error is carried inside `StoresFailed`
(a generic parameter or a boxed `dyn Error`) is not stated.

### Rationale

The generated `main.rs` turns the result into an exit code and a message for
a person or an agent. "A daemon is already running" calls for a different
action from "the port is taken" or "the database would not open", so the
caller needs a variant to branch on and not a string to parse.

### Success Criteria

1. A test holds the lock, calls `run()`, and matches the result against the
   `LockHeld` variant.
2. A test whose stores closure returns an error matches the result against
   `StoresFailed` and asserts the underlying error's message is reachable
   through `source()` or the variant's payload.
3. A test that makes the bind fail (TCP port already in use) matches the
   result against `BindFailed`.
4. A test covers `ServeFailed`, or, if no deterministic trigger exists, an
   inspection confirms the error from `axum::serve` is mapped to `ServeFailed`
   and to no other variant.
5. A test passes an explicit instance root that is a regular file, not a
   directory, calls `run()`, and asserts it returns `Err`, that the error is
   not `LockHeld`, and that the stores closure was never called. Once OPEN
   (a) is decided, the test also matches the decided variant.
6. A test supplies an endpoint override that cannot be parsed as an endpoint,
   calls `run()`, and asserts it returns `Err` and that no listener was
   bound. Once OPEN (b) is decided, the test also matches the decided
   variant.
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
  `Daemon::builder`. The builder MUST NOT gain a method for them. The field
  names are OPEN.
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
  directly. The same constraint is recorded in the OPENs of
  [REQ-RT-0001](requirements.md) and
  [REQ-TRN-0001](../sc-transport/requirements.md).

Consequence, not a separate obligation: with an explicit instance root `R`
and no endpoint override, on macOS and Linux the lock file is
`R/daemon.lock` and the socket is `R/daemon.sock`, because both come from the
one instance root.

This item is decided in this document. The design states only that the
daemon binds "through sc-transport" and that `sc-transport` owns endpoint
resolution (instance root, `--endpoint`, `SC_ENDPOINT`); it does not name the
caller or the carrier.

**OPEN:** the names and types of the `DaemonConfig` fields that carry the
`--endpoint` value, the configured endpoint and the explicit instance root,
and the name and shape of the `sc-transport` type that `DaemonConfig` embeds.

**OPEN:** how the `SC_ENDPOINT` value reaches the resolver in the daemon. The
resolver accepts the value as an explicit input, and `sc-transport` may also
offer a convenience form that reads `std::env::var_os("SC_ENDPOINT")` itself
([REQ-TRN-0001](../sc-transport/requirements.md)). Options: `run()` calls the
convenience form, so the name `SC_ENDPOINT` never appears in `sc-runtime`; or
`run()` reads the variable and passes it as the explicit input. In-process
parallel tests cannot rely on the process environment, so
`testing::DaemonFixture` needs the explicit form either way.

### Rationale

The lock lives in the instance root and, by default, so does the socket. If
`sc-runtime` computed either on its own, the lock directory and the socket
directory could diverge, and a daemon could listen where no CLI looks. One
resolver in one crate, called by both binaries with the same inputs, is what
makes the CLI find the daemon. The builder is limited to four methods, so the
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
   line outside `#[cfg(test)]` code. Once the second OPEN above is decided in
   favour of the convenience form, the same holds for `SC_ENDPOINT`.
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
  names in the right-hand column are illustrative until the contract sprint
  pins them; the limit on which kinds of item exist, and how many of each, is
  binding.

| Kind of public item | How many | Illustrative name |
|---|---|---|
| Entry type whose only public function is the constructor that returns the builder | one | `Daemon` (`Daemon::builder`) |
| Builder type with exactly four public methods: three that each take one project closure (stores, routes, optional MCP service) and one that runs the daemon | one | `DaemonBuilder` (`stores`, `routes`, `mcp`, `run`) |
| Configuration struct implementing `serde::Deserialize` | one | `DaemonConfig` |
| Error enum | one | `RuntimeError` |
| Public module for tests, containing one fixture type | one | `testing::DaemonFixture` |

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
2. The `[public] facade` list of the boundary manifest under
   `boundaries/sc-runtime/` is checked by kind, not by name: it contains
   exactly one entry type, one builder type, one configuration struct, one
   error enum and one test module holding one fixture type, and nothing else;
   in particular no trait, free function, macro or re-exported item. `just
   lint` (which runs `sc-lint-boundary`, the tool that checks the facade list
   against the crate) is clean.
3. Inspection of the crate's rustdoc index (`cargo doc -p sc-runtime
   --no-deps`): the builder type has exactly four public methods, and the
   entry type has exactly one public function.
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
`sc-observability-types`, which arrives through `sc-command`.

The boundary manifest under `boundaries/sc-runtime/` MUST list the edges
`sc-runtime -> sqlx`, `sc-runtime -> sc-observability` and
`sc-runtime -> sc-observability-otlp` under `forbidden_edges`. The
repo-level owner of the forbidden edges of all four crates is
[REQ-RUN-0005](../requirements.md); this item states the three that apply to
`sc-runtime`.

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
   `sc-runtime -> sqlx`, `sc-runtime -> sc-observability` and
   `sc-runtime -> sc-observability-otlp` under `[dependencies]
   forbidden_edges`, and `just lint` (which runs `sc-lint-boundary`) is clean.
3. A search of `crates/sc-runtime/src` (excluding `#[cfg(test)]` code) finds
   no call to `sc_config::load`, no read of a config file, and no read of an
   environment variable other than `SC_ENDPOINT`; `SC_ENDPOINT` is read, if
   at all, only to pass its value to the `sc-transport` endpoint resolver.

---

## NFR-RT-0003: Allowed dependencies of the `sc-runtime` crate

**Status:** Active  

### Requirement Statement

The workspace has four library crates: `sc-config`, `sc-transport`,
`sc-command` and `sc-runtime`. This item owns the list of third-party
dependencies `sc-runtime` is allowed. The workspace-internal edges and the
forbidden edges of all four crates are owned by
[REQ-RUN-0005](../requirements.md): `sc-runtime` is the only one of the four
that may depend on the others, and `sc-config`, `sc-transport` and
`sc-command` MUST NOT depend on `sc-runtime` or on each other.

The normal (non-dev) dependencies of `sc-runtime` MUST be exactly the
following, plus whatever the OPENs below add once decided.

| Dependency | Why it is needed |
|---|---|
| `sc-transport` with `features = ["server"]` | listener binding, the endpoint resolver and the instance-root function |
| `sc-command` with `features = ["server"]` | the axum response and rmcp tool-result conversions of the response envelope |
| `axum` | the merged `axum::Router` and serving |
| `tokio` | the async runtime and signal handling |
| `fd-lock` | the exclusive lock on `daemon.lock` |
| `utoipa-axum` and `utoipa` | the routes closure returns `utoipa_axum::OpenApiRouter`, and the OpenAPI document is a `utoipa` type (follows from the builder signature; the design's own list omits them) |
| `serde` | `DaemonConfig` MUST implement `Deserialize`, because the project loads it as `cfg.daemon` through `sc-config` (follows from that usage; the design's own list omits it) |

- `sc-runtime` MAY depend on `sc-config`. `sc-config` has no `server`
  feature.
- `sc-runtime` MUST NOT depend on `sqlx`, `sc-observability` or
  `sc-observability-otlp` ([NFR-RT-0002](requirements.md)).
- `sc-runtime` is a dependency of a project's daemon binary and tests. That a
  project's CLI does not depend on it, and the `cargo tree -p cli` check for
  that, are owned by [NFR-RUN-0001](../requirements.md).

**OPEN:** the edge `sc-runtime -> sc-config`. The design lists `sc-config`
among this crate's dependencies but names no use for it, and `sc-runtime`
MUST NOT load configuration ([NFR-RT-0002](requirements.md)), which is all
that `sc-config`'s public surface does. Options: drop the edge; or name the
item `sc-runtime` takes from `sc-config`. Until decided the edge is allowed,
not required, and an unused dependency MUST NOT be declared.

**OPEN:** whether `rmcp` is a direct dependency (the `.mcp(...)` closure
returns the concrete rmcp `StreamableHttpService` type) or the MCP service is
accepted as a generic Tower service and `rmcp` is not named.

**OPEN:** which crate supplies the temporary directory used by
`testing::DaemonFixture` ([REQ-RT-0006](requirements.md)), which is a normal
dependency because `testing` is a public, non-test module; and whether the
`testing` module sits behind a cargo feature so that a production daemon does
not link it.

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
   one of: `sc-transport`, `sc-command`, `axum`, `tokio`, `fd-lock`,
   `utoipa-axum`, `utoipa`, `serde`, or an entry admitted by a decided OPEN
   above (`sc-config`, `rmcp`, the tempdir crate).
3. If `sc-config` is declared, a search of `crates/sc-runtime/src` finds at
   least one use of an `sc_config` item, and none of them is `load` or
   `Loader`.
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
