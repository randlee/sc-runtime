# Phase b — stores, adaptor, writer, cron, and lint

## Phase id

`b`. The owner named this phase b. That is an exception to the letter-pair default in `.claude/project/quality-policy.md`, consistent with the grandfathered phase id `a`.

Entry state is the workspace at tag `v0.1.0`: four library crates, template `store-sqlite` on sqlx 0.9 with no write actor, `just lint` running boundary, line-counts, fmt, and clippy. This phase does not retag `v0.1.0` and does not publish.

## Objective

Plan and build the storage phase as one phase. One chain is the rusqlite writer, then the sc-cron port, then Monty hooks. The other chain is the lint profile and the store pairing in wave 1, the two engine crates and the adaptor in wave 2, and the generation integration sprint in wave 3. `b-4` stays parallel with `b-3`. `b-8` stays parallel with `b-9`.

Waves 1–3 close on `integrate/phase-b`. Waves 4–5 are the Monty host bridge and the benchmark publisher on `integrate/phase-b-monty`, cut from `integrate/phase-b` and run in parallel with phase-end review. They do not gate that review. Coordination is [phase-b-integration-close.md](phase-b-integration-close.md).

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
| `sc-cron` (Monty hooks) | `b-9` | `HookTarget::Monty`, feature `monty`, schedule-table schema 4, and the sc-cron doc amendments | `b-3` (merged on `develop`) |
| `sc-cron` (Monty host bridge) | `b-10` | `render_template` resumed inside `monty_run::execute_feed`; `sc-compose` on `PATH` | `b-9`, for `HookTarget::Monty` and `execute_feed`. Branch `integrate/phase-b-monty` |
| benchmark platform and Monty reports | `b-11` | benchmark-run skill, multi-subsystem layout (`scripts/benchmark/<subsystem>/`, `site/reports/benchmark-<subsystem>/`), HTML code examples, publish-preflight `benchmark-pass` gate, first subsystem `monty` with campaign case `production_bridge` | `b-10`, for the ext fn `render_template` |

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
| `b-10` | [sprint-b-10-sc-cron-monty-host-bridge.md](sprint-b-10-sc-cron-monty-host-bridge.md) | `boundary` | cron-monty | 4 |
| `b-11` | [sprint-b-11-benchmark-monty-reports.md](sprint-b-11-benchmark-monty-reports.md) | `integration` | cron-monty | 5 |

## Wave table

| Wave | Sprints | Branch |
|---|---|---|
| 1 | `b-1`, `b-2`, `b-4` | landed on `develop` |
| 2 | `b-3`, `b-5`, `b-6`, `b-7` | landed on `develop` |
| 3 | `b-8`, `b-9` | landed on `develop` |
| 4 | `b-10` | `integrate/phase-b-monty` |
| 5 | `b-11` | `integrate/phase-b-monty` |

Phase-close critical path 3 (`b-4` → `b-5` → `b-8`, and `b-1` → `b-3` → `b-9`). Width 4 (wave 2). Sprint count for that close: 9 (merged to `develop`).

b-11 also owns the **publish preflight gate**: `release/publish-preflight.toml` runs `benchmark-gate` for every registered benchmark subsystem, so a binary publish requires a passing benchmark (this replaces the earlier "campaign is manual only" rule; publishing a report to `site/reports/` stays an operator step). Campaign and index HTML must carry code examples (hook snippet, `feed_start`/resume loop, `sc-compose` argv), not only timing tables. Details: [sprint-b-11-benchmark-monty-reports.md](sprint-b-11-benchmark-monty-reports.md).

Monty extension critical path 2 (`b-10` → `b-11`). Width 1. Combined sprint count 11.

`b-10` and `b-11` are not one wave. Their `owned_paths` are disjoint, so the `must_follow` edge is not a shared file. `b-11` consumes `render_template` from `b-10`. A lab-only publisher in wave 4 would be a thin sprint and would not remove that edge. The cron chain `b-1` → `b-3` → `b-9` → `b-10` → `b-11` is length 5 because waves 4–5 sit after the Monty target. Those waves are not on the phase-close critical path. [phase-b-integration-close.md](phase-b-integration-close.md) records that split.

`b-5` and `b-6` are the same wave so Postgres and MySQL are built against one pairing, not one after the other has hardened a two-engine assumption. `b-7` is in that wave because its `Cargo.toml` depends only on the schema crate. It calls `read_rows` and `write_rows` on `EngineStore` doubles and does not wait for either engine crate. `b-8` renders the adaptor's edges to the concrete stores that layout contains.

`b-3` waits on beads-cache sprint `b-8` before it copies `crates/sc-cron`. That sprint lands the per-row hook target and removes `StopDecision::Stop`. `b-4` stays parallel with `b-3` and does not wait. beads-cache sprint `b-9` is comments, provenance, and tests, with no version bump, and does not hold the port.

## Requirements and ADRs

