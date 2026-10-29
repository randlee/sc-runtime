# sc-cron Architecture

**Status:** Active as of 2026-10-28. Binding decisions: [ADR-RUN-0013](../architecture.md) and [ADR-RUN-0016](../architecture.md). Sprints `b-3` through `b-17` establish the contract below.

Port source: beads-cache `integrate/phase-b` commit `d5fbed94f6c3b7fa9e5a562787964a5e6f298423`.

## What this crate is

An optional drop-in scheduler. The consumer adds one SQLite table. A row is a cron string plus a hook. The crate fires that hook. A richer start/stop API is something the consumer may build on top.

## Schedule strings

A normal string has five fields, minute first, in UTC: `minute hour day-of-month month day-of-week`. Example: `*/5 * * * *`.

`@yearly`, `@monthly`, `@weekly`, `@daily`, and `@hourly` are valid.

A six-field string is seconds first: `second minute hour day-of-month month day-of-week`. Example: `0 */5 * * * *`. A five-field string is never read as seconds.

## Configuration table

One table, created by the crate. Columns cover the schedule name, the cron string, the timeout, a JSON payload, and the hook target. The rest of the database belongs to the consumer. The connection is not `SqliteWriter`.

No sc-runtime binary or sc-cron package shipped the development schema ladder.
The first release schema is therefore `SCHEMA_VERSION = 1`, fingerprinted by
exact `table_xinfo`, byte-for-byte equality with the canonical release-v1
`sqlite_schema.sql` table string (no SQL normalization), and exact
unique/index inventory. Canonical `hook_environment`
JSON uses fixed compact field/key ordering and defaults exactly to
`{"dotenv":[],"variables":{}}`. Earlier development files at user versions 1
through 4 are not a compatibility contract. An existing non-empty file is
fingerprinted read-only before any WAL pragma. An old shape, including an old
file that also says version 1, is refused with `SchemaMismatch` and recreate
guidance without changing the file or SQLite sidecars; it is not silently
reinterpreted or migrated. Compatibility begins with this release-v1 shape.

The number of schedule rows is not capped unless the embedder passes [`ScheduleLimits`](../../crates/sc-cron/src/limits.rs) ([ADR-RUN-0014](../architecture.md)). `max_schedule_rows: None` is the library default and means unlimited. The count is every row that has not ended, enabled and disabled together. Replacing a name that is already counted does not change the count. Ending a row (`Scheduler::stop`, including a stop-check delete) frees a slot in that same store transaction. A new name over the limit is `CronError::InvalidSpec`; the detail names the limit and the current count. There is no new `CronError` variant. Lowering the limit does not delete rows. Each row's name, expression, timeout, module path, source, and payload are capped in the crate. sc-cron does not depend on sc-config. The ceiling is the `ScheduleLimits` that process passed when it opened the store. It is not written into the schedule database. Every writer on one schedule database must open with identical `ScheduleLimits`. A writer opened with a different ceiling, including unlimited, does not enforce another writer's cap.

Generated applications ship sc-config key `cron.max_schedule_rows` (integer or JSON null, default `10` when the key is absent, `0` or `null` means unlimited) in `template/config/default.json`. [`template/config/cron.schema.json`](../../template/config/cron.schema.json) checks that object and allows every other key in the file, so the real default document validates. cargo-generate `exclude` copies a file without Liquid rendering and does not omit it. The cron schema is on `[template] ignore`, so a generated project does not contain it and does not contain `answers.schema.json`. The daemon reads the key into `cron::CronConfig` on every layout. No layout opens `Scheduler` yet. The layout that does maps the value and opens the scheduler:

```rust
let limits = sc_cron::ScheduleLimits::with_max_schedule_rows(
    config.cron.schedule_limits_fields().max_schedule_rows,
);
let scheduler = sc_cron::Scheduler::open(cron_db, limits, clock, jitter)?
    .with_monty_host(sc_cron::MontyHostConfig::new(compose_program)?);
```

`compose_program` is [`MontyHostConfig`](../../crates/sc-cron/src/monty_host.rs) on [`Scheduler::with_monty_host`](../../crates/sc-cron/src/scheduler.rs): any non-empty path (`CronError::InvalidSpec` when empty). Before a child environment is cleared, a basename is resolved on daemon `PATH`/Windows `PATHEXT`, an absolute path is retained, and a relative path with components is made absolute against the daemon current directory captured when host configuration is built. The default, when the embedder does not call `with_monty_host`, is the basename `sc-compose`. The embedder may take that path from configuration or `SC_COMPOSE` in the daemon. sc-cron does not read an application configuration file; it reads daemon-process variables only by the names the embedder passes in `HookEnvironmentHost`. Hook source cannot choose the program or the temp root.

