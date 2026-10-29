---
id: b-13
phase: b
status: planned
closure_type: contract
target_boundary: observability-injection
track: observability
wave: 7
branch: sprint/b-13-observability-contract
worktree: sprint/b-13-observability-contract
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0003
  - REQ-RUN-0005
  - REQ-RUN-0309
  - REQ-RUN-0702
  - REQ-RT-0001
  - REQ-RT-0004
  - NFR-RT-0001
  - NFR-RT-0002
  - NFR-RT-0003
  - NFR-RT-0004
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0005
  - NFR-CRON-0001
  - NFR-RUN-0002
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0004
  - ADR-RUN-0007
  - ADR-RUN-0013
  - ADR-RT-0001
owned_paths:
  - Cargo.toml
  - Cargo.lock
  - docs/requirements.md
  - docs/architecture.md
  - docs/sc-runtime/requirements.md
  - docs/sc-runtime/architecture.md
  - docs/sc-runtime/observability-injection.md
  - docs/sc-cron/requirements.md
  - docs/sc-cron/architecture.md
  - boundaries/sc-runtime/sc-runtime.toml
  - boundaries/sc-cron/sc-cron.toml
  - crates/sc-runtime/Cargo.toml
  - crates/sc-runtime/src/lib.rs
  - crates/sc-runtime/src/builder.rs
  - crates/sc-runtime/tests/observability_contract.rs
  - crates/sc-cron/Cargo.toml
  - crates/sc-cron/src/lib.rs
  - crates/sc-cron/src/coord.rs
  - crates/sc-cron/src/scheduler.rs
  - crates/sc-cron/src/jitter.rs
  - crates/sc-cron/src/sqlite.rs
  - crates/sc-cron/src/process.rs
  - crates/sc-cron/src/monty_compose.rs
  - crates/sc-cron/src/monty_host.rs
  - crates/sc-cron/src/monty_run.rs
  - crates/sc-cron/tests/observability_contract.rs
  - crates/sc-cron/tests/observability_components.rs
must_follow: []
parallel_safe: [b-16]
recommended_agent: solar
recommended_model: sol
---

# Sprint b-13 — observability injection contract and adoption

## Goal

Consume published `sc-observability-types` **1.5.1** and
`sc-observability` **1.5.1** and freeze the injection contract before either library
migrates events. This repository does not own those upstream source trees or
their publication. If matching releases are unavailable, this sprint is
blocked at entry; it does not vendor a replacement.

PR #84 (`integrate/phase-b-monty`) must first be merged. Acceptance reads its
final hosted `mergeCommit.oid` and verifies that commit against fetched
`origin/develop`; no moving head snapshot is treated as the merge proof.

## Relations

PR #84 being merged is a phase entry prerequisite, not an edge in the
continuation DAG. b-13 is `parallel_safe` with b-16: the former owns library
contracts/manifests while the latter owns only generated-template Cargo and
Monty product files.
That prerequisite closes b-10 through b-12 and prevents this amendment from
replanning their work.

## Upstream artifact consumed

Published `sc-observability-types` **1.5.1** and `sc-observability` **1.5.1**
are exact external entry prerequisites. This repository consumes them and
does not own or authorize their publication. The types patch fixes these
semantics:

```rust
pub trait CanonicalLog: Send + Sync {
    fn service_name(&self) -> &ServiceName;
    fn emit(&self, event: LogEvent) -> Result<(), v2::EventError>;
}

pub struct NoopCanonicalLog { /* service name only */ }
impl NoopCanonicalLog {
    pub fn new(service_name: ServiceName) -> Self;
    pub fn named_or_default(service: &'static str) -> Self;
}
pub type SharedCanonicalLog = std::sync::Arc<dyn CanonicalLog>;
```

The matching runtime patch implements `CanonicalLog` for
`sc_observability::v2::Logger` by calling `Logger::try_log` (or an equivalent
API contractually guaranteed never to wait for queue capacity). It must never
call `Logger::log`, `enqueue_blocking`, or any blocking/backpressured path.
Queue-full and disconnected-sink rejection returns `EventError` immediately
and remains best effort at every library call site. It does not require
`Logger: Clone`.
Applications retain `Arc<Logger>` and coerce an Arc clone to
`SharedCanonicalLog`. Noop and the upstream recording/test-double contract are
fixed here so b-14 and b-15 do not renegotiate them.