- [REQ-RUN-0001](../../requirements.md), [REQ-RUN-0004](../../requirements.md), [REQ-RUN-0005](../../requirements.md), [REQ-RUN-0201](../../requirements.md), [REQ-RUN-0301](../../requirements.md), [REQ-RUN-0302](../../requirements.md), [REQ-RUN-0303](../../requirements.md), [REQ-RUN-0304](../../requirements.md), [REQ-RUN-0306](../../requirements.md), [REQ-RUN-0307](../../requirements.md), [REQ-RUN-0308](../../requirements.md), [REQ-RUN-0313](../../requirements.md) through [REQ-RUN-0331](../../requirements.md), [REQ-RUN-0401](../../requirements.md), [REQ-RUN-0402](../../requirements.md), [REQ-RUN-0403](../../requirements.md), [REQ-RUN-0501](../../requirements.md), [REQ-RUN-0503](../../requirements.md), [REQ-RUN-0701](../../requirements.md), [NFR-RUN-0001](../../requirements.md), [NFR-RUN-0002](../../requirements.md), [NFR-RUN-0004](../../requirements.md), [NFR-RUN-0005](../../requirements.md), [NFR-RUN-0006](../../requirements.md), [NFR-RUN-0008](../../requirements.md), [NFR-RUN-0009](../../requirements.md).
- [ADR-RUN-0001](../../architecture.md), [ADR-RUN-0002](../../architecture.md), [ADR-RUN-0003](../../architecture.md), [ADR-RUN-0004](../../architecture.md), [ADR-RUN-0005](../../architecture.md), [ADR-RUN-0006](../../architecture.md), [ADR-RUN-0007](../../architecture.md), [ADR-RUN-0008](../../architecture.md), [ADR-RUN-0011](../../architecture.md), [ADR-RUN-0012](../../architecture.md), [ADR-RUN-0013](../../architecture.md), [ADR-RUN-0014](../../architecture.md), [ADR-RUN-0015](../../architecture.md), [ADR-RUN-0201](../../architecture.md), [ADR-RUN-0202](../../architecture.md), [ADR-RUN-0301](../../architecture.md), [ADR-RUN-0302](../../architecture.md), [ADR-RUN-0303](../../architecture.md), [ADR-RUN-0305](../../architecture.md), [ADR-RUN-0306](../../architecture.md), [ADR-RUN-0401](../../architecture.md), [ADR-RUN-0402](../../architecture.md).
- Acceptance for the writer contract roots at [ADR-RUN-0012](../../architecture.md) and [docs/sc-runtime-db-sqlite/architecture.md](../../sc-runtime-db-sqlite/architecture.md).

REQ-RUN-0313 through REQ-RUN-0331 and ADR-RUN-0305 are in [requirements.md](../../requirements.md) and [architecture.md](../../architecture.md).

`b-9` and `b-10` are governed by [REQ-CRON-0001](../../sc-cron/requirements.md) through [REQ-CRON-0005](../../sc-cron/requirements.md) and [ADR-RUN-0013](../../architecture.md). `b-11` is governed by REQ-CRON-0001, REQ-CRON-0003, REQ-CRON-0005, REQ-RUN-0005, and NFR-RUN-0005, plus [ADR-RUN-0015](../../architecture.md) (not REQ-CRON-0002 or REQ-CRON-0004). `REQ-CRON-0003` on `develop` already carries the Monty sentence (`b-9` landed). The `render_template` sentence is a `b-10` deliverable.

## Planned amendment: schedule row limit (stub)

Plan-only until an implementation sprint is scheduled by the owner. Not assigned to `b-9` (landed), `b-10`, or `b-11` (frozen paths and non-goals). Binding ADR: [ADR-RUN-0014](../../architecture.md) (Active, owner 2026-10-27). Source: [decision-sync-scheduling-at-api.md](decision-sync-scheduling-at-api.md#owner-ruling-embedder-supplied-row-limit-2026-10-27).

| Field | Value |
|---|---|
| Boundary | `sc-cron` (`crates/sc-cron`, `docs/sc-cron`, ADR-RUN-0013 / ADR-RUN-0014) |
| Amends | `b-3` / `b-9` scheduler surface; `REQ-CRON-0003` / `REQ-CRON-0004` text gains the limit sentence |
| Behavior | optional embedder-supplied limit on schedule rows (enabled + disabled); app default **10** from sc-config JSON; **`0` or `null`** → unlimited; sc-cron never reads sc-config; library default is unlimited |
| Enforcement | in the insert transaction; over-limit new name → existing `CronError::InvalidSpec` (no new variant); replace does not change the count; lowering the limit never deletes rows |
| Config reload | **v1:** limit at scheduler open (daemon startup); **v2 (optional):** reconfiguration API on config reload |
| Not included | any BDC sync or lease logic (beads-cache only); an sc-config dependency; a global in-flight lock |
| Open | sprint id and wave; final type and constructor names for the limit parameter |

## Out of this phase

- Implementing sc-lint-runtime rules or raising `minimum_version`. That work is [sc-lint#640](https://github.com/randlee/sc-lint/issues/640).
- SQL Server. sqlx has no driver for it.
- A checkpoint that re-runs every earlier sprint. `b-8` is the generation integration sprint for the six layouts.
- Treating `b-11` as a second phase-wide integration checkpoint. It closes the Monty campaign only. `b-8` still closes the six layouts.

The old narrative at `docs/plan/phase-B/phase-B-plan.md` is a pointer to this file.