`CronConfig::row_limit` maps JSON `null` and `0` to `None` and any positive value to `Some(NonZeroUsize)`. A missing key stays the default `10`. That mapping stays in the generated daemon. An embedder that does not use it builds `ScheduleLimits` itself. sc-cron receives `ScheduleLimits` only and does not read sc-config.

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
    pub environment: HookEnvironment,
}
```

`Rust` calls the consumer's `Job` in process. `Python` runs `python3 <module> <hook-name>`. `Monty` is trusted embedding-chosen source, not a cache path, and is available only with feature `monty`; it uses neither `monty-pool`, `monty-fs`, nor `monty-runtime`. Python and Monty receive a `sc-cron.hook.v1` object containing `protocol`, `event`, `schedule`, `payload`, and a microsecond UTC `deadline`; it has no beads, bd-cache, or ATM field.

### Python and Monty environment

Python and Monty rows share one `HookEnvironment`; constructors default it to
empty. Rust remains trusted in-process application code and rejects a
non-empty schedule environment rather than accepting ignored configuration.
The Rust `Job` trait does not change.

The daemon supplies `HookEnvironmentHost`, whose defaults are an empty
daemon-process allowlist, no authorized `.env` file/root, and a resolver that
resolves no references. It stores names, not captured process values. The
library never infers an invoking agent environment. One occurrence snapshot is
resolved before stop and reused for tick in this order:

1. present daemon values named in the daemon allowlist;
2. authorized absolute `.env` files in declaration order;
3. schedule `Set`, `Reference`, or `Clear` operations.

Set and a resolved reference replace an earlier value; clear removes it.
Explicit set values are canonical plaintext SQLite configuration. Values read
from the daemon, file, or resolver are not persisted. Reference requests carry
schedule name, variable name, opaque reference, and the row deadline so the
application can authorize per-schedule access. Provider errors are fieldless,
resolution is deadline-bounded, and sc-cron emits no provider prose.

Reference strings are persisted plaintext non-secret locators. Every
intermediate/final map has at most 128 entries, each stored/resolved value at
most 16 KiB, and the encoded final block at most 32,000 UTF-8 bytes. Source
oversize failures and aggregate `ENV_SNAPSHOT_LIMIT` failures are value-free.

Names, paths, values, and references are UTF-8 and NUL-free. ASCII case-fold
name collisions are rejected on every platform. An allowlisted non-Unicode
daemon value fails as `ENV_PROCESS_INVALID`. Each hook/provider-controlled
failure detail or stop reason has every distinct non-empty resolved value
replaced with `[REDACTED_ENV]`, longest first, before byte capping and before
fixed code/recovery, outcome, trace, event, or persisted-reason assembly.
Stable text, metadata, and canonical environment configuration writes are not
substituted; deliberate set values remain plaintext. This includes Python
output, Monty exception/return text, and compose stderr. Raw public
`HookOutput` fields remain available only to trusted direct callers; its
`Debug` omits output content. `ChildEnv`, `ChildSpec`, and `PythonHook` also
use value-free custom `Debug` so a resolved snapshot cannot leak through
supervisor configuration objects. Exact-value replacement mitigates accidental
propagation, not transformation or exfiltration by trusted hook code.

At execution, an absolute `.env` path is lexically normalized and authorized
before it is classified missing, so denied paths reveal no existence state.
An existing final regular target must equal an expected canonical file or
remain under an allowed canonical root. A missing optional target is accepted
only when its expected canonical path is covered by that policy. The bounded
UTF-8 parser accepts blanks, comments, optional `export`,
and quoted or unquoted `NAME=VALUE`; it rejects interpolation, command
substitution, duplicate names, malformed escapes, and NUL. Missing optional
files are absent; file-policy, required-file, parse, and reference failures
fail before hook source starts.

Python's child environment is cleared and populated with exactly the snapshot.
Python and compose basenames are resolved through daemon `PATH`/Windows
`PATHEXT` before the child environment is cleared; absolute paths are retained,
and relative paths with components are made absolute against the captured
daemon current directory.
Full ambient inheritance remains available only through explicit direct-call
`ChildEnv::Inherit`. Monty resumes `Getenv` and `GetEnviron` from the same map;
missing or cleared names behave as absent. Source initialization uses a
stage-specific resume driver: top-level environment reads work, but every
function call, including both compose hosts, is denied there. Ordinary
retained-module semantics apply to a value copied into a global. Every other
Monty OS call remains denied. A render-only compose child receives the same
snapshot and not the daemon's complete environment. Neither path mutates the
daemon process environment. Runtime lookup and enumeration preserve spelling
on Unix. Windows runtime keys are uppercased for both the Python child and
Monty `GetEnviron`, and Monty `Getenv` uses case-insensitive lookup; persisted
config retains original spelling.

The generated daemon reads its own config and constructs the typed host
policy; sc-cron never reads `sc-config`. Default generated arrays
`cron.environment.allow_process`, `allow_dotenv_files`, and
`allow_dotenv_roots` are empty. Reference
schemes such as `keychain://` remain opaque and require an embedder resolver.
The generated `cron::open` accepts the completed `HookEnvironmentHost`, so
application resolver injection is not lost during final composition.
See [ADR-RUN-0016](../architecture.md) and
[REQ-CRON-0019](requirements.md).

