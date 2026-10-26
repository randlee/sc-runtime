---
id: b-8
phase: b
status: planned
closure_type: integration
target_boundary: generation-layouts
track: stores
wave: 3
branch: sprint/b-8-generation-layouts
worktree: sprint/b-8-generation-layouts
requirements:
  - REQ-RUN-0326
  - REQ-RUN-0329
  - REQ-RUN-0330
  - REQ-RUN-0331
  - REQ-RUN-0503
  - REQ-RUN-0313
  - REQ-RUN-0314
  - REQ-RUN-0315
  - REQ-RUN-0316
  - REQ-RUN-0317
  - REQ-RUN-0318
  - REQ-RUN-0319
  - REQ-RUN-0320
  - REQ-RUN-0327
  - REQ-RUN-0328
  - REQ-RUN-0303
  - REQ-RUN-0401
  - REQ-RUN-0403
adrs:
  - ADR-RUN-0303
  - ADR-RUN-0305
  - ADR-RUN-0306
  - ADR-RUN-0401
  - ADR-RUN-0002
owned_paths:
  - wizard/answers.schema.json
  - wizard/fixtures/
  - scripts/new_project.py
  - template/cargo-generate.toml
  - docs/generation.md
  - template/crates/api-types/
  - template/crates/cli/
  - template/crates/daemon/
  - template/crates/service/
  - tests/unit/
must_follow: [b-5, b-6, b-7]
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: Each application chooses one sqlx SQL engine, Postgres or MySQL. Both engines are layouts in this sprint so the answers schema is written once.
---

# Sprint b-8 — generation layouts

## Goal

`just new` (`scripts/new_project.py`) creates an application in exactly one layout. The layout is no database, SQLite only, one sqlx SQL engine and no SQLite, or SQLite plus one sqlx SQL engine. That sqlx engine is Postgres or MySQL. One application contains one of those engines. The Postgres-only layout and the MySQL-only layout are the one-engine layouts. The SQLite-plus-Postgres layout and the SQLite-plus-MySQL layout are the SQLite-plus-one-engine layouts. For a SQLite-plus-Postgres layout, SQLite and Postgres meet in an adaptor crate generated into the consumer repository. For a SQLite-plus-MySQL layout, SQLite and MySQL meet in an adaptor crate generated the same way. A one-store layout renders that store and no adaptor. A no-database layout renders no store crate and no adaptor.

The library crate `sc-runtime` does not generate any of this.

## Deliverables

1. REQ-RUN-0326: a new application is created with no database, with SQLite only, with one sqlx SQL engine only, or with SQLite plus one sqlx SQL engine. The engine is Postgres or MySQL, and the application contains that one engine. No application contains both Postgres and MySQL. `wizard/answers.schema.json` `db` enum is `none`, `sqlite`, `postgres`, `mysql`, `sqlite-postgres`, `sqlite-mysql`. `both` is removed and is not an alias. `x-v01-reserved` is removed. `x-schema-version` becomes 2. `template/cargo-generate.toml` `choices` lists those six strings.
2. REQ-RUN-0317 and REQ-RUN-0318: `just new` writes the stores, the models, and the adaptor. `crates/sc-runtime` contains no generator.
3. REQ-RUN-0329: every generated package name is the project name, a hyphen, and its role. That includes `store-sqlite`, `store-postgres`, `store-mysql`, `store-adaptor`, `schema`, `api-types`, `service`, `daemon`, and `cli`. The template workspace keeps the role as the package name until this sprint's render, which is why `b-4`, `b-5`, and `b-6` build `-p store-sqlite`, `-p store-postgres`, and `-p store-mysql`. A generated project uses `<project>-store-sqlite` and the other `<project>-<role>` names. REQ-RUN-0308 success criterion 1 names `<project>-store-sqlite`.
4. REQ-RUN-0330: the rendered project contains `<project>-schema`, the row structs are in that crate, and the adaptor depends on it. `template/crates/service/src/lib.rs` constructs `OpenConfig { url }` and imports the row struct from the schema crate.
5. REQ-RUN-0331 and ADR-RUN-0306: a layout with a store renders general database commands on the CLI, MCP, and HTTP, including `delete row <table> <id>`. An application command calls the same delete operation through its own CLI command, MCP tool, and HTTP route. A flag removes the general commands from those three surfaces. A no-database layout renders none of them.
6. REQ-RUN-0303, REQ-RUN-0401, REQ-RUN-0403, and ADR-RUN-0401 already name the six `db` values. This sprint makes `wizard/answers.schema.json` and the driver match that text. `x-schema-version` is 2. `both` fails schema validation. `GEN.DB_RESERVED` is not a failure this sprint ships. A `db` value outside the six strings fails as `GEN.SCHEMA_INVALID`.
7. REQ-RUN-0503: `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` exits 0. With no file added, changed, or deleted, `just setup`, then `just lint`, then `just test`, then `just build-cli`, then `just build-daemon` exit 0. `just setup` installs the sc-lint consumer and is not a source edit.
8. ADR-RUN-0002: sc-runtime is a cargo-generate template plus published library crates. `just new` renders that template into the new repository. The library crate `sc-runtime` does not contain the generator.
9. REQ-RUN-0313, REQ-RUN-0314, REQ-RUN-0315, REQ-RUN-0316, REQ-RUN-0319, REQ-RUN-0320, REQ-RUN-0327, and REQ-RUN-0328, on a generated project: a two-store layout's adaptor names both stores and depends on those `store-*` crates; the copy pattern creates the shared remote tables in SQLite and the same tables on the remote engine; the host-aggregate pattern is one SQLite database for this host and a Postgres database that aggregates many hosts, and the same shape uses MySQL when MySQL is the layout's engine; a consumer-owned rollup function is present; generation input can name a sqlite-only table, a remote-only table, and a one-sided column; two projects that select the same pattern contain the same adaptor implementation; both rendered stores implement `EngineStore`.

