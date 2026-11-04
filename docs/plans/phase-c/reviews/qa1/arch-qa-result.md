```json
{
  "agent": "arch-qa",
  "branch": "plan/phase-c",
  "scope": {"phase": "c", "sprint": "phase-c"},
  "commit": "1217978e257ed8db9508a5422d4de889286e913e",
  "verdict": "FAIL",
  "blocking": 2,
  "important": 0,
  "findings": [
    {
      "id": "ARCH-C-001",
      "rule": "RULE-ADR-PLAN",
      "adr": "ADR-RUN-0001, ADR-RUN-0006, ADR-RUN-0007, ADR-RUN-0009, ADR-RUN-0011, ADR-RUN-0017",
      "severity": "BLOCKING",
      "file": "docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md",
      "line": 30,
      "description": "The c-3 governing ADR list omits active decisions that directly govern its planned source/API and process-test changes: ADR-RUN-0001 (ordinary project composition/no wrappers, explicitly preserved in the upgrade contract), ADR-RUN-0006 (the affected public IntoMcp/McpError signature exception), ADR-RUN-0007 (request/streamable HTTP async execution), ADR-RUN-0009 (the explicitly required isolated shutdown/process fixture), and ADR-RUN-0011 (locks/await and blocking-call constraints on the owned runtime/template source). The c-4 list also omits ADR-RUN-0017 despite publishing the affected command/runtime/cron logging-identity closure under that amendment. The phase list should include the union of those governing references as well.",
      "remediation": "Add ADR-RUN-0001/0006/0007/0009/0011 to c-3's ADR frontmatter, ADR-RUN-0017 to c-4, and their missing union to the phase list. Keep the existing behavior and release gates; no new process artifact is needed. The implementation handoff must retain the actual host-safety and async/error constraints from those existing decisions."
    },
    {
      "id": "ARCH-C-002",
      "rule": "RULE-GATE",
      "adr": null,
      "severity": "BLOCKING",
      "file": "docs/plans/phase-c/anchors.json",
      "line": 1,
      "description": "The eight approved anchors are locked and have fixtures and tracker links, but current Phase C artifacts do not yet contain the completed live fixture/tuning evidence, latest passing complete-sprint planning runs, and locked-anchor coverage required by Plan Anchor's own planning-gate contract. Calibration activity in scratch space is not the completed sprint gate. The parent correctly reports this work as pending; this finding prevents final plan-readiness closeout until the gate finishes.",
      "remediation": "Complete independent question calibration/tuning and approved fixture checks, run one live planning gate per entire c-1/c-2/c-3/c-4 document, fix retained defects, record deterministic checks for structural ANC-004/005, and preserve final passing run reports plus coverage with no uncovered locked anchor on the PR or beside the plan. Recheck the resulting evidence without substituting JSON shape checks or dry runs."
    }
  ],
  "adr_checks": [
    {"adr":"ADR-RUN-0002","source":"docs/architecture.md","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/phase-c-plan.md:76","docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:58"]},
    {"adr":"ADR-RUN-0003","source":"docs/architecture.md","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:105","docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:97"]},
    {"adr":"ADR-RUN-0005","source":"docs/architecture.md:773","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:53","docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md:55"],"note":"c-1 explicitly owns the authoritative amendment before the generated-consumer exception is implemented; the planning review does not treat future implementation as already authorized by an unamended ADR."},
    {"adr":"ADR-RUN-0001","source":"docs/architecture.md:316","listed_in_sprint_doc":false,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:100"],"note":"Behavior is preserved; missing governing-list coverage is ARCH-C-001."},
    {"adr":"ADR-RUN-0006","source":"docs/architecture.md:855","listed_in_sprint_doc":false,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:128","docs/sc-command/architecture.md:298"],"note":"No opaque error or wrapper is introduced by the plan; the changed public RMCP exception must be in its governing list."},
    {"adr":"ADR-RUN-0007","source":"docs/architecture.md:963","listed_in_sprint_doc":false,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:108"],"note":"Existing runtime/streamable HTTP behavior is retained, but the governing list is incomplete."},
    {"adr":"ADR-RUN-0009","source":"docs/architecture.md:1124","listed_in_sprint_doc":false,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:178"],"note":"Plan explicitly requires isolation; missing reference is not a request for new isolation machinery."},
    {"adr":"ADR-RUN-0011","source":"docs/architecture.md:2476","listed_in_sprint_doc":false,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:53","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:108"],"note":"No changed Rust implementation exists in this docs-only PR; await/guard lifetime analysis is an implementation obligation."},
    {"adr":"ADR-RUN-0012","source":"docs/architecture.md:2512","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:115","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:151"]},
    {"adr":"ADR-RUN-0017","source":"docs/architecture.md:213","listed_in_sprint_doc":false,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:90","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:128","docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:60"],"note":"Listed in phase/c-1/c-3; omitted in c-4, reported in ARCH-C-001. Existing release preflight remains binding."},
    {"adr":"ADR-RUN-0202","source":"docs/architecture.md:1393","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:80","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:100"]},
    {"adr":"ADR-RUN-0203","source":"docs/architecture.md:1483","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:87","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:139","docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:76"],"note":"c-1 probes and amends exact versions before c-3 consumes them; no unsupported compatibility or selective-publisher capability is presumed."},
    {"adr":"ADR-RUN-0301/0302/0303","source":"docs/architecture.md","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:57","docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md:89","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:115"]},
    {"adr":"ADR-RUN-0401/0402","source":"docs/architecture.md","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md:117","docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:104"]},
    {"adr":"ADR-CMD-0001/0002/0003","source":"docs/sc-command/architecture.md","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:128","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:175"]},
    {"adr":"ADR-RT-0001/0002/0003/0004/0005","source":"docs/sc-runtime/architecture.md","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:100","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:178"]}
  ],
  "gate_artifact_checks": [
    {"artifact":"docs/plans/phase-c/anchors.json","status":"open","evidence_refs":["docs/plans/phase-c/anchors.json:1",".codex/skills/plan-anchoring/references/planning-gate.md:79"],"notes":"Eight locked anchors and approved fixtures exist. Live calibrated fixture tests, four complete-sprint passing gates and final coverage remain pending; ARCH-C-002."},
    {"artifact":".sc/plan-anchor/settings.yaml","status":"closed","evidence_refs":[".sc/plan-anchor/settings.yaml:3"],"notes":"Tracker bd, user-authorized remote judge consent true, and correct Phase C plan directory are explicit."},
    {"artifact":"Phase C future c-1/c-3/c-4 implementation and publication artifacts","status":"not-applicable","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:47","docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:58"],"notes":"These are future sprint deliverables, not claimed current plan-review completion. Selective-kit capability is explicitly probe-gated; no package publication is inferred."}
  ],
  "merge_ready": false,
  "notes": "Independent docs-only architecture role executed under the inherited Codex model; no Grok or Cursor-model execution claim. Assigned branch/HEAD verified exactly before review. No source implementation, CI execution, requirement conformance, or live Plan Anchor PASS claimed. All retained findings are in this single pass; no repository edits or subagents."
}
```
