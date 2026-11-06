Published report: [PR #122, QA 3](https://github.com/randlee/sc-runtime/pull/122#issuecomment-6678094749).

Reviewed feature-branch source: `e190e2c8d84b9235ac70b3e442691980e8b69a33`. The published report below is retained unchanged.

# QA Findings Update

Generated: 2026-11-06T05:56:21.243653+00:00
QA Pass: QA3
Sprint/Task: c-4 / qa-pr122-sprint-3
Branch: `sprint/c-4-runtime-ecosystem-release`
Commit: `e190e2c8d84b9235ac70b3e442691980e8b69a33`
PR: #122
Verdict: **FAIL**

## Machine Status (JSON)
```json
{
  "sprint": "c-4",
  "task": "qa-pr122-sprint-3",
  "branch": "sprint/c-4-runtime-ecosystem-release",
  "commit": "e190e2c8d84b9235ac70b3e442691980e8b69a33",
  "pr": 122,
  "verdict": "FAIL",
  "deliverables": {
    "complete": 18,
    "total": 26,
    "percent": 69.23
  },
  "findings": {
    "blocking": 7,
    "important": 0,
    "minor": 0
  },
  "blocking_ids": [
    "REQ-QA-C4-001",
    "REQ-QA-C4-002",
    "REQ-QA-C4-004",
    "REQ-QA-C4-005",
    "REQ-QA-C4-006",
    "ARCH-003",
    "RBQA-C4-001"
  ],
  "merge_readiness": "not ready",
  "merge_reason": "Seven Blocking findings remain across the unanswered tempfile-policy conflict, final composed/parent/benchmark qualification, and actual authorized release/registry proof. Source fixes are credited; full sprint acceptance is not proven. Current exact-head CI: 16 successful, 1 pending, 0 failed; no waiting.",
  "next_action": "triage_and_fix",
  "owner": "parent-orchestrator",
  "reviewer_spawn_gate": "pass",
  "reviewer_manifest": [
    {
      "agent": "req-qa",
      "task_id": "/root/c4_req_qa3",
      "spawn_actor": "parent-orchestrator",
      "fenced_json_received": true,
      "verdict": "FAIL",
      "findings": {
        "blocking": 5,
        "important": 0,
        "minor": 0
      }
    },
    {
      "agent": "arch-qa",
      "task_id": "/root/c4_arch_qa3",
      "spawn_actor": "parent-orchestrator",
      "fenced_json_received": true,
      "verdict": "FAIL",
      "findings": {
        "blocking": 1,
        "important": 0,
        "minor": 0
      }
    },
    {
      "agent": "ruthless-boundary-qa",
      "task_id": "/root/c4_boundary_qa3",
      "spawn_actor": "parent-orchestrator",
      "fenced_json_received": true,
      "verdict": "FAIL",
      "findings": {
        "blocking": 1,
        "important": 0,
        "minor": 0
      }
    },
    {
      "agent": "rust-qa-agent",
      "task_id": "/root/c4_rust_qa3",
      "spawn_actor": "parent-orchestrator",
      "fenced_json_received": true,
      "verdict": "PASS",
      "findings": {
        "blocking": 0,
        "important": 0,
        "minor": 0
      }
    },
    {
      "agent": "rust-best-practices-agent",
      "task_id": "/root/c4_rbp_qa3",
      "spawn_actor": "parent-orchestrator",
      "fenced_json_received": true,
      "verdict": "PASS",
      "findings": {
        "blocking": 0,
        "important": 0,
        "minor": 0
      }
    },
    {
      "agent": "flaky-test-qa",
      "task_id": "/root/c4_flaky_qa3",
      "spawn_actor": "parent-orchestrator",
      "fenced_json_received": true,
      "verdict": "PASS",
      "findings": {
        "blocking": 0,
        "important": 0,
        "minor": 0
      }
    }
  ],
  "evidence_chain": {
    "qa_pass": "qa-3",
    "commit": "e190e2c8d84b9235ac70b3e442691980e8b69a33",
    "coordinator_task_id": "/root/c4_manager_qa3",
    "pr_comment_url": "https://github.com/randlee/sc-runtime/pull/122#issuecomment-6678094749",
    "triage": {
      "ttl_paths": [
        ".triage/c/findings/c4-qa1.ttl"
      ]
    },
    "finding_occurrences": 7,
    "unique_finding_ids": 7,
    "todo_scan": {
      "rust_source_hits": 0
    },
    "central_screen": {
      "keep": 5,
      "not_applicable": 2,
      "ceremony": 0,
      "genuine_findings_dismissed": 0
    },
    "ci_snapshot": {
      "head": "e190e2c8d84b9235ac70b3e442691980e8b69a33",
      "success": 16,
      "pending": 1,
      "failed": 0,
      "pending_names": [
        "just lint and just test"
      ],
      "pending_job_url": "https://github.com/randlee/sc-runtime/actions/runs/42170117713/job/127736085505",
      "source": "Actual one-shot gh pr view 122 --json headRefOid,statusCheckRollup,mergeStateStatus,url during this coordinator turn",
      "waited_for_ci": false
    },
    "model_execution": "Inherited Codex fallback; configured Cursor models unavailable.",
    "release_acceptance": "Open; no tempfile or staging exception adopted; benchmark measurements not yet supplied."
  },
  "missing_reviewers": [],
  "unparsed_reviewers": [],
  "aggregation_source": "reviewer_fenced_json_union_todo_scan"
}
```

## Findings Summary
- Deliverables: 18/26 (69.23%)
- Blocking: 7
- Important: 0
- Minor: 0

## Blocking Findings
- **REQ-QA-C4-001 — Blocking:** Registry logging-facade proof remains absent.
- **REQ-QA-C4-002 — Blocking:** The daemon normal dependency tree reaches sc-cron → tempfile while the existing guard forbids that dependency.
- **REQ-QA-C4-004 — Blocking:** Completed parent qualification and actual authorized all-six publication, positive visibility and v0.2.0 main/tag/source provenance remain unproven.
- **REQ-QA-C4-005 — Blocking:** The unchanged no-patch registry consumer, UI/handler/schema/lifecycle/capacity/customization and public-facade acceptance runs remain unexecuted.
- **REQ-QA-C4-006 — Blocking:** Final composed-head lint/test, completed parent QA/CI and controlled-source benchmark qualification remain open.
- **ARCH-003 — Blocking:** C4 structural release and registry acceptance is incomplete; corrected publication-kit source does not establish actual release qualification.
- **RBQA-C4-001 — Blocking:** Selected cron violates the still-binding daemon tempfile exclusion; the untouched consumer guard remains red.

## Detailed Findings
### REQ-QA-C4-001 — Blocking

- Venue: `crates/sc-cron/tests/fixtures/logging_consumer/Cargo.toml:11`
- Finding: Registry logging-facade proof remains absent.
- Required correction: After authorized publication, run the exact =0.2.0 isolated public facade consumer without public path/git/patch dependencies; retain locked registry identities and actual results.
- Reviewer: `req-qa`.

### REQ-QA-C4-002 — Blocking

- Venue: `template/Justfile; docs/requirements.md:3355`
- Finding: The daemon normal dependency tree reaches sc-cron → tempfile while the existing guard forbids that dependency.
- Required correction: Resolve the requirement conflict through the pending owner decision and verify only its authorized scope; keep the current guard binding meanwhile.
- Reviewer: `req-qa`.

### REQ-QA-C4-004 — Blocking

- Venue: `docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:131`
- Finding: Completed parent qualification and actual authorized all-six publication, positive visibility and v0.2.0 main/tag/source provenance remain unproven.
- Required correction: Complete existing prerequisites and obtain publication authority before the normal all-six publication; retain actual ordering, visibility and immutable provenance.
- Reviewer: `req-qa`.

### REQ-QA-C4-005 — Blocking

- Venue: `tests/integration/release/test_published_runtime_ecosystem.py:9`
- Finding: The unchanged no-patch registry consumer, UI/handler/schema/lifecycle/capacity/customization and public-facade acceptance runs remain unexecuted.
- Required correction: After authorized publication and visibility, execute existing registry-only fixtures and record all six exact package identities and real behavior results.
- Reviewer: `req-qa`.

### REQ-QA-C4-006 — Blocking

- Venue: `docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:271`
- Finding: Final composed-head lint/test, completed parent QA/CI and controlled-source benchmark qualification remain open.
- Required correction: Retain real final composed-head gates and governed controlled-source measurements/review. Focused receipts do not establish a full-suite or current all-platform pass.
- Reviewer: `req-qa`.

### ARCH-003 — Blocking

- Venue: `docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:137`
- Finding: C4 structural release and registry acceptance is incomplete; corrected publication-kit source does not establish actual release qualification.
- Required correction: Finish source/parent and benchmark qualification, resolve the policy conflict, then perform separately authorized publication and unchanged registry-only proofs.
- Reviewer: `arch-qa`.

### RBQA-C4-001 — Blocking

- Venue: `template/scripts/consumer_checks.py:152`
- Finding: Selected cron violates the still-binding daemon tempfile exclusion; the untouched consumer guard remains red.
- Required correction: Preserve the guard until the authorized requirement decision; do not globally allowlist tempfile or remove Python private scratch isolation as a shortcut.
- Reviewer: `ruthless-boundary-qa`.

## Resolved Since Last Pass
- **REQ-QA-C4-003 / RBQA-C4-002:** canonical reviewed sc-publish source is merged and adopted at `1a494e7adb067d93c57807f443ac136a8775c7d8`. All six caller and rendered crate entries require positive exact-version visibility after upload and before dependents, in main and retry workflows; absent/indeterminate visibility stops. Canonical 89-destination inventory, copied bytes and repeat dry-run were independently verified. Generic omitted/false behavior remains compatible.
- **Actual CI source fixes:** optional Monty forwarding/default-off assertions, minimal Liquid MCP registration, native `INVALID_SPEC` startup expectations, bounded captured diagnostics and genuine-secondary-cleanup classification verified. Rust, Rust-practices and flaky-test reviewers report no new findings.
- **Scoped evidence retained:** publisher suite 368 passes/15 existing skips; visibility/preflight/staging subset 21 passes; affected Python contracts 8 passes; real mixed-UI 8 Rust plus 10 schema/ownership passes; generated startup/precedence/redaction and strict Monty Clippy pass. These remain scoped executions, not a final full-suite or registry run.
- **Inventory accounting:** req-qa reports 18/26 (69.23%). Its inventory now explicitly includes the existing controlled-source benchmark obligation and this actual CI follow-up; no previous item was removed. No benchmark measurement has been credited. Six reviewers supplied exact-commit evidence; the TODO scan found no Rust TODOs. Model execution used the inherited Codex fallback.

## Merge Readiness
- Status: **not ready**
- Reason: Seven Blocking findings remain across the unanswered tempfile-policy conflict, final composed/parent/benchmark qualification, and actual authorized release/registry proof. Source fixes are credited; full sprint acceptance is not proven. Current exact-head CI: 16 successful, 1 pending, 0 failed; no waiting.

## Next Action
- Action: triage_and_fix
- Owner: parent-orchestrator