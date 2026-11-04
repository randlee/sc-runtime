# Phase C anchor and fixture approval packet

Prepared from the six proposed entries in this sc-runtime worktree and the user's subsequent boundary-policy instructions. This packet is a proposal, not a gate verdict, a lock record, or fixture approval. Parent owns capture, locking, and tracker writes. No repository file or Beads record was changed by this preparation.

## Simplification proposal for the existing six anchors

The parent reports the live rubric found the original statements compound or insufficiently faithful to their shorter quotes. The statements below are proposed replacements, not edits already applied. Public router identity, SQLx compatibility, profile recipe names, and exact transitive package versions remain governed by existing REQ/ADR text and sprint acceptance criteria rather than being attributed to a quote that did not state them.

### ANC-001 — full consumer lint

Quote: “a full sc-lint is installed on a consumer repo”

Proposed replacement statement: A generated consumer repository must have the full sc-lint toolchain installed.

Pass fixture: Consumer setup installs the full sc-lint toolchain, including its boundary, portability, and runtime analyzers.

Fail fixture: Consumer setup installs only rustfmt and clippy and does not install the sc-lint toolchain.

### ANC-002 — current compatible ecosystem

Quote: “upgrading rmcp/utopia to the latest releave version should be planned in phase-<next>”

Proposed replacement statement: Phase C must upgrade rmcp and utoipa to their latest released versions.

Pass fixture: Phase C checks the released rmcp and utoipa versions and upgrades both dependencies to the latest releases.

Fail fixture: Phase C keeps the older rmcp and utoipa dependencies despite newer released versions being available.

### ANC-003 — sc-lint version

Quote: “both repo and templates should use 0.8.4”

Proposed replacement statement: The repository and generated template must both use sc-lint 0.8.4.

Pass fixture: Both the repository and generated template declare sc-lint 0.8.4 and their bootstrap commands install or verify 0.8.4.

Fail fixture: The repository bootstraps sc-lint 0.8.4 while the generated template declares and installs 0.8.0.

### ANC-004 — phase naming

Quote: “rename to phase-c”

Proposed replacement statement: The current phase must be named phase-c.

Pass fixture: Plan documents live under docs/plans/phase-c, sprint IDs are c-1 through c-4, and current branch, triage, and QA metadata use phase-c.

Fail fixture: The current plan, sprint IDs, branch metadata, triage, and QA evidence still use phase-bc and bc-1 through bc-4.

Kind: retain `structural`; a deterministic scan of the current Phase C artifacts is the evidence. The fixture pair tests the scan/expectation, not semantic Jev judgment. Historical provenance can accurately mention the old name; the current test must distinguish historical text from active identifiers.

### ANC-005 — isolated planning

Quote: “planning should be done on a second pr to develop on a dedicated $sc-git-worktree”

Proposed replacement statement: Phase C planning must use a dedicated worktree and a separate pull request to develop.

Pass fixture: Git worktree metadata shows a separate plan/phase-c checkout, and its dedicated planning pull request targets develop.

Fail fixture: Phase C planning is committed directly on the primary develop checkout and has no separate planning pull request.

Kind: existing record says `directive`; its truth is primarily Git/PR metadata rather than a sprint's promise. Recommend parent evaluate changing to `structural` before any gate baseline is frozen, while retaining statement/quote and using worktree inventory plus PR base/head evidence. Merely saying it in each sprint is weaker than the metadata. Do not silently change its kind to obtain a pass.

### ANC-006 — SQLite compatibility

Quote: “so rusqlite will remain at 0.39? I will have beads-cache downgrade so everything is consistent.”

Proposed replacement statement: Phase C must keep rusqlite on the 0.39 release line.

Pass fixture: Phase C retains rusqlite 0.39.x while upgrading the other approved dependencies.

Fail fixture: Phase C upgrades rusqlite from 0.39 to a newer minor release line.

## Missing boundary-policy candidates

### Candidate ANC-007 — configuration-selected default policy

Verbatim provenance, consecutive user messages:

> and you will need to determine the 'appropriate' default policy to install on consumer repos
>
> based on configuration settings

Proposed immutable quote: “and you will need to determine the 'appropriate' default policy to install on consumer repos”

Proposed context: The next consecutive user message specifies “based on configuration settings”; earlier the user says “yes, you need a boundaries policy”.

Proposed statement: Fresh consumer installation must generate a default boundaries policy selected from the consumer's configuration settings.

