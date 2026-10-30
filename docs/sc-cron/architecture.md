# sc-cron Architecture

**Status:** Active as of 2026-10-28. Binding decisions: [ADR-RUN-0013](../architecture.md) and [ADR-RUN-0016](../architecture.md). Sprints `b-3` through `b-17` establish the contract below.

Port source: beads-cache `integrate/phase-b` commit `d5fbed94f6c3b7fa9e5a562787964a5e6f298423`.

## What this crate is

An optional general-purpose scheduler for application-defined jobs. The consumer adds one SQLite table. A row is a cron string plus a hook. The crate fires that hook. A richer start/stop API is something the consumer may build on top.

## Schedule strings

A normal string has five fields, minute first, in UTC: `minute hour day-of-month month day-of-week`. Example: `*/5 * * * *`.

`@yearly`, `@monthly`, `@weekly`, `@daily`, and `@hourly` are valid.

A six-field string is seconds first: `second minute hour day-of-month month day-of-week`. Example: `0 */5 * * * *`. A five-field string is never read as seconds.

## Configuration table

One table, created by the crate. Columns cover the schedule name, the cron string, the timeout, a JSON payload, and the hook target. The rest of the database belongs to the consumer. The connection is not `SqliteWriter`.

The first published schema is `SCHEMA_VERSION = 1`. Its table contains the
required usable columns, defaults, constraints, and indexes, including
canonical `hook_environment` JSON with empty default
`{"dotenv":[],"variables":{}}`. Validation checks the version and structural
requirements needed to read/write schedules, not exact SQL serialization or
column order. There is no migration ladder or backwards-compatibility promise
for unreleased development files. An incompatible development database is
rejected with recreate guidance without modifying it; future compatibility
starts with the released schema.

The number of schedule rows is not capped unless the embedder passes [`ScheduleLimits`](../../crates/sc-cron/src/limits.rs) ([ADR-RUN-0014](../architecture.md)). `max_schedule_rows: None` is the library default and means unlimited. The count is every row that has not ended, enabled and disabled together. Replacing a name that is already counted does not change the count. Ending a row (`Scheduler::stop`, including a stop-check delete) frees a slot in that same store transaction. A new name over the limit is `CronError::InvalidSpec`; the detail names the limit and the current count. There is no new `CronError` variant. Lowering the limit does not delete rows. Each row's name, expression, timeout, module path, source, and payload are capped in the crate. sc-cron does not depend on sc-config. The ceiling is the `ScheduleLimits` that process passed when it opened the store. It is not written into the schedule database. Every writer on one schedule database must open with identical `ScheduleLimits`. A writer opened with a different ceiling, including unlimited, does not enforce another writer's cap.

Generated applications ship sc-config key `cron.max_schedule_rows` (integer or JSON null, default `10` when the key is absent, `0` or `null` means unlimited) in `template/config/default.json`. [`template/config/cron.schema.json`](../../template/config/cron.schema.json) checks that object and allows every other key in the file, so the real default document validates. cargo-generate `exclude` copies a file without Liquid rendering and does not omit it. The cron schema is on `[template] ignore`, so a generated project does not contain it and does not contain `answers.schema.json`. The daemon reads the key into `cron::CronConfig` on every layout. No layout opens `Scheduler` yet. The layout that does maps the value and opens the scheduler:

```rust
let limits = sc_cron::ScheduleLimits::with_max_schedule_rows(
    config.cron.schedule_limits_fields().max_schedule_rows,
);
let scheduler = sc_cron::Scheduler::open(cron_db, limits, clock, jitter)?
    .with_monty_hosts(host_registry);
```

`host_registry` is an application-built `MontyHostRegistry`. Its default is empty; the application opts into CLI adapters or registers custom handlers. sc-cron receives typed configuration and never opens the application's configuration file. See the host contract below for executable and working-directory policy.

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

`Rust` calls the consumer's `Job` in process. `Python` runs `python3 <module> <hook-name>`. `Monty` is application-supplied source, not a cache path, and is available only with feature `monty`; it uses neither `monty-pool`, `monty-fs`, nor `monty-runtime`. Python and Monty receive a `sc-cron.hook.v1` object containing `protocol`, `event`, `schedule`, `payload`, and a microsecond UTC `deadline`; it has no beads, bd-cache, or ATM field.

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
output, Monty exception/return text, and both CLI adapters' stdout/stderr. Raw public
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
Python and both CLI adapter basenames are resolved through daemon `PATH`/Windows
`PATHEXT` before the child environment is cleared; absolute paths are retained,
and relative paths with components are made absolute against the captured
daemon current directory.
Full ambient inheritance remains available only through explicit direct-call
`ChildEnv::Inherit`. Monty resumes `Getenv` and `GetEnviron` from the same map;
missing or cleared names behave as absent. Source initialization uses a
stage-specific resume driver: top-level environment reads work, but every
registered host call, including sc_compose_run and atm_run, is denied there.
VM-local computation remains permitted. Normal
retained-module semantics apply to a value copied into a global. Every other
Monty OS call remains denied. Both CLI adapter children receive the same
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

