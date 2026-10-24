# sc-runtime-db Architecture

This document defines the architecture of `sc-runtime-db`, the storage
abstraction crate for the sc-runtime workspace.

Related documents:
- [docs/sc-runtime-db/requirements.md](./requirements.md)
- [docs/sc-runtime-db-sqlite/architecture.md](../sc-runtime-db-sqlite/architecture.md)
- [docs/sc-runtime-db-sqlx/architecture.md](../sc-runtime-db-sqlx/architecture.md)
- [docs/sc-runtime-db-fsqlite/architecture.md](../sc-runtime-db-fsqlite/architecture.md)
- [docs/prd/sc-runtime-prd.md](../prd/sc-runtime-prd.md) — §6.4

---

## Role

`sc-runtime-db` defines the `StorageBackend` trait and the `Migration` type.
It contains zero implementations. Every crate that needs to accept storage
accepts `Arc<dyn StorageBackend>` without depending on any backend crate.

---

## Why a Trait-Only Crate

The five sc-runtime capability axes are independent. A consumer that needs
storage should not be forced to compile a SQLite or PostgreSQL dependency unless
it selects that backend. The trait-only crate achieves this separation:

- `sc-runtime-db` — the abstraction; always a cheap dependency
- `sc-runtime-db-sqlite` — rusqlite implementation; pulled in only when needed
- `sc-runtime-db-sqlx` — sqlx implementation; pulled in only when needed
- `sc-runtime-db-fsqlite` — FrankenSQLite implementation (future)

Any crate that accepts storage — including the sc-runtime facade, daemon
plugins, and consumer domain crates — depends only on `sc-runtime-db` and
receives an `Arc<dyn StorageBackend>` at construction time. No concrete backend
type ever leaks into a consumer's public API.

This is ADR-04 from the PRD: four separate crates (trait + one per
implementation) so consumers take exactly one backend with no dead code.

---

## StorageBackend Trait

```rust
pub trait StorageBackend: Send + Sync {
    fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<u64, StorageError>;
    fn query(&self, sql: &str, params: &[SqlParam]) -> Result<Rows, StorageError>;
    fn transaction(
        &self,
        f: Box<dyn FnOnce(&dyn StorageBackend) -> Result<(), StorageError> + Send>,
    ) -> Result<(), StorageError>;
    fn migrate(&self, migrations: &[Migration]) -> Result<(), StorageError>;
    fn health(&self) -> StorageHealth;
}
```

### Method Contracts

| Method | Returns | Notes |
|--------|---------|-------|
| `execute` | affected row count | INSERT, UPDATE, DELETE, DDL |
| `query` | `Rows` — owned, cursor-like result set | SELECT |
| `transaction` | `()` on success | closure receives `&dyn StorageBackend`; callers capture output via `&mut` (see below) |
| `migrate` | `()` on success | ordered, idempotent — see Migration type below |
| `health` | `StorageHealth` — infallible | never `Result`; see RBP-007 note |

### Extracting Values from a Transaction

`transaction` returns `()` rather than a generic `R` in order to remain
object-safe (RBP-008). Callers that need to extract a value produced inside
the transaction write to a captured mutable variable:

```rust
let mut inserted_id: i64 = 0;

backend.transaction(Box::new(|tx| {
    tx.execute(
        "INSERT INTO items (name) VALUES (?1)",
        &[SqlParam::Text("hello".into())],
    )?;
    let rows = tx.query("SELECT last_insert_rowid()", &[])?;
    inserted_id = rows.get_i64(0, 0)?;
    Ok(())
}))?;

// inserted_id is available here
println!("new row id: {}", inserted_id);
```

### `health()` Is Infallible

`health()` returns `StorageHealth`, not `Result<StorageHealth, _>`. An
unhealthy backend is a valid state that `health()` reports; it is not itself an
error. This follows RBP-007 (Infallible accessors return values, not Results).
The sc-runtime health command aggregates `StorageHealth` from all configured
backends into the `HealthResult` envelope.

