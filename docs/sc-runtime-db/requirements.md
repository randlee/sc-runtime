# sc-runtime-db Requirements

This document defines the requirements for `sc-runtime-db`, the storage
abstraction crate for the sc-runtime workspace.

Related documents:
- [docs/sc-runtime-db/architecture.md](./architecture.md)
- [docs/prd/sc-runtime-prd.md](../prd/sc-runtime-prd.md) — §6.4, FR-05, FR-06

---

## Scope

`sc-runtime-db` defines the `StorageBackend` trait and supporting types. It
contains no SQL, no I/O, and no implementation code. All implementation logic
lives in the backend crates (`sc-runtime-db-sqlite`, `sc-runtime-db-sqlx`,
`sc-runtime-db-fsqlite`).

Any future storage-related type or interface that is shared across backends
belongs here — not in any individual backend crate.

---

## Functional Requirements

### FR-DB-01 — Object-safe trait

`StorageBackend` must be object-safe. `Arc<dyn StorageBackend>` must be a
valid Rust type. The compiler must be able to construct a vtable for it.

Methods with generic parameters that prevent object safety must be redesigned
(e.g., using boxed closures) before the trait is published.

### FR-DB-02 — Sealed trait

`StorageBackend` must be sealed. External crates must not be able to implement
`StorageBackend` for their own types. The sealing mechanism is a
workspace-private marker trait in a `private` module.

External crates may hold `Arc<dyn StorageBackend>` and call its methods.
External crates may not implement the trait.

This requirement derives from RBP-003 (Sealed Trait) and ADR-03.

### FR-DB-03 — Migration type and transaction signature

The `Migration` type must carry:

| Field | Type | Purpose |
|-------|------|---------|
| `version` | `u32` | Monotonically increasing schema version identifier |
| `description` | `&'static str` | Human-readable description of the migration |
| `up` | `&'static str` | Forward migration SQL (DDL) |
| `down` | `&'static str` | Rollback migration SQL (DDL) |

`up` and `down` must use a SQL DDL subset compatible with all supported
backends. Vendor-specific extensions are not permitted in migration definitions.

**`transaction` method signature (object-safe).** The `transaction` method
on `StorageBackend` must use the following signature, which is object-safe
and compatible with `Arc<dyn StorageBackend>`:

```rust
fn transaction(
    &self,
    f: Box<dyn FnOnce(&dyn StorageBackend) -> Result<(), StorageError> + Send>,
) -> Result<(), StorageError>;
```

The closure returns `()`. Callers that need to extract a value from inside
the transaction must capture it via a mutable variable in the enclosing
scope. See `sc-runtime-db/architecture.md` § "Extracting Values from a
Transaction" for the canonical example.

**`migrate` parameter contract.** The `migrate(&[Migration])` parameter is
the canonical migration descriptor slice consumed by backends that execute
SQL from `Migration::up` directly (e.g., `sc-runtime-db-sqlite`). Backends
that manage their own migration mechanism (e.g., `sc-runtime-db-sqlx` using
`sqlx::migrate!`) may ignore the parameter; the `migrations_dir` argument
supplied to their constructor is their migration source. This divergence is a
known limitation documented in the per-backend architecture files.

### FR-DB-04 — SC_RUNTIME_DB environment variable contract

The `SC_RUNTIME_DB` contract must be documented in this crate (the trait
crate), not left to individual implementations.

Contract:
- When `SC_RUNTIME_DB` is set, backend implementations must use its value as
  the database path or connection URL, overriding any caller-supplied argument.
- When `SC_RUNTIME_DB` is not set, implementations use the caller-supplied
  argument.
- The variable is read once at construction time, not on every query.

Rationale: test isolation. A test process sets `SC_RUNTIME_DB` to a temporary
path; any backend construction in that process uses the override automatically,
enabling parallel test isolation without config propagation.

This is FR-06 from the PRD.

### FR-DB-05 — Infallible health method

`health()` must return `StorageHealth`, not `Result<StorageHealth, _>`.

An unhealthy backend is a valid, reportable state — not an error that prevents
the method from returning. The sc-runtime health subsystem aggregates
`StorageHealth` values from all configured backends; it must not have to handle
`Result` unwrapping at that aggregation point.

This requirement derives from RBP-007 (Infallible accessors).

