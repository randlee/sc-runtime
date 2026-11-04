```json
{
  "status": "FAIL",
  "reviewed_branch": "plan/phase-c",
  "reviewed_commit": "1217978e257ed8db9508a5422d4de889286e913e",
  "errors": [],
  "scope": {"phase": "c", "sprint": "phase-c"},
  "review_mode": "plan",
  "model_adaptation": "Requested req-qa role executed under inherited Codex model; no Cursor Composer or Sonnet execution claimed.",
  "baselines_read": [".claude/project/quality-policy.md", "docs/requirements.md", "docs/architecture.md", "docs/project-plan.md", "docs/framework-charter.md", "docs/sc-runtime-design.md", "docs/sc-runtime/requirements.md", "docs/sc-command/requirements.md", "docs/sc-cron/requirements.md", "docs/sc-runtime-db-sqlite/requirements.md"],
  "phase_or_sprint_docs_read": ["docs/plans/phase-c/phase-c-plan.md", "docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md", "docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md", "docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md", "docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md", "docs/plans/phase-c/anchors.json", "docs/plans/phase-c/anchors.md"],
  "requirement_checks": [
    {"id":"REQ-CMD-0005","source":"docs/sc-command/requirements.md:445","listed_in_sprint_doc":true,"result":"violated","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:29","docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:88","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:175"],"notes":"The planned RMCP upgrade conflicts with the active explicit rmcp 3.7 type contract; no sprint owns its requirement amendment."},
    {"id":"REQ-RUN-0307","source":"docs/requirements.md:1974","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:54","docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md:52"],"notes":"The user-authorized fresh-install policy/version amendment is explicitly owned before implementation."},
    {"id":"NFR-RUN-0006","source":"docs/requirements.md:4081","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:54","docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md:88"],"notes":"Narrow amendment explicitly distinguishes configuration-derived consumer policy from copied maintainer policy."},
    {"id":"REQ-RUN-0004","source":"docs/requirements.md:404","listed_in_sprint_doc":true,"result":"violated","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:54"],"notes":"Active root lint clause still specifies 0.8.0, while the enumerated amendment only names 0307/NFR0006 and the policy."},
    {"id":"REQ-RUN-0503","source":"docs/requirements.md:3235","listed_in_sprint_doc":false,"result":"violated","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:11","docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md:107"],"notes":"The binding green-with-no-edits definition still installs sc-lint 0.8.0 and lacks an explicit amendment owner."},
    {"id":"NFR-RUN-0007","source":"docs/requirements.md:4138","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:175"],"notes":"Reviewed deliberate minor upgrade, one matrix and fixture proof preserve the bounded-minor pin rule."},
    {"id":"REQ-CRON-0001","source":"docs/sc-cron/requirements.md:5","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:91","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:124"],"notes":"Affected cron command dependency and exact release closure are explicitly probed/amended."},
    {"id":"NFR-CRON-0001","source":"docs/sc-cron/requirements.md:139","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:96"],"notes":"Defaults-off/no-server semantics remain explicit."},
    {"id":"REQ-RUN-0702","source":"docs/requirements.md:3625","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:99","docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:49"],"notes":"Release amendment, canonical selective-publication qualification, authorization and registry proof have owners; no publication is claimed."},
    {"id":"NFR-RUN-0010","source":"docs/requirements.md:4365","listed_in_sprint_doc":true,"result":"upheld","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:178"],"notes":"Child-daemon signal proof explicitly requires isolated venue."}
  ],
  "deliverable_checks": [
    {"item":"Phase C phase plan with boundary map, waves, feature closure and ownership","status":"present","closure_state":"not-applicable","evidence_refs":["docs/plans/phase-c/phase-c-plan.md:102"],"notes":"Plan artifact present; compliance defects below block approval."},
    {"item":"c-1 complete contract sprint specification","status":"present","closure_state":"not-applicable","evidence_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:43"],"notes":"Specification exists; requirement amendment/traceability corrections are findings."},
    {"item":"c-2 complete consumer-lint integration sprint specification","status":"present","closure_state":"not-applicable","evidence_refs":["docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md:36"],"notes":"Full installation, real analyzer receipts, configuration-derived policy, consumer ownership and preservation tests are planned."},
    {"item":"c-3 complete runtime/template ecosystem sprint specification","status":"present","closure_state":"not-applicable","evidence_refs":["docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:78"],"notes":"Actual composition, one identity, affected versions, SQLite guard and lifecycle proof are planned."},
    {"item":"c-4 complete authorized release sprint specification","status":"present","closure_state":"not-applicable","evidence_refs":["docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:47"],"notes":"Selective canonical-kit capability, ordered publishing and no-patch registry proof are planned."}
  ],
  "findings": [
    {
      "id":"REQ-QA-001",
      "severity":"Blocking",
      "category":"cross-doc-conflict",
      "source_refs":["docs/sc-command/requirements.md:445"],
      "target_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:29","docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:88","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:175"],
      "issue":"REQ-CMD-0005 explicitly defines CallToolResult as the rmcp 3.7 type. Phase C moves that public contract to the newly frozen RMCP line, but c-1 only owns repository and cron requirement files and does not amend this command requirement. c-3 consequently promises both changed RMCP types and preservation of an unchanged 3.7 requirement.",
      "required_correction":"Give c-1 ownership of docs/sc-command/requirements.md and explicitly amend only the REQ-CMD-0005 version identity to the frozen new RMCP matrix while preserving conversion/error semantics. c-3 must consume that amended contract. No new process artifact or implementation behavior is needed.",
      "compliance_result":"non-compliant"
    },
    {
      "id":"REQ-QA-002",
      "severity":"Blocking",
      "category":"cross-doc-conflict",
      "source_refs":["docs/requirements.md:404","docs/requirements.md:3235"],
      "target_refs":["docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:54","docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:120"],
      "issue":"The active root-lint clause of REQ-RUN-0004 and green-with-no-edits definition of REQ-RUN-0503 still require sc-lint 0.8.0. c-1's enumerated requirement amendment updates 0307 and NFR0006 only, so its promised consistent 0.8.4 contract can leave binding contradictory version statements behind.",
      "required_correction":"Explicitly include the current operational sc-lint-version clauses of REQ-RUN-0004 and REQ-RUN-0503 in c-1's amendment and acceptance check; add 0503 to c-1's requirement list. Preserve historical evidence/version observations rather than rewriting history. Existing documentation validation is sufficient.",
      "compliance_result":"non-compliant"
    },
    {
      "id":"REQ-QA-003",
      "severity":"Blocking",
      "category":"requirement-not-listed",
      "source_refs":[".claude/agents/req-qa.md:99","docs/requirements.md:277","docs/requirements.md:1434","docs/requirements.md:3745","docs/sc-runtime/requirements.md:408","docs/sc-runtime-db-sqlite/requirements.md:5","docs/sc-runtime-db-sqlite/requirements.md:21"],
      "target_refs":["docs/plans/phase-c/phase-c-plan.md:2","docs/plans/phase-c/sprint-c-1-consumer-runtime-contract.md:11","docs/plans/phase-c/sprint-c-2-consumer-sc-lint-full-profile.md:11","docs/plans/phase-c/sprint-c-3-runtime-ecosystem-upgrade.md:11","docs/plans/phase-c/sprint-c-4-runtime-ecosystem-release.md:11"],
      "issue":"The governing requirements lists omit directly exercised constraints: c-1/c-2 derive consumer crate/edge policy without REQ-RUN-0301; c-3/c-4 assert path-patched versus registry dependencies without REQ-RUN-0003; c-3 owns CLI dependency changes, graceful-shutdown regression, and SQLite manifest/resolver guard without NFR-RUN-0001, REQ-RT-0005, REQ-DBSQ-0001 or REQ-DBSQ-0005. The phase list also omits these governing IDs.",
      "required_correction":"Add these existing governing IDs to the respective sprint lists and phase union, and explicitly pair c-3's lifecycle/CLI/SQLite preservation checks with their existing requirements. For c-1's selective per-crate version contract also include REQ-RUN-0002. Audit the edited source/API and dependency boundaries for any other directly applicable requirement IDs using the same rule; no new requirement or behavior is requested.",
      "compliance_result":"non-compliant"
    }
  ],
  "summary": {"total_findings":3,"blocking_findings":3,"important_findings":0,"minor_findings":0,"overall_compliance":"non-compliant","deliverables_total":5,"deliverables_complete":5,"deliverables_incomplete":0,"deliverable_completion_percent":100.0},
  "gate_reason":"All five planning documents are present and full future sprint scopes are specified, but three requirement amendment/traceability defects block plan approval. The 100% figure is planning-artifact presence only, not implementation or final gate completion. Live Plan Anchor verification is running separately and no verdict is inferred here."
}
```
