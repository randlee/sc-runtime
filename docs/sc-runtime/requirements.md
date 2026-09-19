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
[`../sc-runtime-design.md`](../sc-runtime-design.md). Entries follow the
shared SC requirement and ADR templates: the obligation, then **Why** and
**Verified by**. Every id is binding and is never reused.

## Purpose

`sc-runtime` is the assembled daemon: a bootstrap that builds the parts and
hands them to the project. It is the single assembly point, reduced to
assembly only. It is a dependency of the daemon binary and of tests, never
of the CLI.

## ID Ranges

| Range | Area | Scope |
|---|---|---|
| REQ-RT-0001 through REQ-RT-0007 | Requirements | - |
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

`run()` MUST perform exactly these five steps, in this order:

1. Take the `DaemonConfig` passed to `builder`. `run()` MUST NOT read a config
   file or an environment variable to load configuration.
2. Resolve the instance root (the per-user directory for `app`, or the one
   supplied explicitly) and take an OS exclusive lock on
   `<instance-root>/daemon.lock`.
3. Call the stores closure exactly once and await its result.
4. Build one `axum::Router` by merging: the router from the routes closure,
   a route serving the OpenAPI document produced by that `OpenApiRouter`, a
   health route, and, only if `.mcp(...)` was called, the rmcp service at
   `/mcp`.
5. Bind the listener through `sc-transport` (built with its `server` feature)
   and serve the merged router until SIGINT or SIGTERM.

`run()` MUST NOT call the stores closure before the lock is held.
`run()` MUST NOT bind a listener before steps 1 to 4 have succeeded.
If a step fails, `run()` MUST return an `Err(RuntimeError)` and MUST NOT
execute any later step.
The builder MUST NOT offer hooks, callbacks or configuration that insert work
between steps or reorder them.

**OPEN:** the exact Rust signatures of `builder`, `stores`, `routes`, `mcp` and
`run` (how `Stores` is passed to the two closures, the error type the stores
closure returns, the router state type, the concrete rmcp service type) are
not stated by the design and are pinned by the contract sprint.

**OPEN:** the fields of `DaemonConfig` are not stated by the design. The design
states only that it is the `daemon` section of the project's config
(`&cfg.daemon`) and is loaded by the project before `builder` is called.

### Rationale

Before this crate, every project hand-assembled lock, stores, router and
listener, each slightly differently. One builder makes start-up identical in
every project. The order is the safety property: the lock comes before the
stores so a second daemon never opens the local database, and binding comes
last so no request is accepted before it can be served.

### Success Criteria

1. A test in `crates/sc-runtime` builds a daemon whose stores, routes and MCP
   closures each append their name to a shared list, runs it on a tempdir
   instance root, and asserts the list is exactly `stores`, `routes`, `mcp`
   and that `<instance-root>/daemon.lock` existed and was locked when the
   stores closure ran.
2. A test holds the lock on the instance root, calls `run()`, and asserts it
   returns `Err`, and that the stores, routes and MCP closures were never
   called and no listener was bound.
3. A test whose stores closure returns an error asserts `run()` returns `Err`,
   and that the routes and MCP closures were never called and no listener was
   bound (connecting to the endpoint fails).
4. A test that makes the bind fail (the TCP port is already in use) asserts
   `run()` returns `Err` after the stores closure ran, and that no request was
   served.
5. Inspection of the public API of `crates/sc-runtime`: `DaemonBuilder` has no
   method other than the four in the table that alters start-up.

---

## REQ-RT-0002: Hard singleton through an OS lock on `daemon.lock`

**Status:** Active  

### Requirement Statement

`sc-runtime` MUST enforce that at most one daemon runs per instance root. The
instance root is the per-user directory for the application name, or a
directory supplied explicitly (tests supply a tempdir).

- The mechanism MUST be an OS exclusive file lock, taken with the `fd-lock`
  crate, on the file `<instance-root>/daemon.lock`.
- The lock MUST be taken by `run()` before the stores closure is called and
  before any listener is bound.
- The lock MUST be held until the process exits or `run()` returns.
- The attempt to lock MUST NOT block waiting for the holder. If another
  process holds the lock, `run()` MUST return a `RuntimeError` whose variant
  means "lock held" (illustrative name `RuntimeError::LockHeld`;
  [REQ-RT-0007](requirements.md) defines the variants).