### FR-DB-06 — Send + Sync bounds

`StorageBackend` must have `Send + Sync` supertraits. Backends are stored in
`Arc<dyn StorageBackend>` and shared across async tasks in the plugin
registry. The trait must be usable in a multithreaded async runtime.

### FR-DB-07 — StorageHealth must carry ErrorCode in non-Healthy variants

`StorageHealth::Degraded` and `StorageHealth::Unavailable` must each carry an
`ErrorCode` field in addition to the human-readable `reason: String`. Consumers
must be able to branch on a stable, machine-readable code without string parsing.

`StorageHealth::Healthy` carries no payload.

### FR-DB-08 — SqlParam must cover the full common type set

`SqlParam` must have variants covering:

| Variant | Rust type |
|---------|-----------|
| `Null` | — |
| `Integer` | `i64` |
| `Real` | `f64` |
| `Text` | `&str` (borrowed) |
| `Blob` | `&[u8]` (borrowed) |
| `Bool` | `bool` |

No additional variants may be added without a minor-version bump. Backends
must map all six to their native parameter binding equivalents.

### FR-DB-09 — Rows::next_row() must return Option\<Row\>, not Result\<Option\<Row\>\>

Row exhaustion is a normal condition, not an error. `next_row()` returns
`Option<Row>`:

- `Some(Row)` — a row is available
- `None` — the result set is exhausted

An error during row materialization must be surfaced by the `Row::get_*`
methods on the row that was returned, not by `next_row()`.

### FR-DB-10 — StorageError variants must carry ErrorCode for programmatic handling

Every `StorageError` variant that represents a backend-originated failure must
carry a `code: ErrorCode` field. The stable codes defined in the
`SC_RUNTIME.DB.*` namespace are:

| Code | Variant |
|------|---------|
| `SC_RUNTIME.DB.CONNECTION_FAILED` | `Connection` |
| `SC_RUNTIME.DB.QUERY_FAILED` | `Query` |
| `SC_RUNTIME.DB.MIGRATION_FAILED` | `Migration` |
| `SC_RUNTIME.DB.TRANSACTION_ABORTED` | `TransactionAborted` |

Caller-error variants (`ColumnOutOfRange`, `TypeMismatch`) are exempt — they
indicate programming errors and do not need stable codes.

---

## Boundary Requirements

### BR-DB-01 — No impl crate dependencies

`sc-runtime-db` must not depend on any backend implementation crate. Its
dependency tree must not include `rusqlite`, `r2d2-sqlite`, `sqlx`, or any
FrankenSQLite crate.

This boundary is enforced by the `sc-lint-boundary` boundary definition in
`boundaries/sc-runtime-db/`. Violation is a CI gate failure.

### BR-DB-02 — Permitted dependencies

The only permitted workspace dependency is `sc-runtime-core`.

### Dependency Boundary Rules

| Category | Rule |
|----------|------|
| Permitted workspace dependencies | `sc-runtime-core` only |
| Permitted external dependencies | `thiserror`, `futures` (for object-safe async in future) |
| Forbidden | `sc-runtime-cli`, `sc-runtime-daemon`, `sc-runtime-web`, `sc-runtime-db-*`; `rusqlite`, `r2d2-sqlite`, `sqlx`, any FrankenSQLite crate; any SC domain crate |
| Boundary file | `boundaries/sc-runtime-db/Boundary.toml` |

Note: `sc-runtime-db` defines the trait only — no SQL drivers, no I/O. Keeping
it clean is critical for the sealed-trait guarantee. The daemon depends on the
trait (`Arc<dyn StorageBackend>`), not on any impl crate; backend selection is
deferred to the consumer.

---

## Non-Functional Requirements

### NF-DB-01 — No unsafe code

`sc-runtime-db` must compile with `#![forbid(unsafe_code)]`. This crate
contains only trait and type definitions; there is no reason for unsafe code.

### NF-DB-02 — No I/O

This crate must not perform file I/O, network I/O, or any operating-system
call. It is a pure Rust type definition crate.

### NF-DB-03 — Edition and toolchain

Edition 2024, `rust-version = 1.94.1`, consistent with the workspace
`rust-toolchain.toml`.

### NF-DB-04 — Version

`0.1.0`, consistent with all sc-runtime workspace crates.
