# Local unreleased 0.8.8 calibration preparation

These inputs are staged for independent calibration against an unreleased
canonical installer local test package from Plan Anchor PR #62, source commit
`b410ceab3b737785e85d2084151168962af5f1f8`. The local bd runtime script digest is
`a99b50920aa3516df2df09673f6e924166ab0390d98bd0c8a02bfb8cba6d84e3`.
The source audit and smoke report retain their exact historic scope and bytes.

This directory does not establish a published release or a Phase C consumer
installation. Repository settings, installed tooling, anchors and questions
remain unchanged. The five inputs were copied byte-for-byte from the frozen
materialization plan and verified before independent execution. A live result
is still pending; preparation proves no new seal or planning-gate PASS.

The initial independent dry-run rejected the three question-file wrappers:
`RUBRIC.QUESTIONS`, `sealed rubric requires exactly questions and digest`.
Exact refusal evidence is retained in `rejected-wrapper-materialization/`;
there were no live calls. The original committed inputs are preserved.

The additive `r2/` correction supplies the required `{questions,digest}`
wrappers. Question contents, intent files, oracle and frozen wire bodies are
unchanged. All three corrected question files with each of the two intent
variants passed the supported CLI `--dry-run`; all six emitted requests
exactly match the supplied frozen questions and intent descriptors.
`r2/dry-run-validation.json` records the checks and request digests. This
serialization correction claims no live judgment or accepted calibration.
Upstream test-only successor `c313d007` retains the same local runtime script
bytes; the smoke package's original source remains `b410ceab`.

The additive `r3/` bindings distinguish the existing redacted CLI presentation
from the unredacted canonical request digest. All 63 supported CLI dry-runs
matched the existing redaction of frozen requests, and provenance digests
matched their original canonical bodies. Only the three copies of the generic
Bearer-display fixture differed in presentation; frozen inputs were unchanged.

The independent local execution in `attempt-01/` returned `success:true` and
`accepted:false`, with 126 raw answer confirmations. Each candidate passed all
20 supplement cells. The best candidate matched and repeated all 85 original
cells, but three lacked confident pairs: faithful/global scope,
observable/unrecorded approval, and faithful/duplicate second. No seal was
accepted. Source reports retain their exact explanations and historical names,
including the diagnostic sealed-rubric filename; that filename grants no seal.

The parent approved one evidence-backed retry after the agent-context automatic
review rejection; payloads were generic calibration cases. No threshold,
oracle label or fixed-role question was changed. An independent append-only
candidate extension is being prepared separately. Upstream PR #62 is at
`b113bfa`, with 518 full-suite tests reported per tracker and unchanged runtime
`a99b5092`; this checkpoint remains a local unreleased test, not Phase C
readiness, a consumer installation or a live planning-gate PASS.
