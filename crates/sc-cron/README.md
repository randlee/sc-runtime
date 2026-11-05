# sc-cron

A SQLite-backed cron scheduler for tokio, with generation fencing, per-schedule
jitter and missed-tick handling.

`sc-cron` runs a caller's `Job` on cron schedules kept in a `ScheduleStore`.
It was extracted from an earlier in-house scheduler and carries no
consumer-specific type: a schedule's input is an opaque JSON payload, and every
failure is a `CronError` with a stable `code()`. Its temporary workspace edge is defaults-off `sc-command` for the nominal
canonical logger contract. Supporting event values come from registry
`sc-observability-types`; this does not activate command servers or the full
observability implementation.

- **Validated input.** `ScheduleName` (at most 128 bytes), `CronExpr` (in UTC:
  standard five-field crontab, minute first; or the `cron` crate's six or
  seven fields with seconds first; or `@hourly`, `@daily` and the like),
  `RunTimeout` (more than zero, at most 24 h) and the JSON payload (at most
  64 KiB).
- **Start.** `Scheduler::start` checks that the expression has a future
  occurrence before anything is written, draws a fixed jitter for the
  schedule and stores its first run time.
- **The worker.** `Scheduler::spawn_worker` starts one tokio task that runs
  every active, enabled schedule. Before each run it asks
  `Job::should_stop`, which answers `Continue`, `Disable(reason)` or
  `Delete(reason)`. It bounds the stop check and the run by the schedule's
  timeout on the monotonic clock (recording `RunOutcome::TimedOut` without
  delaying the next tick), skips a tick while the previous run is still in
  flight, and handles missed ticks per `MissedTickPolicy` (`Coalesce` by
  default, or `Skip`).
- **Enabled flag.** `Scheduler::set_enabled(name, bool)` turns a schedule off
  and on again without rewriting its row; a disabled schedule never fires.
- **Python hook target.** A row whose hook is `Python { module }` runs
  `python3 <module> cron.stop` and `python3 <module> cron.tick` from a
  temporary directory, with one JSON object on stdin. The object is
  `sc-cron.hook.v1` with `protocol`, `event`, `schedule`, `payload`, and
  `deadline`. Every hook is killed, with everything
  it started, at the schedule's timeout capped by the time left before the
  run's deadline, and its output is capped. A `Rust` row calls the job
  passed to `Scheduler::spawn_worker`.
- **Fencing.** Every run-state write names `(id, generation)`. A run from an
  older generation is fenced and records nothing, so a stale run never
  writes over a restarted schedule.
- **Stores read no clock.** Every instant is a `Timestamp` argument.
  `SqliteScheduleStore` is the rusqlite implementation (SQLite bundled), on a
  database file of its own (`PRAGMA user_version` = `SCHEMA_VERSION`, 1).
  The complete first-release schema includes canonical `hook_environment`.
  Incompatible development tables are rejected with recreate guidance.
  `list_active` skips a row that does not decode and emits the canonical
  `cron.store` / `row.corrupt` event, so other schedules keep running; `get`
  on that row still returns the `Corrupt` error. Observing the event requires
  a logger-aware store/open construction; legacy constructors install a fixed
  Noop logger.
- **Injected time and randomness**, through `Clock` and `JitterSource`.

## Example

The crate's `Job` trait uses `async-trait`, so a caller needs `async-trait`,
`serde_json` and `tokio` as well.

```rust,no_run
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sc_cron::{
    Job, RunOutcome, ScheduleLimits, ScheduleSpec, Scheduler, SqliteScheduleStore, StopDecision,
    SystemClock, SystemJitter,
};

struct Sync;

#[async_trait::async_trait]
impl Job for Sync {
    async fn run(&self, payload: &serde_json::Value, deadline: Instant) -> RunOutcome {
        let _ = (payload, deadline); // do the work, finishing by `deadline`
        RunOutcome::Ok
    }

    async fn should_stop(&self, _payload: &serde_json::Value) -> StopDecision {
        StopDecision::Continue
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), sc_cron::CronError> {
    let limits = ScheduleLimits {
        max_schedule_rows: NonZeroUsize::new(10),
    };
    let scheduler = Scheduler::open("cron.db", limits, Arc::new(SystemClock), Arc::new(SystemJitter))?;
    // Every 30 seconds; each run (stop check included) is cut off after 25 s.
    let spec = ScheduleSpec::new("sync", "*/30 * * * * *", Duration::from_secs(25), serde_json::json!({}))?;
    scheduler.start_async(spec).await?;
    let worker = scheduler.spawn_worker(Arc::new(Sync));
    // ... later
    worker.shutdown(tokio::time::Instant::now() + Duration::from_secs(30)).await;
    Ok(())
}
```

