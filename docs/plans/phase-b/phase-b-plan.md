---
requirements:
  - REQ-RUN-0001
  - REQ-RUN-0002
  - REQ-RUN-0003
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0201
  - REQ-RUN-0301
  - REQ-RUN-0302
  - REQ-RUN-0303
  - REQ-RUN-0308
  - REQ-RUN-0309
  - REQ-RUN-0313
  - REQ-RUN-0314
  - REQ-RUN-0315
  - REQ-RUN-0316
  - REQ-RUN-0317
  - REQ-RUN-0318
  - REQ-RUN-0319
  - REQ-RUN-0320
  - REQ-RUN-0321
  - REQ-RUN-0322
  - REQ-RUN-0323
  - REQ-RUN-0324
  - REQ-RUN-0325
  - REQ-RUN-0326
  - REQ-RUN-0327
  - REQ-RUN-0328
  - REQ-RUN-0329
  - REQ-RUN-0330
  - REQ-RUN-0331
  - REQ-RUN-0401
  - REQ-RUN-0402
  - REQ-RUN-0403
  - REQ-RUN-0503
  - REQ-RUN-0701
  - REQ-RUN-0702
  - NFR-RUN-0001
  - NFR-RUN-0002
  - NFR-RUN-0005
  - NFR-RUN-0008
  - NFR-RUN-0009
  - NFR-RUN-0010
  - REQ-RT-0001
  - REQ-RT-0004
  - REQ-RT-0005
  - NFR-RT-0001
  - NFR-RT-0002
  - NFR-RT-0003
  - NFR-RT-0004
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0005
  - NFR-CRON-0001
---

# Phase b — stores, adaptor, writer, cron, and lint

## Phase id

`b`. The owner named this phase b. That is an exception to the letter-pair default in `.claude/project/quality-policy.md`, consistent with the grandfathered phase id `a`.

Entry state is the workspace at tag `v0.1.0`: four library crates, template `store-sqlite` on sqlx 0.9 with no write actor, `just lint` running boundary, line-counts, fmt, and clippy. The original b-1 through b-12 cut does not retag `v0.1.0` or publish; the continuation establishes and releases the new `0.1.1` public contract under b-13/b-17.

## Objective

Plan and build the storage phase as one phase. One chain is the rusqlite writer, then the sc-cron port, then Monty hooks. The other chain is the lint profile and the store pairing in wave 1, the two engine crates and the adaptor in wave 2, and the generation integration sprint in wave 3. `b-4` stays parallel with `b-3`. `b-8` stays parallel with `b-9`.

Waves 1–3 close on `integrate/phase-b`. Waves 4–5 are the Monty host bridge and the benchmark publisher on `integrate/phase-b-monty`, cut from `integrate/phase-b` and run in parallel with phase-end review. They do not gate that review. Coordination is [phase-b-integration-close.md](phase-b-integration-close.md).

Sprints b-13 through b-17 are a Phase B continuation amendment, not a new
phase. PR #84 must merge b-10 through b-12 before they begin, and none of that
closed scope is replanned. Both continuation tracks integrate on
`integrate/phase-b-observability`; their sprint branches remain parallel and
disjoint, and b-16 merges before b-17 consumes its feature.
This amendment is authored on `plan/spinoff-monty-observability`; sprint
dispatch uses the branches declared in each sprint frontmatter.

The owner selected continuation numbering after reviewing the phase policy.
That is consistent with Phase B's already-grandfathered single-letter id and
with the source outline's b-13-forward handoff. The amendment adds only the
residual product/injection surfaces proven absent at the current integration
head; it does not relabel or materially expand b-10 through b-12.

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
| `sc-cron` (Monty host bridge) | `b-10` | `render_template` resumed inside `monty_run::drive_feed` / `execute_feed_with_compose_program`; default `sc-compose` on `PATH` | `b-9`, for `HookTarget::Monty`. Branch `integrate/phase-b-monty` |
| benchmark platform and Monty reports | `b-11` | benchmark-run skill, multi-subsystem layout (`scripts/benchmark/<subsystem>/`, `site/reports/benchmark-<subsystem>/`), HTML code examples, publish-preflight `benchmark-pass` gate, first subsystem `monty` with campaign case `production_bridge` | `b-10`, for the ext fn `render_template` |
| `sc-cron` (schedule row limit) | `b-12` | optional embedder `max_schedule_rows`, template sc-config default 10, ADR-RUN-0014 implementation | `b-9` (on `develop`; parallel with Monty track) |
| injected observability contract | `b-13` | upstream non-blocking types-only contract adoption, ADR/REQ amendments, boundary allowlists, public injection signatures, Noop defaults, whole jitter/store/process/Monty component propagation and events, workspace `0.1.1` and publishable `sc-cron` | matching published observability `1.5.1` patches; PR #84 merge is an external phase-entry gate |
| `sc-runtime` canonical events | `b-14` | HTTP request lifecycle events through the injected handle | `b-13` |
| `sc-cron` canonical events | `b-15` | due-run/worker event taxonomy and residual stable failure tokens | `b-13`, which owns scheduler and component logger seams plus component-owned events |
| generated Monty product surface | `b-16` | both template dependency inventories, daemon Cargo `cron`/`monty` features, checked trusted-source hook example, product documentation | `b-10` host artifact and b-13 graph/REQ amendments; parallel with b-14 and b-15 |
| generated observability composition and release proof | `b-17` | one Arc-owned logger injected into runtime and real scheduler; HTTP+cron JSONL; exact pinned-kit install and complete installed-output ownership; sc-publish `0.1.1` order/preflight and registry-only release build | `b-14`, `b-15`, `b-16`; human release authorization and credentials |

