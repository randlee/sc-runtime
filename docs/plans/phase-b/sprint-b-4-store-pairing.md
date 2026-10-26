---
id: b-4
phase: b
status: complete
closure_type: boundary
target_boundary: store-pairing
track: stores
wave: 1
branch: sprint/b-4-store-pairing
worktree: sprint/b-4-store-pairing
requirements:
  - REQ-RUN-0301
  - REQ-RUN-0306
  - REQ-RUN-0308
  - REQ-RUN-0321
  - REQ-RUN-0327
  - REQ-RUN-0328
  - REQ-RUN-0330
  - NFR-RUN-0002
  - NFR-RUN-0004
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
  - template/workspace.template.toml
  - template/crates/service/src/lib.rs
  - crates/sc-runtime/src/builder.rs
  - crates/sc-runtime/src/router.rs
  - docs/architecture.md
  - docs/requirements.md
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

Row type structs are in the schema crate. The types crate is for non-database types. `template/crates/api-types` gains no row struct. The schema crate holds the table description, the row structs, and this trait. `WidgetRow` is defined there. `store-sqlite` re-exports it and implements the trait. `b-5` and `b-6` implement the same trait. They do not choose a second shape.

```rust
#[non_exhaustive]
pub struct OpenConfig {
    pub url: String,
}

impl OpenConfig {
    pub fn new(target: impl Into<String>) -> Self {
        Self { url: target.into() }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum StoreCode {
    Connection,
    Query,
    Migration,
    NotFound,
    Conflict,
    Unavailable,
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

`OpenConfig.url` is the database string for that engine. SQLite reads a filesystem path from it. Postgres and MySQL read a database URL. There is one `OpenConfig`. A path newtype and a URL newtype would be a second config shape, which this phase does not add. Crates other than the schema crate call `OpenConfig::new`. `store-sqlite::open` takes `&OpenConfig` with `url: String`. `StoreError` implements `Display` and `std::error::Error`. `Display` includes the code. `StoreCode::remediation` is the recovery text. `Unavailable` is the only retryable variant. The schema crate does not wrap a sqlx or rusqlite error, so `Error::source` returns `None`. The engine crate copies the driver text into `message` and picks `StoreCode` before it returns. `read_rows` and `write_rows` are the row operations the adaptor calls. They reject more than 10_000 rows or a payload over 1 MiB.

`RowRecord.payload` is a UTF-8 JSON object. Keys are column names. NULL is `null`. Booleans are `true` or `false`. Integers inside `-(2^53)+1 ..= 2^53-1` are JSON numbers, and integers outside that range are decimal strings. Text is a JSON string. Bytes are standard base64 with no whitespace. Timestamps are RFC 3339 strings.

Each engine implements the trait with that engine's SQL. `EngineStore` is the adaptor seam. Application functions such as `insert_widget` stay on the concrete store. `EngineStore` is used with static dispatch. `dyn EngineStore` is not a supported form. This is the closed `cfg` and error type in REQ-RUN-0308 and ADR-RUN-0301 point 2. The read-pool size N stays OPEN.

`store-sqlite` re-exports `WidgetRow`, `WidgetId`, `OpenConfig`, `StoreCode`, and `StoreError`. It does not re-export `EngineStore` or `RowRecord`. The adaptor imports those from the schema crate. `WidgetId` is the column-text newtype on `WidgetRow`. The validated application id stays `api-types`'s `WidgetId`. `template/crates/service/src/lib.rs` `Stores::open` passes the filesystem path as `OpenConfig::new(url)`. Package renames and the adaptor import stay with `b-8`. NFR-RUN-0002: the new query functions do not call the sleep or `block_on` names ADR-RUN-0011 point 1 lists.

## Deliverables

1. REQ-RUN-0330: add `template/crates/schema/`. In this template workspace the package name is `schema`. `b-8` renders it as `<project>-schema`. Row type structs are in the schema crate. The types crate is for non-database types. `WidgetRow` is defined in the schema crate and re-exported from `store-sqlite`. The schema crate contains no SQL literal. It contains `template/crates/schema/membership.toml`, which names shared tables and, by engine name, tables and columns that exist on only one side (`sqlite`, `postgres`, and `mysql`). An empty description is valid when the engines share one schema. That file is not an answers-schema property. `EngineStore`, `StoreError`, and `OpenConfig` are consumer-owned copies. The trait shape is the cross-project contract.
2. REQ-RUN-0327 and NFR-RUN-0002: the schema crate describes one shared table and one table or column that exists on only one side. Schema-crate tests prove that description. They do not generate a project.
3. REQ-RUN-0328: `store-sqlite` implements `EngineStore`, including `read_rows` and `write_rows`. A Postgres stub and a MySQL stub in the schema crate's tests implement the same trait. Both a shared trait and a trait per store stay legal for code the consumer writes. The generated patterns use `EngineStore` only.
4. REQ-RUN-0321: SQLite SQL stays in `store-sqlite`. Set `template/Cargo.toml` members to `crates/*`. Add `template/Cargo.lock` to the root `.gitignore`. Cargo commands in this sprint pass `--manifest-path template/Cargo.toml` and do not commit that lockfile.
5. REQ-RUN-0308: `store-sqlite` keeps the direct `sqlx` dependency. The schema crate does not gain one. The driver row and success criterion 5 already say a direct `sqlx` dependency belongs to each rendered `store-*` crate and to no other generated crate. `Stores::open` in `template/crates/service/src/lib.rs` passes the filesystem path through `OpenConfig::new`, so the field is `url`.
6. REQ-RUN-0301: `template/Cargo.toml` members are the glob `crates/*`, which is the rendered workspace layout. The checkout manifest excludes `cli`, `daemon`, and `service` because those manifests are not valid Cargo until generation finishes. `template/workspace.template.toml` is the same workspace metadata without that exclude. `template/hooks/drop-template-exclude.rhai` deletes the checkout manifest and renames the template one into place. A schema-crate test diffs the two manifests aside from the exclude block.
7. REQ-RUN-0306: plain `cargo generate` must leave a workspace where `cargo build --workspace` exits 0. The post hook above is that path's manifest. `Stores::open` uses `OpenConfig::new` so `service` compiles against the pairing contract on that build.

## Standard tools (NFR-RUN-0004)

1. Schema crate, `EngineStore`, `StoreError`, and `OpenConfig`. Considered `sqlx` (checked SQL for one engine, not a cross-engine trait), `serde_json` (encodes a payload, does not name tables or move rows), and `cargo-generate` (renders files, does not define the store contract). The trait is the contract those tools do not supply.
2. Membership description. Considered `sqlx` migrations (one engine, so they cannot name a column that exists on only one side) and `cargo-generate` placeholders (generation options, not the table list the adaptor reads). The `toml` crate parses the file in tests. A runtime parser waits for the b-7 adaptor, the first non-test consumer.
3. `store-sqlite` `read_rows` and `write_rows`. `sqlx` is the SQL tool and stays the driver. It does not define the shared row-move trait or the row cap. `futures-util` streams the read so the cap can stop the scan. The caps live on the schema crate so b-5 and b-6 use the same bounds.
4. Workspace glob and the cargo-generate hook. Cargo workspace members and cargo-generate's post hook are the tools. No second template is added. The manifest diff test is the drift check.
5. Service open line. No new crate. The existing path string is passed as `OpenConfig.url` so plain `cargo generate` can build.
6. Request body limit and handler-timeout drop. Axum's `DefaultBodyLimit` is the limiter. The runtime states the 2 MiB default and the per-route disable hatch. The timeout middleware already drops the handler future. The docs state that drop is cooperative cancellation, which is Tokio's tool. No second timeout layer is added.

## This sprint does not close

`WidgetRow` has one definition, in the schema crate. `store-sqlite` re-exports it. `template/crates/service/src/lib.rs` `Stores::open` passes that path as `OpenConfig.url`. It does not switch to a schema-crate import, and it does not grow an adaptor. Generated layouts, package renames, and `just new` are `b-8`. The request-body limit and the timeout-drop note in `sc-runtime` are the daemon bounds for this template. They are not a new runtime product sprint.

## Acceptance criteria

Root: `boundary:store-pairing`.

- The schema crate compiles, holds `WidgetRow` and `EngineStore`, and contains no SQL literal.
- A Postgres stub and a MySQL stub in the schema crate's tests implement `open`, `read_rows`, and `write_rows`. An engine-incompatible trait fails here.
- `store-sqlite::open` takes `OpenConfig { url: String }` (`OpenConfig::new`) and returns `StoreError` with `StoreCode`. `Display` includes the code. `Stores::open` passes the filesystem path as that `url`.
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

No Postgres SQL. No MySQL SQL. No adaptor. No change to `crates/sc-runtime-db-sqlite/`. The only `template/crates/service/` edit is `Stores::open` passing `OpenConfig.url`. No change to the six generation layouts (`b-8` owns those). No package-name rule for `service`, `daemon`, or `cli` (`b-8` owns REQ-RUN-0329).
