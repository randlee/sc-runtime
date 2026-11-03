# Anchors - phase-c

Constraints captured during planning. Each anchor is a testable statement written by the orchestrator and approved by the user; the user's verbatim words are kept beneath it as provenance. Every anchor is judged at the planning gate and again at the development gate. Edit `anchors.json`, not this file; this file is rendered.

Intent: `docs/plans/phase-c/phase-c-plan.md`

| ID | Status | Summary | Stated |
| --- | --- | --- | --- |
| ANC-001 | proposed | Generated consumer runs every configured analyzer | 2026-11-02 |
| ANC-002 | proposed | One compatible runtime ecosystem upgrade | 2026-11-02 |
| ANC-003 | proposed | Repository and template share sc-lint 0.8.4 | 2026-11-02 |
| ANC-004 | proposed | Phase C uses c consistently | 2026-11-02 |
| ANC-005 | proposed | Dedicated Phase C planning worktree and PR | 2026-11-02 |
| ANC-006 | proposed | Keep rusqlite 0.39.x SQLite compatibility | 2026-11-02 |

## ANC-001

Anchor: A Phase C generated consumer must install sc-lint through just setup and run sc-boundary, sc-portability, and sc-runtime through ordinary just lint and named default and full profiles.

> a full sc-lint is installed on a consumer repo

Said by randlee, Phase C planning conversation, 2026-11-02. Status: proposed. Version 2.

Summary: Generated consumer runs every configured analyzer

Scope: the c-2 generated-consumer lint sprint and any plan item that edits template sc-lint setup or default or full lint commands

Deterministic check to pair with the development gate: A generated fixture runs just setup, ordinary just lint, and named default and full profiles while its receipt records sc-boundary, sc-portability, and sc-runtime.

## ANC-002

Anchor: Phase C must select and validate newest stable compatible versions of rmcp, utoipa, and utoipa-axum through one runtime/template composition graph.

> upgrading rmcp/utopia to the latest releave version should be planned in phase-<next>

Said by randlee, Phase C planning conversation, 2026-11-02. Status: proposed. Version 1.

Summary: One compatible runtime ecosystem upgrade

Scope: the c-1 compatibility contract and c-3 runtime ecosystem upgrade sprints

Deterministic check to pair with the development gate: Cargo metadata for the runtime and generated daemon composition graph reports one reachable package identity for rmcp, utoipa, and utoipa-axum.

## ANC-003

Anchor: The repository and generated template must both declare and bootstrap sc-lint 0.8.4.

> both repo and templates should use 0.8.4

Said by randlee, Phase C planning conversation, 2026-11-02. Status: proposed. Version 1.

Summary: Repository and template share sc-lint 0.8.4

Scope: the c-1 consumer contract and c-2 template lint profile sprints

Deterministic check to pair with the development gate: The root and template sc-lint configuration require 0.8.4, and each bootstrap path installs or verifies that version.

## ANC-004

Anchor: Phase C plan paths, branches, sprint identifiers, triage records, and QA evidence must use the phase identifier c.

> rename to phase-c

Said by randlee, Phase C planning conversation, 2026-11-02. Status: proposed. Version 1.

Summary: Phase C uses c consistently

Scope: every Phase C plan item and its plan metadata

Deterministic check to pair with the development gate: A targeted repository scan of Phase C assets reports no phase-bc, bc-<n>, or ARCH-BC identifiers.

## ANC-005

Anchor: Phase C planning must be maintained on a dedicated plan/phase-c worktree and proposed to develop in a separate pull request.

> planning should be done on a second pr to develop on a dedicated $sc-git-worktree

Said by randlee, Phase C planning conversation, 2026-11-02. Status: proposed. Version 1.

Summary: Dedicated Phase C planning worktree and PR

Scope: the Phase C plan branch, worktree, and pull request metadata

Deterministic check to pair with the development gate: Git worktree metadata records plan/phase-c separately, and its pull request targets develop.

## ANC-006

Anchor: Phase C must retain rusqlite 0.39.x with libsqlite3-sys 0.37 and must not upgrade the SQLite writer dependency graph.

> so rusqlite will remain at 0.39? I will have beads-cache downgrade so everything is consistent.

Said by randlee, Phase C planning conversation, 2026-11-02. Status: proposed. Version 1.

Summary: Keep rusqlite 0.39.x SQLite compatibility

Scope: the c-3 runtime ecosystem upgrade sprint and any plan item that changes Cargo dependency versions for rusqlite, sqlx, or libsqlite3-sys

Deterministic check to pair with the development gate: Cargo metadata and the SQLite resolver test show rusqlite 0.39.x and libsqlite3-sys 0.37 with no libsqlite3-sys 0.38.
