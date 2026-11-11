---
id: c-6
phase: c
status: complete
closure_type: boundary
target_boundary: sc-cron-commands
track: cron-commands
wave: 2
branch: sprint/c-6-standard-cron-commands
worktree: sprint/c-6-standard-cron-commands
requirements:
  - REQ-RUN-0005
  - REQ-CRON-0005
  - REQ-CRON-0020
  - REQ-CRNC-0001
  - REQ-CRNC-0002
  - NFR-CRNC-0001
  - REQ-CMD-0001
  - REQ-CMD-0002
  - REQ-CMD-0003
  - REQ-CMD-0006
  - REQ-CRON-0002
  - REQ-CRON-0003
  - REQ-CRON-0021
  - NFR-RUN-0001
  - NFR-RUN-0009
  - NFR-RUN-0002
  - NFR-RUN-0005
  - NFR-RUN-0003
  - NFR-RUN-0004
  - NFR-RUN-0008
  - NFR-CMD-0003
  - NFR-CMD-0001
  - NFR-CMD-0002
adrs:
  - ADR-RUN-0001
  - ADR-RUN-0002
  - ADR-RUN-0005
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0008
  - ADR-RUN-0011
  - ADR-RUN-0013
  - ADR-RUN-0016
  - ADR-RUN-0019
  - ADR-RUN-0018
  - ADR-RUN-0003
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CRNC-0001
  - ADR-CMD-0003
  - ADR-RUN-0203
owned_paths:
  - crates/sc-cron-commands/src/
  - crates/sc-cron-commands/tests/
  - crates/sc-cron-commands/schemas/
  - crates/sc-cron-commands/README.md
must_follow: [c-1]
parallel_safe: [c-2, c-3, c-5]
recommended_agent: solar
recommended_model: sol
---

# Sprint c-6 — standard cron commands

## Goal

Implement one optional start/end/list command contract and scheduler adapter,
with consistent JSON data/errors for CLI, HTTP and MCP projections. Rust,
Python and optional Monty select existing hook targets. The scheduler core
continues to work without commands; no mandatory binary or framework registry.

Merge every c-1 development push before every development/fix round; parent
QA is not a start barrier and c-1 merges before this PR completes. Consume its
versioned DTOs, exact errors/limits, callable scheduler signatures and admitted
operation contract double. c-5 real implementation is not this sprint's test
dependency. c-1 owns initial crate manifest/membership; c-3 owns later shared
versions/lock. No template source or shared registry edits here.

## Deliverables

1. REQ-CRNC-0001: preserve c-1's compileable sole versioned/tagged DTO/schema
   declarations and implement their strict validation and canonical typed
   response behavior. Start validates expression/timeout/target/payload
   and allowed environment; end requires name and generation; list provides
   active/ended/all bounded id-ordered continuation. Shared generation types
   remain DTO primitives at the default boundary, not imports of core cron.
2. REQ-CRNC-0001/0002: StandardCronExecutor implements default-surface
   CronExecutor over object-safe native CronOperations and scheduler-feature
   ApplicationTargetBindings, exactly as compatibility-contract.md specifies.
   Its sole CronRequest DTO derives a child context with the earliest deadline;
   no concrete Scheduler or undefined second DTO type enters the executor seam.
   The optional `scheduler` executor delegates to c-1's tracked operations.
   Application binding resolves
   the configured Rust Job/payload and authorizes Python/Monty sources; no
   dynamic function pointers, second registry or independent scheduling store.
   Unsupported Monty builds fail capability validation before any write.
3. REQ-CRNC-0001: stable namespaced error mapping to existing Envelope/OpError.
   Reject unknown/duplicate fields, unsupported version, invalid generation,
   bad expression/limit/cursor and checked-duration overflow. Bound total
   request, source/path/payload sizes by c-1's frozen core limits. Do not echo
   environment values, source, payload, SQL or provider prose in errors/logs.
   Preserve typed admission/persistence evidence on DeadlineExceeded or
   Indeterminate errors: neither means rollback/not-persisted or permits an
   automatic mutation retry. Reconciliation is separately authorized/bounded.