Runtime notes:

- `Scheduler::start` and `Scheduler::stop` call the store synchronously. From
  async code, use `start_async` and `stop_async`, which run on tokio's
  blocking pool.
- Run timeouts are cooperative: a job should finish by the `deadline` it is
  handed. Use a multi-thread tokio runtime so a misbehaving job cannot stall
  the worker.
- Stop the worker by awaiting `WorkerHandle::shutdown`. Dropping the handle
  drains detached, and in-flight outcomes can be lost at process exit.
  A shutdown deadline cancels and joins cooperative runs; the worker itself
  remains owned until all runs and its active blocking store call finish.
  `false` means the deadline was missed, not that ownership was detached.
  Physical completion can exceed the deadline for a store or native job that
  does not cooperate; Tokio cannot forcibly preempt running blocking work.
  Await shutdown to completion before consuming the application's logger.

## Feature flags

| Feature | Enables |
|---|---|
| `test-utils` | `ManualClock` (a deterministic clock whose parks tell a test the worker has settled) and `SeqJitter` (a scripted jitter source) |
| `monty` | Trusted in-process source hooks and empty-default application host registry; requires Rust 1.96.1. |

Build a `MontyHostRegistry` with `.register("jev_call", handler)?` for custom
JSON-shaped callbacks. Install it with `Scheduler::with_monty_hosts(registry)`.
Arguments and results preserve the complete JSON `i64`/`u64` integer domain,
including nested arrays and objects, as exact Monty integers. Monty integers
outside that domain are rejected with `HOST_DENIED` and range guidance.
Register optional `sc_compose_run` or `atm_run` with `.with_sc_compose(config)?`
or `.with_atm(config)?`; `ScComposeHostConfig` and `AtmHostConfig` choose the
executable and cwd. Both forward any command and argv without a shell, clear
the child environment and install the shared occurrence snapshot, enforce row
deadlines/group cleanup and cap each stream.
Completed nonzero exits are returned to the script. No CLI is required by an
empty registry or a custom application handler. `render_template` is not a
product API. See [the generated example](../../docs/sc-cron/monty-generated-example.md)
for API, result/error shapes, development generation, and in-process limits.

## Shared hook environment

Build schedule policy with `HookEnvironment::new().set(...)?`, `.reference(...)?`,
`.clear(...)?` and ordered absolute `.dotenv(path, required)?` sources, then
attach it with `ScheduleSpec::with_environment`. Nonempty Rust policy is invalid.
Inject administrator authorization with `Scheduler::with_hook_environment` and
`HookEnvironmentHost`: process-name, exact-file and directory-root allowlists
are empty by default, and an application may attach an
`Arc<dyn EnvironmentReferenceResolver>`.

One immutable snapshot resolves daemon values, ordered files and schedule
operations before stop; tick, Monty initialization and both CLI adapters reuse
it. Python defaults to an empty `ChildEnv::Only`; direct callers may opt into
Inherit. Source initialization permits environment reads and denies application
host calls. Retained module globals initialize once per generation; later getenv
reads use the current occurrence. Values in scheduled details/reasons are
redacted before fixed diagnostics are assembled. Explicit Set values are
intentional plaintext configuration; resolved values never persist in registry
writes. Direct `HookOutput` fields remain raw and Debug output is value-free.

## Links

- Repository: <https://github.com/randlee/sc-runtime>
- Architecture: [`docs/sc-cron/architecture.md`](../../docs/sc-cron/architecture.md)
- Requirements: [`docs/sc-cron/requirements.md`](../../docs/sc-cron/requirements.md)
- sc-compose hooks: [`docs/sc-cron/hooks-sc-compose.md`](../../docs/sc-cron/hooks-sc-compose.md)