Monty source defines `cron_stop(hook)` and `cron_tick(hook)`. One REPL is retained per schedule name while no feed is active, so module state survives ticks. The worker always calls stop before tick; `disable` or `delete` skips tick, and an external replacement, disable, delete, or shutdown after a continuing stop check skips tick and drops the REPL. A REPL is never dropped or replaced before its feed returns. Monty feed time uses `ResourceLimits::max_feed_duration`; it is observed at interpreter checkpoints, has no memory limit, and is not aborted by worker shutdown. This is trusted in-process code: stack-overflow and allocator-abort risks are accepted just as they are for a Rust `Job`.

Under feature `monty`, `cron_tick` or `cron_stop` may call the host functions `render_template(template, context)` and `sc_compose_run(subcommand, args)`. Amended [ADR-RUN-0013](../architecture.md) decision point 4 names both functions: other host function names fail the feed. The embedder sets the compose program on [`Scheduler::with_monty_host`](../../crates/sc-cron/src/scheduler.rs) via [`MontyHostConfig`](../../crates/sc-cron/src/monty_host.rs) (default `sc-compose` on `PATH`). Production runs those calls inside `drive_feed` / `execute_feed_with_compose_program`. Render paths stage a temp root the hook cannot choose (`render --root <temp> --mode file --file t.j2 --var-file v.json`). `sc_compose_run` allows only subcommand `render` on that path; a hook-chosen `--root` is ignored, and `bead`, `init`, and `extract` fail the feed with `COMPOSE_SUBCOMMAND`. Hook source cannot choose the program or register another host function. Tests pass another program through the crate-private `execute_feed_with_compose_program`.

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
    NonZeroExit { status: Option<i32>, stderr: String },
    SubcommandDenied { subcommand: String },
}

impl ComposeHostError {
    pub(crate) const fn code(&self) -> &'static str;
    pub(crate) const fn recovery(&self) -> &'static str;
}

