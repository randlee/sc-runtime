# sc-runtime-db-sqlite Architecture

This document defines the architecture of `sc-runtime-db-sqlite`, the rusqlite
implementation of `StorageBackend` for the sc-runtime workspace.

Related documents:
- [docs/sc-runtime-db-sqlite/requirements.md](./requirements.md)
- [docs/sc-runtime-db/architecture.md](../sc-runtime-db/architecture.md)
- [docs/prd/sc-runtime-prd.md](../prd/sc-runtime-prd.md) — §6.5

---

## Role

`sc-runtime-db-sqlite` implements the `StorageBackend` trait using rusqlite
with the bundled SQLite feature. It is the recommended backend for all
single-process SC tools — no external database server, no system SQLite library
required.

---

## SQLite Connection Configuration

Every connection opened by `sc-runtime-db-sqlite` is configured with WAL
(Write-Ahead Logging) journal mode immediately after open:

```sql
PRAGMA journal_mode = WAL;
```

WAL mode enables concurrent reads alongside writes. In WAL mode, readers do not
block writers and writers do not block readers. This matters for daemon
configurations where multiple plugins share a single `Arc<dyn StorageBackend>`:
a plugin executing a long read query does not stall another plugin's write.

WAL mode is set at the connection level inside the r2d2 connection
customizer, ensuring every connection in the pool carries it regardless of
which pool slot is dispatched.

Additional connection-level pragmas set on open:

| Pragma | Value | Reason |
|--------|-------|--------|
| `journal_mode` | `WAL` | Concurrent read/write |
| `synchronous` | `NORMAL` | Safe with WAL; faster than FULL |
| `foreign_keys` | `ON` | Enforce FK constraints |
| `busy_timeout` | `5000` | 5s retry before SQLITE_BUSY error |

---

## Connection Pool

`sc-runtime-db-sqlite` uses `r2d2-sqlite` for connection pooling. The pool
manages a fixed-size set of rusqlite `Connection` objects and loans them to
callers for the duration of each operation.

Pool parameters:

| Parameter | Default | Notes |
|-----------|---------|-------|
| `max_size` | 4 | Tunable; SQLite WAL supports limited concurrency |
| `min_idle` | 1 | Keep at least one connection warm |
| `connection_timeout` | 10s | Error if pool exhausted for this long |

Pool configuration defaults:
- max_size: 4 connections (SQLite WAL supports multiple readers; a small pool avoids write contention)
- min_idle: None (connections are created on demand)
- max_lifetime: None (SQLite connections are cheap; no recycling needed)
- idle_timeout: 60 seconds (close idle connections after 1 minute)
- connection_timeout: 5 seconds (fail fast if pool is exhausted)

The pool is held in `SqliteBackend` behind an `Arc`, making `SqliteBackend`
cheaply cloneable and shareable across threads as `Arc<dyn StorageBackend>`.

```
SqliteBackend
  └── Arc<Pool<SqliteConnectionManager>>
        └── [Connection₀, Connection₁, ..., ConnectionN]
              each configured with WAL + pragmas on creation
```

---

## Migration Runner

`sc-runtime-db-sqlite` implements `migrate()` as follows:

1. **Version table creation** — on first call, create `_sc_migrations` if it
   does not exist:

   ```sql
   CREATE TABLE IF NOT EXISTS _sc_migrations (
       version     INTEGER PRIMARY KEY,
       description TEXT    NOT NULL,
       applied_at  TEXT    NOT NULL   -- ISO 8601
   );
   ```

2. **Ordering** — sort the supplied `&[Migration]` by `version` ascending.

3. **Idempotency check** — for each migration, query `_sc_migrations` for its
   `version`. If a row exists, skip. If not, execute `up` SQL and insert the
   version record.

4. **Transaction** — each migration's `up` SQL and the version record insert
   are wrapped in a single transaction. A failure rolls back that migration
   without affecting already-applied migrations.

5. **No rollback on re-run** — `migrate()` never calls `down` SQL
   automatically. Rollback is a deliberate operator action, not an automatic
   behavior.

This makes `migrate()` safe to call at every startup. Calling it multiple times
on the same database applies only new migrations.

---

## SC_RUNTIME_DB Override

At construction time, `SqliteBackend::open()` and `SqliteBackend::in_memory()`
both check the `SC_RUNTIME_DB` environment variable before using their
argument:

```rust
let effective_path = std::env::var("SC_RUNTIME_DB")
    .map(PathBuf::from)
    .unwrap_or_else(|_| path.as_ref().to_owned());
```

For `in_memory()`, if `SC_RUNTIME_DB` is set to `:memory:` or is empty, the
in-memory path is used. If set to a file path, that file path is used instead.

This ensures test isolation: a test harness sets `SC_RUNTIME_DB` to a
temporary directory path and all `SqliteBackend` constructions in that process
use the override automatically.

---

## Bundled SQLite

`sc-runtime-db-sqlite` enables the `bundled` feature of `rusqlite`:

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
```

The `bundled` feature compiles SQLite from source as part of the Cargo build.
The resulting binary carries its own SQLite; it does not link against any
system-provided `libsqlite3`. This guarantees:

- identical SQLite version across all developer machines and CI environments
- no build failure on systems without SQLite development headers installed
- deterministic behavior regardless of OS-provided SQLite version

Cross-compilation via `cargo xwin` works without separate SQLite configuration
on the Windows target.

---

## Constructors

```rust
pub struct SqliteBackend { /* private */ }

impl SqliteBackend {
    /// Open or create a SQLite database at `path`.
    /// Respects SC_RUNTIME_DB override.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError>;

    /// Open an in-memory SQLite database.
    /// Intended for tests. Respects SC_RUNTIME_DB override.
    pub fn in_memory() -> Result<Self, StorageError>;
}
```

`open()` creates the file if it does not exist. Parent directories must exist;
`open()` does not create them. The caller (typically the application entry
point) is responsible for ensuring the directory exists.

`in_memory()` opens `sqlite://:memory:`. Each call returns an independent
in-memory database; they do not share state. This is the recommended
constructor for unit tests and integration tests that do not need persistence.

---

## StorageBackend Implementation

`SqliteBackend` implements `StorageBackend` from `sc-runtime-db`:

| Method | Implementation |
|--------|---------------|
| `execute` | `conn.execute(sql, params)` — returns affected row count |
| `query` | `conn.query_map(sql, params)` — collects into owned `Rows` |
| `transaction` | `conn.transaction()` — runs closure, commits or rolls back |
| `migrate` | Migration runner described above |
| `health` | Ping via `SELECT 1` — returns `StorageHealth::Healthy` or `StorageHealth::Degraded` with reason string |

All methods acquire a connection from the r2d2 pool for the duration of the
call and return it on drop.

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `sc-runtime-db` | workspace | `StorageBackend` trait, `Migration`, `StorageError` |
| `rusqlite` | 0.31 (bundled) | SQLite connection and query execution |
| `r2d2-sqlite` | 0.23 | Connection pool management |

No other sc-runtime crates appear in `sc-runtime-db-sqlite`'s dependency tree.
This boundary is enforced by `sc-lint-boundary`.

---

## Crate Position

```
sc-runtime-db           ← StorageBackend trait
    ↑
sc-runtime-db-sqlite    ← this crate — rusqlite + WAL + r2d2 pool
```

Consumers receive `Arc<dyn StorageBackend>` from the sc-runtime builder; they
never depend on `sc-runtime-db-sqlite` directly unless constructing the backend
themselves (in the application entry point).
