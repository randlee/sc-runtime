# Phase d plan hardening rounds

| Round | Step | Reviewer | reviewed_commit | status | blocking | important | minor | findings_hash | supersedes | Note |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | — | parent | (pre-hardening) | draft | — | — | — | — | — | Initial fast draft; **not** guidelines-compliant |
| 1 | 1 | developer | 6e6a754 | PASS | 0 | 0 | 0 | — | 0 | Step-1 JSON: guidelines pass; beads run-ajx; `.sprints/d/structure.ttl` on PR #135 |
| 2 | 2 | plan-scope-reviewer | 6e6a754 | FAIL | 0 | 8 | 3 | pd-6e6a754-r1 | — | [Plan scope review phase d](5dad6113-5729-416d-8e79-c9362fbd97bf); step-3b addresses scope (2-sprint parallel doc wave) |
| 2b | 2 | plan-scope-reviewer | 14031f4 | FAIL | 4 | — | — | phase-d-scope-r1 | — | Superseded by 2-sprint shape; [earlier scope review](e2f06dbc-ec7d-4e68-9bcf-4026c3db78ae) wanted 3 sprints — rejected per later scope pass |
| 3 | 3 | developer | pending | pending | — | — | — | — | — | |
| 4 | 4 | critical-plan-reviewer | 518022a | FAIL | 1 | 6 | 6 | phase-d-518022a-CRIT-R1 | — | Background [Phase d critical plan](8ba0bcb7-5d70-4493-96b0-7e3ff51989ae); step-3 developer pass addresses PLAN-CRIT-001..007 |
| 3b | 3 | developer | pending | pending | — | — | — | — | 4 | Routed CRIT findings → sprint/plan edits |
| 5 | 5 | developer | pending | pending | — | — | — | — | — | |
| 6 | 6 | cursor-quality-mgr | pending | pending | — | — | — | — | — | Plan QA-1 after steps 1–5 PASS |

Cycle limits (default 3): `plan_scope_review_cycle_limit`, `critical_review_cycle_limit`, `plan_qa_cycle_limit`.

## November 11 intent and anchor alignment

At7e084a6, the three revised sprint documents passed the live planning anchor gate. The
independent scoped guidelines recheck closed four structural findings; the final
HTML report records exact runs and limits. These checks do not change the
historical pending rows above or certify formal QA-1. Human approval, normal plan
QA and merge to develop remain prerequisites to implementation.

## Lead-directed scope/critical correction — November 11

Input: STEP2-R1 scope review and STEP4-R1 critical review of7e084a6. Both failed;
that supersedes the earlier limited guidelines assurance. The lead requested one
correction pass, then lead-owned plan review, not another self-certification loop.

| Finding | Planner ruling | Concrete correction |
| --- | --- | --- |
| PLAN-SCOPE-001 / PLAN-CRIT-007 | fixed | d-4 is a disjoint wave1 tooling sibling; d-1 retains shared workflow/Justfile ownership and d-4 alone owns existing tool action. Width2→3; critical path2→2; count3→4. |
| PLAN-SCOPE-002 / PLAN-CRIT-002 | fixed via reviewer's alternative | Move registry workflow and complete consumer interface to d-3. d-1 source CI has no future-consumer branch. Numbered release acceptance and deliverable own actual positive and negative runs. |
| PLAN-SCOPE-M1 | fixed | d-1 ranges criterion refers to actual Cargo requirements/rendered inputs, not inventory rows. |
| PLAN-CRIT-001 | held | Lead instruction22:37:37Z: do not ship the proposed legacy cap pending user's same-database direction. Existing floor claim remains unresolved. |
| PLAN-CRIT-003 | fixed | d-3 defines the exact beads-cache reduction from02d795c, manifest/features/native-link identity and separate rmcp server proof; no absent-dependency inference. |
| PLAN-CRIT-004 | fixed | Packaged-archive rehearsal before upload; same-hash retry for missing uploads; defect-specific yank rules; source owner bumps all six together for an occupied defective version. Issues stay open. |
| PLAN-CRIT-005 | held | Architecture/versioning/configuration decision awaits SQLite direction; no new unsupported-graph decision is normalized. |
| PLAN-CRIT-006 | held | Shared supported-configuration facts await SQLite decision; docs dependency is not silently resolved. |
| PLAN-CRIT-M1 | fixed | Snapshot tracing row applies only to template; d-1 owns manifest bumps, d-3 publication. |
| PLAN-CRIT-M2 | fixed | Existing benchmark checker, report path and exact source-equivalence refresh condition named in d-3. |
| PLAN-CRIT-M3 | fixed | All d-1 integration criteria carry req:REQ/NFR roots; tooling criteria moved to d-4. |

These are planner rulings, not reviewer PASS results. No live anchor gate against
this four-sprint correction is claimed. Prior HTML reports/validation are historical
at7e084a6. Lead plan review and all held SQLite findings remain outstanding.