`named_or_default` is infallible: it uses the requested valid static name and
falls back to the upstream Noop identity if validation ever rejects it. The
runtime default is always `sc-runtime`; the cron default is always `sc-cron`.
Neither default parses an application name or calls `unwrap`/`expect` on
consumer input. The generated daemon supplies the real application identity
only by injecting its production logger.

The types patch also exposes deterministic doubles behind its `test-util`
feature:

```rust
#[cfg(feature = "test-util")]
pub struct RecordingCanonicalLog { /* Arc-safe ordered LogEvent buffer */ }
#[cfg(feature = "test-util")]
impl RecordingCanonicalLog {
    pub fn new(service: ServiceName) -> Self;
    pub fn events(&self) -> Vec<LogEvent>;
    pub fn clear(&self);
}

#[cfg(feature = "test-util")]
pub struct RejectingCanonicalLog { /* fixed EventError per emit */ }
#[cfg(feature = "test-util")]
impl RejectingCanonicalLog {
    pub fn new(service: ServiceName) -> Self;
    pub fn attempts(&self) -> u64;
}
```

Both implement `CanonicalLog`, recover poisoned test-only locks without
panicking, and are enabled only as a library dev-dependency. Recording keeps
clone snapshots in emit order. Rejecting increments `attempts` and returns a
fresh `v2::EventError::Routing` whose diagnostic code is
`DIAGNOSTIC_INVALID`; no production crate depends on the full logger for
tests.

## Public signatures fixed here

```rust
impl<P, S, StoresFn, RoutesFn> DaemonBuilder<P, S, StoresFn, RoutesFn> {
    #[must_use]
    pub fn logger(mut self, logger: SharedCanonicalLog) -> Self;
}

impl<S, C, J> Scheduler<S, C, J>
where
    S: ScheduleStore,
    C: Clock,
    J: JitterSource,
{
    #[must_use]
    pub fn with_logger(self, logger: SharedCanonicalLog) -> Self;
}
```

Existing constructors remain source-compatible and install an infallible
Noop with fixed library identity (`sc-runtime` or `sc-cron`). `share()` and
worker construction clone the same Arc through `RunCoord`. Libraries never
construct, flush, or shut down the full logger. Only an injected production
logger carries the generated application service name.

The component propagation seams are also fixed here and preserve every legacy
entry point by installing the fixed `sc-cron` Noop:

```rust
pub struct LoggedJitter<J> { /* inner J plus SharedCanonicalLog */ }
impl SystemJitter {
    pub fn with_logger(logger: SharedCanonicalLog) -> LoggedJitter<Self>;
}

impl SqliteScheduleStore {
    pub fn open_with_logger(
        path: impl AsRef<std::path::Path>,
        limits: ScheduleLimits,
        logger: SharedCanonicalLog,
    ) -> Result<Self, CronError>;
}

impl<C, J> Scheduler<SqliteScheduleStore, C, J> {
    pub fn open_with_logger(
        path: impl AsRef<std::path::Path>,
        limits: ScheduleLimits,
        clock: Arc<C>,
        jitter: Arc<J>,
        logger: SharedCanonicalLog,
    ) -> Result<Self, CronError>;
}

pub async fn supervise_with_logger<F>(
    spec: &ChildSpec<'_>,
    cap: usize,
    expiry: F,
    logger: SharedCanonicalLog,
) -> std::io::Result<Captured>
where F: std::future::Future<Output = ()>;
```

`SqliteScheduleStore::open`, `Scheduler::open`, and `process::supervise`
delegate with the fixed Noop. Internal `run_compose_*_with_logger`,
`drive_feed`, and `execute_feed_with_logger` functions carry the same handle
through `monty_compose.rs`, `monty_host.rs`, and `monty_run.rs`; their legacy
forms delegate with the Noop. b-15 supplies `RunCoord.logger()` at worker and
due-run call sites without reopening these component files.

This sprint migrates the component-owned events in its whole-file ownership:

| Target | Action | Level | Outcome | `Diagnostic.code` | Required fields / cardinality |
|---|---|---|---|---|---|
| `cron.jitter` | `source.unavailable` | warning | `failed` | `JITTER_SOURCE_UNAVAILABLE` | error; once per random-source failure |
| `cron.store` | `row.corrupt` | warning | `failed` | `CORRUPT_SCHEDULE_ROW` | schedule id/name when decoded, error; once per skipped corrupt row |
| `cron.process` | `child.kill_failed` | debug | `failed` | `CHILD_KILL_FAILED` | pid when available, error; once per failed kill attempt |
| `cron.process` | `child.reap_timeout` | warning | `timeout` | `CHILD_REAP_TIMEOUT` | pid when available, `deadline_remaining_ms = 0`; once per reap deadline |
| `cron.monty.compose` | `pipe.drain_timeout` | warning | `timeout` | `COMPOSE_PIPE_DRAIN_TIMEOUT` | `deadline_remaining_ms = 0`; once per pipe-drain deadline |
| `cron.monty.compose` | `child.kill_timeout` | warning | `timeout` | `COMPOSE_CHILD_KILL_TIMEOUT` | pid when available, `deadline_remaining_ms = 0`; once per post-kill deadline |

