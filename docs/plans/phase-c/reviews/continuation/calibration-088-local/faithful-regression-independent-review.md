# Independent frozen generic faithful instrument/oracle audit

## Finding

The contested positive labels are consistent with an **explicitly documented normalization convention in the old sealed instrument**. They are not arbitrary labels inferred from recent answers. However, the supplied generic quote/statement pairs do not independently establish every channel mapping or JSON-shape equivalence that convention permits. The strongest unresolved issue is the boundary between authorized normalization and a substantive change, especially JSON → JSON object. That is a construct/fixture-authority issue as well as a possible question or evaluator limitation; the latest failures alone cannot decide between them.

Keep current labels, evidence, and the old seal unchanged. No candidate passes the required full gate. This review supplies no replacement labels, question wording, new context, or authorization to change fixtures.

## Scope and evidence

This read-only audit examined the shipped 17 generic semantic cases; the four existing generic supplements; the two recorded-approval positive cases; the original sealed faithful-06 question and recorded candidate/ranking provenance; extension-05 frozen questions, rationales, request shapes, results, acceptance table, and supported CLI records; and extension-04 result rankings. No actual consumer plans/anchors or producer implementation were inspected. No live requests or outbound messages were made. The sole new file is this review. `cm context` ran first; its returned Rust CI rule was irrelevant and its CASS retrieval was degraded.

The old sealed faithful-06 explicitly permits “technical output-channel formalization for printing or logging” and “ordinary JSON container terminology where no container distinction was specified.” Its criteria still reject an additional policy, implementation selection, strength, trigger, or result. The recorded faithful-04 and faithful-05 candidates make the JSON/object and log/print/stdout/stderr permissions even more explicit. Their recorded rankings, like faithful-06's, show 17/17 accepted faithful cells. These are documented historical instrument decisions and performance evidence; successful calibration is not independent proof that the convention preserves meaning in every application. No separate substantive approval record establishing the full scope of those equivalences was identified in the generic materials inspected.

The six extension-05 candidates retain general normalization language but do not all reproduce the old explicit JSON-container permission. They also add intent handling. Thus extension-05 tests changed faithful instruments, rather than establishing that the original faithful-06 itself now fails on the identical original request set.

## Recorded outcomes and their limits

Extension-04-03 recorded 115 exact, stable cells and 114 confident cells; a0-02 faithful was `yes` at 0.61/0.55. It remained unaccepted. Extension-05's selected diagnostic bundle, extension-05-01, records 114/115 exact, 115/115 stable, and 112/115 confident cells. Its faithful failures are a0-04 (`no` 0.49/0.33), a0-15 (`yes` 0.44/0.37), and a0-16 (`yes` 0.30/0.26). Every extension-05 bundle remains unaccepted. The displayed `candidate_count: 30` counts the role-level candidates; the frozen experiment contains six five-role bundles.

Across the six bundles, a0-04 never clears the oracle gate: one returns confident `no` 0.64/0.68; others are low confidence or unstable. a0-16 also never clears it. a0-02 clears in four bundles, fails with `no` in one, and has `yes` 0.60/0.50 in one. a0-15 clears in three bundles. This variability documents instrument sensitivity, including occasional confident disagreement. It does not make any answer an authority over the oracle. Neither exactness nor probability mass substitutes for two agreeing oracle-matching responses whose reported confidence is at least 0.6.

The supported CLI records expose answer objects, probabilities, confirmations, and request bindings, not semantic rationales. No direct attribution of these failures to a particular reasoning process, model revision, or hidden context is possible. Four other role questions were fixed in extension-05; that helps localize the experiment but does not eliminate joint-request interaction or evaluator variation.

## Independent semantic examination

| Cases | Documented convention and independent semantic boundary |
| --- | --- |
| a0-02: request logs/errors → request logger stdout/stderr | The same token prohibition and informal-to-formal modality are preserved. The old question explicitly permits channel formalization. But logs may be persisted outside standard streams, and error output need not be produced by the request logger. Conversely stdout/stderr may carry other output. Without an identified mapping, these substitutions can change governed surfaces or actor and omit an original surface. The statement is defensible under the frozen convention; equivalence is not established by the quote alone. |
| a0-04: print JSON on stdout → emit a JSON object on stdout | Channel and global command boundary are already explicit; the material difference is object shape. An object is a particular JSON container, while JSON need not have that shape. A permissible non-object output under the broader quote could fail the statement. This is an independent semantic reason to scrutinize the convention, irrespective of model answers. Absence of a specified shape is not, by itself, evidence that selecting one preserves the allowed outcomes. |
| a0-15: CLI output always JSON → every command emits JSON to stdout | The global CLI boundary is supported by quote/scope. Stdout is a conventional machine-output destination and is permitted by the old instrument, but not stated in the quote. If “output” also includes stderr, restricting the formal statement to stdout can omit an obligation; requiring stdout could also select a previously unspecified destination. Whether it is a faithful selected duty or a change needs the intended output-channel convention. |
| a0-16: commands print machine-readable JSON → emit JSON to stdout | Removing “machine-readable” from JSON and print→emit are ordinary formalization. Print→stdout is expressly allowed by the original rubric. This case has stronger conventional channel support than a0-15, but the universal channel assumption still comes from the documented convention rather than an explicit channel in its quote. Its poor answers do not independently establish a defective oracle. |

