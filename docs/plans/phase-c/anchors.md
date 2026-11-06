# Anchors - phase-c

Constraints captured during planning. Each anchor is a testable statement written by the orchestrator and approved by the user; the user's verbatim words are kept beneath it as provenance. Every anchor is judged at the planning gate and again at the development gate. Edit `anchors.json`, not this file; this file is rendered.

Intent: `docs/plans/phase-c/phase-c-plan.md`

| ID | Status | Summary | Stated |
| --- | --- | --- | --- |
| ANC-001 | locked | Generated consumer runs every configured analyzer | 2026-11-02 |
| ANC-002 | retired | One compatible runtime ecosystem upgrade | 2026-11-02 |
| ANC-003 | locked | Repository and template share sc-lint 0.8.4 | 2026-11-02 |
| ANC-004 | locked | Phase C uses c consistently | 2026-11-02 |
| ANC-005 | retired | Dedicated Phase C planning worktree and PR | 2026-11-02 |
| ANC-006 | locked | Keep rusqlite 0.39.x SQLite compatibility | 2026-11-02 |
| ANC-007 | locked | Configuration-selected default boundaries | 2026-11-02 |
| ANC-008 | locked | Consumer-owned boundaries policy | 2026-11-02 |
| ANC-009 | locked | upgrade rmcp to its latest released version. | 2026-11-02 |
| ANC-010 | locked | upgrade utoipa to its latest released version. | 2026-11-02 |
| ANC-011 | locked | use a dedicated Git worktree. | 2026-11-02 |
| ANC-012 | locked | have a separate pull request targeting develop. | 2026-11-02 |
| ANC-013 | locked | Compatibility precedes publication | 2026-11-03 |
| ANC-014 | locked | Optional standard cron commands | 2026-11-03 |
| ANC-015 | locked | Shared cron JSON contract | 2026-11-03 |
| ANC-016 | locked | Common schedule-management subcommands across handlers | 2026-11-03 |
| ANC-017 | locked | Supported consumer overrides | 2026-11-03 |
| ANC-018 | locked | Documented integration and override responsibilities | 2026-11-03 |

## ANC-001

Anchor: Generated consumer setup must install sc-lint with its sc-boundary, sc-portability, and sc-runtime analyzers.

> a full sc-lint is installed on a consumer repo

Said by randlee, Phase C planning conversation, 2026-11-02. Status: locked. Version 4.

Summary: Generated consumer runs every configured analyzer

Scope: c-2 generated consumer setup and any plan item that installs the consumer lint toolchain.

Deterministic check to pair with the development gate: A generated fixture runs just setup, ordinary just lint, and named default and full profiles while its receipt records sc-boundary, sc-portability, and sc-runtime.

Excluded from (not judged; the cell passes as `excluded`):
- `c-3`: C-3 upgrades runtime/projection code; consumer lint toolchain installation is owned by C-2.
- `c-4`: C-4 verifies runtime composition/publication; it does not install the consumer lint toolchain owned by C-2.
- `c-5`: C-5 implements cron behavior and consumer examples; it does not install lint tools.
- `c-6`: C-6 is the command-adapter boundary, not consumer lint installation.
- `c-1`: C-1 qualifies the released lint tool in an internal temporary probe but does not ship consumer installation, which is owned by C-2.

Beads: anchor `run-32b`, plan questions `run-32b.1`, dev questions `run-32b.2`

Fixtures:
- pass: Consumer setup installs the full sc-lint toolchain, including its boundary, portability, and runtime analyzers.
- fail: Consumer setup installs only rustfmt and clippy and does not install the sc-lint toolchain.

## ANC-002

Anchor: Phase C must upgrade rmcp and utoipa to their latest released versions.

> upgrading rmcp/utopia to the latest releave version should be planned in phase-<next>

Said by randlee, Phase C planning conversation, 2026-11-02. Status: retired. Version 3.

Summary: One compatible runtime ecosystem upgrade

Scope: the c-1 compatibility contract and c-3 runtime ecosystem upgrade sprints

