---
id: b-9
phase: b
status: planned
closure_type: boundary
target_boundary: sc-cron
track: cron
wave: 3
branch: sprint/b-9-sc-cron-monty-hooks
worktree: sprint/b-9-sc-cron-monty-hooks
requirements:
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
  - REQ-RUN-0005
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0013
  - ADR-RUN-0012
  - ADR-RUN-0002
owned_paths:
  - crates/sc-cron/
  - boundaries/sc-cron/
  - docs/sc-cron/
  - docs/architecture.md
  - Cargo.lock
must_follow: [b-3]
blocked_by: run-9ny
parallel_safe: [b-8]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One crate. b-3 lands `HookTarget` (Rust and Python `{ module }`) and the one schedule table; this sprint extends those artifacts with Monty and does not hoist them to another boundary. A feature flag keeps Monty off the default build.
---

# Sprint b-9 — Monty hook target for sc-cron

Plan for bead `run-zif.1` (parent `run-zif`). This document is the plan. It contains no crate change. Implementation starts only after this plan is accepted and after `run-9ny` (sprint b-3, PR #68) has merged.

## Slot

Indexed in [phase-b-plan.md](phase-b-plan.md) and [.sprints/b/structure.ttl](../../../.sprints/b/structure.ttl): cron track, wave 3, `must_follow` `b-3`, `parallel_safe` with `b-8`. The stores critical path stays `b-4` → `b-5` → `b-8` and does not wait on `b-9`.

beads-cache sprint `b-9` (comments, provenance, and tests) is a different sprint in the other repo. It does not hold this work, and this work does not hold it.

## Blocker

`blocked_by: run-9ny`. Sprint b-3 ports `crates/sc-cron` and lands `HookTarget::Rust` and `HookTarget::Python { module }`. This sprint reads that merged tree and adds the third target. It does not edit PR #68.

## Goal

Add an optional in-process Monty hook for the same two events the Python hook already runs, `cron.tick` and `cron.stop`, aimed at fast trusted hooks. The default build stays Rust plus CPython and does not link Monty.

## Decisions

### `HookTarget::Monty`

The variant is always present, so a schedule row round-trips on a build that did not enable the feature. Launching it without the feature fails as below. Storing and listing the row still succeed.

```rust
pub enum HookTarget {
    Rust,
    Python { module: std::path::PathBuf },
    /// Source the embedding process already trusts.
    /// sc-cron does not read this from a file.
    Monty { source: String },
}
```

`source` is the hook program text. It is not a filesystem path.

Monty and the ported Python hook receive the same JSON object on each call. Required fields: `protocol` (string, always `"sc-cron.hook.v1"`), `event` (string, `"cron.tick"` or `"cron.stop"`), `schedule` (string, the schedule name), `payload` (object, the row payload), and `deadline` (string, UTC instant with microsecond precision, same encoding the Python path already writes). The object has no beads, bd-cache, or ATM field.

```json
{
  "protocol": "sc-cron.hook.v1",
  "event": "cron.tick",
  "schedule": "sync",
  "payload": { "scope": "a", "n": [1, 2] },
  "deadline": "2026-10-25T00:00:20.000000Z"
}
```

The source defines two functions. The worker calls them. Module-level state survives later ticks of the same schedule generation.

```python
def cron_stop(hook):
    """Return {"action": "continue" | "disable" | "delete", "reason": "..."}."""

def cron_tick(hook):
    """A normal return is RunOutcome::Ok. An exception is Failed."""
```

`reason` is required for `disable` and `delete`, the same rule as the Python stop hook. A stop answer other than `continue`, `disable`, or `delete` is `RunOutcome::Failed`, and that run does not start (beads-cache stop-check rule; not sc-runtime `REQ-CRON-0005`, which is the error enum).

One `MontyRepl` (crate `monty`) per schedule name **while a feed is not running inside it**. The first event of a generation feeds `source`. Later events of that generation may call `cron_stop` and `cron_tick` in the same repl, so the hook may keep state across ticks. Names do not share a repl.

**Stop then tick.** The **worker** decides whether `cron_tick` runs after `cron_stop` returns (not the feed task alone). After `cron_stop` returns, apply any queued replace, external delete, external disable, or shutdown **before** starting `cron_tick`. A queued delete, external disable, or shutdown does **not** call `cron_tick`; drop the repl after that stop feed returns and record the stop outcome under the existing generation fence. A queued **replace** does **not** call the old generation's `cron_tick`; drop the old repl and compile the new `source` on the **new generation's first** `cron_stop` or `cron_tick`. A stop answer of `disable` or `delete` still skips `cron_tick`. Otherwise `cron_tick` runs in the same repl.

**Repl lifetime.** Do not drop, replace, or reuse a repl until the Monty feed inside it has returned. A replace that arrives while a feed is running **queues** until that feed returns; then follow the stop-then-tick rules above. After `Failed` or `TimedOut`, drop that name's repl only **after** the feed that recorded the outcome has returned; the next tick compiles again.

**Worker (Monty vs port paths).** Rust and Python keep the port's cooperative cancel, `Worker::merge` slot drop (`InFlight`'s `Drop` aborts the join), `Worker::drain` deadline abort with `TimedOut` on cancellation, `WorkerHandle::shutdown` worker abort at the handle deadline, and `Scheduler::run_due`'s parent deadline select with `DueTask` abort on drop.

