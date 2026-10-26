---
id: b-5
phase: b
status: planned
closure_type: boundary
target_boundary: store-postgres
track: stores
wave: 2
branch: sprint/b-5-store-postgres
worktree: sprint/b-5-store-postgres
requirements:
  - REQ-RUN-0321
  - REQ-RUN-0328
  - REQ-RUN-0308
  - NFR-RUN-0002
adrs:
  - ADR-RUN-0301
  - ADR-RUN-0303
  - ADR-RUN-0305
  - ADR-RUN-0011
  - ADR-RUN-0007
owned_paths:
  - template/crates/store-postgres/
must_follow: [b-4]
parallel_safe: [b-6, b-7]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One store crate. Its SQL and its EngineStore implementation live in the same crate, and b-6 owns the other engine so the two can be written against the same pairing.
---

# Sprint b-5 — store-postgres

## Goal

`store-postgres` is the sqlx Postgres store. It implements `EngineStore` from `b-4`. Its SQL is Postgres SQL. It does not contain SQLite SQL or MySQL SQL. It does not edit `template/Cargo.toml` or `template/crates/schema/`.

REQ-RUN-0308's offline-data rule applies to this crate: checked-in `.sqlx` data, and the compile proof runs with `SQLX_OFFLINE=true` and no live database. The crate implements `EngineStore::open`, `read_rows`, and `write_rows`. It does not declare adaptor types. Cargo commands pass `--manifest-path template/Cargo.toml` and do not commit `template/Cargo.lock`.

## Deliverables

1. REQ-RUN-0321: Postgres queries live in `template/crates/store-postgres`.
2. ADR-RUN-0301 and REQ-RUN-0308: the crate depends on sqlx with the `postgres` feature. `sqlx.toml` names `POSTGRES_DATABASE_URL`. Migrations live in the crate and run with `sqlx::migrate!`. Check in `template/crates/store-postgres/.sqlx/` from `just db-prepare`.
3. REQ-RUN-0328 and NFR-RUN-0002: the crate implements `EngineStore::open`, `read_rows`, and `write_rows` from the `b-4` sample. It does not declare a second trait and it does not declare adaptor types. The query functions do not call the sleep or `block_on` names.

## Acceptance criteria

Root: `boundary:store-postgres`.

- `SQLX_OFFLINE=true cargo build -p store-postgres --manifest-path template/Cargo.toml` exits 0 with no live database, and does not commit `template/Cargo.lock`.
- `git ls-files template/crates/store-postgres/.sqlx` lists at least one file.
- `rg -n 'AUTOINCREMENT|PRAGMA |WITHOUT ROWID' template/crates/store-postgres` exits 1.

## Required validation

- `SQLX_OFFLINE=true cargo test -p store-postgres --manifest-path template/Cargo.toml` exits 0.
- `SQLX_OFFLINE=true cargo build -p store-postgres --manifest-path template/Cargo.toml` exits 0.
- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' template/crates/store-postgres` exits 1.

## Non-goals

No MySQL crate edits. No adaptor. No answers-schema edit (`b-8`). No `just new` of a generated project (`b-8`). No rusqlite writer edit.