Deterministic check to pair with the development gate: Cargo metadata for the runtime and generated daemon composition graph reports one reachable package identity for rmcp, utoipa, and utoipa-axum.

Beads: anchor `run-dle`, plan questions `run-dle.1`, dev questions `run-dle.2`

Fixtures:
- pass: Phase C checks the released rmcp and utoipa versions and upgrades both dependencies to the latest releases.
- fail: Phase C keeps the older rmcp and utoipa dependencies despite newer released versions being available.

## ANC-003

Anchor: The repository and generated template must both use sc-lint 0.8.4.

> both repo and templates should use 0.8.4

Said by randlee, Phase C planning conversation, 2026-11-02. Status: locked. Version 2.

Summary: Repository and template share sc-lint 0.8.4

Scope: the c-1 consumer contract and c-2 template lint profile sprints

Deterministic check to pair with the development gate: The root and template sc-lint configuration require 0.8.4, and each bootstrap path installs or verifies that version.

Excluded from (not judged; the cell passes as `excluded`):
- `c-3`: This anchor scope explicitly names C-1 and C-2; C-3 does not select the repository/template sc-lint version.
- `c-4`: This anchor scope names C-1/C-2 sc-lint version selection, not C-4 runtime publication.
- `c-5`: The sc-lint version anchor explicitly governs C-1/C-2, not C-5 cron code or fixtures.
- `c-6`: The sc-lint version scope explicitly names C-1/C-2, not C-6.

Beads: anchor `run-83r`, plan questions `run-83r.1`, dev questions `run-83r.2`

Fixtures:
- pass: Both the repository and generated template declare sc-lint 0.8.4 and their bootstrap commands install or verify 0.8.4.
- fail: The repository bootstraps sc-lint 0.8.4 while the generated template declares and installs 0.8.0.

## ANC-004

Anchor: sc-runtime's Phase C must use the identifier phase-c.

> rename to phase-c

Said by randlee, Phase C planning conversation, 2026-11-02. Status: locked. Version 3.

Summary: Phase C uses c consistently

Scope: every Phase C plan item and its plan metadata

Deterministic check to pair with the development gate: A targeted repository scan of Phase C assets reports no phase-bc, bc-<n>, or ARCH-BC identifiers.

Beads: anchor `run-xiw`, plan questions `run-xiw.1`, dev questions `run-xiw.2`

Fixtures:
- pass: Plan documents live under docs/plans/phase-c, sprint IDs are c-1 through c-4, and current branch, triage, and QA metadata use phase-c.
- fail: The current plan, sprint IDs, branch metadata, triage, and QA evidence still use phase-bc and bc-1 through bc-4.

## ANC-005

Anchor: Phase C planning must use a dedicated worktree and a separate pull request to develop.

> planning should be done on a second pr to develop on a dedicated $sc-git-worktree

Said by randlee, Phase C planning conversation, 2026-11-02. Status: retired. Version 3.

Summary: Dedicated Phase C planning worktree and PR

Scope: the Phase C plan branch, worktree, and pull request metadata

Deterministic check to pair with the development gate: Git worktree metadata records plan/phase-c separately, and its pull request targets develop.

Beads: anchor `run-aov`, plan questions `run-aov.1`, dev questions `run-aov.2`

Fixtures:
- pass: Git worktree metadata shows a separate plan/phase-c checkout, and its dedicated planning pull request targets develop.
- fail: Phase C planning is committed directly on the primary develop checkout and has no separate planning pull request.

## ANC-006

Anchor: Phase C's rusqlite dependency must remain at version 0.39.x.

> so rusqlite will remain at 0.39? I will have beads-cache downgrade so everything is consistent.

Said by randlee, Phase C planning conversation, 2026-11-02. Status: locked. Version 3.

Summary: Keep rusqlite 0.39.x SQLite compatibility

Scope: the c-3 runtime ecosystem upgrade sprint and any plan item that changes Cargo dependency versions for rusqlite, sqlx, or libsqlite3-sys

