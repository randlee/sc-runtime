---
id: a-8
phase: a
status: planned
closure_type: integration
target_boundary: wizard
vertical_rationale: "The copied Wyvern UI and its narrow driver entry point form one small producer/consumer pair over an already-settled JSON contract."
track: wizard
wave: 4
branch: sprint/a-8-wizard
worktree: sprint/a-8-wizard
recommended_agent: cipher
recommended_model: luna
must_follow: [a-6]
parallel_safe: [a-7]
requirements:
  - REQ-RUN-0401
  - REQ-RUN-0402
  - REQ-RUN-0403
  - REQ-RUN-0601
  - REQ-RUN-0602
  - REQ-RUN-0603
  - NFR-RUN-0004
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0008
  - ADR-RUN-0401
  - ADR-RUN-0402
owned_paths:
  - "wizard/wizard.json"
  - "wizard/pages/**"
  - "scripts/run_wizard.py"
  - "scripts/new_project.py"
  - "tests/unit/wizard/**"
  - "tests/integration/wizard/**"
  - "docs/generation.md"
---

# Sprint a-8 — wizard

## Outcome and closure

Copy the proven Wyvern example and adapt it to produce the already-settled
answers JSON. The wizard is a replaceable input UI, not part of the generated
application or reusable runtime.

## Decision prerequisites

The p3 reference revision must be readable. Resolve prefill, cancellation, and
finished-result behavior only where the copied implementation requires it.
Do not add application options or reopen the schema during UI work.

## Dependencies and parallel safety

- `must_follow a-6`: consumes the final schema, fixtures, and `--var-file`
  pipeline as its contract and oracle.
- `parallel_safe a-7`: it modifies no crate, template, release workflow, or
  publication metadata. Its narrow sequential ownership of driver/docs begins
  only after a-6 merges.

## Deliverables

1. **REQ-RUN-0601** — Copy the descriptor, pages, helper, and styles from the
   specified p3 revision; edit fields and lists rather than building a new UI
   framework.
2. **REQ-RUN-0602** — Add interactive and prefill entry modes that finish with
   the same validated answers object used by `--var-file`; cancellation creates
   no destination.
3. **REQ-RUN-0603** — Demonstrate fixture-equivalent answers and generated
   output without changing the core crates or template contract.
4. **Inherited constraint** — Preserve the a-6-owned REQ-RUN-0501/0502
   pipeline and non-interactive guarantees unchanged while adding the two
   wizard modes.

## Acceptance criteria

1. `req:REQ-RUN-0601` — the wizard exposes exactly the schema's current options
   and defaults, with no second independently maintained option model.
2. `req:REQ-RUN-0602` — interactive finish, prefill finish, and cancellation
   produce the documented driver outcomes.
3. `req:REQ-RUN-0603` — equivalent wizard and fixture answers produce equivalent
   generated project files, excluding documented nondeterministic metadata.
4. `ADR-RUN-0401` — no wizard asset is copied into generated projects and
   no reusable crate depends on Wyvern or UI code.

## This sprint does not close

It does not add runtime behavior, new template options, CLI auto-start,
attribute-generated tests, or generated application policy.

## Paths to delete

None.

## Required validation

Run descriptor/page unit tests, prefill/cancel tests, one interactive smoke,
fixture-equivalence generation, and the unchanged a-6 generation suite. Confirm
the crate and template trees are unchanged from their a-6/a-7 artifacts.
