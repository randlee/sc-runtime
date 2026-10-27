# Crosswalk: beads-cache REQ-CRON onto sc-runtime

**Status:** Planning record for bead `run-zif.3` (parent `run-zif`).  
**Date:** 2026-10-26  
**Sources:** beads-cache `docs/sc-cron/requirements.md` (`REQ-CRON-0001` through `REQ-CRON-0018`, including `0014` and `0015`) and `crates/sc-cron/src/error.rs` on beads-cache `integrate/phase-b`. sc-runtime `docs/sc-cron/requirements.md` (`REQ-CRON-0001` through `REQ-CRON-0005`), `docs/sc-cron/architecture.md`, and [ADR-RUN-0013](../../architecture.md). Sprint b-3 (PR #68, bead `run-9ny`) implements the shorter contract. This file does not amend that PR.

Marks:

| Mark | Meaning |
|---|---|
| covered | The b-3 port or an sc-runtime requirement or ADR already requires the behavior. |
| narrowed | The sc-runtime written contract keeps a smaller rule on purpose. |
| deferred | An sc-runtime follow-on owns it. It is not a beads-cache requirement of this port. |
| missing | The beads-cache text and the sc-runtime text disagree, and the sc-runtime text should be amended after PR #68 merges. |

Where a row is covered only because b-3 copies the crate, the sc-runtime requirement ids do not restate the numeric limits, jitter, missed-tick policy, injected clock, or shutdown drain. That omission is not a second mark.

## Table

| beads-cache | sc-runtime | Mark | Note |
|---|---|---|---|
| `REQ-CRON-0001` Parse before write | Port. Expression shape is `REQ-CRON-0002` and ADR-RUN-0013 point 2. | covered | Name, expression, timeout, and payload limits travel with the copy. `BadExpression` versus `InvalidSpec` travels with the copy. |
| `REQ-CRON-0002` Next run in UTC | `REQ-CRON-0002`, ADR-RUN-0013 point 2 | covered | Five-field minute-first strings, the `@` macros, and the six-field seconds-first compatibility form are the sc-runtime rule. Next-run storage and the jitter window travel with the copy. |
| `REQ-CRON-0003` Jitter | Port | covered | Not restated as its own sc-runtime requirement. |
| `REQ-CRON-0004` Missed-tick policy | Port | covered | `Coalesce` default and `Skip` travel with the copy. |
| `REQ-CRON-0005` Stop check and per-run timeout | `docs/sc-cron/architecture.md` (`StopDecision`, `RunOutcome`) | narrowed | `StopDecision::Stop` is removed. `Delete(reason)` is the end-from-stop-check variant (`Continue`, `Disable(String)`, `Delete(String)`). For **Rust and Python**, the monotonic deadline, shared stop/run timeout, and cooperative cancel (`Worker::expire` abort) travel with the copy. **Monty** (b-9) is exempt: no deadline abort while the feed runs; `TimedOut` only when the feed returns a feed-time-limit error. |
| `REQ-CRON-0006` One run in flight per schedule name | Port | covered | Scheduler behavior of the copy. Not restated as its own sc-runtime id. This is not the bd-cache tick lease. See below. |
| `REQ-CRON-0007` Generation fencing | ADR-RUN-0013 point 6 (`Fenced`, code `FENCED`) | covered | `(id, generation)` writes travel with the copy. |
| `REQ-CRON-0008` Stores read no clock | Port | covered | Not restated. |
| `REQ-CRON-0009` SQLite store | `REQ-CRON-0004`, ADR-RUN-0013 point 3 | covered | One schedule table, its own file, writes that do not go through `SqliteWriter`. The `ScheduleStore` seam travels with the copy. The b-3 port opens at **`SCHEMA_VERSION` 3** with `hook_kind` and `hook_module`. Sprint **b-9** bumps that table to **version 4** (`hook_source`, `monty` kind). |
| `REQ-CRON-0010` Injected time and randomness | Port | covered | `Clock` and `JitterSource` travel with the copy. Feature `test-utils` is not an sc-runtime requirement id. |
| `REQ-CRON-0011` General-purpose scheduler and `CronError` codes | `REQ-CRON-0001`, `REQ-CRON-0005`, ADR-RUN-0013 points 1, 5, and 6 | missing | No beads, bd-cache, or other workspace type is required and covered. Point 6 omits `NotFound` / `NOT_FOUND` and `HookFailed` / `HOOK_FAILED`. See below. |
| `REQ-CRON-0012` Bounded shutdown | Port | narrowed | **Rust and Python:** worker-task abort at the handle deadline when no Monty is outstanding; sequential drain abort with `TimedOut` on cancellation when no Monty is outstanding; `run_due` parent deadline `TimedOut` and `DueTask` abort on drop. **Monty** (b-9): concurrent shutdown wait; `monty_outstanding` from join through outcome write; at handle deadline abort Rust/Python siblings only; `run_due` spawned task registers per-name in-flight (`REQ-CRON-0006`), self-records, `DueTask` forgets on drop. |
| `REQ-CRON-0013` Python hook target | `REQ-CRON-0003`, ADR-RUN-0013 point 4 | narrowed | The port writes the shared **`sc-cron.hook.v1`** object (`protocol`, `event`, `schedule`, `payload`, required `deadline`) for Python; b-9 requires the **same object** for Monty `cron_stop` / `cron_tick`. beads-cache also specifies `<interpreter> -I -m`, 64 KiB output caps, and a process-group kill. b-3 re-reviews the copy if it drifts from `docs/sc-cron/architecture.md`. `HookFailed` on a hook that cannot start is the point 6 gap, not a separate product cut. |
| `REQ-CRON-0014` Enabled flag | Port | covered | The `enabled` column, reload poll, and `NotFound` when no active schedule has that name travel with the copy. The `NOT_FOUND` code is the point 6 doc gap. |
| `REQ-CRON-0015` Five-field expressions | `REQ-CRON-0002`, ADR-RUN-0013 point 2 | covered | Minute-first, UTC, second 0. Crontab day-of-week numbering (Sunday 0 or 7) and the day-of-month / day-of-week OR rule travel with the beads-cache parser the port copies. |
| `REQ-CRON-0016` A hook target per schedule row | `REQ-CRON-0003`, `HookTarget` on `ScheduleSpec` | narrowed | Mixed Rust and Python rows are in both contracts. beads-cache maps a row through `TargetResolver` (`spawn_worker_with`) and says the target stays Rust or Python. sc-runtime b-3 uses one shared `Job` for every Rust row and `Python { module }` on the spec. Monty is outside this beads-cache requirement. |
| `REQ-CRON-0017` `set_enabled_many` and `replace_and_set_enabled` | Port | covered | Both store methods travel with the copy. They are the primitives `REQ-BDC-0720` calls. They are not a two-row mutex inside sc-cron. See [decision-sync-scheduling-at-api.md](decision-sync-scheduling-at-api.md). |
| `REQ-CRON-0018` Public child-process supervisor | Port | covered | `sc_cron::process` (`ChildSpec`, `supervise`, and the related types) travels with the copy for the CPython path. sc-runtime `REQ-CRON-0003` does not make that module its own requirement. In-process Monty is not the child-process supervisor; see the Monty section below. |

## ADR-RUN-0013 point 6 omits two codes

beads-cache `crates/sc-cron/src/error.rs` and `REQ-CRON-0011` define `CronError` as `BadExpression`, `InvalidSpec`, `AlreadyActive`, `NotFound(ScheduleName)`, `HookFailed(String)`, `Store`, and `Fenced`, with codes `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, and `FENCED`.

ADR-RUN-0013 point 6 and `docs/sc-cron/architecture.md` list `BadExpression`, `InvalidSpec`, `AlreadyActive`, `Store`, and `Fenced` only, with codes `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `STORE_FAILED`, and `FENCED`. Point 6 also says those variants "stay ... with the stable codes already on the beads-cache crate." The omitted `NOT_FOUND` and `HOOK_FAILED` codes contradict that sentence. `REQ-CRON-0014` returns `NotFound` for a missing active name. `REQ-CRON-0013` returns `HookFailed` from `PythonHook::invoke` when a hook cannot be started; the worker then records `RunOutcome::Failed`.

**Recommendation.** Sprint **b-9** amends ADR-RUN-0013 point 6 and `docs/sc-cron/architecture.md` so the variant set and codes match the beads-cache crate, including `NotFound` / `NOT_FOUND` and `HookFailed` / `HOOK_FAILED`. Apply the crate edit on the merged port after `run-9ny`. Do not edit PR #68. A deadline stays `RunOutcome::TimedOut`. `HookFailed` stays "the hook could not be started." See [sprint-b-9-sc-cron-monty-hooks.md](sprint-b-9-sc-cron-monty-hooks.md).

## Per-row target and the child-process supervisor

beads-cache `REQ-CRON-0016`: `Scheduler::spawn_worker_with(resolver)` takes a `TargetResolver` that maps each row to a Rust `Job` or a Python module. The worker calls the resolver once per launch. `spawn_worker_for` and `spawn_worker` stay. The same requirement freezes the beads-cache target set at Rust or Python. maturin / PyO3 bindings for sc-cron wait for the move to sc-runtime and are not Monty.

sc-runtime b-3: `HookTarget` is `Rust` or `Python { module }` on `ScheduleSpec`. Every Rust row calls the one `Job` the consumer supplied. The payload tells that job which row fired. A Python row runs `python3 <module> cron.tick` or `cron.stop`.

beads-cache `REQ-CRON-0018`: `sc_cron::process` exports the supervisor bd-cache-sync uses for the post-sync hook and for a Python target's environment. The sc-runtime contract does not require callers to use that module. The port inherits it with the crate. b-3 exposes no HTTP route, MCP tool, or binary.

## Per-schedule in-flight skip and the tick lease

These are different mechanisms. This crosswalk does not fold the product mutex into the scheduler.

| | Per-schedule in-flight skip | Tick lease |
|---|---|---|
| Id | beads-cache `REQ-CRON-0006` | beads-cache `REQ-BDC-0722` |
| Where it lives | sc-cron, one scheduler | bd-cache, the cache file |
| Scope | One schedule name | Every sync run (built-in or Python, scheduled, on demand, or the immediate `sync use-python` run) |
| What a clash does | That tick is skipped and logged. Other names keep running. | The run replies `skipped: "in_flight"` or `sync use-python` fails with `TICK_IN_FLIGHT`. |
| What it is not | A repository-wide sync lock | An sc-cron rule, and not `REQ-CRON-0006` |

beads-cache **REQ-BDC-0720** mutual enable is **temporary** bd-cache behavior under the [owner ruling](decision-sync-scheduling-at-api.md#owner-ruling-no-slot-restriction-2026-10-26), not lasting scheduling policy. sc-cron does not implement that ceiling.

## Monty is outside beads-cache REQ-CRON

`REQ-CRON-0016` says the beads-cache target stays Rust or Python. No beads-cache `REQ-CRON-0001` through `REQ-CRON-0018` row requires Monty, `monty`, or `monty-pool`. Monty is an sc-runtime follow-on: a third hook target beside the in-process Rust `Job` and the CPython subprocess, for fast trusted `cron.stop` and `cron.tick`. beads-cache does not integrate it. The plan sprint is [sprint-b-9-sc-cron-monty-hooks.md](sprint-b-9-sc-cron-monty-hooks.md), blocked on `run-9ny`.
