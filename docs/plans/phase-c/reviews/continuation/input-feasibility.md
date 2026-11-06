# Plan Anchor 0.8.7 rubric input feasibility

Assessment date: 2026-11-02 America/Los_Angeles. Read-only production assessment; no source, installed asset, anchor, or question edits and no live calls were made.

## Finding

The supported 0.8.7 rubric CLI cannot supply the approved intent to the evaluator and cannot select an independently calibrated rubric from a consumer file. These are concrete input seams missing from the runtime. Better question wording alone cannot recover facts that were never sent.

Inspected upstream source: `/Users/randlee/github/plan-anchor-worktrees/codex/testability-triage-contract`, commit `273780863780a86c79be7f59c2f37c06039ee3a6`, VERSION `0.8.7`. Inspected installed runtime: `/Users/randlee/github/sc-runtime-worktrees/plan/phase-c/.codex/skills/plan-anchoring/scripts/jev_judge.py`, install lock 0.8.7.

## Evidence

Source paths below are relative to the upstream worktree.

- `plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/scripts/jev_judge.py.j2:2866` (`rubric_entries`) always calls `load_simple_question_set("testability", RUBRIC_QUESTIONS)`. That loader at line 2176 constructs the installed `assets/questions/testability.json` path internally; it has no override argument.
- The request construction at line 2890 puts only `statement`, `quote`, and `scope` in `state.anchor`. `cmd_rubric` at line 2924 selects those three values from the anchor record. It discards record `context`, document `intent_ref`, fixtures, and any approved clarification packet.
- The rubric parser at line 3353 exposes no intent or question file option. Reproduced with the installed script: `rubric --statement 'The API must emit JSON.' --quote 'emit JSON' --dry-run --questions unused.json` exits 2 with `VALIDATION.INPUT`, unrecognized `--questions`.
- Installed `rubric --anchors docs/plans/phase-c/anchors.json --id ANC-001 --dry-run` produced two request bodies, both with top-level keys `model,questions,state`, `state` containing only `anchor`, and `state.anchor` containing exactly the three fields above. No intent/context was sent. Captured read-only output: `/tmp/sc-runtime-phase-c-review/continuation-rubric-dry-run.json`. `data.pass` correctly remained false on dry run.
- General `judge --questions` is for plan/development questions, with required role mappings (`check_roles` at line 1545), and `build_state` at line 1603 constructs anchor statement/scope plus an item. It does not implement five-role rubric semantics or carry quote/intent. Recasting testability as a plan gate is not a supported equivalent.
- Low-level Python functions such as `call_jev`, `judge_with_policy`, and `rubric_entries` are script implementation details, not a documented alternate rubric API. Constructing arbitrary HTTP bodies there would bypass the missing supported input/provenance seam. No such workaround was used.
- Existing `load_json` at line 299 accepts arbitrary readable paths and stdin; the new optional authority/question inputs should instead reuse the stricter `canonical_workspace_path` helper at line 368, which resolves symlinks and rejects repository escapes.
- Existing `remote_preflight` at line 2150 preserves the recorded `allow_remote_judge` decision before live calls. Existing `fit_request` at line 1701 refuses oversized item-less rubric requests rather than shortening them. Both behaviors should remain.

R2 actual adjudications are consumer evidence, not new question labels chosen here:

- `docs/plans/phase-c/reviews/triage-087/r2/ANC-001-result/result.json` diagnoses `question_defect` on `faithful`: approved provenance authorizes the statement, but the actual request and question restrict the comparison to an abbreviated quote. The evaluator explicitly leaves the separately low-confidence `self_contained` issue unresolved and the original gate inconclusive.
- `docs/plans/phase-c/reviews/triage-087/r2/ANC-003-result/result.json` diagnoses `question_defect` on `single_constraint` from the pre-existing frozen contrast between one predicate on several surfaces and different independent behaviors. This is a wording/instrument distinction; it does not itself require extra runtime context. It authorizes reopening a tournament, not assigning a desired live answer.

## Smallest coherent proposal

Add two optional rubric inputs, leaving the existing zero-option behavior unchanged:

1. `rubric --approved-intent-file <repository-local JSON>` reads the existing authority intent shape `{id, version, sha256, content}`. Require exact fields, nonblank strings, an integer-or-nonblank-string version excluding bool, lowercase SHA-256, and `sha256 == SHA256(content UTF-8)`. Reuse the wire shape already used by `anchor-tuning-request.authority.intent`; do not invent a second authority model. Carry this exact object as optional `state.intent` beside `state.anchor`. Keep `state.anchor` unchanged, retaining quote provenance. An input being hash-valid does not establish human approval: the orchestration must select the already approved packet and cite its approval. Do not label an arbitrary file approved merely because a caller supplied it.
2. `rubric --questions <repository-local JSON>` reads an exact sealed rubric `{questions, digest}` produced from an independently accepted calibration result. Validate the five exact IDs, choice question structure, nonempty instructions and yes/no/unclear criteria, and `digest == SHA256(canonical JSON {questions})`. Reuse the existing seal convention in `anchor_tuning_contract.validate_questions`, including rejection of confidence/verdict meta-instructions. Refuse role mixing, extras, malformed/empty question bodies, digest mismatches, and nonfinite JSON numbers. The runtime must not silently rewrite role names, replace installed assets, or recalibrate. A seal proves question integrity; the separate accepted tournament/result evidence proves calibration. Document both requirements without adding a new approval ceremony or a large calibration framework.

