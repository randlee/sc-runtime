# sc-runtime-db-sqlite Requirements

This document defines the requirements for `sc-runtime-db-sqlite`, the rusqlite
implementation of `StorageBackend`.

Related documents:
- [docs/sc-runtime-db-sqlite/architecture.md](./architecture.md)
- [docs/sc-runtime-db/requirements.md](../sc-runtime-db/requirements.md)
- [docs/prd/sc-runtime-prd.md](../prd/sc-runtime-prd.md) — §6.5, NF-04

---

## Scope

`sc-runtime-db-sqlite` is the rusqlite implementation of `StorageBackend`. It
covers connection management, WAL configuration, connection pooling, and the
migration runner. It does not define the `StorageBackend` trait or the
`Migration` type — those belong to `sc-runtime-db`.

---

## Functional Requirements

### FR-SQLITE-01 — WAL journal mode on every connection

Every SQLite connection opened by this crate must have WAL journal mode
enabled. WAL mode must be set via `PRAGMA journal_mode = WAL` immediately after
the connection is established, before it is returned from the pool for use.

Connections that do not carry WAL mode are a correctness defect: a write may
block a concurrent reader, violating the expected concurrency model.

### FR-SQLITE-02 — Connection pooling via r2d2-sqlite

Connection pooling must be implemented using `r2d2-sqlite`. A raw rusqlite
`Connection` must not be held permanently in `SqliteBackend`. All methods
acquire a connection from the pool for the duration of the call.

The pool must configure the WAL pragma through the r2d2 connection
customizer so the pragma applies to every new connection regardless of
pool slot assignment.

### FR-SQLITE-03 — Migration runner: ordered, idempotent, version-tracked

The `migrate()` method must:

1. Create a `_sc_migrations` tracking table on first run if it does not exist.
2. Sort the supplied migrations by `version` ascending before applying.
3. Skip any migration whose `version` is already recorded in `_sc_migrations`
   (idempotency).
4. Execute each new migration's `up` SQL inside a transaction.
5. Record the applied `version` in `_sc_migrations` within the same
   transaction.

It is not a requirement for `migrate()` to ever call `down` SQL
automatically. Rollback is a deliberate operator action.

### FR-SQLITE-04 — SC_RUNTIME_DB environment variable override

Both `SqliteBackend::open()` and `SqliteBackend::in_memory()` must check the
`SC_RUNTIME_DB` environment variable at construction time. If the variable is
set, its value is used as the database file path instead of the
caller-supplied argument.

This enables test isolation: setting `SC_RUNTIME_DB` in the test environment
overrides the database path for all backend constructions in that process
without modifying application code.

### FR-SQLITE-05 — in_memory() constructor

`SqliteBackend::in_memory()` must exist and must open a SQLite in-memory
database (`:memory:`). This constructor is the recommended path for unit and
integration tests that do not require persistence.

Each call to `in_memory()` produces an independent database. In-memory
databases do not share state across `SqliteBackend` instances.

### FR-SQLITE-06 — rusqlite bundled feature

The `rusqlite` dependency must be declared with the `bundled` feature enabled:

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
```

The bundled feature compiles SQLite from source as part of the Cargo build. No
system SQLite library (libsqlite3) must be required to build or run this crate.
This is NF-04 from the PRD.

### FR-SQLITE-07 — StorageBackend implementation completeness

`SqliteBackend` must implement all methods of `StorageBackend`:
`execute`, `query`, `transaction`, `migrate`, and `health`.

`health()` must return `StorageHealth::Healthy` when a ping query succeeds and
`StorageHealth::Degraded` (with a descriptive reason string) when it fails.
`health()` must never panic or return an error type.

---

## Boundary Requirements

### BR-SQLITE-01 — Permitted dependencies

`sc-runtime-db-sqlite` may depend on:
- `sc-runtime-db` (workspace) — required for the `StorageBackend` trait
- `rusqlite` (0.31, bundled feature)
- `r2d2-sqlite` (latest)

It must not depend on `sc-runtime-db-sqlx`, `sc-runtime-db-fsqlite`, or any
other backend implementation crate.

### BR-SQLITE-02 — No sc-runtime-core direct dependency

`sc-runtime-db-sqlite` receives `StorageError` and `StorageHealth` through
`sc-runtime-db`. It must not take a direct dependency on `sc-runtime-core`
unless a specific type from that crate is required by the implementation.

---

## Non-Functional Requirements

### NF-SQLITE-01 — No unsafe code

`sc-runtime-db-sqlite` must compile with `#![forbid(unsafe_code)]`. rusqlite's
internal FFI is encapsulated inside rusqlite itself; this crate's Rust code has
no reason to use unsafe.

### NF-SQLITE-02 — Edition and toolchain

Edition 2024, `rust-version = 1.94.1`, consistent with the workspace
`rust-toolchain.toml`.

### NF-SQLITE-03 — Version

`0.1.0`, consistent with all sc-runtime workspace crates.

### NF-SQLITE-04 — Cross-platform build

`sc-runtime-db-sqlite` must compile on macOS, Linux, and Windows. The bundled
rusqlite feature must be the mechanism that achieves this without system
library configuration. `cargo xwin check` must pass for
`x86_64-pc-windows-msvc`.
