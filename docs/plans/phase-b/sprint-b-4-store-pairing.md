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
adrs:
  - ADR-RUN-0301
  - ADR-RUN-0303
  - ADR-RUN-0305
owned_paths:
  - template/crates/store-sqlite/
  - template/crates/schema/
  - template/Cargo.toml
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

pub trait EngineStore: Send + Sync {
    fn open(
        cfg: &OpenConfig,
    ) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send
    where
        Self: Sized;
}
```

`OpenConfig.url` is the database string for that engine. SQLite reads a file path from it. Postgres and MySQL read a database URL. `StoreError` is the store's error. This closes the OPEN `cfg` and error type in REQ-RUN-0308 and ADR-RUN-0301 point 2 for the generated stores. The read-pool size N stays OPEN.

## Deliverables

1. REQ-RUN-0330: add `template/crates/schema/`. Its package name renders as `<project>-schema`. Move `WidgetRow` out of `store-sqlite` into that crate. The crate contains no SQL literal.
2. REQ-RUN-0327: the copy pattern generates the same schema on both stores. The host-aggregate pattern generates a similar schema, with tables or columns that exist on only one side.
3. REQ-RUN-0328: generated stores implement `EngineStore`. Both a shared trait and a trait per store stay legal for code the consumer writes. The generated patterns use `EngineStore` only.
4. REQ-RUN-0321: SQLite SQL stays in `store-sqlite`. Set `template/Cargo.toml` members to `crates/*`.
5. REQ-RUN-0308: amend the driver row and success criterion 5 so a direct `sqlx` dependency belongs to each rendered `store-*` crate, and to no other generated crate.

## Acceptance criteria

Root: `boundary:store-pairing`.

- The schema crate compiles, holds `WidgetRow` and `EngineStore`, and contains no SQL literal.
- A Postgres stub and a MySQL stub in the schema crate's tests implement `EngineStore`. An engine-incompatible trait fails here.
- `SQLX_OFFLINE=true cargo build -p store-sqlite` in the template workspace exits 0 with only the crates that exist in this sprint.
- `template/Cargo.toml` does not name `store-postgres` or `store-mysql`.

## Required validation

- `SQLX_OFFLINE=true cargo test -p schema` exits 0.
- `SQLX_OFFLINE=true cargo build -p store-sqlite` exits 0.
- `rg` of a SQL literal under `template/crates/schema` prints nothing.

## Non-goals

No Postgres SQL. No MySQL SQL. No adaptor. No change to `crates/sc-runtime-db-sqlite/`. No change to the six generation layouts (`b-8` owns those). No package-name rule for `service`, `daemon`, or `cli` (`b-8` owns REQ-RUN-0329).
