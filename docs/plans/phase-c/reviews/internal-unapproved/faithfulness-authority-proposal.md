# Faithfulness authority: human decision proposal

**Status: proposal only. No approval is recorded. No new corpus, question wording, seal, or PASS is adopted. Further live and wording-only trials are paused.**

The decision is what **faithful** should mean when a formal requirement is more specific than its source. The old instrument explicitly permitted some JSON-shape and output-channel normalizations, but the generic quotes do not independently establish all of those equivalences. Higher answer confidence cannot settle that semantic boundary.

## Recommended decision

Approve this policy for a **separate, newly versioned generic calibration corpus**:

> Fidelity is preservation of the meaning authorized by the verbatim quote together with explicitly approved context bound to the applicable fixture. Ordinary paraphrase and formalization of a directive into requirement language are allowed. A statement may extract one supported duty from a compound quote; it need not reproduce unrelated duties. It must retain substantive conditions, prohibitions, applicability, and allowed behavior. It must not select a new JSON shape, output stream, governed surface, actor, trigger, algorithm, or policy without authority for that selection. An unspecified distinction is not permission to choose one alternative.

For test fixtures, approve this additional boundary:

> When the source requests a test using synthetic canary data, harmless concrete test values may instantiate that requested test. The instantiation must be identified as fixture-only data and must not add a production configuration binding, new trigger, output surface, storage policy, or behavior. Additional domain premises require explicit approval; they must not be invented or presented as previous approval.

This changes the generic fidelity construct where the old convention was permissive. It does **not** change actual consumer requirements, approve a Phase C plan, or authorize silently rewriting any user quote.

## Five exact fixture proposals

These are proposed **new-version** revisions. The quoted old text and every existing v2 label remain untouched. The positive/negative fidelity treatments below are proposals for the human decision, not an adopted oracle or a prediction of Jev answers. A complete five-role oracle, lint expectations, and duplicate relations must be independently reviewed and frozen **after approval and before any new answers**.

### 1. JSON format does not by itself select object shape — a0-04

**Existing quote:** “all CLI commands should print JSON on stdout”

**Existing statement:** “Every CLI subcommand must emit a JSON object on stdout.”

**Proposed new-version statement, with the quote unchanged:** “Every CLI subcommand must emit JSON on stdout.”

Proposed contrasts:

- **Positive:** the revised statement preserves the explicit stdout channel and JSON format without selecting a shape.
- **Negative:** the old object-specific statement, evaluated under the recommended new policy **without extra authority**, adds a restriction: JSON arrays and scalars need not be objects.
- **Optional NEW context proposal, requiring separate explicit approval:** “For this generic fixture, the required JSON output is a JSON object.” Only if that newly proposed context is approved and bound to a distinct fixture would the object-specific statement have the missing authority. This is not previously approved context and must not be attached retroactively to rescue the old result.

### 2. Preserve the named log/error surfaces without assuming stream mappings — a0-02

**Existing quote:** “do not print raw auth tokens in request logs or errors”

**Existing statement:** “The request logger must never write raw authentication tokens to stdout or stderr.”

**Proposed new-version statement, with the quote unchanged:** “Request logs and errors must omit raw authentication tokens.”

Proposed contrasts:

- **Positive:** the revision retains the common token prohibition over the two named surfaces.
- **Negative:** without a supplied mapping, replacing those surfaces with the request logger's stdout/stderr may change both the actor and the governed outputs. Persisted request logs or errors from another component are not automatically equivalent to those streams.
- **Optional NEW context proposal, requiring separate explicit approval:** “In this generic fixture, request logs and errors are produced only by the request logger and are written respectively to stdout and stderr; there are no other governed log or error surfaces.” This is an additional domain premise, not ordinary paraphrase or historical approval.

### 3. Do not narrow unspecified CLI output to stdout — a0-15

**Existing quote:** “CLI output must always be JSON”

**Existing statement:** “Every CLI command must emit JSON to stdout.”

**Proposed new-version statement, with the quote unchanged:** “All CLI command output must be JSON.”

Proposed contrasts:

- **Positive:** the revision preserves the format constraint on the output named by the quote.
- **Negative:** adding stdout without authority chooses a destination and may exclude stderr or other output that the original wording covered.
- **Explicit-channel positive contrast, proposed as a distinct generic fixture:** quote “CLI standard output must always be JSON”; statement “Every CLI command's stdout output must be JSON.” The channel is now in the source rather than assumed.

### 4. Keep ordinary directive formalization separate from an assumed stream — a0-16

**Existing quote:** “all commands should print machine-readable JSON”

**Existing statement:** “Every CLI command must emit JSON to stdout.”

**Proposed new-version statement, with the quote unchanged:** “Every CLI command must print machine-readable JSON.”

Proposed contrasts:

