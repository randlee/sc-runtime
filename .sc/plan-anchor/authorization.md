# Durable Jev review authorization

The repository's committed `.sc/plan-anchor/settings.yaml` already sets
`allow_remote_judge: true`. This record preserves the user's scope and approval
provenance for delegated reviews; it contains no credentials and does not change
the settings schema.

## Approved scope

The user authorizes sc-runtime Phase C plan and development-anchor reviews for
sprints C1–C6, including ordinary rechecks after fixes, through TypeSafe Jev at
`POST https://api.typesafe.ai/v1/systemone`.

Permitted review content:

- The owning sprint plan and approved anchor statements and scopes.
- Normal delivery reports and PR titles and descriptions.
- Changed-file names and counts, without full diffs.
- Concise test, build and lint validation summaries.

Full source files, full code diffs, raw diagnostic logs and credentials are
outside this authorization. Keep authentication in the existing credential
provider; never include its values in review inputs, records or reports.

This approval persists across sessions, worktrees and delegated agents. Routine
reviews and rechecks within this scope use this existing approval rather than
requesting consent again. A materially new destination or payload outside this
scope, or an explicit user revocation, requires a new decision.

## Human provenance

Earlier plan-review approval in this conversation was:

> Approve Phase C reviews via Jev/TypeSafe

On November 5, 2026, the user answered the development-evidence approval question
`request_user_input_async / call_e7RY4mx1BXJ5RTRXJsBV468B / item 0`.

The question specified:

> may I send the C1–C6 sprint plans, approved anchors, normal PR descriptions,
> changed-file names/counts, and concise validation summaries to Jev/TypeSafe
> (api.typesafe.ai)? This covers rechecks after fixes. No source files, full
> diffs, raw logs, or credentials.

The user's answer was:

> Approve Phase C development evidence via Jev/TypeSafe

The user then instructed that this approval be committed to the repository so
previously granted authorization does not become a repeated blocking question.

## Delegation and approval review

Before invoking a Plan Anchor gate, the orchestrator reads this record and the
committed settings, cites the authorization record's Git commit in the delegated
request, and includes the permitted payload scope and human approval reference
in the leaf agent's context. A worktree that predates the record can read the
committed record from its current integration target without changing its
qualified source revision.

Do not replace this authorization with an agent's unsupported claim of consent.
Respect automatic approval review: if an action is rejected, report the exact
action and reason with this authorization reference so the parent can resolve
the context. Do not bypass a rejection through another route or indirect
execution.

## Phase C calibration approval — November 6, 2026

The user additionally approves Phase C question calibration and subsequent tuning
and rechecks for C1–C6 through the same endpoint,
`POST https://api.typesafe.ai/v1/systemone`. This extends the approved payload
class to approved anchor text, candidate questions, and fictional pass/fail,
boundary and evidence cases. The prepared C5 batch contains 90 requests pinned
to Jev 1.14.0. The approval covers subsequent Phase C batches and rechecks of
this same payload class; it is not limited to that first batch and must not
be requested again per batch, session, worktree or delegated agent.

Source files, full diffs, raw logs, credentials and private Beads data remain
outside this approval. Existing ordinary plan and development-review consent
above remains in force. Authentication stays in the existing credential provider.
This export approval does not adopt a question bundle, alter approved fixture
meanings, certify a gate or authorize publication. Independent tuning, frozen
expectations and all normal verification remain required.

Direct human provenance is `request_user_input_async`,
`call_xkFLs3bj2zJVxVUjG6u9Lo1o`, item 2. The user answered:

> Approve Phase C calibration exports and rechecks

The exact question and all three approved decisions, including the C1 stage
fixtures and narrow Python scratch dependency exception, are recorded in
[the approval receipt](../../docs/plans/phase-c/reviews/authority-approvals/phase-c-stage-policy-and-calibration-approved-2026-11-06.json).
The receipt preserves proposal digests and scope. Delegate with this committed
record and the actual human approval provenance. Respect automatic approval
review; this approval resolves the earlier expanded-payload scope gap without
changing destination or bypassing review.

## Phase D anchor validation approval — November 11, 2026

The user authorizes validation of the rewritten Phase D anchors through the same
TypeSafe Jev endpoint, `POST https://api.typesafe.ai/v1/systemone`, including
testability checks, independent question tuning, fixture checks, and routine
rechecks after corrections.

Permitted Phase D payloads are anchor statements, verbatim source quotes and
scopes, intent excerpts, candidate questions, and fictional pass/fail, boundary,
and evidence cases used to validate those anchors. Send one anchor or test case
per request. This approval persists across sessions, worktrees, and delegated
agents for this payload scope; do not request the same approval again.

Full source files, full diffs, raw diagnostic logs, credentials, and unrelated
private Beads data remain excluded. Phase D implementation evidence, publication,
and exports beyond the payload scope above are not authorized by this record.
Keep authentication in the existing credential provider. Respect automatic
approval review and report any actual rejection; never bypass it.

Direct human provenance is this conversation on November 11, 2026. The concrete
anchor packet was committed as `95e86db534e6c06a858449550a93bc55a4128544` on
`plan/phase-d`, PR #135 targeting `develop`.

The approval question was:

> May I send Phase D anchor text, intent excerpts, candidate questions, and fictional test cases to Jev/TypeSafe for validation and rechecks?

The user answered:

> yes. make sure jev approval is committed and in pr to develop

This is export authorization, not adoption of the five proposed anchors or a
claim that validation passed. Existing approval and independent-verification
requirements for anchor adoption remain in force.

### Phase D plan assessment clarification

Later in the same conversation on November 11, 2026, the user directed:

> tuning is send to jev, access ancors + plan -> fix -> run jev (repeat until all pass)

This extends the Phase D validation payload scope to the owning sprint plan
text, one complete sprint per Jev request, for plan assessment and rechecks
after fixes. Anchor validation and independent question tuning retain their
existing requirements. The source-code, full-diff, raw-log, credential, and
unrelated-private-data exclusions above remain in force.

### Phase D calibration recovery approval

The independent tuner identified errors in its own frozen ANC-005 and ANC-012
calibration cases. The runner correctly refused replacing the recorded oracle
within the existing history (`TOURNAMENT.ORACLE_CHANGED`). No rejected result
was converted to an accepted result.

On November 11, 2026, the user explicitly authorized separate, versioned
calibration lineages for correcting those cases while preserving all prior
evidence. The approval question was:

> May I start a new, explicitly versioned calibration lineage, preserving all failed cases/results, so the tuner can correct those errors?

The user answered:

> Approve fresh calibration; preserve prior evidence

Direct provenance: `request_user_input_async`,
`call_1bb282cd72514f579f36e1144d681cff`, question item 0.

This permits new independently frozen corpora and new canonical history IDs
with local bindings to the previous failed lineages. Original corpora,
histories, results, and the refused correction attempt remain immutable. It
does not authorize changing an accurate question to fit a plan, relabeling a
past result, discarding failed evidence, or weakening production thresholds.
The existing Jev destination and permitted payload scope remain unchanged.
