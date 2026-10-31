# sc-cron

A SQLite-backed cron scheduler for tokio, with generation fencing, per-schedule
jitter and missed-tick handling.

`sc-cron` runs a caller's `Job` on cron schedules kept in a `ScheduleStore`.
It was extracted from an earlier in-house scheduler and carries no
consumer-specific type: a schedule's input is an opaque JSON payload, and every
failure is a `CronError` with a stable `code()`. The crate depends on no
other workspace crate.

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

## Feature flags

| Feature | Enables |
|---|---|
| `test-utils` | `ManualClock` (a deterministic clock whose parks tell a test the worker has settled) and `SeqJitter` (a scripted jitter source) |
| `monty` | Trusted in-process source hooks and empty-default application host registry; requires Rust 1.96.1. |

Build a `MontyHostRegistry` with `.register("jev_call", handler)?` for custom
JSON-shaped callbacks. Install it with `Scheduler::with_monty_hosts(registry)`.
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
