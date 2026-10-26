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
  - REQ-RUN-0503
  - REQ-RUN-0317
  - REQ-RUN-0318
  - REQ-RUN-0303
adrs:
  - ADR-RUN-0305
  - ADR-RUN-0401
  - ADR-RUN-0002
owned_paths:
  - wizard/answers.schema.json
  - scripts/new_project.py
  - template/cargo-generate.toml
  - docs/generation.md
must_follow: [b-7]
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: Each application chooses one sqlx SQL engine, Postgres or MySQL. Both engines are layouts in this sprint so the answers schema is written once.
---

# Sprint b-8 — generation layouts

## Goal

`just new` (`scripts/new_project.py`) creates an application in exactly one layout. The layout is no database, SQLite only, one sqlx SQL engine and no SQLite, or SQLite plus one sqlx SQL engine. That sqlx engine is Postgres or MySQL. One application contains one of those engines. The Postgres-only layout and the MySQL-only layout are the one-engine layouts. The SQLite-plus-Postgres layout and the SQLite-plus-MySQL layout are the SQLite-plus-one-engine layouts. For a SQLite-plus-Postgres layout, SQLite and Postgres meet in an adaptor crate generated into the consumer repository. For a SQLite-plus-MySQL layout, SQLite and MySQL meet in an adaptor crate generated the same way. A one-store layout renders that store and no adaptor. A no-database layout renders no store crate and no adaptor.

The library crate `sc-runtime` does not generate any of this.

## Tasks

1. REQ-RUN-0326: a new application is created with no database, with SQLite only, with one sqlx SQL engine only, or with SQLite plus one sqlx SQL engine. The engine is Postgres or MySQL, and the application contains that one engine. `wizard/answers.schema.json` accepts the Postgres-only layout, the MySQL-only layout, the SQLite-plus-Postgres layout, and the SQLite-plus-MySQL layout as those two families. No application contains both Postgres and MySQL.
2. REQ-RUN-0317 and REQ-RUN-0318: `just new` writes the stores, the models, and the adaptor. `crates/sc-runtime` contains no generator.
3. REQ-RUN-0329 and REQ-RUN-0330: each generated package name is the project name, a hyphen, and its role. Row structs are in `<project>-schema`. `<project>-types` is for non-database types. The adaptor depends on `<project>-schema`.
4. REQ-RUN-0331: a layout with a store renders general database commands on the CLI, MCP, and HTTP, including `delete row <table> <id>`. An application command calls the same delete operation through its own CLI command, MCP tool, and HTTP route. A flag removes the general commands from those three surfaces. A no-database layout renders none of them.
5. REQ-RUN-0503: `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` exits 0. With no file added, changed, or deleted, `just setup`, then `just lint`, then `just test`, then `just build-cli`, then `just build-daemon` exit 0. `just setup` installs the sc-lint consumer and is not a source edit.
6. ADR-RUN-0002: sc-runtime is a cargo-generate template plus published library crates. `just new` renders that template into the new repository. The library crate `sc-runtime` does not contain the generator.

## Required validation

- `just new` for each of the six layouts exits 0.
- `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` exits 0. With no file added, changed, or deleted, `just setup`, then `just lint`, then `just test`, then `just build-cli`, then `just build-daemon` exit 0.
- The SQLite-only project contains `store-sqlite` and no adaptor.
- The Postgres-only project contains `store-postgres` and no `store-sqlite`.
- The MySQL-only project contains `store-mysql` and no `store-sqlite`.
- Each two-store project contains SQLite, the store for its one sqlx engine, and the adaptor. No project contains both `store-postgres` and `store-mysql`.
- The no-database project contains no `store-*` crate.
- `rg` of a generator under `crates/sc-runtime` prints nothing.
- A project named `acme` has packages `acme-types` and `acme-schema`. The row structs are in `acme-schema`. The adaptor depends on `acme-schema`.

## Non-goals

No SQL Server layout. No new SQL inside the store crates. No edit to `sc-cron` or the rusqlite writer.
