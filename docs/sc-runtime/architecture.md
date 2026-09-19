# `sc-runtime` Architecture

**ID Range:** ADR-RT-0001 through ADR-RT-0004  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

Crate-level architecture and ADRs for `sc-runtime`. Repo-level architecture is in
[`../architecture.md`](../architecture.md); this crate's requirements are in
[`requirements.md`](requirements.md); its boundary manifest is
`boundaries/sc-runtime/`. ADRs follow
the shared SC requirement and ADR templates. Every ADR here is Active
and binding; an ADR changes only through a later ADR that names it.

## Role

The single assembly point. It is the only crate that knows the other three, and
it is a dependency of the daemon only. Everything it receives from the project
(config, stores, router, MCP service) it passes through untouched.

## Public surface

| Item | Purpose |
|---|---|
| `Daemon::builder(app, &DaemonConfig)` | start assembly |
| `DaemonBuilder::stores / routes / mcp / run` | the project's three closures, then serve |
| `DaemonConfig` | the daemon's own settings, deserialisable as part of the project's config; its fields are undecided (see REQ-RT-0001) |
| `RuntimeError` | typed error naming the failing step |
| `testing::DaemonFixture` | an isolated daemon per test |

Shape of use, from the design:

```rust
let result = sc_runtime::Daemon::builder("my-app", &cfg.daemon)
    .stores(|| async { Stores::open(&cfg.stores).await })
    .routes(|stores| routes::router(stores))   // utoipa_axum::OpenApiRouter
    .mcp(|stores| mcp::service(stores))        // optional; an rmcp service
    .run()
    .await;
```

Names are illustrative until the contract sprint pins them; the shape is the
decision.

---

## ADR-RT-0001: `DaemonBuilder` `run()` is assembly only, in a fixed order

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19, section "The crates" (sc-runtime is a bootstrap, not a framework layer); applies repo [ADR-RUN-0001](../architecture.md), [ADR-RUN-0201](../architecture.md)  

### Context

The crate `sc-runtime` (`crates/sc-runtime`) is the one place where a
generated project's daemon is put together: singleton lock, stores, router,
listener. An assembly point like this tends to grow into a framework layer
with lifecycle hooks, plugin points and a configurable start-up sequence.
Two repo-level decisions rule that out. The framework instantiates parts and
the project wires them in ordinary Rust, with no registry and no wrapper
types ([ADR-RUN-0001](../architecture.md)). The daemon is the only process
that opens the local database, guarded by an OS lock on
`<instance-root>/daemon.lock` ([ADR-RUN-0201](../architecture.md)), so
start-up order is a safety property and not a preference.

### Decision

`run()` on the builder returned by `Daemon::builder(app, &daemon_config)`
does exactly five things, always in this order:

1. take the `DaemonConfig` the project already loaded;
2. resolve the instance root and take the exclusive lock on
   `<instance-root>/daemon.lock`;
3. call the project's stores closure;
4. merge the project's `utoipa_axum::OpenApiRouter`, the OpenAPI document
   route, a health route, and the rmcp service at `/mcp` if one was given;
5. bind the listener through `sc-transport` and serve until SIGINT or SIGTERM.

The lock is taken before the stores closure is called. Binding is the last
step. A failure in any step returns a `RuntimeError` and no later step runs.
The builder has no hook, callback, plugin registry or option that inserts
work between steps or changes their order. API names are illustrative until
the contract sprint pins them; the five steps and their order are the
decision.

### Consequences

Start-up behaviour is identical in every project and can be read in one
function. A second daemon on the same instance root never opens the database,
and no request is accepted before the stores and routes exist. A project that
needs work between steps does it inside its own closure: for example, extra
start-up work goes in the stores closure, and extra routes or middleware go
in the routes closure. The crate cannot serve a project that needs a
different order; that project assembles its daemon by hand from `sc-transport`
and `sc-command`.

### Alternatives Considered

- **Lifecycle hooks** (`on_before_bind`, `on_after_stores` and similar):
  rejected because each hook is framework API a project must learn, and the
  project's own closures already give it a place to run code at each stage.
- **A plugin registry** in which components register themselves with the
  daemon: rejected because it puts a framework-owned abstraction between the
  project and axum and rmcp, which the repo-level decision forbids.
- **A configurable step order**: rejected because lock-before-stores and
  bind-last are safety properties; making them configurable makes them
  breakable.

### Implementation

**Enforced by:** the step-order and failure-stops-later-steps tests required
by [REQ-RT-0001](requirements.md); `arch-qa` architecture review of the
public surface of `DaemonBuilder` (only `stores`, `routes`, `mcp`, `run`).

### Related Documents

- [REQ-RT-0001](requirements.md)
- [NFR-RT-0001](requirements.md)

---

## ADR-RT-0002: Daemon singleton by `fd-lock` on `daemon.lock`

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19, section "Runtime architecture"  

### Context

Only one daemon may run per instance root (the per-user directory for an
application, or a tempdir in tests), because the daemon is the only process
allowed to open the local database and SQLite allows one writer. Singletons
are commonly implemented with a PID file or by probing the listening port.
A PID file goes stale when the process crashes, and PIDs are reused. Probing
a port or socket races: two daemons can both probe, both see nothing, and
both start. Both need clean-up code that does not run on a crash.

### Decision

`sc-runtime` takes an OS exclusive file lock, using the `fd-lock` crate, on
the file `<instance-root>/daemon.lock`. The lock is taken in `run()` before
the project's stores closure is called and before any listener is bound, and
is held for the life of the process. If the lock is already held, `run()`
returns the "lock held" variant of `RuntimeError` without waiting. The crate
writes no PID file and probes no port.

### Consequences

