# sc-cron Architecture

**Status:** Active as of 2026-10-25. Binding decision: [ADR-RUN-0013](../architecture.md). Sprint `b-3` ports beads-cache `crates/sc-cron` and applies the expression rule below.

Port source: beads-cache `integrate/phase-b` commit `d5fbed94f6c3b7fa9e5a562787964a5e6f298423`.

## What this crate is

An optional drop-in scheduler. The consumer adds one SQLite table. A row is a cron string plus a hook. The crate fires that hook. A richer start/stop API is something the consumer may build on top.

## Schedule strings

A normal string has five fields, minute first, in UTC: `minute hour day-of-month month day-of-week`. Example: `*/5 * * * *`.

`@yearly`, `@monthly`, `@weekly`, `@daily`, and `@hourly` are valid.

A six-field string is seconds first: `second minute hour day-of-month month day-of-week`. Example: `0 */5 * * * *`. A five-field string is never read as seconds.

## Configuration table

One table, created by the crate. Columns cover the schedule name, the cron string, the timeout, a JSON payload, and the hook target. The rest of the database belongs to the consumer. The connection is not `SqliteWriter`.

The number of schedule rows is not capped unless the embedder passes [`ScheduleLimits`](../../crates/sc-cron/src/limits.rs) ([ADR-RUN-0014](../architecture.md)). `max_schedule_rows: None` is the library default and means unlimited. The count is every row that has not ended, enabled and disabled together. Replacing a name that is already counted does not change the count. Ending a row (`Scheduler::stop`, including a stop-check delete) frees a slot in that same store transaction. A new name over the limit is `CronError::InvalidSpec`; the detail names the limit and the current count. There is no new `CronError` variant. Lowering the limit does not delete rows. Each row's name, expression, timeout, module path, source, and payload are capped in the crate. sc-cron does not depend on sc-config. The ceiling is the `ScheduleLimits` that process passed when it opened the store. It is not written into the schedule database. Every writer on one schedule database must open with identical `ScheduleLimits`. A writer opened with a different ceiling, including unlimited, does not enforce another writer's cap.

Generated applications ship sc-config key `cron.max_schedule_rows` (integer or JSON null, default `10` when the key is absent, `0` or `null` means unlimited) in `template/answers.schema.json` and `template/config/default.json`. The daemon reads it into `cron::CronConfig` on every layout. No layout opens `Scheduler` yet. The layout that does maps the value and opens the scheduler:

```rust
let limits = sc_cron::ScheduleLimits {
    max_schedule_rows: config.cron.row_limit(),
};
let scheduler = sc_cron::Scheduler::open(cron_db, limits, clock, jitter)?;
```

`CronConfig::row_limit` maps JSON `null` and `0` to `None` and any positive value to `Some(NonZeroUsize)`. A missing key stays the default `10`. `ScheduleLimits::from_config_rows` is the same integer mapping for an embedder that does not use the generated daemon. sc-cron does not read sc-config.

## Hook

```rust
#[non_exhaustive]
pub enum HookTarget {
    Rust,
    Python { module: std::path::PathBuf },
    Monty { source: String },
}

pub struct ScheduleSpec {
    pub name: String,
    pub expr: String,
    pub timeout: std::time::Duration,
    pub payload: serde_json::Value,
    pub hook: HookTarget,
}
```

`Rust` calls the consumer's `Job` in process. `Python` runs `python3 <module> <hook-name>`. `Monty` is trusted embedding-chosen source, not a cache path, and is available only with feature `monty`; it uses neither `monty-pool`, `monty-fs`, nor `monty-runtime`. Python and Monty receive a `sc-cron.hook.v1` object containing `protocol`, `event`, `schedule`, `payload`, and a microsecond UTC `deadline`; it has no beads, bd-cache, or ATM field.

Monty source defines `cron_stop(hook)` and `cron_tick(hook)`. One REPL is retained per schedule name while no feed is active, so module state survives ticks. The worker always calls stop before tick; `disable` or `delete` skips tick, and an external replacement, disable, delete, or shutdown after a continuing stop check skips tick and drops the REPL. A REPL is never dropped or replaced before its feed returns. Monty feed time uses `ResourceLimits::max_feed_duration`; it is observed at interpreter checkpoints, has no memory limit, and is not aborted by worker shutdown. This is trusted in-process code: stack-overflow and allocator-abort risks are accepted just as they are for a Rust `Job`.