Component tests assert this exact target/action/level/outcome/code/field/
cardinality table. Rejecting emission may not
change jitter fallback, corrupt-row skipping, kill/reap, pipe draining,
compose exit, or Monty feed results.

## Canonical `LogEvent` construction

Every b-14/b-15 event uses `OBSERVATION_SCHEMA_VERSION`,
`Timestamp::now_utc()`, `logger.service_name().clone()`,
`ProcessIdentity::default()`, `trace: None`, and `state_transition: None`.
Targets/actions and levels come from the layer sprint tables. `message` may
carry a secret-free human summary but is never parsed by tests.

| Semantic value | Canonical location and type |
|---|---|
| HTTP request id | top-level `request_id: Option<CorrelationId>`; never duplicated in `fields` |
| cron run id | top-level `correlation_id: Option<CorrelationId>`; absent when no run exists |
| stable failure code | `diagnostic.code: ErrorCode`; never `fields["code"]` |
| outcome | top-level `outcome: Option<OutcomeLabel>` using `success`, `client_error`, `server_error`, `timeout`, `fenced`, `partial`, `stopped`, `refused`, or `failed` |
| method, path, schedule/target/hook stage, error text | string JSON values in `fields` |
| status, elapsed milliseconds, remaining deadline milliseconds, attempt, pid | non-negative integer JSON values in `fields` under `status`, `elapsed_ms`, `deadline_remaining_ms`, `attempt`, `pid` |

Diagnostics use the event timestamp, secret-free message/cause, recoverable
remediation, and the fixed code from the owning sprint. Successful/client
events have `diagnostic: None`. Arbitrary application 5xx responses also have
`diagnostic: None`; only framework-owned failures with a specified stable
code attach one. Deadline fields are remaining monotonic duration at the
event boundary, never a fabricated wall-clock deadline.

## Public release contract

b-13 changes the root workspace version from `0.1.0` to **`0.1.1`**, so the
new `sc-runtime` API and first publishable `sc-cron` injection API have an
immutable registry version. It removes `publish = false` from `sc-cron` and
keeps `sc-runtime-db-sqlite` unpublished. This sprint updates manifests and
lockfile only; it does not publish any crate. b-17 owns the existing
sc-publish consumer input, authorization gate, publication order, and
registry-only generated build.

Every emission is best effort. `emit` failure is observed in tests but never
replaces an HTTP response, `RuntimeError`, `CronError`, `RunOutcome`, stop
decision, or store result. The interface-owned scheduler events migrate in
this sprint; remaining cron taxonomy belongs to b-15.

## Required work and deliverables

Every numbered item must land production-ready for contract closure: public
signatures compile, defaults behave, manifests and decisions agree, and the
contract tests prove the committed semantics without placeholders.

1. Amend ADR-RUN-0002, ADR-RUN-0004, ADR-RUN-0007, ADR-RUN-0013, ADR-RT-0001 and the listed REQ/NFRs to
   allow only the types-level dependency, define Noop defaults, Arc identity,
   direct canonical emission, best-effort failure semantics, and the internal
   `0.1.1` publication boundary.
2. Add `sc-observability-types` to both boundary allowlists and crate
   manifests at exact version `1.5.1`; enable its `test-util` feature only for
   dev-dependencies. Neither library may resolve `sc-observability` or OTLP.
3. Implement the two public signatures, propagate one handle through runtime
   state and `RunCoord`, and preserve all old constructor calls.
   `Scheduler::with_logger` emits the production
   `cron.scheduler/logger.attached` event after replacement; b-17 consumes
   this interface-owned event without manufacturing a fake cron record.
   The same `scheduler.rs` ownership migrates its two shutdown-deadline trace
   calls to `cron.scheduler/shutdown.worker_timeout` (error) and
   `cron.scheduler/shutdown.deadline_exceeded` (warning), each emitted once
   per triggered shutdown branch with deadline/worker fields,
   `WORKER_SHUTDOWN_TIMEOUT` / `SHUTDOWN_DEADLINE_EXCEEDED`, and no outcome
   change.
   Implement the fixed jitter/store/scheduler/process/Monty propagation seams,
   keep every legacy constructor/function source-compatible with a Noop, and
   migrate the six component-owned event rows above.