The kernel releases the lock on any exit, including a crash or `kill -9`, so
there is nothing to clean up and the file `daemon.lock` may stay on disk.
Holding the lock also proves no other daemon for this instance root is alive,
which is what makes it safe for `sc-transport` to replace a stale Unix socket
file when binding ([REQ-TRN-0004](../sc-transport/requirements.md)). The
guarantee covers the local instance root only; a shared Postgres store may
still be written by daemons on several hosts. File-lock semantics on network
filesystems are weaker, so an instance root on such a filesystem is not
protected.

### Alternatives Considered

- **PID file**: a file holding the daemon's process id, checked at start.
  Rejected because it goes stale after a crash and PID reuse makes the check
  unreliable.
- **Port or socket probing**: try to connect, and start only if nothing
  answers. Rejected because two starters race, and a hung daemon looks
  absent.
- **A lock inside the database**: rejected because it requires opening the
  database first, which is exactly what a second daemon must never do, and
  because `sc-runtime` has no database dependency.

### Implementation

**Enforced by:** the two-daemon test required by
[REQ-RT-0002](requirements.md): a second `run()` on the same instance root
returns the "lock held" error and its stores closure never runs.
`crates/sc-runtime/Cargo.toml` lists `fd-lock`.

### Related Documents

- [REQ-RT-0002](requirements.md)
- [REQ-TRN-0004](../sc-transport/requirements.md)

---

## ADR-RT-0003: The project's `Stores`, router and rmcp service pass through

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19, section "The crates" (`Stores` is a plain struct defined by the project); applies repo [ADR-RUN-0001](../architecture.md)  

### Context

A generated project has one or two stores, each a project-owned crate bound
to one sqlx driver (SQLite, Postgres). `sc-runtime` sits between the code
that opens those stores and the routes and MCP tools that use them. If it
constructed the stores itself it would need a sqlx dependency, knowledge of
each backend, and an opinion about how many stores a project has. The
repo-level decision is that the framework instantiates and the project wires,
with no wrapper around an axum, rmcp or sqlx type
([ADR-RUN-0001](../architecture.md)).

### Decision

`Stores` is a generic type parameter of the builder. Its value is produced by
the project's stores closure and handed only to the project's routes closure
and optional MCP closure. `sc-runtime` never reads a field of it, calls a
method on it, or requires a trait describing its contents. The
`utoipa_axum::OpenApiRouter` returned by the routes closure and the rmcp
Streamable HTTP service returned by the MCP closure are mounted as they are,
without being wrapped in an `sc-runtime` type. Opening pools and running
migrations happen inside the project's closure. API names are illustrative
until the contract sprint pins them.

### Consequences

`sc-runtime` has no sqlx dependency, and a project with no database at all
can use it (for example with `Stores = ()`). Which route uses which store
stays entirely in project code. The crate cannot offer store-aware features
such as a database health check or a migration command; a project adds those
in its own router. The health route `sc-runtime` mounts therefore reports on
the process only.

### Alternatives Considered

- **A `Store` trait the framework calls** (for example `open`, `migrate`,
  `health`): rejected because it makes the framework define what a store is
  and forces every project store to fit it.
- **Built-in pool construction** from config inside `sc-runtime`: rejected
  because it requires a sqlx dependency and per-backend code in this crate,
  and sqlx requires one crate per database for checked queries.

### Implementation

**Enforced by:** the `forbidden_edges` entry for `sqlx` in the boundary
manifest under `boundaries/sc-runtime/`, checked by `sc-lint-boundary`
through `just lint`; the opaque-`Stores` tests required by
[REQ-RT-0003](requirements.md).

### Related Documents

- [REQ-RT-0003](requirements.md)
- [REQ-RT-0004](requirements.md)
- [NFR-RT-0002](requirements.md)

---

## ADR-RT-0004: `DaemonFixture` runs the real daemon

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19, section "The crates" (`DaemonFixture` starts a daemon on a tempdir instance root per test)  

### Context

Generated projects test each operation through REST, MCP and the CLI client.
Test fixtures often take a shortcut, such as calling the axum router
in-process with no listener. Such tests skip the lock, the bind, the
transport and shutdown, and drift from production behaviour without anyone
noticing. A single daemon shared by all tests in a binary avoids the start-up
cost but makes tests share a database and a socket, so they cannot run in
parallel or in isolation.

### Decision

`sc_runtime::testing::DaemonFixture` starts a daemon through the same
`Daemon::builder()` path as production, on a fresh temporary directory used
as the instance root, with a real `daemon.lock`, a real listener bound
through `sc-transport`, and a real `sc_transport::Client` connected to it.
There is one daemon per fixture and one fixture per test. There is no
in-memory or listener-less mode. The name is illustrative until the contract
sprint pins it.

### Consequences

Every test exercises lock, bind, transport and shutdown. Because each fixture
has its own instance root, fixtures share no socket, lock or database file
and run in parallel in one test binary. Each fixture costs a tempdir and a
socket, which is cheap enough for that. A bug in start-up or shutdown shows
up as a failure in many tests at once, which is the intended signal.

### Alternatives Considered

- **An in-memory router mode** that calls the router with no listener:
  rejected because it skips lock, bind, transport and shutdown, the parts
  most likely to differ between test and production.
- **One shared daemon per test binary**: rejected because tests would share
  state and a socket, could not run in parallel safely, and would break the
  repo-level rule that every test uses its own tempdir instance root.

### Implementation

**Enforced by:** `req-qa` requirements review of the tests required by
[REQ-RT-0006](requirements.md), in particular the concurrent-fixtures test;
inspection that `DaemonFixture` calls `Daemon::builder()` and has no separate
serve path.

### Related Documents

- [REQ-RT-0006](requirements.md)
- [NFR-RUN-0005](../requirements.md)
