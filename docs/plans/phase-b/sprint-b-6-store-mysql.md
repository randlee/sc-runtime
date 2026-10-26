---
id: b-6
phase: b
status: complete
closure_type: boundary
target_boundary: store-mysql
track: stores
wave: 2
branch: sprint/b-6-store-mysql
worktree: sprint/b-6-store-mysql
requirements:
  - REQ-RUN-0301
  - REQ-RUN-0321
  - REQ-RUN-0328
  - REQ-RUN-0308
  - NFR-RUN-0002
adrs:
  - ADR-RUN-0301
  - ADR-RUN-0303
  - ADR-RUN-0305
  - ADR-RUN-0012
  - ADR-RUN-0011
  - ADR-RUN-0007
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

`store-mysql` is the sqlx MySQL store. It implements `EngineStore` from `b-4`, the same trait `store-postgres` implements. Its SQL is MySQL SQL. It does not contain SQLite SQL or Postgres SQL. REQ-RUN-0301 places this crate in the template workspace when the layout's engine is MySQL. ADR-RUN-0012 keeps the template store on sqlx; the rusqlite writer stays in `sc-runtime-db-sqlite`. This sprint does not edit `template/Cargo.toml` or `template/crates/schema/`.

Checked-in `.sqlx` data is a deliverable of REQ-RUN-0308. The compile proof runs with `SQLX_OFFLINE=true` and no live database. The crate implements `EngineStore::open`, `read_rows`, and `write_rows`. It does not declare adaptor types. Cargo commands pass `--manifest-path template/Cargo.toml` and do not commit `template/Cargo.lock`.

## Deliverables

1. REQ-RUN-0321: MySQL queries live in `template/crates/store-mysql`.
2. ADR-RUN-0301 and REQ-RUN-0308: the crate depends on sqlx with the `mysql`, `mysql-rsa`, and `tls-rustls-ring-webpki` features. `mysql-rsa` finishes MySQL 8 `caching_sha2_password` when the server has no TLS. `tls-rustls-ring-webpki` lets an `ssl-mode` URL negotiate TLS. `sqlx.toml` names `MYSQL_DATABASE_URL`. Migrations live in the crate and run with `sqlx::migrate!`. Check in `template/crates/store-mysql/.sqlx/` from `just db-prepare`. The `widgets.id` column is `VARCHAR(255)` with `utf8mb4_bin`, so primary-key equality is case-sensitive like the other engines. `widgets.name` is `MEDIUMTEXT`, so a legal 1 MiB pairing payload fits. `write_rows` and `insert_widget` reject an id over that 255-character limit before MySQL sees it.
3. REQ-RUN-0328 and NFR-RUN-0002: the crate implements `EngineStore::open`, `read_rows`, and `write_rows` from the `b-4` sample. It does not declare a second trait and it does not declare adaptor types. The query functions do not call the sleep or `block_on` names.

## Acceptance criteria

Root: `boundary:store-mysql`.

- `SQLX_OFFLINE=true cargo build -p store-mysql --manifest-path template/Cargo.toml` exits 0 with no live database, and does not commit `template/Cargo.lock`.
- `git ls-files template/crates/store-mysql/.sqlx` lists at least one file.
- `rg -n 'RETURNING |ON CONFLICT|AUTOINCREMENT|PRAGMA ' template/crates/store-mysql` exits 1.

## Required validation

- `SQLX_OFFLINE=true cargo test -p store-mysql --manifest-path template/Cargo.toml` exits 0.
- `SQLX_OFFLINE=true cargo build -p store-mysql --manifest-path template/Cargo.toml` exits 0.
- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' template/crates/store-mysql` exits 1.
- `python3 scripts/consumer_checks.py store-boundaries`, run from `template/`, exits 0. It rejects `RETURNING `, `ON CONFLICT`, `AUTOINCREMENT`, and `PRAGMA ` in `store-mysql`, rejects `UUID()` and backtick-quoted identifiers in `store-postgres` and `store-sqlite`, rejects the blocking-call names above under every `template/crates/store-*`, and fails when the shared codec and redaction functions drift.

That offline test skips live SQL when `MYSQL_DATABASE_URL` is unset. The required live run is the `store-mysql live SQL` job in `.github/workflows/store-mysql.yml`. It sets `REQUIRE_MYSQL=1` and `MYSQL_DATABASE_URL` against a MySQL service, so a skip panics, and the job fails if the skip text is printed. The same job migrates that database and runs `cargo sqlx prepare --check`. Local development keeps the skip unless `REQUIRE_MYSQL=1` is set. `SQLX_OFFLINE=true` stays set for the test compile.

## Non-goals

No Postgres crate edits. No adaptor. No answers-schema edit. No `just new` of a generated project (`b-8`). SQL Server is not a store. No edit to `template/crates/service/` or `template/crates/schema/`.

## Follow-ups

RBQA-F001: `known_table`, the `RowRecord` codec, and credential redaction are duplicated across the store crates. Moving them into `template/crates/schema` is outside this sprint's `owned_paths` (`template/crates/store-mysql/` only). Beads `run-brx` tracks the hoist. `store-sqlite`'s `record_from_columns` still omits the 1 MiB payload check; aligning that copy is part of `run-brx`. Until the hoist, `store-boundaries` fails if the shared copies drift.

QA-001: `template/crates/service/src/lib.rs` `Stores::open` uses `PathBuf::to_string_lossy` for the SQLite URL. A non-UTF-8 path would open a different file. That crate is outside this sprint's `owned_paths`, so this sprint does not change it. Beads `run-rt3` tracks the explicit `to_str()` error.