4. NFR-CRNC-0001: DTO-only build uses serde/serde_json and defaults-off
   sc-command; scheduler feature alone introduces optional cron/async edges.
   Preserve minimal graph and no core runtime edge. Add standard crate usage
   docs naming the shared contract and opt-in composition. Extend the existing
   crate README with the selected-default recipe frozen by c-1: exact generation
   selection, bundled SQLite/store and Rust/Python/optional Monty registration,
   explicit worker startup, standard commands and stop/join/outcome handling.
   Link c-5's custom-store example; document replacing CronOperations or
   CronExecutor, consumer-owned lifecycle/authorization/error responsibilities,
   and where the standard cooperative guarantee ends. Keep defaults off and
   disabled MCP disabled; no automatic job start or application migration.

## Acceptance criteria

- Preserve c-1's owner-local schema trait implementations and frozen optional
  feature/dependency baseline. Test defaults-only, schema-only, scheduler-only
  and schema+scheduler against the canonical DTO/fixture shapes, including
  actual command-owned Envelope<CronResponse>/OpError schemas and their
  serialized success/failure/nullability cases, with no
  accidental feature leakage or second wire model. This does not consume
  c-5's real implementation; c-4 owns actual schema-bearing UI composition.

- Map the core's fixed `command_capacity_exhausted` reason through the existing
  typed unavailable Envelope category without exposing SQL/provider/request
  values or creating a second capacity-wait queue. Operation-double tests cover
  immediate refused/no-mutation results versus admitted cancellation/uncertain
  outcomes; only the former may recommend retrying a refused request. Document
  default64, validated finite pre-start core configuration, shared capacity and
  the caller's responsibility for custom executor/operation overload policy.
  C-5 proves physical permit retention; c-4 proves actual mixed-UI saturation.

Root: `boundary:BOUNDARY-sc-cron-commands`, `adr:ADR-CRNC-0001`.

- Serialization/schema tests demonstrate one canonical versioned start/end/list
  and Envelope result/error across projection-neutral fixtures. All invalid
  forms above fail before scheduler/store work; no absent name means end-all.
- Integration tests #[path]-import c-1's canonical
  crates/sc-cron-commands/tests/support/operation_double.rs and construct a real context
  via the ordinary public WorkerStopControl::new/operation_context factory.
  The dev-only Tokio runtime hosts this normal API; no c-5 Scheduler
  implementation or default/production test helper is needed. Tests prove
  exact request conversion, context forwarding,
  generation conflict, bounded query continuation, app target binding and
  stable error mapping. Dropped waiter behavior follows c-1's owned-operation
  contract; c-6 does not implement a second timeout wrapper.
- DTO-only dependency graph excludes cron/rusqlite/Tokio/Monty/Axum/RMCP/clap
  and command/server. Scheduler and explicit Monty enable only approved edges.
  The adapter never imports ATM/sc-compose product types or six-field-only
  grammar, team/phase fields, mandatory-until or a second response envelope.
- No placeholder/stub path remains within adapter serialization/handler
  implementation; real SQLite worker and generated UI composition remain
  explicitly owned by c-4, using c-5's implementation.

- A focused public-boundary test supplies an application CronExecutor and uses
  the same CronRequest/Envelope protocol without concrete Scheduler, private
  APIs or a new timeout registry in StandardCronExecutor. Document that custom
  executor policy and physical ownership are the application's responsibility;
  this does not relax standard executor scope forwarding or validation tests.
  Local recipe snippets compile against public APIs; c-4 alone proves the real
  selected generated root wiring after consuming c-5/c-6 artifacts.

## Required validation

Adapter target tests/lints under DTO-only, scheduler and scheduler+Monty
configurations; contract-double tests at c-1's declared allowed-test-double
path; workspace build and configured boundary validation (`just lint gates`).
Run strict schema/serialization and dependency graph regressions.

## This sprint does not close

Real cron SQL/worker behavior (c-5), template implementation (c-3), actual
CLI/HTTP/MCP composition or registry publication (c-4). C-4 is the sole
feature-level integration owner for the standard command surface.
