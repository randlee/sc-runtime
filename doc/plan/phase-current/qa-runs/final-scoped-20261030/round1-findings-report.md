# QA Findings Update

Generated: 2026-10-30T07:06:30.694653+00:00
QA Pass: final-scoped-1
Sprint/Task: phase-b b-13..b-18 (plan) / run-ogi
Branch: `plan/spinoff-monty-observability`
Commit: `67c9dc25aab0e4706db9637067b35d1a06690195`
PR: #85
Verdict: **FAIL**

## Machine Status (JSON)
```json
{
  "sprint": "phase-b b-13..b-18 (plan)",
  "task": "run-ogi",
  "branch": "plan/spinoff-monty-observability",
  "commit": "67c9dc25aab0e4706db9637067b35d1a06690195",
  "pr": 85,
  "verdict": "FAIL",
  "deliverables": {
    "complete": 86,
    "total": 92,
    "percent": 93.5
  },
  "findings": {
    "blocking": 7,
    "important": 3,
    "minor": 1
  },
  "blocking_ids": ["REQ-QA-FINAL-001", "REQ-QA-FINAL-002", "REQ-QA-FINAL-003", "REQ-QA-FINAL-004", "ARCH-001", "ARCH-002", "ARCH-003"],
  "merge_readiness": "not ready",
  "merge_reason": "7 blocking and 3 important findings require correction and scoped verification; no owner product decision is required.",
  "next_action": "Apply minimal contract/metadata fixes and generic logger-constructor correction; owners recheck only assigned findings.",
  "owner": "codexis",
  "reviewer_spawn_gate": "pass",
  "reviewer_manifest": [{"agent": "req-qa", "task_id": "/root/final_scoped_req_qa", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "FAIL", "reviewed_commit": "67c9dc25aab0e4706db9637067b35d1a06690195"}, {"agent": "arch-qa", "task_id": "/root/final_scoped_arch_qa", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "FAIL", "reviewed_commit": "67c9dc25aab0e4706db9637067b35d1a06690195"}, {"agent": "extensibility-qa", "task_id": "/root/final_scoped_extensibility_qa", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "FAIL", "reviewed_commit": "67c9dc25aab0e4706db9637067b35d1a06690195"}],
  "evidence_chain": {"reviewed_commit": "67c9dc25aab0e4706db9637067b35d1a06690195", "report_scope": "pre-implementation plan", "round_limit": 3, "ceremony_screen": "all11 covered; no ceremony remedy"},
  "missing_reviewers": [],
  "unparsed_reviewers": []}
```

## Findings Summary
- Deliverables: 86/92 (93.5%)
- Blocking: 7
- Important: 3
- Minor: 1

