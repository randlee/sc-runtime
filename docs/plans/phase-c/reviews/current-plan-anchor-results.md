# Current Phase C plan-anchor results

**NOT FINALIZED.** The human has approved all six new statements and their
original pass/fail fixtures, with the explicit ANC-016 wording clarification
below. The records remain **proposed**, pending actual testability/live checks;
they are not locked or given question Beads. The record validates: 18 total, comprising 10 locked,
six proposed and two retired. All 12 original record objects are unchanged.

## Results against the current plan

| Evidence | Actual result |
| --- | --- |
| [Current structural checks](current-structural-anchor-checks.json) | Three PASS: Phase C naming, dedicated worktree and separate PR to develop. |
| New-anchor live testability and live sprint-plan gates | Not run; no current-plan Jev verdict or Plan Anchor PASS is claimed. |
| [Engineering baseline diagnostic](internal-unapproved/consumer-extension-baseline-diagnostic.md) | Four manual pre-gate findings; **not Jev**. The [focused plan revision](consumer-extension-plan-revision.md) now addresses their acceptance/ownership gaps; fresh independent reviews and actual live checks remain pending. |
| [Critical R2](compatibility-critical-r2/report.json) | PASS at a265f65 for its reviewed six-sprint contract; subsequent consumer-control clarification still needs follow-up plan review. |

## Bounded instrument result

[Strict final result](continuation/calibration-v3/evidence/v3-round05-final/result.json):
`success:true`, `accepted:false`. Calibration ended after five rounds, with
21 complete candidates tested against the unchanged 36-case oracle. There
were 756 semantic CLI executions and 1,548 confirmations, including 36 third
calls triggered by initial choice disagreement; no retries.

The diagnostic leader had 179/180 exact, 180/180 stable and 170/180 confident
cells. One high-confidence semantic mismatch and nine insufficient-confidence
cells still fail acceptance. The separately evaluated dedupe correction passed
all eight pairs with two confident agreeing answers. That success alone does
not authorize a partial rubric or new seal. See the exact
[final diagnosis](continuation/calibration-v3/evidence/v3-round05-final/final-diagnosis.md).
No further calibration round was started. All original failures, answers and
correlation/summary corrections are preserved in byte-exact archives with
[per-file hashes](continuation/calibration-v3/archive-snapshots/final-snapshot.index.json).
The old corpus, labels, evidence, production seal and installed tools remain
unchanged.

## Proposed records and approval packet

The [six-statement/fixture packet](internal-unapproved/phase-c-six-candidate-approval-packet.md)
retains its original provisional references. The
[allocation mapping](proposed-anchor-allocation.json) binds those references
to actual record IDs:

| Packet reference | Actual proposed record | Obligation |
| --- | --- | --- |
| ANC-013 | ANC-013 | Compatibility before publication |
| ANC-014 | ANC-014 | Optional standard cron command set |
| ANC-015 | ANC-015 | Shared cron JSON contract |
| ANC-016 | ANC-016 | Common verbs across handler kinds |
| ANC-018 | ANC-017 | Supported consumer overrides |
| ANC-019 | ANC-018 | Documented integration and consumer responsibilities |

The exact [human approval receipt](authority-approvals/new-anchor-approval-with-anc016-clarification.json)
preserves the assistant question, human answer and preceding clarity approval.
ANC-016 now states: **The standard cron command set must use common
schedule-management subcommands across Python, Monty, and Rust handlers.**
Its quote and fixtures are unchanged; version/history preserve the earlier
wording. The original packet and allocation file remain immutable historical
proposals, including their earlier pending-approval fields; this receipt records
the later approval and its explicit exception.

Packet ANC-017's three-job-target quote is retained under existing
REQ-CRON-0003 rather than counted as another obligation. Ordinary defensive
validation remains; consumer control does not mean mandatory removal of
security/integrity checks. The existing out-of-box cron intent is retained as
source context for baseline coverage, not an invented seventh anchor.

Statement/fixture approval is now recorded. Actual successful checks and locking
remain pending because no full rubric was accepted. Record validation and
proposed persistence are not live testability or gate approval. Tracking remains
in run-x8d.5/.6. The focused plan/REQ/ADR revision follows the direct approved
consumer-extension intent, not calibration failures. Runtime source, evaluator,
anchor quotes/fixtures and seals remain unchanged. The four manual findings
remain to_fix pending independent verification; no new Plan Anchor PASS is claimed.