---

## Migration Type

```rust
pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub up: &'static str,    // SQL DDL — cross-backend subset; see permitted/forbidden lists below
    pub down: &'static str,
}
```

`version` is a monotonically increasing integer. The migration runner in each
backend implementation orders by `version` ascending, applies only migrations
whose version exceeds the current schema version, and records each applied
version in a `_sc_migrations` tracking table. Re-running `migrate()` with
already-applied migrations is a no-op (idempotent).

`up` and `down` SQL must use the cross-backend DDL subset defined below — no
vendor-specific syntax. Backends are responsible for executing the SQL; the
trait crate defines the data carrier only.

### Cross-Backend DDL Subset

The following SQL constructs work for both SQLite and PostgreSQL backends.

**Permitted in migration files:**

- `CREATE TABLE IF NOT EXISTS`, `DROP TABLE IF EXISTS`
- `CREATE INDEX IF NOT EXISTS`, `DROP INDEX IF EXISTS`
- Data types: `INTEGER`, `TEXT`, `REAL`, `BLOB`, `BOOLEAN` (stored as INTEGER 0/1 in SQLite, native BOOLEAN in Postgres)
- Constraints: `NOT NULL`, `PRIMARY KEY`, `UNIQUE`, `CHECK`, `DEFAULT`
- `ALTER TABLE ... ADD COLUMN` (nullable or with DEFAULT only — SQLite restriction)
- `INSERT`, `UPDATE`, `DELETE`, `SELECT` with standard `WHERE`/`JOIN`/`GROUP BY`/`ORDER BY`

**Forbidden in migration files** (use sparingly; document in migration comment if needed):

- `AUTOINCREMENT` keyword — use `INTEGER PRIMARY KEY` instead (auto-increments in both backends)
- `VARCHAR(n)` — use `TEXT` instead (SQLite ignores the length anyway)
- PostgreSQL-specific: `SERIAL`, `BIGSERIAL`, `UUID` type, `JSONB`, `ARRAY[]`
- SQLite-specific: `WITHOUT ROWID`
- `PRAGMA` statements (SQLite only — backend handles these internally)

---

## RBP-003: Sealed Trait

`StorageBackend` is sealed within the sc-runtime workspace. The sealing
mechanism uses a private marker trait in a `private` module that external
crates cannot name:

```rust
mod private {
    pub trait Sealed {}
}

pub trait StorageBackend: private::Sealed + Send + Sync {
    // ...
}
```

External crates may:
- hold `Arc<dyn StorageBackend>`
- call any method on `Arc<dyn StorageBackend>`
- pass `Arc<dyn StorageBackend>` to `ScRuntimeBuilder::db()`

External crates may not:
- implement `StorageBackend` for their own types (the `private::Sealed` bound
  is unnameable outside the workspace)

This preserves the workspace's ability to add methods to `StorageBackend` in a
minor release without a public semver break. Consumers hold the trait object —
they never implement it.

---

## RBP-008: Trait Object Safety

`StorageBackend` must be object-safe so that `Arc<dyn StorageBackend>` is a
valid type. Design decisions that preserve object safety:

1. **No generic methods with unconstrained type parameters.** A generic
   `transaction<F, R>` would be object-safe only if bounded by `where Self:
   Sized`, which would remove it from the vtable entirely. Instead,
   `transaction` takes a boxed closure and returns `()`:

   ```rust
   fn transaction(
       &self,
       f: Box<dyn FnOnce(&dyn StorageBackend) -> Result<(), StorageError> + Send>,
   ) -> Result<(), StorageError>;
   ```

   The closure returns `()`. Callers that need a value out of the transaction
   capture it via a mutable reference rather than returning it from the
   closure. See the "Extracting Values from a Transaction" example in the
   Method Contracts section above.

