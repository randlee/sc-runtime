# Testability triage preparation — schema blocked

Prepared exact evidence drafts for ANC-001 `faithful` and ANC-003 `single_constraint`. These are **not ready for strict tuner dispatch** because the installed 0.8.6 request schema cannot represent testability triage.

Each `<ID>-request.md` was rendered with the installed triage template using `--strict --check-render`. Its fenced object equals `<ID>-request.json` exactly. Both inner requests pass `anchor_tuning_contract.py request` with `data.valid: true`, because that relational validator's `ROLE_IDS` supports testability.

However, both fail the installed request JSON Schema for two independent reasons:

1. The triage branch at `anchor-tuning-request.schema.json:125` restricts `gate` to `planning` or `development`, despite the top-level enum including `testability`.
2. `sealed_question_set.questions` points to `productionQuestions`, whose `additionalProperties: false` only permits planning/development roles. The actual sealed rubric's `faithful`, `observable`, `scoped`, `self_contained`, and `single_constraint` roles are rejected.

The result schema also restricts `triageData.gate` to planning/development and its sealed question representation similarly uses production roles. A strict result would therefore hit the analogous schema limitation.

Do not bypass by relabeling the actual gate as planning, dropping rubric roles, substituting different questions, or accepting only relational validation. The installed agent requires both schema and relational validation. A schema/runtime contract correction belongs in plan-anchor, with tests and a canonical installer update; no schema or runtime was edited here.

## Evidence integrity

- `sealed_question_set.digest` was recomputed from canonical JSON `{"questions": <exact request questions>}` and matches `96d8f74cf022d7515aaa0d4a8f4e103b9be8c7b776bf99794d9926ef13d8fdb4` in both cases.
- `review.canonical_request` is the complete original actual-anchor rubric request from `calibrated-anchor-rubric-clarity.json`, unchanged.
- `review.raw_response` is the exact first response object for the representative role; all repeated responses and original request are retained unchanged in `<ID>-repeat-evidence.json`.
- `review.source_digest` is SHA-256 of canonical JSON for the exact source object `canonical_request.state.anchor`, including the actual statement, quote, and scope sent. This binds the judged source, rather than substituting the updated authority record or a filesystem path.
- The authority contains the current locked anchor record, approved fixtures, canonical record digest, and the user's exact checklist-completion intent. It includes no active plan or requested disposition.
- Planner observation is marked untrusted and requests independent examination; it names no favored answer.
- `<ID>-invocation-fence.md` retains the immutable Codex Sol tuner descriptor and exact draft params. Parent should dispatch only after the schema/contract limitation is legitimately resolved and script/contract hashes are refreshed to the installed corrected version.

No live calls, adjudication, tuning, authority edits, repository edits, or gate PASS occurred in this task.
