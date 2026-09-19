# `sc-runtime` Architecture

Crate-level architecture and ADRs. Repo-level: [`../architecture.md`](../architecture.md).
Requirements: [`requirements.md`](requirements.md). Boundary manifest: `boundaries/sc-runtime/`.
All ADRs are `accepted` unless stated; an ADR changes only through a later ADR that names it.

## Role

The single assembly point. It is the only crate that knows the other three,
and it is a dependency of the daemon only.

## Public surface

`Daemon`, `DaemonBuilder`, `DaemonConfig`, `RuntimeError`,
`testing::DaemonFixture`.

## ADR

### ADR-RUNTIME-001 Assembly only, in a fixed order

Config, lock, stores, router merge, bind and serve, always in that order. The
lock is taken before any store opens so a second daemon never touches the
database. Applies repo ADR-001 and ADR-003.

### ADR-RUNTIME-002 Singleton by OS lock

`fd-lock` exclusive lock on `<instance-root>/daemon.lock`, held for the life
of the process. No PID files and no port probing.

### ADR-RUNTIME-003 The project's types pass through untouched

`Stores` is a generic parameter handed to the project's closures. Routers and
rmcp services are taken and mounted as they are. Applies repo ADR-001.

### ADR-RUNTIME-004 The fixture is the real daemon

`DaemonFixture` runs the same builder path on a tempdir instance root; there
is no separate in-memory mode to drift from production.
