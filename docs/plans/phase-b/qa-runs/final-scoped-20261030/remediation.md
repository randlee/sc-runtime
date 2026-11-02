# Scoped QA remediation

Reviewed source: 67c9dc25aab0e4706db9637067b35d1a06690195. All 11 findings
passed ceremony screening before edits. No owner product decision is needed.

| Finding | Minimal correction | Existing authority |
| --- | --- | --- |
| REQ-QA-FINAL-001 | Reconcile b-13/b-16/b-17 requirement frontmatter and body citations. | REQ-CRON-0005, REQ-RUN-0329/0503 |
| REQ-QA-FINAL-002 | Allow logger in the existing builder criterion, retain string-based StoresFailed while allowing logging types, use logger() in both canonical examples; add REQ-RT-0007 to b-13 amendments. | REQ-RT-0001/0007, REQ-RUN-0309, ADR-RUN-0004 |
| REQ-QA-FINAL-003 | Root lockfile proves types 1.5.1; generated lockfiles prove the full pair. No new library dependency or harness. | REQ-RUN-0309 |
| REQ-QA-FINAL-004 | Preserve failure_parts' supported parser; event codes still come from typed origins. | REQ-CRON-0005 |
| REQ-QA-FINAL-005 | Distinguish library entry points from application-implemented callback protocols; resolver Denied/Unavailable maps to ENV_REFERENCE without provider prose. Clarify the legacy REQ anchor with its original quote retained and recheck its existing fixtures. | REQ-CRON-0005/0019, ADR-RUN-0016 |
| REQ-QA-FINAL-006 | HOST_DENIED belongs to bridge conversion/supplied-adapter shape validation; every custom handler error, including InvalidSpec, is HOST_CALL_FAILED. Use private typed origin dispatch and the existing handler-error case, no argument-schema API. | REQ-CRON-0003/0005, ADR-RUN-0013 |
| ARCH-001 | List/cite the governing dependency ADR in b-16. | ADR-RUN-0003 |
| ARCH-002 | List/cite the existing crate/template placement ADR in b-17. | ADR-RUN-0002 |
| ARCH-003 | Deny registered host calls during initialization, preserving VM-local computation and environment reads. | ADR-RUN-0016 |
| ARCH-004 | Name b-14's fixture-private shutdown trigger; real process signals remain b-18-owned. | ADR-RT-0004 |
| EXT-001 | Add generic new_with_logger, preserve supplied S/C/J, keep open_with_logger as built-in convenience; legacy new delegates with Noop. One behavioral case in the existing contract test verifies custom-component use and scheduler/worker emission. | General-purpose/open ScheduleStore seam, REQ-CRON-0001/0003, ADR-RUN-0004/0013 |

The generic constructor installs only scheduler/RunCoord/worker/hook logging.
Application-supplied components own their internal logging. The built-in open
helper constructs its logger-aware store/jitter and delegates generic scheduler
construction; logger.attached is emitted once, not once by each constructor.
No new trait, store backend, wrapper, post-construction setter, sprint, checker,
or required process artifact is introduced.

All added passages implement the pre-existing authorities above. Duplicate
release/ownership prose may be consolidated; no interface, limit, precedence,
acceptance command, or safety requirement is removed to fit a gate.

The clarified legacy error anchor passed its existing planning fixtures.
The development fixture checker correctly rejected the old positive example
as an unsupported assertion: it contained no artifact reference. Preserve
that result. The successor positive example keeps the same approved CronError
obligation and adds checkable existing signatures in expr.rs and scheduler.rs;
the negative example and expected labels stay unchanged. No question, threshold,
or product requirement is weakened. Recheck both fixture gates and ask req-qa
to verify the references along with REQ-QA-FINAL-005.

b-17's refreshed planning gate found the library-error invariant unclear after
code reduction (REQ-CRON-0005, confidence 0.58). Replace its generic model
introduction with the same explicit library-entry-point/CronError versus
resolver-protocol/ENV_REFERENCE boundary as canonical REQ-CRON-0005/0019.
Existing constructor/accessor compile checks already cover those return types;
do not add a duplicate API test or another gate.
