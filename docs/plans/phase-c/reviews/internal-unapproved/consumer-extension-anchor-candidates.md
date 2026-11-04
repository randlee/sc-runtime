# Consumer extension and documented-use anchor candidates

Candidate preparation only: not recorded, locked, rubric-tested or fixture-
approved. The exact complete human messages, including spelling and whitespace,
are preserved in `consumer-extension-anchor-candidates.json`. The user expressly
identified these sentences as good anchor statements. The formal statements
and fixtures here are agent-authored drafts, not substituted human quotations.

Existing record IDs end at ANC-012; the earlier candidate packet uses provisional
ANC-013 through ANC-017. The following next IDs are provisional until actual
capture. Durable progress remains in run-x8d.5 and run-x8d.6.

## Candidate ANC-018 — consumer control through supported extension points

Quote: "it is not sc-runtime job to guarantee a customer can not misuse the core libraries."

Statement: **sc-runtime must expose supported extension points that let consumers replace default core-library behavior with application-specific behavior.**

The preceding user message explicitly requires hooks and full control to
override behavior, with consumer-owned row schemas and shutdown policy as
examples. This candidate preserves that control; it does not require disabling
defensive validation, security or data-integrity checks, or making every private
implementation detail configurable.

Draft pass: The cron integration accepts an application-provided store with an
extended row schema and a consumer shutdown policy through supported extension
points instead of requiring its bundled defaults.

Draft fail: Every consumer must use the bundled cron store and default shutdown
policy; custom row schemas and application-specific lifecycle behavior have no
supported integration seam.

## Candidate ANC-019 — documented correct integration path

Quote: "it is sc-runtime job to provide an easy documented path that consumers will not accidently misuse the core libraries."

Statement: **sc-runtime must document a supported integration path that explains correct use of the core libraries and the responsibilities of consumer overrides.**

This is an observable documentation obligation, not an absolute proof that
consumers cannot make mistakes. The already-stated working default cron path
remains context rather than a third obligation bundled into this statement.

Draft pass: The integration guide shows the default cron setup and a supported
custom store or shutdown policy example, including the consumer's validation,
lifecycle and ownership responsibilities.

Draft fail: Consumers must inspect private scheduler internals to discover
override wiring; the documentation explains neither supported hooks nor
consumer responsibilities.

## Distinctness and pending work

Replaceability can exist without documentation; documentation can exist while
the implementation still forces default coupling. These obligations are not
duplicates. Existing ANC-008 concerns consumer boundary-policy ownership, and
REQ-CRON-0003 concerns job targets; neither necessarily provides these two
properties. The framework charter is related composition authority, not a new
job engine or a reason to count the same obligation twice.

No locked record, question override, fixture verdict or plan-readiness PASS is
changed. Actual rubric/deduplication, statement/fixture approval, capture and
question linkage remain pending after independent generic calibration. The
clarification requires follow-up plan review; critical R2's prior PASS remains
limited to a265f65. These project-specific candidates never enter the generic
calibration oracle.
