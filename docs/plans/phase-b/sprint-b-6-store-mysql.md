---
id: b-6
phase: b
status: planned
closure_type: boundary
target_boundary: store-mysql
track: stores
wave: 3
branch: sprint/b-6-store-mysql
worktree: sprint/b-6-store-mysql
requirements:
  - REQ-RUN-0321
adrs:
  - ADR-RUN-0301
  - ADR-RUN-0305
owned_paths:
  - template/crates/store-mysql/
must_follow: [b-4]
parallel_safe: [b-5]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: MySQL is the other sqlx SQL engine. Building it in the same wave as Postgres keeps the pairing from hardening around one remote engine.
---

# Sprint b-6 — store-mysql

## Goal

`store-mysql` is the sqlx MySQL store. It implements the same pairing `store-postgres` implements. Its SQL is MySQL SQL.

## Tasks

1. REQ-RUN-0321: MySQL queries live in `template/crates/store-mysql`.
2. ADR-RUN-0301: the crate depends on sqlx with the `mysql` feature. `sqlx.toml` names `MYSQL_DATABASE_URL`. Migrations live in the crate and run with `sqlx::migrate!`.
3. The crate exposes `open(cfg) -> Result<Store>` and the trait from `b-4`.

## Required validation

- A generated project that includes `store-mysql` compiles.
- `rg` of Postgres SQL and SQLite SQL inside `template/crates/store-mysql` prints nothing.

## Non-goals

No Postgres crate edits. No adaptor. No answers-schema edit. SQL Server is not a store.
