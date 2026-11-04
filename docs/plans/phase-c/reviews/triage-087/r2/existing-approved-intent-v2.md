Approval provenance supplied by the parent from this human conversation:
1. The assistant-authored packet anchor-proposal.md was presented for approval of its statements and fixtures. The human answer was exactly: "Approve statements and fixtures".
2. The assistant-authored packet anchor-clarity-diagnosis.md was presented by a question listing clarity corrections and split fixtures, while stating the shared 0.8.4 rule stays intact. The human answer was exactly: "Approve clarity corrections and split fixtures".
These human approvals authorize the applicable statements, scope corrections, and fixtures. The packets remain verbatim assistant-authored drafts below: their pre-approval wording is historical, and analytical opinions do not become independent human facts or requested adjudication outcomes merely because the changes were approved.
Additional original human provenance for ANC-001, verbatim: "do we install sc-lint as part of the template w/ pre-populated boundary lint and cros-platform lint and runtime lint set up by default?" This records the existing question; it asserts no new requirement beyond the approved fixture and clarity correction.
Current locked anchor records in this request are the approved authoritative versions. Their original quotes are unchanged.

--- Verbatim existing packet: anchor-proposal.md ---
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

--- End packet: anchor-proposal.md ---

--- Verbatim existing packet: anchor-clarity-diagnosis.md ---
# Phase C anchor clarity diagnosis and proposed approval packet

Read-only diagnosis of the eight approved locked anchors against `/tmp/sc-runtime-phase-c-review/calibrated-anchor-rubric.json`. No tuning, live calls, repository changes, question overrides, or gate PASS is claimed.

The JSON stores raw requests and responses, not per-anchor `verdict` fields. Below, a “failure” means a role response cannot satisfy the required two agreeing answers at confidence >= 0.6. Several failures reflect genuine incomplete definitions, while others are low-confidence answers to ordinary technical language; confidence alone does not establish which instrument or statement is wrong.

## Diagnosis independent of obtaining a yes

| Anchor | Observed response issue | Semantic diagnosis | Minimal action |
|---|---|---|---|
| ANC-001 | `observable` changes unclear→yes at .11; `self_contained` changes no→yes at .16–.20 | “full sc-lint toolchain” leaves the decisive installed components undefined inside the statement. The already approved pass fixture supplies exactly the three analyzers. This is a real standalone clarity gap. | Replace the shorthand with the approved analyzer list, retaining one installation outcome. |
| ANC-002 | `single_constraint` no .28/.37, `faithful` no .43/.44, `observable` yes .02/.19 | rmcp and utoipa can be upgraded independently, so this is genuine bundling. “latest” is temporal but not intrinsically unobservable; the existing c-1 query/frozen-matrix process provides the selection time. The quote's “should be planned” versus “must upgrade” strength was explicitly approved in the previous packet; a new interpretation must not silently revert that approval. | Split the two package duties with the same immutable source quote. Preserve the approved upgrade meaning and latest-release instruction. |
| ANC-003 | `single_constraint` no .53/.48; other roles yes above .6 | One version equality rule applied to repository and template is permitted by the rubric's own instruction (“possibly several output surfaces”). A repository-only implementation is insufficient, but that does not necessarily make the shared equality invariant two unrelated obligations. This looks like a plausible instrument false rejection, not demonstrated anchor ambiguity. | Retain the approved statement. Optional “Both sc-lint installations—the repository and generated template—must be version 0.8.4” restates the shared invariant but is not necessary. |
| ANC-004 | all yes; `self_contained` .58/.52 | “current phase” is deictic when detached from its scope, though its structural deterministic check already identifies the concrete artifacts. | Name “sc-runtime's Phase C” explicitly; retain structural kind and verify current naming in metadata. |
| ANC-005 | `single_constraint` no .64/.67; `self_contained` yes .22/.19 | Worktree isolation and separate PR targeting develop are independently enforceable: either can hold without the other. This is genuine bundling. | Split into two structural anchors; check actual worktree and PR metadata separately. |
| ANC-006 | `self_contained` alternates no/yes/no .16–.22; observable/scoped/single yes strongly; faithful .53/.64/.53 | “rusqlite”, “0.39”, and a Cargo release line are ordinary technical terms, not private shorthand. No independently missing condition is shown. The quote is a question, but the approved packet and later user acknowledgment establish the decision. | Keep the requirement; optionally spell its Cargo constraint as `0.39.x` without importing transitive SQLite rules into the quote. |
| ANC-007 | all yes; faithful .38/.44, observable .26/.35, self-contained .28/.28 | “configuration settings” does not identify which settings drive policy or what “matching” means. The user approved the relational requirement, but the precise supported configuration-to-policy rule is an output of c-1. This is a real contract-detail question, not a license to invent a policy matrix. | Preserve the anchor as that relational requirement; require c-1 to define the mapping. A wording replacement alone cannot supply the missing configuration contract. |
| ANC-008 | all yes; self-contained .43/.34 | Ownership may mean file location, modification rights, lifecycle decisions, or central maintenance responsibility. The approved pass fixture makes the intended policy-decision meaning explicit. | Replace “owned” with the approved subsequent consumer policy-decision responsibility. |

## Exact proposed changes for user approval

These are proposals only. Preserve every existing quote byte-for-byte. A split adds a new anchor ID and retires/replaces the bundled record through the installed lifecycle commands; never silently erase the old approval or reuse an ID to mean only half its previous duty.

### ANC-001 clarity replacement

Current: “A generated consumer repository must have the full sc-lint toolchain installed.”

Proposed: “Generated consumer setup must install sc-lint with its sc-boundary, sc-portability, and sc-runtime analyzers.”