- In that case the stores closure MUST NOT be called and a listener MUST NOT
  be bound.
- The crate MUST NOT use a PID file or probe a port to detect another daemon.

**OPEN:** whether `run()` creates the instance root directory when it does not
exist, or returns an error, is not stated by the design.

**OPEN:** which `RuntimeError` variant reports an I/O failure opening
`daemon.lock` that is not lock contention (for example permission denied) is
not stated.

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
- `sc-runtime` MUST NOT depend on `sqlx`.

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
3. `cargo tree -p sc-runtime` does not list `sqlx` or any `sqlx-*` crate.
4. The boundary manifest under `boundaries/sc-runtime/` lists `sqlx` as a
   forbidden edge, and `just lint` (which runs `sc-lint-boundary`) is clean.

---

## REQ-RT-0004: Routes mounted on the daemon's single router

**Status:** Active  

### Requirement Statement

In step 4 of `run()`, `sc-runtime` MUST build one `axum::Router`, served on
one listener, containing the following.

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
- If `.mcp(...)` was not called, `sc-runtime` MUST NOT mount anything at
  `/mcp`; with no project route or fallback there, axum answers HTTP 404.
- MCP MUST NOT be served on a separate listener, port or process.

**OPEN:** the path of the OpenAPI document route is not stated by the design
(it says only that the daemon serves `openapi.json`).

**OPEN:** the path, HTTP method, status code and response body of the health
route are not stated by the design.

**OPEN:** the behaviour when the project's router already defines the health
path, the OpenAPI document path or `/mcp` is not stated.

### Rationale

One route registration yields both the router and the OpenAPI document, so
the document cannot drift from the routes. MCP shares the binary and the
port, so a project deploys and secures one listener. The repo-level
requirement that one listener serves everything is
[REQ-RUN-0205](../requirements.md); the library choice (utoipa-axum plus rmcp
on one axum router) is [ADR-RUN-0202](../architecture.md), whose version
compatibility is still to be verified by the spike sprint.

### Success Criteria

1. A `DaemonFixture` test registers one project route, fetches the OpenAPI
   document route, parses the body as JSON, and asserts the `paths` object
   contains that route's path.
2. A `DaemonFixture` test requests the health route and asserts the pinned
   status and body.
3. A `DaemonFixture` test built with an MCP closure sends an MCP `initialize`
   request to `/mcp` and asserts a successful MCP response.
4. A `DaemonFixture` test built without an MCP closure requests `/mcp` and
   asserts HTTP 404.
5. One test performs criteria 1 to 3 plus a call to the project route through
   a single client connected to a single endpoint.

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
3. Close the listener.
4. Before returning, remove the socket file if the endpoint is a Unix domain
   socket (`<instance-root>/daemon.sock` by default) (decided in this
   document).
5. Before returning, release the lock on `<instance-root>/daemon.lock`.
6. Return `Ok(())` (decided in this document, following from the
   `Result<(), RuntimeError>` return type in [REQ-RT-0007](requirements.md)).

Steps 1 to 3 happen in that order. The relative order of steps 4 and 5 is not
stated by the design.

`run()` MUST NOT delete `daemon.lock`; releasing the lock is sufficient.

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

1. A Unix test starts a daemon in a child process with a route that sleeps
   before replying, sends a request, sends SIGTERM to the child while the
   request is in flight, and asserts the client receives the complete
   successful response.
2. The same test asserts the child then exits, that a new connection attempt
   after the signal fails, and that the socket file no longer exists.
3. The same scenario with SIGINT passes the same assertions.
4. After the child exits, a new daemon on the same instance root starts
   successfully, proving the lock was released.
5. A test asserts `run()` returns `Ok(())` after a signal-initiated shutdown
   (the child's exit code is the success code).

---

## REQ-RT-0006: Test fixture `DaemonFixture` in the `testing` module

**Status:** Active  

### Requirement Statement

`sc-runtime` MUST expose `sc_runtime::testing::DaemonFixture` for use in the
tests of generated projects and of this workspace. The name is illustrative
until the contract sprint pins it.

- Starting a fixture MUST create a new temporary directory and use it as the
  daemon's instance root.
- The fixture MUST start the daemon through the same `Daemon::builder()` path
  as production: real `daemon.lock`, the project's stores, routes and optional
  MCP closures, a real listener bound through `sc-transport`.
- The fixture MUST expose an `sc_transport::Client` already connected to that
  daemon's endpoint (decided in this document).
- Dropping the fixture MUST stop the daemon (decided in this document).
- Any number of fixtures MUST be able to run at the same time inside one test
  binary under the default parallel test runner, with no shared socket, port,
  lock file or database file.
- The fixture MUST NOT use a fixed path such as `/tmp/...` or a path under the
  user's home directory.

**OPEN:** the fixture's constructor signature (how a test supplies the stores,
routes and MCP closures and any `DaemonConfig` overrides) is not stated by the
design.

