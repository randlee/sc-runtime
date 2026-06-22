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
    fn transaction<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&dyn StorageBackend) -> Result<R, StorageError>;
    fn migrate(&self, migrations: &[Migration]) -> Result<(), StorageError>;
    fn health(&self) -> StorageHealth;
}
```

### Method Contracts

| Method | Returns | Notes |
|--------|---------|-------|
| `execute` | affected row count | INSERT, UPDATE, DELETE, DDL |
| `query` | `Rows` — owned, cursor-like result set | SELECT |
| `transaction` | caller-supplied `R` | closure receives `&dyn StorageBackend` |
| `migrate` | `()` on success | ordered, idempotent — see Migration type below |
| `health` | `StorageHealth` — infallible | never `Result`; see RBP-007 note |

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
    pub up: &'static str,    // SQL DDL — standard subset compatible with all backends
    pub down: &'static str,
}
```

`version` is a monotonically increasing integer. The migration runner in each
backend implementation orders by `version` ascending, applies only migrations
whose version exceeds the current schema version, and records each applied
version in a `_sc_migrations` tracking table. Re-running `migrate()` with
already-applied migrations is a no-op (idempotent).

`up` and `down` SQL must use the common DDL subset — no vendor-specific syntax.
Backends are responsible for executing the SQL; the trait crate defines the
data carrier only.

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

1. **No generic methods with unconstrained type parameters.** The `transaction`
   closure uses `F: FnOnce(&dyn StorageBackend) -> Result<R, StorageError>`,
   which is not object-safe. To resolve this, `transaction` is a concrete
   method taking a boxed closure:

   ```rust
   fn transaction(
       &self,
       f: Box<dyn FnOnce(&dyn StorageBackend) -> Result<Box<dyn std::any::Any>, StorageError>>,
   ) -> Result<Box<dyn std::any::Any>, StorageError>;
   ```

   The exact signature is resolved during implementation. The constraint is:
   the method must be callable through a trait object.

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

## Related ADRs

| ADR | Decision |
|-----|----------|
| ADR-03 | StorageBackend sealed trait — workspace-owned impls |
| ADR-04 | Five separate db crates (trait + one per impl) |
| ADR-09 | FrankenSQLite deferred to native mode stability |
