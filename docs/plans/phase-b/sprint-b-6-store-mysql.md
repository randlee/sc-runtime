---
id: b-6
phase: b
status: planned
closure_type: boundary
target_boundary: store-mysql
track: stores
wave: 2
branch: sprint/b-6-store-mysql
worktree: sprint/b-6-store-mysql
requirements:
  - REQ-RUN-0321
adrs:
  - ADR-RUN-0301
  - ADR-RUN-0303
  - ADR-RUN-0305
owned_paths:
  - template/crates/store-mysql/
must_follow: [b-4]
parallel_safe: [b-5, b-7]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: MySQL is the other sqlx SQL engine. Building it in the same wave as Postgres keeps the pairing from hardening around one remote engine.
---

# Sprint b-6 — store-mysql

## Goal

`store-mysql` is the sqlx MySQL store. It implements `EngineStore` from `b-4`, the same trait `store-postgres` implements. Its SQL is MySQL SQL. It does not edit `template/Cargo.toml` or `template/crates/schema/`.

Checked-in `.sqlx` data is a deliverable. The compile proof runs with `SQLX_OFFLINE=true` and no live database.

## Deliverables

1. REQ-RUN-0321: MySQL queries live in `template/crates/store-mysql`.
2. ADR-RUN-0301: the crate depends on sqlx with the `mysql` feature. `sqlx.toml` names `MYSQL_DATABASE_URL`. Migrations live in the crate and run with `sqlx::migrate!`. Check in `template/crates/store-mysql/.sqlx/` from `just db-prepare`.
3. The crate implements `EngineStore::open` from the `b-4` sample. It does not declare a second trait.

## Acceptance criteria

Root: `boundary:store-mysql`.

- `SQLX_OFFLINE=true cargo build -p store-mysql` exits 0 with no live database.
- `git ls-files template/crates/store-mysql/.sqlx` lists at least one file.
- `rg` of Postgres SQL and SQLite SQL inside `template/crates/store-mysql` prints nothing.

## Required validation

- `SQLX_OFFLINE=true cargo test -p store-mysql` exits 0.
- `SQLX_OFFLINE=true cargo build -p store-mysql` exits 0.

## Non-goals

No Postgres crate edits. No adaptor. No answers-schema edit. No `just new` of a generated project (`b-8`). SQL Server is not a store.
