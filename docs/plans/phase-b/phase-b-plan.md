# Phase b — stores, adaptor, writer, cron, and lint

## Phase id

`b`. The owner named this phase b. That is an exception to the letter-pair default in `.claude/project/quality-policy.md`, consistent with the grandfathered phase id `a`.

Entry state is the workspace at tag `v0.1.0`: four library crates, template `store-sqlite` on sqlx 0.9 with no write actor, `just lint` running boundary, line-counts, fmt, and clippy. This phase does not retag `v0.1.0` and does not publish.

## Objective

Plan and build the storage phase as one phase. One chain is the rusqlite writer, then the sc-cron port. The other chain is the lint profile and the store pairing in wave 1, the two engine crates and the adaptor in wave 2, and the generation integration sprint in wave 3. `b-4` stays parallel with `b-3`.

`just new` creates an application with no database, SQLite only, Postgres only, MySQL only, SQLite plus Postgres, or SQLite plus MySQL. Each generated package is named `<project>-<role>`, including the store crates and the adaptor. `<project>-schema` holds the tables, the columns, and the row structs. `<project>-types` holds non-database types. The adaptor depends on `<project>-schema`. Paired stores use the same schema or a similar schema, and one shared trait or a trait per store. Generated stores implement `EngineStore`, whose `read_rows` and `write_rows` are what the adaptor calls. Each engine's SQL stays in that engine's crate. The adaptor is generated for a two-store layout. `store-sqlite` stays the sqlx store it is. The rusqlite writer is a separate building block. `sc-cron` is the scheduler the application uses. The lint profile is the one every crate in this phase runs.

## Boundary map

| Boundary | Sprint | Owns | Depends on |
|---|---|---|---|
| `sc-runtime-db-sqlite` | `b-1` | the rusqlite writer crate, root `Cargo.toml` membership glob `crates/*`, and the host boundary manifests | none |
| host lint profile | `b-2` | `sc-lint.toml`, the hot-path grep, the xwin install declaration, and the host crates | none |
| `sc-cron` | `b-3` | `crates/sc-cron`, `boundaries/sc-cron`, and `docs/sc-cron` | `b-1`, for the `crates/*` membership glob. Also blocked on beads-cache sprint `b-8` (`sprint/b-8-tick-python`) merging |
| store pairing | `b-4` | the schema crate, the pairing trait, `store-sqlite`, and `template/Cargo.toml` | none |
| `store-postgres` | `b-5` | `template/crates/store-postgres` | `b-4`, for the `EngineStore` trait |
| `store-mysql` | `b-6` | `template/crates/store-mysql` | `b-4`, for the `EngineStore` trait |
| store adaptor | `b-7` | `template/crates/store-adaptor`, tested against `EngineStore` doubles | `b-4`, for `EngineStore` including `read_rows` and `write_rows` |
| generation layouts | `b-8` | `just new`, the six layouts, package names, the rendered store edges, and the database command surface | `b-5`, `b-6`, and `b-7`, for the store crates and the adaptor |
| `sc-cron` (Monty hooks) | `b-9` | `HookTarget::Monty`, feature `monty`, schedule-table schema 4, and the sc-cron doc amendments | `b-3`, blocked on bead `run-9ny` |

## Sprint index

| Sprint | Doc | Closure | Track | Wave |
|---|---|---|---|---|
| `b-1` | [sprint-b-1-sqlite-writer.md](sprint-b-1-sqlite-writer.md) | `boundary` | writer | 1 |
| `b-2` | [sprint-b-2-lint-profile.md](sprint-b-2-lint-profile.md) | `boundary` | lint | 1 |
| `b-3` | [sprint-b-3-sc-cron.md](sprint-b-3-sc-cron.md) | `boundary` | cron | 2 |
| `b-4` | [sprint-b-4-store-pairing.md](sprint-b-4-store-pairing.md) | `boundary` | stores | 1 |
| `b-5` | [sprint-b-5-store-postgres.md](sprint-b-5-store-postgres.md) | `boundary` | stores | 2 |
| `b-6` | [sprint-b-6-store-mysql.md](sprint-b-6-store-mysql.md) | `boundary` | stores | 2 |
| `b-7` | [sprint-b-7-store-adaptor.md](sprint-b-7-store-adaptor.md) | `boundary` | stores | 2 |
| `b-8` | [sprint-b-8-generation-layouts.md](sprint-b-8-generation-layouts.md) | `integration` | stores | 3 |
| `b-9` | [sprint-b-9-sc-cron-monty-hooks.md](sprint-b-9-sc-cron-monty-hooks.md) | `boundary` | cron | 3 |

