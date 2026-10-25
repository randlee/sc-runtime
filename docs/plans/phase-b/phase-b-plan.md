# Phase b — stores, adaptor, writer, cron, and lint

## Phase id

`b`. The owner named this phase b. That is an exception to the letter-pair default in `.claude/project/quality-policy.md`, consistent with the grandfathered phase id `a`.

Entry state is the workspace at tag `v0.1.0`: four library crates, template `store-sqlite` on sqlx 0.9 with no write actor, `just lint` running boundary, line-counts, fmt, and clippy. This phase does not retag `v0.1.0` and does not publish.

## Objective

Plan and build the storage phase as one phase. The layouts, the schema, the trait, the adaptor, the writer, cron, and the lint profile are all in this plan because each one changes the others.

`just new` creates an application with no database, SQLite only, Postgres only, MySQL only, SQLite plus Postgres, or SQLite plus MySQL. Paired stores use the same schema or a similar schema, and one shared trait or a trait per store. Each engine's SQL stays in that engine's crate. The adaptor is generated for a two-store layout. `store-sqlite` stays the sqlx store it is. The rusqlite writer is a separate building block. `sc-cron` is the scheduler the application uses. The lint profile is the one every crate in this phase runs.

## Boundary map

| Boundary | Sprint | Owns | Depends on |
|---|---|---|---|
| `sc-runtime-db-sqlite` | `b-1` | the rusqlite writer crate and its manifest | none |
| host lint profile | `b-2` | `sc-lint.toml`, the hot-path grep, the xwin install declaration | none |
| `sc-cron` | `b-3` | the cron crate and its manifest | `b-1`, for `Cargo.toml` and the boundary manifests |
| store pairing | `b-4` | the schema rule and the trait rule, applied to `store-sqlite` and prepared for Postgres and MySQL | `b-2`, because `b-2` lints `template/` |
| `store-postgres` | `b-5` | the Postgres store crate in the template | `b-4` |
| `store-mysql` | `b-6` | the MySQL store crate in the template | `b-4` |
| store adaptor | `b-7` | the copy adaptor and the host-aggregate adaptor | `b-5` and `b-6` |
| generation layouts | `b-8` | `just new` selecting one of the six layouts | `b-7` |

## Sprint index

| Sprint | Doc | Closure | Track | Wave |
|---|---|---|---|---|
| `b-1` | [sprint-b-1-sqlite-writer.md](sprint-b-1-sqlite-writer.md) | `boundary` | writer | 1 |
| `b-2` | [sprint-b-2-lint-profile.md](sprint-b-2-lint-profile.md) | `boundary` | lint | 1 |
| `b-3` | [sprint-b-3-sc-cron.md](sprint-b-3-sc-cron.md) | `boundary` | cron | 2 |
| `b-4` | [sprint-b-4-store-pairing.md](sprint-b-4-store-pairing.md) | `boundary` | stores | 2 |
| `b-5` | [sprint-b-5-store-postgres.md](sprint-b-5-store-postgres.md) | `boundary` | stores | 3 |
| `b-6` | [sprint-b-6-store-mysql.md](sprint-b-6-store-mysql.md) | `boundary` | stores | 3 |
| `b-7` | [sprint-b-7-store-adaptor.md](sprint-b-7-store-adaptor.md) | `boundary` | stores | 4 |
| `b-8` | [sprint-b-8-generation-layouts.md](sprint-b-8-generation-layouts.md) | `boundary` | stores | 5 |

## Wave table

| Wave | Sprints | Critical path | Width | Sprint count |
|---|---|---|---|---|
| 1 | `b-1`, `b-2` | 1 | 2 | 8 |
| 2 | `b-3`, `b-4` | 2 | 2 | 8 |
| 3 | `b-5`, `b-6` | 3 | 2 | 8 |
| 4 | `b-7` | 4 | 1 | 8 |
| 5 | `b-8` | 5 | 1 | 8 |

`b-5` and `b-6` are the same wave so Postgres and MySQL are built against one pairing, not one after the other has hardened a two-engine assumption.

## Requirements and ADRs

- [REQ-RUN-0001](../../requirements.md), [REQ-RUN-0004](../../requirements.md), [REQ-RUN-0005](../../requirements.md), [REQ-RUN-0308](../../requirements.md), [REQ-RUN-0313](../../requirements.md) through [REQ-RUN-0328](../../requirements.md), [NFR-RUN-0002](../../requirements.md), [NFR-RUN-0005](../../requirements.md), [NFR-RUN-0008](../../requirements.md), [NFR-RUN-0009](../../requirements.md).
- [ADR-RUN-0003](../../architecture.md), [ADR-RUN-0005](../../architecture.md), [ADR-RUN-0006](../../architecture.md), [ADR-RUN-0011](../../architecture.md), [ADR-RUN-0012](../../architecture.md), [ADR-RUN-0013](../../architecture.md), [ADR-RUN-0301](../../architecture.md), [ADR-RUN-0305](../../architecture.md).
- Acceptance for the writer contract roots at [ADR-RUN-0012](../../architecture.md) and [docs/sc-runtime-db-sqlite/architecture.md](../../sc-runtime-db-sqlite/architecture.md).

REQ-RUN-0313 through REQ-RUN-0328 and ADR-RUN-0305 are recorded on `plan/runtime-anchors`.

## Out of this phase

- Implementing sc-lint-runtime rules or raising `minimum_version`. That work is [sc-lint#640](https://github.com/randlee/sc-lint/issues/640).
- SQL Server. sqlx has no driver for it.
- A phase-wide checkpoint. Each sprint closes on its own boundary.

The old narrative at `docs/plan/phase-B/phase-B-plan.md` is a pointer to this file.
