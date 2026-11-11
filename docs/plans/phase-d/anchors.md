# Anchors - phase-d

Constraints captured during planning. Each anchor is a testable statement written by the orchestrator and approved by the user; the user's verbatim words are kept beneath it as provenance. Every anchor is judged at the planning gate and again at the development gate. Edit `anchors.json`, not this file; this file is rendered.

Intent: `docs/plans/phase-d/intent.md`

| ID | Status | Summary | Stated |
| --- | --- | --- | --- |
| ANC-001 | locked | Registry floors and consumer lag policy | 2026-11-11 |
| ANC-002 | locked | Hermetic external CLI install | 2026-11-11 |
| ANC-003 | locked | SC_COMPOSE override independence | 2026-11-11 |
| ANC-004 | locked | Upstream sc-compose distribution issue | 2026-11-11 |
| ANC-005 | locked | CLI pin separate from crate graph | 2026-11-11 |
| ANC-006 | locked | Plan gate before implementation | 2026-11-11 |
| ANC-007 | locked | Rusqlite #132 clearance | 2026-11-11 |
| ANC-008 | locked | Frozen sqlx vs rusqlite ruling | 2026-11-11 |
| ANC-009 | locked | #133 floor sweep | 2026-11-11 |
| ANC-010 | locked | Publish after dependency sprints | 2026-11-11 |
| ANC-011 | locked | rmcp >=3.9 floor | 2026-11-11 |
| ANC-012 | locked | Production tool boundary | 2026-11-11 |
| ANC-013 | locked | #131 consumer router docs | 2026-11-11 |
| ANC-014 | locked | Dedicated plan PR | 2026-11-11 |
| ANC-015 | locked | Skills use CLI floor not exact pin | 2026-11-11 |

## ANC-001

Anchor: Phase d MUST document and implement tested registry dependency floors with CI proving low and high resolve, and explicit consumer-lag caps with ADR text when a security fix cannot use crates.io latest.

> we need a robust model both for ci and for downstream consumers (which could not upgrade rusqlite w/ had a security violation)

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 4.

Summary: Registry floors and consumer lag policy

Scope: d-1, d-1b, d-2, and issue-133-dependency-sweep.md

Deterministic check to pair with the development gate: The sweep doc and sprint plans name >= floors, < caps, and minimal-versions recipe; cargo update smoke is cited in d-1b acceptance.

Beads: anchor `run-a4wv`, plan questions `run-a4wv.1`, dev questions `run-a4wv.2`

Fixtures:
- pass: Phase d MUST document tested registry dependency floors with CI low/high resolve: issue-133-dependency-sweep.md names >= floors and caps and sprint d-1b acceptance cites minimal-versions smoke.
- fail: Phase d ships bare carets with no minimal-versions recipe.

## ANC-002

Anchor: CI MUST install pinned external CLIs from immutable GitHub release binaries, not cargo install, when release artifacts exist for that tool pin.

> sc-runtime is not tolerant of 'upgrades' which occur outside of itself

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: Hermetic external CLI install

Scope: rust-build-cache and docs/development/ci.md

Deterministic check to pair with the development gate: rust-build-cache action.yml installs sc-compose from release tarballs with checksums.txt; no cargo install sc-compose step.

Beads: anchor `run-r85j`, plan questions `run-r85j.1`, dev questions `run-r85j.2`

Fixtures:
- pass: rust-build-cache installs sc-compose from a GitHub release tarball verified with checksums.txt.
- fail: CI runs cargo install sc-compose on cache miss.

## ANC-003

Anchor: scripts/new_project.py MUST honor SC_COMPOSE when set; this repository's CI sc-compose pin MUST NOT require downstream generated apps to use the same CLI version.

> A downstream template should be able to upgrade to sc-compose 1.8.0 and assuming there were no breaking changes, this should NOT be an issue

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: SC_COMPOSE override independence

Scope: template generation driver and .sc/generation-tools.json

Deterministic check to pair with the development gate: new_project.py resolves SC_COMPOSE before PATH; generation-tools.json documents tool pin separate from consumer choice.

Beads: anchor `run-d4yq`, plan questions `run-d4yq.1`, dev questions `run-d4yq.2`

Fixtures:
- pass: Generation honors SC_COMPOSE while CI uses generation-tools.json version pin.
- fail: The driver ignores SC_COMPOSE and hard-codes one global sc-compose path.

## ANC-004

Anchor: Phase d MUST link sc-compose#1041 for upstream distribution improvements serving multiple consumer version pins.

> If there is an issue w/ sc-compose release/distribution process, there needs to be a clear gh issue

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 3.

Summary: Upstream sc-compose distribution issue

Scope: phase-d intent and maintenance docs

Deterministic check to pair with the development gate: intent.md links https://github.com/randlee/sc-compose/issues/1041

