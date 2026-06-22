# sc-runtime-db-fsqlite Requirements

This document defines the requirements for `sc-runtime-db-fsqlite`, the
FrankenSQLite implementation of `StorageBackend`.

**Status: RESERVED — not implemented in v0.1.0.**

Related documents:
- [docs/sc-runtime-db-fsqlite/architecture.md](./architecture.md)
- [docs/sc-runtime-db/requirements.md](../sc-runtime-db/requirements.md)
- [docs/sc-runtime-db-sqlite/requirements.md](../sc-runtime-db-sqlite/requirements.md)
- [docs/prd/sc-runtime-prd.md](../prd/sc-runtime-prd.md) — §6.6, §4 (Non-Goals), ADR-09
- FrankenSQLite repository: https://github.com/Dicklesworthstone/frankensqlite.git

---

## Status

`sc-runtime-db-fsqlite` is reserved. The crate name, boundary definition, and
this requirements document exist in v0.1.0 to document the intent and hold the
specification. No implementation code exists in this crate for v0.1.0.

**Trigger condition for implementation:** FrankenSQLite ships a stable,
publicly documented `Connection` API toggle for native MVCC mode.

Until that condition is met, this document is the specification.

---

## Scope (when implemented)

`sc-runtime-db-fsqlite` will implement the `StorageBackend` trait using
FrankenSQLite in native MVCC mode. The implementation scope is identical to
`sc-runtime-db-sqlite` with one substitution: the connection type and pool use
FrankenSQLite native mode instead of rusqlite.

---

## Functional Requirements (apply when implemented)

### FR-FSQLITE-01 — Native MVCC mode only

`sc-runtime-db-fsqlite` must implement `StorageBackend` using FrankenSQLite's
native MVCC mode `Connection`. Compatibility mode must not be used.

Rationale: FrankenSQLite compatibility mode is rusqlite-over-C-SQLite and
provides no additional value over `sc-runtime-db-sqlite`. The only reason to
implement this crate is the native MVCC mode throughput characteristics
(approximately 41× concurrent write throughput at 8 threads compared to
standard SQLite WAL mode). Using compatibility mode would duplicate
`sc-runtime-db-sqlite` with additional indirection.

If a consumer needs rusqlite semantics, they should use `sc-runtime-db-sqlite`.

### FR-FSQLITE-02 — Full StorageBackend implementation

`FsqliteBackend` must implement all methods of `StorageBackend`:
`execute`, `query`, `transaction`, `migrate`, and `health`.

The behavior contract for each method is identical to `sc-runtime-db-sqlite`.
The migration runner must be ordered, idempotent, and version-tracked via
`_sc_migrations` using the same schema defined in `sc-runtime-db-sqlite`.

### FR-FSQLITE-03 — SC_RUNTIME_DB environment variable override

Both `FsqliteBackend::open()` and `FsqliteBackend::in_memory()` must honour
the `SC_RUNTIME_DB` environment variable contract defined in `sc-runtime-db`.

When `SC_RUNTIME_DB` is set, its value overrides the caller-supplied path or
URL. The variable is read once at construction time.

### FR-FSQLITE-04 — WAL and connection pooling

`sc-runtime-db-fsqlite` must support WAL mode and connection pooling
appropriate for FrankenSQLite's native mode connection API.

If FrankenSQLite's native mode uses a different concurrency primitive than
WAL journal mode, the implementation must use whatever mechanism provides
equivalent or superior concurrent read/write behavior. The requirement is
concurrent reader/writer capability — not the specific WAL pragma name.

### FR-FSQLITE-05 — in_memory() constructor

`FsqliteBackend::in_memory()` must exist and must open an in-memory
FrankenSQLite database in native MVCC mode. This constructor is required for
test use.

### FR-FSQLITE-06 — Zero consumer-visible change at switchover

The switchover from `sc-runtime-db-sqlite` to `sc-runtime-db-fsqlite` must
require no code changes beyond the constructor call at the application entry
point. The `Arc<dyn StorageBackend>` interface must be identical.

---

## Boundary Requirements (apply when implemented)

### BR-FSQLITE-01 — Permitted dependencies

`sc-runtime-db-fsqlite` may depend on:
- `sc-runtime-db` (workspace) — required for the `StorageBackend` trait
- FrankenSQLite crate (version TBD at implementation time)
- A connection pool crate appropriate for FrankenSQLite's native mode API

It must not depend on `sc-runtime-db-sqlite`, `sc-runtime-db-sqlx`, or any
other sc-runtime backend crate.

---

## Non-Functional Requirements (apply when implemented)

### NF-FSQLITE-01 — No unsafe code

`sc-runtime-db-fsqlite` must compile with `#![forbid(unsafe_code)]`. FFI to
FrankenSQLite internals is encapsulated inside the FrankenSQLite crate itself.

### NF-FSQLITE-02 — Edition and toolchain

Edition 2024, `rust-version = 1.94.1`, consistent with the workspace
`rust-toolchain.toml`.

### NF-FSQLITE-03 — Version

`0.1.0` when first published, consistent with all sc-runtime workspace crates.