Scope: c-1's consumer policy contract, c-2's fresh-install implementation, and any item that selects or generates consumer boundaries policies.

Kind: directive.

Deterministic check: Generate consumers with representative supported configurations and assert that each emitted policy matches its selected configuration and is accepted by sc-boundary.

Pass fixture: Fresh consumer installation reads the selected configuration and generates the matching default boundaries policy before running sc-boundary.

Fail fixture: Fresh consumer installation emits no boundaries policy and runs sc-boundary only in no-manifest mode, regardless of configuration.

### Candidate ANC-008 — consumer ownership

Quote: “you will need to generate a boundaries policy for a 'fresh install' once a consumer is set up, they will have their own boundaries policy”

Proposed statement: After fresh setup, the generated boundaries policy must be owned by the consumer repository.

Scope: c-1's consumer policy ownership contract and c-2's generated policy handoff.

Kind: directive.

Deterministic check: The generated policy lives in the consumer repository and the documented setup contract identifies the consumer as its owner.

Pass fixture: Fresh setup writes the initial boundaries policy into the consumer repository and documents that the consumer owns subsequent policy decisions.

Fail fixture: The generated consumer's boundaries policy remains centrally owned by sc-runtime, which decides all subsequent consumer boundary changes.

This anchor does not invent a specific merge algorithm, upgrade preservation protocol, never-overwrite rule, or policy format. Any such implementation choice needs the existing plan's ownership contract and tests; the quote alone does not authorize imposing one.

## Provenance and overlap audit

- Every fixture in this packet was newly authored from the quoted instruction and proposed statement; none is already user-approved, and none is evidence of live Jev calibration.
- ANC-001/003 overlap REQ-RUN-0307, NFR-RUN-0006, and ADR-RUN-0005's consumer lint installation/version contract. These register texts currently include the old no-handwritten-boundaries restriction; c-1 is already assigned their amendment. Retain user anchors for provenance but import relevant governing REQ/ADR constraints as their original register IDs rather than inventing duplicate ANC entries.
- ANC-002 overlaps ADR-RUN-0203's one-library-type-universe constraint. User's latest-version instruction adds a selection constraint, so it is not identical to the current ADR.
- ANC-004 overlaps the naming guidelines and quality-policy convention. Existing ANC-004 records the explicit user override to `c` and remains valuable; deterministic checks must respect that override.
- ANC-005 overlaps the worktree workflow. Its dedicated plan PR constraint is user provenance, and actual Git/PR evidence is appropriate.
- ANC-006 carries a user-approved version decision; retain it and pair with the existing resolver guard rather than adding a second SQLite anchor.
- ANC-007/008 are not duplicates of the six existing anchors: full sc-lint can run with no boundary manifests, and configuration selection does not imply consumer ownership. They address the explicit policy instruction that the old no-manifest plan violates.
- ANC-007 and ANC-008 should remain separate: a policy could be correctly selected but centrally owned, or consumer-owned but incorrectly selected.

## Parent execution commands after approval

Run in the Phase C worktree; use the installed script `.codex/skills/plan-anchoring/scripts/jev_judge.py` and `docs/plans/phase-c/anchors.json`.

1. Run `rubric --anchors docs/plans/phase-c/anchors.json --all` (parent already started this). Preserve rubric outputs.
2. Rubric each new candidate with `rubric --statement '<statement above>' --quote '<quote above>' --scope '<scope above>'` before proposing/locking. Pass the adjacent configuration-setting utterance as explicit provenance in the record's context; never silently concatenate two messages as a fake verbatim quote.
3. After user approval, use `update --anchors ... --id ANC-00N --fixture-pass '<approved text>' --fixture-fail '<approved text>'` for each existing anchor. Use the supported lifecycle command from `--help` to lock existing proposals; do not hand-edit statuses.
4. After approval, use `add --anchors ... --quote ... --statement ... --scope ... --deterministic-check ... --kind directive --status locked` for each new candidate, then `update` for its fixtures. Preserve original quotes exactly.
5. Run `lint`, `validate`, and `render`; create anchor/question bead trees and record `set-beads` IDs in the main session as the installed bead-layout reference requires.
6. Freeze the approved fixtures before independent question tuning or live gate judgments. Delegate the tuner using the installed runner contract; parent must not author overrides to make current plans pass.
7. After hardening, delegate one full sprint file per planning gate. Compute coverage in code and record structural metadata checks. A JSON validation or dry run is not a gate PASS.

No live Jev call or Beads mutation was performed by this preparation task.