### Application-owned Monty hosts

Monty hosts are application-owned registrations in an immutable `MontyHostRegistry` installed with `Scheduler::with_monty_hosts`. The empty registry registers no application call. `MontyHostHandler: Send + Sync` accepts a `MontyHostCall` containing its name, positional/keyword JSON arguments, schedule name, hook phase, and row deadline and returns `Result<serde_json::Value, CronError>`; matching closures are supported. Registration rejects invalid identifiers, duplicate names, and reserved `__hook`, `cron_stop`, and `cron_tick` bindings with `InvalidSpec`. Source can invoke registered functions but cannot install or replace handlers. An unregistered name fails with `HOST_UNDEFINED`. Bridge argument conversion and supplied CLI adapter shape rejection fail with `HOST_DENIED`; a custom handler's returned error, including `CronError::InvalidSpec`, records `HOST_CALL_FAILED` after configured-value redaction. Private typed dispatch preserves supplied-adapter errors without parsing custom error prose. The host checks the row deadline before and after a synchronous handler, but cannot preempt arbitrary application code.

Phase B supplies opt-in `sc_compose_run(subcommand, args)` and `atm_run(subcommand, args)` adapters through this same registry. `ScComposeHostConfig` and `AtmHostConfig` select non-empty executable paths (defaults `sc-compose` and `atm`) and working directories (default the daemon directory captured at configuration). Each adapter accepts one command/subcommand string and an optional list of string arguments, with no keyword arguments. It forwards the non-empty command/subcommand followed by those ordered arguments directly as argv, never through a shell. Every command/subcommand is supported; flags including `--root` are forwarded unchanged. Neither adapter forces render arguments or temp-root staging. Scripts choose arguments, not the configured executable or working directory. Both adapters share the existing blocking child driver in `monty_compose`, generalized for argv/cwd, with row deadline, process-group termination, and `MAX_HOOK_OUTPUT_BYTES` per stream. They do not call the async `process::supervise` from the blocking feed thread or introduce a runtime/`block_on` bridge. A completed child returns `{stdout, stderr, exit_code}`, including nonzero exits; script code decides how that result affects its hook. A deadline expiration records `TimedOut`; spawn, decoding, and capture-limit failures record stable host errors. The adapters receive the resolved environment snapshot from ADR-RUN-0016. Neither CLI is required when its adapter is unused. `render_template` is not a product requirement; an internal helper may remain an implementation convenience without becoming a mandatory registered host function.

MVP accepts application-supplied Monty source without signatures, a separate script-approval mechanism, or a consumer option enabling such a mechanism. Capability registration and environment policy are enforced independently. In-process Monty and custom Rust handlers retain stack-overflow, allocator-abort, panic, and non-cooperative deadline risks; they are not an isolation boundary for hostile code.

The production `drive_feed` loop dispatches calls by the configured registry. Its cfg(test) entry points drive the same dispatcher. CLI supervision retains process-group kill, per-stream capture caps, and bounded completion under the row deadline. A feed without a CLI registration/call works with neither executable installed. VM time limits remain interpreter-checkpoint based; synchronous custom handlers must cooperate with the deadline. The seven public CronError variants remain unchanged.

