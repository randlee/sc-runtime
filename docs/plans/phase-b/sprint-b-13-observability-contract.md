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
  - REQ-RUN-0301
  - REQ-RUN-0309
  - REQ-RUN-0503
  - REQ-RUN-0701
  - REQ-RUN-0702
  - REQ-RT-0001
  - REQ-RT-0005
  - NFR-RT-0001
  - NFR-RT-0002
  - NFR-RT-0003
  - REQ-CRON-0001
  - REQ-CRON-0003
  - NFR-CRON-0001
  - NFR-RUN-0002
  - NFR-RUN-0005
  - NFR-RUN-0008
  - NFR-RUN-0009
  - NFR-RUN-0010
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0004
  - ADR-RUN-0007
  - ADR-RUN-0011
  - ADR-RUN-0013
  - ADR-RUN-0303
  - ADR-CMD-0001
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
  - docs/sc-command/architecture.md
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
  - crates/sc-cron/src/hook.rs
  - crates/sc-cron/src/monty_compose.rs
  - crates/sc-cron/src/monty_host.rs
  - crates/sc-cron/src/monty_run.rs
  - crates/sc-cron/tests/observability_contract.rs
  - crates/sc-cron/tests/observability_components.rs
must_follow: []
parallel_safe: []
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

The baseline is `develop` after Track B Monty merged at `240e445`. This sprint
extends that landed host contract with logger propagation; it does not reopen
or restate the host allowlist, program-selection, or recovery rules.

## Relations

b-10 through b-12 are already part of the baseline. b-13 establishes the
graph and requirement authority that b-14, b-15, and b-16 consume in the next
wave without replanning that closed work.

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
This is a trusted synchronous trait boundary: custom implementations must
return promptly, perform no sink I/O or capacity wait, and not panic. The
libraries do not spawn or time-box `emit`; acceptance proves the published
Logger adapter and supplied doubles, not adversarial third-party trait code.

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

```

Existing constructors remain source-compatible and install an infallible
Noop with fixed library identity (`sc-runtime` or `sc-cron`). The builder
`logger` method is available in `NeedStores`, `NeedRoutes`, and `Ready`,
returns the same state, and adds no `NeedLogger` state or `run()` step.
Repeated `logger` calls are last-write-wins and drop the replaced handle.
`share()` and
worker construction clone the same Arc through `RunCoord`. Libraries never
construct, flush, or shut down the full logger. Only an injected production
logger carries the generated application service name.

The component propagation seams are also fixed here and preserve every legacy
entry point by installing the fixed `sc-cron` Noop:

```rust
impl SystemJitter {
    pub fn with_logger(logger: SharedCanonicalLog) -> Self;
}

impl SqliteScheduleStore {
    pub fn open_with_logger(
        path: impl AsRef<std::path::Path>,
        limits: ScheduleLimits,
        logger: SharedCanonicalLog,
    ) -> Result<Self, CronError>;
}

impl<C> Scheduler<SqliteScheduleStore, C, SystemJitter> {
    pub fn open_with_logger(
        path: impl AsRef<std::path::Path>,
        limits: ScheduleLimits,
        clock: Arc<C>,
        logger: SharedCanonicalLog,
    ) -> Result<Self, CronError>;
}

