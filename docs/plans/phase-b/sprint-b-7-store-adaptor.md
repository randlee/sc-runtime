---
id: b-7
phase: b
status: planned
closure_type: boundary
target_boundary: store-adaptor
track: stores
wave: 4
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
  - ADR-RUN-0305
  - ADR-RUN-0013
owned_paths:
  - template/crates/store-adaptor/
must_follow: [b-5, b-6]
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The copy pattern and the host-aggregate pattern are one adaptor crate. Splitting them would let each pattern invent its own sync shape.
---

# Sprint b-7 — store adaptor

## Goal

The relationship between the local SQLite store and a remote SQL store lives in an adaptor crate on the generated project. The adaptor depends on `<project>-schema`, which holds the schema and the row structs ([REQ-RUN-0330](../../requirements.md)). `<project>-types` is for non-database types. The copy pattern uses the remote schema. The host-aggregate pattern uses a similar schema, and the consumer owns the rollup function. The same pattern is generated the same way for Postgres and for MySQL.

The adaptor moves rows. It starts no timer. The application schedules it, using `sc-cron` when it wants that scheduler. The `sc-runtime` crates do not move rows.

## Tasks

1. REQ-RUN-0314: generate the copy adaptor. REQ-RUN-0315: generate the host-aggregate adaptor as one SQLite database for this host and a Postgres database that aggregates many hosts.
2. REQ-RUN-0316: the consumer repository defines the rollup, and that function maps host rows into the Postgres aggregate. REQ-RUN-0319: flags name tables and columns that exist on only one side, in either direction.
3. REQ-RUN-0320: a consumer that selects the copy pattern gets the same adaptor implementation as every other consumer that selects it. The same for the host-aggregate pattern.
4. REQ-RUN-0323 and REQ-RUN-0324: the adaptor moves rows and starts no timer. A schedule is application code. A cron row points at a Rust method or a Python hook, which is the `b-3` contract.
5. REQ-RUN-0325: no crate under `crates/` in this repository moves rows between stores.
6. REQ-RUN-0322: a third adaptor is a file the consumer writes. This sprint does not generate one.

## Required validation

- The adaptor crate contains the copy from one store to the other.
- The adaptor crate starts no timer.
- `crates/` contains no store-to-store copy, outbox, or routing policy.

## Non-goals

No `just new` layout switch (`b-8`). No HTTP, MCP, or CLI API on `sc-cron`. No edit to the rusqlite writer.