**OPEN:** the endpoint a fixture uses on Windows, where the default transport
is TCP on `127.0.0.1` and a tempdir alone does not make the endpoint unique,
is not stated.

**OPEN:** how many concurrent fixtures the parallelism test must run is not
stated.

### Rationale

Every generated project tests its operations against a real daemon. Without
an isolated fixture those tests would share one socket and one database and
could not run in parallel, and could collide with a developer's real daemon;
the repo-level rule that every test uses its own tempdir instance root and
runs in parallel is [NFR-RUN-0005](../requirements.md). The template's worked
example (`widget.create` and `widget.get`) ships with a test built on this
fixture ([REQ-RUN-0302](../requirements.md)).

### Success Criteria

1. A test starts a fixture, calls a project route through the fixture's
   client, and asserts the expected response.
2. A test starts several fixtures concurrently in one test binary, each writes
   a distinct value through its own daemon and reads it back, and each asserts
   it sees only its own value.
3. A test asserts two live fixtures have different instance roots, and a
   search of `crates/sc-runtime/src` finds no literal `/tmp` or home path.
4. A test drops a fixture and asserts that connecting to its former endpoint
   fails.
5. `cargo test -p sc-runtime` passes under the default parallel runner, and
   passes when two such runs execute at the same time.

---

## REQ-RT-0007: `run()` returns the typed error `RuntimeError`

**Status:** Active  

### Requirement Statement

`run()` MUST return `Result<(), RuntimeError>`. `RuntimeError` is a public
enum in `sc-runtime` with one variant per failing step. The variant set is
decided in this document; the variant names are illustrative until the
contract sprint pins them.

| Variant (illustrative name) | Returned when |
|---|---|
| `LockHeld` | another process holds the lock on `<instance-root>/daemon.lock` |
| `StoresFailed` | the project's stores closure returned an error |
| `BindFailed` | `sc-transport` could not bind the listener |
| `ServeFailed` | serving the router ended with an error |

- Each variant MUST carry the underlying error so the caller can print it.
- `RuntimeError` MUST implement `std::error::Error`, `Debug` and `Display`.
- A caller MUST be able to distinguish the variants with a `match`; in
  particular "daemon already running" (`LockHeld`) MUST be distinguishable
  from "cannot bind" (`BindFailed`).
- `run()` MUST NOT panic and MUST NOT call `std::process::exit`; mapping the
  result to an exit code is done by the project's `main`.

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
5. A compile-time assertion in the tests checks
   `RuntimeError: std::error::Error + Send + Sync + 'static`.

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
- The public surface MUST be limited to: `Daemon`, `DaemonBuilder` (with
  `stores`, `routes`, `mcp`, `run`), `DaemonConfig`, `RuntimeError` and
  `testing::DaemonFixture`. These names are illustrative until the contract
  sprint pins them; the limit on what kinds of item exist is binding.

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
2. The public facade recorded in the boundary manifest under
   `boundaries/sc-runtime/` lists only the items named above, and `just lint`
   (which runs `sc-lint-boundary`) is clean.
3. The `arch-qa` architecture review confirms no public type has a field or
   newtype payload of an `axum`, `rmcp` or `sqlx` type, and no registry
   structure exists.

---

## NFR-RT-0002: No sqlx, no `sc-observability`, no config loading

**Status:** Active  

### Requirement Statement

- `sc-runtime` MUST NOT depend on `sqlx`, directly or transitively.
- `sc-runtime` MUST NOT depend on `sc-observability` or its OTel export crate,
  MUST NOT initialise logging, and MUST NOT re-export either crate.
- `sc-runtime` MUST NOT load configuration: it MUST NOT read a config file or
  call `sc_config::load`. It receives a `&DaemonConfig` that the project has
  already loaded.

The only observability-related crate permitted in its dependency tree is
`sc-observability-types`, which arrives through `sc-command`.

### Rationale

