---
id: b-4
phase: b
status: planned
closure_type: boundary
target_boundary: store-pairing
track: stores
wave: 2
branch: sprint/b-4-store-pairing
worktree: sprint/b-4-store-pairing
requirements:
  - REQ-RUN-0321
  - REQ-RUN-0327
  - REQ-RUN-0328
adrs:
  - ADR-RUN-0301
  - ADR-RUN-0305
owned_paths:
  - template/crates/store-sqlite/
  - template/Cargo.toml
must_follow: [b-2]
parallel_safe: [b-3]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The schema rule and the trait rule are one decision. Splitting them would let a store sprint pick a shape the other stores then have to undo.
---

# Sprint b-4 — store pairing

## Goal

Two stores in one application use the same schema or a similar schema, and one shared trait or a different trait per store. Each engine's SQL stays in that engine's crate. `store-sqlite` keeps its sqlx driver, two pools, WAL, and `sqlx::migrate!`. This sprint does not replace it with the rusqlite writer.

The trait is shaped for SQLite, Postgres, and MySQL. Postgres and MySQL crates are filled in `b-5` and `b-6`. This sprint reserves their place in the template workspace so those two sprints do not edit `template/Cargo.toml`.

`<project>-schema` holds the table description and the row structs ([REQ-RUN-0330](../../requirements.md)). `<project>-types` holds non-database types. Package names are the project name, a hyphen, and the role ([REQ-RUN-0329](../../requirements.md)).

## Tasks

1. REQ-RUN-0327: the copy pattern generates the same schema on both stores. The host-aggregate pattern generates a similar schema, with tables or columns that exist on only one side.
2. REQ-RUN-0328: the stores name one trait both implement, or a trait per store. Both shapes are legal. The generated common patterns pick one shape and use it for every consumer.
3. REQ-RUN-0321: SQLite SQL stays in `store-sqlite`. The trait carries no SQL string that runs on more than one engine.

## Required validation

- `cargo test` in a project generated with `db` `sqlite` exits 0.
- The trait's source contains no SQL literal.
- `template/Cargo.toml` names `store-postgres` and `store-mysql`.

## Non-goals

No Postgres SQL. No MySQL SQL. No adaptor. No change to `crates/sc-runtime-db-sqlite/`. No change to the six generation layouts (`b-8` owns those).