Deterministic check to pair with the development gate: Cargo metadata and the SQLite resolver test show rusqlite 0.39.x and libsqlite3-sys 0.37 with no libsqlite3-sys 0.38.

Excluded from (not judged; the cell passes as `excluded`):
- `c-2`: C-2 installs lint tools and consumer policies; it does not change rusqlite, sqlx or libsqlite3-sys dependency versions.
- `c-4`: C-4 consumes the frozen dependency graph and owns no manifest selecting or changing rusqlite/sqlx/libsqlite3-sys versions.
- `c-6`: C-6 owns adapter source/tests/docs and no manifest changing rusqlite/sqlx/libsqlite3-sys versions; C-1/C-3 own that baseline.

Beads: anchor `run-ioz`, plan questions `run-ioz.1`, dev questions `run-ioz.2`

Fixtures:
- pass: Phase C retains rusqlite 0.39.x while upgrading the other approved dependencies.
- fail: Phase C upgrades rusqlite from 0.39 to a newer minor release line.

## ANC-007

Anchor: Fresh consumer installation must generate a default boundaries policy selected from the consumer's configuration settings.

> and you will need to determine the 'appropriate' default policy to install on consumer repos
based on configuration settings

Said by randlee, User-approved packet; ANC-007 quote contains two consecutive verbatim user messages separated by a newline. Statements and fixtures approved in this chat. Approved C1 contract-stage clarification (2026-11-06): C1 fulfills its contract-stage obligation by delivering a checkable contract that defines how selected configuration determines boundary owners and permitted dependency edges, omits disabled components, and qualifies the contract with the representative consumer-shaped configuration required by C1. C1 need not ship C2's bootstrap or claim C2's full matrix. C2 still must implement real fresh-install policy generation from settings and pass the approved supported-configuration/edge matrix and real analyzer checks. A future promise without the specified contract and qualification does not fulfill C1. Approval record: docs/plans/phase-c/reviews/authority-approvals/phase-c-stage-policy-and-calibration-approved-2026-11-06.json., 2026-11-02. Status: locked. Version 3.

Summary: Configuration-selected default boundaries

Scope: c-1 consumer policy contract and c-2 fresh-install implementation

Deterministic check to pair with the development gate: Generate supported consumer configurations and verify their emitted policy matches selected crates and edges.

Excluded from (not judged; the cell passes as `excluded`):
- `c-3`: Default consumer boundaries-policy generation is owned by C-1/C-2; C-3 does not implement it.
- `c-4`: Fresh-install boundaries-policy generation is C-1/C-2 work, not C-4 runtime composition.
- `c-5`: C-5 does not generate consumer boundaries policies; that is C-1/C-2 scope.
- `c-6`: C-6 does not implement fresh-install boundaries-policy generation.

Beads: anchor `run-a7e`, plan questions `run-a7e.1`, dev questions `run-a7e.2`

Fixtures:
- pass: Fresh consumer installation reads the selected configuration and generates the matching default boundaries policy before running sc-boundary.
- pass: C1 delivers the selected-crate/edge mapping contract and a representative qualification showing the selected consumer policy; the shipped generator and full installation matrix remain explicitly owned by C2.
- fail: Fresh consumer installation emits no boundaries policy and runs sc-boundary only in no-manifest mode, regardless of configuration.
- fail: C1's contract merely promises appropriate boundaries later and neither defines the configuration-to-owner/edge mapping nor supplies its required representative qualification.

## ANC-008

Anchor: After fresh setup, the consumer repository must control subsequent boundaries-policy decisions.

> you will need to generate a boundaries policy for a 'fresh install' once a consumer is set up, they will have their own boundaries policy

