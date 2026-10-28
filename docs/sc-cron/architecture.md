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

The number of schedule rows is not capped unless the embedder passes a limit ([ADR-RUN-0014](../architecture.md)). Each row's name, expression, timeout, module path, source, and payload are capped in the crate. A caller that accepts schedules from untrusted input should pass a row limit or enforce one before insert.

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

Under feature `monty`, `cron_tick` or `cron_stop` may call the host function `render_template(template, context)`. Production runs that call inside `execute_feed` (which may call `feed_start` internally). The host spawns `sc-compose` on `PATH`. Hook source cannot choose the program or register another host function. Tests pass a program through the crate-private `execute_feed_with_compose_program`; that function is not an embedder setting ([ADR-RUN-0013](../architecture.md) names `sc-compose`).

```text
sc-compose render --root <temp> --mode file --file t.j2 --var-file v.json
```

```rust
pub(crate) struct ComposeProgram<'a>(&'a str);

pub(crate) enum ComposeHostError {
    ProgramNotFound { program: String },
    TempRootFailed { program: String, source: std::io::Error },
    StageFailed { program: String, source: std::io::Error },
    ContextEncodeFailed { program: String, source: serde_json::Error },
    ProgramNotRunnable { program: String, source: std::io::Error },
    OutputNotUtf8 { program: String },
    OutputTooLarge { program: String },
    HostTimedOut { program: String },
    NonZeroExit { status: Option<i32>, stderr: String },
}

impl ComposeHostError {
    pub(crate) const fn code(&self) -> &'static str;
    pub(crate) const fn recovery(&self) -> &'static str;
}

pub(crate) fn render_template(
    template: &str,
    context: &serde_json::Value,
    compose_program: ComposeProgram<'_>,
    host_deadline: Option<std::time::Instant>,
) -> Result<String, ComposeHostError>;
```

`ComposeHostError` implements `std::error::Error`. I/O and JSON failures keep their `source()`. `NonZeroExit.status` is `None` when a signal ends the child. These variants are not `CronError` variants.

The Monty source does not choose the temp root. `--mode file` reads only those two files. Stdout is the rendered string. `--output` is omitted. `OsCall` and any other function name fail the feed. The tick does not start CPython. `ResourceTracker` pauses `max_feed_duration` while the feed is suspended on `render_template`; the host kills the child and its process group at that `Instant` deadline. No new `CronError` variant. The seven variants stay `BadExpression`, `InvalidSpec`, `AlreadyActive`, `NotFound`, `HookFailed`, `Store`, and `Fenced`.

| Condition | Code | `RunOutcome` | Operator recovery |
|---|---|---|---|
| Compose program not found (`ProgramNotFound`) | `COMPOSE_NOT_FOUND` | `Failed` — detail names the program (not `HookFailed`). A passed host deadline does not turn this into `TimedOut`. | Install `sc-compose` on PATH or fix the injected program name in tests. |
| Temp root or staged file I/O (`TempRootFailed`, `StageFailed`) | `COMPOSE_SPAWN` | `Failed` | Check permissions and disk space for temp roots under `tempfile::TempDir`. |
| Context JSON encode (`ContextEncodeFailed`) | `COMPOSE_CONTEXT` | `Failed` | Fix the template context; it could not be encoded as JSON. |
| Program cannot be executed (`ProgramNotRunnable`) | `COMPOSE_NOT_RUNNABLE` | `Failed` | Check that the compose program is executable and allowed to start. |
| Stdout is not UTF-8 (`OutputNotUtf8`) | `COMPOSE_OUTPUT` | `Failed` | Fix the compose program so its stdout is UTF-8. |
| Stdout longer than the hook output cap (`OutputTooLarge`) | `COMPOSE_OUTPUT` | `Failed` | Shrink the rendered output. |
| CLI exits non-zero (`NonZeroExit`) | `COMPOSE_EXIT` | `Failed` | Fix template/context or run `sc-compose render` manually with the logged stderr. |
| Denied host call (wrong arity, type, or name) | `HOST_DENIED` | `Failed` | Fix hook source; only `render_template` is allowlisted. |
| Undefined name | `HOST_UNDEFINED` | `Failed` | Fix hook source; that name is not defined. |
| `OsCall` | `HOST_OS` | `Failed` — detail names the operation | Fix hook source; only `render_template` is allowlisted. |
| Monty raises during the feed | `HOST_RAISED` | `Failed` | The detail is the exception message. |
| Host `Instant` deadline during the child, or the child returns after that deadline |  | `TimedOut`, session dropped, child killed | Reduce compose work or raise the row deadline; the VM clock was paused during the host call. |
| Monty feed-time-limit while the VM is executing (not during the host suspend) |  | `TimedOut` | Same checkpoint path as a Monty feed that never calls `render_template`. |