Stores are project-owned code, one crate per database backend, so sqlx
belongs there ([ADR-RUN-0301](../architecture.md)). Observability is
initialised directly by the project's `main.rs` with plain values, so that
neither this crate nor `sc-config` becomes unusable without it
([ADR-RUN-0004](../architecture.md)). Keeping config loading in `main.rs`
lets the project report a config error before anything else starts.

### Success Criteria

1. `cargo tree -p sc-runtime` lists no `sqlx`, `sqlx-*`, `sc-observability`
   or OTel export crate (`sc-observability-types` is allowed).
2. The boundary manifest under `boundaries/sc-runtime/` lists `sqlx` and
   `sc-observability` under `forbidden_edges`, and `just lint` is clean.
3. A search of `crates/sc-runtime/src` finds no call to `sc_config::load` and
   no file or environment read used to obtain configuration.

---

## NFR-RT-0003: `sc-runtime` is the only crate that depends on the other three

**Status:** Active  

### Requirement Statement

The workspace has four library crates: `sc-config`, `sc-transport`,
`sc-command` and `sc-runtime`.

- `sc-runtime` MUST depend on `sc-config`, `sc-transport` and `sc-command`.
- `sc-runtime` MUST enable the `server` cargo feature of `sc-transport`
  (listener binding) and of `sc-command` (the axum response and rmcp
  tool-result conversions). `sc-config` has no `server` feature.
- `sc-config`, `sc-transport` and `sc-command` MUST NOT depend on
  `sc-runtime` or on each other.
- `sc-runtime` MUST be a dependency of a project's daemon binary and tests
  only. A project's CLI MUST NOT depend on it.

**OPEN:** the design lists this crate's dependencies as the three crates
above plus `axum`, `tokio` and `fd-lock`. Naming `utoipa_axum::OpenApiRouter`
in the builder needs `utoipa-axum` as well, and whether `rmcp` is a direct
dependency or the MCP service is accepted as a generic Tower service is not
stated. The full allowed-dependency list is pinned by the contract sprint.

### Rationale

The CLI must not link Axum, rmcp or sqlx, yet daemon and CLI both use
`sc-transport` and `sc-command`. The repo-level decision
[ADR-RUN-0003](../architecture.md) resolves this by putting server-side code
behind a `server` feature that is off by default, and making `sc-runtime`
the one place that turns it on. If any other crate depended on the three, or
the three on each other, none could be used alone.

### Success Criteria

1. `crates/sc-runtime/Cargo.toml` declares `sc-config`, `sc-transport` with
   `features = ["server"]` and `sc-command` with `features = ["server"]`.
2. The `Cargo.toml` of `sc-config`, `sc-transport` and `sc-command` name none
   of the four workspace crates as a dependency.
3. The boundary manifests under `boundaries/<crate>/` record these edges and
   `just lint` (which runs `sc-lint-boundary`) is clean.
4. In a generated project, `cargo tree -p cli` does not list `sc-runtime`.

---

## NFR-RT-0004: No public function in `sc-runtime` panics

**Status:** Active  

### Requirement Statement

- No public function or method of `sc-runtime` MUST panic on caller input or
  on the state of the environment (lock held, missing directory, port in use,
  stores failure, signal delivery).
- Every fallible public operation MUST return `Result<_, RuntimeError>`.
- Non-test code in `crates/sc-runtime/src` MUST NOT call `unwrap()`,
  `expect()`, `panic!`, `unreachable!` or `todo!` on a path reachable from a
  public function with a value that depends on input or environment.
- A panic raised inside a project closure is the project's; `sc-runtime` is
  not required to catch it.

### Rationale

These crates run inside long-lived daemons and under CLIs driven by agents; a
panic there is an outage or a failure no caller can parse. The repo-level
decision [ADR-RUN-0006](../architecture.md) is that every library API returns
`Result<T, E>` with one typed error enum per crate.

### Success Criteria

1. A search of `crates/sc-runtime/src` (excluding `#[cfg(test)]` code) for
   `unwrap(`, `expect(`, `panic!`, `unreachable!` and `todo!` finds no use on
   a path that depends on caller input or environment state; each remaining
   use has a comment stating why it cannot fail.
2. The tests of [REQ-RT-0007](requirements.md) (one per `RuntimeError`
   variant: lock held, stores failed, bind failed, serve failed) pass, showing
   each failure is returned as a value.