Said by randlee, User-approved packet; ANC-007 quote contains two consecutive verbatim user messages separated by a newline. Statements and fixtures approved in this chat. Approved C1 contract-stage clarification (2026-11-06): C1 fulfills its contract-stage obligation by explicitly defining the initial policy handoff and the consumer's authority over all subsequent boundary-policy decisions, with checkable contract/probe evidence required by C1. C1 need not ship the C2 handoff implementation. C2 still must write the policy into the consumer repository and verify the approved preservation/ownership behaviors. The narrow existing successful-customized-native-init waiver remains unchanged; it does not waive consumer control or silently permit overwriting policy. Approval record: docs/plans/phase-c/reviews/authority-approvals/phase-c-stage-policy-and-calibration-approved-2026-11-06.json., 2026-11-02. Status: locked. Version 4.

Summary: Consumer-owned boundaries policy

Scope: c-1 consumer policy ownership contract and c-2 generated policy handoff

Deterministic check to pair with the development gate: Generated policy lives in the consumer repository and documented setup identifies the consumer as owner.

Excluded from (not judged; the cell passes as `excluded`):
- `c-3`: Consumer boundaries-policy ownership is explicitly scoped to C-1/C-2, not C-3 runtime upgrades.
- `c-4`: The existing policy-ownership scope names C-1/C-2; C-4 owns no consumer policy handoff.
- `c-5`: C-5 does not manage consumer boundaries-policy handoff or subsequent policy decisions.
- `c-6`: C-6 does not own consumer boundaries-policy handoff or later policy decisions.

Beads: anchor `run-81q`, plan questions `run-81q.1`, dev questions `run-81q.2`

Fixtures:
- pass: Fresh setup writes the initial boundaries policy into the consumer repository and documents that the consumer owns subsequent policy decisions.
- pass: C1's contract makes the consumer the decision owner after initial setup and supplies its required ownership qualification, while explicitly assigning the shipped policy handoff/preservation implementation to C2.
- fail: The generated consumer's boundaries policy remains centrally owned by sc-runtime, which decides all subsequent consumer boundary changes.
- fail: C1's contract leaves subsequent consumer boundary decisions with sc-runtime, or only promises to decide ownership in a later sprint without defining the handoff.

## ANC-009

Anchor: Phase C must upgrade rmcp to its latest released version.

> upgrading rmcp/utopia to the latest releave version should be planned in phase-<next>

Said by randlee, User approved clarity corrections and split fixtures; split from ANC-002, original quote unchanged., 2026-11-02. Status: locked. Version 2.

Summary: upgrade rmcp to its latest released version.

Scope: the c-1 compatibility contract and c-3 runtime ecosystem upgrade sprints

Deterministic check to pair with the development gate: Verify the c-1 dated registry query and frozen rmcp version used by the runtime/template graph.

Excluded from (not judged; the cell passes as `excluded`):
- `c-2`: The existing scope assigns RMCP version selection/upgrades to C-1 and C-3, not the C-2 lint profile.
- `c-4`: The existing RMCP upgrade scope names C-1/C-3; C-4 consumes their frozen matrix without selecting upgrade versions.
- `c-5`: RMCP version selection/upgrades are assigned to C-1/C-3; C-5 uses their existing contracts.
- `c-6`: The existing RMCP version-selection/upgrade scope is C-1/C-3, not C-6 adapter behavior.

Beads: anchor `run-qt3`, plan questions `run-qt3.1`, dev questions `run-qt3.2`

Fixtures:
- pass: Phase C checks the released rmcp versions and upgrades rmcp to its latest release.
- fail: Phase C keeps the older rmcp dependency despite a newer released version being available.

## ANC-010

Anchor: Phase C must upgrade utoipa to its latest released version.

> upgrading rmcp/utopia to the latest releave version should be planned in phase-<next>

Said by randlee, User approved clarity corrections and split fixtures; split from ANC-002, original quote unchanged., 2026-11-02. Status: locked. Version 2.

Summary: upgrade utoipa to its latest released version.

Scope: the c-1 compatibility contract and c-3 runtime ecosystem upgrade sprints

Deterministic check to pair with the development gate: Verify the c-1 dated registry query and frozen utoipa version used by the runtime/template graph.

