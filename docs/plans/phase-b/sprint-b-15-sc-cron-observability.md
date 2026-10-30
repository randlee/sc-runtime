---
id: b-15
phase: b
status: planned
closure_type: boundary
target_boundary: sc-cron
track: observability
wave: 8
branch: sprint/b-15-sc-cron-observability
worktree: sprint/b-15-sc-cron-observability
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0005
  - NFR-CRON-0001
  - NFR-RUN-0005
  - NFR-RUN-0002
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0004
  - ADR-RUN-0007
  - ADR-RUN-0011
  - ADR-RUN-0013
owned_paths:
  - crates/sc-cron/src/worker.rs
  - crates/sc-cron/src/worker/worker_runs.rs
  - crates/sc-cron/src/scheduler/run_due.rs
  - crates/sc-cron/tests/observability.rs
  - crates/sc-cron/tests/target_resolver.rs
  - crates/sc-cron/tests/worker.rs
must_follow: [b-13]
parallel_safe: [b-14, b-16]
recommended_agent: solar
recommended_model: sol
---

# Sprint b-15 — sc-cron canonical scheduler events

## Goal

Implement the `sc-cron` half of the fixed injected handle without changing
scheduling, lease, jitter, process, Monty, or persistence outcomes. Normalize
the two residual persisted free-text failures at the sites already owned here.
Preserve sc-cron as an optional general-purpose SQLite cron library for
application-defined Rust, Python, and optional Monty jobs. sc-runtime stays
independent of sc-cron; no default cron feature is enabled. Application-owned
host registration and the sc_compose_run/atm_run adapters are b-16's work.
This event migration adds no MVP script signing, separate approval mechanism,
or consumer option enabling either.

## Relations

`must_follow` b-13 because this sprint consumes `SharedCanonicalLog` from
`RunCoord` and the fixed scheduler injection contract. Merge b-13 into this
branch before every development or fix round. It is `parallel_safe` with b-14
and b-16, with one explicit shared-file exception: `worker.rs`,
`worker/worker_runs.rs`, and `scheduler/run_due.rs` under
`crates/sc-cron/src/` have a serial sole-writer window. b-15 owns those files
through a compilable development push; b-16 then merges that push forward
and owns only the mechanical registry transport changes to the Worker
field/constructor, `spawn_feed`/`FeedLaunch`, and due-run feed parameters.
The same `RunCoord` logger Arc and event behavior are preserved. All other
wave-8 work remains parallel. The handoff waits for the development push,
not QA, a PR merge, or completion of the whole sprint.

b-13's authorized development facade permits implementing these library event
sites before upstream publication. Development applications may inject the
real registry `1.5.0` logger through the unpublished `0.0.0` local
`LoggerAdapter`, which forwards actual `service_name`/`try_log`; its tests
exercise the real queue and sinks rather than Noop behavior. The application
owns logger construction, flush, and shutdown. Explicit workspace path
overrides select the development packages, with no observability registry
patch. The real matching `1.5.1` pair, direct upstream Arc coercion, and
development-package removal remain b-18's release gate.

## Event contract

Every migrated call site uses the exact row below. `—` means
`diagnostic: None`; no implementation derives a code by parsing error or
outcome prose.

