---
id: b-4
phase: b
status: planned
closure_type: boundary
target_boundary: store-pairing
track: stores
wave: 1
branch: sprint/b-4-store-pairing
worktree: sprint/b-4-store-pairing
requirements:
  - REQ-RUN-0321
  - REQ-RUN-0327
  - REQ-RUN-0328
  - REQ-RUN-0330
  - REQ-RUN-0308
adrs:
  - ADR-RUN-0301
  - ADR-RUN-0303
  - ADR-RUN-0305
owned_paths:
  - template/crates/store-sqlite/
  - template/crates/schema/
  - template/Cargo.toml
  - .gitignore
must_follow: []
parallel_safe: [b-1, b-2, b-3]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The schema crate and the trait are one decision. Splitting them would let a store sprint pick a shape the other stores then have to undo.
---

# Sprint b-4 — store pairing

## Goal

Two stores in one application use the same schema or a similar schema, and one shared trait or a different trait per store. Each engine's SQL stays in that engine's crate. `store-sqlite` keeps its sqlx driver, two pools, WAL, and `sqlx::migrate!`. This sprint does not replace it with the rusqlite writer.

The generated common patterns use one shared trait, `EngineStore`, in the template crate that renders as `<project>-schema`. A consumer-written adaptor may still use a different trait per store. The trait and the schema crate contain no SQL string.

`template/Cargo.toml` members become the glob `crates/*`. A new directory under `template/crates/` is a member when it exists. This sprint does not create `store-postgres` or `store-mysql`.

## Pairing contract

The schema crate holds the table description, the row structs, and this trait. `store-sqlite` moves `WidgetRow` into that crate and implements the trait. `b-5` and `b-6` implement the same trait. They do not choose a second shape.

```rust
pub struct OpenConfig {
    pub url: String,
}

pub struct StoreError {
    pub message: String,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for StoreError {}

pub struct RowRecord {
    pub table: String,
    pub payload: Vec<u8>,
}

pub trait EngineStore: Send + Sync {
    fn open(
        cfg: &OpenConfig,
    ) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send
    where
        Self: Sized;

    fn read_rows(
        &self,
        table: &str,
    ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send;

    fn write_rows(
        &self,
        table: &str,
        rows: &[RowRecord],
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
}
```

`OpenConfig.url` is the database string for that engine. SQLite reads a file path from it. Postgres and MySQL read a database URL. `StoreError` implements `Display` and `std::error::Error`. `read_rows` and `write_rows` are the row operations the adaptor calls. Each engine implements them with that engine's SQL. This is the closed `cfg` and error type in REQ-RUN-0308 and ADR-RUN-0301 point 2. The read-pool size N stays OPEN.

`store-sqlite` keeps its inherent `open` that takes `OpenConfig { path: PathBuf }` and returns the store error `service` already calls with `err.to_string()`. It re-exports `WidgetRow` from the schema crate. `template/crates/service/src/lib.rs` is not edited in this sprint.

## Deliverables

1. REQ-RUN-0330: add `template/crates/schema/`. In this template workspace the package name is `schema`. `b-8` renders it as `<project>-schema`. Move `WidgetRow` into the schema crate and re-export it from `store-sqlite`. The schema crate contains no SQL literal.
2. REQ-RUN-0327: the schema crate describes one shared table and one table or column that exists on only one side. Schema-crate tests prove that description. They do not generate a project.
3. REQ-RUN-0328: `store-sqlite` implements `EngineStore`, including `read_rows` and `write_rows`. A Postgres stub and a MySQL stub in the schema crate's tests implement the same trait. Both a shared trait and a trait per store stay legal for code the consumer writes. The generated patterns use `EngineStore` only.
4. REQ-RUN-0321: SQLite SQL stays in `store-sqlite`. Set `template/Cargo.toml` members to `crates/*`. Add `template/Cargo.lock` to the root `.gitignore`. Cargo commands in this sprint pass `--manifest-path template/Cargo.toml` and do not commit that lockfile.
5. REQ-RUN-0308: `store-sqlite` keeps the direct `sqlx` dependency. The schema crate does not gain one. The driver row and success criterion 5 already say a direct `sqlx` dependency belongs to each rendered `store-*` crate and to no other generated crate.

## This sprint does not close

`template/crates/service/src/lib.rs` stays on `OpenConfig { path }` and `store_sqlite::WidgetRow`. `b-8` owns that file and updates the call site. Generated layouts, package renames, and `just new` are `b-8`.

## Acceptance criteria

Root: `boundary:store-pairing`.

- The schema crate compiles, holds `WidgetRow` and `EngineStore`, and contains no SQL literal.
- A Postgres stub and a MySQL stub in the schema crate's tests implement `open`, `read_rows`, and `write_rows`. An engine-incompatible trait fails here.
- `store-sqlite` still implements `Display` and `std::error::Error` on the error its inherent `open` returns.
- `SQLX_OFFLINE=true cargo build -p store-sqlite --manifest-path template/Cargo.toml` exits 0 with only the crates that exist in this sprint, and does not commit `template/Cargo.lock`.
- `template/Cargo.toml` does not name `store-postgres` or `store-mysql`.
- The schema crate's tests describe one shared table and one one-sided table or column.

## Required validation

- `SQLX_OFFLINE=true cargo test -p schema --manifest-path template/Cargo.toml` exits 0.
- `SQLX_OFFLINE=true cargo build -p store-sqlite --manifest-path template/Cargo.toml` exits 0.
- `rg -n 'SELECT |INSERT |UPDATE |DELETE |CREATE TABLE' template/crates/schema` exits 1.
- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' template/crates/schema template/crates/store-sqlite` exits 1.
- `git check-ignore -q template/Cargo.lock` exits 0.

## Non-goals

No Postgres SQL. No MySQL SQL. No adaptor. No change to `crates/sc-runtime-db-sqlite/`. No edit to `template/crates/service/`. No change to the six generation layouts (`b-8` owns those). No package-name rule for `service`, `daemon`, or `cli` (`b-8` owns REQ-RUN-0329).
