---
id: b-8
phase: b
status: planned
closure_type: boundary
target_boundary: generation-layouts
track: stores
wave: 5
branch: sprint/b-8-generation-layouts
worktree: sprint/b-8-generation-layouts
requirements:
  - REQ-RUN-0326
  - REQ-RUN-0329
  - REQ-RUN-0330
  - REQ-RUN-0331
  - REQ-RUN-0317
  - REQ-RUN-0318
  - REQ-RUN-0303
adrs:
  - ADR-RUN-0305
  - ADR-RUN-0401
owned_paths:
  - wizard/answers.schema.json
  - scripts/new_project.py
  - template/cargo-generate.toml
  - docs/generation.md
must_follow: [b-7]
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The six layouts are one answers contract. Accepting Postgres first and MySQL in a later phase would change the schema twice.
---

# Sprint b-8 — generation layouts

## Goal

`just new` (`scripts/new_project.py`) creates an application in any layout from REQ-RUN-0326: no database, SQLite only, Postgres only, MySQL only, SQLite plus Postgres, or SQLite plus MySQL. A two-store layout renders the adaptor from `b-7`. A one-store layout renders that store and no adaptor. A no-database layout renders no store crate and no adaptor.

The library crate `sc-runtime` does not generate any of this.

## Tasks

1. REQ-RUN-0326: `wizard/answers.schema.json` accepts each layout.
2. REQ-RUN-0317 and REQ-RUN-0318: `just new` writes the stores, the models, and the adaptor. `crates/sc-runtime` contains no generator.
3. REQ-RUN-0329 and REQ-RUN-0330: each generated package name is the project name, a hyphen, and its role. Row structs are in `<project>-schema`. `<project>-types` is for non-database types. The adaptor depends on `<project>-schema`.
4. REQ-RUN-0331: a layout with a store renders general database commands on the CLI, MCP, and HTTP, including `delete row <table> <id>`. The consumer can disable that surface. A no-database layout renders none of them.

## Required validation

- `just new` for each of the six layouts exits 0.
- The SQLite-only project contains `store-sqlite` and no adaptor.
- The Postgres-only project contains `store-postgres` and no `store-sqlite`.
- The MySQL-only project contains `store-mysql` and no `store-sqlite`.
- Each two-store project contains both stores and the adaptor.
- The no-database project contains no `store-*` crate.
- `rg` of a generator under `crates/sc-runtime` prints nothing.
- A project named `acme` has packages `acme-types` and `acme-schema`. The row structs are in `acme-schema`. The adaptor depends on `acme-schema`.

## Non-goals

No SQL Server layout. No new SQL inside the store crates. No edit to `sc-cron` or the rusqlite writer.