2. **No `Self` return positions.** Constructors (`open`, `in_memory`) live on
   concrete types, not on the trait.

3. **No associated types that vary per impl.** `Rows` and `SqlParam` are
   concrete types defined in `sc-runtime-db`, not associated types.

4. **`health()` returns a concrete `StorageHealth` struct.** No associated
   return type.

Object safety is verified at design time by ensuring `dyn StorageBackend` is
nameable in the workspace codebase (the compiler will reject it if violated).

---

## SC_RUNTIME_DB Environment Variable

Every backend implementation must read `SC_RUNTIME_DB` at construction time.
The contract is defined here in the trait crate because it applies uniformly
across all implementations:

- If `SC_RUNTIME_DB` is set, the backend uses its value as the database path
  or connection URL instead of the caller-supplied path or URL.
- If `SC_RUNTIME_DB` is not set, the backend uses the caller-supplied argument.
- The variable is read once at construction — not on every query.

This contract enables test isolation: a test sets `SC_RUNTIME_DB` to a
temporary path (or `sqlite://:memory:`) and any backend opened during the test
process uses that override, regardless of what the application code passes to
the constructor.

The override is documented here rather than left to each implementation to
discover independently, ensuring all backends honour it consistently.

---

## Dependency

`sc-runtime-db` depends on:

| Crate | Version | Purpose |
|-------|---------|---------|
| `sc-runtime-core` | workspace | `StorageError` uses `ScRuntimeError` variants; `StorageHealth` reports into `PluginContext` health |

No backend crates (`rusqlite`, `sqlx`, `r2d2-sqlite`) appear in
`sc-runtime-db`'s dependency tree. This boundary is enforced by the
`sc-lint-boundary` boundary definition in `boundaries/sc-runtime-db/`.

---

## Crate Position in the Storage Axis

```
sc-runtime-db           ← trait only — this crate
    ↑
sc-runtime-db-sqlite    ← rusqlite implementation
sc-runtime-db-sqlx      ← sqlx implementation (sqlite + postgres)
sc-runtime-db-fsqlite   ← FrankenSQLite implementation (future)
```

Consumers and the sc-runtime facade depend on `sc-runtime-db` and receive an
`Arc<dyn StorageBackend>` at construction. They never depend on any
implementation crate directly.

---

## Supporting Types

These types are used throughout the `StorageBackend` trait and appear in
consumer code. They are defined in `sc-runtime-db` alongside the trait.

### `StorageHealth`

```rust
// StorageHealth — infallible snapshot of backend health
pub enum StorageHealth {
    Healthy,
    Degraded {
        reason: String,   // human-readable, not stable for parsing
        code: ErrorCode,  // stable machine-readable code e.g. "SC_RUNTIME.DB.CONNECTION_POOL_EXHAUSTED"
    },
    Unavailable {
        reason: String,
        code: ErrorCode,
    },
}
```

**Notes:**
- `Degraded.reason` and `Unavailable.reason` are human-readable only. Consumers
  must not key on the string value; use `.code` for programmatic handling.
- The `reason` field in `Degraded` and `Unavailable` is a human-readable string
  for display purposes only. Its format is not stable across versions and must
  not be parsed programmatically. For machine-readable health discrimination,
  use the `code: ErrorCode` field, which is stable. Example codes:
  `SC_RUNTIME.DB.CONNECTION_POOL_EXHAUSTED`, `SC_RUNTIME.DB.WAL_CHECKPOINT_FAILED`,
  `SC_RUNTIME.DB.MIGRATION_PENDING`.
- `Healthy` carries no payload — zero-allocation fast path for the common case.

### `SqlParam`

```rust
// SqlParam — value bound to a positional query parameter
pub enum SqlParam<'a> {
    Null,
    Integer(i64),
    Real(f64),
    Text(&'a str),
    Blob(&'a [u8]),
    Bool(bool),
}
```

