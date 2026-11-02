# Phase B anchor reconciliation and review

Approved authority: ANC-027 through ANC-042 and the versioned Phase B intent.
This is planning evidence; no runtime implementation is claimed.

## Resolved drift

- Replaced the closed host list and render-only restriction with an application
  registry and both any-command CLI integrations in the existing b-16 sprint.
- Preserved render_template as an optional internal convenience rather than
  requiring its deletion or making it a product API requirement.
- Applied the tool-crate boundary to all production release crates, including
  actual ATM package identities and aliases, with test/benchmark exceptions.
- Removed signing/approval implications from MVP and unreleased-schema migration
  obligations from the first-release schema plan.
- Reconciled old REQ/ADR anchor snapshots, preserving provenance and history.
- Made executable resolution independent of child working directory; both CLI
  adapters use the shared blocking child driver without an async/runtime bridge.
- Fixed generated host-registry injection and explicit environment parity,
  daemon-name allowlists, and no ambient capture in b-18.
- Consolidated duplicated b-17 prose while preserving its interface, limits,
  resolution order, redaction, authorization rules, and acceptance commands.

## Instrument and fixture verification

The original calibration corpus changed the meanings of two source quotes.
The corrected, independently frozen successor passed all 85 calibration cells;
original corpus, attempted candidates, and raw evidence remain preserved.

The tuner compared multiple question candidates through repeated Jev calls and
versioned successor oracles, without changing approved statement meanings or
lowering production thresholds. All 32 current-authority planning/development
question sets are accepted. Both pass/fail fixtures passed at both gates for
every new anchor: 32 check-fixtures invocations, 64 fixture cases. Questions and
canonical histories are deployed in the linked Beads question records; local
copies are under deployed-question-bodies/.

See tuning-final-report.md and final-tuning-inventory.json for exact oracle,
authority, history, and question digests. Diagnostic-plan-comparison/README.md
preserves the actual before/after plan results and edits that improved them.

The Git commit keeps human-facing reports and an unmodified raw-evidence.tar.gz
archive, rather than thousands of generated request/response files in the PR
diff. The working directory retains those original files. The archive contains
the complete tuning-20261029 tree, including every predecessor, raw call,
candidate, frozen oracle, fixture check, deployed question body, and canonical
history. Extract it into a fresh temporary directory to inspect the historical
paths without replacing current reports. No raw response was dropped to reduce
the review diff.

## Final planning gate state — PASS

Latest exact-source rechecks after scoped QA remediation are recorded in
../../qa-runs/final-scoped-20261030/anchor-recheck-summary.json. All six pass.
The 16 approved ANC statements/scopes/questions remain unchanged. The legacy
REQ-CRON-0005 snapshot now distinguishes library entry points from external
callback protocols, preserving its original quote; its planning/development
fixtures pass with concrete existing-signature evidence. The original gate
archive below remains the immutable 67c9dc2-era snapshot. Final scoped QA
verification passed for req-qa, arch-qa, and extensibility-qa on defb2530,
closing all 11 findings. The current qa-runs directory contains the exact
source hashes, full reports, and preserved reviewer evidence.

All six final b-13–b-18 runs pass with all 44 locked anchors selected. All 16
new anchors are satisfied in the continuation; unchanged parser/store anchors
have version-matched passing baseline coverage. No locked anchor is uncovered.

The owner explicitly authorized the full payload and TypeSafe/Jev destination;
the earlier automatic approval block is resolved. Clear plan defects were
fixed, duplicate prose consolidated, and independently audited item exclusions
recorded. No statement, scope, question wording, version, threshold, or sprint
count changed. No unresolved product-intent decision remains.

See approval-gate-20261030/README.md for final run IDs, exact validation,
allowed growth authorities, and complete immutable evidence archive. Historical
failed/rejected attempts remain in that archive and the earlier raw archive.
