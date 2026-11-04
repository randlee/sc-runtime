```json
{
  "success": true,
  "data": {
    "status": "findings",
    "reviewed_commit": "1217978e257ed8db9508a5422d4de889286e913e",
    "reviewed_branch": "plan/phase-c",
    "review_mode": "doc_review",
    "findings": [
      {
        "id": "CQA-C-001",
        "severity": "important",
        "class": "unjustified_artifact",
        "file": "docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md",
        "line": 28,
        "issue": "c-2 requires a new dedicated consumer-lint workflow in addition to the focused generation regression and the existing full template fixture matrix, without explaining why that existing validation venue cannot execute the required analyzer proof or when the extra workflow is retired.",
        "recommendation": "Remove the dedicated workflow path and dedicated-workflow acceptance requirement; retain the focused generated-consumer regression and its real analyzer invocation assertions in the already required template fixture/test validation. Reuse existing CI logs for the receipt.",
        "evidence": "The new workflow is owned at line28, required as a dedicated CI receipt at98-102, and required again at127-128; the same sprint already requires the focused profile regression at132-133, the actual generated-consumer command sequence at134-136, and the full existing template fixture matrix plus root tests at137. The observed defect justifies executing all three analyzers, but the plan does not establish a distinct capability that requires an additional workflow or a retirement condition for it."
      }
    ],
    "summary": {
      "important": 1,
      "minor": 0
    },
    "review_targets_read": [
      "docs/plans/phase-c/phase-c-plan.md",
      "docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md",
      "docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md",
      "docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md",
      "docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md",
      "docs/plans/phase-c/anchors.json",
      "docs/plans/phase-c/anchors.md",
      ".claude/project/quality-policy.md"
    ],
    "notes": [
      "Exact assigned commit verified before review; all plan targets read in full.",
      "The temporary c-1 probe has an explicit consumer, capability, observed adoption defects and deletion before merge; no removal recommended.",
      "Package identity and registry-source assertions address the named split-type and local-patch adoption defects; no redundant-enforcement finding raised.",
      "Existing bootstrap invocation lines are useful proof that configured analyzers actually ran; the finding concerns the extra workflow, not removal of that execution proof.",
      "The three dependency edges consume concrete contract or package artifacts and have explicit parent-push merge-forward execution; no unjustified serialization found.",
      "No implementation, live Plan Anchor, or release completion is claimed. Review executed with inherited Codex model; no Cursor or Sonnet execution claim."
    ]
  },
  "error": null
}
```