- **Positive:** an ordinary directive is formalized into requirement language while its output description is retained.
- **Negative:** without an approved convention defining “print” as stdout in this domain, selecting stdout is not independently established by this quote alone.
- **Explicit-channel positive contrast, proposed as a distinct generic fixture:** quote “All commands must print machine-readable JSON on stdout”; statement “Every CLI command must emit JSON on stdout.” This supports testing ordinary print/emit terminology with the channel already authorized.

Proposals 3 and 4 may no longer be duplicate requirements: a constraint on the format of output is not automatically identical to an obligation to print output. Their **new-version** duplicate relation must follow the approved semantics; the old frozen duplicate labels must not be silently carried across the revision or changed in place.

### 5. Concrete canary test data is different from invented product requirements — a0-10

**Existing quote:** “test a transport failure with a canary TypeSafe key and make sure console output and saved run files contain neither the key nor its bearer header”

**Existing statement:** “When TYPESAFE_API_KEY is set to canary-123 and transport fails, stdout, stderr, and saved run artifacts must omit canary-123 and Authorization: Bearer canary-123.”

**Proposed new-version statement, with the quote unchanged:** “In a transport-failure test using a synthetic canary key, console output and saved run files must omit that key and its bearer header.”

Proposed contrasts:

- **Positive:** the revised statement preserves the requested test, failure condition, named output surfaces, and redaction duty without inventing a configuration binding or stream mapping.
- **Permitted concrete-data proposal, only after approval of the fixture-only rule:** use the non-secret value `fixture-key-7` and its synthetic header `Authorization: Bearer fixture-key-7` to instantiate the requested canary test. These are plainly **NEW proposed fixture values**, not previous user words or proof of earlier approval. They impose no production environment-variable name or additional output/storage policy.
- **Negative:** “Transport must load its credential from TYPESAFE_API_KEY and delete saved run files after failure” invents a configuration binding and deletion duty. Choosing concrete canary test data does not authorize either requirement.

The generic encryption contrast remains useful: “Keep backup archives encrypted” does not alone select AES-256-GCM. An explicitly approved algorithm clarification can supply that authority; absent or contrary authority cannot. The same rule applies to meaningful format and channel choices.

## Alternative: explicitly retain permissive domain conventions

The responsible human may instead choose domain-authorized normalization. That requires approving its actual premises and scope, rather than treating a successful old question as authority for them. At minimum, define:

| Boundary | Premise the human must decide explicitly |
| --- | --- |
| JSON shape | Does “JSON” require an object in this generic domain, or may it include arrays/scalars? |
| CLI output | Does “output” mean stdout alone, stdout plus stderr, or some other defined set? |
| Print/channel mapping | When may “print” be formalized as stdout, and what happens when the source specifies another channel? |
| Logs/errors | Which actors and destinations are included? Are persisted logs and errors outside the request logger covered or excluded? |
| Canary instantiation | Which choices are harmless fixture-only data, and which configuration bindings or new behaviors still require authority? |

Any approved convention needs exact text, scope, provenance, version, digest, and a consistent binding to applicable fixtures. It cannot override an explicit contrary quote or be inferred merely because a particular answer would pass. **No alternative premises are approved by this packet.**

The recommended approach is the quote-preserving revisions above, which avoid adding domain assumptions or changing actual consumer requirements.

## Decision and subsequent work

The human decision needed is:

> **Approve the recommended fidelity policy, fixture-only test-data boundary, and five generic new-version revisions above; leave the optional fictional context/domain-premise examples unapproved.**

Alternatively, identify the exact permissive premises to approve, their scope, and any fixture proposals to change. Approval of the recommended statement must not be interpreted as approval of every optional example.

After that decision, create a separate versioned authority/corpus proposal with the approval provenance and immutable predecessor hashes. Independently derive every role expectation and the positive/negative contrasts from the approved meaning, including intent-use and duplicate boundaries; review and freeze the full oracle and protocol before responses. Then calibrate without changing the required confidence or confirmation rules.

**Approval of semantics is not calibration acceptance.** A new instrument may still fail exactness, stability, or confidence. No higher score, new seal, production adoption, or readiness result is guaranteed.

## Preserved evidence

The existing v2 original corpus, its four-case generic supplement and two-case recorded-approval supplement, all 23 old states/labels, every raw response/result, and seal `96d8f74cf022d7515aaa0d4a8f4e103b9be8c7b776bf99794d9926ef13d8fdb4` remain immutable. Existing failures are not retroactively changed to PASS. No actual Phase C facts were used in these proposals, and no source, installed setting, question wording, or corpus file was edited.

Sources: [independent fidelity audit](/tmp/sc-runtime-phase-c-review/continuation/calibration/faithful-regression-independent-review.md), [preserved extension-05 result](/tmp/sc-runtime-phase-c-review/continuation/calibration/088-local-extension-05-execution-01/result.json), and [frozen generic requests](/tmp/sc-runtime-phase-c-review/continuation/calibration/088-local-extension-05-prepared/frozen-request-shapes.json).
