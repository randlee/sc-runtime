# sc-runtime-db-sqlx Architecture

This document defines the architecture of `sc-runtime-db-sqlx`, the sqlx
implementation of `StorageBackend` supporting SQLite and PostgreSQL via
connection URL.

Related documents:
- [docs/sc-runtime-db-sqlx/requirements.md](./requirements.md)
- [docs/sc-runtime-db/architecture.md](../sc-runtime-db/architecture.md)
- [docs/sc-runtime-db-sqlite/architecture.md](../sc-runtime-db-sqlite/architecture.md)
- [docs/prd/sc-runtime-prd.md](../prd/sc-runtime-prd.md) — §6.7

---

## Role

`sc-runtime-db-sqlx` implements `StorageBackend` using sqlx with `AnyPool`,
supporting SQLite and PostgreSQL from a single crate. It is the path to
PostgreSQL for SC tools that need a server-backed database, and the migration
path for tools that start on SQLite and need to move to PostgreSQL without
code changes.

---

## Why sqlx Here and Not in sc-runtime-db-sqlite

`sc-runtime-db-sqlite` and `sc-runtime-db-sqlx` serve different consumer
needs:

| | `sc-runtime-db-sqlite` | `sc-runtime-db-sqlx` |
|---|---|---|
| Backend | SQLite only | SQLite or PostgreSQL (URL-selected) |
| Library | rusqlite (synchronous, bundled) | sqlx (async, AnyPool) |
| Runtime requirement | none | tokio |
| Build dependency | bundled SQLite C source | none for SQLite; libpq or network for Postgres |
| Best for | single-process tools, offline, zero config | tools needing Postgres or backend flexibility |

`sc-runtime-db-sqlite` is simpler and faster for SQLite-only tools. sqlx adds
async overhead and tokio coupling that single-process tools do not need.
`sc-runtime-db-sqlx` is the right choice when PostgreSQL support or
runtime-selectable backends are a requirement.

---

## AnyPool for Runtime Backend Selection

`sc-runtime-db-sqlx` uses `sqlx::AnyPool` rather than a typed
`Pool<Sqlite>` or `Pool<Postgres>`. This is a deliberate design choice.

`AnyPool` routes queries to the backend selected by the connection URL at
runtime. The consumer does not recompile to switch backends — they change the
URL string.

```rust
// SQLite
let backend = SqlxBackend::new("sqlite://./my-tool.db", "migrations")?;

// PostgreSQL — same type, same migrations_dir, zero other changes
let backend = SqlxBackend::new("postgres://localhost/my-tool", "migrations")?;
```

Both calls produce an `Arc<dyn StorageBackend>`. The downstream code is
identical.

AnyPool configuration defaults:
- max_connections: 5
- min_connections: 0
- connect_timeout: 5 seconds
- idle_timeout: 600 seconds (10 minutes)
- max_lifetime: 1800 seconds (30 minutes)
- acquire_timeout: 5 seconds (time to wait for a connection from the pool)

### AnyPool vs. query! macros

sqlx's `query!` macros perform compile-time SQL verification against a
specific backend. They require a `DATABASE_URL` at compile time and cannot
work with `AnyPool` because `AnyPool` selects the backend at runtime.

`sc-runtime-db-sqlx` uses runtime string queries (`query()`, `execute()`)
rather than `query!` macros. This is an intentional trade-off:

- `query!` macros would require knowing the backend at compile time, defeating
  the purpose of `AnyPool`.
- Runtime string queries require the SQL to be correct by testing rather than
  by compile-time verification.
- The migration SQL is validated by the test suite against both SQLite and
  PostgreSQL paths.

---

## URL Scheme Routing

`SqlxBackend::new()` parses the scheme prefix of the connection URL to
select the backend pool:

| URL scheme | Backend |
|------------|---------|
| `sqlite://` | `AnyPool` backed by SQLite |
| `postgres://` | `AnyPool` backed by PostgreSQL |
| `postgresql://` | `AnyPool` backed by PostgreSQL (alias) |

The URL is passed to `sqlx::AnyPool::connect()`. The `any` feature of sqlx
enables the runtime backend dispatch.

```
"sqlite://./my-tool.db"      → sqlx selects SQLite driver
"postgres://host/db"         → sqlx selects PostgreSQL driver
```

If the URL scheme is not recognized, `SqlxBackend::new()` returns a `StorageError`.

---

## SC_RUNTIME_DB Override

`SqlxBackend::new()` checks `SC_RUNTIME_DB` before using its URL argument:

```rust
let effective_url = std::env::var("SC_RUNTIME_DB")
    .unwrap_or_else(|_| url.to_owned());
```

The override value is treated as a full connection URL — it may be a different
scheme than the original argument. Setting `SC_RUNTIME_DB=sqlite://:memory:`
in a test environment redirects all `SqlxBackend` constructions to an
in-memory SQLite database regardless of what URL the application code passes.

