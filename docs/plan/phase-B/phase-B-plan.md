# Phase B: Storage

**Version target:** 0.1.0-alpha.2
**Entry state:** Phase A complete (v0.1.0-alpha.1 tagged)
**Exit state:** Both storage backends working, consumer can swap backends by changing one argument

Related: [Project Plan](../project-plan.md) · [Phase A](../phase-A/phase-A-plan.md) · [PRD §14 Phase B](../../prd/sc-runtime-prd.md)

---

## Objectives

By the end of Phase B, a consumer can add `.db(Arc::new(SqliteBackend::open("tool.db")?))` to their builder and get a WAL-mode SQLite database with connection pooling and an idempotent migration runner. The same consumer can swap to `SqlxBackend::from_url("postgres://...")` by changing a single constructor call, with no other changes required in their binary.

Specific outcomes:

- `StorageBackend` is a sealed trait. External crates can hold `Arc<dyn StorageBackend>` but cannot implement it, preserving the workspace's ability to evolve the trait without a public semver break (FR-05, ADR-03)
- `SC_RUNTIME_DB` environment variable overrides the database path or URL on both backends, enabling test isolation without config changes (FR-06)
- `sc-runtime-db-fsqlite` directory and boundary file are created and reserved; there is no implementation code, only a `FUTURE` status annotation making the intent explicit
- All Phase A exit criteria continue to pass

---

## Crates Delivered

| Crate | Status at end of phase | Complete | Deferred to |
|-------|------------------------|----------|-------------|
| `sc-runtime-db` | v0.1.0-alpha.2 | `StorageBackend` sealed trait, `Migration`, `SqlParam`, `Rows`, `StorageError`, `StorageHealth` | – |
| `sc-runtime-db-sqlite` | v0.1.0-alpha.2 | `SqliteBackend::open()`, `SqliteBackend::in_memory()`, rusqlite bundled feature, WAL journal mode, r2d2 connection pool, migration runner, `SC_RUNTIME_DB` override, full test suite | – |
| `sc-runtime-db-sqlx` | v0.1.0-alpha.2 | `SqlxBackend::from_url()`, `sqlx::AnyPool`, `sqlite://` and `postgres://` URL routing, `sqlx::migrate!` migration runner, `SC_RUNTIME_DB` URL override, cross-backend parity tests | – |
| `sc-runtime-db-fsqlite` | reserved | `Boundary.toml` only, crate directory created, `lib.rs` with `// FUTURE: FrankenSQLite native MVCC mode — see ADR-09` comment, no implementation | Phase F (post-0.1.0, when FrankenSQLite native mode ships) |
| `sc-runtime` | v0.1.0-alpha.2 | `.db(Arc<dyn StorageBackend>)` added to `ScRuntimeBuilder<HasCli>` and `ScRuntimeBuilder<HasDaemon>` (HasDaemon stub for Phase C) | – |
| `sc-runtime-example` | v0.1.0-alpha.2 | Updated to exercise `.db(Arc::new(SqliteBackend::in_memory()?))`, verify migration runs, verify a simple read/write round-trip | – |

---

## StorageBackend Design Notes

The sealed trait pattern (RBP-003) is implemented by placing the sealing supertrait in a private module within `sc-runtime-db`:

```rust
// sc-runtime-db/src/sealed.rs  (not pub)
pub trait StorageBackendSealed {}

// sc-runtime-db/src/lib.rs
pub trait StorageBackend: sealed::StorageBackendSealed + Send + Sync {
    fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<u64, StorageError>;
    fn query(&self, sql: &str, params: &[SqlParam]) -> Result<Rows, StorageError>;
    fn transaction<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&dyn StorageBackend) -> Result<R, StorageError>;
    fn migrate(&self, migrations: &[Migration]) -> Result<(), StorageError>;
    fn health(&self) -> StorageHealth;
}
```

Only crates inside the `sc-runtime` workspace can implement `StorageBackendSealed`, which is not re-exported. External crates can hold `Arc<dyn StorageBackend>` but the compiler rejects any attempt to implement the trait outside the workspace.

