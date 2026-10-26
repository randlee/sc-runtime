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
  - NFR-RUN-0002
adrs:
  - ADR-RUN-0301
  - ADR-RUN-0303
  - ADR-RUN-0305
  - ADR-RUN-0012
  - ADR-RUN-0011
  - ADR-RUN-0002
  - ADR-RUN-0007
  - ADR-RUN-0008
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

#[derive(Debug)]
pub enum StoreCode {
    Connection,
    Query,
    Migration,
}

pub struct StoreError {
    pub code: StoreCode,
    pub message: String,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for StoreError {}

pub struct RowRecord {
    pub table: String,
    /// UTF-8 JSON object. Keys are column names.
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

`OpenConfig.url` is the database string for that engine. SQLite reads a file path from it. Postgres and MySQL read a database URL. There is one `OpenConfig`. `store-sqlite::open` takes `&OpenConfig` with `url: String`. `StoreError` implements `Display` and `std::error::Error`. `Display` includes the code. The schema crate does not wrap a sqlx or rusqlite error, so `Error::source` returns `None`. The engine crate maps its driver error into `StoreCode` and `message` before it returns. `read_rows` and `write_rows` are the row operations the adaptor calls. `RowRecord.payload` is UTF-8 JSON whose keys are column names. Each engine implements the trait with that engine's SQL. `EngineStore` is used with static dispatch. `dyn EngineStore` is not a supported form. This is the closed `cfg` and error type in REQ-RUN-0308 and ADR-RUN-0301 point 2. The read-pool size N stays OPEN.

`store-sqlite` re-exports `WidgetRow` from the schema crate. `template/crates/service/src/lib.rs` is not edited in this sprint. That file does not compile against the new `open` until `b-8`. This sprint builds `store-sqlite` and the schema crate, not `service`. NFR-RUN-0002: the new query functions do not call the sleep or `block_on` names ADR-RUN-0011 point 1 lists.

## Deliverables

1. REQ-RUN-0330: add `template/crates/schema/`. In this template workspace the package name is `schema`. `b-8` renders it as `<project>-schema`. Move `WidgetRow` into the schema crate and re-export it from `store-sqlite`. The schema crate contains no SQL literal. It contains `template/crates/schema/membership.toml`, which names a sqlite-only table, a remote-only table, and a column that exists on only one side. That file is not an answers-schema property.
2. REQ-RUN-0327 and NFR-RUN-0002: the schema crate describes one shared table and one table or column that exists on only one side. Schema-crate tests prove that description. They do not generate a project.
3. REQ-RUN-0328: `store-sqlite` implements `EngineStore`, including `read_rows` and `write_rows`. A Postgres stub and a MySQL stub in the schema crate's tests implement the same trait. Both a shared trait and a trait per store stay legal for code the consumer writes. The generated patterns use `EngineStore` only.
4. REQ-RUN-0321: SQLite SQL stays in `store-sqlite`. Set `template/Cargo.toml` members to `crates/*`. Add `template/Cargo.lock` to the root `.gitignore`. Cargo commands in this sprint pass `--manifest-path template/Cargo.toml` and do not commit that lockfile.
5. REQ-RUN-0308: `store-sqlite` keeps the direct `sqlx` dependency. The schema crate does not gain one. The driver row and success criterion 5 already say a direct `sqlx` dependency belongs to each rendered `store-*` crate and to no other generated crate.

## This sprint does not close

`template/crates/service/src/lib.rs` still calls `OpenConfig { path }` and `store_sqlite::WidgetRow` until `b-8`. This sprint does not edit it, and it does not compile until that call site uses `OpenConfig { url }` and the schema row. Generated layouts, package renames, and `just new` are `b-8`.

## Acceptance criteria

Root: `boundary:store-pairing`.

- The schema crate compiles, holds `WidgetRow` and `EngineStore`, and contains no SQL literal.
- A Postgres stub and a MySQL stub in the schema crate's tests implement `open`, `read_rows`, and `write_rows`. An engine-incompatible trait fails here.
- `store-sqlite::open` takes `OpenConfig { url: String }` and returns `StoreError` with `StoreCode`. `Display` includes the code.
- `SQLX_OFFLINE=true cargo build -p store-sqlite --manifest-path template/Cargo.toml` exits 0 with only the crates that exist in this sprint, and does not commit `template/Cargo.lock`.
- `template/Cargo.toml` does not name `store-postgres` or `store-mysql`.
- The schema crate's tests describe one shared table and one one-sided table or column.

## Required validation

- `SQLX_OFFLINE=true cargo test -p schema --manifest-path template/Cargo.toml` exits 0.
- `SQLX_OFFLINE=true cargo test -p store-sqlite --manifest-path template/Cargo.toml` exits 0, including the concurrent-insert test and the WAL test in REQ-RUN-0308 success criteria 2 and 3.
- `rg -n 'SELECT |INSERT |UPDATE |DELETE |CREATE TABLE' template/crates/schema` exits 1.
- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' template/crates/schema template/crates/store-sqlite` exits 1.
- `git check-ignore -q template/Cargo.lock` exits 0.

## Non-goals

No Postgres SQL. No MySQL SQL. No adaptor. No change to `crates/sc-runtime-db-sqlite/`. No edit to `template/crates/service/`. No change to the six generation layouts (`b-8` owns those). No package-name rule for `service`, `daemon`, or `cli` (`b-8` owns REQ-RUN-0329).