## Acceptance criteria

Root: `req:REQ-RUN-0326`, `req:REQ-RUN-0329`, `req:REQ-RUN-0330`, `req:REQ-RUN-0331`, `req:REQ-RUN-0503`.

- `just new` for each of the six layouts exits 0, and `SQLX_OFFLINE=true cargo check --workspace` in that project exits 0 with no live database.
- The sqlite-mcp fixture then passes `just setup`, `just lint`, `just test`, `just build-cli`, and `just build-daemon` with no file added, changed, or deleted.
- A project named `acme` has packages `acme-types`, `acme-schema`, `acme-service`, `acme-daemon`, and `acme-cli`. A layout that renders a store also has `acme-store-sqlite`, `acme-store-postgres`, or `acme-store-mysql` for that store, and a two-store layout also has `acme-store-adaptor`. The row structs are in `acme-schema`. The adaptor depends on `acme-schema` and on the store crates whose rows it moves.
- `template/crates/service` constructs `OpenConfig { url }` and imports the row struct from the schema crate.
- On a layout with a store, `delete row <table> <id>` is registered on the CLI, on MCP, and over HTTP. The disable flag removes all three. One application command calls that same delete operation on all three surfaces. A no-database layout registers none of them.
- A two-store layout's adaptor names both stores. The copy pattern creates the shared remote tables in SQLite. The host-aggregate pattern is one SQLite database for this host and a Postgres database that aggregates many hosts. A consumer-owned rollup function is present. Both rendered stores implement `EngineStore`.

## Required validation

- `just new` for each of the six layouts exits 0, and `SQLX_OFFLINE=true cargo check --workspace` in that project exits 0.
- `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` exits 0. With no file added, changed, or deleted, `just setup`, then `just lint`, then `just test`, then `just build-cli`, then `just build-daemon` exit 0.
- The SQLite-only project contains `store-sqlite` and no adaptor.
- The Postgres-only project contains `store-postgres` and no `store-sqlite`.
- The MySQL-only project contains `store-mysql` and no `store-sqlite`.
- Each two-store project contains SQLite, the store for its one sqlx engine, and the adaptor. No project contains both `store-postgres` and `store-mysql`.
- The no-database project contains no `store-*` crate.
- `rg` of a generator under `crates/sc-runtime` prints nothing.
- A project named `acme` has packages `acme-types` and `acme-schema`. The row structs are in `acme-schema`. The adaptor depends on `acme-schema`. A sqlite-only `acme` project also has package `acme-store-sqlite`.
- On a store layout, `delete row <table> <id>` is registered on the CLI, on MCP, and over HTTP; the disable flag removes all three; one application command calls that delete on all three. A no-database layout registers none of them.
- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' template/crates/api-types template/crates/cli template/crates/daemon template/crates/service` exits 1.

## Non-goals

No SQL Server layout. No new SQL inside the store crates. No edit to `sc-cron` or the rusqlite writer.