A task-specific language convention can legitimately define how informal requirements are normalized. It must be distinguished from general semantic entailment. The old instrument is internally aligned with these fixtures, but its blanket absence-of-distinction exception risks classifying actual precision as mere vocabulary. The JSON-shape issue is especially close to the forbidden algorithm selection: choosing one member of a broader allowed set can impose a substantive requirement even when the quote never distinguishes members. JSON terminology cannot be exempt merely because it appears less consequential than a cryptographic algorithm. The relevant test is whether the permitted behavior changes under the authorized domain convention.

This identifies a plausible boundary defect or insufficiently documented authority; it does not adjudicate a current label as wrong. The corpus currently operationalizes the permissive convention. Changing to strict entailment would be a versioned construct change, not correction justified solely by low scores.

## All frozen contrasts

The other original cases constrain what an explanation may claim. a0-01 and a0-03 preserve explicit artifact/path requirements. a0-05 and a0-13/14 intentionally extract one supported duty from compound provenance; exhaustiveness is not required. a0-06 preserves both duties and is faithful even though another role rejects bundling. a0-08 and a0-11 retain the original vagueness/private label; making them untestable is not invention. a0-12 preserves an unrecorded approval dependency without adding a record. a0-17 preserves the duty while deterministic grammar lint remains separate. a0-09 retains the privacy trigger and replacement action. These support role separation and ordinary requirement-language normalization without establishing blanket channel/shape equivalence.

a0-10 expands console output to standard streams and saved run files to artifacts, and instantiates a canary credential, bearer header, and environment-variable trigger. Its stated corpus rationale concerns concrete redaction test behavior. It therefore also relies on a test-instantiation/representation convention beyond literal quoted tokens; the exact environment name and canary value are not in the quote. That does not justify silently admitting arbitrary implementation precision. Any clarified normalization policy should explicitly address when concrete test instantiation is authorized and when it changes a duty.

a0-07 forbids selecting AES-256-GCM from generic encryption authority. The authorized and unauthorized algorithm supplements hold the same technical statement constant while supplying opposite explicit intent. They provide a sound structural contrast for the role of additional authority. All extension-05 bundles confidently reject the unsupported original and the explicitly unauthorized supplement; one rejects the explicitly authorized supplement and another lacks confidence on it. This shows residual intent-use limitations as well as normalization sensitivity. It does not establish that JSON-shape precision is harmless.

The common-predicate surfaces supplement preserves enumerated output surfaces; the independent-duties supplement preserves both quoted actions. They primarily constrain atomicity and supply no missing shape/channel authority. Both approval positives already name records in their quotes: an approvals.json decision and a signed checklist. They demonstrate recorded versus unrecorded approval boundaries without requiring invented records. None supplies authority for the contested output conventions. There is no frozen negative boundary contrast explicitly testing a genuine JSON-shape change or a conflicting channel requirement; broad generalization is therefore unverified.

## What would justify a new version

Before changing fixtures or supplying context, the responsible human/authority must explicitly decide the intended fidelity construct: domain-authorized normalization versus quote-level semantic preservation, including the limits of channel and container conventions. That decision needs provenance independent of the observed desired answers. Existing seal text documents the convention's existence; it does not alone supply all its substantive domain premises.

For a new version, the needed evidence is specific:

- For a channel convention: an approved, scoped definition of normal CLI output and error/log destinations, including whether persisted logs, diagnostics, actors, and stderr are covered or excluded. Mere scope names do not identify the mapping.
- For JSON shape: explicit authority that the relevant output contract requires an object, or an approved terminology convention that makes “JSON” mean that shape in a defined domain. Otherwise the distinction must remain visible for adjudication; a blanket exception cannot prove equivalence.
- For concrete redaction test instantiation: approval of which placeholder choices are representation/test data and which triggers or surfaces alter the requirement.
- For supplemental authority: exact approved content, its source, version and digest, and a recorded binding to the applicable generic fixture. Context must describe real intended semantics and be consistently available under the same rule, including contrasting cases; it must not contain answer cues or be fabricated to improve scores.

Any authorized revision should have a distinct fixture/context version, immutable predecessor, hashes, an explicit semantic rationale and change record, and independently reviewed contrasts protecting against both false invention and missed added policy. Freeze that evidence and candidate protocol before responses. Continue to score the current version under its current labels; do not retroactively rewrite its failures or adopt a partial bundle. A version change and subsequent acceptance are separate decisions.

The present evidence supports recording a bounded normalization-authority ambiguity and unresolved instrument/evaluator limitations. Further wording-only trials may test the frozen convention, but cannot resolve its substantive boundary by producing higher confidence. No PASS, new seal, production adoption, or readiness conclusion follows from this audit.

## Sources

All paths are local preserved evidence:

- `/tmp/plan-anchor-088-smoke-56bxrvbm/bd/.codex/skills/plan-anchoring/assets/fixtures/testability-calibration.v2.json`
- `/tmp/sc-runtime-phase-c-review/tuning/calibration-r1/live-authorized/seal-01/`: `sealed-rubric.json`, `candidates.json`, `candidate-rankings.json`, `diagnostic-role-selection.json`, `provenance.json`, generic request metadata.
- `/tmp/sc-runtime-phase-c-review/continuation/calibration/088-local-extension-05-prepared/`: `frozen-candidates.json`, `candidate-rationales.json`, `frozen-request-shapes.json`.
- `/tmp/sc-runtime-phase-c-review/continuation/calibration/088-local-extension-05-execution-01/`: `result.json`, `acceptance-table.json`, `all-cli-results.json`.
- `/tmp/sc-runtime-phase-c-review/continuation/calibration/088-local-extension-04-execution-01/result.json`.