## Blocking Findings
- REQ-QA-FINAL-001: The final additions are not fully reflected in requirements frontmatter: b-13 omits REQ-CRON-0005 despite adding fallible public sc-cron signatures and an explicit CronError rule; b-16 omits REQ-RUN-0503 despite its new no-edits validation and REQ-RUN-0329 despite its generated package-name preservation obligation; b-17 omits REQ-RUN-0503 despite explicitly requiring that sequence.
- REQ-QA-FINAL-002: Canonical observability authority retains conflicting remnants: REQ-RT-0001 success criterion6 forbids every builder method except stores/routes/mcp/run while its own method table and NFR-RT-0001 permit logger; REQ-RT-0007 still says sc-runtime has no direct sc-observability-types dependency although REQ-RUN-0309 and the amended graph require it. The canonical main examples also call Observability::shared_log(), but b-18 fixes the generated getter as logger().
- REQ-QA-FINAL-003: b-13 requires the root Cargo.lock to prove the exact 1.5.1 observability pair immediately after forbidding either library from resolving the full logger. The maintainer workspace contains crates/*, not the generated daemon, and has no permitted full-logger dependency edge. Its lockfile cannot prove full sc-observability resolution without adding an unplanned or forbidden edge. The canonical requirement correctly locates pair resolution in the generated lockfile.
- REQ-QA-FINAL-004: b-15 declares the CODE: detail; recovery display form to be neither public nor parseable schema. REQ-CRON-0005 explicitly documents RunOutcome::failure_parts/RunFailureParts as the public parser of that form for agents and UIs. The plan cannot both produce that stable format and disclaim the supported parser contract.
- ARCH-001: b-16 changes dependency/import enforcement in all five production boundary manifests and production feature/dependency-tree gates, but its adrs list omits ADR-RUN-0003, which governs the library dependency graph and feature/dependency enforcement. REQ-RUN-0005 and ADR-RUN-0013 do not replace the governing-ADR listing requirement.
- ARCH-002: b-17 adds public library environment types while editing template configuration, daemon cron composition, and a generated example. Those deliverables are governed by ADR-RUN-0002 placement of reusable behavior in crates and application configuration/wiring in the template, but that ADR is absent from its adrs list even though the Goal now states the architecture explicitly.
- ARCH-003: The component architecture says every function call is denied during Monty source initialization. ADR-RUN-0016 decision 4 and b-17 deny registered host calls during initialization while permitting Getenv/GetEnviron; they do not prohibit ordinary VM-local function calls. The unqualified component rule is a broader incompatible execution contract and could even be read to prohibit the promised top-level os.getenv call.

## Detailed Findings
### REQ-QA-FINAL-001 — Blocking

The final additions are not fully reflected in requirements frontmatter: b-13 omits REQ-CRON-0005 despite adding fallible public sc-cron signatures and an explicit CronError rule; b-16 omits REQ-RUN-0503 despite its new no-edits validation and REQ-RUN-0329 despite its generated package-name preservation obligation; b-17 omits REQ-RUN-0503 despite explicitly requiring that sequence.

Correction: Add those already-governing IDs to the existing requirements lists (b-13: REQ-CRON-0005; b-16: REQ-RUN-0503 and REQ-RUN-0329; b-17: REQ-RUN-0503). This is metadata reconciliation only; no new scope, checker, or inventory is needed.

### REQ-QA-FINAL-002 — Blocking

Canonical observability authority retains conflicting remnants: REQ-RT-0001 success criterion6 forbids every builder method except stores/routes/mcp/run while its own method table and NFR-RT-0001 permit logger; REQ-RT-0007 still says sc-runtime has no direct sc-observability-types dependency although REQ-RUN-0309 and the amended graph require it. The canonical main examples also call Observability::shared_log(), but b-18 fixes the generated getter as logger().

Correction: Reconcile these existing canonical clauses/examples to the already approved injection contract: include logger in REQ-RT-0001 criterion6, state that StoresFailed remains string-based while types-only logging is permitted, and use the same generated Observability getter name everywhere. Make the b-13 authority-amendment scope/list explicitly cover REQ-RT-0007 if its text changes. Do not alter runtime error representation or the public interface to preserve stale prose.

### REQ-QA-FINAL-003 — Blocking

b-13 requires the root Cargo.lock to prove the exact 1.5.1 observability pair immediately after forbidding either library from resolving the full logger. The maintainer workspace contains crates/*, not the generated daemon, and has no permitted full-logger dependency edge. Its lockfile cannot prove full sc-observability resolution without adding an unplanned or forbidden edge. The canonical requirement correctly locates pair resolution in the generated lockfile.

Correction: State that the root lockfile proves sc-observability-types 1.5.1 only; the matching full-logger release is an external entry prerequisite, and b-16/b-18 generated-project lockfiles prove the pair. Keep full logger dependencies out of library tests/manifests and do not add a new workspace harness merely to populate the root lockfile.

### REQ-QA-FINAL-004 — Blocking

b-15 declares the CODE: detail; recovery display form to be neither public nor parseable schema. REQ-CRON-0005 explicitly documents RunOutcome::failure_parts/RunFailureParts as the public parser of that form for agents and UIs. The plan cannot both produce that stable format and disclaim the supported parser contract.

Correction: Preserve the documented failure_parts format/parser. Clarify instead that canonical LogEvent diagnostic codes are chosen from typed outcomes/origin sites and must never be inferred by parsing arbitrary consumer prose. No new parser or public error variant is needed.

### REQ-QA-FINAL-005 — Important

The new blanket sentence says every fallible public sc-cron function returns CronError, while the public application-implemented EnvironmentReferenceResolver::resolve protocol returns EnvironmentReferenceError. The finite callback result is appropriate for value-free provider failures, but the documentation does not distinguish that external provider protocol from fallible library entry points.

Correction: Clarify REQ-CRON-0005 and the b-13 blanket sentence: library-originating fallible entry points use CronError; the application resolver callback has the narrow Denied/Unavailable result protocol and is mapped to the fixed ENV_REFERENCE class. Retain the existing callback design and no-provider-prose rule; do not add another library error channel or opaque provider errors.

### REQ-QA-FINAL-006 — Important

The host error inventory says invalid registered-handler arguments yield HOST_DENIED, while b-16 maps every registered handler error to HOST_CALL_FAILED and exposes no generic argument-schema metadata. If an application handler rejects arguments by returning CronError::InvalidSpec, the two descriptions imply different codes.

Correction: Fix the inventory boundary explicitly: HOST_DENIED covers bridge conversion/supplied CLI adapter shape rejection; custom-handler returned errors, including InvalidSpec, map to HOST_CALL_FAILED, or document one consistent alternative. Reuse the existing handler-error acceptance case to make the selected mapping explicit rather than adding schema machinery.

### ARCH-001 — BLOCKING

b-16 changes dependency/import enforcement in all five production boundary manifests and production feature/dependency-tree gates, but its adrs list omits ADR-RUN-0003, which governs the library dependency graph and feature/dependency enforcement. REQ-RUN-0005 and ADR-RUN-0013 do not replace the governing-ADR listing requirement.

Correction: Add ADR-RUN-0003 to b-16 adrs and cite it on the existing production tool-crate boundary deliverable/acceptance. Preserve the approved prohibition and existing enforcement plan; no new artifact or gate is needed.

### ARCH-002 — BLOCKING

b-17 adds public library environment types while editing template configuration, daemon cron composition, and a generated example. Those deliverables are governed by ADR-RUN-0002 placement of reusable behavior in crates and application configuration/wiring in the template, but that ADR is absent from its adrs list even though the Goal now states the architecture explicitly.

Correction: Add ADR-RUN-0002 to b-17 adrs and cite it on the existing generated policy/configuration/example deliverable. Keep the existing library/template split; do not introduce a new acceptance artifact.

### ARCH-003 — BLOCKING

The component architecture says every function call is denied during Monty source initialization. ADR-RUN-0016 decision 4 and b-17 deny registered host calls during initialization while permitting Getenv/GetEnviron; they do not prohibit ordinary VM-local function calls. The unqualified component rule is a broader incompatible execution contract and could even be read to prohibit the promised top-level os.getenv call.

Correction: Replace every function call with every registered host call (including sc_compose_run and atm_run), explicitly leaving ordinary Monty-local computation and configured environment reads intact. This aligns the component text to the existing ADR and sprint; no API or policy expansion is required.

### ARCH-004 — MINOR

The acceptance text calls the fixture-based shutdown proof a real signal regression, while ADR-RT-0004 requires the fixture to stop through a crate-private trigger rather than a process signal. The deliverable already selects the existing fixture, so the required behavior is recoverable, but this phrase unnecessarily blurs b-14 with b-18 real-signal CI testing.

Correction: Name the fixture-private shutdown trigger in the existing deliverable/acceptance and reserve real SIGINT/SIGTERM language for b-18. No new test or safety gate is needed.

### EXT-001 — important

Logger injection is unavailable to the scheduler’s existing custom-store/custom-jitter consumers.

Correction: Add a generic Scheduler::new_with_logger(store: Arc<S>, clock: Arc<C>, jitter: Arc<J>, logger: SharedCanonicalLog) constructor so current custom-store/jitter consumers can inject scheduler/worker observability without replacing their providers or editing sc-cron. Keep open_with_logger as the coherent SQLite/SystemJitter convenience path, creating its own logger-aware concrete components and delegating generic scheduler construction. Legacy new delegates with Noop. Document that caller-supplied store/jitter implementations own their internal logging; the generic constructor installs the supplied handle in scheduler/RunCoord and its worker/hook paths. Replace the sole-constructor prohibition in the sprint and injection contract in lockstep, and add one existing contract-test case preserving custom store/jitter identity while asserting scheduler/worker emission through the injected handle. No new trait, wrapper, backend, or post-construction setter is necessary.

## Resolved Since Last Pass
Ceremony screen covered all11 findings; all remedies are product-contract or existing-metadata corrections, with no new process artifact.

## Merge Readiness
- Status: **not ready**
- Reason: 7 blocking and 3 important findings require correction and scoped verification; no owner product decision is required.

## Next Action
- Action: Apply minimal contract/metadata fixes and generic logger-constructor correction; owners recheck only assigned findings.
- Owner: codexis