```rust
#[async_trait::async_trait]
pub trait Job: Send + Sync {
    async fn run(&self, payload: &serde_json::Value, deadline: std::time::Instant) -> RunOutcome;
    async fn should_stop(&self, payload: &serde_json::Value) -> StopDecision;
}

pub struct CronExpr;

pub struct Scheduler<S: ScheduleStore, C: Clock, J: JitterSource>;

pub enum RunOutcome {
    Ok,
    Refused(String),
    Failed(String),
    TimedOut,
}

pub enum StopDecision {
    Continue,
    Disable(String),
    Delete(String),
}
```

`CronExpr::parse` accepts the schedule strings above. `Scheduler` opens the one schedule table. `StopDecision` is `Continue`, `Disable(String)`, and `Delete(String)`. `Stop` is not a variant. The port copies this enum from beads-cache after sprint b-8 removes the `Stop` alias.

`ScheduleStore` stays an open seam. The landed scheduler is generic:

```rust
pub struct Scheduler<S: ScheduleStore, C: Clock, J: JitterSource>;
```

`SqliteScheduleStore` is the production store. The crosswalk retains this seam with the copy (REQ-CRON-0009). The public module `process` and `PythonHook` also stay public. They are the retained REQ-CRON-0018 supervisor port (`ChildSpec`, `supervise`, and the related supervisor types). That surface is not an sc-runtime HTTP route, MCP tool, or product API.

## Public exports

The crate-root exports, also listed on `boundaries/sc-cron/sc-cron.toml`, are the contract: `Clock`, `CronError`, `CronExpr`, `DEFAULT_RELOAD_POLL`, `DueRun`, `Generation`, `HOOK_PROTOCOL`, `HookCall`, `HookEvent`, `HookOutput`, `HookStatus`, `HookTarget`, `is_identifier`, `JitterSource`, `Job`, `kill_budget`, `ManualClock`, `MAX_EXPR_LEN`, `MAX_HOOK_OUTPUT_BYTES`, `MAX_HOOK_SOURCE_BYTES`, `MAX_NAME_LEN`, `MAX_PAYLOAD_BYTES`, `MAX_TIMEOUT`, `MissedTickPolicy`, `NewSchedule`, `process`, `PythonHook`, `RunOutcome`, `RunTimeout`, `Schedule`, `ScheduleId`, `ScheduleLimits`, `ScheduleName`, `ScheduleSpec`, `ScheduleStore`, `Scheduler`, `SCHEMA_VERSION`, `SeqJitter`, `SqliteScheduleStore`, `StopDecision`, `StoreError`, `StoreErrorKind`, `SystemClock`, `SystemJitter`, `Timestamp`, and `WorkerHandle`. `ManualClock` and `SeqJitter` need feature `test-utils`.

A cache-sync adaptor in the consumer repo exposes the sync methods. The schedule row points at one of them. This crate does not know whether that method talks to beads or to Postgres.

## Error

`CronError` is `#[non_exhaustive]` and implements `Display` and `std::error::Error`. Variants: `BadExpression(String)`, `InvalidSpec(String)`, `AlreadyActive(ScheduleName)`, `NotFound(ScheduleName)`, `HookFailed(String)`, `Store(StoreError)`, `Fenced { id, generation }`. Codes: `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, `FENCED`. `StoreError` is this crate's error for the schedule table. It is not the schema crate's `StoreError`. It wraps `rusqlite::Error` through `Error::source` and uses the code `STORE_FAILED`. The `BadExpression` recovery text tells the caller to use a five-field string, a six-field seconds-first string, or `@hourly` through `@yearly`.

`Scheduler::open` is the production path. It takes the path of the schedule database and a `ScheduleLimits`, opens the store with that value, and returns `CronError::InvalidSpec` when the value does not match the store's limits. `Scheduler::new` is for a store the caller already opened; pass the same `ScheduleLimits` that store was opened with. That file is not the application's `store-sqlite` file.

## What this crate is not

It does not open Postgres, call beads, or ship an HTTP, MCP, or CLI API. The consumer may add that API and have it insert rows. beads-cache's hook calls beads. Another consumer's hook is the sync method on its adaptor.
