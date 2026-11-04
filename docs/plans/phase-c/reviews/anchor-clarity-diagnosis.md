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
