# Candidate worklist for PRD v2.0

Candidate changes to bring the code and the descriptive documents in line with [PRD v2.0](sc-runtime-prd.md). Paths and line numbers are from `develop` at `4da05e3`, found by reading and searching. Nothing here has been compiled, run or tested, and no code has been changed. Sizes are not estimated unless a line count is given. This is not an exhaustive or hardened implementation plan.

Each item says what to do, where, which PRD requirement asks for it, what it waits on, and how to tell it is done. Items marked **Owner** cannot start until the decision in section 6 is made.

## 1. Storage (PRD §4)

| Id | Change | Where | Waits on | Done when |
|---|---|---|---|---|
| STO-1 | Define `Storage<T>`: an async trait that reads and writes `T` (PRD-STO-1) | new | **Owner** O1, O2, O4 | the trait exists with the agreed methods and no SQL in its signature |
| STO-2 | Give the SQLite writer an async input (PRD-STO-3). Replace `std::sync::mpsc` and the blocking wait with a bounded `tokio::sync::mpsc` channel and an awaited oneshot reply. Use the settled rusqlite 0.41 security floor (O8 below), retaining the yield-based drain as `tokio::task::yield_now` on the dedicated worker runtime. Copy from atm-core `crates/atm-storage-rusqlite/src/writer/mod.rs` 253–340 (thread with its own current-thread runtime) and 405–458 (`submit_async`) | `crates/sc-runtime-db-sqlite/src/writer.rs` 5, 86, 102–103, 154–156, 193–199, 218; `src/batch.rs` 153–165, 479–485 | — | callers enqueue and await a oneshot without a blocking receive or `spawn_blocking`; the 1 ms yield-based drain and commit-before-reply remain. Only the dedicated SQLite worker threads may drive their own runtime with `Runtime::block_on` (ADR-RUN-0011); no Tokio request worker or caller does so |
| STO-3 | Preserve the batching behavior: 1 ms yield-based drain, `BEGIN IMMEDIATE`, savepoints, reply after commit. It already matches atm-core `writer/batch.rs` | `crates/sc-runtime-db-sqlite/src/batch.rs` 181–470 | STO-2 | the existing commit-order and batching tests pass against the async input |
| STO-4 | Add the reader pool. Copy from atm-core `crates/atm-storage-rusqlite/src/reader_pool.rs` (1,686 lines; it contains atm-specific lanes that need removing) | new, beside the writer | — | reads go through reader threads awaited over channels; no read uses the write connection |
| STO-5 | SQLite implementation of `Storage<T>` over STO-2 to STO-4. Callers pass `T`; SQL stays inside | new | STO-1 to STO-4 | a row type can be written and read back through the trait with no SQL at the call site |
| STO-6 | Postgres implementation of `Storage<T>` on sqlx (PRD-STO-2). Reuse connection and migration code; drop the per-table functions | `template/crates/store-postgres/src/lib.rs` (980 lines) | STO-1 | same test as STO-5, against Postgres |
| STO-7 | MySQL implementation of `Storage<T>` on sqlx | `template/crates/store-mysql/src/lib.rs` (1,066 lines) | STO-1 | same test as STO-5, against MySQL |
| STO-8 | Update the crate's boundary manifest, `REQ-DBSQ-0001` and `REQ-DBSQ-0002` for the new dependency and public surface | `boundaries/sc-runtime-db-sqlite/`, `docs/sc-runtime-db-sqlite/requirements.md` | STO-2, STO-5 | lint passes with the manifest matching the code |
| STO-9 | Rewrite the descriptive docs for the writer | `crates/sc-runtime-db-sqlite/README.md` 5–32 (blocking API and `spawn_blocking` advice); `docs/sc-runtime-db-sqlite/architecture.md` body | STO-5 | neither document describes a blocking handle |

## 2. Cron (PRD §6)

