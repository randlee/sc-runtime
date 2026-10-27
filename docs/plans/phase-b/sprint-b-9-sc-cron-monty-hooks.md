---
id: b-9
phase: b
status: draft
closure_type: boundary
target_boundary: sc-cron
track: cron
wave: 4
branch: sprint/b-9-sc-cron-monty-hooks
worktree: sprint/b-9-sc-cron-monty-hooks
requirements:
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0005
  - REQ-RUN-0005
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0013
  - ADR-RUN-0012
  - ADR-RUN-0002
owned_paths:
  - crates/sc-cron/
  - docs/sc-cron/
  - docs/architecture.md
  - Cargo.lock
must_follow: [b-3]
blocked_by: run-9ny
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One crate. The third hook target, its session, and its deadline live in the scheduler the b-3 port already owns. A feature flag keeps Monty off the default build. No second crate.
---

# Sprint b-9 — Monty hook target for sc-cron

Draft for bead `run-zif.1` (parent `run-zif`). This document is the plan. It contains no crate change. Implementation starts only after this draft is accepted and after `run-9ny` (sprint b-3, PR #68) has merged.

## Slot

`b-9` is the next free sprint id. [phase-b-plan.md](phase-b-plan.md) lists `b-1` through `b-8` and stops at wave 3. `.sprints/b/structure.ttl` does the same, and `b-8` follows `b-7`. This draft does not edit either file, so the stores critical path stays `b-4` → `b-5` → `b-8` and `b-8` does not wait on `b-9`.

Wave 4 above is the proposed slot on the cron track after `b-3`. `must_follow` is `b-3` because both sprints own `crates/sc-cron/`. No sprint in the current index waits on this one. When the draft is accepted, the phase index and `.sprints/b/structure.ttl` gain a `b-9` row whose only `follows` edge is `b-3`. That edit is not part of this planning commit.

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

`source` is the hook program text. It is not a filesystem path. The JSON object carries the schedule name, the row payload, and the event (`cron.tick` or `cron.stop`). It carries a deadline field when the ported Python object already has one, so the two targets match. It carries no beads, bd-cache, or ATM field.

The source defines two functions. The worker calls them. Module-level state survives later ticks of the same schedule generation.

```python
def cron_stop(hook):
    """Return {"action": "continue" | "disable" | "delete", "reason": "..."}."""

def cron_tick(hook):
    """A normal return is RunOutcome::Ok. An exception is Failed."""
```

`reason` is required for `disable` and `delete`, the same rule as the Python stop hook. A stop answer other than those three actions is `RunOutcome::Failed`, and that run does not start, matching beads-cache `REQ-CRON-0005`.

One `MontyRepl` (crate `monty`) per schedule name. The first event feeds `source`. Later events of that generation call `cron_stop` and `cron_tick` in the same repl, so the hook may keep state across ticks. A replace, a generation change, or a delete drops that repl. Names do not share a repl.

### `monty`, and not `monty-pool`

Use the in-process `monty` crate. Do not depend on `monty-pool`.

Pydantic's split is: `monty` runs in-process when the host controls the code; `monty-pool` runs the interpreter in worker subprocesses for untrusted code, with crash isolation and a parent-side `request_timeout` that kills the worker. This target is the trusted case. A subprocess pool would spend the process spawn the target exists to avoid, and it is the untrusted path. Untrusted scripts and any hook read from a cache file stay on `HookTarget::Python`, which already has the process-group kill (beads-cache `REQ-CRON-0018`, inherited by the port).

There is no pool checkout, so a failure to lend an interpreter does not occur. A source that does not compile, or that does not define both functions, never starts: `CronError::HookFailed` (`HOOK_FAILED`) from the invoke path, and the worker records `RunOutcome::Failed`. After `Failed` or `TimedOut`, drop that name's repl. The next tick compiles again. A time-limit or memory-limit failure leaves a heap the next feed must not trust.

In-process Monty can abort the host on a stack overflow or an allocator abort. A Rust `Job` can abort the host the same way. That risk is accepted for this target. `monty-pool` is how untrusted code contains it, and this sprint does not take that dependency.

### Timeout

Each row already has a timeout. The CPython process-group kill does not apply.

The time remaining until the row deadline is `ResourceLimits::max_feed_duration` for that feed (`monty-types`). A feed-time-limit error is `RunOutcome::TimedOut`. Any other exception raised by the running hook is `RunOutcome::Failed`. `CronError::HookFailed` is only "the hook could not be started" (feature off, compile error, missing `cron_stop` or `cron_tick`). A deadline is not `HookFailed`.

The interpreter runs on tokio's blocking pool, the same place the port puts synchronous store work. The async worker is not blocked past the deadline. Monty enforces `max_feed_duration` at interpreter checkpoints. Code between checkpoints can run past the deadline until the next checkpoint. That gap is the bound. There is no second kill switch.

`ResourceLimits::max_memory` stays unset. A trusted Monty hook has the same memory freedom as a `Job`.

### Trusted hooks

Monty code is trusted the way a Rust `Job` is trusted: the embedding process chose the source when it built the `ScheduleSpec`. sc-cron does not load that source from a cache file, does not accept a path, and does not mount a host directory (`monty-fs` is not a dependency). beads-cache ownership and writability checks (`REQ-BDC-0723`) stay in beads-cache, on the CPython path. Monty is not a sandbox for arbitrary cache-file code.

### Dependency

Cargo feature `monty` on `sc-cron`. Optional crates.io dependencies: `monty` and `monty-types`. No `monty-pool`, `monty-fs`, or `monty-runtime`. The feature is off by default. `sc-cron` still depends on no other workspace crate (`REQ-CRON-0001`). A consumer who does not enable `monty` does not link the interpreter.

## ADR-RUN-0013 amendment this sprint applies

This planning commit does not edit `docs/architecture.md`. After `run-9ny` merges, this sprint applies the following to ADR-RUN-0013 and to `docs/sc-cron/architecture.md`.

Amend point 4 so the hook target is a Rust function, a Python module, or trusted in-process Monty source under feature `monty`. The Monty events and the JSON object are the ones in the decisions above. The crate still ships no HTTP, MCP, or CLI API.

Amend point 6 if it still omits the two codes recorded in [sc-cron-req-crosswalk.md](sc-cron-req-crosswalk.md). The variant set is `BadExpression`, `InvalidSpec`, `AlreadyActive`, `NotFound`, `HookFailed`, `Store`, and `Fenced`, with codes `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, and `FENCED`. That sentence replaces the point 6 list that named five variants and claimed the beads-cache stable codes. `NotFound(ScheduleName)` is the missing-active-name error the port already returns. `HookFailed(String)` is a hook that could not be started.

Add a consequence: sprint `b-9` adds feature `monty` and `HookTarget::Monty`. beads-cache does not gain a Monty target. The two-slot sync mutex stays bd-cache API policy ([decision-sync-scheduling-at-api.md](decision-sync-scheduling-at-api.md)).

`REQ-CRON-0003` gains one sentence: a third target, `Monty { source }`, available when the crate is built with feature `monty`, with the entry contract above. `REQ-CRON-0005` points at that amended variant set.

## Deliverables

1. `REQ-CRON-0003`: `HookTarget::Monty { source }` beside `Rust` and `Python { module }`, with the `cron_stop` / `cron_tick` entry contract and the JSON object above. No beads types.
2. `REQ-CRON-0001`: feature `monty` linking crates.io `monty` and `monty-types` only. The default build does not link them. No new workspace crate. No dependency on another workspace crate.
3. `REQ-CRON-0005`: a feed that hits `max_feed_duration` records `RunOutcome::TimedOut`. A hook that cannot start returns `CronError::HookFailed`. ADR-RUN-0013 point 6 and `docs/sc-cron/architecture.md` list `NotFound` and `HookFailed` with the codes above.
4. `REQ-RUN-0005`: `boundaries/sc-cron` still forbids every other workspace crate. The new dependencies are crates.io crates, so the manifest's forbidden edges do not gain a workspace name. This sprint does not edit `Cargo.toml` at the workspace root. `Cargo.lock` changes only to record the optional Monty crates.
5. `NFR-RUN-0005`: tests that open a schedule file use a `tempfile::TempDir`. They do not set process environment variables.

## Acceptance criteria

Root: `boundary:sc-cron` and ADR-RUN-0013 as amended by this sprint.

- A build without `--features monty` stores a `Monty` row and, on launch, returns `HookFailed` and records `Failed`. `cargo tree -p sc-cron -e normal --prefix none` prints no line `monty`.
- `cargo test -p sc-cron --features monty` covers: `cron_stop` returning `continue` then `cron_tick` seeing state stored by an earlier tick of the same generation; a replaced row running with a fresh repl; a `disable` stop answer that does not run `cron_tick`; a feed that exceeds `max_feed_duration` recording `TimedOut`; a source that does not compile returning `HookFailed`.
- `cargo tree -p sc-cron --features monty -e normal --prefix none` prints `monty` and `monty-types` and prints no workspace crate and no `monty-pool`.
- The JSON object passed into `cron_stop` and `cron_tick` has the schedule name and the row payload and has no beads, bd-cache, or ATM field.
- `docs/architecture.md` point 6 lists `NotFound` (`NOT_FOUND`) and `HookFailed` (`HOOK_FAILED`).

## Required validation

- `cargo test -p sc-cron` exits 0.
- `cargo test -p sc-cron --features monty` exits 0.
- `cargo build --workspace` exits 0.
- `cargo tree -p sc-cron -e normal --prefix none` prints no line `monty` and no workspace crate.
- `cargo tree -p sc-cron --features monty -e normal --prefix none` prints no line `monty-pool` and no workspace crate.
- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' crates/sc-cron/src` exits 1.

## Non-goals

No beads-cache Monty integration and no edit to `REQ-BDC-0720`. No `monty-pool`. No sandbox and no hook loaded from a cache file. No replacement of the CPython target. No HTTP, MCP, or CLI API. No two-slot mutex and no global in-flight lock. No edit to PR #68. No edit to `b-8` or to the stores sprints. Workspace `Cargo.toml` stays as `b-3` left it. `Cargo.lock` changes only to record the optional Monty crates.
