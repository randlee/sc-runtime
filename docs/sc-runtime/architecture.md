# `sc-runtime` Architecture

Crate-level architecture and ADRs for `sc-runtime`. Repo-level architecture is in
[`../architecture.md`](../architecture.md); this crate's requirements are in
[`requirements.md`](requirements.md); its boundary manifest is
`boundaries/sc-runtime/`. ADRs follow
`.claude/skills/plan-hardening/req-adr-format.md`. Every ADR here is `accepted`
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

## ADR

### ADR-RUNTIME-001 Assembly only, in a fixed order

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design "sc-runtime is a bootstrap, not a framework layer"; applies repo ADR-001, ADR-003 |
| Relates to | `REQ-RUNTIME-001`, `NFR-RUNTIME-001` |

**Context.** The assembly point could grow into a framework layer with its own
lifecycle hooks and plugin points.

**Decision.** `run()` does five things in one fixed order: config, lock,
stores, router merge, bind and serve. There are no hooks between them. The lock
precedes stores; binding is last.

**Consequences.** Start-up behaviour is identical in every project and can be
read in one function. A project that needs something between steps does it
inside its own closure.

**Rejected.** Lifecycle hooks; a plugin registry; configurable step order.

**Enforced by.** `arch-qa`; the step-order test.

### ADR-RUNTIME-002 Singleton by OS lock

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design "Runtime architecture" |
| Relates to | `REQ-RUNTIME-002`, `REQ-TRANSPORT-004` |

**Context.** Singletons are commonly done with PID files or by probing the
port. Both go stale after a crash and both race.

**Decision.** An `fd-lock` exclusive lock on `<instance-root>/daemon.lock`,
held for the life of the process.

**Consequences.** The kernel releases the lock on any exit, so there is nothing
to clean up. Holding it is also what makes replacing a stale socket file safe.

**Rejected.** PID files; port probing; a lock inside the database.

**Enforced by.** The two-daemon test.

### ADR-RUNTIME-003 The project's types pass through untouched

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design "`Stores` is a plain struct defined by the project"; applies repo ADR-001 |
| Relates to | `REQ-RUNTIME-003`, `REQ-RUNTIME-004`, `NFR-RUNTIME-002` |

**Context.** To construct stores itself, the crate would need to know sqlx and
the project's backends.

**Decision.** `Stores` is a generic parameter produced by the project's closure
and handed back to the project's closures. Routers and rmcp services are taken
and mounted as they are.

**Consequences.** No sqlx dependency here; a project with no database at all
can use the crate. The crate cannot offer store-aware features such as a
database health check; a project adds those in its own router.

**Rejected.** A `Store` trait the framework calls; built-in pool construction.

**Enforced by.** `forbidden_edges` to sqlx.

### ADR-RUNTIME-004 The fixture is the real daemon

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design decision 8 (`DaemonFixture`) |
| Relates to | `REQ-RUNTIME-006`, `NFR-REPO-005` |

**Context.** Test fixtures often take a shortcut, such as calling the router
in-process without a listener, and then drift from production behaviour.

**Decision.** `DaemonFixture` runs the same builder path on a tempdir instance
root with a real listener and a real client.

**Consequences.** Tests exercise lock, bind, transport and shutdown every time.
Each fixture costs a socket and a tempdir, which is cheap enough to run in
parallel.

**Rejected.** An in-memory router mode; a shared daemon per test binary.

**Enforced by.** `req-qa` on `REQ-RUNTIME-006`.