Pre-existing authority: quote “a full sc-lint is installed on a consumer repo”; user also asked for “pre-populated boundary lint and cros-platform lint and runtime lint set up by default”; approved pass fixture explicitly lists “boundary, portability, and runtime analyzers.” The replacement defines the installed set; it adds neither profile names nor a receipt mechanism.

Proposed scope: “c-2 generated consumer setup and any plan item that installs the consumer lint toolchain.” This removes command/profile signals unrelated to the narrowed installation statement. Scope change requires approval.

Fixtures: retain the approved current pair; they already distinguish installing this analyzer toolchain from installing only rustfmt/clippy.

### ANC-002 split proposal

Duty A: “Phase C must upgrade rmcp to its latest released version.”

Duty B: “Phase C must upgrade utoipa to its latest released version.”

Pre-existing authority for both: unchanged quote “upgrading rmcp/utopia to the latest releave version should be planned in phase-<next>”; user approved the corrected package spelling and upgrade requirement in the previous packet. Separate package obligations prevent one upgrade from masking omission of the other. Do not add utoipa-axum as if present in this quote; its compatibility requirement remains in the governing runtime ADR/plan and other user context.

Scope for both: retain “the c-1 compatibility contract and c-3 runtime ecosystem upgrade sprints.”

Fixture projection proposal (requires approval): A-pass “Phase C checks the released rmcp versions and upgrades rmcp to its latest release.” A-fail “Phase C keeps the older rmcp dependency despite a newer released version being available.” B-pass/B-fail are the same sentences with utoipa substituted. These are exact one-package projections of the previously approved two-package fixtures.

Timing: c-1's already planned query determines the current release selection and freezes its exact version. Do not append “latest forever” or require an unbounded stream of new upgrades after that contract freeze.

### ANC-003 retain

Retain statement, scope, and fixtures. Do not split a shared version invariant solely because one confidence measure rejected it. If the independent evaluator concludes the common constraint is truly compound, present that rationale with a proposed split; this diagnosis does not supply it.

### ANC-004 clarity replacement

Current: “The current phase must be named phase-c.”

Proposed: “sc-runtime's Phase C must use the identifier phase-c.”

Pre-existing authority: unchanged quote “rename to phase-c”; approved fixture identifies the sc-runtime plan paths and c-1…c-4 metadata. The change resolves a deictic subject without changing phase identity. Retain existing scope and structural kind.

### ANC-005 split proposal

Duty A: “Phase C planning must use a dedicated Git worktree.”

Duty B: “Phase C planning must have a separate pull request targeting develop.”

Pre-existing authority: unchanged quote “planning should be done on a second pr to develop on a dedicated $sc-git-worktree”; the approved pass fixture separately asserts Git worktree metadata and a dedicated planning PR to develop. Both are structural checks over actual state, not Jev judgments of a sprint's promise.

Scopes: A “the Phase C planning branch and Git worktree metadata”; B “the Phase C planning pull request metadata.” Both scope changes require approval.

Fixture projection proposal (requires approval): A-pass “Git worktree metadata shows a dedicated Phase C planning checkout”; A-fail “Phase C planning is performed in the primary develop checkout.” B-pass “A separate Phase C planning pull request targets develop”; B-fail “Phase C planning has no separate pull request to develop.”

### ANC-006 retain, optional notation clarification

Current meaning is adequate. If the user wants notation clarification, proposed replacement: “Phase C's rusqlite dependency must remain at version 0.39.x.”

Pre-existing authority: unchanged original quote names 0.39; approved pass fixture explicitly says rusqlite 0.39.x. This changes no version boundary. Retain existing fixtures. Do not claim libsqlite3-sys 0.37 or the whole unchanged graph came from this quote; those remain separately governed compatibility constraints.

### ANC-007 contract clarification, no invented mapping

Retain the approved statement and fixtures. Its two consecutive original user messages already say the default policy is selected “based on configuration settings.”

The observed gap is the missing concrete mapping: the c-1 plan must identify the actual consumer generation configuration inputs and how enabled crates/roles change the initial policy. The approved user instruction authorizes determining that mapping, rather than asking the user to design it. The existing full-plan hardening developer owns that detail. Do not add an exact list of configuration fields to this anchor before that list has a reviewed source.

If a lexical clarification is wanted, “Fresh consumer setup must generate the default boundaries policy for the selected consumer configuration” preserves the existing relational requirement, but by itself does not cure a missing c-1 configuration contract. This optional wording is not a promised rubric fix.

### ANC-008 clarity replacement

Current: “After fresh setup, the generated boundaries policy must be owned by the consumer repository.”

Proposed: “After fresh setup, the consumer repository must control subsequent boundaries-policy decisions.”

Pre-existing authority: unchanged quote ends “once a consumer is set up, they will have their own boundaries policy”; approved pass fixture states “the consumer owns subsequent policy decisions”; approved fail fixture assigns those subsequent decisions centrally to sc-runtime. The replacement names the actual ownership right already approved. It does not impose a merge algorithm, never-overwrite rule, or new preservation workflow.

Retain scope and approved fixtures, which already assert this ownership meaning.

## Instrument limitation versus plan changes

The current results do not justify rewriting all eight anchors until they pass. ANC-003's shared-invariant rejection and ANC-006's low-confidence ordinary technical terminology are specifically plausible instrument limitations. ANC-004/005 are structural and never need semantic planning judgments, although their pre-gate testability process remains distinct.

Apply only clarity changes independently justified above after user approval; keep the existing locked versions and gate evidence in history. Re-run the proper testability workflow on affected anchors without lowering 0.6, averaging, majority voting, or borrowing the tournament's threshold-free semantics. If results stay inconclusive despite a clear supported requirement, report the instrument limitation for independent adjudication rather than repeatedly changing approved intent.

--- End packet: anchor-clarity-diagnosis.md ---