### Continuation boundary edges

| Boundary | New allowed edge / contract | Edge that remains forbidden | Owner |
|---|---|---|---|
| upstream observability crates | published `CanonicalLog`, `NoopCanonicalLog`, `SharedCanonicalLog`; `CanonicalLog for v2::Logger` | no upstream source path is owned here | external entry prerequisite |
| `BOUNDARY-sc-runtime` | direct `sc-observability-types`; `DaemonBuilder::logger(SharedCanonicalLog)` with fixed `sc-runtime` Noop | `sc-observability`, OTLP, config, and cron | b-13 contract; b-14 event implementation |
| `BOUNDARY-sc-cron` | direct `sc-observability-types`; `SqliteScheduleStore::open_with_logger`, the sole coherent `Scheduler::open_with_logger` that internally constructs logger-aware `SystemJitter`, `supervise_with_logger`, logger-aware Python hook API, and internal Monty propagation, all with source-compatible Noop legacy entry points | full observability runtime and every other workspace library | b-13 contract/component/hook API; b-15 worker call site and worker/due-run events |
| generated daemon | full logger in one composition root; Arc coercion and injection to both libraries | libraries do not depend back on daemon | b-16 owns both template Cargo inventories; b-17 owns wiring and registry-release proof |
| generated Monty example | opt-in `monty` forwards trusted embedding-chosen in-process source to `sc-cron/monty`; allowed render-only hosts | untrusted source, any sandbox claim, root selection, arbitrary compose subcommands, or new hosts; host allowlisting does not remove abort risk | b-16 |

At the planning baseline, both library boundary manifests allow `tracing` and
neither allows `sc-observability-types`; their test-double lists are empty.
b-13 therefore consumes the upstream Noop/recording contract instead of
inventing repository-specific fake paths. Shared tracing dependency cleanup
is deferred; direct owned production calls are removed across b-13, b-14,
and b-15.

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
| `b-12` | [sprint-b-12-sc-cron-schedule-row-limit.md](sprint-b-12-sc-cron-schedule-row-limit.md) | `boundary` | cron | 6 |
| `b-13` | [sprint-b-13-observability-contract.md](sprint-b-13-observability-contract.md) | `contract` | observability | 7 |
| `b-14` | [sprint-b-14-sc-runtime-observability.md](sprint-b-14-sc-runtime-observability.md) | `boundary` | observability | 8 |
| `b-15` | [sprint-b-15-sc-cron-observability.md](sprint-b-15-sc-cron-observability.md) | `boundary` | observability | 8 |
| `b-16` | [sprint-b-16-monty-template-examples.md](sprint-b-16-monty-template-examples.md) | `integration` | monty-product | 8 |
| `b-17` | [sprint-b-17-template-observability-integration.md](sprint-b-17-template-observability-integration.md) | `integration` | observability | 9 |

### Continuation dispatch projection

This compact projection is for scope review; each linked sprint's frontmatter
is authoritative for its exact `owned_paths`, relations, branch, and model.

