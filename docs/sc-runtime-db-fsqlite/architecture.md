# sc-runtime-db-fsqlite Architecture

This document defines the architecture of `sc-runtime-db-fsqlite`, the
FrankenSQLite implementation of `StorageBackend` for the sc-runtime workspace.

**Status: FUTURE — not implemented in v0.1.0.**

Related documents:
- [docs/sc-runtime-db-fsqlite/requirements.md](./requirements.md)
- [docs/sc-runtime-db/architecture.md](../sc-runtime-db/architecture.md)
- [docs/sc-runtime-db-sqlite/architecture.md](../sc-runtime-db-sqlite/architecture.md)
- [docs/prd/sc-runtime-prd.md](../prd/sc-runtime-prd.md) — §6.6, ADR-09
- FrankenSQLite repository: https://github.com/Dicklesworthstone/frankensqlite.git

---

## Role

`sc-runtime-db-fsqlite` will implement the `StorageBackend` trait using
FrankenSQLite's native MVCC mode. When implemented, it will be a drop-in
replacement for `sc-runtime-db-sqlite` — same trait, new implementation,
builder configuration change only.

---

## Why This Crate Exists Now

This crate directory and documentation exist in v0.1.0 to:

1. Document the intent and timeline for the FrankenSQLite backend.
2. Reserve the crate name and boundary definition in the workspace.
3. Provide the implementation team with a clear specification to work from
   when FrankenSQLite native mode reaches stability.
4. Prevent the FrankenSQLite integration from being treated as an
   afterthought that requires consumer-visible API changes.

The `StorageBackend` trait is designed so this backend can be added with zero
consumer-facing change. Consumers will switch from `SqliteBackend::open()` to
`FsqliteBackend::open()` in their builder configuration — no other code
changes required.

---

## Why Implementation Is Deferred

FrankenSQLite operates in two modes:

**Compatibility mode** — rusqlite-over-C-SQLite. In this mode, FrankenSQLite is
functionally identical to standard rusqlite. It provides no additional value
over `sc-runtime-db-sqlite` and would be the same dependency with extra
indirection. ADR-09 explicitly rejects compatibility mode as a basis for this
crate.

**Native MVCC mode** — FrankenSQLite's purpose-built mode with native
multi-version concurrency control. This mode is the reason the crate is worth
implementing. It provides the throughput characteristics that make it
meaningfully different from standard SQLite.

As of v0.1.0, FrankenSQLite native MVCC mode does not expose a stable
`Connection` toggle in its public API. Implementing this crate against an
unstable internal API would create a maintenance burden with no benefit.

Implementation begins when FrankenSQLite ships a stable `Connection` toggle
for native mode in its public API.

---

## Performance Motivation

FrankenSQLite native mode benchmarks show approximately 41× concurrent write
throughput at 8 threads compared to standard SQLite WAL mode.

This matters for multi-plugin daemons in which multiple plugins write to a
shared database concurrently. In a daemon with 8 or more active plugins, each
performing frequent writes, the WAL mode ceiling in `sc-runtime-db-sqlite`
becomes visible. `sc-runtime-db-fsqlite` is the upgrade path for those
configurations without requiring any change to consuming code.

Single-process, single-writer tools — the common case for Phase B consumers —
will not observe a meaningful difference. The recommendation is:
- Start with `sc-runtime-db-sqlite`.
- Switch to `sc-runtime-db-fsqlite` when write throughput under concurrent
  plugin load is a measured concern.

---

## Planned Architecture (when implemented)

### Constructor

```rust
pub struct FsqliteBackend { /* private */ }

impl FsqliteBackend {
    /// Open or create a FrankenSQLite database at `path` in native MVCC mode.
    /// Respects SC_RUNTIME_DB override.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError>;

    /// Open an in-memory FrankenSQLite database in native MVCC mode.
    /// Intended for tests. Respects SC_RUNTIME_DB override.
    pub fn in_memory() -> Result<Self, StorageError>;
}
```

### Connection Mode

The `open()` constructor will use FrankenSQLite's native mode `Connection`
toggle, not the compatibility mode path. The compatibility mode is
intentionally excluded — if compatibility mode is needed, `sc-runtime-db-sqlite`
already provides it.

### SC_RUNTIME_DB Override

Both constructors will honour the `SC_RUNTIME_DB` environment variable
contract defined in `sc-runtime-db`, identical to the rusqlite implementation.

### Connection Pooling

`sc-runtime-db-fsqlite` will use a connection pool compatible with
FrankenSQLite's native mode. If FrankenSQLite ships an r2d2-compatible
connection manager, `r2d2` will be used for consistency with
`sc-runtime-db-sqlite`. If not, the implementation will use an equivalent pool
that satisfies the `StorageBackend` concurrency requirements.

### Migration Runner

The migration runner will follow the same logic as `sc-runtime-db-sqlite`:
ordered, idempotent, version-tracked via `_sc_migrations`. The migration SQL
format is defined in `sc-runtime-db::Migration` and is backend-neutral.

---

## Zero Consumer Impact

The `StorageBackend` trait is the mechanism that makes this upgrade
transparent. A consumer using `sc-runtime-db-sqlite` today switches to
`sc-runtime-db-fsqlite` by changing one line:

```rust
// Before
.db(Arc::new(SqliteBackend::open("my-tool.db")?))

// After
.db(Arc::new(FsqliteBackend::open("my-tool.db")?))
```

No other code changes. No changes to migrations, queries, or transaction code.
No changes to plugin implementations. The `Arc<dyn StorageBackend>` type that
flows through the application is identical.

---

## Trigger Condition for Implementation

Implementation of `sc-runtime-db-fsqlite` begins when **both** of the
following are true:

1. FrankenSQLite ships a stable, public `Connection` API toggle for native
   MVCC mode.
2. The sc-runtime team has verified the native mode API against the
   `StorageBackend` trait contract.

Until both conditions are met, this crate remains a reserved boundary
definition and this document remains the specification.

---

## Crate Position

```
sc-runtime-db                  ← StorageBackend trait
    ↑
sc-runtime-db-sqlite           ← rusqlite (available now)
sc-runtime-db-fsqlite          ← FrankenSQLite native mode (this crate, future)
sc-runtime-db-sqlx             ← sqlx (available now)
```