MSRV: Rust 1.94.1 (edition 2024) for the default build; enable `monty` only on Rust 1.96.1 or newer. License: MIT.

## Cooperative consumers and physical ownership

Use `Scheduler::spawn_worker_cooperative` with a store declaring
`StoreOperationSupport::Cooperative`. Obtain its independently cloneable
`WorkerStopControl` before moving the single `WorkerHandle` into the physical
join owner. Its `operation_context()` is the shared scope for
`start_with_context`, `stop_with_context`, `query_with_context`, or the native
`CronOperations` port. A child `with_deadline` keeps the earliest deadline and
the same live cancellation state.

```rust,no_run
# async fn example() -> Result<(), sc_cron::CronError> {
# use std::{sync::Arc, time::{Duration, Instant}};
# use sc_cron::*;
# struct ApplicationJob;
# #[async_trait::async_trait]
# impl Job for ApplicationJob {
# async fn run(&self, _: &serde_json::Value, _: Instant) -> RunOutcome { RunOutcome::Ok }
# async fn should_stop(&self, _: &serde_json::Value) -> StopDecision { StopDecision::Continue }
# }
let scheduler = Scheduler::new(
    Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default())?),
    Arc::new(SystemClock), Arc::new(SystemJitter),
).with_command_capacity(64)?;
let worker = scheduler.spawn_worker_cooperative(Arc::new(ApplicationJob))?;
let stop = worker.stop_control();
let context = stop.operation_context();
let row = scheduler.start_with_context(
    ScheduleSpec::new("sync", "*/5 * * * *", Duration::from_secs(10),
        serde_json::json!({}))?, context.clone(),
).await?;
let page = scheduler.query_with_context(ScheduleQuery {
    selection: ScheduleSelection::All, limit: 100, ..Default::default()
}, context.clone()).await?;
let _ = (row, page);
let now = Instant::now();
stop.request_stop(ShutdownBudget::new(
    now + Duration::from_secs(5), now + Duration::from_secs(10),
)?)?;
let report = worker.join_owned().await?;
let _ = report;
# Ok(())
# }
```

The default command capacity is 64 shared by every handle and executor using
that scheduler. The pre-start override accepts `1..=65536`; any worker or
scheduling operation latches the configuration. Admission refuses full intake
immediately with `Unavailable` / `command_capacity_exhausted` and
`NoMutationAttempted` evidence. There is no capacity waiter queue. A cancelled
transport does not release its permit: the retained owner joins the blocking
closure first. Private worker bookkeeping and terminal cleanup have independent
admission; they cannot serve as public overload bypasses.

Queries return active, ended, or all current rows, optionally by exact name.
Limits are `1..=1000`, ids ascend, and `next_after_id` is an exclusive continuation.
This is current row state, rather than an execution-history table. Ending always
names the expected generation; an old request cannot end a replacement.

First stop closes intake and run-launch admission. Ordinary operations already
admitted can drain until the absolute grace cutoff G. Only generation-fenced
terminal bookkeeping can enter the private cleanup scope until K. A second stop
cancels ordinary work immediately and can only shorten K. Equal G/K is valid;
zero budget admits no new commit or cleanup. Stop time uses real `Instant`,
independent of the scheduling clock. Status subscriptions publish expiry without
waiting for the join owner.

The built-in SQLite provider propagates the scope through connection acquisition,
busy/progress callbacks, row decoding, and every write. Its final commit-hook
admission serializes briefly with stop; the controller lock is released before
native COMMIT continues. Known hooks and calling-thread scope are restored on
safe paths. Uncertain persistence quarantines the connection; any reconciliation
must be separately authorized, using a separate live scope/connection. No expired
operation performs a readback or automatically replays a mutation.