| Sprint | Track / target boundary | Ownership class | Consumed artifact |
|---|---|---|---|
| b-13 | observability / `observability-injection` | shared contracts, version/manifests, signatures, whole jitter/store/process/hook/Monty components | non-blocking published types/runtime `1.5.1` pair |
| b-14 | observability / `sc-runtime` | runtime HTTP event sites and tests | `SharedCanonicalLog`, `DaemonBuilder::logger` |
| b-15 | observability / `sc-cron` | cron worker and due-run event sites and tests | `SharedCanonicalLog` in `RunCoord` plus b-13 component seams |
| b-16 | monty-product / `generated-daemon-monty` | checkout/registry template Cargo inventories, trusted-source example, docs, render test | b-10 `render_template`/render-only `sc_compose_run` plus b-13 graph authority |
| b-17 | observability / `generated-daemon-observability` | generated composition, smoke test, sc-publish pin/input plus every exact pinned-kit installed output | b-14 events, b-15 events, b-16 Cargo features and exact `0.1.1` pins; a later qualifying kit revision with no `docs/sprints/**` |

Static review must parse the five frontmatter blocks, reject duplicate exact
owned paths inside a wave, and verify every `.sprints/b/structure.ttl`
criteria path exists. This checks the authoritative source without creating a
second path inventory.

## Wave table

| Wave | Sprints | Branch |
|---|---|---|
| 1 | `b-1`, `b-2`, `b-4` | landed on `develop` |
| 2 | `b-3`, `b-5`, `b-6`, `b-7` | landed on `develop` |
| 3 | `b-8`, `b-9` | landed on `develop` |
| 4 | `b-10` | `integrate/phase-b-monty` |
| 5 | `b-11` | `integrate/phase-b-monty` |
| 6 | `b-12` | `integrate/phase-b-monty` (after `b-9` on `develop`; parallel with waves 4–5) |
| 7 | `b-13` | `integrate/phase-b-observability` (shared contract and graph authority) |
| 8 | `b-14`, `b-15`, `b-16` | `integrate/phase-b-observability` (parallel runtime, cron call-site, and generated-product boundaries) |
| 9 | `b-17` | `integrate/phase-b-observability` after b-16 merges forward |

Phase-close critical path 3 (`b-4` → `b-5` → `b-8`, and `b-1` → `b-3` → `b-9`). Width 4 (wave 2). Sprint count for that close: 9 (merged to `develop`).

b-11 also owns the **publish preflight gate**: `release/publish-preflight.toml` runs `benchmark-gate` for every registered benchmark subsystem, so a binary publish requires a passing benchmark (this replaces the earlier "campaign is manual only" rule; publishing a report to `site/reports/` stays an operator step). Campaign and index HTML must carry code examples (hook snippet, `feed_start`/resume loop, `sc-compose` argv), not only timing tables. Details: [sprint-b-11-benchmark-monty-reports.md](sprint-b-11-benchmark-monty-reports.md).

Monty extension critical path 2 (`b-10` → `b-11`). Width 1. Row-limit path 1 (`b-12` on `integrate/phase-b-monty`, parallel with Monty). Phase-end on `integrate/phase-b-monty`, then merge to `develop`. Combined sprint count 12.

Post-Track-B continuation critical path is 3
(`b-13` -> `b-14`, `b-15`, or `b-16` -> `b-17`).
Continuation width is 3 because b-14, b-15, and b-16 are disjoint siblings
in wave 8; continuation sprint count is 5, and the combined
Phase B sprint count is 17.

Continuation ownership is disjoint. b-13 exclusively owns shared ADR/REQ,
boundary, lockfile, library manifests, public injection signatures, and the
whole jitter/store/process/hook/Monty component files and their events/API.
b-14 owns runtime event sites; b-15 owns only worker and due-run trace sites
and the worker call passing the handle into b-13's hook API. b-16 exclusively
owns both template dependency inventories, daemon Cargo, and the Monty
example. b-17 consumes those artifacts and owns generated composition, the
integration smoke, the sc-publish consumer input, and every exact output of a
real install from the pinned kit revision for the `0.1.1` release. Thus b-17
`must_follow` b-14, b-15, and b-16; b-14 and
b-15 and b-16 all `must_follow` b-13; b-16 also names b-10's host artifact.

The current sc-publish pin is not a valid b-17 input because its installer
package includes upstream sprint history under `docs/sprints/**`. b-17 is
blocked until an exact later qualifying revision exists; it then owns the pin
advance and proves the installed inventory exactly matches its explicit
installed-output paths. This plan does not invent the future revision.

`b-10` and `b-11` are not one wave. Their `owned_paths` are disjoint, so the `must_follow` edge is not a shared file. `b-11` consumes `render_template` from `b-10`. A lab-only publisher in wave 4 would be a thin sprint and would not remove that edge. The cron chain `b-1` → `b-3` → `b-9` → `b-10` → `b-11` is length 5 because waves 4–5 sit after the Monty target. Those waves are not on the phase-close critical path. [phase-b-integration-close.md](phase-b-integration-close.md) records that split.