pub async fn supervise_with_logger<F>(
    spec: &ChildSpec<'_>,
    cap: usize,
    expiry: F,
    logger: SharedCanonicalLog,
) -> Result<Captured, CronError>
where F: std::future::Future<Output = ()>;
```

`SqliteScheduleStore::open`, `Scheduler::open`, and `process::supervise`
delegate with the fixed Noop. `Scheduler::open_with_logger` is the sole
public logger-aware scheduler constructor: it constructs
`SystemJitter::with_logger(logger.clone())` internally and installs the same
Arc into the store, concrete jitter, and `RunCoord`, then emits
`cron.scheduler/logger.attached`. There is no post-construction
`Scheduler::with_logger` path and no public jitter wrapper. Internal
`run_compose_*_with_logger`,
`drive_feed`, and `execute_feed_with_logger` functions carry the same handle
through `monty_compose.rs`, `monty_host.rs`, and `monty_run.rs`; their legacy
forms delegate with the Noop. This sprint implements logger-aware
`PythonHook` construction/run and the `supervise_with_logger` call inside
`hook.rs`. b-15 owns only the worker call site that passes
`RunCoord.logger()` into this API, plus its worker and due-run event sites,
without reopening these b-13 component files.

`execute_feed_with_logger` is the internal logger-bearing counterpart of the
landed `execute_feed_with_compose_program` path: it receives both the
embedder-selected compose program and `SharedCanonicalLog`. It is not a new
public entry point and does not replace `drive_feed` as the production path.

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

HTTP `path` is the Axum matched-route template, never the raw URI, query, or
path-parameter value; unmatched requests use the fixed `<unmatched>` token.

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
lockfile only; it does not publish any crate. b-18 owns the existing
sc-publish consumer input, authorization gate, publication order, and
registry-only generated build.

Every emission is best effort. `emit` failure is observed in tests but never
replaces an HTTP response, `RuntimeError`, `CronError`, `RunOutcome`, stop
decision, or store result. The interface-owned scheduler events migrate in
this sprint; remaining cron taxonomy belongs to b-15.

## Required work and deliverables

1. Amend ADR-RUN-0002, ADR-RUN-0003, ADR-RUN-0004, ADR-RUN-0007,
   ADR-RUN-0011, ADR-RUN-0013, ADR-RUN-0303, ADR-CMD-0001, ADR-RT-0001 and the listed REQ/NFRs to
   allow only the types-level dependency, define Noop defaults, Arc identity,
   direct canonical emission, best-effort failure semantics, and the internal
   `0.1.1` publication boundary. In particular, amend
   REQ-RUN-0301/ADR-RUN-0303 for the generated daemon's direct
   observability/types/transport/command edges and optional cron graph; amend
   the REQ-RT-0001 and
   NFR-RT-0001 method tables for optional all-phase `.logger()` without a new
   typestate; REQ-RUN-0309 for generated/runtime/cron caret `1.5.1`
   requirements, the retained sc-command tested minimum `1.5.0`, and exact
   lock resolution `1.5.1`; REQ-CRON-0001 and ADR-RUN-0303 for the optional generated
   cron graph; REQ-RUN-0702 for the five-crate release order; and
   ADR-CMD-0001 to remove its stale only-library assertion. Amend REQ-RT-0005
   to close its stale OPEN around the already-landed 30-second request/drain
   envelopes and stable codes. Amend NFR-RUN-0010 to bind the generated real-daemon signal tests to the
   dedicated `ubuntu-latest` GitHub-hosted job and its fail-closed three-value
   detection contract; b-18 remains the sole integration acceptance root.
   This sprint preserves the landed host behavior while adding observability.
   Its successor b-16 explicitly amends ADR-RUN-0013 and REQ-CRON-0003 for
   application-defined host registration and the two CLI adapters. That
   authorized interface change supersedes the pre-release fixed host list;
   it is not part of this sprint's logger migration.
2. Add `sc-observability-types` to both boundary allowlists and crate
   manifests with the caret-compatible requirement `1.5.1`; enable its `test-util` feature only for
   dev-dependencies. Neither library may resolve `sc-observability` or OTLP.
   ANC-031 / REQ-RUN-0005 applies throughout: this sprint adds no production
   sc-compose or ATM crate dependency/import to any of the five release
   crates. Test/benchmark-only references remain allowed. b-16 owns the
   complete cross-crate enforcement fixtures and package-identity checks.
   `Cargo.lock` proves the exact resolved `1.5.1` pair.
3. Implement the public signatures fixed above, propagate one handle through runtime
   state and `RunCoord`, and preserve all old constructor calls.
   `Scheduler::open_with_logger` installs one handle coherently and emits the
   production `cron.scheduler/logger.attached` event; b-18 consumes
   this interface-owned event without manufacturing a fake cron record.
   The same `scheduler.rs` ownership migrates its two shutdown-deadline trace
   calls to `cron.scheduler/shutdown.worker_timeout` (error) and
   `cron.scheduler/shutdown.deadline_exceeded` (warning), each emitted once
   per triggered shutdown branch with deadline/worker fields,
   `WORKER_SHUTDOWN_TIMEOUT` / `SHUTDOWN_DEADLINE_EXCEEDED`, and no outcome
   change.
   Implement the fixed jitter/store/scheduler/process/Monty propagation seams,
   keep every legacy constructor/function source-compatible with a Noop, and
   migrate the six component-owned event rows above. In `hook.rs`, add the
   logger-aware `PythonHook` API and route it to `supervise_with_logger`; b-15
   later passes the existing `RunCoord` handle at its worker-owned call site.
4. NFR-RT-0002 / NFR-CRON-0001 / NFR-RUN-0002 / NFR-RUN-0005: contract tests prove object safety, Noop compatibility, replacement,
   Arc identity through `share()`/worker construction, immediate queue-full
   and disconnected rejection, component propagation, and isolated
   temp-root/parallel-safe execution. Source-scoped static assertions reject
   full-runtime dependencies and direct blocking adapter calls in this
   repository; the upstream adapter's non-blocking behavior is proved by its
   published contract tests, not by grepping unrelated test helpers here.
5. NFR-RT-0003 / NFR-RUN-0009: keep `tracing` manifest dependencies during the staged migration. Their
   cleanup is explicitly deferred; b-14 and b-15 remove owned production
   calls without reopening these shared manifests.
6. REQ-RUN-0002 / REQ-RUN-0003 / REQ-RUN-0005 / REQ-RUN-0503 /
   REQ-RUN-0701 / REQ-RUN-0702: bump the root workspace to `0.1.1`, make
   `sc-cron` the fifth independently packaged public crate and generated
   versioned dependency, retain `sc-runtime-db-sqlite` as unpublished, amend
   both generated green/CI observability checks to `1.5.1`, and inspect the
   package file lists for both changed public crates without claiming
   publication of the upstream observability pair. This sprint amends the
   release and boundary authority; b-18 owns each integration acceptance
   root.
7. NFR-RUN-0008: keep every changed production Rust source below the existing
   1000-source-line gate; this ID governs only `just lint` line counts, not
   request, scheduler, or error behavior.
8. ADR-RUN-0011: production changes add no sleep or `block_on` call and hold
   no `std::sync::Mutex` guard across `.await`; test-only deadline polling is
   exempt under the existing scanner rules.

## Acceptance criteria

1. ADR-RUN-0004 / ADR-RUN-0007 / ADR-RUN-0013 / ADR-RT-0001:
   `observability_contract` for both crates passes against the documented
   object-safe trait, Arc ownership, Noop defaults, dependency edges, and
   best-effort failure policy.
2. ADR-RUN-0004: the required `sc-lint lint sc-boundary` command exits zero
   with both types-only edges allowed and the full runtime and OTLP rejected.
3. `boundary:BOUNDARY-sc-runtime`: the runtime `observability_contract` test
   passes only when `DaemonBuilder::logger` preserves typestate and the public
   builder surface remains `logger`, `stores`, `routes`, `mcp`, and `run`.
4. `boundary:BOUNDARY-sc-cron`: the cron `observability_contract` and
   `observability_components` tests pass only when legacy constructors use a
   Noop and `open_with_logger` installs one identical handle in store, jitter,
   shared, and worker state as the sole public logger-aware constructor.
5. `boundary:BOUNDARY-sc-cron`: `observability_components` passes only when
   each scheduler shutdown branch emits its fixed row once and rejection does
   not change the returned shutdown result.
6. ADR-RUN-0004 / ADR-RUN-0007: both `observability_contract` tests pass only
   when rejecting, saturated, and disconnected emitters return immediately
   without changing scheduler, HTTP, process, or Monty outcomes.
7. `boundary:observability-injection`: both `observability_contract` tests
   pass only when legacy constructors install fixed `sc-runtime` / `sc-cron`
   Noops and only explicit injection changes service identity.
8. ADR-RUN-0002: both `cargo package --list` commands and `cargo metadata`
   inspection pass with workspace version `0.1.1`, publishable `sc-runtime`
   and `sc-cron`, and no publication performed by this sprint.

## Required validation

```sh
cargo test -p sc-runtime --test observability_contract
cargo test -p sc-cron --test observability_contract
cargo test -p sc-cron --test observability_components
cargo +1.96.1 test -p sc-cron --features monty --test observability_components
cargo test -p sc-runtime --features test-util
cargo test -p sc-cron
cargo tree -p sc-runtime -e normal --depth 1 --prefix none
cargo tree -p sc-cron -e normal --depth 1 --prefix none
cargo metadata --no-deps --format-version 1
cargo package --list -p sc-runtime
cargo package --list -p sc-cron
sc-lint lint sc-boundary --root . --config sc-lint.toml
! rg -n '#\[(tracing::)?instrument|tracing::|(trace|debug|info|warn|error|event|span|trace_span|debug_span|info_span|warn_span|error_span)!' crates/sc-cron/src/lib.rs crates/sc-cron/src/coord.rs crates/sc-cron/src/scheduler.rs crates/sc-cron/src/jitter.rs crates/sc-cron/src/sqlite.rs crates/sc-cron/src/process.rs crates/sc-cron/src/hook.rs crates/sc-cron/src/monty_compose.rs crates/sc-cron/src/monty_host.rs crates/sc-cron/src/monty_run.rs
rg -n '\bsupervise_with_logger\(' crates/sc-cron/src/hook.rs
! rg -n '\bsupervise\(' crates/sc-cron/src/hook.rs
! rg -n 'enqueue_blocking|\.log\(' crates/sc-runtime/src crates/sc-cron/src
just lint
just lint gates
```

The tree output must contain the tested types patch directly and contain no
direct `sc-observability` or `sc-observability-otlp` line.

## This sprint does not close

- HTTP event taxonomy (b-14), cron event taxonomy (b-15), generated
  observability wiring (b-18), or upstream release work.
- Removal of every diagnostics dependency.
