---
id: b-5
phase: b
status: planned
closure_type: boundary
target_boundary: store-postgres
track: stores
wave: 3
branch: sprint/b-5-store-postgres
worktree: sprint/b-5-store-postgres
requirements:
  - REQ-RUN-0321
  - REQ-RUN-0308
adrs:
  - ADR-RUN-0301
  - ADR-RUN-0305
owned_paths:
  - template/crates/store-postgres/
must_follow: [b-4]
parallel_safe: [b-6]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One store crate. Its SQL and its trait implementation live in the same crate, and b-6 owns the other engine so the two can be written against the same pairing.
---

# Sprint b-5 — store-postgres

## Goal

`store-postgres` is the sqlx Postgres store. It implements the pairing from `b-4`. Its SQL is Postgres SQL. It does not contain SQLite SQL or MySQL SQL.

This sprint is in the same wave as `b-6`. The pairing was fixed in `b-4` so Postgres does not become a special case that MySQL then has to imitate.

## Tasks

1. REQ-RUN-0321: Postgres queries live in `template/crates/store-postgres`.
2. ADR-RUN-0301: the crate depends on sqlx with the `postgres` feature. `sqlx.toml` names `POSTGRES_DATABASE_URL`. Migrations live in the crate and run with `sqlx::migrate!`.
3. The crate exposes `open(cfg) -> Result<Store>` and the trait from `b-4`.

## Required validation

- A generated project that includes `store-postgres` compiles.
- `rg` of SQLite-only SQL inside `template/crates/store-postgres` prints nothing.

## Non-goals

No MySQL crate edits. No adaptor. No answers-schema edit (`b-8`). No rusqlite writer edit.
