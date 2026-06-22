# sc-runtime-db-sqlx Requirements

This document defines the requirements for `sc-runtime-db-sqlx`, the sqlx
implementation of `StorageBackend` supporting SQLite and PostgreSQL.

Related documents:
- [docs/sc-runtime-db-sqlx/architecture.md](./architecture.md)
- [docs/sc-runtime-db/requirements.md](../sc-runtime-db/requirements.md)
- [docs/prd/sc-runtime-prd.md](../prd/sc-runtime-prd.md) — §6.7

---

## Scope

`sc-runtime-db-sqlx` is the sqlx implementation of `StorageBackend`. It
provides runtime backend selection between SQLite and PostgreSQL via connection
URL. It does not define the `StorageBackend` trait or the `Migration` type —
those belong to `sc-runtime-db`.

---

## Functional Requirements

### FR-SQLX-01 — SQLite and PostgreSQL URL scheme support

`SqlxBackend::from_url()` must accept and correctly route both URL schemes:

| URL scheme | Backend |
|------------|---------|
| `sqlite://` | SQLite via sqlx AnyPool |
| `postgres://` or `postgresql://` | PostgreSQL via sqlx AnyPool |

Passing an unsupported URL scheme must return a `StorageError` with a
descriptive message.

### FR-SQLX-02 — AnyPool for runtime backend selection

The implementation must use `sqlx::AnyPool`, not a typed `Pool<Sqlite>` or
`Pool<Postgres>`. The backend must be selected at runtime from the URL — no
recompile required to switch from SQLite to PostgreSQL.

This requirement exists to enable the consumer switchover pattern: a consumer
changes the URL string in their builder configuration and nothing else. No code
changes, no recompilation with different features.

### FR-SQLX-03 — SC_RUNTIME_DB environment variable override

`SqlxBackend::from_url()` must check the `SC_RUNTIME_DB` environment variable
at construction time. If set, its value is used as the connection URL instead
of the caller-supplied argument.

The override is treated as a full connection URL and may reference a different
backend than the original argument (e.g., `SC_RUNTIME_DB=sqlite://:memory:`
overrides a `postgres://` URL in test environments).

The variable is read once at construction time.

### FR-SQLX-04 — Migration runner: ordered and idempotent

The `migrate()` implementation must use sqlx's built-in migration runner
(`sqlx::migrate!`). Migrations must be applied in version-ascending order.
Re-running migrations on a database where they are already applied must be a
no-op (idempotent).

Migration SQL must use a DDL subset compatible with both SQLite and PostgreSQL
backends.

### FR-SQLX-05 — Consumer backend switchover with URL change only

A consumer using `sc-runtime-db-sqlx` must be able to switch from a
`sqlite://` URL to a `postgres://` URL by changing only the URL string passed
to `SqlxBackend::from_url()`. No other code changes — not in migration
definitions, not in query code, not in plugin implementations — must be
required.

This is the primary value proposition of this crate over `sc-runtime-db-sqlite`.

### FR-SQLX-06 — Full StorageBackend implementation

`SqlxBackend` must implement all methods of `StorageBackend`:
`execute`, `query`, `transaction`, `migrate`, and `health`.

`health()` must return `StorageHealth::Healthy` when a ping query succeeds and
`StorageHealth::Degraded` (with a descriptive reason string) otherwise.
`health()` must never panic or return an error type.

---

## Boundary Requirements

### BR-SQLX-01 — Permitted dependencies

`sc-runtime-db-sqlx` may depend on:
- `sc-runtime-db` (workspace) — required for the `StorageBackend` trait
- `sqlx` (0.8, features: any, sqlite, postgres, runtime-tokio-rustls, migrate)

It must not depend on `sc-runtime-db-sqlite`, `sc-runtime-db-fsqlite`,
`rusqlite`, `r2d2-sqlite`, or any other backend implementation crate.

### BR-SQLX-02 — No direct sc-runtime-core dependency

`sc-runtime-db-sqlx` receives error and health types through `sc-runtime-db`.
It must not take a direct dependency on `sc-runtime-core` unless a specific
type from that crate is required by the implementation.

---

## Non-Functional Requirements

### NF-SQLX-01 — tokio runtime compatibility

`sc-runtime-db-sqlx` requires the `runtime-tokio-rustls` sqlx feature. All
async operations must be compatible with the tokio runtime. The crate must not
use the `async-std` runtime or any other async executor.

### NF-SQLX-02 — No unsafe code

`sc-runtime-db-sqlx` must compile with `#![forbid(unsafe_code)]`. sqlx
internally manages its FFI to SQLite and the PostgreSQL wire protocol; this
crate's Rust code has no reason to use unsafe.

### NF-SQLX-03 — Edition and toolchain

Edition 2024, `rust-version = 1.94.1`, consistent with the workspace
`rust-toolchain.toml`.

### NF-SQLX-04 — Version

`0.1.0`, consistent with all sc-runtime workspace crates.

### NF-SQLX-05 — No compile-time query! macros

`sc-runtime-db-sqlx` must not use sqlx's `query!` macros in its
`StorageBackend` implementation. `query!` macros require a specific backend at
compile time and are incompatible with the `AnyPool` runtime backend selection
model. Runtime string queries must be used instead. SQL correctness is
validated by the test suite against both SQLite and PostgreSQL paths.
