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

## Final planning gate state

- b-13: passed the scoped final gate with deployed questions.
- b-16: no final verdict. Automatic approval review rejected sending the full
  sprint and anchors to https://api.typesafe.ai/v1/systemone, even after the
  agent checked committed allow_remote_judge:true and the owner's instructions
  to iterate Jev reviews. The rejected export was not retried after that second
  rejection. Exact rejection evidence is under final-plan-gates/.
- b-17: the final reviewed variant satisfied nine anchors; ANC-031 was
  underspecified, leaning satisfied. An explicit inherited production boundary
  statement now corrects that omission. The changed variant has not received a
  new final remote gate; earlier outcomes remain unchanged as evidence.
- b-14, b-15, b-18: local/adversarial review completed; final remote gates pending.

No unresolved product-intent decision was identified. Final remote gate
completion remains pending the explicit payload/destination authorization that
automatic approval review requires. The overall final planning gate is not
reported as passed. The tracking task remains in progress.
