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
