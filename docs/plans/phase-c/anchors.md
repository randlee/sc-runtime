# Anchors - phase-c

Constraints captured during planning. Each anchor is a testable statement written by the orchestrator and approved by the user; the user's verbatim words are kept beneath it as provenance. Every anchor is judged at the planning gate and again at the development gate. Edit `anchors.json`, not this file; this file is rendered.

Intent: `docs/plans/phase-c/phase-c-plan.md`

| ID | Status | Summary | Stated |
| --- | --- | --- | --- |
| ANC-001 | locked | Generated consumer runs every configured analyzer | 2026-11-02 |
| ANC-002 | locked | One compatible runtime ecosystem upgrade | 2026-11-02 |
| ANC-003 | locked | Repository and template share sc-lint 0.8.4 | 2026-11-02 |
| ANC-004 | locked | Phase C uses c consistently | 2026-11-02 |
| ANC-005 | locked | Dedicated Phase C planning worktree and PR | 2026-11-02 |
| ANC-006 | locked | Keep rusqlite 0.39.x SQLite compatibility | 2026-11-02 |
| ANC-007 | locked | Configuration-selected default boundaries | 2026-11-02 |
| ANC-008 | locked | Consumer-owned boundaries policy | 2026-11-02 |

## ANC-001

Anchor: A generated consumer repository must have the full sc-lint toolchain installed.

> a full sc-lint is installed on a consumer repo

Said by randlee, Phase C planning conversation, 2026-11-02. Status: locked. Version 3.

Summary: Generated consumer runs every configured analyzer

Scope: the c-2 generated-consumer lint sprint and any plan item that edits template sc-lint setup or default or full lint commands

Deterministic check to pair with the development gate: A generated fixture runs just setup, ordinary just lint, and named default and full profiles while its receipt records sc-boundary, sc-portability, and sc-runtime.

Beads: anchor `run-32b`, plan questions `run-32b.1`, dev questions `run-32b.2`

Fixtures:
- pass: Consumer setup installs the full sc-lint toolchain, including its boundary, portability, and runtime analyzers.
- fail: Consumer setup installs only rustfmt and clippy and does not install the sc-lint toolchain.

## ANC-002

Anchor: Phase C must upgrade rmcp and utoipa to their latest released versions.

> upgrading rmcp/utopia to the latest releave version should be planned in phase-<next>

Said by randlee, Phase C planning conversation, 2026-11-02. Status: locked. Version 2.

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

Beads: anchor `run-83r`, plan questions `run-83r.1`, dev questions `run-83r.2`

Fixtures:
- pass: Both the repository and generated template declare sc-lint 0.8.4 and their bootstrap commands install or verify 0.8.4.
- fail: The repository bootstraps sc-lint 0.8.4 while the generated template declares and installs 0.8.0.

## ANC-004

Anchor: The current phase must be named phase-c.

> rename to phase-c

Said by randlee, Phase C planning conversation, 2026-11-02. Status: locked. Version 2.

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

Said by randlee, Phase C planning conversation, 2026-11-02. Status: locked. Version 2.

Summary: Dedicated Phase C planning worktree and PR

Scope: the Phase C plan branch, worktree, and pull request metadata

Deterministic check to pair with the development gate: Git worktree metadata records plan/phase-c separately, and its pull request targets develop.

Beads: anchor `run-aov`, plan questions `run-aov.1`, dev questions `run-aov.2`

Fixtures:
- pass: Git worktree metadata shows a separate plan/phase-c checkout, and its dedicated planning pull request targets develop.
- fail: Phase C planning is committed directly on the primary develop checkout and has no separate planning pull request.

## ANC-006

Anchor: Phase C must keep rusqlite on the 0.39 release line.

> so rusqlite will remain at 0.39? I will have beads-cache downgrade so everything is consistent.

Said by randlee, Phase C planning conversation, 2026-11-02. Status: locked. Version 2.

Summary: Keep rusqlite 0.39.x SQLite compatibility

Scope: the c-3 runtime ecosystem upgrade sprint and any plan item that changes Cargo dependency versions for rusqlite, sqlx, or libsqlite3-sys

Deterministic check to pair with the development gate: Cargo metadata and the SQLite resolver test show rusqlite 0.39.x and libsqlite3-sys 0.37 with no libsqlite3-sys 0.38.

Beads: anchor `run-ioz`, plan questions `run-ioz.1`, dev questions `run-ioz.2`

Fixtures:
- pass: Phase C retains rusqlite 0.39.x while upgrading the other approved dependencies.
- fail: Phase C upgrades rusqlite from 0.39 to a newer minor release line.

## ANC-007

Anchor: Fresh consumer installation must generate a default boundaries policy selected from the consumer's configuration settings.

> and you will need to determine the 'appropriate' default policy to install on consumer repos
based on configuration settings

Said by randlee, User-approved packet; ANC-007 quote contains two consecutive verbatim user messages separated by a newline. Statements and fixtures approved in this chat., 2026-11-02. Status: locked. Version 2.

Summary: Configuration-selected default boundaries

Scope: c-1 consumer policy contract and c-2 fresh-install implementation

Deterministic check to pair with the development gate: Generate supported consumer configurations and verify their emitted policy matches selected crates and edges.

Beads: anchor `run-a7e`, plan questions `run-a7e.1`, dev questions `run-a7e.2`

Fixtures:
- pass: Fresh consumer installation reads the selected configuration and generates the matching default boundaries policy before running sc-boundary.
- fail: Fresh consumer installation emits no boundaries policy and runs sc-boundary only in no-manifest mode, regardless of configuration.

## ANC-008

Anchor: After fresh setup, the generated boundaries policy must be owned by the consumer repository.

> you will need to generate a boundaries policy for a 'fresh install' once a consumer is set up, they will have their own boundaries policy

Said by randlee, User-approved packet; ANC-007 quote contains two consecutive verbatim user messages separated by a newline. Statements and fixtures approved in this chat., 2026-11-02. Status: locked. Version 2.

Summary: Consumer-owned boundaries policy

Scope: c-1 consumer policy ownership contract and c-2 generated policy handoff

Deterministic check to pair with the development gate: Generated policy lives in the consumer repository and documented setup identifies the consumer as owner.

Beads: anchor `run-81q`, plan questions `run-81q.1`, dev questions `run-81q.2`

Fixtures:
- pass: Fresh setup writes the initial boundaries policy into the consumer repository and documents that the consumer owns subsequent policy decisions.
- fail: The generated consumer's boundaries policy remains centrally owned by sc-runtime, which decides all subsequent consumer boundary changes.