The `migrations_dir` argument is not affected by `SC_RUNTIME_DB`; only the
connection URL is overridden.

---

## Migration Runner

`sc-runtime-db-sqlx` uses `sqlx::migrate!` for migration execution.

The `migrate!` macro is a compile-time macro that embeds `.sql` migration
files from a directory into the binary, producing a `Migrator` struct. The
`Migrator` is stored in `SqlxBackend` at construction time and used to apply
unapplied migrations at runtime.

### Constructor: `migrations_dir` Argument

`SqlxBackend::new(url, migrations_dir)` takes an additional `migrations_dir:
&str` argument pointing to the directory containing numbered `.sql` migration
files. This directory is passed to `sqlx::migrate!` at build time:

```rust
// Construction — migrations_dir points to a directory of .sql files
let backend = SqlxBackend::new(
    "sqlite://./my-tool.db",
    "migrations",            // directory containing 0001_init.sql, etc.
)?;
```

The embedded `Migrator` is stored in the `SqlxBackend` struct at construction
and used whenever `StorageBackend::migrate()` is called.

### `migrate(&[Migration])` Parameter Is Ignored

`StorageBackend::migrate(&self, migrations: &[Migration])` accepts a
`&[Migration]` slice as defined in the trait. On the sqlx backend, **this
parameter is unused**. The migration source is the `Migrator` embedded at
compile time via `sqlx::migrate!(migrations_dir)`. Passing an empty slice or
any slice to `migrate()` has no effect on which migrations are applied.

This means sqlx consumers must organise their migrations as numbered `.sql`
files in the directory passed to `SqlxBackend::new()`. They must not rely on
the `Migration` structs passed to `migrate()` to reach the sqlx backend.

At runtime, `StorageBackend::migrate()` on `SqlxBackend` is equivalent to:

```rust
self.migrator.run(&self.pool).await?;
```

### Idempotency

sqlx's migration runner is idempotent. Running `migrate().run()` on a database
where all migrations are already applied is a no-op. This makes it safe to
call at every application startup.

### Cross-backend SQL Compatibility

Migration SQL must use the DDL subset supported by both SQLite and PostgreSQL.
Vendor-specific syntax (e.g., PostgreSQL sequences without a fallback, or
SQLite-specific `WITHOUT ROWID`) must not appear in shared migration files.
Backend-specific migrations, if needed, are handled with separate files
selected at build time via feature flags.

### Known Limitation: Migration Tracking Table Name Mismatch

`sc-runtime-db-sqlite` tracks applied migrations in a `_sc_migrations` table.
`sc-runtime-db-sqlx` uses sqlx's built-in tracking table `_sqlx_migrations`.
These are incompatible schemas.

Consequence: a database file written by one backend cannot be transparently
handed to the other — the migration state recorded in one table is invisible
to the other backend. Switching backends requires manual migration state
reconciliation.

This is a known limitation. A cross-backend migration reconciliation tool is
planned for Phase E and will be documented in `MIGRATION.md` when implemented.

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `sc-runtime-db` | workspace | `StorageBackend` trait, `StorageError` |
| `sqlx` | 0.8 | `AnyPool`, `migrate!`, query execution |

sqlx features required:

```toml
[dependencies]
sqlx = { version = "0.8", features = [
    "any",             # AnyPool runtime dispatch
    "sqlite",          # SQLite driver
    "postgres",        # PostgreSQL driver
    "runtime-tokio-rustls",  # tokio async runtime + TLS
    "migrate",         # migration runner
] }
```

No rusqlite, r2d2-sqlite, or other backend crates appear in this crate's
dependency tree.

---

## Consumer Switchover Pattern

A consumer migrating from SQLite to PostgreSQL changes only the URL argument:

```rust
// SQLite (development / single-user)
ScRuntime::builder()
    .name("my-tool")
    .cli(commands::register)
    .db(Arc::new(SqlxBackend::new("sqlite://./my-tool.db", "migrations")?))
    .logger(logger)
    .build()
    .run()?;

// PostgreSQL (production / multi-user) — only the URL argument changes
ScRuntime::builder()
    .name("my-tool")
    .cli(commands::register)
    .db(Arc::new(SqlxBackend::new("postgres://db.host/my-tool", "migrations")?))
    .logger(logger)
    .build()
    .run()?;
```

All plugin code, migration definitions, and query code are unchanged.

---

## Crate Position

```
sc-runtime-db           ← StorageBackend trait
    ↑
sc-runtime-db-sqlx      ← this crate — sqlx AnyPool, SQLite + PostgreSQL
sc-runtime-db-sqlite    ← rusqlite (separate crate, no cross-dependency)
```

`sc-runtime-db-sqlx` and `sc-runtime-db-sqlite` do not depend on each other.
A consumer selects one; the other does not appear in their dependency tree.
