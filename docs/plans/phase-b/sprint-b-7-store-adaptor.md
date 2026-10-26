---
id: b-7
phase: b
status: planned
closure_type: boundary
target_boundary: store-adaptor
track: stores
wave: 2
branch: sprint/b-7-store-adaptor
worktree: sprint/b-7-store-adaptor
requirements:
  - REQ-RUN-0313
  - REQ-RUN-0314
  - REQ-RUN-0315
  - REQ-RUN-0316
  - REQ-RUN-0319
  - REQ-RUN-0320
  - REQ-RUN-0322
  - REQ-RUN-0323
  - REQ-RUN-0324
  - REQ-RUN-0325
adrs:
  - ADR-RUN-0303
  - ADR-RUN-0305
  - ADR-RUN-0013
owned_paths:
  - template/crates/store-adaptor/
must_follow: [b-4]
parallel_safe: [b-5, b-6]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The copy pattern and the host-aggregate pattern are one adaptor crate. Splitting them would let each pattern invent its own sync shape.
---

# Sprint b-7 — store adaptor

## Goal

SQLite and Postgres meet in an adaptor crate generated into the consumer repository. The relationship between the local SQLite store and the remote Postgres or SQL store is defined in an adaptor crate on the consumer repository. This sprint writes that adaptor in the template. `b-8` renders it into the consumer repository.

This sprint's `Cargo.toml` depends only on the schema crate from `b-4`. Tests use `EngineStore` doubles. The adaptor calls `read_rows` on the source store and `write_rows` on the destination store. It contains no engine SQL and does not depend on sqlx. `b-8` renders the [ADR-RUN-0303](../../architecture.md) edges from the adaptor to the concrete `store-*` crates the layout contains.

The copy adaptor is a local SQLite database with the remote schema, for faster or offline access. The host-aggregate adaptor is one SQLite database for this host and a Postgres database that aggregates many hosts. The same functions serve MySQL when MySQL is the layout's one sqlx engine. The consumer repository defines the rollup from host rows into the Postgres aggregate.

The template adaptor moves rows between the local store and the remote store. It starts no timer. The application schedules it, using `sc-cron` when it wants that scheduler. The sc-runtime crates do not move rows between stores. No file under `crates/` moves rows between stores. The template adaptor is not one of those crates.

## Adaptor sample

`b-5`, `b-6`, and `b-8` use this sample and do not restate it. It is generic over `EngineStore`. The host-aggregate remote side is the layout's one sqlx engine.

```rust
pub struct TableFlags {
    pub sqlite_only: Vec<String>,
    pub remote_only: Vec<String>,
    pub column_on_one_side: Vec<String>,
}

pub trait Rollup {
    fn rollup(&self, host_rows: &[RowRecord]) -> Vec<RowRecord>;
}

pub async fn copy_rows<S, D>(source: &S, dest: &D, table: &str) -> Result<(), StoreError>
where
    S: EngineStore,
    D: EngineStore,
{
    let rows = source.read_rows(table).await?;
    dest.write_rows(table, &rows).await
}
```

`copy_rows` is the copy function both engines use. `Rollup` is the function the consumer implements. `TableFlags` names a sqlite-only table, a remote-only table, and a column that exists on only one side.

## Deliverables

1. REQ-RUN-0314: the template adaptor contains `copy_rows`. The copy adaptor is a local SQLite database with the remote schema. REQ-RUN-0315: the template adaptor contains the host-aggregate path. The host-aggregate adaptor is one SQLite database for this host and a Postgres database that aggregates many hosts.
2. REQ-RUN-0316: `Rollup` is the consumer-owned function that maps host rows into the Postgres aggregate. REQ-RUN-0319: `TableFlags` names tables and columns that exist on only one side, in either direction.
3. REQ-RUN-0320: the template holds one copy implementation and one host-aggregate implementation. `b-8` renders that same implementation for every consumer that selects the pattern.
4. REQ-RUN-0323 and REQ-RUN-0324: the adaptor moves rows through `read_rows` and `write_rows` and starts no timer. A schedule is application code. A cron row points at a Rust method or a Python hook, which is the `b-3` contract.
5. REQ-RUN-0325: the sc-runtime crates do not move rows between stores. No file under `crates/` moves rows between stores. `copy_rows` lives only in `template/crates/store-adaptor`.
6. REQ-RUN-0322: a third adaptor is a file the consumer writes. This sprint does not generate one.

## This sprint does not close

`just new`, the six layouts, and the rendered adaptor's dependency on concrete `store-*` crates are `b-8`. This sprint does not wait for `b-5` or `b-6`.

## Acceptance criteria

Root: `boundary:store-adaptor`.

- The template adaptor contains `copy_rows` and the host-aggregate path, and its `Cargo.toml` depends on the schema crate and on no `store-*` crate.
- The adaptor calls `read_rows` and `write_rows`. It starts no timer and contains no engine SQL literal.
- `SQLX_OFFLINE=true cargo test -p store-adaptor --manifest-path template/Cargo.toml` exits 0 against `EngineStore` doubles, and does not commit `template/Cargo.lock`.
- `crates/` in this repository contains no store-to-store copy.

## Required validation

- `SQLX_OFFLINE=true cargo test -p store-adaptor --manifest-path template/Cargo.toml` exits 0.
- `rg -n 'SELECT |INSERT |UPDATE |DELETE |CREATE TABLE' template/crates/store-adaptor` exits 1.
- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' template/crates/store-adaptor` exits 1.
- `rg -n 'path = \"../store-' template/crates/store-adaptor/Cargo.toml` exits 1.
- `crates/` contains no store-to-store copy, outbox, or routing policy.

## Non-goals

No `just new` layout switch (`b-8`). No HTTP, MCP, or CLI API on `sc-cron`. No edit to the rusqlite writer. No dependency on `store-postgres` or `store-mysql`.
