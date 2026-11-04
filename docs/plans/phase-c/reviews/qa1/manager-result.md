# QA Findings Update

Generated: 2026-11-03T03:19:27.601303+00:00
QA Pass: qa-1
Sprint/Task: phase-c / phase-c-plan-qa1
Branch: `plan/phase-c`
Commit: `1217978e257ed8db9508a5422d4de889286e913e`
PR: #114
Verdict: **FAIL**

## Machine Status (JSON)
```json
{
  "sprint": "phase-c",
  "task": "phase-c-plan-qa1",
  "branch": "plan/phase-c",
  "commit": "1217978e257ed8db9508a5422d4de889286e913e",
  "pr": 114,
  "verdict": "FAIL",
  "deliverables": {
    "complete": 5,
    "total": 5,
    "percent": 100.0
  },
  "findings": {
    "blocking": 5,
    "important": 2,
    "minor": 0
  },
  "blocking_ids": ["REQ-QA-001", "REQ-QA-002", "REQ-QA-003", "ARCH-C-001", "ARCH-C-002", "RBQA-C-001", "CQA-C-001"],
  "merge_readiness": "not ready",
  "merge_reason": "Seven reviewer outputs are complete; five Blocking and two Important findings remain, including unfinished live Plan Anchor verification.",
  "next_action": "triage_and_fix",
  "owner": "parent-orchestrator",
  "reviewer_spawn_gate": "pass",
  "reviewer_manifest": [{"agent": "req-qa", "task_id": "/root/phase_c_qa_req", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "FAIL", "findings": {"blocking": 3, "important": 0, "minor": 0}}, {"agent": "arch-qa", "task_id": "/root/phase_c_qa_arch", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "FAIL", "findings": {"blocking": 2, "important": 0, "minor": 0}}, {"agent": "extensibility-qa", "task_id": "/root/phase_c_qa_extensibility", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "PASS", "findings": {"blocking": 0, "important": 0, "minor": 0}}, {"agent": "ruthless-boundary-qa", "task_id": "/root/phase_c_qa_boundary", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "findings", "findings": {"blocking": 0, "important": 1, "minor": 0}}, {"agent": "rust-best-practices-agent", "task_id": "/root/phase_c_qa_practices", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "pass", "findings": {"blocking": 0, "important": 0, "minor": 0}}, {"agent": "rust-service-hardening-agent", "task_id": "/root/phase_c_qa_service", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "pass", "findings": {"blocking": 0, "important": 0, "minor": 0}}, {"agent": "ceremony-qa", "task_id": "/root/phase_c_qa_ceremony", "spawn_actor": "parent-orchestrator", "fenced_json_received": true, "verdict": "findings", "findings": {"blocking": 0, "important": 1, "minor": 0}}],
  "evidence_chain": {"qa_pass": "qa-1", "commit": "1217978e257ed8db9508a5422d4de889286e913e", "pr_comment_url": "https://github.com/randlee/sc-runtime/pull/114#issuecomment-6594574410", "coordinator_task_id": "/root/phase_c_quality_manager", "triage": {"ttl_paths": [], "screen_path": "/tmp/sc-runtime-phase-c-review/qa1/screen.json"}, "todo_scan": {"status": "not_applicable", "reason": "Plan review only; no implementation sprint-end or integration TODO scan is required."}},
  "missing_reviewers": [],
  "unparsed_reviewers": []}
```

## Findings Summary
- Deliverables: 5/5 (100.0%)
- Blocking: 5
- Important: 2
- Minor: 0

## Blocking Findings
- REQ-QA-001: REQ-CMD-0005 explicitly defines CallToolResult as the rmcp 3.7 type. Phase C moves that public contract to the newly frozen RMCP line, but c-1 only owns repository and cron requirement files and does not amend this command requirement. c-3 consequently promises both changed RMCP types and preservation of an unchanged 3.7 requirement.
- REQ-QA-002: The active root-lint clause of REQ-RUN-0004 and green-with-no-edits definition of REQ-RUN-0503 still require sc-lint 0.8.0. c-1's enumerated requirement amendment updates 0307 and NFR0006 only, so its promised consistent 0.8.4 contract can leave binding contradictory version statements behind.
- REQ-QA-003: The governing requirements lists omit directly exercised constraints: c-1/c-2 derive consumer crate/edge policy without REQ-RUN-0301; c-3/c-4 assert path-patched versus registry dependencies without REQ-RUN-0003; c-3 owns CLI dependency changes, graceful-shutdown regression, and SQLite manifest/resolver guard without NFR-RUN-0001, REQ-RT-0005, REQ-DBSQ-0001 or REQ-DBSQ-0005. The phase list also omits these governing IDs.
- ARCH-C-001: The c-3 governing ADR list omits active decisions that directly govern its planned source/API and process-test changes: ADR-RUN-0001 (ordinary project composition/no wrappers, explicitly preserved in the upgrade contract), ADR-RUN-0006 (the affected public IntoMcp/McpError signature exception), ADR-RUN-0007 (request/streamable HTTP async execution), ADR-RUN-0009 (the explicitly required isolated shutdown/process fixture), and ADR-RUN-0011 (locks/await and blocking-call constraints on the owned runtime/template source). The c-4 list also omits ADR-RUN-0017 despite publishing the affected command/runtime/cron logging-identity closure under that amendment. The phase list should include the union of those governing references as well.
- ARCH-C-002: The eight approved anchors are locked and have fixtures and tracker links, but current Phase C artifacts do not yet contain the completed live fixture/tuning evidence, latest passing complete-sprint planning runs, and locked-anchor coverage required by Plan Anchor's own planning-gate contract. Calibration activity in scratch space is not the completed sprint gate. The parent correctly reports this work as pending; this finding prevents final plan-readiness closeout until the gate finishes.

