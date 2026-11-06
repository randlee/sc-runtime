# Consumer extension baseline diagnostic

Status: provisional engineering diagnostic against the direct human clarification. ANC-018/019 remain candidates, with statements/fixtures not locked; this is neither a Jev result nor a Plan Anchor gate or new critical-review certification. The six-statement approval packet is separate. No plan, requirement, source, Git or Beads changes were made.

Baseline: worktree `plan/phase-c` at `1e0145c6ac6973ac8cc936d5434220f764f24dc8`; all eight authoritative plan/contract files have no diff from critical-reviewed `a265f65ad7b047c0c7641ebdd0fbae42cb5aff4c`. Authority inspected: the complete direct human message preserved in `docs/plans/phase-c/reviews/internal-unapproved/consumer-extension-anchor-candidates.json`.

The existing design already has useful seams: public open `ScheduleStore` (`crates/sc-cron/src/store.rs:28`), generic `Scheduler::new(store, clock, jitter)` (`scheduler.rs:279`), injected `CronOperations`, `ApplicationTargetBindings` and `CronExecutor` (contract:297–360). Consequently, the gap is not a missing universal plug-in framework. It is insufficiently explicit replaceable defaults, shutdown-policy composition and documented application responsibilities.

## 1. Custom store/schema path is possible, but not an acceptance obligation

Exact passages: c-1:224 preserves “schema/generation/environment behavior”; c-5:95 preserves “row schema and transaction semantics.” Its external consumer proof (c-5:90–95,130–133) exercises targets/logging, not an application store with extra persisted fields. The contract:98 says “No new table,” and :155 says “No new schema/idempotency ledger.” These are sensible restrictions on this framework change, but do not expressly distinguish its built-in schema from a consumer-owned store/schema.

Minimal change: qualify those restrictions as preserving the bundled store/protocol, without forbidding application-owned columns/tables implemented through `ScheduleStore`. Add a supported custom-store example and a small existing c-5 external-consumer test that preserves required scheduling/fencing semantics while persisting an application field. Do not expand `Schedule`/wire DTOs with arbitrary product fields or add a portable query engine. Clarify whether a wrapper forwards cooperative scope and query or deliberately uses the legacy path; no mandatory cooperative certification of every custom store.

Ties: framework charter:19–24,39,51–53 assigns stores/schemas to consumers; sc-cron architecture:201–207 explicitly retains the open generic seam (REQ-CRON-0009). REQ-CRON-0020:241–242 already preserves legacy implementations and makes cooperation opt-in.

## 2. SQLite admission machinery is written more broadly than its supported default

Exact passages: contract:105 says “All worker/command operations”; :118 says “Every explicit or implicit write transaction installs the owned commit_hook.” C-5:68–84 mandates this SQLite machinery and rejects unsupported cooperative stores. The text never positively explains how a non-SQLite/custom store supplies equivalent semantics or chooses a different consumer-owned path.

Minimal change: scope SQLite hooks/TLS/interrupt proofs to `SqliteScheduleStore` under the built-in cooperative contract. Explain support declaration, scope forwarding and the promises a custom cooperative implementation accepts. Preserve the existing unsupported rejection for that mode. Document the independent legacy/custom integration alternative and its caller-owned guarantees; do not imply that a custom implementation must install SQLite hooks or silently satisfies the default guarantee.

Ties: REQ-CRON-0020, ADR-RUN-0018; the existing open-store contract and charter prohibit treating one database implementation as every consumer's architecture. Preserve truthful outcomes, fencing and default safe behavior.

## 3. Budget control exists; application shutdown-policy composition is unexplained

Exact passages: contract:51–66 exposes `ShutdownBudget`, `request_stop`, `cancel_now` and `join_owned`; :111–138 freezes the cooperative G/K policy. Contract:342–360 and c-6:70–72 always derive the standard executor's child context. REQ-CRNC-0002:31 says “never a separate timeout/ownership policy.” These describe the standard path, but provide no default-versus-override guide, nor acceptance proof of application-specific shutdown composition. Budget values alone should not be described as arbitrary behavior override.

Minimal change: c-1 names which choices applications make with existing controls, which guarantees belong to the standard cooperative path, and when an application supplies its own `CronOperations`/`CronExecutor` or uses the legacy core. C-5/c-6 usage docs show supported wiring and ownership/error responsibilities. Add a focused composition test only where needed to prove that seam. If an actual required override cannot use these public seams, identify that precise missing hook before adding an API. Retain default outcome honesty and owned release; consumer freedom is not a blanket weakening of standard guarantees.

## 4. Working selected defaults are planned, but the onboarding recipe is incomplete

Positive evidence: c-4:124–138 requires real template binding plus actual mutation/execution across CLI/HTTP/MCP and targets. This is substantial out-of-box proof once cron/commands are selected. `cron_commands=false` (contract:404; REQ-CRNC-0002:36–38) is intentional optional generation, not itself a defect.

Missing: c-6:87–88 only requires “standard crate usage docs”; c-1/c-5 do not require one easy guide showing the selected configuration, working default SQLite/worker/job wiring, startup/stop ownership, and replacement of store/schema or lifecycle choices. Minimal change: extend existing cron/adapter usage documentation and c-4 generated-consumer acceptance to follow that recipe without private APIs or manual framework edits. State selection requirements; keep optional features and disabled MCP intact. Do not create consumer product migration work.

These are concrete revision candidates under the newly explicit intent. The earlier critical PASS remains evidence for the earlier contract, not evidence that these new extension/documentation obligations have passed actual anchor evaluation.