pub(crate) fn render_template(
    template: &str,
    context: &serde_json::Value,
    compose_program: ComposeProgram<'_>,
) -> Result<String, ComposeHostError>;
```

`ComposeHostError` implements `std::error::Error`. I/O and JSON failures keep their `source()`. `NonZeroExit.status` is `None` when a signal ends the child. These variants are not `CronError` variants.

The Monty source does not choose the temp root. `--mode file` reads only those two files. Stdout is the rendered string. `--output` is omitted. `Getenv` and `GetEnviron` use the resolved hook snapshot; every other `OsCall` and any other function name fail the feed. The tick does not start CPython. `ResourceTracker` pauses `max_feed_duration` while the feed is suspended on `render_template`; the blocking-pool thread stalls until that compose child exits. After the child exits, a host `Instant` past the row budget records `TimedOut` and drops the repl. No new `CronError` variant. The seven variants stay `BadExpression`, `InvalidSpec`, `AlreadyActive`, `NotFound`, `HookFailed`, `Store`, and `Fenced`.

| Condition | Code | `RunOutcome` | Operator recovery |
|---|---|---|---|
| Compose program not found (`ProgramNotFound`) | `COMPOSE_NOT_FOUND` | `Failed` — detail names the program (not `HookFailed`). A passed host deadline does not turn this into `TimedOut`. | Install `sc-compose` on PATH or fix the injected program name in tests. |
| Temp root or staged file I/O (`TempRootFailed`, `StageFailed`) | `COMPOSE_SPAWN` | `Failed` | Check permissions and disk space for temp roots under `tempfile::TempDir`. |
| Context JSON encode (`ContextEncodeFailed`) | `COMPOSE_CONTEXT` | `Failed` | Fix the template context; it could not be encoded as JSON. |
| Program cannot be executed (`ProgramNotRunnable`) | `COMPOSE_NOT_RUNNABLE` | `Failed` | Check that the compose program is executable and allowed to start. |
| Stdout is not UTF-8 (`OutputNotUtf8`) | `COMPOSE_OUTPUT` | `Failed` | Fix the compose program so its stdout is UTF-8. |
| Stdout longer than the hook output cap (`OutputTooLarge`) | `COMPOSE_OUTPUT` | `Failed` | Shrink the rendered output. |
| CLI exits non-zero (`NonZeroExit`) | `COMPOSE_EXIT` | `Failed` | Fix template/context or run `sc-compose render` manually with the logged stderr. |
| Denied `sc-compose` subcommand (`SubcommandDenied`) | `COMPOSE_SUBCOMMAND` | `Failed` | Call `sc_compose_run` with subcommand `render` only. `bead`, `init`, and `extract` are denied. |
| Denied host call (wrong arity, type, or name) | `HOST_DENIED` | `Failed` | Fix hook source; only `render_template` and `sc_compose_run` are allowlisted. |
| Undefined name | `HOST_UNDEFINED` | `Failed` | Fix hook source; that name is not defined. |
| Monty `OsCall` other than `Getenv` or `GetEnviron` | `HOST_OS` | `Failed` — detail names the operation | Fix hook source; only the two environment reads plus `render_template` and `sc_compose_run` are allowlisted. |
| Allowlisted daemon variable is not Unicode or exceeds the value cap | `ENV_PROCESS_INVALID` | `Failed` — no value or reference text | Restart the daemon with a bounded UTF-8 value or remove the name from the allowlist. |
| `.env` path outside daemon policy | `ENV_FILE_DENIED` | `Failed` — no value or reference text | Authorize the canonical file/root or remove the schedule source. |
| Required `.env` file missing | `ENV_FILE_MISSING` | `Failed` — no value or reference text | Restore the absolute file or mark it optional. |
| `.env` invalid | `ENV_FILE_INVALID` | `Failed` — no value or reference text | Fix the bounded data-file syntax. |
| Reference missing or resolver failed | `ENV_REFERENCE` | `Failed` — no value or reference text | Configure the daemon resolver and restore the referenced value. |
| Resolved entry-count or encoded-block limit exceeded | `ENV_SNAPSHOT_LIMIT` | `Failed` — no value or reference text | Remove variables or reduce values before retrying. |
| Monty raises during the feed | `HOST_RAISED` | `Failed` | The detail is the exception message. |
| `cron_stop` return value is not a valid stop reply | `HOST_REPLY` | `Failed` — detail is `stop check: HOST_REPLY:` then the parse error | Fix hook source; return an object with string `"action"` (`continue`, `disable`, or `delete`) and a non-empty `"reason"` when disabling or deleting. |
| Monty feed task join failed (cancelled blocking-pool task) | `FEED_TASK_FAILED` | `Failed` | Retry the run; if it persists, inspect blocking-pool capacity and hook source. |
| Monty feed task panicked (worker join) | `FEED_JOIN_FAILED` | `Failed` | Fix the panic in the hook or host and retry the schedule. |
| Monty due task panicked or was cancelled | `DUE_JOIN_FAILED` | `Failed` | Retry the schedule; if it persists, inspect the Monty due task and hook source. |
| Monty due path used for a non-Monty hook | `MONTY_RESOLVE` | `Failed` | Run this row on its Rust or Python target, or set `HookTarget::Monty`. |
| `cron_tick` returned a stop action (disable/delete) | `MONTY_TICK` | `Failed` | Return normally from `cron_tick`; disable and delete belong in `cron_stop`. |
| After the CLI returns, host `Instant` exceeds the row budget |  | `TimedOut`, session dropped | Reduce compose work or raise the row deadline; the VM clock was paused during the host call. |
| Monty feed-time-limit while the VM is executing (not during the host suspend) |  | `TimedOut` | Same checkpoint path as a Monty feed that never calls `render_template`. |

A feed that never calls `render_template` still runs when the compose program is absent. The blocking-pool thread stalls until the compose `Command` returns. After that child exits, a host `Instant` past the row budget records bare `RunOutcome::TimedOut` (no code token in the stored outcome). Stdout and stderr are each kept up to `MAX_HOOK_OUTPUT_BYTES`. Stderr copied into a failure detail uses the same 512-byte slice as a Python hook. Failed run details use `CODE: message; recovery`. [`RunOutcome::failure_parts`](../../crates/sc-cron/src/run_failure.rs) parses that layout. The default compose program basename is `sc-compose` on `PATH`; the embedder overrides it with [`MontyHostConfig`](../../crates/sc-cron/src/monty_host.rs).

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

`SqliteScheduleStore` is the production store. `ScheduleStore::limits` defaults to `ScheduleLimits::default()` (unlimited). That store returns the ceiling it was opened with. The crosswalk retains this seam with the copy (REQ-CRON-0009). The public module `process` and `PythonHook` also stay public. They are the retained REQ-CRON-0018 supervisor port (`ChildSpec`, `supervise`, and the related supervisor types). That surface is not an sc-runtime HTTP route, MCP tool, or product API.

## Public exports

The crate-root exports, also listed on `boundaries/sc-cron/sc-cron.toml`, are the contract: `Clock`, `CronError`, `CronExpr`, `DEFAULT_RELOAD_POLL`, `DotenvSource`, `DueRun`, `EnvironmentReferenceError`, `EnvironmentReferenceRequest`, `EnvironmentReferenceResolver`, `EnvName`, `EnvOperationRef`, `Generation`, `HOOK_PROTOCOL`, `HookCall`, `HookEnvironment`, `HookEnvironmentHost`, `HookEvent`, `HookOutput`, `HookStatus`, `HookTarget`, `is_identifier`, `JitterSource`, `Job`, `kill_budget`, `ManualClock`, `MAX_ENV_DOCUMENT_BYTES`, `MAX_ENV_DOTENV_SOURCES`, `MAX_ENV_ENTRIES`, `MAX_ENV_FILE_BYTES`, `MAX_ENV_NAME_BYTES`, `MAX_ENV_PATH_BYTES`, `MAX_ENV_REFERENCE_BYTES`, `MAX_ENV_SNAPSHOT_BYTES`, `MAX_ENV_VALUE_BYTES`, `MAX_EXPR_LEN`, `MAX_HOOK_OUTPUT_BYTES`, `MAX_HOOK_SOURCE_BYTES`, `MAX_NAME_LEN`, `MAX_PAYLOAD_BYTES`, `MAX_TIMEOUT`, `MissedTickPolicy`, `MontyHostConfig`, `NewSchedule`, `process`, `PythonHook`, `RunFailureParts`, `RunOutcome`, `RunTimeout`, `Schedule`, `ScheduleId`, `ScheduleLimits`, `ScheduleName`, `ScheduleSpec`, `ScheduleStore`, `Scheduler`, `SCHEMA_VERSION`, `SeqJitter`, `SqliteScheduleStore`, `StopDecision`, `StoreError`, `StoreErrorKind`, `SystemClock`, `SystemJitter`, `Timestamp`, and `WorkerHandle`. `ManualClock` and `SeqJitter` need feature `test-utils`. `MontyHostConfig` needs feature `monty`.

A cache-sync adaptor in the consumer repo exposes the sync methods. The schedule row points at one of them. This crate does not know whether that method talks to beads or to Postgres.

## Error

`CronError` is `#[non_exhaustive]` and implements `Display` and `std::error::Error`. Variants: `BadExpression(String)`, `InvalidSpec(String)`, `AlreadyActive(ScheduleName)`, `NotFound(ScheduleName)`, `HookFailed(String)`, `Store(StoreError)`, `Fenced { id, generation }`. Codes: `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, `FENCED`. `StoreError` is this crate's error for the schedule table. It is not the schema crate's `StoreError`. It wraps `rusqlite::Error` through `Error::source` and uses the code `STORE_FAILED`. The `BadExpression` recovery text tells the caller to use a five-field string, a six-field seconds-first string, or `@hourly` through `@yearly`.

`Scheduler::open` is the production path. It takes the path of the schedule database and a `ScheduleLimits`, opens the store with that value, and returns `CronError::InvalidSpec` when that value does not match the opened store's `limits()`. The comparison is the store this call opened, not another writer's ceiling. The ceiling is not written into the file. `Scheduler::new` reads `ScheduleStore::limits` and does not take a second copy. `Scheduler::limits` reads the same store value, and the store enforces it when a new name is inserted. That file is not the application's `store-sqlite` file.

## What this crate is not

It does not open Postgres, call beads, or ship an HTTP, MCP, or CLI API. The consumer may add that API and have it insert rows. beads-cache's hook calls beads. Another consumer's hook is the sync method on its adaptor.