`StoreError::transaction_outcome()` distinguishes admission from physical
persistence. An admitted commit can finish after G/K and still be `Committed`
with `completed_after_cutoff = true`; it remains a deadline failure. A veto with
observed rollback is `RolledBack`. Missing evidence stays `Indeterminate`, never
false, not-found, or “safe to retry.” The lost-result test injects missing result
evidence; it does not reproduce a physical fsync failure. `join_owned` retains
physical ownership even after a cutoff. Neither SQLite native commit/fsync nor
arbitrary Rust jobs, native code, or synchronous custom Monty hosts receive a
hard physical completion bound.

## Consumer-owned schedule schema

The complete runnable example is
[`ApplicationStore`](../../tests/integration/cron-consumer/src/lib.rs), exercised
by [`application_schema.rs`](../../tests/integration/cron-consumer/tests/application_schema.rs).
Run it from the repository root:

```sh
cargo test --manifest-path tests/integration/cron-consumer/Cargo.toml --test application_schema
```

The example implements the public `ScheduleStore` port. It keeps the bundled
schedule database unchanged and persists an extra `tenant` field, keyed by
schedule id and generation, in its own application database:

```sql
CREATE TABLE application_schedule (
    schedule_id TEXT PRIMARY KEY,
    generation INTEGER NOT NULL,
    tenant TEXT NOT NULL
);
```

`ApplicationStore::create` forwards the schedule write and persists the tenant.
Its `query` forwards bounded current-row queries; `stop` forwards the existing
generation fence. `tenant(id, generation)` reads the application field without
adding application data to core `Schedule` or command DTOs. Tests schedule,
query, end, restart, reject a stale generation, and reopen both databases to
verify persistence.

This example deliberately inherits the **legacy caller-owned lifecycle**.
Its two databases are not one atomic transaction; the consumer owns reconciliation
if its application write fails after the schedule write. Cooperative admission
rejects this provider before invoking work. Call its synchronous public scheduler
methods from an application-owned blocking task and retain/join that task yourself.
The example does not claim the bundled cooperative shutdown contract.

A forwarding cooperative provider must forward both `operation_support` and
`with_operation_context`, preserve bounded query/fencing, and propagate the same
live scope through **all** application persistence. For example, forwarding only
the bundled operations while performing an unscoped metadata write would be an
incorrect cooperative promise. Provider-specific cancellation/admission mechanisms
are supported; SQLite hooks are not a requirement for a different store. The
bundled row schema is an invariant of `SqliteScheduleStore`, while a consumer-owned
schema remains an open extension point.

Applications may instead implement `CronOperations` or provide their own
`CronExecutor` at the command projection seam. That transfers finite intake,
physical joins, cancellation/deadline policy, generation conflicts, persistence
accuracy, target authorization, and error mapping to the application. Budget
values alone do not make arbitrary custom code cooperative. Direct legacy store
and scheduler calls remain caller-owned; await legacy `WorkerHandle::shutdown`
fully to join its worker, and independently join any direct legacy store tasks.

The isolated consumer's default and `--features monty` execution tests use only
`sc-cron` as their SDK, including `sc_cron::observability` for the exact nominal
receiver/event/service/error types. Runtime and test utilities are explicit. Its
normal graph guard checks default Monty exclusion and excludes full observability,
command-server, clap, Axum, RMCP, runtime, and tool crates:

```sh
cargo test --manifest-path tests/integration/cron-consumer/Cargo.toml
cargo test --manifest-path tests/integration/cron-consumer/Cargo.toml --features monty
python3 tests/integration/cron-consumer/graph_guard.py
```

For a custom `CronOperations`, create `WorkerStopControl::new()?` inside the
owning Tokio runtime and pass `control.operation_context()` to the adapter.
This ordinary-feature factory does not spawn a framework worker. It shares
live cancellation and earliest child deadlines. Stop can be requested from
another ordinary thread while the creating runtime remains alive. The
application owns command admission and every task/native call/cleanup/join in
its provider: close intake, request stop, physically join that work, then call
`control.finish_application_shutdown()?` to acknowledge `Joined`. This call
cannot verify untracked custom work. Framework-worker controls refuse it and
require `WorkerHandle::join_owned()`. Neither completion nor stop reopens
intake. See the [ordinary-feature custom context example](../../tests/integration/cron-consumer/custom-context/src/main.rs).
