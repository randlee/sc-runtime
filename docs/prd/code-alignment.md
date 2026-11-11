# Code alignment with PRD v2.0

What in the code has to be deleted or changed for [PRD v2.0](sc-runtime-prd.md). Paths and line numbers are from `develop` at `4da05e3`, found by reading and searching. Nothing here has been compiled, run or tested, and no code has been changed.

Order matters: section 1 first, because sections 2 to 4 depend on `Storage<T>` existing.

## 1. Storage (PRD §4)

**Build**

| What | Where it comes from |
|---|---|
| `Storage<T>` trait: async, reads and writes `T` | New. Method set is open (ADR-RUN-0020, open item 1). |
| SQLite implementation: async input | Copy from atm-core `crates/atm-storage-rusqlite/src/writer/mod.rs`: bounded `tokio::sync::mpsc` channel, `submit_async` with a oneshot reply and deadline (lines 405–458), writer thread with its own current-thread runtime (253–340). |
| SQLite implementation: batching | Already here and already matches atm-core: `crates/sc-runtime-db-sqlite/src/batch.rs` (1 ms drain, `BEGIN IMMEDIATE`, savepoints, reply after commit). atm-core's version is `writer/batch.rs` `collect_batch` (208–245). |
| SQLite implementation: reads | Copy from atm-core `crates/atm-storage-rusqlite/src/reader_pool.rs`. Nothing equivalent exists here. |
| Postgres and MySQL implementations | sqlx. The connection and migration code in `template/crates/store-postgres/src/lib.rs` and `store-mysql/src/lib.rs` is reusable; the per-table functions are not. |

**Delete or replace in `crates/sc-runtime-db-sqlite`**

| File | Lines | What |
|---|---|---|
| `src/writer.rs` | 5, 102–103, 156, 199, 218 | `std::sync::mpsc` channels: replace with the async channel |
| `src/writer.rs` | 86, 154, 193 | blocking `open`, `execute`, `transaction`: replace with async equivalents |
| `src/batch.rs` | 153–165 | `collect_batch` on a std receiver with `thread::yield_now` |
| `src/batch.rs` | 479–485 | `wait_reply` with `recv_timeout`: the blocking wait |
| `src/writer.rs` doc comment | 20–23 | "call them inside `spawn_blocking`"; "This crate does not add an async wrapper" |

The payload is raw SQL strings today (`execute(sql, params)`). Under `Storage<T>` callers pass `T`; the SQL stays inside the implementation.

## 2. Cron (PRD §6)

**Delete in `crates/sc-cron`**

| File | What |
|---|---|
| `src/store.rs` (319 lines) | the synchronous `ScheduleStore` trait |
| `src/sqlite.rs` (1,205), `src/sqlite_schema.rs` (158), `src/sqlite_scope.rs` (378) | the rusqlite schedule store, including the "sc-cron needs its own database file" guard in `init_schema` |
| `src/sqlite_cooperative_tests.rs`, `sqlite_cooperative_lifecycle_tests.rs`, `sqlite_cooperative_transaction_tests.rs` (1,422) | tests of that store |
| `tests/sqlite_store.rs`, `tests/two_process.rs` | tests of the own-file, two-process store |
| `src/lib.rs` 137–138, 180, 182 | `mod sqlite`, `mod store` and their re-exports |
| `src/scheduler.rs` 704, 741 | `Scheduler::open`, `open_with_logger` (construct the SQLite store from a path) |
| `Cargo.toml` | the `rusqlite` dependency; `boundaries/sc-cron/sc-cron.toml` line 41 |

**Change in `crates/sc-cron`**

| File | Lines | What |
|---|---|---|
| `src/scheduler.rs` | 403, 427, 472, 505 | synchronous `start`, `stop`, `set_enabled`, `set_enabled_many`: keep one async form of each; remove the sync twins and the `_async` wrappers at 413 and 442 |
| `src/scheduler.rs`, `src/worker/store_calls.rs` | 838; whole file | the `blocking` bridge and `store_call`: gone once storage is async |
| new | — | `CronRow` struct; the rules now inside the store (active-name uniqueness, row limit, generation fencing) move into cron's own logic, the transform of PRD-CMD-4 |

Not sized: the cooperative-operation work (ADR-RUN-0018, `src/operation.rs`, `src/consumer_contract.rs`) is built on `sqlite_scope.rs` and has to be re-planned.

**Stays:** `src/monty_run.rs`, `monty_host.rs`, `monty_compose.rs` (Monty is done, PRD §7); expression parsing, jitter, clock, hooks, environment.

## 3. Template and generator

| File | Lines | What |
|---|---|---|
| `template/crates/daemon/src/cron.rs` | 15–16, 29–31, 40, 157–191, test at 237 | `database_path` and its path and symlink validation: delete |
| `template/crates/daemon/src/cron.rs` | 146–153 | `open` returns `Scheduler<SqliteScheduleStore, …>`: take the app's storage instead |
| `template/crates/daemon/src/composition.rs` | 90–91, 267 | `open_scheduler` and `CronRuntime` name `SqliteScheduleStore` |
| `template/config/default.json` | 20 | `cron.database_path` |
| `template/config/cron.schema.json` | 12 | `database_path` |
| `template/crates/daemon/src/open/none.rs` | 10–12 | the no-database layout carries a `CronConfig`: remove |
| `template/hooks/drop-template-exclude.rhai` | 2–3 | add the `cron` requires a database rule beside the existing `cron_commands requires cron` rule |
| `wizard/answers.schema.json` | 50–73 | add the same rule to `allOf` |
| `wizard/pages/page-3-options.html`, `wizard/fixtures/none.json` | — | reflect the rule |
| `template/crates/store-sqlite/` | `src/lib.rs` 24, 41–42, 78, 91, 104, 179–185; `sqlx.toml`; `.sqlx/` | the sqlx SQLite store and its per-table `insert_widget`, `get_widget`: replaced by the SQLite `Storage<T>` implementation |
| `template/crates/schema/src/lib.rs` | 369, 486 | `RowRecord` and `EngineStore`: superseded by `Storage<T>` |

## 4. Command path (PRD §3): interim, change strategically

These files are the hand-written form of the pattern. They are not wrong; they are the stand-in until a shared path or a generator exists (PRD §9). Do not delete them before that.

| File | What it hand-writes |
|---|---|
| `template/crates/daemon/src/routes.rs` 59–92 | one REST handler per widget operation |
| `template/crates/daemon/src/mcp.rs` 124–148 | one MCP tool per operation |
| `template/crates/cli/src/lib.rs` (935 lines) | one CLI command per operation |
| `template/crates/service/src/layouts/*.rs` (8 files) | one service implementation per database layout |
| `template/crates/store-postgres/src/lib.rs`, `store-mysql/src/lib.rs` | per-table functions per engine |

The widget entity appears in 22 non-test source files. The target is two structs and a transform.

## 5. Leave alone for now

| What | Why |
|---|---|
| `template/crates/store-adaptor*` (three crates) | synchronization is deferred (PRD §9). They depend on `EngineStore`, so they break when section 3 removes it; decide then whether to park or delete them. |
| Per-interface permissions | wanted (PRD-CMD-7) but not specified; today only transport-level access control exists (`sc-transport`, ADR-TRN-0005). |
| `sc-config`, `sc-transport`, `sc-command`, `sc-runtime` | not examined against the PRD beyond what is listed here. |

## 6. `sc-config` (PRD-ENG-5, PRD-ENG-6)

No change. The crate stays synchronous with no watcher and no reload, and its boundary manifest stays as it is. Nothing is to be built or tested for reload unless the owner asks.
