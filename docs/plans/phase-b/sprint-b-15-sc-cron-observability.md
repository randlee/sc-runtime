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
  - NFR-RUN-0008
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0004
  - ADR-RUN-0013
owned_paths:
  - crates/sc-cron/src/process.rs
  - crates/sc-cron/src/monty_compose.rs
  - crates/sc-cron/src/jitter.rs
  - crates/sc-cron/src/worker.rs
  - crates/sc-cron/src/worker/worker_runs.rs
  - crates/sc-cron/src/scheduler/run_due.rs
  - crates/sc-cron/src/sqlite.rs
  - crates/sc-cron/tests/observability.rs
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

## Relations

`must_follow` b-13 because this sprint consumes `SharedCanonicalLog` from
`RunCoord` and the fixed scheduler injection contract. Merge b-13 into this
branch before every development or fix round. It is `parallel_safe` with b-14
and b-16 because their runtime/template paths do not intersect this crate.

## Event contract

Every migrated call site uses the exact row below. `—` means
`diagnostic: None`; no implementation derives a code by parsing error or
outcome prose.

| Target | Action | Level | Outcome | `Diagnostic.code` | Required fields / cardinality |
|---|---|---|---|---|---|
| `cron.jitter` | `source.unavailable` | warning | `failed` | `JITTER_SOURCE_UNAVAILABLE` | error; once per failed jitter calculation |
| `cron.store` | `row.corrupt` | warning | `failed` | `CORRUPT_SCHEDULE_ROW` | row id when known, error; once per skipped row id |
| `cron.worker` | `target.resolve_failed` | warning | `failed` | `TARGET_RESOLVE_FAILED` | schedule, target, error; once per failed resolution |
| `cron.worker` | `schedule.reload_failed` | error | `failed` | `SCHEDULE_RELOAD_FAILED` | error; once per failed reload attempt |
| `cron.worker` | `schedule.cancelled` | info | `stopped` | — | schedule, reason; once per cancelled slot |
| `cron.worker` | `run.timed_out` | warning | `timeout` | `RUN_TIMEOUT` | schedule, deadline remaining `0`; once per expired run |
| `cron.worker` | `run.skipped_in_flight` | info | — | — | schedule, reason; once per skipped tick |
| `cron.worker` | `run.missed_skipped` | info | — | — | schedule; once per skipped missed-tick decision |
| `cron.worker` | `run.missed_replayed` | info | — | — | schedule; once per replayed missed-tick decision |
| `cron.worker` | `run.launched` | info | — | — | schedule, run when allocated, deadline remaining; once per launch |
| `cron.worker` | `run.finished` | outcome table below | outcome table below | outcome table below | schedule, run when known, exact `RunOutcome` detail; once per recorded run |
| `cron.worker` | `run.record_failed` | error | `failed` | `RUN_RECORD_FAILED` | schedule, run when known, error; once per failed record write |
| `cron.worker` | `schedule.next_failed` | error | `failed` | `SCHEDULE_NEXT_FAILED` | schedule, error; once per failed next-run write |
| `cron.worker` | `schedule.stopped` | info | `stopped` | — | schedule, reason; once per successful stop write |
| `cron.worker` | `schedule.stop_failed` | error | `failed` | `SCHEDULE_STOP_FAILED` | schedule, error; once per failed stop write |
| `cron.worker` | `schedule.disabled` | info | `stopped` | — | schedule, reason; once per successful disable write |
| `cron.worker` | `schedule.disable_failed` | error | `failed` | `SCHEDULE_DISABLE_FAILED` | schedule, error; once per failed disable write |
| `cron.worker` | `run.fenced` | warning | `fenced` | `SCHEDULE_FENCED` | schedule, operation; once per refused generation-guarded write |
| `cron.scheduler` | `due.launched` | info | — | — | schedule, run when allocated, deadline remaining; once per launch |
| `cron.scheduler` | `due.finished` | outcome table below | outcome table below | outcome table below | schedule, run, exact `RunOutcome` detail; once per recorded due run |
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
| `cron.process` | `child.kill_failed` | debug | `failed` | `CHILD_KILL_FAILED` | run and pid when known, error; once per failed kill attempt |
| `cron.process` | `child.reap_timeout` | warning | `timeout` | `CHILD_REAP_TIMEOUT` | run and pid when known, deadline remaining `0`; once per exhausted reap bound |
| `cron.monty.compose` | `pipe.drain_timeout` | warning | `timeout` | `COMPOSE_PIPE_DRAIN_TIMEOUT` | run and pid when known, deadline remaining `0`; once per exhausted drain bound |
| `cron.monty.compose` | `child.kill_timeout` | warning | `timeout` | `COMPOSE_CHILD_KILL_TIMEOUT` | run and pid when known, deadline remaining `0`; once per exhausted kill bound |

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