Beads: anchor `run-lhqu`, plan questions `run-lhqu.1`, dev questions `run-lhqu.2`

Fixtures:
- pass: docs/plans/phase-d/intent.md links https://github.com/randlee/sc-compose/issues/1041 for sc-compose consumer install drift.
- fail: Phase d plan docs include no URL or reference to sc-compose GitHub issue 1041.

## ANC-005

Anchor: .sc/generation-tools.json MUST pin the sc-compose CLI at minimum_version >= 1.8.0; production Cargo.toml files MUST NOT treat that file as a crate dependency pin.

> you need to make sure this repo pins sc-compose tool at 1.8+ .  /plan-anchoring uses the sc-compose tool.  we also reference sc-compose crates which is a spearate concern

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: CLI pin separate from crate graph

Scope: generation-tools.json, boundary lint, plan-anchoring skills

Deterministic check to pair with the development gate: generation-tools.json exists with minimum_version 1.8.0; boundaries reject production sc-compose crate deps.

Beads: anchor `run-7ahk`, plan questions `run-7ahk.1`, dev questions `run-7ahk.2`

Fixtures:
- pass: generation-tools.json sets minimum_version 1.8.0; boundaries forbid production sc-compose crate deps.
- fail: sc-runtime adds sc-compose as a production crate dependency.

## ANC-006

Anchor: Plan QA-1 MUST pass and the phase-d plan MUST merge to develop before any sprint/d-* implementation branch opens.

> No sprint branches until this plan merges to develop.

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 3.

Summary: Plan gate before implementation

Scope: phase-d-plan.md readiness section

Deterministic check to pair with the development gate: phase-d-plan states hardening in flight and no sprint branches until plan merges; QA-1 recorded in plan-hardening-rounds.md

Beads: anchor `run-tx9g`, plan questions `run-tx9g.1`, dev questions `run-tx9g.2`

Fixtures:
- pass: phase-d-plan.md forbids sprint/d-* branches until the plan merges to develop; plan-hardening-rounds.md records QA-1 PASS before any sprint branch opens.
- fail: sprint/d-1 opens while phase-d plan QA-1 is still failing.

## ANC-007

Anchor: Sprint d-1 MUST ship rusqlite >=0.41.0 with tested low/high resolve and close GitHub issue #132 including RUSTSEC-2026-0459 clearance.

> Close maintenance gaps after Phase C 0.2.0: Issue #132 — rusqlite 0.41+ with tested minimum/latest range; RUSTSEC-2026-0459 clearance.

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 3.

Summary: Rusqlite #132 clearance

Scope: sprint d-1 and sc-cron/sqlite writer crates

Deterministic check to pair with the development gate: d-1 acceptance includes cargo test at rusqlite floor and highest 0.41.x; issue #132 closable on deliver.

Beads: anchor `run-49x2`, plan questions `run-49x2.1`, dev questions `run-49x2.2`

Fixtures:
- pass: Sprint d-1 closes issue #132 by upgrading rusqlite to 0.41 and Cargo.lock resolves rusqlite 0.41.x, clearing RUSTSEC-2026-0459.
- fail: Phase d stays on rusqlite 0.39 with open RUSTSEC-2026-0459.

## ANC-008

Anchor: Phase d MUST NOT support sqlx 0.9 and rusqlite 0.41 in the same process writer on 0.2.1; the sqlx-resolve fixture is deleted and ADR-RUN-0012 is amended.

> Blocked upgrades (sqlx major, rusqlite vs sqlx same-process) stay explicit in that doc and d-1 ruling — not every dep can move to crates.io latest in 0.2.1.

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: Frozen sqlx vs rusqlite ruling

Scope: d-1 sqlite vertical and ADR-RUN-0012

Deterministic check to pair with the development gate: sqlx-resolve fixture absent; architecture text documents no same-process sqlx 0.9 writer with rusqlite 0.41

Beads: anchor `run-lo0p`, plan questions `run-lo0p.1`, dev questions `run-lo0p.2`

Fixtures:
- pass: sqlx-resolve fixture is deleted; ADR-RUN-0012 forbids same-process sqlx 0.9 writer with rusqlite 0.41.
- fail: Phase d keeps sqlx-resolve and claims same-process sqlx 0.9 + rusqlite 0.41 is supported.

## ANC-009

Anchor: Sprint d-1b MUST add a documented minimal-versions validation recipe and replace third-party ~ and = pins with >= tested floors per issue #133.

> Issue #133 — dependency >= tested floors (not ~ / = pins); rmcp ≥ 3.9 and dual lockfile CI.

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: #133 floor sweep

Scope: d-1b owned Cargo.toml files and Justfile

Deterministic check to pair with the development gate: Justfile or CI doc names minimal-versions command; sprint acceptance req:133-pins passes review