Excluded from (not judged; the cell passes as `excluded`):
- `c-2`: The existing scope assigns Utoipa version selection/upgrades to C-1 and C-3, not the C-2 lint profile.
- `c-4`: The existing Utoipa upgrade scope names C-1/C-3; C-4 consumes their frozen matrix without selecting upgrade versions.
- `c-5`: Utoipa version selection/upgrades are assigned to C-1/C-3; C-5 uses their existing contracts.
- `c-6`: The existing Utoipa version-selection/upgrade scope is C-1/C-3, not C-6 adapter behavior.

Beads: anchor `run-evb`, plan questions `run-evb.1`, dev questions `run-evb.2`

Fixtures:
- pass: Phase C checks the released utoipa versions and upgrades utoipa to its latest release.
- fail: Phase C keeps the older utoipa dependency despite a newer released version being available.

## ANC-011

Anchor: Phase C planning must use a dedicated Git worktree.

> planning should be done on a second pr to develop on a dedicated $sc-git-worktree

Said by randlee, User approved clarity corrections and split fixtures; split from ANC-005, original quote unchanged., 2026-11-02. Status: locked. Version 2.

Summary: use a dedicated Git worktree.

Scope: the Phase C planning branch and Git worktree metadata

Deterministic check to pair with the development gate: Git worktree metadata identifies the separate plan/phase-c checkout.

Excluded from (not judged; the cell passes as `excluded`):
- `c-1`: C-1 implements the runtime contract rather than creating the initial Phase C planning worktree, already completed in the dedicated plan/phase-c checkout.
- `c-3`: C-3 upgrades the runtime ecosystem rather than creating the initial Phase C planning worktree, already completed in the dedicated plan/phase-c checkout.

Beads: anchor `run-alr`, plan questions `run-alr.1`, dev questions `run-alr.2`

Fixtures:
- pass: Git worktree metadata shows a dedicated Phase C planning checkout.
- fail: Phase C planning is performed in the primary develop checkout.

## ANC-012

Anchor: Phase C planning must have a separate pull request targeting develop.

> planning should be done on a second pr to develop on a dedicated $sc-git-worktree

Said by randlee, User approved clarity corrections and split fixtures; split from ANC-005, original quote unchanged., 2026-11-02. Status: locked. Version 2.

Summary: have a separate pull request targeting develop.

Scope: the Phase C planning pull request metadata

Deterministic check to pair with the development gate: GitHub PR114 head plan/phase-c targets develop and is separate from dependency PR109.

Excluded from (not judged; the cell passes as `excluded`):
- `c-1`: C-1 implements the runtime contract rather than creating the initial Phase C planning pull request, already completed separately as PR114 to develop.
- `c-3`: C-3 upgrades the runtime ecosystem rather than creating the initial Phase C planning pull request, already completed separately as PR114 to develop.

Beads: anchor `run-mwv`, plan questions `run-mwv.1`, dev questions `run-mwv.2`

Fixtures:
- pass: A separate Phase C planning pull request targets develop.
- fail: Phase C planning has no separate pull request to develop.

## ANC-013

Anchor: Phase C must complete its dedicated compatibility sprint before publishing sc-runtime 0.2.0.

> Add a dedicated Phase C compatibility sprint (Recommended)

Said by randlee, Proposed only at the explicit user request to finalize anchors. Human approved all six statements and original fixtures, with the clarified ANC-016 statement. Testability/live verification remains pending; no accepted v3 rubric or live planning gate. Approval receipt: docs/plans/phase-c/reviews/authority-approvals/new-anchor-approval-with-anc016-clarification.json. Verbatim approved-source context: docs/plans/phase-c/reviews/internal-unapproved/actual-candidate-intent/ANC-013.json. Packet reference ANC-013 is provisional, not its allocated ID., 2026-11-03. Status: locked. Version 4.

Summary: Compatibility precedes publication

Scope: Phase C compatibility completion and the 0.2.0 publication handoff