`b-5` and `b-6` are the same wave so Postgres and MySQL are built against one pairing, not one after the other has hardened a two-engine assumption. `b-7` is in that wave because its `Cargo.toml` depends only on the schema crate. It calls `read_rows` and `write_rows` on `EngineStore` doubles and does not wait for either engine crate. `b-8` renders the adaptor's edges to the concrete stores that layout contains.

`b-3` waits on beads-cache sprint `b-8` before it copies `crates/sc-cron`. That sprint lands the per-row hook target and removes `StopDecision::Stop`. `b-4` stays parallel with `b-3` and does not wait. beads-cache sprint `b-9` is comments, provenance, and tests, with no version bump, and does not hold the port.

## Requirements and ADRs

- [REQ-RUN-0001](../../requirements.md), [REQ-RUN-0004](../../requirements.md), [REQ-RUN-0005](../../requirements.md), [REQ-RUN-0201](../../requirements.md), [REQ-RUN-0301](../../requirements.md), [REQ-RUN-0302](../../requirements.md), [REQ-RUN-0303](../../requirements.md), [REQ-RUN-0304](../../requirements.md), [REQ-RUN-0306](../../requirements.md), [REQ-RUN-0307](../../requirements.md), [REQ-RUN-0308](../../requirements.md), [REQ-RUN-0313](../../requirements.md) through [REQ-RUN-0331](../../requirements.md), [REQ-RUN-0401](../../requirements.md), [REQ-RUN-0402](../../requirements.md), [REQ-RUN-0403](../../requirements.md), [REQ-RUN-0501](../../requirements.md), [REQ-RUN-0503](../../requirements.md), [REQ-RUN-0701](../../requirements.md), [NFR-RUN-0001](../../requirements.md), [NFR-RUN-0002](../../requirements.md), [NFR-RUN-0004](../../requirements.md), [NFR-RUN-0005](../../requirements.md), [NFR-RUN-0006](../../requirements.md), [NFR-RUN-0008](../../requirements.md), [NFR-RUN-0009](../../requirements.md).
- [ADR-RUN-0001](../../architecture.md), [ADR-RUN-0002](../../architecture.md), [ADR-RUN-0003](../../architecture.md), [ADR-RUN-0004](../../architecture.md), [ADR-RUN-0005](../../architecture.md), [ADR-RUN-0006](../../architecture.md), [ADR-RUN-0007](../../architecture.md), [ADR-RUN-0008](../../architecture.md), [ADR-RUN-0009](../../architecture.md), [ADR-RUN-0011](../../architecture.md), [ADR-RUN-0012](../../architecture.md), [ADR-RUN-0013](../../architecture.md), [ADR-RUN-0014](../../architecture.md), [ADR-RUN-0015](../../architecture.md), [ADR-RUN-0201](../../architecture.md), [ADR-RUN-0202](../../architecture.md), [ADR-RUN-0301](../../architecture.md), [ADR-RUN-0302](../../architecture.md), [ADR-RUN-0303](../../architecture.md), [ADR-RUN-0305](../../architecture.md), [ADR-RUN-0306](../../architecture.md), [ADR-RUN-0307](../../architecture.md), [ADR-RUN-0401](../../architecture.md), [ADR-RUN-0402](../../architecture.md), [ADR-RT-0001](../../sc-runtime/architecture.md), [ADR-RT-0002](../../sc-runtime/architecture.md), [ADR-RT-0003](../../sc-runtime/architecture.md), [ADR-RT-0004](../../sc-runtime/architecture.md), [ADR-TRN-0001](../../sc-transport/architecture.md), [ADR-TRN-0002](../../sc-transport/architecture.md), [ADR-TRN-0003](../../sc-transport/architecture.md), [ADR-TRN-0004](../../sc-transport/architecture.md), [ADR-TRN-0005](../../sc-transport/architecture.md), [ADR-TRN-0006](../../sc-transport/architecture.md), [ADR-CMD-0001](../../sc-command/architecture.md), [ADR-CMD-0002](../../sc-command/architecture.md), [ADR-CMD-0003](../../sc-command/architecture.md).
- Acceptance for the writer contract roots at [ADR-RUN-0012](../../architecture.md) and [docs/sc-runtime-db-sqlite/architecture.md](../../sc-runtime-db-sqlite/architecture.md).

REQ-RUN-0313 through REQ-RUN-0331 and ADR-RUN-0305 are in [requirements.md](../../requirements.md) and [architecture.md](../../architecture.md).