## Detailed Findings
### REQ-QA-001 — Blocking — req-qa

REQ-CMD-0005 explicitly defines CallToolResult as the rmcp 3.7 type. Phase C moves that public contract to the newly frozen RMCP line, but c-1 only owns repository and cron requirement files and does not amend this command requirement. c-3 consequently promises both changed RMCP types and preservation of an unchanged 3.7 requirement.

Required correction: Give c-1 ownership of docs/sc-command/requirements.md and explicitly amend only the REQ-CMD-0005 version identity to the frozen new RMCP matrix while preserving conversion/error semantics. c-3 must consume that amended contract. No new process artifact or implementation behavior is needed.

### REQ-QA-002 — Blocking — req-qa

The active root-lint clause of REQ-RUN-0004 and green-with-no-edits definition of REQ-RUN-0503 still require sc-lint 0.8.0. c-1's enumerated requirement amendment updates 0307 and NFR0006 only, so its promised consistent 0.8.4 contract can leave binding contradictory version statements behind.

Required correction: Explicitly include the current operational sc-lint-version clauses of REQ-RUN-0004 and REQ-RUN-0503 in c-1's amendment and acceptance check; add 0503 to c-1's requirement list. Preserve historical evidence/version observations rather than rewriting history. Existing documentation validation is sufficient.

### REQ-QA-003 — Blocking — req-qa

The governing requirements lists omit directly exercised constraints: c-1/c-2 derive consumer crate/edge policy without REQ-RUN-0301; c-3/c-4 assert path-patched versus registry dependencies without REQ-RUN-0003; c-3 owns CLI dependency changes, graceful-shutdown regression, and SQLite manifest/resolver guard without NFR-RUN-0001, REQ-RT-0005, REQ-DBSQ-0001 or REQ-DBSQ-0005. The phase list also omits these governing IDs.

Required correction: Add these existing governing IDs to the respective sprint lists and phase union, and explicitly pair c-3's lifecycle/CLI/SQLite preservation checks with their existing requirements. For c-1's selective per-crate version contract also include REQ-RUN-0002. Audit the edited source/API and dependency boundaries for any other directly applicable requirement IDs using the same rule; no new requirement or behavior is requested.

### ARCH-C-001 — Blocking — arch-qa

The c-3 governing ADR list omits active decisions that directly govern its planned source/API and process-test changes: ADR-RUN-0001 (ordinary project composition/no wrappers, explicitly preserved in the upgrade contract), ADR-RUN-0006 (the affected public IntoMcp/McpError signature exception), ADR-RUN-0007 (request/streamable HTTP async execution), ADR-RUN-0009 (the explicitly required isolated shutdown/process fixture), and ADR-RUN-0011 (locks/await and blocking-call constraints on the owned runtime/template source). The c-4 list also omits ADR-RUN-0017 despite publishing the affected command/runtime/cron logging-identity closure under that amendment. The phase list should include the union of those governing references as well.

Required correction: Add ADR-RUN-0001/0006/0007/0009/0011 to c-3's ADR frontmatter, ADR-RUN-0017 to c-4, and their missing union to the phase list. Keep the existing behavior and release gates; no new process artifact is needed. The implementation handoff must retain the actual host-safety and async/error constraints from those existing decisions.

### ARCH-C-002 — Blocking — arch-qa

The eight approved anchors are locked and have fixtures and tracker links, but current Phase C artifacts do not yet contain the completed live fixture/tuning evidence, latest passing complete-sprint planning runs, and locked-anchor coverage required by Plan Anchor's own planning-gate contract. Calibration activity in scratch space is not the completed sprint gate. The parent correctly reports this work as pending; this finding prevents final plan-readiness closeout until the gate finishes.

Required correction: Complete independent question calibration/tuning and approved fixture checks, run one live planning gate per entire c-1/c-2/c-3/c-4 document, fix retained defects, record deterministic checks for structural ANC-004/005, and preserve final passing run reports plus coverage with no uncovered locked anchor on the PR or beside the plan. Recheck the resulting evidence without substituting JSON shape checks or dry runs.

### RBQA-C-001 — Important — ruthless-boundary-qa

The affected command/cron version migration has no owner for the existing mandatory observability boundary guard, which hard-codes the old command dependency version and rejects the planned upgrade.

Required correction: Assign scripts/check_observability_dependencies.py to c-3 and explicitly migrate its command-version alignment checks to the c-1 frozen affected-package contract. Retain the exact defaults-off edge, no server/test-util activation, no alias/target/build bypass, and isolated default/Monty dependency checks. Cover new valid and stale dependency versions through the existing c-3 regression venue. Do not disable or weaken the guard.

### CQA-C-001 — Important — ceremony-qa

c-2 requires a new dedicated consumer-lint workflow in addition to the focused generation regression and the existing full template fixture matrix, without explaining why that existing validation venue cannot execute the required analyzer proof or when the extra workflow is retired.

Required correction: Remove the dedicated workflow path and dedicated-workflow acceptance requirement; retain the focused generated-consumer regression and its real analyzer invocation assertions in the already required template fixture/test validation. Reuse existing CI logs for the receipt.

All seven findings were retained by the independent ceremony screen. Reviewer roles ran under the inherited Codex model; no Cursor model execution is claimed. Planning artifact presence is 5/5; this is not implementation or live anchor completion.

## Resolved Since Last Pass
- none

## Merge Readiness
- Status: **not ready**
- Reason: Seven reviewer outputs are complete; five Blocking and two Important findings remain, including unfinished live Plan Anchor verification.

## Next Action
- Action: triage_and_fix
- Owner: parent-orchestrator