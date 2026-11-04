```json
{
  "agent": "arch-qa",
  "branch": "plan/phase-c",
  "scope": {"phase": "c", "sprint": "phase-c"},
  "commit": "45ffd3fee9ca83eb27d48cf861551baa61b4a204",
  "previous_reviewed_commit": "1217978e257ed8db9508a5422d4de889286e913e",
  "qa_pass": "qa-2",
  "review_mode": "round_limit",
  "round_limit": true,
  "verdict": "FAIL",
  "blocking": 1,
  "important": 0,
  "minor": 0,
  "findings": [
    {
      "id": "ARCH-C-002",
      "rule": "RULE-GATE",
      "adr": null,
      "severity": "BLOCKING",
      "file": "docs/plans/phase-c/anchors.json",
      "line": 1,
      "description": "The existing Plan Anchor readiness gate remains open. The current Phase C plan directory contains the anchor record and sprint plans but no completed live sprint-run or coverage artifacts. The parent reports calibrated anchor testability, user approval for clarity/split corrections, fixture/tuning checks, four full-sprint gates and locked-anchor coverage are still pending. Developer fixes and triage correctly retain this finding as open and do not claim a live gate PASS.",
      "remediation": "Finish the approved Plan Anchor workflow, including required clarity/split approval, accepted calibrated anchor testability, independent question/fixture checks, one passing live run for each whole c-1/c-2/c-3/c-4 sprint document, deterministic structural checks and complete locked-anchor coverage. Preserve actual evidence on the PR or beside the plan and request the bounded final evidence recheck."
    }
  ],
  "resolved_findings": [
    {
      "id": "ARCH-C-001",
      "status": "fixed",
      "evidence_refs": [
        "docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:35",
        "docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:17",
        "docs/plans/phase-c/phase-c-plan.md:28",
        ".triage/c/findings/qa-1-plan-readiness-pr114.ttl:8"
      ],
      "notes": "Exact assigned source delta adds all requested c-3 ADR-RUN-0001/0006/0007/0009/0011 entries, c-4 ADR-RUN-0017 and the full phase union. Existing error, async, host-safety and logging decisions remain binding. The author fixes JSON matches the actual changes."
    }
  ],
  "adr_checks": [
    {"adr":"ADR-RUN-0001","source":"docs/architecture.md:316","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:45"]},
    {"adr":"ADR-RUN-0006","source":"docs/architecture.md:855","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:46"]},
    {"adr":"ADR-RUN-0007","source":"docs/architecture.md:963","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:47"]},
    {"adr":"ADR-RUN-0009","source":"docs/architecture.md:1124","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:48"]},
    {"adr":"ADR-RUN-0011","source":"docs/architecture.md:2476","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:49"]},
    {"adr":"ADR-RUN-0017","source":"docs/architecture.md:213","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:19","docs/plans/phase-c/phase-c-plan.md:48"]}
  ],
  "gate_artifact_checks": [
    {
      "artifact": "docs/plans/phase-c/anchors.json",
      "status": "open",
      "evidence_refs": ["docs/plans/phase-c/anchors.json:1",".codex/skills/plan-anchoring/references/planning-gate.md:79",".triage/c/findings/qa-1-plan-readiness-pr114.ttl:9"],
      "notes": "No completed live full-sprint gate/coverage artifacts were supplied for this recheck; scratch calibration cannot substitute for the planning-gate exit criteria."
    },
    {
      "artifact": ".sc/plan-anchor/settings.yaml",
      "status": "closed",
      "evidence_refs": [".sc/plan-anchor/settings.yaml:3"],
      "notes": "Tracker bd, existing remote-judge consent and Phase C directory remain configured."
    }
  ],
  "merge_ready": false,
  "validation": [
    "Assigned branch and HEAD resolve exactly to plan/phase-c at 45ffd3fee9ca83eb27d48cf861551baa61b4a204.",
    "Read the actual prior-to-current source delta, author fixes JSON and canonical QA-1 triage record.",
    "Verified every requested governing reference directly in c-3, c-4 and phase frontmatter.",
    "Inspected current Phase C gate-artifact inventory and installed planning-gate contract.",
    "git diff --check passed."
  ],
  "notes": "Scoped QA-2 verification only, under the inherited Codex model. ARCH-C-001 is reviewer-verified fixed; ARCH-C-002 remains open. No new findings, repository edits, subagents, implementation-completion claims or inferred live gate verdicts. A final evidence recheck can close ARCH-C-002 when actual passing artifacts are supplied within the QA cycle cap."
}
```