Host diagnostics and recovery are single-sourced in [REQ-CRON-0005](requirements.md#req-cron-0005-one-error-enum); no render-specific subcommand rejection or temp-root staging error belongs to the general adapters. Both CLI streams and custom-handler errors pass through the configured-value redaction boundary before scheduled persistence or logging.

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

`Scheduler::new_with_logger(store, clock, jitter, logger)` retains the supplied
generic components and installs scheduler/worker/hook logging. Supplied store
and jitter implementations own their internal logging. Legacy `new` delegates
with the fixed Noop. `open_with_logger(path, limits, clock, logger)` constructs
the logger-aware SQLite store and `SystemJitter`, then delegates generic
construction with the same handle. The generic constructor emits
`cron.scheduler/logger.attached` once; the convenience path does not duplicate
it. No logging wrapper, new component trait, or post-construction setter is
required.

The `0.1.1` injection implementation adds `PythonHook::new_with_logger`
and `process::supervise_with_logger` (returning `CronError`). Legacy hook and
component constructors install a fixed `sc-cron` Noop; both supervisor entry
points return `CronError` with their existing argument lists. `RunCoord` owns the shared Arc used by
`share()`, workers and Monty feeds. `new_with_logger` preserves caller-owned
store, clock and jitter; the SQLite/SystemJitter open helper shares one Arc
across all built-ins and emits `cron.scheduler/logger.attached` exactly once.
No library path flushes or shuts down the full runtime. Component rejection
preserves zero jitter, corrupt-row skipping, process capture, Monty outcomes
and shutdown decisions. The five component rows and both shutdown rows are
verified by `observability_components`; b-16 owns the future child-kill row.

## Public exports

The crate-root exports, also listed on `boundaries/sc-cron/sc-cron.toml`, are the contract: `Clock`, `CronError`, `CronExpr`, `DEFAULT_RELOAD_POLL`, `DotenvSource`, `DueRun`, `EnvironmentReferenceError`, `EnvironmentReferenceRequest`, `EnvironmentReferenceResolver`, `EnvName`, `EnvOperationRef`, `Generation`, `HOOK_PROTOCOL`, `HookCall`, `HookEnvironment`, `HookEnvironmentHost`, `HookEvent`, `HookOutput`, `HookStatus`, `HookTarget`, `is_identifier`, `JitterSource`, `Job`, `kill_budget`, `ManualClock`, `MAX_ENV_DOCUMENT_BYTES`, `MAX_ENV_DOTENV_SOURCES`, `MAX_ENV_ENTRIES`, `MAX_ENV_FILE_BYTES`, `MAX_ENV_NAME_BYTES`, `MAX_ENV_PATH_BYTES`, `MAX_ENV_REFERENCE_BYTES`, `MAX_ENV_SNAPSHOT_BYTES`, `MAX_ENV_VALUE_BYTES`, `MAX_EXPR_LEN`, `MAX_HOOK_OUTPUT_BYTES`, `MAX_HOOK_SOURCE_BYTES`, `MAX_NAME_LEN`, `MAX_PAYLOAD_BYTES`, `MAX_TIMEOUT`, `MissedTickPolicy`, `MontyHostRegistry`, `MontyHostHandler`, `MontyHostCall`, `ScComposeHostConfig`, `AtmHostConfig`, `NewSchedule`, `process`, `PythonHook`, `RunFailureParts`, `RunOutcome`, `RunTimeout`, `Schedule`, `ScheduleId`, `ScheduleLimits`, `ScheduleName`, `ScheduleSpec`, `ScheduleStore`, `Scheduler`, `SCHEMA_VERSION`, `SeqJitter`, `SqliteScheduleStore`, `StopDecision`, `StoreError`, `StoreErrorKind`, `SystemClock`, `SystemJitter`, `Timestamp`, and `WorkerHandle`. `ManualClock` and `SeqJitter` need feature `test-utils`. `MontyHostRegistry`, `MontyHostHandler`, `MontyHostCall`, `ScComposeHostConfig`, `AtmHostConfig` need feature `monty`.

A cache-sync adaptor in the consumer repo exposes the sync methods. The schedule row points at one of them. This crate does not know whether that method talks to beads or to Postgres.

## Error

`CronError` is `#[non_exhaustive]` and implements `Display` and `std::error::Error`. Variants: `BadExpression(String)`, `InvalidSpec(String)`, `AlreadyActive(ScheduleName)`, `NotFound(ScheduleName)`, `HookFailed(String)`, `Store(StoreError)`, `Fenced { id, generation }`. Codes: `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, `FENCED`. `StoreError` is this crate's error for the schedule table. It is not the schema crate's `StoreError`. It wraps `rusqlite::Error` through `Error::source` and uses the code `STORE_FAILED`. The `BadExpression` recovery text tells the caller to use a five-field string, a six-field seconds-first string, or `@hourly` through `@yearly`.

`Scheduler::open` is the production path. It takes the path of the schedule database and a `ScheduleLimits`, opens the store with that value, and returns `CronError::InvalidSpec` when that value does not match the opened store's `limits()`. The comparison is the store this call opened, not another writer's ceiling. The ceiling is not written into the file. `Scheduler::new` reads `ScheduleStore::limits` and does not take a second copy. `Scheduler::limits` reads the same store value, and the store enforces it when a new name is inserted. That file is not the application's `store-sqlite` file.

## What this crate is not

It does not open Postgres, call beads, or ship an HTTP, MCP, or CLI API. The consumer may add that API and have it insert rows. beads-cache's hook calls beads. Another consumer's hook is the sync method on its adaptor.

The implemented registry builder uses `register(name, handler)`,
`with_sc_compose(ScComposeHostConfig)`, and `with_atm(AtmHostConfig)`, each
returning `Result<MontyHostRegistry, CronError>`. The generated development
example and custom closure setup are in [monty-generated-example.md](monty-generated-example.md).
The b-16 child environment is explicitly empty; the resolved-snapshot text
above is the b-17 successor contract.
