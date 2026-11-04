# UNAPPROVED commit/deadline policy proposal

HOLD: this packet is for the parent/human decision. It amends no strict
guarantee, requirement, fixture label, question or supported-capability claim.
COMPAT-CRIT-002 remains open. The exact primary-source design assessment is
`commit-deadline-design-note.md`; it is not an independent fix-verification PASS.

A concrete implementable admission alternative would serialize a short live
stop-state check with SQLite's commit_hook for ordinary and restricted-cleanup
transactions (including implicit writes), record allow/veto and release that
controller lock before SQLite commit/native I/O. Stop-first vetoes; admission-
first can still physically complete later. Busy/progress/interrupt cannot
promise to preempt arbitrary fsync. No controller mutex across COMMIT, mutation
replay or detached ownership is acceptable.

The alternative must distinguish confirmed commit, confirmed late commit,
confirmed rollback and indeterminate state. Unknown is not cleanup_persisted=false;
late confirmed persistence is not rollback or bounded success. Physical join
remains owned; no effects after actual owned release. Readback, if authorized,
uses a bounded reconciliation budget; no expired-context reopening or blind
replay. No budget/proof means typed indeterminate, not an invented successful
rollback. Generation fencing remains, without claiming exactly-once identity.

Causal proof would pause after final precheck and at the real commit-hook
admission point, exercise stop-first/admit-first and post-allow expiry/shortening
for ordinary and cleanup transactions, then inspect actual persisted generation
state after physical join and test accurate typed outcomes. A simulated lost
result tests classification only, not a reproduced fsync failure. Strict
physical no-commit-after-G/K and universal completion-by-K would remain a
capability gap with supported APIs. Selecting this admission alternative changes
that strict physical interpretation and therefore requires the explicit pending
policy decision; nothing in this packet adopts it.