A feed that never calls `render_template` still runs when the compose program is absent. The compose child is supervised like a Python hook: at the host deadline the child and its process group are killed, the blocking-pool thread is released, and the feed records `TimedOut`. Stdout and stderr are each kept up to `MAX_HOOK_OUTPUT_BYTES`. Stderr copied into a failure detail uses the same 512-byte slice as a Python hook. The program name is fixed as `sc-compose` on `PATH`; install that binary or symlink the name onto `PATH`. There is no embedder override.

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

The crate-root exports, also listed on `boundaries/sc-cron/sc-cron.toml`, are the contract: `Clock`, `CronError`, `CronExpr`, `DEFAULT_RELOAD_POLL`, `DueRun`, `Generation`, `HOOK_PROTOCOL`, `HookCall`, `HookEvent`, `HookOutput`, `HookStatus`, `HookTarget`, `is_identifier`, `JitterSource`, `Job`, `kill_budget`, `ManualClock`, `MAX_EXPR_LEN`, `MAX_HOOK_OUTPUT_BYTES`, `MAX_HOOK_SOURCE_BYTES`, `MAX_NAME_LEN`, `MAX_PAYLOAD_BYTES`, `MAX_TIMEOUT`, `MissedTickPolicy`, `NewSchedule`, `process`, `PythonHook`, `RunOutcome`, `RunTimeout`, `Schedule`, `ScheduleId`, `ScheduleName`, `ScheduleSpec`, `ScheduleStore`, `Scheduler`, `SCHEMA_VERSION`, `SeqJitter`, `SqliteScheduleStore`, `StopDecision`, `StoreError`, `StoreErrorKind`, `SystemClock`, `SystemJitter`, `Timestamp`, and `WorkerHandle`. `ManualClock` and `SeqJitter` need feature `test-utils`.

A cache-sync adaptor in the consumer repo exposes the sync methods. The schedule row points at one of them. This crate does not know whether that method talks to beads or to Postgres.

## Error

`CronError` is `#[non_exhaustive]` and implements `Display` and `std::error::Error`. Variants: `BadExpression(String)`, `InvalidSpec(String)`, `AlreadyActive(ScheduleName)`, `NotFound(ScheduleName)`, `HookFailed(String)`, `Store(StoreError)`, `Fenced { id, generation }`. Codes: `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, `FENCED`. `StoreError` is this crate's error for the schedule table. It is not the schema crate's `StoreError`. It wraps `rusqlite::Error` through `Error::source` and uses the code `STORE_FAILED`. The `BadExpression` recovery text tells the caller to use a five-field string, a six-field seconds-first string, or `@hourly` through `@yearly`.

`Scheduler::open` takes the path of the schedule database. That file is not the application's `store-sqlite` file.

## What this crate is not

It does not open Postgres, call beads, or ship an HTTP, MCP, or CLI API. The consumer may add that API and have it insert rows. beads-cache's hook calls beads. Another consumer's hook is the sync method on its adaptor.