Excluded from (not judged; the cell passes as `excluded`):
- `c-2`: C-2 owns the independent lint profile, not the dedicated C-5 compatibility completion or C-4 publication handoff.
- `c-6`: C-6 closes the adapter boundary and explicitly excludes publication; C-5 owns dedicated compatibility completion and C-4 enforces its publication precedence.
- `c-1`: C-1 defines the future release contract but performs neither C-5 compatibility completion nor the actual C-4 publication handoff governed by this anchor.
- `c-3`: C-3 prepares versions and prepublication inputs without performing C-5 compatibility completion or the actual C-4 publication handoff, and retains its required C-5-before-publication dependency guard.

Beads: anchor `run-299`, plan questions `run-299.1`, dev questions `run-299.2`

Fixtures:
- pass: The dedicated Phase C compatibility sprint completes before the release handoff publishes sc-runtime 0.2.0.
- fail: The release handoff publishes sc-runtime 0.2.0 before the dedicated Phase C compatibility sprint is complete.

## ANC-014

Anchor: A consumer application must be able to omit the standard cron command set.

> it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface.

Said by randlee, Proposed only at the explicit user request to finalize anchors. Human approved all six statements and original fixtures, with the clarified ANC-016 statement. Testability/live verification remains pending; no accepted v3 rubric or live planning gate. Approval receipt: docs/plans/phase-c/reviews/authority-approvals/new-anchor-approval-with-anc016-clarification.json. Verbatim approved-source context: docs/plans/phase-c/reviews/internal-unapproved/actual-candidate-intent/ANC-014.json. Packet reference ANC-014 is provisional, not its allocated ID., 2026-11-03. Status: locked. Version 4.

Summary: Optional standard cron commands

Scope: Standard cron command-set adoption and composition

Excluded from (not judged; the cell passes as `excluded`):
- `c-2`: C-2 does not adopt or compose cron commands; that contract/adapter/composition work belongs to C-1/C-6/C-4.
- `c-5`: C-5 owns cron core/store behavior; optional standard-command adoption/composition is C-1/C-6/C-4 work.

Beads: anchor `run-8co`, plan questions `run-8co.1`, dev questions `run-8co.2`

Fixtures:
- pass: The consumer can choose whether to expose the standard cron command set and can omit it entirely.
- fail: Every consumer is required to expose the standard cron command set.

## ANC-015

Anchor: Supported cron user interfaces must share the standard JSON command contract.

> it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface.

Said by randlee, Proposed only at the explicit user request to finalize anchors. Human approved all six statements and original fixtures, with the clarified ANC-016 statement. Testability/live verification remains pending; no accepted v3 rubric or live planning gate. Approval receipt: docs/plans/phase-c/reviews/authority-approvals/new-anchor-approval-with-anc016-clarification.json. Verbatim approved-source context: docs/plans/phase-c/reviews/internal-unapproved/actual-candidate-intent/ANC-015.json. Packet reference ANC-015 is provisional, not its allocated ID., 2026-11-03. Status: locked. Version 4.

Summary: Shared cron JSON contract

Scope: Supported cron UI JSON command contracts

Excluded from (not judged; the cell passes as `excluded`):
- `c-2`: C-2 installs analyzers and boundaries policy, not cron UI JSON projections.
- `c-5`: C-5 has no CLI/HTTP/MCP standard JSON projections; the adapter and composition owners are C-6/C-4.

Beads: anchor `run-m9k`, plan questions `run-m9k.1`, dev questions `run-m9k.2`

Fixtures:
- pass: Supported cron user interfaces use the same standard JSON command contract to configure schedules.
- fail: Each supported cron user interface defines a different, incompatible JSON command contract for configuring schedules.

## ANC-016

Anchor: The standard cron command set must use common schedule-management subcommands across Python, Monty, and Rust handlers.

> yes, we have a few more options i.e. python, monty, rust handler. but the general subcommands should be standard

Said by randlee, Proposed only at the explicit user request to finalize anchors. Human approved all six statements and original fixtures, with the clarified ANC-016 statement. Testability/live verification remains pending; no accepted v3 rubric or live planning gate. Approval receipt: docs/plans/phase-c/reviews/authority-approvals/new-anchor-approval-with-anc016-clarification.json. Verbatim approved-source context: docs/plans/phase-c/reviews/internal-unapproved/actual-candidate-intent/ANC-016.json. Packet reference ANC-016 is provisional, not its allocated ID., 2026-11-03. Status: locked. Version 5.