| Target | Action | Level | Outcome | `Diagnostic.code` | Required fields / cardinality |
|---|---|---|---|---|---|
| `cron.worker` | `target.resolve_failed` | warning | `failed` | `TARGET_RESOLVE_FAILED` | schedule, target, error; once per failed resolution |
| `cron.worker` | `schedule.reload_failed` | error | `failed` | `SCHEDULE_RELOAD_FAILED` | error; once per failed reload attempt |
| `cron.worker` | `schedule.cancelled` | info | `stopped` | — | schedule, reason; once per cancelled slot |
| `cron.worker` | `run.timed_out` | warning | `timeout` | `RUN_TIMEOUT` | schedule, deadline remaining `0`; once per expired run |
| `cron.worker` | `run.skipped_in_flight` | info | — | — | schedule, reason; once per skipped tick |
| `cron.worker` | `run.missed_skipped` | info | — | — | schedule; once per skipped missed-tick decision |
| `cron.worker` | `run.missed_replayed` | info | — | — | schedule; once per replayed missed-tick decision |
| `cron.worker` | `run.launched` | info | — | — | schedule, run when allocated, deadline remaining; once per launch |
| `cron.worker` | `run.finished` | outcome table below | outcome table below | outcome table below | schedule, run when known, the normalized detail written to `RunOutcome`; once per recorded run |
| `cron.worker` | `run.record_failed` | error | `failed` | `RUN_RECORD_FAILED` | schedule, run when known, error; once per failed record write |
| `cron.worker` | `schedule.next_failed` | error | `failed` | `SCHEDULE_NEXT_FAILED` | schedule, error; once per failed next-run write |
| `cron.worker` | `schedule.stopped` | info | `stopped` | — | schedule, reason; once per successful stop write |
| `cron.worker` | `schedule.stop_failed` | error | `failed` | `SCHEDULE_STOP_FAILED` | schedule, error; once per failed stop write |
| `cron.worker` | `schedule.disabled` | info | `stopped` | — | schedule, reason; once per successful disable write |
| `cron.worker` | `schedule.disable_failed` | error | `failed` | `SCHEDULE_DISABLE_FAILED` | schedule, error; once per failed disable write |
| `cron.worker` | `run.fenced` | warning | `fenced` | `SCHEDULE_FENCED` | schedule, operation; once per refused generation-guarded write |
| `cron.scheduler` | `due.launched` | info | — | — | schedule, run when allocated, deadline remaining; once per launch |
| `cron.scheduler` | `due.finished` | outcome table below | outcome table below | outcome table below | schedule, run, the normalized detail written to `RunOutcome`; once per recorded due run |
| `cron.scheduler` | `due.fenced` | warning | `fenced` | `SCHEDULE_FENCED` | schedule, operation; once per refused write |
| `cron.scheduler` | `due.partial_write` | warning | `partial` | `FOLLOW_UP_FENCED` | schedule, run, outcome; once when outcome committed but follow-up was refused |
| `cron.scheduler` | `due.follow_up_failed` | error | `partial` | `FOLLOW_UP_WRITE_FAILED` | schedule, run, outcome, error; once when outcome committed but follow-up failed |
| `cron.scheduler` | `due.record_failed` | error | `failed` | `RUN_RECORD_FAILED` | schedule, run, error; once per failed outcome write |
| `cron.scheduler` | `due.stopped` | info | `stopped` | — | schedule; once per successful final stop |
| `cron.scheduler` | `due.stop_failed` | error | `failed` | `SCHEDULE_STOP_FAILED` | schedule, error; once per failed final stop |
| `cron.scheduler` | `due.next_failed` | error | `failed` | `SCHEDULE_NEXT_FAILED` | schedule, error; once per failed pre-launch next write |
| `cron.worker` | `job.join_failed` | error | `failed` | `JOB_JOIN_FAILED` | schedule, run when known, error; once at the worker join-failure origin |
| `cron.scheduler` | `due.join_failed` | error | `failed` | `DUE_JOIN_FAILED` | schedule, run when known, error; once at the due-task join-failure origin |
| `cron.scheduler` | `due.task_failed` | error | `failed` | `RUN_TASK_FAILED` | schedule, run when known, error; once at the native due-task failure origin |
| `cron.scheduler` | `monty.resolve_failed` | error | `failed` | `MONTY_RESOLVE` | schedule, run when known, error; once at the Monty resolve origin |
| `cron.scheduler` | `monty.tick_failed` | error | `failed` | `MONTY_TICK` | schedule, run when known, error; once at the Monty tick origin |
| `cron.monty` | `feed.join_failed` | error | `failed` | `FEED_JOIN_FAILED` | schedule, run when known, error; once at the feed join-failure origin |

The two terminal rows use this complete `RunOutcome` mapping:

| `RunOutcome` | Level | `OutcomeLabel` | `Diagnostic.code` | Detail rule |
|---|---|---|---|---|
| `Ok` | info | `success` | — | no error/detail field |
| `Refused(reason)` | info | `refused` | — | copy the consumer reason to `fields.reason`; never parse it |
| `Failed(detail)` | error | `failed` | `JOB_FAILED` | copy the consumer detail to `fields.error`; never parse it for a token |
| `TimedOut` | warning | `timeout` | `RUN_TIMEOUT` | deadline remaining is integer `0` |

Resolver and join sites emit their fixed origin rows before returning their
persisted `RunOutcome::Failed`; the later terminal row remains `JOB_FAILED`.
Thus arbitrary consumer `Failed`/`Refused` strings cannot forge a diagnostic
code, while `TARGET_RESOLVE_FAILED`, `JOB_JOIN_FAILED`, `DUE_JOIN_FAILED`,
`RUN_TASK_FAILED`, `MONTY_RESOLVE`, `MONTY_TICK`, and `FEED_JOIN_FAILED`
remain explicit origin diagnostics without parsing stored prose.
Consumer `Failed`/`Refused` prose is trusted application text, not a general
secret-classification boundary. b-17 later guarantees that configured hook
environment values are redacted before this normalized detail reaches either
the persisted outcome or these event fields; this sprint must not preserve a
second unredacted copy that would bypass that boundary.

The interface-owned `cron.scheduler/logger.attached` event is introduced by
b-13 in `Scheduler::new_with_logger`; b-13 also owns the jitter/store/process/
compose propagation and component events. This sprint consumes those seams
and owns only worker and due-run event sites. Timestamps and service names use the types
contract. Every field and envelope location follows b-13's canonical mapping:
stable codes exist only at `diagnostic.code`, run IDs use top-level
`correlation_id`, and typed elapsed/deadline/attempt/pid values use the fixed
field keys. Every failed emission is ignored after the attempt.

## Required work and deliverables