Resolve each optional path to a regular file inside the actual repository found from `--repository-root`, resolving symlinks before reads. Do not let an arbitrary supplied directory broaden that root. No stdin or home/global-file fallback for these two options. Validate local inputs before remote credential lookup or transport. Dry-run and fixture modes remain offline and can show the complete context-bearing canonical requests. Do not silently follow `intent_ref`: explicit selection prevents accidental transmission of a full plan or unrelated context.

Record input provenance on each rubric entry, both dry-run and live: selected question seal, canonical request SHA-256, and optional intent ID/version/content digest. Persist these digests with original answers so triage can identify the exact request without relying on a mutable filename. Keep existing fields and outcomes; do not change 0.6 rubric confidence, two confirmations, disagreement handling, lint, retry/size policy, default corpus, or frozen labels. Content is included in the actual request and dry-run; metadata-only live output need not redundantly echo full intent.

The new context-bearing request needs a matching narrow triage check: if `canonical_request.state.intent` is present, it must be a valid intent descriptor, hash correctly, and equal `authority.intent` exactly. Reject a forged or different intent even when the anchor and question seal match. Continue allowing legacy requests with no `state.intent`; `review.source_digest` continues to hash only `state.anchor` for backwards compatibility. Optional state intent does not alter quote, record, or existing anchor digest conventions.

Limit these flags to `rubric` initially. `add --rubric` retains its existing default behavior and must not be represented as consuming a context-dependent calibration. Adding the same flags to `add` is a separate extension unless the actual consumer requires it.

## Exact source scope

Required implementation files:

- `plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/scripts/jev_judge.py.j2`: optional loaders, rubric parser, request construction, provenance result fields; preserve default behavior. Extend the simple loader only for rubric, so dedupe stays unchanged.
- `plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/scripts/anchor_tuning_contract.py.j2`: optional intent binding on testability triage; existing planning/development checks unchanged.
- `plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/assets/schemas/anchor-tuning-request.schema.json`: constrain optional `review.canonical_request.state.intent` to the existing intent descriptor shape only for testability triage. Schema and relational checks must agree; legacy no-intent requests remain accepted.
- `plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/references/question-tuning.md.j2`: explicit optional input selection, seal versus calibration distinction, request provenance and context behavior.
- `plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/references/jev-api.md.j2`: document exact state.intent wire shape and input/output digests in the existing API reference.
- `plugins/sc-plan-anchor/install/templates/agents/anchor-tuning.md.j2` and relevant existing tuning request/feedback templates: only short references describing context passed to actual runtime and preserving the same authority in triage. Do not duplicate the full contract or exceed current prompt caps.
- Normal next-unused patch metadata and changelog, then canonical installer render. Do not hand-edit installed consumer copies.

Proposed regression tests:

- `tests/sc-plan-anchor/test_jev_judge.py` / `RubricTests`: actual CLI parser acceptance; context present byte-for-byte in both confirmations; expected question seal really selects the consumer set; no flags preserves existing request shape/default set; candidate and anchors modes; no-op installed asset hashes.
- Strict negative files: missing/blank/malformed intent, incorrect digest, wrong type/bool version, extra authority fields; missing/extra rubric roles, wrong criteria/type, forged seal, prohibited metadata instructions; missing/nonregular/outside-root and symlink escape files. All rejected before remote transport.
- Fixture and dry-run paths need no key/consent and make no transport call; live false/unset consent refuses including context, true uses the existing provider/transport policy; no settings changes. Verify oversized context fails without truncation or partial send.
- Preserve two identical confirmations, third-call disagreement, low-confidence/inconclusive behavior, lint behavior, and existing fixture results. Test independent shared-predicate vs bundled fixtures as frozen inputs; do not encode an actual consumer's desired answer as a test oracle.
- `tests/sc-plan-anchor/test_anchor_tuning_contracts.py`: both schema and relational positive acceptance for optional exact intent; reject changed content/digest/id/version/extra fields or authority mismatch; retain all current no-intent/planning/development payloads and earlier binding-hole regressions.
- Both tracker full suites plus installer all-target rendering/version/ownership checks. Current baseline proof is 506 passing tests for each tracker; a new release needs its own post-change test proof.

## Disposition

There is no supported 0.8.7 command that completes this context-aware calibrated runtime evaluation without the input extension. The independent tournament can continue freezing/assessing questions and corpus; reserve runtime live acceptance until the approved context and sealed questions are actually in the serialized request. Source changes would be an upstream optional feature requiring review, a patch release/canonical installer upgrade, and consumer installation. This assessment makes no live gate PASS claim and requests no authority to merge or publish.