The trait is also object-safe by design (RBP-008): the `transaction` method uses a `dyn StorageBackend` parameter rather than `Self`, and there are no generic methods on the trait. Object-safety is verified by a `static_assertions::assert_obj_safe!(StorageBackend)` call in the crate test suite.

---

## PluginContext Storage Access

When storage is not configured, `PluginContext` must not expose an `Option<Arc<dyn StorageBackend>>` that plugins need to unwrap. The design uses a sealed access pattern: `PluginContext` carries storage internally, and a companion `PluginContextWithStorage` wrapper provides the `storage()` method. Plugins that require storage declare `fn init(&mut self, ctx: &PluginContextWithStorage)` at the type level. This is a compile-time guarantee, not a runtime check.

The exact mechanism is deferred to the implementation sprint; the key constraint is that a plugin cannot accidentally ignore a missing storage dependency — the compiler enforces it.

---

## Implementation Sequence

### Sprint B-1: sc-runtime-db

**Goal:** Sealed trait, supporting types, and boundary file established. No implementation yet.

Tasks:
- Create `crates/sc-runtime-db/` with sealed supertrait pattern in `src/sealed.rs`
- Implement `StorageBackend` trait with all five methods
- Implement `SqlParam` enum: `Text(String)`, `Integer(i64)`, `Real(f64)`, `Blob(Vec<u8>)`, `Null`
- Implement `Rows`: ordered list of rows, each row a `Vec<SqlParam>`
- Implement `Migration { version: u32, description: &'static str, up: &'static str, down: &'static str }`
- Implement `StorageError { code: ErrorCode, message: &'static str, cause: Option<Box<dyn Error + Send + Sync>> }` — consistent with `ScRuntimeError` RBP-001 pattern
- Implement `StorageHealth { state: StorageState, pool_size: u32, active_connections: u32 }`
- Write `static_assertions::assert_obj_safe!(StorageBackend)` test
- Update `boundaries/sc-runtime-db/Boundary.toml`: no dependency on any `sc-runtime-db-*` implementation crate
- Run `sc-lint lint fast`: boundary gate must pass on the new crate

Deliverable: `sc-runtime-db` compiles with sealed trait. Boundary gate passes.

### Sprint B-2: sc-runtime-db-sqlite

**Goal:** Fully working rusqlite backend with WAL, connection pool, migration runner, and `SC_RUNTIME_DB` override. Zero system SQLite dependency.

Tasks:
- Add `rusqlite = { version = "0.31", features = ["bundled"] }` and `r2d2-sqlite` to `sc-runtime-db-sqlite/Cargo.toml` (NF-04)
- Implement `SqliteBackend { pool: r2d2::Pool<SqliteConnectionManager> }`
- `SqliteBackend::open(path)`: opens file-backed SQLite; sets `PRAGMA journal_mode=WAL` on every connection via `r2d2` initialization hook; reads `SC_RUNTIME_DB` env var to override the path
- `SqliteBackend::in_memory()`: opens `:memory:` backend; reads `SC_RUNTIME_DB`; if the override is a `file::memory:?cache=shared` URL, uses that instead
- Implement `StorageBackend` for `SqliteBackend`: execute, query, transaction, migrate, health
- Migration runner: stores applied migrations in a `_sc_migrations` table; `migrate()` applies only unapproved versions in ascending order; re-running is idempotent
- Implement `StorageBackendSealed` for `SqliteBackend` (required by sealed trait)
- Tests:
  - WAL mode confirmed: after `open()`, `PRAGMA journal_mode` returns `wal`
  - Migration idempotency: run same migration set twice, assert table state unchanged
  - Two concurrent readers via `in_memory()` with `file::memory:?cache=shared`
  - `SC_RUNTIME_DB` override: set env var before `open()`, assert different path used
  - `execute` and `query` round-trip with all `SqlParam` variants
- Update `boundaries/sc-runtime-db-sqlite/Boundary.toml`: only `sc-runtime-db`; must not appear in `sc-runtime-db-sqlx` dependency tree

Deliverable: `sc-runtime-db-sqlite` fully implemented. All tests pass. WAL and migration idempotency confirmed.

### Sprint B-3: sc-runtime-db-sqlx

**Goal:** sqlx backend supporting `sqlite://` and `postgres://` via URL routing. Consumer can swap from rusqlite to postgres by changing a single constructor argument.