| Id | Change | Where | Waits on | Done when |
|---|---|---|---|---|
| CRON-1 | Define `CronRow` (PRD-CRON-2). Its fields are the 18 columns the SQLite store checks today | columns listed in `crates/sc-cron/src/sqlite_schema.rs` 29–54 | STO-1, O2 | `CronRow` exists and round-trips through `Storage<CronRow>` |
| CRON-2 | Keep domain rules in cron, with atomic conditional and multi-row operations in a `CronStorage: Storage<CronRow>` extension (PRD-STO-5): active-name uniqueness, row-limit checks, generation fencing, `set_enabled_many` and `replace_and_set_enabled` | existing rules in `crates/sc-cron/src/sqlite.rs` and `src/store.rs` | CRON-1 | the existing behaviors hold across independent callers; conditional checks and writes are one storage operation, and multi-row changes commit atomically, never a client-side read/check/write sequence |
| CRON-3 | Make the scheduler use the application's `Storage<CronRow>` / `CronStorage`; replace `ScheduleStore` and its re-exports only in the storage-alignment minor, with the CRON-9 consumer migration | `crates/sc-cron/src/store.rs`; `src/lib.rs`; `Scheduler<S, C, J>` in `src/scheduler.rs` | CRON-2, CRON-6, **Owner** O5 | the migrated beads-cache adapter works with the new scheduler; no `ScheduleStore` removal occurs in a 0.2.x patch |
| CRON-4 | Replace the bundled rusqlite store and own-file guard after the cooperative contract is resolved; port schema/scope behavior tests before retiring their old implementation | `src/sqlite.rs`, `src/sqlite_schema.rs`, `src/sqlite_scope.rs`, `sqlite_cooperative_*_tests.rs`; path constructors, `Cargo.toml`, boundary manifest | CRON-3, CRON-6, **Owner** O5 | cron uses app storage without a direct rusqlite dependency; all retained schema, fencing, atomicity and cooperative guarantees have equivalent tests; only the own-file expectation is withdrawn |
| CRON-5 | One async form of each public method; remove the sync twins and the blocking bridge (PRD-ENG-1, PRD-ENG-2) | `src/scheduler.rs` 403, 427, 472, 505 (sync), 413, 442 (`_async` wrappers), 838 (`blocking`); `src/worker/store_calls.rs` | CRON-3 | no store call in `sc-cron` goes through `spawn_blocking` |
| CRON-6 | Decide and re-plan the Phase C cooperative-operation work, which is built on the deleted store | `src/operation.rs`, `src/consumer_contract.rs`, `src/sqlite_scope.rs`; ADR-RUN-0018, REQ-CRON-0020, REQ-CRON-0021 | **Owner** O5 | ADR-RUN-0018 is rewritten or withdrawn |
| CRON-7 | Keep and port behavioral tests to the application's `Storage<CronRow>` / `CronStorage`; remove only assertions requiring an exclusive cron file | `tests/sqlite_store.rs`, `tests/two_process.rs`, `sqlite_cooperative_*_tests.rs`, and consumers naming `SqliteScheduleStore` / `Scheduler::open` | CRON-3, CRON-4, CRON-6, **Owner** O5 | generation fencing across two processes, multi-row atomicity, row limits and retained cooperative guarantees pass with controlled inputs; shared app tables are accepted; no behavioral suite is deleted merely for naming the old store |
| CRON-8 | Descriptive docs | `crates/sc-cron/README.md` 44–45, 66, 89, 189, 296; `crates/sc-cron-commands/README.md` 31; `docs/sc-cron/architecture.md` store sections; `docs/sc-cron/monty-generated-example.md` 78; `NFR-CRON-0001` dependency list | CRON-4 | no document says cron has its own database file |
| CRON-9 | Migrate beads-cache `CacheScheduleStore` from `ScheduleStore` to `Storage<CronRow>` / `CronStorage`, keeping schedules in the existing app database | randlee/beads-cache at `8e883ee99cb9c34dfb89fb39e25323d54ae6f2ad`, `crates/bd-cache-store/src/schedule.rs:80,164`, and its scheduler call sites | CRON-1, CRON-2, CRON-6, **Owner** O5 | the real consumer adapter and scheduler calls compile and its existing persisted rows, fencing and atomic operations pass against the new contracts before old API removal in the storage-alignment minor |

Unchanged in `sc-cron`: Monty (`src/monty_run.rs`, `monty_host.rs`, `monty_compose.rs`; PRD §7), expression parsing, jitter, clock, hooks, hook environment.

## 3. Template and generator

