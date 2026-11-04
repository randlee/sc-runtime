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
  - NFR-RUN-0009
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
by c-1. c-3 alone changes later shared ecosystem manifests and Cargo.lock.
C-1 first enables rusqlite 0.39.x `bundled,hooks` and qualifies interrupt/busy/
progress facilities with its lock baseline; c-3 retains these features during
migration.
C-5 preserves that qualified version/features in its SQLite implementation and
isolated consumer manifest/lock, with no rusqlite/SQLx/libsqlite3-sys version
upgrade. It compiles against c-1's committed prerequisite, with no shared manifest
edits or dependency on an uncommitted c-3 feature change.

## Deliverables

1. REQ-CRON-0020: expose the same nominal CanonicalLog receiver through
   `sc_cron::observability`, including its supporting event/service/error
   values. Keep `Scheduler::new_with_logger` and defaults-off command edge;
   no new logging trait or full observability dependency.
2. REQ-CRON-0020: implement c-1's cooperative store scope before every
   potentially blocking lock, initial load, reload, row decode and write in the
   built-in cooperative path,
   including retry/error/end paths. SQLite busy/progress/interrupt checks
   observe shared mutable deadlines. Final owned commit_hook serializes briefly
   with live stop state/class/generation/real time and releases controller lock
   before COMMIT. No new ordinary commit admission at/after G or cleanup
   admission at/after K. Admitted commits may complete later; retain ownership,
   classify actual persisted result, and restore known hooks/TLS on safe paths.
   Never infer rollback from expiry/Drop or bool false; restricted cleanup and
   typed TransactionOutcome follow ADR-RUN-0018.
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
   generation fencing, bundled row schema and transaction semantics.
6. REQ-CRON-0020/0021 and the framework charter: extend crates/sc-cron/README.md
   with a runnable custom ScheduleStore example persisting an application field
   in a consumer-owned schema. Exercise it in tests/integration/cron-consumer/
   through public scheduling/query/end and replacement-generation fencing.
   State whether the example forwards cooperative support/scope/query or uses
   the legacy caller-owned path; prove the advertised choice. Custom cooperative
   implementations meet the contract through provider-specific mechanisms,
   not mandatory SQLite hooks. Document legacy lifecycle ownership and custom
   CronOperations integration, with no default guarantee inferred for arbitrary
   custom code. Do not extend core Schedule/DTOs with application fields.

## Acceptance criteria

- Implement and causally prove the default command-intake gate frozen by c-1
  under REQ-CRON-0020/0021 and ADR-RUN-0018: default64 per Scheduler, immutable
  validated pre-start override 1..=65536, shared across clones/context-aware
  public operations. Permit ownership covers registered/queued/physical work,
  including cancellation of its waiter. Full intake refuses immediately with
  fixed typed unavailable/no-mutation evidence; no hidden waiter queue. Hold
  store work, exceed capacity and race cancellation/deadline/stop; verify bounded
  submitted work, exact permit release after physical completion, and independent
  joined generation-fenced cleanup while full. Public commands cannot enter
  private worker/cleanup paths to bypass saturation. Existing scheduled-job
  concurrency and embedder row-limit policy remain separate; no global job lock.

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
  detect unauthorized post-cutoff admission, effects after physical release,
  detached work, callbacks waiting for
  join, unsupported store admission and timer restart after lock acquisition.
- Physical joins retain every admitted blocking closure/owned task through
  completion; timeout reporting does not claim hard preemption of arbitrary
  Rust/native/custom Monty hosts or universal physical commit/fsync by 2T.
  Confirmed committed-late and indeterminate remain accurate deadline failures,
  never rollback/not-persisted or successful bounded cleanup.
- Causal gates after final precheck and at the real commit-hook decision race
  ordinary G/cleanup K, shortening and second stop in both serial orderings.
  Stop-first vetoes; admission-first records admission and actual later state.
  Pause after allow without holding controller lock; independent stop/expiry
  must still acknowledge while blocked. Inspect real rows/generation after
  physical join for accurate committed-late/rollback/indeterminate classification,
  including implicit writes. No replay of uncertain create/end/toggle, no
  expired-context readback. An injected lost-result test proves classification
  only, not a physically reproduced fsync failure.
- Real query returns ended as well as active rows with deterministic bounded
  continuation; malformed limits/cursors fail; replacement races preserve
  generation fencing. Cancellation of command waiters leaves no untracked work.
- External default consumer runs Rust/Python; explicit Monty consumer runs
  Monty/custom host. Both receive actual attached/launched/finished events
  without direct sc-command dependency. Default excludes Monty; neither
  activates clap/Axum/RMCP/command-server/full observability or tool Rust crates.

- The small custom-store consumer persists and reads its extra field across
  schedule/query/end, preserving generation conflicts and the example's declared
  lifecycle policy. Unsupported cooperative capability still fails before work;
  a forwarding provider preserves live cancellation scope where advertised.
  The README demonstrates only public APIs and distinguishes bundled-schema
  invariants from application schema freedom. No portable query framework or
  consumer-product migration is needed.

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
publication (c-4). C-5 is Phase C's dedicated compatibility sprint and completes before c-4
publishes sc-runtime 0.2.0, alongside its existing c-3/c-6 real-artifact
prerequisites.
