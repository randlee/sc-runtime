# Source-only intent descriptors for actual candidate rubric calls

These six repository-local files have exactly id/version/sha256/content.
SHA256 binds the UTF-8 content bytes. Content contains only selected exact
conversation excerpts: speaker attribution and verbatim text. It includes no
drafted anchor statement, fixture, agent interpretation or expected role verdict.
ANC-013 distinguishes the assistant's actual scope question from the human's
selected answer. ANC-018/019 retain the complete preceding human override
context and explicit request, rather than treating the negative quote alone
as authorization for every implementation choice.

The CLI's supported approved-intent file input can use these descriptors with
the matching candidate statement/quote/scope after the root binds an accepted
instrument. Source excerpts are already directly stated/accepted intent;
new statement/fixture approval is separate. The descriptors do not lock an
anchor, allocate its provisional ID or grant a rubric/fixture PASS.

No source-only descriptor includes other candidates merely to make one pass.
ANC-018 and ANC-019 share identical content digests because the same full human
messages support two distinct obligations; their descriptor IDs remain distinct.
The generic calibrator must never receive these project-specific source facts.
