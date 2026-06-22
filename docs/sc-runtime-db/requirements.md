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

### FR-DB-03 — Migration type

The `Migration` type must carry:

| Field | Type | Purpose |
|-------|------|---------|
| `version` | `u32` | Monotonically increasing schema version identifier |
| `description` | `&'static str` | Human-readable description of the migration |
| `up` | `&'static str` | Forward migration SQL (DDL) |
| `down` | `&'static str` | Rollback migration SQL (DDL) |

`up` and `down` must use a SQL DDL subset compatible with all supported
backends. Vendor-specific extensions are not permitted in migration definitions.

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