`b-9` and `b-10` are governed by [REQ-CRON-0001](../../sc-cron/requirements.md) through [REQ-CRON-0005](../../sc-cron/requirements.md) and [ADR-RUN-0013](../../architecture.md). `b-11` is governed by REQ-CRON-0001, REQ-CRON-0003, REQ-CRON-0005, REQ-RUN-0005, and NFR-RUN-0005, plus [ADR-RUN-0015](../../architecture.md) (not REQ-CRON-0002 or REQ-CRON-0004). **`b-12`** implements [ADR-RUN-0014](../../architecture.md) ([sprint-b-12-sc-cron-schedule-row-limit.md](sprint-b-12-sc-cron-schedule-row-limit.md)). `REQ-CRON-0003` on `develop` already carries the Monty sentence (`b-9` landed). The `render_template` sentence is a `b-10` deliverable.

`b-13` through `b-17` are governed by REQ-RUN-0002 through REQ-RUN-0005,
REQ-RUN-0309, REQ-RUN-0503, REQ-RUN-0701, REQ-RUN-0702, REQ-RT-0001,
REQ-RT-0004, REQ-RT-0005, NFR-RT-0001 through
NFR-RT-0004, REQ-CRON-0001, REQ-CRON-0003, REQ-CRON-0005,
NFR-CRON-0001, NFR-RUN-0002, NFR-RUN-0005, NFR-RUN-0008, NFR-RUN-0009,
and NFR-RUN-0010. Their decision authority is ADR-RUN-0003, ADR-RUN-0004,
ADR-RUN-0007, ADR-RUN-0011, ADR-RUN-0013, ADR-RUN-0014, ADR-RUN-0303,
ADR-CMD-0001, ADR-RT-0001, ADR-RT-0002, and ADR-TRN-0006. ADR-RT-0002 and
ADR-TRN-0006 apply narrowly to b-17's proof that real signal shutdown releases
the instance lock and endpoint lock/socket. ADR-RT-0003 governs transparent
Stores/router/rmcp pass-through; that decision remains unchanged and is
inapplicable to this continuation. ADR-RT-0004 is referenced narrowly by
b-14's existing real-daemon fixture regression; it is not amended or added as
phase-wide authority.
Matching published patches of `sc-observability-types` and
`sc-observability` at `1.5.1` are an entry prerequisite, not
repository-owned work. b-13 establishes but does not publish this repository's
`0.1.1` contract; b-17 closes only after the human-authorized sc-publish order
and a no-checkout-patch registry render succeed.

## Out of this phase

- Implementing sc-lint-runtime rules or raising `minimum_version`. That work is [sc-lint#640](https://github.com/randlee/sc-lint/issues/640).
- SQL Server. sqlx has no driver for it.
- A checkpoint that re-runs every earlier sprint. `b-8` is the generation integration sprint for the six layouts.
- Treating `b-11` as a second phase-wide integration checkpoint. It closes the Monty campaign only. `b-8` still closes the six layouts.
- Reopening b-10 through b-12 allowlist, render-only compose, fixed-wait,
  process-group deadline, benchmark, or row-limit work. The continuation only
  verifies those merged results.
- Owning upstream observability source/publication, default OTLP, a tracing
  bridge, new Monty hosts, or injection into every workspace crate.

## Continuation acceptance

```sh
just lint
just test
cargo test -p sc-runtime --features test-util
cargo test -p sc-cron
cargo +1.96.1 test -p sc-cron --features monty
sc-lint lint sc-boundary --root . --config sc-lint.toml
cargo tree -p sc-runtime -e normal --depth 1 --prefix none
cargo tree -p sc-cron -e normal --depth 1 --prefix none
! rg -n '#\[(tracing::)?instrument|tracing::|(trace|debug|info|warn|error|event|span|trace_span|debug_span|info_span|warn_span|error_span)!' crates/sc-cron/src
rg -n '\bsupervise_with_logger\(' crates/sc-cron/src/hook.rs
! rg -n '\bsupervise\(' crates/sc-cron/src/hook.rs
python3 -m unittest tests.unit.generation.test_monty_example
python3 -m unittest tests.integration.generation.test_observability
# Run b-17's clean-checkout pinned-kit fetch, real install, repeat dry-run,
# inventory rejection of docs/sprints/**, and clean git-drift block before
# the generated consumer preflight.
bash .github/scripts/run-consumer-preflight.sh
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_mode_uses_registry
```

The tree commands show a direct `sc-observability-types` patch and no full
runtime/OTLP dependency in either library. The generated integration test
parses one HTTP and one real scheduler event from the same JSONL file.

The old narrative at `docs/plan/phase-B/phase-B-plan.md` is a pointer to this file.
