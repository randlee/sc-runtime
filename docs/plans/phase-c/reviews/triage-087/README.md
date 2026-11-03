# Actual 0.8.7 testability triage

Canonical plan-anchor source: `273780863780a86c79be7f59c2f37c06039ee3a6`.
sc-runtime installation commit: `3c1d5d7f71ad75954e3fed422792205530104049`.

| Recorded case | Actual result | Remaining authority gap |
| --- | --- | --- |
| ANC-001 faithful | `EVIDENCE.INCOMPLETE` | Approved fixtures name the analyzer set, but the judged canonical source contains only statement, quote and scope. Decisive evidence for what the original “full sc-lint” quote denotes is still needed. |
| ANC-003 single_constraint | `EVIDENCE.INCOMPLETE` | The same 0.8.4 invariant on two artifacts admits differing readings; the supplied evidence lacks a frozen approved rubric-role expectation resolving that distinction. |

Request/result schemas and relational contracts, runner fences, raw-response
bindings and attestations were verified. Files are copied unchanged from the
actual independent executions. Scripts and redundant rendering variables are
excluded. The br/bd installer smoke report is retained separately; no source
archive is vendored into this repository.

These are typed incomplete-evidence results, not anchor/question-defect
adjudications. The sealed questions, approved anchors and fixtures, recorded
probabilities and failing rubric outcomes remain unchanged. The parent is
working on the missing authority evidence. No full-sprint live gate, coverage
completion or overall plan approval is claimed.

## Second pilot round

R2 adds existing approved human intent/provenance and already frozen corpus
references; it leaves the original source, raw repeats, anchors and question
seal unchanged. Both independent results return `question_defect` with valid
result fences and contract/schema checks:

- [ANC-001](r2/ANC-001-result/result.json): the faithful question considers
  only the abbreviated quote, dropping already approved analyzer context.
- [ANC-003](r2/ANC-003-result/result.json): the single-constraint question
  misapplies shared-version predicate versus independent duties compared with
  frozen corpus cases `a0-06-genuinely-bundled` and `a0-10-api-key-redaction`.

The added [intent context](r2/existing-approved-intent-v2.md) distinguishes
exact human approvals from assistant-authored analytical opinions. Raw repeat
bodies remain in the original sibling `ANC-001-repeat-evidence.json` and
`ANC-003-repeat-evidence.json` files and in the bound requests; comparison
artifacts verify their unchanged identities. Successful adjudication does not
change a rubric verdict or authorize an untested override. Retuning and
resealing the affected questions, then actual fixture/full-sprint gate runs,
remain the next work. Earlier incomplete-evidence results are historical
records, not overwritten failures.