Beads: anchor `run-yypr`, plan questions `run-yypr.1`, dev questions `run-yypr.2`

Fixtures:
- pass: d-1b Justfile documents minimal-versions command and removes third-party ~ pins.
- fail: d-1b leaves rmcp pinned at ~3.8 without documented exception.

## ANC-010

Anchor: The d-2 publish tail MUST not run until d-1 and d-1b lockfile changes merge-forward and required quality gates pass.

> Compatibility precedes publication

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: Publish after dependency sprints

Scope: sprint d-2 publish tail and release kit

Deterministic check to pair with the development gate: d-2 plan orders publish after d-1 and d-1b merge-forward; version bumps only in publish tail

Beads: anchor `run-gyab`, plan questions `run-gyab.1`, dev questions `run-gyab.2`

Fixtures:
- pass: d-2 version bumps run only after d-1 and d-1b merge-forward with green gates.
- fail: 0.2.1 tags ship before rusqlite and rmcp floors land.

## ANC-011

Anchor: Sprint d-1b MUST raise rmcp to >=3.9.0 with tested low and highest <4 resolve and document RUSTSEC-2026-0462 clearance.

> Phase C checks the released rmcp versions and upgrades rmcp to its latest release.

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: rmcp >=3.9 floor

Scope: d-1b MCP manifests in sc-runtime sc-command template daemon

Deterministic check to pair with the development gate: Cargo.toml rmcp lines use >=3.9.0; tests pass at 3.9.0 floor

Beads: anchor `run-vepv`, plan questions `run-vepv.1`, dev questions `run-vepv.2`

Fixtures:
- pass: sc-runtime Cargo.toml sets rmcp >=3.9.0 and tests pass at 3.9.0 floor.
- fail: Phase d keeps rmcp 3.8 with open RUSTSEC-2026-0462.

## ANC-012

Anchor: Phase d sprints MUST NOT add sc-compose or agent-team-mail production crate dependencies to release crates.

> sc-compose and atm crates should be blocked from any production release crate.

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: Production tool boundary

Scope: every phase-d sprint touching Cargo.toml

Deterministic check to pair with the development gate: cargo tests/dependencies.rs and boundary lint stay green on production crates

Beads: anchor `run-2ucg`, plan questions `run-2ucg.1`, dev questions `run-2ucg.2`

Fixtures:
- pass: No phase-d PR adds sc-compose as a production dependency in release crates.
- fail: sc-runtime adds sc-compose crate to sc-cron production dependencies.

## ANC-013

Anchor: Sprint d-2 MUST document test-util instrument_router for consumers per merged issue #131.

> document test-util / instrument_router for consumers

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 7.

Summary: #131 consumer router docs

Scope: d-2 README wave

Deterministic check to pair with the development gate: sc-runtime README or crate README names instrument_router and test-util usage from #131

Beads: anchor `run-pddk`, plan questions `run-pddk.1`, dev questions `run-pddk.2`

Fixtures:
- pass: Sprint d-2 MUST document test-util instrument_router for consumers: the d-2 README adds a section explaining instrument_router behind test-util per issue #131.
- fail: Sprint d-2 plan defers issue #131 and ships 0.2.1 without adding README consumer guidance for instrument_router or test-util.

## ANC-014

Anchor: Phase d planning MUST stay on branch plan/phase-d and ship through a dedicated pull request to develop separate from sprint implementation PRs.

> have a separate pull request targeting develop.

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: Dedicated plan PR

Scope: plan/phase-d branch and phase-d-plan.md

Deterministic check to pair with the development gate: Open PR targets develop from plan/phase-d; sprint branches are separate

Beads: anchor `run-zn66`, plan questions `run-zn66.1`, dev questions `run-zn66.2`

Fixtures:
- pass: Phase d planning merges via a dedicated PR to develop; implementation uses sprint branches.
- fail: Sprint implementation commits land on plan/phase-d mixed with plan prose.

## ANC-015

Anchor: Repository skills that invoke sc-compose MUST enforce minimum_version from .sc/generation-tools.json as a >= floor, not an exact version match.

> grep all skills and verify they are tolerant of sc-compose >= a floor threshold

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 2.

Summary: Skills use CLI floor not exact pin

Scope: plan-anchoring change_report.py and orchestration skills

Deterministic check to pair with the development gate: change_report.py rejects sc-compose below minimum_version; SKILL step 1 documents >= floor

Beads: anchor `run-tngz`, plan questions `run-tngz.1`, dev questions `run-tngz.2`

Fixtures:
- pass: change_report.py rejects sc-compose below minimum_version; accepts 1.9.0 when floor is 1.8.0.
- fail: Skills require sc-compose --version to match exactly 1.8.0 and reject 1.9.0.