1. REQ-CRON-0001 / REQ-CRON-0003: replace production trace calls in the owned files with canonical events;
   do not change `RunOutcome`, retry, stop, lease, deadline, or kill logic.
   At the worker-owned call site, pass `RunCoord`'s `SharedCanonicalLog`
   through `run_target` into b-13's logger-aware `PythonHook` API. `hook.rs`
   remains wholly b-13-owned, so the sprints never overlap an exact path.
2. REQ-CRON-0003 / NFR-CRON-0001: preserve the baseline's stable Monty codes (`DUE_JOIN_FAILED`,
   `FEED_JOIN_FAILED`) and existing compose behavior. b-16 owns the
   already-planned deadline/process-group driver and its
   `COMPOSE_CHILD_KILL_TIMEOUT` event;
   this event migration adds no kill policy.
3. REQ-CRON-0005: replace persisted `"resolve: ..."` and
   job-panic prose with exact `CODE: detail; recovery` shapes:
   `TARGET_RESOLVE_FAILED: {detail}; fix hook target configuration and retry`
   and `JOB_JOIN_FAILED: {detail}; fix the job panic and retry`. One typed
   internal formatter owns these display strings. Structured `LogEvent` is
   the event contract; the supported `RunOutcome::failure_parts` /
   `RunFailureParts` parser retains the public `CODE: detail; recovery` format.
   Choose event diagnostic codes from typed origins, never arbitrary prose.
   Update the existing `target_resolver` and `worker` regression expectations
   for these two fixed tokens in the owned legacy tests.
4. ADR-RUN-0007 / NFR-RUN-0002 / NFR-RUN-0005 / NFR-RUN-0009: test native
   and Monty success/failure paths with a recording emitter and a rejecting
   emitter. Queue-full and disconnected rejection returns immediately and
   cannot change a deadline or scheduler result; assert event order only
   where the implementation guarantees it.
5. ADR-RUN-0004 / NFR-CRON-0001: remove direct tracing calls from owned
   source and retain only the types-level observability edge. The shared manifest remains
   b-13-owned and dependency cleanup is not part of this sprint.
6. ADR-RUN-0011: production changes add no sleep or `block_on` call and hold
   no `std::sync::Mutex` guard across `.await`; deadline-based test polling is
   exempt.

## Acceptance criteria

1. `boundary:BOUNDARY-sc-cron`: `observability` passes only when native and
   Monty runs emit one terminal event from the complete `RunOutcome` table,
   arbitrary strings are copied rather than parsed, and each resolver/join
   origin emits its fixed code.
2. ADR-RUN-0013: `observability` passes only when rejecting every event leaves
   stop and retry decisions unchanged.
3. `boundary:BOUNDARY-sc-cron`: `observability` and `monty_hook` pass with
   condition/deadline coordination, and the required sleep grep finds no
   fixed-delay sleep in `monty_hook`.
4. `boundary:BOUNDARY-sc-cron`: `observability` passes only when identifiers
   correlate scheduler, worker, and process/feed records without bodies or
   hook source. b-17's follow-on canary cases prove configured environment
   values are absent; arbitrary consumer prose remains trusted input rather
   than an unimplementable claim that all possible secrets are detected.
5. `cargo +1.96.1 test -p sc-cron --features monty` passes without this sprint
   changing host deadline or process-group behavior. b-16 owns that driver
   implementation and the kill-timeout event acceptance. The pre-release
   render-only host list is b-16-owned successor work; this sprint does not
   freeze it as a product rule.
6. `boundary:BOUNDARY-sc-cron`: the table-driven `observability` test passes
   every exact event row and cardinality, and the owned-file tracing grep
   returns no match.
7. `boundary:BOUNDARY-sc-cron`: worker cases in `observability` pass only when
   the identical Arc reaches b-13's logger-aware `PythonHook`; the required
   `run_target|PythonHook` search identifies those owned call sites and the
   typed formatter remains the only producer of the two display strings.

## Required validation

```sh
cargo test -p sc-cron --test observability
cargo +1.96.1 test -p sc-cron --features monty --test observability
cargo test -p sc-cron
cargo +1.96.1 test -p sc-cron --features monty
cargo clippy -p sc-cron --all-targets --all-features -- -D warnings
cargo check --workspace
! rg -n '#\[(tracing::)?instrument|tracing::|(trace|debug|info|warn|error|event|span|trace_span|debug_span|info_span|warn_span|error_span)!' crates/sc-cron/src/worker.rs crates/sc-cron/src/worker/worker_runs.rs crates/sc-cron/src/scheduler/run_due.rs
rg -n 'run_target|PythonHook' crates/sc-cron/src/worker.rs crates/sc-cron/src/worker/worker_runs.rs crates/sc-cron/src/scheduler/run_due.rs
! rg -n 'tokio::time::sleep|sleep\(' crates/sc-cron/tests/monty_hook.rs
just adr-run-0011
sc-lint lint sc-boundary --root . --config sc-lint.toml
```

## This sprint does not close

- New Monty host functions, compose lifecycle redesign, schedule-row limits,
  template wiring, or a repository-wide diagnostics dependency cleanup.
