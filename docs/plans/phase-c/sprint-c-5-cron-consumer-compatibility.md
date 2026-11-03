---
id: c-5
phase: c
status: planned
closure_type: integration
target_boundary: sc-cron-consumer-compatibility
track: cron
wave: 2
branch: sprint/c-5-cron-consumer-compatibility
worktree: sprint/c-5-cron-consumer-compatibility
requirements:
  - REQ-CRON-0001
  - REQ-CRON-0002
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
  - REQ-CRON-0019
  - REQ-CRON-0020
  - REQ-CRON-0021
  - NFR-CRON-0001
  - NFR-CRON-0002
  - REQ-RUN-0005
  - NFR-RUN-0002
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0013
  - ADR-RUN-0014
  - ADR-RUN-0016
  - ADR-RUN-0017
  - ADR-RUN-0018
  - ADR-RUN-0011
owned_paths:
  - crates/sc-cron/src/
  - crates/sc-cron/tests/
  - crates/sc-cron/README.md
  - tests/integration/cron-consumer/
must_follow: [c-1]
parallel_safe: [c-2, c-3, c-6]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One existing scheduler implementation and an isolated external consumer prove the same public compatibility seam; consumer product migration and shared command composition remain outside this sprint.
---

# Sprint c-5 — cron consumer compatibility

## Goal

Make the existing cron library consumable through its public logging facade
and cooperative operation/lifecycle contract. This preserves the already
ported Rust, Python and optional Monty execution targets; it is no new port,
handler catalog or job type.

Merge every c-1 development push before each development/fix round. Parent
QA does not delay development; c-1 merges before this PR completes. Consume
the signatures, error policy, clock/budget policy and query contract frozen
by c-1. c-3 alone changes shared ecosystem manifests and Cargo.lock. Prove
this implementation compiles against c-1's baseline; a required SQLite
feature must be hoisted into c-1 before declaring parallel-safe, never patched
concurrently in c-3/c-5.

## Deliverables

1. REQ-CRON-0020: expose the same nominal CanonicalLog receiver through
   `sc_cron::observability`, including its supporting event/service/error
   values. Keep `Scheduler::new_with_logger` and defaults-off command edge;
   no new logging trait or full observability dependency.
2. REQ-CRON-0020: implement c-1's cooperative store scope before every
   potentially blocking lock, initial load, reload, row decode and write,
   including retry/error/end paths. SQLite busy/progress/interrupt checks
   observe the shared mutable deadline; cancelled transactions roll back and
   restore connection hooks/settings before release. No ordinary commit is
   admitted after grace; restricted cleanup follows ADR-RUN-0018.
3. REQ-CRON-0020/0021: implement independently callable stop control,
   monotonic deadline shortening and one owning physical join. Track admitted
   command CRUD/query and worker store tasks before submission until actual
   completion. A dropped transport waiter cannot detach a blocking write.
   Legacy stores keep compiling; cooperative admission rejects unsupported
   stores before launching work. Arbitrary direct legacy store calls remain
   caller-owned and arbitrary native/custom hosts get no hard-bound claim.
4. REQ-CRON-0021: implement bounded active/ended/all query with optional name,
   positive capped limit, stable id ordering and continuation. This is a new
   store capability, not an alias for `list_active`. Legacy default returns
   unsupported. Context-aware start/stop/query use the same admission registry;
   stop includes generation fencing and cannot end a replacement generation.
5. REQ-CRON-0003/0019: isolated external consumer depends on sc-cron only as
   its SDK (ordinary test/runtime utilities explicit), implements the receiver
   via its facade, registers and causally executes Rust, Python and Monty
   schedules through existing APIs. Monty includes a custom application host.
   Preserve empty-default environment, resolver/redaction, stop-before-tick,
   generation fencing, row schema and transaction semantics.

## Acceptance criteria

Roots: `req:REQ-CRON-0003`, `req:REQ-CRON-0019`,
`req:REQ-CRON-0020`, `req:REQ-CRON-0021`.

- Causal tests hold the connection mutex before admission and shorten an
  already waiting deadline; exercise initial load/list/reload and every
  persistence path, busy/progress/decode and queued closures. Advancing the
  real stop clock and explicit barriers prove cancellation reaches work while
  blocked, rather than only the async loop after return.
- Expiry notification and consumer cancellation run at grace independently of
  join. Second stop shortens the same state without extending cleanup.
  T=0 follows c-1's explicit grace/cleanup budget contract. Negative tests
  detect late writes after cancellation, detached work, callbacks waiting for
  join, unsupported store admission and timer restart after lock acquisition.
- Physical joins retain every admitted blocking closure/owned task through
  completion; timeout reporting does not claim hard preemption of arbitrary
  Rust/native/custom Monty hosts. Missed cleanup is reported honestly.
- Real query returns ended as well as active rows with deterministic bounded
  continuation; malformed limits/cursors fail; replacement races preserve
  generation fencing. Cancellation of command waiters leaves no untracked work.
- External default consumer runs Rust/Python; explicit Monty consumer runs
  Monty/custom host. Both receive actual attached/launched/finished events
  without direct sc-command dependency. Default excludes Monty; neither
  activates clap/Axum/RMCP/command-server/full observability or tool Rust crates.

## Required validation

Targeted cron tests in default and Monty configurations, isolated external
consumer tests and graph guards, `just lint gates`, workspace build and
`just lint && just test`. Use causal handshakes and real-Instant stop-budget
tests; ManualClock remains only for schedule semantics. A test-only forced
expiry proves propagation separately from elapsed completion bounds.
The independent fixture owns its manifest/lock under its dedicated path;
it does not alter shared workspace/template manifests.

## This sprint does not close

Consumer beads-cache migration or QA, runtime signals/fixtures (c-3), standard
command DTO/handler boundary (c-6), generated UI command composition and registry
publication (c-4). The last requires c-3, c-5 and c-6 real artifacts together.
