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
| `DaemonConfig` | the daemon's own settings (endpoint, instance root), deserialisable as part of the project's config |
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

## ADR-RT-0001: Assembly only, in a fixed order

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "sc-runtime is a bootstrap, not a framework layer"; applies repo [ADR-RUN-0001](../architecture.md), [ADR-RUN-0201](../architecture.md)  

### Context

The assembly point could grow into a framework layer with its own
lifecycle hooks and plugin points.

### Decision

`run()` does five things in one fixed order: config, lock,
stores, router merge, bind and serve. There are no hooks between them. The lock
precedes stores; binding is last.

### Consequences

Start-up behaviour is identical in every project and can be
read in one function. A project that needs something between steps does it
inside its own closure.

### Alternatives Considered

Lifecycle hooks; a plugin registry; configurable step order.

### Implementation

**Enforced by:** `arch-qa`; the step-order test.

### Related Documents

- [REQ-RT-0001](requirements.md)
- [NFR-RT-0001](requirements.md)

---

## ADR-RT-0002: Singleton by OS lock

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "Runtime architecture"  

### Context

Singletons are commonly done with PID files or by probing the
port. Both go stale after a crash and both race.

### Decision

An `fd-lock` exclusive lock on `<instance-root>/daemon.lock`,
held for the life of the process.

### Consequences

The kernel releases the lock on any exit, so there is nothing
to clean up. Holding it is also what makes replacing a stale socket file safe.

### Alternatives Considered

PID files; port probing; a lock inside the database.

### Implementation

**Enforced by:** The two-daemon test.

### Related Documents

- [REQ-RT-0002](requirements.md)
- [REQ-TRN-0004](../sc-transport/requirements.md)

---

## ADR-RT-0003: The project's types pass through untouched

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "`Stores` is a plain struct defined by the project"; applies repo [ADR-RUN-0001](../architecture.md)  

### Context

To construct stores itself, the crate would need to know sqlx and
the project's backends.

### Decision

`Stores` is a generic parameter produced by the project's closure
and handed back to the project's closures. Routers and rmcp services are taken
and mounted as they are.

### Consequences

No sqlx dependency here; a project with no database at all
can use the crate. The crate cannot offer store-aware features such as a
database health check; a project adds those in its own router.

### Alternatives Considered

A `Store` trait the framework calls; built-in pool construction.

### Implementation

**Enforced by:** `forbidden_edges` to sqlx.

### Related Documents

- [REQ-RT-0003](requirements.md)
- [REQ-RT-0004](requirements.md)
- [NFR-RT-0002](requirements.md)

---

## ADR-RT-0004: The fixture is the real daemon

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design decision 8 (`DaemonFixture`)  

### Context

Test fixtures often take a shortcut, such as calling the router
in-process without a listener, and then drift from production behaviour.

### Decision

`DaemonFixture` runs the same builder path on a tempdir instance
root with a real listener and a real client.

### Consequences

Tests exercise lock, bind, transport and shutdown every time.
Each fixture costs a socket and a tempdir, which is cheap enough to run in
parallel.

### Alternatives Considered

An in-memory router mode; a shared daemon per test binary.

### Implementation

**Enforced by:** `req-qa` on [REQ-RT-0006](requirements.md).

### Related Documents

- [REQ-RT-0006](requirements.md)
- [NFR-RUN-0005](../requirements.md)
