# Optional rubric inputs — independent bounded audit

Repository: plan-anchor, worktree `codex/rubric-approved-inputs`, base `273780863780a86c79be7f59c2f37c06039ee3a6`. Core implementation was declared stable by its developer before conclusions; release/documentation metadata was still being finalized. This is not Phase C QA-3 or a merge/publication authorization.

## Disposition

No open findings in the requested optional-input scope after independently rechecking one corrected malformed-input defect.

- **RUBRIC-INPUT-001 — Minor — fixed.** The original `load_rubric_inputs` strict JSON reader accepted ASCII JSON containing an escaped lone surrogate, then UTF-8 hashing raised `UnicodeEncodeError` and returned nonrecoverable `SCRIPT.INTERNAL`. The reader at `plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/scripts/jev_judge.py.j2:2895` now verifies UTF-8 serialization with `allow_nan=False` and catches the error as `INPUT.MALFORMED`. Independent reproductions in both the approved-intent and sealed-question files now fail before credential lookup or transport. Overflowing numeric JSON is covered by the added regression as well.

## Checks performed

- Ten optional-input tests and two intent/legacy-triage contract tests passed independently: 12/12.
- The existing rubric regression class passed: 9/9, including `add --rubric`, no-option candidate/record forms, confirmation policy, deterministic lint and fixture handling.
- Additional independent mutations rejected missing criteria, extra criteria, duplicate sealed-JSON keys, a nonregular questions directory and an untrusted nonrepository root before credential lookup and transport.
- Existing and new tests directly exercise real repository-root lookup from a nested hint, external absolute paths, symlink escapes, missing files and stdin rejection. Optional inputs resolve beneath the actual selected Git root; paths are not based on the caller's arbitrary nested hint.
- Exact five-role sealed question set, strict choice/instruction/criteria shapes, digest integrity, duplicate keys and non-finite JSON refusals are enforced before credential or transport access.
- Explicit input request-size preflight covers live, dry-run and answer-fixture modes. Existing consent and confidence/confirmation rules remain enforced.
- Optional canonical intent is validated and compared against triage authority using canonical JSON bytes, including the `1` versus `1.0` representation distinction. Ordinary absent-intent triage remains compatible.
- Dry-run exposes the full exact request. Live provenance intentionally retains the canonical request digest, question seal and intent ID/version/content digest; the exact body is obtained from the paired dry-run or frozen inputs and checked against that digest. This metadata-only live-output behavior was confirmed with the developer rather than inferred as a missing feature.
- Windows CI already runs the existing suite's unguarded symlink tests. The new symlink test also runs unguarded. No evidence of a hosted-Windows privilege regression was found; no hypothetical symlink finding was raised. Local execution proves the tested host only.
- `git diff --check` passed. Reviewer made no repository changes, Git writes, live external judgments, releases or installs.

## Audited SHA-256

```json
{
  "plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/scripts/jev_judge.py.j2": "993a1a4ef061b219f0cd7d508dab1bc5fb68e590d50aa07057e82185ecc8203f",
  "plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/scripts/anchor_tuning_contract.py.j2": "91a39d2977167106a134c8bee043a11948d34a6a9c62907d143961720e437515",
  "plugins/sc-plan-anchor/install/templates/skills/plan-anchoring/assets/schemas/anchor-tuning-request.schema.json": "1e05228679d881debdb269514ab519e3b66b9778ade4ea902c4731f2400b1f99",
  "tests/sc-plan-anchor/test_rubric_inputs.py": "344a7120e63e0cdd67f1f68d422ac98b5216680ce9ea709d8027bfe8bf684647",
  "tests/sc-plan-anchor/test_anchor_tuning_contracts.py": "281781bd97eb8954588cfee64fb240ade2cd20210e38cad7624de953a305cd02"
}
```

The full suite and final release metadata remain the developer's separate validation responsibilities. A later core-byte change needs a delta recheck; this report does not claim a final moving-baseline PASS.
