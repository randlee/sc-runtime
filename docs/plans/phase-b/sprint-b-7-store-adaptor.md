---
id: b-7
phase: b
status: complete
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
  - NFR-RUN-0002
  - NFR-RUN-0005
  - NFR-RUN-0008
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0303
  - ADR-RUN-0305
  - ADR-RUN-0301
  - ADR-RUN-0013
  - ADR-RUN-0002
  - ADR-RUN-0007
  - ADR-RUN-0008
  - ADR-RUN-0011
owned_paths:
  - template/crates/store-adaptor/
must_follow: [b-4]
parallel_safe: [b-5, b-6]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The copy pattern and the host-aggregate pattern are one adaptor crate. Splitting them would let each pattern invent its own sync shape. The adaptor is template consumer store wiring under ADR-RUN-0002 point 6, not a crate under crates/.
---

# Sprint b-7 — store adaptor

## Goal

SQLite and Postgres meet in an adaptor crate generated into the consumer repository. The relationship between the local SQLite store and the remote Postgres or SQL store is defined in an adaptor crate on the consumer repository. This sprint writes that adaptor in the template. `b-8` renders it into the consumer repository.

This sprint's `Cargo.toml` depends only on the schema crate from `b-4`. Tests use `EngineStore` doubles. The adaptor calls `read_rows` on the source store and `write_rows` on the destination store. It contains no engine SQL and does not depend on sqlx. `b-8` renders the [ADR-RUN-0303](../../architecture.md) edges from the adaptor to the concrete `store-*` crates the layout contains.

The copy adaptor is a local SQLite database with the remote schema, for faster or offline access. The host-aggregate adaptor is one SQLite database for this host and a Postgres database that aggregates many hosts. The same functions serve MySQL when MySQL is the layout's one sqlx engine. The consumer repository defines the rollup from host rows into the Postgres aggregate.

The template adaptor moves rows between the local store and the remote store. It starts no timer. The application schedules it, using `sc-cron` when it wants that scheduler. The sc-runtime crates do not move rows between stores. No file under `crates/` moves rows between stores. The template adaptor is not one of those crates.

ADR-RUN-0002 point 6: this adaptor is consumer-owned store wiring, so it stays in the template. It cannot live in a crate under `crates/`. ADR-RUN-0305 point 9 and ADR-RUN-0301 point 6 forbid the sc-runtime crates and the framework crates from moving rows, and the adaptor must name the layout's concrete store crates, which `b-8` renders into the project. This sprint does not move that code into `crates/`.

## Adaptor sample

`b-5`, `b-6`, and `b-8` use this sample and do not restate it. It is generic over `EngineStore`. The host-aggregate remote side is the layout's one sqlx engine.

```rust
pub struct TableFlags {
    pub sqlite_only: Vec<String>,
    pub remote_only: Vec<String>,
    pub column_on_one_side: Vec<String>,
}

pub trait Rollup {
    fn rollup(&self, host_rows: &[RowRecord], one_sided_columns: &[String]) -> Vec<RowRecord>;
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

`copy_rows` is the copy function both engines use. `Rollup` is the function the consumer implements. `copy_shared_rows` and `aggregate_host` both call it with `column_on_one_side`, so one consumer function decides whether a one-sided column is dropped. `Rollup` is total. Aggregation that can fail is a third adaptor the consumer writes. `TableFlags` names a sqlite-only table, a remote-only table, and a column that exists on only one side. An unmatched `remote_engine` key contributes empty one-sided lists by design. `b-8` renders that key as a generated constant.

Each call moves one whole table, bounded by 10_000 rows and a 1 MiB payload. A repeated call replaces that table on the destination. This sprint does not clear or upsert. `b-8` renders `write_rows` to match that replace. Until then a destination that inserts can return `Conflict` on the second scheduled run.

## Deliverables

1. REQ-RUN-0314: the template adaptor contains `copy_rows`. The copy adaptor is a local SQLite database with the remote schema. REQ-RUN-0315: the template adaptor contains the host-aggregate path. The host-aggregate adaptor is one SQLite database for this host and a Postgres database that aggregates many hosts.
2. REQ-RUN-0316: `Rollup` is the consumer-owned function that maps host rows into the Postgres aggregate. REQ-RUN-0319: `TableFlags` is the struct the adaptor reads. The generation input is `template/crates/schema/membership.toml` from `b-4`, not an answers-schema property. `b-8` checks that a rendered adaptor honors that file.
3. REQ-RUN-0320: the template holds one copy implementation and one host-aggregate implementation. `b-8` renders that same implementation for every consumer that selects the pattern.
4. REQ-RUN-0323 and REQ-RUN-0324: the adaptor moves rows through `read_rows` and `write_rows` and starts no timer. A schedule is application code. A cron row points at a Rust method or a Python hook, which is the `b-3` contract.
5. REQ-RUN-0325: the sc-runtime crates do not move rows between stores. No file under `crates/` moves rows between stores. `copy_rows` lives only in `template/crates/store-adaptor`.
6. REQ-RUN-0322: a third adaptor is a file the consumer writes. This sprint does not generate one.
7. NFR-RUN-0002: `copy_rows`, `copy_shared_rows`, and `aggregate_host` are async and do not call the sleep or `block_on` names. NFR-RUN-0005: tests use in-memory `EngineStore` doubles, write no fixed path, and bind no port. NFR-RUN-0008: `template/crates/store-adaptor/src/lib.rs` stays within the 1000 source-line limit. NFR-RUN-0009: fallible public functions return `StoreError`. Production code does not unwrap on caller input.

## This sprint does not close

`just new`, the six layouts, and the rendered adaptor's dependency on concrete `store-*` crates are `b-8`. This sprint does not wait for `b-5` or `b-6`. `b-8` renders `EngineStore::write_rows` so a repeated copy or host aggregate replaces the destination table. `b-8` also renders the remote engine key passed to `TableFlags::from_membership` as a generated constant.

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
