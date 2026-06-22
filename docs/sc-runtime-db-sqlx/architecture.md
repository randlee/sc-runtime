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
let backend = SqlxBackend::from_url("sqlite://./my-tool.db")?;

// PostgreSQL — same type, zero other changes
let backend = SqlxBackend::from_url("postgres://localhost/my-tool")?;
```

Both calls produce an `Arc<dyn StorageBackend>`. The downstream code is
identical.

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

`SqlxBackend::from_url()` parses the scheme prefix of the connection URL to
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

If the URL scheme is not recognized, `from_url()` returns a `StorageError`.

---

## SC_RUNTIME_DB Override

`SqlxBackend::from_url()` checks `SC_RUNTIME_DB` before using its argument:

```rust
let effective_url = std::env::var("SC_RUNTIME_DB")
    .unwrap_or_else(|_| url.to_owned());
```

The override value is treated as a full connection URL — it may be a different
scheme than the original argument. Setting `SC_RUNTIME_DB=sqlite://:memory:`
in a test environment redirects all `SqlxBackend` constructions to an
in-memory SQLite database regardless of what URL the application code passes.

---

## Migration Runner

`sc-runtime-db-sqlx` uses `sqlx::migrate!` for migration execution.

The `migrate!` macro compiles SQL migration files from a directory into the
binary at build time. At runtime, `migrate().run(&pool)` applies any
unapplied migrations.

sqlx maintains its own migrations tracking table (`_sqlx_migrations`) with a
schema compatible with its built-in versioning. The version numbering scheme
used in `sc-runtime-db::Migration` (`u32`) maps directly to sqlx's integer
version field.

Migration files are stored in the crate under `migrations/`. They follow the
sqlx naming convention: `<version>_<description>.sql`.

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

A consumer migrating from SQLite to PostgreSQL changes one line:

```rust
// SQLite (development / single-user)
ScRuntime::builder()
    .name("my-tool")
    .cli(commands::register)
    .db(Arc::new(SqlxBackend::from_url("sqlite://./my-tool.db")?))
    .logger(logger)
    .build()
    .run()?;

// PostgreSQL (production / multi-user) — only the URL changes
ScRuntime::builder()
    .name("my-tool")
    .cli(commands::register)
    .db(Arc::new(SqlxBackend::from_url("postgres://db.host/my-tool")?))
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
