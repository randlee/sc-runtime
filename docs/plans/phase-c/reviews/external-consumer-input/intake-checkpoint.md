# External consumer intake checkpoint

These are external source assessments and reported consumer validation, not
sc-runtime implementation, owner QA, publication authority, consumer baseline
adoption or a waiver. No patch was applied and no source/build was executed
for this evidence checkpoint.

The exact logging `STATUS.md`, `provenance.json` and `validation.json` under
`logging-proposal/` report the proposed facade receiver's actual events, locked
normal/build graph and expected E0432 unpatched negative. The combined proposed
patch at `/tmp/phase-c-execution/c-1/qa-4/logging-proposal/logging-proposal.patch`
was read-only hash verified as
`135e52e2be4e02789980325c606ebe2f376576241fc5f2cf703e112395218837`.
That hash/reference is retained without applying or adopting the source patch.
The reports keep their external source/tag identities and still-open findings.

`portable-endpoint/flaky-test-qa-result.md` is the exact external consumer
report at `e7bafc2e5bdda1bf480823c7dd3ac73d0e16b702`. C4-FTQ-001 describes a
source-proven reserve/drop/rebind race, not a claimed observed hosted failure.
The upstream read-only endpoint assessment is separately preserved at
`../compatibility-scope-r1/portable-fixture-endpoint-addendum.md`. It identifies
existing continuous port-zero ownership and the planned runtime observation
seam; neither report closes the external finding or claims shipped changes.