For **Monty**:

- **`Worker::expire`:** no `handle.abort()` and no `TimedOut` from `JoinError::is_cancelled()`.
- **`Worker::wake`:** while the Monty feed has not returned, do not treat that run's monotonic deadline as an immediate wake; park on the task-join arm.
- **`Worker::merge`:** do not drop a slot whose in-flight run is Monty while its feed has not returned, even when `(id, generation)` no longer matches. Do not build or launch the new generation until that return; a **replace** then follows the stop-then-tick rules (new source on the new generation's first hook call only).
- **`InFlight`:** Monty `InFlight` must not be dropped (and must not run `Drop`'s abort) until the feed returns.
- **`Worker::drain`:** do not abort a Monty join or synthesize `TimedOut` from cancellation; park on that join until the feed returns, then record the feed's outcome.
- **`WorkerHandle::shutdown`:** when the handle's shutdown deadline passes, do **not** abort the worker task while a Monty feed has not returned; park until each in-flight Monty feed returns, then record that feed's outcome. Rust and Python runs may still be aborted at the handle deadline. Dropping `WorkerHandle` may signal stop; it must not abort a Monty join.
- **`Scheduler::run_due`:** do not `select` a parent deadline into `TimedOut` for Monty; `DueTask`'s `Drop` must not abort a Monty join. Wait for the Monty feed and record only the feed's outcome. Rust and Python keep `run_due`'s current select and `DueTask` abort.

Cancellation does not record `TimedOut` or `Failed`. A feed return records `TimedOut` only for a feed-time-limit error and `Failed` for any other exception. A hook that never starts records `RunOutcome::Failed` from `CronError::HookFailed` and leaves no repl. A source missing `cron_stop` or `cron_tick` returns `HookFailed` and does not start.

The schedule-table connection is not held across a feed (store work stays on the blocking pool like the port today). Many schedule names may each occupy a blocking-pool thread until their feeds hit the next interpreter checkpoint after the deadline; that stall is accepted because sc-cron does not cap row count.

### Schedule table (`REQ-CRON-0004`)

b-3 opens at `PRAGMA user_version` = 3 with `hook_kind` (`rust` | `python`) and `hook_module`. This sprint bumps to version 4 inside `crates/sc-cron` only. The consumer's other tables are untouched.

Opening a file at user_version 1, 2, or 3 runs the port's existing upgrades **without** stamping user_version 4. The version-4 step then **rebuilds** `sc_cron_schedule` for **every** file still at user_version 1, 2, or 3, including a version-3 table whose `hook_kind` column has **no** CHECK (the port's v1→v2→v3 `ALTER TABLE` path). Do not gate the rebuild on an existing `hook_kind` CHECK; SQLite cannot add that CHECK to an existing column. After rebuild, user_version is 4, `hook_source` is present, and `hook_kind` is restricted to `rust`, `python`, and `monty`.

Row shape at version 4:

- `rust`: `hook_kind='rust'`, `hook_module=''`, `hook_source=''`
- `python`: `hook_kind='python'`, `hook_module='<path>'`, `hook_source=''`
- `monty`: `hook_kind='monty'`, `hook_module=''`, `hook_source='<program text>'`

`hook_source` is capped at **`MAX_HOOK_SOURCE_BYTES`** (64 KiB, same as `MAX_PAYLOAD_BYTES` in the b-3 port's `limits.rs`), enforced in `ScheduleSpec` validation and in table CHECKs like `payload`. Fresh files created at version 4 include `hook_source` and the three-way `hook_kind` check in the initial `CREATE TABLE`.

### `monty`, and not `monty-pool`

Use the in-process `monty` crate. Do not depend on `monty-pool`.

Pydantic's split is: `monty` runs in-process when the host controls the code; `monty-pool` runs the interpreter in worker subprocesses for untrusted code, with crash isolation and a parent-side `request_timeout` that kills the worker. This target is the trusted case. A subprocess pool would spend the process spawn the target exists to avoid, and it is the untrusted path. Untrusted scripts and any hook read from a cache file stay on `HookTarget::Python`, which already has the process-group kill (beads-cache `REQ-CRON-0018`, inherited by the port).

There is no pool checkout, so a failure to lend an interpreter does not occur. A source that does not compile, or that does not define both functions, never starts: `CronError::HookFailed` (`HOOK_FAILED`) from the invoke path, and the worker records `RunOutcome::Failed`. After a feed-time-limit error the repl is dropped only after that feed returns (see repl lifetime above).

In-process Monty can abort the host on a stack overflow or an allocator abort. A Rust `Job` can abort the host the same way. That risk is accepted for this target. `monty-pool` is how untrusted code contains it, and this sprint does not take that dependency.

### Timeout

Each row already has a timeout. The CPython process-group kill does not apply.

The time remaining until the row deadline is `ResourceLimits::max_feed_duration` for that feed (`monty-types`). **`RunOutcome::TimedOut` is recorded only when the Monty feed returns a feed-time-limit error** from Monty. Do not wrap the feed in a parent timeout that records `TimedOut` while the interpreter is still running; Monty is exempt from `Worker::expire`, `Worker::drain`, merge/`InFlight` drop-abort, `WorkerHandle::shutdown` worker abort, and `Scheduler::run_due` parent timeout (see Worker above). The task joined to the feed **stays joined** until the next interpreter checkpoint after the deadline; that return is the only timeout path. Code between checkpoints may run past the wall deadline until that checkpoint; there is no second kill switch.

Any other exception raised by the running hook is `RunOutcome::Failed`. `CronError::HookFailed` is only "the hook could not be started" (feature off, compile error, missing `cron_stop` or `cron_tick`). A deadline is not `HookFailed`.

The interpreter runs on tokio's blocking pool, the same place the port puts synchronous store work.

`ResourceLimits::max_memory` stays unset (no memory-limit error from Monty for this target). A trusted Monty hook has the same memory freedom as a `Job`.

### Trusted hooks

Monty code is trusted the way a Rust `Job` is trusted: the embedding process chose the source when it built the `ScheduleSpec`. sc-cron does not load that source from a cache file, does not accept a path, and does not mount a host directory (`monty-fs` is not a dependency). beads-cache ownership and writability checks (`REQ-BDC-0723`) stay in beads-cache, on the CPython path. Monty is not a sandbox for arbitrary cache-file code.

### Dependency

Cargo feature `monty` on `sc-cron`. Optional crates.io dependencies: `monty` and `monty-types`. No `monty-pool`, `monty-fs`, or `monty-runtime`. The feature is off by default. `sc-cron` still depends on no other workspace crate (`REQ-CRON-0001`). A consumer who does not enable `monty` does not link the interpreter.

## ADR-RUN-0013 amendment this sprint applies

After `run-9ny` merges, this sprint edits `docs/architecture.md` and `docs/sc-cron/architecture.md`.

Amend point 4 so the hook target is a Rust function, a Python module, or **trusted in-process Monty source** under feature `monty` (embedding-chosen text, not a cache file; no `monty-pool`, `monty-fs`, or `monty-runtime`). **Replace** the sentence that the hook JSON carries only the schedule name and the row payload: Python and Monty both receive the **`sc-cron.hook.v1`** object with required `protocol`, `event`, `schedule`, `payload`, and `deadline`, and the object has no beads, bd-cache, or ATM field. **One repl per schedule name**; do not drop or replace a repl until the feed inside it returns; external delete/disable after an in-flight feed does not dispatch another hook on the old generation. **Worker:** Monty exempt from `expire`, `merge`/`InFlight` drop-abort, `drain` cancellation-`TimedOut`, `WorkerHandle::shutdown` worker abort, and `Scheduler::run_due` parent timeout; stop-then-tick gating before `cron_tick`. **Timeout:** `max_feed_duration` only at interpreter checkpoints; `max_memory` unset. **Host abort** from stack overflow or allocator failure is accepted for trusted Monty, same class of risk as an in-process Rust `Job`. The crate still ships no HTTP, MCP, or CLI API.

Amend point 6 so the variant set is `BadExpression`, `InvalidSpec`, `AlreadyActive`, `NotFound`, `HookFailed`, `Store`, and `Fenced`, with codes `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, and `FENCED`. That sentence replaces the point 6 list that named five variants and claimed the beads-cache stable codes.

Add a consequence: sprint `b-9` adds feature `monty` and `HookTarget::Monty`. beads-cache does not gain a Monty target. sc-cron has **no row cap, no global in-flight lock, and no two-row mutex** per the [owner ruling](decision-sync-scheduling-at-api.md#owner-ruling-no-slot-restriction-2026-10-26) (beads-cache REQ-BDC-0720 two-row ceiling is temporary bd-cache behavior only).

`REQ-CRON-0003` gains one sentence: a third target, `Monty { source }`, available when the crate is built with feature `monty`, with the entry contract and lifecycle above. `REQ-CRON-0005` points at the amended variant set.

`docs/sc-cron/architecture.md` shows the three-variant `HookTarget` enum (matching the sample in Decisions), the repl and timeout lifecycle in prose, and updates the error list to include `NotFound` and `HookFailed`.

## Deliverables

1. `REQ-CRON-0003`: `HookTarget::Monty { source }` beside `Rust` and `Python { module }`, with the `cron_stop` / `cron_tick` entry contract, stop-then-tick gating, and the JSON sample above; Monty integration in the worker (`expire`, `wake`, `merge`, `InFlight`, `drain`), `WorkerHandle::shutdown`, and `Scheduler::run_due`. No beads types.
2. `REQ-CRON-0004`: persist Monty rows with `hook_kind = monty` and capped `hook_source`; bump `SCHEMA_VERSION` to 4; open every file at user_version 1–3 through the version-4 rebuild path; do not migrate the consumer's other tables.
3. `REQ-CRON-0001`: feature `monty` linking crates.io `monty` and `monty-types` only. The default build does not link them. No new workspace crate. No dependency on another workspace crate.
4. `REQ-CRON-0003` and ADR-RUN-0013 point 4: amended point 4 text in `docs/architecture.md` (trusted Monty, repl lifecycle, checkpoint timeout, no monty-pool/fs/runtime); the new ADR consequence; the one `REQ-CRON-0003` sentence; `HookTarget` enum and Monty lifecycle prose in `docs/sc-cron/architecture.md` matching Decisions.
5. `REQ-CRON-0005` and ADR-RUN-0013 point 6: amended point 6 and the seven-variant `CronError` list in `docs/architecture.md` and `docs/sc-cron/architecture.md`, including `NotFound` / `NOT_FOUND` and `HookFailed` / `HOOK_FAILED`.
6. `REQ-RUN-0005`: edit `boundaries/sc-cron/sc-cron.toml`: append `monty` and `monty-types` to `allowed_dependencies`; do not add `monty-pool`, `monty-fs`, or `monty-runtime`; leave `forbidden_edges` and `allowed_dependents` unchanged; update the `[public]` notes so `HookTarget` includes `Monty { source }`. No new facade type. This sprint does not edit `Cargo.toml` at the workspace root. `Cargo.lock` changes only to record the optional Monty crates.
7. `NFR-RUN-0005`: tests that open a schedule file use a `tempfile::TempDir`. They do not set process environment variables.

## Acceptance criteria

Root: `boundary:sc-cron` and ADR-RUN-0013 as amended by this sprint.

- A build without `--features monty` stores a `Monty` row and, on launch, returns `HookFailed` and records `Failed`. `cargo tree -p sc-cron -e normal --prefix none` prints no line whose crate name is exactly `monty` (a separate `monty-types` line is allowed only with `--features monty`).
- Opening a `user_version` **1** file and a `user_version` **3** file both land at `PRAGMA user_version` 4 with `hook_source` and the three-way `hook_kind` check; a Python row from before the upgrade still runs; `hook_kind` outside `rust`, `python`, and `monty` fails the table CHECK; a newly created file is version 4 and a Monty row round-trips.
- `cargo test -p sc-cron --features monty` covers: `cron_stop` returning `continue` then `cron_tick` seeing state stored by an earlier tick of the same generation; a replace during an in-flight Monty feed that does not free the repl until the feed returns, then the new generation's first call using the new source; an external delete or disable during an in-flight Monty feed that, after the feed returns, drops the repl **without** another `cron_stop` or `cron_tick` on the old generation; an external delete and a `WorkerHandle::shutdown` **during `cron_stop`** that do **not** call `cron_tick`; `WorkerHandle::shutdown` whose deadline elapses during an in-flight Monty feed that does not abort the worker join and does not record `TimedOut` until the feed returns a feed-time-limit error; `Scheduler::run_due` on a Monty row that does not record `TimedOut` until that feed returns a feed-time-limit error; a `disable` stop answer that does not run `cron_tick` and drops the repl after the stop feed returns; a stop answer other than `continue`, `disable`, or `delete` recording `Failed` without starting the run; a feed that returns a feed-time-limit error recording `TimedOut` only after that feed returns (not via parent abort paths), then dropping that name's repl so the next tick compiles again; a running hook that raises recording `Failed`, then dropping that name's repl so the next tick compiles again; a source that does not compile returning `HookFailed`; a source missing `cron_stop` or `cron_tick` returning `HookFailed`.
- `cargo tree -p sc-cron --features monty -e normal --prefix none` prints `monty` and `monty-types` and prints no workspace crate, no `monty-pool`, no `monty-fs`, and no `monty-runtime`.
- The JSON object passed into `cron_stop` and `cron_tick` matches the sample in Decisions (`protocol`, `event`, `schedule`, `payload`, `deadline` required) and has no beads, bd-cache, or ATM field.
- `docs/architecture.md` point 4 names Rust, Python, and Monty under feature `monty` **and** replaces the two-field hook JSON sentence with the **`sc-cron.hook.v1`** object (required `protocol`, `event`, `schedule`, `payload`, `deadline`; no beads, bd-cache, or ATM field); point 6 lists `NotFound` (`NOT_FOUND`) and `HookFailed` (`HOOK_FAILED`) with the full seven-variant set.
- `docs/sc-cron/architecture.md` shows `HookTarget::Monty { source }`, documents the repl and checkpoint-timeout lifecycle, and lists `NotFound` and `HookFailed` on `CronError`.
- `docs/sc-cron/requirements.md` includes the third-target sentence on `REQ-CRON-0003`.

## Required validation

- `cargo test -p sc-cron` exits 0.
- `cargo test -p sc-cron --features monty` exits 0.
- `cargo build --workspace` exits 0.
- `sc-lint lint sc-boundary` exits 0 with the updated `boundaries/sc-cron` manifest.
- `cargo tree -p sc-cron -e normal --prefix none` prints no line whose crate name is exactly `monty` and no workspace crate.
- `cargo tree -p sc-cron --features monty -e normal --prefix none` prints no line `monty-pool`, no line `monty-fs`, no line `monty-runtime`, and no workspace crate.
- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' crates/sc-cron/src` exits 1.

## Non-goals

No beads-cache Monty integration. No `monty-pool`. No sandbox and no hook loaded from a cache file. No replacement of the CPython target. No HTTP, MCP, or CLI API. No global in-flight lock and no cap on schedule row count. No edit to PR #68. No edit to `b-8` or to the stores sprints. Workspace `Cargo.toml` stays as `b-3` left it. `Cargo.lock` changes only to record the optional Monty crates. beads-cache **REQ-BDC-0720** slot-cap text is out of scope here; track under `run-zif.2` in beads-cache.