The interface-owned `cron.scheduler/logger.attached` event is introduced by
b-13 in `Scheduler::with_logger`; this sprint consumes that contract and owns
the remaining event sites. Timestamps and service names use the types
contract. Every field and envelope location follows b-13's canonical mapping:
stable codes exist only at `diagnostic.code`, run IDs use top-level
`correlation_id`, and typed elapsed/deadline/attempt/pid values use the fixed
field keys. Every failed emission is ignored after the attempt.

## Required work and deliverables

Every numbered item must land production-ready for `sc-cron` boundary
closure. Every owned trace site and each stable persisted failure shape must
be migrated and behavior-tested; partial taxonomy coverage cannot close it.

1. REQ-CRON-0001 / REQ-CRON-0003: replace production trace calls in the owned files with canonical events;
   do not change `RunOutcome`, retry, stop, lease, deadline, or kill logic.
2. REQ-CRON-0003 / NFR-CRON-0001: preserve PR #84's stable Monty codes (`DUE_JOIN_FAILED`,
   `FEED_JOIN_FAILED`) and compose deadline/process-group implementation.
3. REQ-CRON-0005 / NFR-RUN-0008: replace persisted `"resolve: ..."` and
   job-panic prose with exact `CODE: detail; recovery` shapes:
   `TARGET_RESOLVE_FAILED: {detail}; fix hook target configuration and retry`
   and `JOB_JOIN_FAILED: {detail}; fix the job panic and retry`.
4. NFR-RUN-0005 / NFR-RUN-0009: test native and Monty success/failure paths with a recording emitter and a
   rejecting emitter; assert event order only where the implementation
   guarantees it.
5. NFR-CRON-0001: remove direct tracing calls from owned source. The shared manifest remains
   b-13-owned and dependency cleanup is not part of this sprint.

## Acceptance criteria

1. `boundary:BOUNDARY-sc-cron`: native and Monty runs emit one terminal event
   using the complete `RunOutcome` table; arbitrary consumer strings are
   copied, never parsed, and every internal resolver/join failure also emits
   its exact fixed origin-code row.
2. ADR-RUN-0013: stop and retry decisions are unchanged when emission
   rejects every event.
3. `boundary:BOUNDARY-sc-cron`: tests coordinate with condition/deadline helpers and add
   no fixed-delay sleep.
4. `boundary:BOUNDARY-sc-cron`: identifiers correlate scheduler, worker, and process/feed
   records without bodies, hook source, or secret values.
5. The previously closed allowlist, host deadline, process-group kill, and
   render-only host behavior are verified, not reimplemented.
6. `boundary:BOUNDARY-sc-cron`: table-driven tests trigger every row above
   and assert exact target, action, level, outcome, `Diagnostic.code`,
   required fields, terminal `RunOutcome` mapping, and stated
   cardinality; the complete owned-file grep below returns no tracing site.

## Required validation

```sh
cargo test -p sc-cron --test observability
cargo test -p sc-cron
cargo +1.96.1 test -p sc-cron --features monty
cargo clippy -p sc-cron --all-targets --all-features -- -D warnings
cargo check --workspace
! rg -n 'tracing::|trace!|debug!|info!|warn!|error!' crates/sc-cron/src/process.rs crates/sc-cron/src/monty_compose.rs crates/sc-cron/src/jitter.rs crates/sc-cron/src/worker.rs crates/sc-cron/src/worker/worker_runs.rs crates/sc-cron/src/scheduler/run_due.rs crates/sc-cron/src/sqlite.rs
! rg -n 'tokio::time::sleep|sleep\(' crates/sc-cron/tests/monty_hook.rs
sc-lint lint sc-boundary --root . --config sc-lint.toml
```

## This sprint does not close

- New Monty host functions, compose lifecycle redesign, schedule-row limits,
  template wiring, or a repository-wide diagnostics dependency cleanup.