4. NFR-RT-0002 / NFR-CRON-0001 / NFR-RUN-0002: contract tests prove object safety, Noop compatibility, replacement,
   Arc identity through `share()`/worker construction, immediate queue-full
   and disconnected rejection, and component propagation. Static assertions
   forbid `log(` and `enqueue_blocking` in the upstream adapter contract.
5. NFR-RT-0003 / NFR-RUN-0009: keep `tracing` manifest dependencies during the staged migration. Their
   cleanup is explicitly deferred; b-14 and b-15 remove owned production
   calls without reopening these shared manifests.
6. REQ-RUN-0002 / REQ-RUN-0003 / REQ-RUN-0702: bump the root workspace to
   `0.1.1`, make `sc-cron` publishable, retain `sc-runtime-db-sqlite` as
   unpublished, and inspect the package file lists for both changed public crates without
   claiming publication of the upstream observability pair.

## Acceptance criteria

1. ADR-RUN-0004 / ADR-RUN-0007 / ADR-RUN-0013 / ADR-RT-0001: the amended decision and
   requirement documents record the exact object-safe trait, Arc ownership,
   Noop defaults, dependency edges, and best-effort failure policy.
2. ADR-RUN-0004: boundary lint accepts the two types-only
   edges and continues to reject the full runtime and OTLP.
3. `boundary:BOUNDARY-sc-runtime`: rustdoc exposes `DaemonBuilder::logger`; the typestate
   surface is exactly `logger`, `stores`, `routes`, `mcp`, and `run`.
4. `boundary:BOUNDARY-sc-cron`: `Scheduler::new` and `open` still compile; an injected
   handle is identical in shared and worker state.
5. `boundary:BOUNDARY-sc-cron`: each scheduler shutdown branch emits its
   fixed target/action/level/code once, and a rejecting emitter does not
   change the returned shutdown result.
6. ADR-RUN-0004 / ADR-RUN-0007 / NFR-RUN-0002: a rejecting emitter cannot
   change any primary outcome; saturated and disconnected production logger
   paths reject immediately without blocking scheduler, HTTP, process, or
   Monty execution.
7. NFR-RT-0004 / NFR-CRON-0001: legacy constructors install fixed
   `sc-runtime` / `sc-cron` Noops without parsing application input or
   panicking; only injection changes the service identity.
8. REQ-RUN-0702: `cargo metadata` reports workspace version `0.1.1`, both
   public crates package, `sc-cron` is publishable, and no release command has
   been run by this sprint.

## Required validation

```sh
test "$(gh pr view 84 --json state --jq '.state')" = MERGED
git fetch origin develop
git merge-base --is-ancestor "$(gh pr view 84 --json mergeCommit --jq '.mergeCommit.oid')" origin/develop
cargo test -p sc-runtime --test observability_contract
cargo test -p sc-cron --test observability_contract
cargo test -p sc-runtime --features test-util
cargo test -p sc-cron
cargo tree -p sc-runtime -e normal --depth 1 --prefix none
cargo tree -p sc-cron -e normal --depth 1 --prefix none
cargo package --list -p sc-runtime
cargo package --list -p sc-cron
sc-lint lint sc-boundary --root . --config sc-lint.toml
! rg -n 'tracing::|trace!|debug!|info!|warn!|error!' crates/sc-cron/src/scheduler.rs
! rg -n 'tracing::|trace!|debug!|info!|warn!|error!' crates/sc-cron/src/jitter.rs crates/sc-cron/src/sqlite.rs crates/sc-cron/src/process.rs crates/sc-cron/src/monty_compose.rs crates/sc-cron/src/monty_host.rs crates/sc-cron/src/monty_run.rs
! rg -n 'enqueue_blocking|\.log\(' crates/sc-runtime crates/sc-cron
just lint gates
```

The tree output must contain the tested types patch directly and contain no
direct `sc-observability` or `sc-observability-otlp` line.

## This sprint does not close

- HTTP event taxonomy (b-14), cron event taxonomy (b-15), generated wiring
  (b-17), or upstream release work.
- Removal of every diagnostics dependency.