## Wave table

| Wave | Sprints |
|---|---|
| 1 | `b-1`, `b-2`, `b-4` |
| 2 | `b-3`, `b-5`, `b-6`, `b-7` |
| 3 | `b-8`, `b-9` |

Critical path 3 on each track (`b-4` → `b-5` → `b-8` and `b-1` → `b-3` → `b-9`). Width 4. Sprint count 9. `b-8` and `b-9` are parallel in wave 3.

`b-5` and `b-6` are the same wave so Postgres and MySQL are built against one pairing, not one after the other has hardened a two-engine assumption. `b-7` is in that wave because its `Cargo.toml` depends only on the schema crate. It calls `read_rows` and `write_rows` on `EngineStore` doubles and does not wait for either engine crate. `b-8` renders the adaptor's edges to the concrete stores that layout contains.

`b-3` waits on beads-cache sprint `b-8` before it copies `crates/sc-cron`. That sprint lands the per-row hook target and removes `StopDecision::Stop`. `b-4` stays parallel with `b-3` and does not wait. beads-cache sprint `b-9` is comments, provenance, and tests, with no version bump, and does not hold the port.

## Requirements and ADRs

- [REQ-RUN-0001](../../requirements.md), [REQ-RUN-0004](../../requirements.md), [REQ-RUN-0005](../../requirements.md), [REQ-RUN-0201](../../requirements.md), [REQ-RUN-0301](../../requirements.md), [REQ-RUN-0302](../../requirements.md), [REQ-RUN-0303](../../requirements.md), [REQ-RUN-0308](../../requirements.md), [REQ-RUN-0313](../../requirements.md) through [REQ-RUN-0331](../../requirements.md), [REQ-RUN-0401](../../requirements.md), [REQ-RUN-0402](../../requirements.md), [REQ-RUN-0403](../../requirements.md), [REQ-RUN-0503](../../requirements.md), [REQ-RUN-0701](../../requirements.md), [NFR-RUN-0001](../../requirements.md), [NFR-RUN-0002](../../requirements.md), [NFR-RUN-0005](../../requirements.md), [NFR-RUN-0008](../../requirements.md), [NFR-RUN-0009](../../requirements.md).
- [ADR-RUN-0001](../../architecture.md), [ADR-RUN-0002](../../architecture.md), [ADR-RUN-0003](../../architecture.md), [ADR-RUN-0005](../../architecture.md), [ADR-RUN-0006](../../architecture.md), [ADR-RUN-0007](../../architecture.md), [ADR-RUN-0008](../../architecture.md), [ADR-RUN-0011](../../architecture.md), [ADR-RUN-0012](../../architecture.md), [ADR-RUN-0013](../../architecture.md), [ADR-RUN-0201](../../architecture.md), [ADR-RUN-0301](../../architecture.md), [ADR-RUN-0302](../../architecture.md), [ADR-RUN-0303](../../architecture.md), [ADR-RUN-0305](../../architecture.md), [ADR-RUN-0306](../../architecture.md), [ADR-RUN-0401](../../architecture.md), [ADR-RUN-0402](../../architecture.md).
- Acceptance for the writer contract roots at [ADR-RUN-0012](../../architecture.md) and [docs/sc-runtime-db-sqlite/architecture.md](../../sc-runtime-db-sqlite/architecture.md).

REQ-RUN-0313 through REQ-RUN-0331 and ADR-RUN-0305 are in [requirements.md](../../requirements.md) and [architecture.md](../../architecture.md).

## Out of this phase

- Implementing sc-lint-runtime rules or raising `minimum_version`. That work is [sc-lint#640](https://github.com/randlee/sc-lint/issues/640).
- SQL Server. sqlx has no driver for it.
- A checkpoint that re-runs every earlier sprint. `b-8` is the generation integration sprint for the six layouts.

The old narrative at `docs/plan/phase-B/phase-B-plan.md` is a pointer to this file.
