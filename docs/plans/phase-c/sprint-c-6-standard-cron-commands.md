---
id: c-6
phase: c
status: planned
closure_type: boundary
target_boundary: sc-cron-commands
track: cron-commands
wave: 2
branch: sprint/c-6-standard-cron-commands
worktree: sprint/c-6-standard-cron-commands
requirements:
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
adrs:
  - ADR-RUN-0019
  - ADR-RUN-0018
  - ADR-RUN-0003
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CRNC-0001
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

1. REQ-CRNC-0001: strict versioned/tagged start/end/list DTOs, schema and
   canonical typed response. Start validates expression/timeout/target/payload
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
   docs naming the shared contract and opt-in composition.

## Acceptance criteria

Root: `boundary:BOUNDARY-sc-cron-commands`, `adr:ADR-CRNC-0001`.

- Serialization/schema tests demonstrate one canonical versioned start/end/list
  and Envelope result/error across projection-neutral fixtures. All invalid
  forms above fail before scheduler/store work; no absent name means end-all.
- Integration tests #[path]-import c-1's canonical
  crates/sc-cron-commands/tests/support/operation_double.rs and construct a real context
  via sc-cron/test-utils operation_context_probe enabled only on the dev edge.
  No c-5 Scheduler implementation or default/production test helper is needed.
  The explicit non-default dev-only context probe is the sole setup seam. Tests prove
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

## Required validation

Adapter target tests/lints under DTO-only, scheduler and scheduler+Monty
configurations; contract-double tests at c-1's declared allowed-test-double
path; workspace build and configured boundary validation (`just lint gates`).
Run strict schema/serialization and dependency graph regressions.

## This sprint does not close

Real cron SQL/worker behavior (c-5), template implementation (c-3), actual
CLI/HTTP/MCP composition or registry publication (c-4). C-4 is the sole
feature-level integration owner for the standard command surface.