**Notes:**
- `Text` holds a borrowed `&str` — the caller owns the string for the duration
  of the `execute()` or `query()` call.
- `Blob` likewise borrows `&[u8]` from the caller.
- Positional binding uses 1-based `?1`, `?2`, … placeholders in SQL.

### `Rows` and `Row`

```rust
// Rows — iterator over query result rows
pub struct Rows { /* opaque */ }

impl Rows {
    // Returns the next row, or None if exhausted.
    // Row exhaustion is not an error — next_row() returns Option, not Result.
    pub fn next_row(&mut self) -> Option<Row>;
    pub fn column_count(&self) -> usize;
    pub fn column_name(&self, idx: usize) -> Option<&str>;
}

pub struct Row { /* opaque */ }

impl Row {
    pub fn get_i64(&self, idx: usize) -> Result<i64, StorageError>;
    pub fn get_f64(&self, idx: usize) -> Result<f64, StorageError>;
    pub fn get_str(&self, idx: usize) -> Result<&str, StorageError>;
    pub fn get_bytes(&self, idx: usize) -> Result<&[u8], StorageError>;
    pub fn get_bool(&self, idx: usize) -> Result<bool, StorageError>;
    pub fn is_null(&self, idx: usize) -> bool;
}
```

**Notes:**
- `Rows` is not `Clone` — it is consumed by iteration.
- Column indices are 0-based.
- `get_*` returns `StorageError::ColumnOutOfRange` if `idx` exceeds
  `column_count()`, and `StorageError::TypeMismatch` if the stored type does
  not match the requested accessor.
- `is_null` is infallible and does not return `Result`.

### `StorageError`

```rust
// StorageError — all errors from StorageBackend operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("connection failed: {reason}")]
    Connection { reason: String, code: ErrorCode },
    #[error("query failed: {reason}")]
    Query { reason: String, code: ErrorCode },
    #[error("migration failed at version {version}: {reason}")]
    Migration { version: u32, reason: String, code: ErrorCode },
    #[error("serialization error: {reason}")]
    Serialization { reason: String, code: ErrorCode },
    #[error("transaction aborted: {reason}")]
    TransactionAborted { reason: String, code: ErrorCode },
    #[error("column index {idx} out of range")]
    ColumnOutOfRange { idx: usize },
    #[error("type mismatch at column {idx}: expected {expected}, got {actual}")]
    TypeMismatch { idx: usize, expected: &'static str, actual: &'static str },
}
```

#### Stable Error Codes

All variants that carry `ErrorCode` use codes from the `SC_RUNTIME.DB.*`
namespace. The stable codes are:

| Code | Variant | Meaning |
|------|---------|---------|
| `SC_RUNTIME.DB.CONNECTION_FAILED` | `Connection` | Backend could not open or acquire a connection |
| `SC_RUNTIME.DB.QUERY_FAILED` | `Query` | SQL execution error (syntax, constraint, etc.) |
| `SC_RUNTIME.DB.MIGRATION_FAILED` | `Migration` | Schema migration could not be applied |
| `SC_RUNTIME.DB.TRANSACTION_ABORTED` | `TransactionAborted` | Transaction rolled back (conflict, deadlock, etc.) |

`ColumnOutOfRange` and `TypeMismatch` carry no `ErrorCode` — they indicate
caller programming errors and are not expected to be handled by code paths
that inspect `.code`. `Serialization` is reserved for backends that
serialize structured data (e.g., JSON columns).

**Consumers must match on `.code` for programmatic error handling, not on
`.reason` strings.** Reason strings are human-readable and may change between
patch releases.

---

## Related ADRs

| ADR | Decision |
|-----|----------|
| ADR-03 | StorageBackend sealed trait — workspace-owned impls |
| ADR-04 | Five separate db crates (trait + one per impl) |
| ADR-09 | FrankenSQLite deferred to native mode stability |