Summary: Common schedule-management subcommands across handlers

Scope: Standard cron subcommand vocabulary and handler-kind selection

Excluded from (not judged; the cell passes as `excluded`):
- `c-2`: C-2 has no cron management verbs or Rust/Python/Monty target selection.
- `c-5`: C-5 does not implement the standard management-command vocabulary or its target-selection UI; C-6/C-4 do.

Beads: anchor `run-fwe`, plan questions `run-fwe.1`, dev questions `run-fwe.2`

Fixtures:
- pass: Python, Monty, and Rust cron handlers use the same general subcommands, with handler-specific choices represented by their options.
- fail: Python, Monty, and Rust cron handlers each require a different set of general cron subcommands.

## ANC-017

Anchor: sc-runtime must expose supported extension points that let consumers replace default core-library behavior with application-specific behavior.

> it is not sc-runtime job to guarantee a customer can not misuse the core libraries.

Said by randlee, Proposed only at the explicit user request to finalize anchors. Human approved all six statements and original fixtures, with the clarified ANC-016 statement. Testability/live verification remains pending; no accepted v3 rubric or live planning gate. Approval receipt: docs/plans/phase-c/reviews/authority-approvals/new-anchor-approval-with-anc016-clarification.json. Verbatim approved-source context: docs/plans/phase-c/reviews/internal-unapproved/actual-candidate-intent/ANC-018.json. Packet reference ANC-018 is provisional, not its allocated ID., 2026-11-03. Status: locked. Version 4.

Summary: Supported consumer overrides

Scope: Core-library integration hooks for consumer store/schema and shutdown policy

Excluded from (not judged; the cell passes as `excluded`):
- `c-2`: C-2 policy ownership is covered by ANC-008; it does not replace core-library stores, schemas or shutdown policies.

Beads: anchor `run-ifo`, plan questions `run-ifo.1`, dev questions `run-ifo.2`

Fixtures:
- pass: The cron integration accepts an application-provided store with an extended row schema and a consumer shutdown policy through supported extension points instead of requiring its bundled defaults.
- fail: Every consumer must use the bundled cron store and default shutdown policy; custom row schemas and application-specific lifecycle behavior have no supported integration seam.

## ANC-018

Anchor: sc-runtime must document a supported integration path that explains correct use of the core libraries and the responsibilities of consumer overrides.

> it is sc-runtime job to provide an easy documented path that consumers will not accidently misuse the core libraries.

Said by randlee, Proposed only at the explicit user request to finalize anchors. Human approved all six statements and original fixtures, with the clarified ANC-016 statement. Testability/live verification remains pending; no accepted v3 rubric or live planning gate. Approval receipt: docs/plans/phase-c/reviews/authority-approvals/new-anchor-approval-with-anc016-clarification.json. Verbatim approved-source context: docs/plans/phase-c/reviews/internal-unapproved/actual-candidate-intent/ANC-019.json. Packet reference ANC-019 is provisional, not its allocated ID., 2026-11-03. Status: locked. Version 4.

Summary: Documented integration and override responsibilities

Scope: Default and extension integration guides and consumer responsibilities

Excluded from (not judged; the cell passes as `excluded`):
- `c-2`: C-2 documents lint setup and policy ownership, not the core-library default/override integration guides.
- `c-3`: Default/override guides are owned by C-5/C-6 and consumed by C-4; C-3 owns code/projection fixtures, not those guides.

Beads: anchor `run-c5i`, plan questions `run-c5i.1`, dev questions `run-c5i.2`

Fixtures:
- pass: The integration guide shows the default cron setup and a supported custom store or shutdown policy example, including the consumer's validation, lifecycle and ownership responsibilities.
- fail: Consumers must inspect private scheduler internals to discover override wiring; the documentation explains neither supported hooks nor consumer responsibilities.