Tasks:
- Add `sqlx = { version = "0.8", features = ["any", "sqlite", "postgres", "runtime-tokio-rustls", "migrate"] }` to `sc-runtime-db-sqlx/Cargo.toml`
- Implement `SqlxBackend { pool: AnyPool }`
- `SqlxBackend::from_url(url: &str)`: routes on URL scheme; reads `SC_RUNTIME_DB` env var to override the URL entirely; returns `StorageError` with remediation if scheme is unrecognized
- Implement `StorageBackend` for `SqlxBackend`: execute, query, transaction, migrate via `sqlx::migrate!`, health via pool state
- `SqlParam` ↔ `sqlx::types::AnyValue` conversion layer
- Cross-backend parity tests: run identical migration fixtures against both `SqliteBackend::in_memory()` and `SqlxBackend::from_url("sqlite::memory:")`, assert identical row content returned by `query()`
- Postgres integration tests: gated behind a `POSTGRES_TEST_URL` env var; skipped in `fast` profile, run in `full` profile; use `testcontainers` if available, otherwise require a caller-provided URL
- `SC_RUNTIME_DB` override: set to a `sqlite::memory:` URL, assert `SqlxBackend::from_url("postgres://unreachable")` uses the override instead
- Update `boundaries/sc-runtime-db-sqlx/Boundary.toml`: only `sc-runtime-db`; explicitly excludes `sc-runtime-db-sqlite`

Deliverable: sqlx backend works. `sqlite://` and `postgres://` paths validated. Cross-backend parity confirmed.

### Sprint B-4: Facade `.db()` + Consumer Integration Test Updated

**Goal:** `.db()` wired into the builder. `sc-runtime-example` exercises storage. `PluginContext` storage access pattern established.

Tasks:
- Add `.db(Arc<dyn StorageBackend>) -> Self` to `ScRuntimeBuilder<HasCli>` (and stub it for `HasDaemon` for Phase C)
- Wire the backend into `ScRuntime`: when `.build()` is called with a backend, the runtime stores `Arc<dyn StorageBackend>` and passes it to `PluginContext` during plugin init
- Implement the `PluginContextWithStorage` companion (or whichever access pattern is chosen in B-1 design review) so storage-requiring plugins declare their dependency at compile time
- Update `sc-runtime-example`: add a `StoragePlugin` that requires storage, call `.db(Arc::new(SqliteBackend::in_memory()?))`, verify the plugin's `init()` receives storage and can execute a simple `INSERT` + `SELECT` round-trip
- Integration test: run the example binary with storage configured, assert query results match inserted values; run without `.db()`, assert `StoragePlugin` produces a compile error (compile-fail test)
- Run `cargo test --workspace`; run `sc-lint lint fast`

Deliverable: Phase B complete. Tag `v0.1.0-alpha.2`.

---

## Exit Criteria

All of the following must be true before Phase C begins:

- `cargo test --workspace` passes, including all storage tests; postgres integration tests are either passing (if a test DB is available) or clearly marked as skipped with `#[ignore = "requires POSTGRES_TEST_URL"]`
- `sc-lint lint fast` passes: boundary rules for all four storage crates are clean; `sc-runtime-db` carries no implementation dependency; `sc-runtime-db-sqlite` and `sc-runtime-db-sqlx` are independent of each other
- `SqliteBackend`: WAL mode confirmed in test, migration idempotency confirmed across at least three migration versions, `SC_RUNTIME_DB` override tested
- `SqlxBackend`: `sqlite://` and `postgres://` backends produce identical query results for the same migration fixtures (cross-backend parity test)
- A consumer can switch from `SqliteBackend::open("tool.db")` to `SqlxBackend::from_url("postgres://localhost/tool")` by changing one line — this is demonstrated in the `sc-runtime-example` test via parameterized backend selection
- `sc-runtime-db-fsqlite` crate directory exists with `Boundary.toml` and stub `lib.rs` containing the FUTURE annotation; the crate compiles but exports nothing
- `ScRuntime::run()` still returns `Result` and does not panic; all Phase A exit criteria continue to pass
- Git tag `v0.1.0-alpha.2` applied to the passing commit
