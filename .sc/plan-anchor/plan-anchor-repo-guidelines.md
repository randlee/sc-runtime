# Plan-anchor guidelines (sc-runtime)

Stated by the product owner (randlee), 2026-11-12, during the Phase D plan
gate. They are binding on every plan-anchoring step in this repository:
anchor capture, gate triage, plan fixes, question tuning and drift review.

Verbatim:

> Tighten as asked. Verification difficulty is not a reason to loosen.
> - Where anchors conflict, take the stricter reading.
> - It is not this project's scope to limit what a consumer chooses; the consumer assumes that risk.
> - A requirement says what is built; an open option is not a work item.

## How each one applies

- **Tighten as asked.** When the owner tightens a requirement, the plan and
  its tests take the tighter form. A finding that the tight form is hard to
  test or verify is not grounds to restore the looser one. Fix the test
  instead, or escalate.
- **Stricter reading.** When two anchors (or an anchor and a REQ/ADR) read
  differently on the same text, the plan satisfies the stricter one. Do not
  resolve the conflict by excluding the stricter anchor or tuning its question.
- **Consumer choice.** Dependency requirements, generated templates and
  published APIs do not restrict what a consumer may select. That includes
  exact or tilde pins and upper bounds without a demonstrated, named break.
  The consumer assumes the risk of a newer compatible release. A
  protective restriction is not a reason to pin. Where an ADR requires one,
  the plan amends the ADR rather than carrying an exception.
- **No open options.** Each plan item states what is built. "Optional",
  "may", "only if later qualified" and similar open work are not deliverables.
  Either make it a deliverable with a defined outcome for each result, or
  remove it.

Phase D example: the d-11 manifest test bans exact and tilde pins outright.
ADR-RUN-0017 and ADR-RUN-0203 pins are removed by amendment, and the rusqlite
0.42 widening is a d-6 deliverable.

Remove this file only if the owner withdraws these guidelines.

## Roles (owner ruling, 2026-11-12)

|  | Planner | Lead |
| --- | --- | --- |
| Goal | Get the anchors to converge | Make sure nothing drifted |
| Freedom | Change anything | Change nothing |
| Output | A plan that passes | Accept, or reject with the reason |

- The lead never diagnoses failures into fixes, dictates replacement wording or
  chooses sprint wiring. Those are the planner's. A lead that writes the fix
  ends up reviewing its own work, and the planner stops thinking.
- Exclusions are the planner's. Excluding an item from an anchor is another way
  to make a gate pass, so the drift checker must not hold that tool. The lead
  reviews each exclusion as possible drift: it accepts or rejects it.
- The lead runs the gates and the drift review against the owner's quotes and
  intent, and reports the results.