| Id | Change | Where | Waits on | Done when |
|---|---|---|---|---|
| TPL-1 | Reject cron with no database (PRD-CRON-4) | `template/hooks/drop-template-exclude.rhai` 2–3 (add beside the existing rule); `wizard/answers.schema.json` 50–73; `wizard/pages/page-3-options.html`; `wizard/fixtures/none.json`; the schema and fixture tests under `tests/unit/` | — | generating `db = none` with `cron = true` fails before `cargo generate` runs, and a test proves it |
| TPL-2 | Remove `cron.database_path` (PRD-CRON-3) | `template/crates/daemon/src/cron.rs` 15–16, 29–31, 40, 157–191, test at 237; `template/config/default.json` 20; `template/config/cron.schema.json` 12; `template/crates/daemon/src/main.rs`; `open/sqlite.rs`, `open/sqlite-postgres.rs`, `open/sqlite-mysql.rs`; `tests/unit/generation/test_cron_environment.py`; `tests/integration/release/test_standard_cron_commands.py`; `tests/integration/cron-consumer/tests/application_schema.rs` | CRON-3 | no generated project contains `database_path` or `cron.db` |
| TPL-3 | The daemon hands its own storage to cron | `template/crates/daemon/src/cron.rs` 146–153; `src/composition.rs` 90–91, 267 | CRON-3, STO-5 | the generated daemon opens one SQLite database and cron's table is in it |
| TPL-4 | Remove `CronConfig` from the no-database layout | `template/crates/daemon/src/open/none.rs` 10–12 | TPL-1 | the `none` layout has no cron configuration |
| TPL-5 | Replace the sqlx SQLite store with the SQLite `Storage<T>` implementation (PRD-STO-3) | `template/crates/store-sqlite/`: `src/lib.rs` 24, 41–42, 78, 91, 104, 179–185; `sqlx.toml`; `.sqlx/`; `migrations/` | STO-5, O3 | the generated SQLite app has no `sqlx` SQLite driver in `cargo tree` |
| TPL-6 | Remove `EngineStore` and `RowRecord` | `template/crates/schema/src/lib.rs` 369, 486 | TPL-5, O6 | neither name exists |
| TPL-7 | Store the widget example through `Storage<WidgetRow>` | `template/crates/service/src/layouts/*.rs` (8 files); per-table functions in the three store crates | STO-5 to STO-7 | `insert_widget` and `get_widget` are gone from the store crates |
| TPL-8 | Descriptive docs for generated projects | `docs/getting-started.md`, `docs/generation.md`, REQ-RUN-0301, REQ-RUN-0302, REQ-RUN-0308 bodies | TPL-5, TPL-7 | the documents describe the generated tree as it is |

## 4. Command path (PRD §3)

| Id | Change | Where | Waits on | Done when |
|---|---|---|---|---|
| CMD-1 | Implement a hand-written shared read/write path with `Tcli`, `Tsql` and an application transform (PRD-CMD-4/5), including a real Python/Monty transform caller. Only generating that wiring is deferred | `template/crates/daemon/src/routes.rs`; `src/mcp.rs`; `template/crates/cli/src/lib.rs`; `template/crates/service/src/layouts/*.rs` | STO-1, relevant storage implementation | CLI, MCP and HTTP use the shared path; a Python/Monty-authored transform runs through it and its result is stored/read through `Storage<Tsql>`; no code generator is required for this proof |

## 5. No change

| What | Why |
|---|---|
| `sc-config` | stays synchronous, no watcher, no reload; boundary manifest unchanged (PRD-ENG-5, PRD-ENG-6) |
| Synchronization, including `template/crates/store-adaptor*` | deferred (PRD §9). Keep the existing adaptor code as interim, without implementing synchronization; these crates depend on `EngineStore`, so TPL-6 waits on O6 |
| Per-interface permissions | wanted (PRD-CMD-7), not specified |
| `sc-transport`, `sc-command`, `sc-runtime` | a keyword scan of their requirements found no conflict; their code was not examined beyond what is listed here |

## 6. Open decisions and settled dependency floor

| Id | Decision | Blocks |
|---|---|---|
| O1 | The `Storage<T>` method set. Sketch: `read(key)`, `read_all()`, `write(&T)`; whether delete exists | STO-1 |
| O2 | How `T` names its table, columns and key | STO-1, CRON-1 |
| O3 | Who creates and migrates tables | TPL-5 |
| O4 | Where the trait and each implementation live (this repo, or one crate per repo) | STO-1 |
| O5 | Keep, rewrite or drop the Phase C cooperative-operation contract | CRON-3, CRON-4, CRON-6, CRON-7, CRON-9 |
| O6 | Any later disposition of the existing adaptor crates; keep them as interim code while synchronization is deferred | TPL-6 |
| O7 | Whether/how to generate the shared command wiring (syn or sc-compose); the hand-written path and transform caller are not blocked | future generator only |
| O8 — settled | Use rusqlite 0.41 as the SQLite implementation's tested base: RUSTSEC-2026-0459 is fixed in libsqlite3-sys >=0.39, first shipped by rusqlite 0.41. sqlx-sqlite 0.9.0's >=0.30.1,<0.38.0 range cannot share that identity; [Phase D dependency evidence, line 36](https://github.com/randlee/sc-runtime/blob/562ca49a0bd9423ce9de2189a2077b7a317695e3/docs/plans/phase-d/issue-133-dependency-sweep.md#L36). Normal compatible upgrades remain allowed; this is not an exact production pin | no owner decision blocks STO-2 |

## 7. Work in flight that this affects

- `plan/phase-d` plans dependency work on the assumption that generated SQLite projects stay on sqlx.
- Phase C anchor ANC-006 keeps rusqlite at 0.39.x for the same reason.
- `docs/plan/` and `docs/plans/` have not been reviewed against PRD v2.0.
