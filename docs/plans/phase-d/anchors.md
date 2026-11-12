# Anchors - phase-d

Constraints captured during planning. Each anchor is a testable statement written by the orchestrator and approved by the user; the user's verbatim words are kept beneath it as provenance. Every anchor is judged at the planning gate and again at the development gate. Edit `anchors.json`, not this file; this file is rendered.

Intent: `docs/plans/phase-d/intent.md`

| ID | Status | Summary | Stated |
| --- | --- | --- | --- |
| ANC-001 | retired | Registry floors and consumer lag policy | 2026-11-11 |
| ANC-002 | retired | Hermetic external CLI install | 2026-11-11 |
| ANC-003 | locked | Template accepts compatible sc-compose 1.8.0 | 2026-11-11 |
| ANC-004 | retired | Upstream sc-compose distribution issue | 2026-11-11 |
| ANC-005 | locked | Repository sc-compose CLI minimum | 2026-11-11 |
| ANC-006 | retired | Plan gate before implementation | 2026-11-11 |
| ANC-007 | retired | Rusqlite #132 clearance | 2026-11-11 |
| ANC-008 | retired | Frozen sqlx vs rusqlite ruling | 2026-11-11 |
| ANC-009 | retired | #133 floor sweep | 2026-11-11 |
| ANC-010 | retired | Publish after dependency sprints | 2026-11-11 |
| ANC-011 | retired | rmcp >=3.9 floor | 2026-11-11 |
| ANC-012 | retired | Generation and orchestration tools stay outside production graphs | 2026-11-11 |
| ANC-013 | retired | #131 consumer router docs | 2026-11-11 |
| ANC-014 | retired | Dedicated plan PR | 2026-11-11 |
| ANC-015 | locked | Skill checks accept versions above the minimum | 2026-11-11 |
| ANC-016 | retired | Consumers can take compatible dependency upgrades | 2026-11-11 |
| ANC-017 | retired | CI proves dependency range endpoints | 2026-11-11 |
| ANC-018 | retired | Advisory proof uses the released consumer graph | 2026-11-11 |
| ANC-019 | retired | Additional upper bounds explain their known break | 2026-11-11 |
| ANC-020 | locked | Pinned generation tools use release binaries | 2026-11-11 |
| ANC-021 | locked | cron rows default to app SQLite DB | 2026-11-11 |
| ANC-022 | locked | all SQLite writes via one async-channel writer | 2026-11-11 |
| ANC-023 | locked | writer thread owns the only write connection | 2026-11-11 |
| ANC-024 | locked | writer replies after commit | 2026-11-11 |
| ANC-025 | locked | queued writes batched in one transaction | 2026-11-11 |
| ANC-026 | locked | no spawn_blocking to reach writer | 2026-11-11 |
| ANC-027 | locked | async callers do not block awaiting writer reply | 2026-11-11 |
| ANC-028 | locked | no sqlx SQLite driver in SQLite app tree | 2026-11-11 |
| ANC-029 | locked | generation rejects cron with no-database layout | 2026-11-11 |
| ANC-030 | locked | cron keeps one active row per schedule name | 2026-11-11 |
| ANC-031 | locked | cron keeps caller-supplied row limit | 2026-11-11 |
| ANC-032 | locked | SQLite/Postgres/MySQL implement one Storage<T> | 2026-11-11 |
| ADR-RUN-0011 | locked | block_on banned except SQLite writer thread (ADR-RUN-0011 pt1, amended d9dd55e) | 2026-11-11 |
| ANC-033 | locked | generic atomic unit per engine | 2026-11-12 |
| ANC-034 | locked | no consumer table code in storage backends | 2026-11-12 |
| ANC-035 | locked | public storage functions do not panic | 2026-11-12 |
| ANC-036 | locked | writer panic reaches waiting callers as error | 2026-11-12 |
| ANC-037 | locked | No upper bounds on any registry dependency | 2026-11-11 |
| ANC-038 | locked | Tested dependency combination recorded in .sc-runtime/ | 2026-11-11 |
| ANC-039 | locked | sc-monty depends only on Monty crates | 2026-11-12 |
| ANC-040 | locked | sc-monty has no cron types or names | 2026-11-12 |
| ANC-041 | locked | one Monty implementation, in sc-monty | 2026-11-12 |
| ANC-042 | locked | cron hook tests ported, not weakened | 2026-11-12 |
| ANC-043 | locked | hooks receive values, not JSON strings | 2026-11-12 |
| ANC-044 | locked | compiled hook sessions saved as an sc-cron row | 2026-11-12 |
| ANC-045 | locked | blob key = sc-sha SHA-256 of source + dump version | 2026-11-12 |
| ANC-046 | locked | blob used only when its key matches the source | 2026-11-12 |
| ANC-047 | locked | blob captured right after compile, no run state | 2026-11-12 |
| ANC-048 | locked | hooks compile when the schedule is written | 2026-11-12 |
| ANC-049 | locked | Monty transform source is run-time data | 2026-11-12 |
| ANC-050 | locked | command seam documented as not yet stable | 2026-11-12 |
| ANC-051 | locked | no prohibition of the three seam extensions | 2026-11-12 |
| ANC-052 | locked | seam shape leaves room for three extensions | 2026-11-12 |
| ANC-053 | locked | plan names cron commands on the shared path | 2026-11-12 |
| ANC-054 | locked | reason stated for cron commands off the path | 2026-11-12 |
| ANC-055 | locked | apps can modify cron JSON before write | 2026-11-12 |
| ANC-056 | locked | direct conversions need no hand-written transform | 2026-11-12 |
| ANC-057 | locked | no REQ/ADR binds hand-writing | 2026-11-12 |
| ANC-058 | locked | no REQ/ADR forbids a consumer-side generator | 2026-11-12 |
| ANC-059 | locked | plan names widget commands on the path | 2026-11-12 |
| ANC-060 | locked | a widget write uses storage_write | 2026-11-12 |
| ANC-061 | locked | no sc-compose repo crates except sc-sha, no ATM crates | 2026-11-12 |
| ANC-062 | locked | sc-monty starts as an exact copy of current Monty code and tests | 2026-11-12 |
| ANC-063 | locked | a widget read uses storage_read | 2026-11-12 |

## ANC-001

Anchor: Phase d MUST document and implement tested registry dependency floors with CI proving low and high resolve, and explicit consumer-lag caps with ADR text when a security fix cannot use crates.io latest.

> we need a robust model both for ci and for downstream consumers (which could not upgrade rusqlite w/ had a security violation)

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 5.

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

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 3.

Summary: Hermetic external CLI install

Scope: rust-build-cache and docs/development/ci.md

Deterministic check to pair with the development gate: rust-build-cache action.yml installs sc-compose from release tarballs with checksums.txt; no cargo install sc-compose step.

Beads: anchor `run-r85j`, plan questions `run-r85j.1`, dev questions `run-r85j.2`

Fixtures:
- pass: rust-build-cache installs sc-compose from a GitHub release tarball verified with checksums.txt.
- fail: CI runs cargo install sc-compose on cache miss.

## ANC-003

Anchor: Template generation MUST succeed with sc-compose 1.8.0 when the CLI commands used by the template remain compatible.

> A downstream template should be able to upgrade to sc-compose 1.8.0 and assuming there were no breaking changes, this should NOT be an issue

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 4.

Summary: Template accepts compatible sc-compose 1.8.0

Scope: Changes scripts/new_project.py, the sc-compose executable selection logic, or generated-app generation instructions; excludes Rust crate dependency resolution.

Deterministic check to pair with the development gate: Plan: names independent consumer CLI selection. Development: run the generation driver with SC_COMPOSE selecting a compatible version above the CI pin; verify the selected executable is invoked and generation succeeds.

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-8`: d-8 changes MySQL storage crate dependencies only, not template generation's sc-compose version selection.
- `run-d-10`: d-10 changes generated service code; it does not change new_project.py or sc-compose version selection.
- `run-d-5`: d-5 defines storage APIs and shared manifests; it changes neither sc-compose executable selection nor generated-app generation instructions, so ANC-003 does not govern this work.
- `run-d-12`: d-12 changes Cargo.lock recommendation metadata/post-hook delivery, not scripts/new_project.py, sc-compose executable selection or instructions for selecting/running compatible sc-compose commands; d-4/d-11 own that CLI compatibility behavior.

Beads: anchor `run-d4yq`, plan questions `run-d4yq.1`, dev questions `run-d4yq.2`

Fixtures:
- pass: The generation driver selects sc-compose 1.8.0; all CLI commands used by the template remain compatible and generation succeeds.
- fail: The template requires sc-compose 1.7.0 and rejects the compatible sc-compose 1.8.0 release.

## ANC-004

Anchor: Phase d MUST link sc-compose#1041 for upstream distribution improvements serving multiple consumer version pins.

> If there is an issue w/ sc-compose release/distribution process, there needs to be a clear gh issue

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 4.

Summary: Upstream sc-compose distribution issue

Scope: phase-d intent and maintenance docs

Deterministic check to pair with the development gate: intent.md links https://github.com/randlee/sc-compose/issues/1041

Beads: anchor `run-lhqu`, plan questions `run-lhqu.1`, dev questions `run-lhqu.2`

Fixtures:
- pass: docs/plans/phase-d/intent.md links https://github.com/randlee/sc-compose/issues/1041 for sc-compose consumer install drift.
- fail: Phase d plan docs include no URL or reference to sc-compose GitHub issue 1041.

## ANC-005

Anchor: Repository tooling MUST declare a sc-compose CLI minimum version of at least 1.8.0.

> you need to make sure this repo pins sc-compose tool at 1.8+ .  /plan-anchoring uses the sc-compose tool.  we also reference sc-compose crates which is a spearate concern

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 3.

Summary: Repository sc-compose CLI minimum

Scope: Changes the repository sc-compose CLI requirement in .sc/generation-tools.json, generation-tool setup instructions, or plan-anchoring dependency checks; excludes the sc-compose Rust crate graph.

Deterministic check to pair with the development gate: Plan: declares a CLI minimum >=1.8.0. Development: inspect .sc/generation-tools.json minimum_version and the setup checks that consume it; reject a declared minimum below 1.8.0.

Beads: anchor `run-7ahk`, plan questions `run-7ahk.1`, dev questions `run-7ahk.2`

Fixtures:
- pass: Repository tooling declares sc-compose CLI minimum_version 1.8.0 in .sc/generation-tools.json.
- fail: Repository tooling declares sc-compose CLI minimum_version 1.7.0.

## ANC-006

Anchor: Plan QA-1 MUST pass and the phase-d plan MUST merge to develop before any sprint/d-* implementation branch opens.

> No sprint branches until this plan merges to develop.

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 4.

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

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 4.

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

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 3.

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

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 3.

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

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 3.

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

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 3.

Summary: rmcp >=3.9 floor

Scope: d-1b MCP manifests in sc-runtime sc-command template daemon

Deterministic check to pair with the development gate: Cargo.toml rmcp lines use >=3.9.0; tests pass at 3.9.0 floor

Beads: anchor `run-vepv`, plan questions `run-vepv.1`, dev questions `run-vepv.2`

Fixtures:
- pass: sc-runtime Cargo.toml sets rmcp >=3.9.0 and tests pass at 3.9.0 floor.
- fail: Phase d keeps rmcp 3.8 with open RUSTSEC-2026-0462.

## ANC-012

Anchor: Production release crates MUST NOT depend on sc-compose or ATM crates.

> sc-compose and atm crates should be blocked from any production release crate.

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 5.

Summary: Generation and orchestration tools stay outside production graphs

Scope: Changes normal/build dependency declarations or resolved production dependency graphs of published sc-runtime crates. ATM denotes the agent-team-mail workspace crates covered by ADR-RUN-0013, including agent-team-mail and atm-core. Dev-only test and benchmark references are excluded.

Deterministic check to pair with the development gate: Plan: inspect proposed normal/build dependencies. Development: inspect cargo metadata and cargo tree normal/build graphs for each published crate and supported feature set for sc-compose or agent-team-mail.

Beads: anchor `run-2ucg`, plan questions `run-2ucg.1`, dev questions `run-2ucg.2`

Fixtures:
- pass: Production release crates have no normal or build dependency on sc-compose or ATM crates; adapters invoke external executables.
- fail: A production release crate adds the ATM crate atm-core as a normal dependency.

## ANC-013

Anchor: Sprint d-2 MUST document test-util instrument_router for consumers per merged issue #131.

> document test-util / instrument_router for consumers

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 8.

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

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: retired. Version 3.

Summary: Dedicated plan PR

Scope: plan/phase-d branch and phase-d-plan.md

Deterministic check to pair with the development gate: Open PR targets develop from plan/phase-d; sprint branches are separate

Beads: anchor `run-zn66`, plan questions `run-zn66.1`, dev questions `run-zn66.2`

Fixtures:
- pass: Phase d planning merges via a dedicated PR to develop; implementation uses sprint branches.
- fail: Sprint implementation commits land on plan/phase-d mixed with plan prose.

## ANC-015

Anchor: Repository skills invoking sc-compose MUST enforce the declared minimum as a lower bound rather than an exact-version match.

> grep all skills and verify they are tolerant of sc-compose >= a floor threshold

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 3.

Summary: Skill checks accept versions above the minimum

Scope: Changes a skill script or skill instruction that checks the sc-compose CLI version; excludes CI selection of a reproducible binary and Rust crate version requirements.

Deterministic check to pair with the development gate: Plan: specifies lower-bound CLI validation. Development: controlled version-output inputs below, equal to, and above the declared minimum produce reject, accept, and accept respectively; malformed version output is rejected.

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-11`: d-11 changes generated layouts, setup documentation and generation tests; it does not edit any repository skill script/instruction that checks the sc-compose CLI version. d-4 owns those skill lower-bound checks, so ANC-015 does not govern d-11.

Beads: anchor `run-tngz`, plan questions `run-tngz.1`, dev questions `run-tngz.2`

Fixtures:
- pass: The skill requires sc-compose >=1.8.0 and accepts installed sc-compose 1.8.1.
- fail: The skill rejects installed sc-compose 1.8.1 because its configured minimum is exactly 1.8.0.

## ANC-016

Anchor: Published crate and template dependency requirements MUST allow consumers to select newer Cargo SemVer-compatible dependency releases.

> we need to be following 'best practices' for a scaffolding framework which does not block consumers from building or upgrading dependencies w/ non-breaking changes/upgrades.

Said by randlee, User planning handover in this conversation, 2026-11-11; revised intent.md Purpose and Dependency policy direction. Proposed interpretation: normal Cargo-compatible ranges, with known incompatibilities justified separately; blanket unbounded >= is not implied., 2026-11-11. Status: retired. Version 5.

Summary: Consumers can take compatible dependency upgrades

Scope: Changes registry dependency requirements in published crates or template manifests. Cargo-compatible versions share the leftmost nonzero version component with the tested minimum. An upstream release with a demonstrated API, links, or toolchain break is outside the non-breaking guarantee.

Deterministic check to pair with the development gate: Plan: proposed requirements admit compatible updates without republishing sc-runtime. Development: resolve and build a registry consumer at the minimum and a newer compatible dependency version without changing the sc-runtime crate version or adding a path/patch override.

Beads: anchor `run-41jv`, plan questions `run-41jv.1`, dev questions `run-41jv.2`

Fixtures:
- pass: A published crate declares rmcp 3.9.0 with the normal Cargo-compatible range, allowing the consumer to select a compatible 3.9.1 release.
- fail: A published crate requires rmcp =3.9.0 with no demonstrated incompatibility and blocks a compatible 3.9.1 release.

## ANC-017

Anchor: Phase D dependency CI MUST test both the minimum and latest allowed versions of changed registry dependency ranges.

> Run CI at both ends of the range: the lowest allowed versions (`cargo +nightly update -Z minimal-versions`) and the latest (`cargo update`). Every floor is then a tested claim, not a guess.

Said by randlee, randlee-authored GitHub issue #133, Requested change item 3: https://github.com/randlee/sc-runtime/issues/133; same method required in #132., 2026-11-11. Status: retired. Version 4.

Summary: CI proves dependency range endpoints

Scope: Changes a registry dependency range or dependency validation for the root workspace, generated template features, or separate tests/consumer workspaces; excludes prose-only README edits.

Deterministic check to pair with the development gate: Plan: requires passing low/latest CI for each changed dependency resolve and affected features. Development: inspect CI commands, resolved package versions, and test results for both endpoints; a written command or toolchain-prerequisite note alone fails.

Beads: anchor `run-qxzw`, plan questions `run-qxzw.1`, dev questions `run-qxzw.2`

Fixtures:
- pass: CI resolves minimum and latest allowed dependency sets in the root and affected generated/consumer workspaces, then runs their affected tests at both endpoints.
- fail: The PR documents a minimal-versions command but CI tests only the committed latest lockfile.

## ANC-018

Anchor: Phase D security-advisory validation MUST run against a consumer depending on the released registry crates.

> `cargo deny check advisories` is clean for RUSTSEC-2026-0459 in a consumer that depends on the released crates.

Said by randlee, randlee-authored GitHub issue #132, Done when: https://github.com/randlee/sc-runtime/issues/132; applied to Phase D advisory proof including the rmcp advisory in #133., 2026-11-11. Status: retired. Version 3.

Summary: Advisory proof uses the released consumer graph

Scope: Claims advisory clearance or closes #132/#133 using release evidence; includes the d-2 publish tail and beads-cache consumer proof, not prepublication unit-test sprints.

Deterministic check to pair with the development gate: Plan: names a released-crate consumer for advisory validation. Development: inspect consumer Cargo.lock/cargo metadata for registry identities at 0.2.1, exclude local path/patch substitutes, and run cargo deny check advisories on that graph for the targeted advisories.

Fixtures:
- pass: A consumer resolves sc-cron 0.2.1 from crates.io with no local overrides and passes the targeted cargo deny advisory check.
- fail: Advisory clearance is claimed from a workspace test using a path-patched sc-cron while the consumer still resolves the vulnerable released crate.

## ANC-019

Anchor: Each additional dependency upper bound MUST have a manifest comment identifying the known break that requires it.

> Declare the allowed range as `>=` a reasonable, tested floor, instead of `=x.y.z`, `~x.y` or a single 0.x minor. Use an upper bound only where a known break requires one, and say why in a comment.

Said by randlee, randlee-authored GitHub issue #133, Requested change item 2 and Done when: https://github.com/randlee/sc-runtime/issues/133. This candidate captures justification of extra restrictions; it does not adopt the older blanket >= prescription., 2026-11-11. Status: retired. Version 5.

Summary: Additional upper bounds explain their known break

Scope: Adds or retains an explicit third-party dependency upper bound beyond Cargo standard SemVer compatibility. Normal caret compatibility boundaries and internal path package identity versions are excluded. Applies to published crates, generated templates, and test manifests.

Deterministic check to pair with the development gate: Plan: each retained extra restriction names the incompatibility and the owning manifest comment. Development: enumerate exact/tilde/extra-upper requirements and inspect comments for the affected API, links conflict, toolchain limit, or deliberate negative-test compatibility boundary.

Beads: anchor `run-weer`, plan questions `run-weer.1`, dev questions `run-weer.2`

Fixtures:
- pass: The manifest explicitly caps monty below 1.6 and explains in a comment that monty 1.6 requires Rust 1.98 while the supported toolchain is Rust 1.96.1.
- fail: The manifest explicitly caps rmcp below an unpublished major without a comment identifying a known breaking change.

## ANC-020

Anchor: The primary CI installation path for sc-compose MUST use GitHub Release archives.

> **Primary install path for CI and production** — GitHub Release tarballs/zip + `checksums.txt` (already published). Treat these as the **stability contract** for a tagged version.

Said by randlee, randlee-authored sc-compose issue #1041, What consumers need item 1: https://github.com/randlee/sc-compose/issues/1041. Replaces ANC-002 with direct provenance for the binary-install mechanism., 2026-11-11. Status: locked. Version 4.

Summary: Pinned generation tools use release binaries

Scope: Changes CI installation or caching of the sc-compose CLI in .github/actions/rust-build-cache or workflow setup; excludes Cargo dependencies of published sc-runtime crates.

Deterministic check to pair with the development gate: Plan: requires versioned release-binary installation. Development: exercise the CI cache-miss installer against the selected release archive and checksums, verify its CLI version, and verify that no cargo install sc-compose recompilation path supplies that tool.

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-3`: d-3 changes published Cargo dependency ranges and CI range jobs only; it does not change how CI installs sc-compose.

Beads: anchor `run-kaze`, plan questions `run-kaze.1`, dev questions `run-kaze.2`

Fixtures:
- pass: The primary CI installation path fetches the selected sc-compose GitHub Release archive and verifies its checksums.txt entry.
- fail: The primary CI installation path recompiles sc-compose with cargo install against the current crates.io index.

## ANC-021

Anchor: In an app that has a SQLite database and enables cron, the default configuration MUST store cron rows in the app's SQLite database file.

> When an app has a SQLite database and enables cron, cron rows MUST by default be stored in that same SQLite database file.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: cron rows default to app SQLite DB

Scope: plan items that change sc-cron storage, cron persistence configuration, or the generated template's SQLite store or cron wiring

Deterministic check to pair with the development gate: generated SQLite+cron app: cron schedule table exists in the app database file and no other .db file is created under the instance root (test)

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-9`: Defaulting cron rows into the app's SQLite file is generated-app composition wired in d-11; d-9 is engine-neutral.
- `run-d-5`: d-5 defines the generic storage contract; generated apps defaulting cron rows into the app SQLite database is wired in d-11.
- `run-d-6`: d-6 is generic SQLite storage with no cron; default cron placement in the app SQLite file is wired by d-11.
- `run-d-7`: Default cron placement in the app SQLite file is generated-app composition in d-11, not the Postgres backend.
- `run-d-8`: d-8 implements the generic MySQL engine and its dependency probes; it changes neither SQLite store or cron wiring nor cron persistence configuration. d-11 owns default cron rows in the app SQLite database.

Beads: anchor `run-x17o`, plan questions `run-x17o.1`, dev questions `run-x17o.2`

Fixtures:
- pass: With the SQLite layout and cron enabled, the generated daemon opens one SQLite database file and creates the cron schedule table inside it by default.
- fail: With the SQLite layout and cron enabled, the generated daemon keeps app data in app.db and opens a separate cron.db file for schedule rows.

## ANC-022

Anchor: All writes to an app's SQLite database MUST be submitted to a single writer through an async channel.

> Every write to an app's SQLite database MUST go through one writer with an async channel input

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: all SQLite writes via one async-channel writer

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: grep: no SQLite write path outside the writer; writer API takes an async channel sender

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-5`: d-5 defines the contract only; the SQLite channel writer that all writes go through is implemented and tested in d-6.
- `run-d-7`: d-7 is the Postgres backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-8`: d-8 is the MySQL backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-9`: d-9 builds sc-cron over the engine-neutral Storage<CronRow> contract; it does not change the SQLite writer, which d-6 owns.
- `run-d-10`: d-10 does not touch the SQLite backend or its writer, which d-6 owns.
- `run-d-12`: d-12 migrates beads-cache onto and publishes the 0.3.0 crates; SQLite writer behaviour is implemented and tested in d-6 and composed in d-11.
- `run-d-3`: d-3 only qualifies dependency ranges and runs existing SQLite tests at both range ends; the single-writer channel is implemented in d-6.

Beads: anchor `run-b3q4`, plan questions `run-b3q4.1`, dev questions `run-b3q4.2`

Fixtures:
- pass: Every SQLite write from the app and from cron is sent as a request on the writer's tokio mpsc channel and awaited.
- fail: Cron opens its own rusqlite connection and executes its INSERT and UPDATE statements directly, while app writes go through the channel writer.

## ANC-023

Anchor: The app's only SQLite write connection MUST be owned by the SQLite writer thread.

> a single thread that owns the only write connection

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 3.

Summary: writer thread owns the only write connection

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: grep: one write-mode Connection::open in the writer thread; other connections opened read-only

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-5`: d-5 defines the contract only; writer-thread ownership of the single write connection is implemented in d-6.
- `run-d-7`: d-7 is the Postgres backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-8`: d-8 is the MySQL backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-9`: d-9 builds sc-cron over the engine-neutral Storage<CronRow> contract; it does not change the SQLite writer, which d-6 owns.
- `run-d-12`: d-12 migrates beads-cache onto and publishes the 0.3.0 crates; SQLite writer behaviour is implemented and tested in d-6 and composed in d-11.

Beads: anchor `run-wq51`, plan questions `run-wq51.1`, dev questions `run-wq51.2`

Fixtures:
- pass: The writer's dedicated thread opens the only read-write connection; reader threads open their connections read-only.
- fail: Each request handler opens its own read-write SQLite connection to perform its update.

## ANC-024

Anchor: The SQLite writer MUST reply to each write request only after the transaction containing that write has committed.

> Every write to an app's SQLite database MUST go through one writer with an async channel input, a single thread that owns the only write connection, and batched transactions that reply after commit. No caller blocks or uses spawn_blocking to reach it.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: writer replies after commit

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: deterministic test: reply observed only after commit hook fires

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-5`: d-5 defines the contract only; commit-before-reply in the SQLite writer is implemented and tested in d-6.
- `run-d-7`: d-7 is the Postgres backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-8`: d-8 is the MySQL backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-9`: d-9 builds sc-cron over the engine-neutral Storage<CronRow> contract; it does not change the SQLite writer, which d-6 owns.
- `run-d-12`: d-12 migrates beads-cache onto and publishes the 0.3.0 crates; SQLite writer behaviour is implemented and tested in d-6 and composed in d-11.
- `run-d-11`: Reply-after-commit is implemented and tested in d-6; d-11 composes the writer by reference to the root writer constraints.

Beads: anchor `run-ox1n`, plan questions `run-ox1n.1`, dev questions `run-ox1n.2`

Fixtures:
- pass: The writer sends each caller's oneshot reply only after the transaction that contains the caller's write has committed.
- fail: The writer replies Ok to the caller as soon as the write is queued, and commits the batch later.

## ANC-025

Anchor: The SQLite writer MUST commit writes that are queued together in a single transaction.

> Every write to an app's SQLite database MUST go through one writer with an async channel input, a single thread that owns the only write connection, and batched transactions that reply after commit. No caller blocks or uses spawn_blocking to reach it.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: queued writes batched in one transaction

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: deterministic test: N writes queued before drain commit in one transaction (commit hook count 1)

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-5`: d-5 defines the contract only; writer batching of queued writes into one transaction is implemented in d-6.
- `run-d-7`: d-7 is the Postgres backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-8`: d-8 is the MySQL backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-9`: d-9 builds sc-cron over the engine-neutral Storage<CronRow> contract; it does not change the SQLite writer, which d-6 owns.
- `run-d-12`: d-12 migrates beads-cache onto and publishes the 0.3.0 crates; SQLite writer behaviour is implemented and tested in d-6 and composed in d-11.
- `run-d-11`: Single-transaction commit of queued writes is implemented and tested in d-6; d-11 composes the writer by reference to the root writer constraints.

Beads: anchor `run-g9ca`, plan questions `run-g9ca.1`, dev questions `run-g9ca.2`

Fixtures:
- pass: When several writes are waiting in the channel, the writer drains them and commits them together in a single transaction.
- fail: The writer opens and commits a separate transaction for every individual write, even when many are queued.

## ANC-026

Anchor: Callers MUST NOT use spawn_blocking to submit writes to the SQLite writer.

> Every write to an app's SQLite database MUST go through one writer with an async channel input, a single thread that owns the only write connection, and batched transactions that reply after commit. No caller blocks or uses spawn_blocking to reach it.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: no spawn_blocking to reach writer

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: grep: no spawn_blocking in callers of the SQLite writer

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-7`: d-7 is the Postgres backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-8`: d-8 is the MySQL backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-9`: d-9 builds sc-cron over the engine-neutral Storage<CronRow> contract; it does not change the SQLite writer, which d-6 owns.
- `run-d-10`: d-10 implements generic sc-command combinators and Python/Monty transforms against a Storage<T> double; it changes no SQLite backend, writer or generated SQLite store. Those callers/implementations are owned by d-6 and integrated by d-11; d-10 retains its own nonblocking storage-submission constraint.

Beads: anchor `run-x8rt`, plan questions `run-x8rt.1`, dev questions `run-x8rt.2`

Fixtures:
- pass: Async handlers call the writer's async submit method and await the reply; no caller uses spawn_blocking.
- fail: Handlers wrap each storage write in tokio::task::spawn_blocking to call the writer.

## ANC-027

Anchor: Async callers MUST submit writes to the SQLite writer without blocking their thread while waiting for the reply.

> Every write to an app's SQLite database MUST go through one writer with an async channel input, a single thread that owns the only write connection, and batched transactions that reply after commit. No caller blocks or uses spawn_blocking to reach it.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: async callers do not block awaiting writer reply

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: grep: writer submit API is async (await on oneshot), no blocking recv in async callers

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-5`: d-5 defines the contract only; the non-blocking async caller path to the SQLite writer is implemented in d-6.
- `run-d-7`: d-7 is the Postgres backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-8`: d-8 is the MySQL backend over async sqlx; it does not touch the SQLite writer, which d-6 owns.
- `run-d-9`: d-9 builds sc-cron over the engine-neutral Storage<CronRow> contract; it does not change the SQLite writer, which d-6 owns.
- `run-d-12`: d-12 migrates beads-cache onto and publishes the 0.3.0 crates; SQLite writer behaviour is implemented and tested in d-6 and composed in d-11.
- `run-d-10`: d-10 implements engine-neutral sc-command combinators and Python/Monty transforms against a Storage<T> double; it changes no SQLite backend, writer or generated SQLite store. SQLite submission/reply behavior is owned by d-6 and integrated by d-11; d-10 retains its generic nonblocking storage-submission constraint.

Beads: anchor `run-n4fl`, plan questions `run-n4fl.1`, dev questions `run-n4fl.2`

Fixtures:
- pass: Async callers await the writer's oneshot reply future, so their Tokio worker thread is not blocked while waiting.
- fail: Async callers use std::sync::mpsc recv_timeout to wait for the writer's reply on the Tokio worker thread.

## ANC-028

Anchor: The cargo tree of an app that uses SQLite MUST NOT include a sqlx SQLite driver crate.

> An app with SQLite MUST link exactly one SQLite client library: cargo tree shows rusqlite, no sqlx SQLite driver, and one libsqlite3-sys version.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: no sqlx SQLite driver in SQLite app tree

Scope: plan items that change SQLite-related dependencies, the generated template's SQLite store, or sc-cron's storage dependencies

Deterministic check to pair with the development gate: cargo tree -e normal on generated SQLite app: no sqlx-sqlite

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-10`: The generated SQLite app's cargo tree is composed and checked in d-11; d-10 adds no SQLite dependency.

Beads: anchor `run-d6no`, plan questions `run-d6no.1`, dev questions `run-d6no.2`

Fixtures:
- pass: cargo tree for the generated SQLite app shows rusqlite and no sqlx-sqlite crate.
- fail: The generated SQLite app's store still depends on sqlx with the sqlite feature, so cargo tree lists sqlx-sqlite.

## ANC-029

Anchor: Template generation MUST reject a project that enables cron with the no-database layout.

> Cron needs a database. Generation MUST reject cron with the no-database layout.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: generation rejects cron with no-database layout

Scope: plan items that change template generation options, layouts, or the cron feature

Deterministic check to pair with the development gate: generation test: cron enabled + no-database layout exits non-zero with a typed error; no project written

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-5`: d-5 is the storage contract; rejecting cron with the no-database template layout is template generation work in d-11.
- `run-d-6`: Rejecting cron with the no-database template layout is template generation work in d-11, not generic SQLite storage.
- `run-d-7`: Rejecting cron with the no-database layout is template generation work in d-11; d-7 changes no generator.
- `run-d-8`: Rejecting cron with the no-database layout is template generation work in d-11; d-8 changes no generator.
- `run-d-9`: Rejecting cron with the no-database layout is template generation work in d-11; d-9 changes no generator.
- `run-d-10`: d-10 changes no generation options, layouts or cron feature; d-11 owns the no-database cron rejection.
- `run-d-12`: Template rejection of cron with the no-database layout is owned and satisfied by d-11.
- `run-d-3`: d-3 qualifies dependency ranges and CI range jobs only; rejecting cron with the no-database layout is template generation work in d-11.

Beads: anchor `run-1opo`, plan questions `run-1opo.1`, dev questions `run-1opo.2`

Fixtures:
- pass: Generating a project with cron enabled and the no-database layout fails with a typed error and writes no project.
- fail: Generating a project with cron enabled and the no-database layout succeeds and gives cron its own private database.

## ANC-030

Anchor: When sc-cron's schedule rows move from a separate cron database file into the application's database, sc-cron MUST still reject a second active schedule row that has the same schedule name as an active row.

> Moving the cron table MUST NOT drop one-active-schedule-per-name, the row limit, or generation fencing; their tests are ported, not deleted.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: cron keeps one active row per schedule name

Scope: plan items that move or change cron storage, CronStorage, sc-cron's schedule store, or cron tests

Deterministic check to pair with the development gate: ported sc-cron test: unique schedule-name constraint/behaviour test passes against the app-database CronStorage

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-3`: d-3 only qualifies dependency ranges; cron schedule-name uniqueness on the app database is owned by d-9.
- `run-d-5`: d-5 defines the generic transaction; sc-cron's active-name rejection over it is implemented and its tests ported in d-9.
- `run-d-6`: One-active-schedule-per-name is sc-cron domain policy implemented over the generic transaction in d-9; d-6 has no cron schema.
- `run-d-7`: Active-name rejection is sc-cron policy over the generic transaction, implemented in d-9; d-7 has no cron code.
- `run-d-8`: Active-name rejection is sc-cron policy over the generic transaction, implemented in d-9; d-8 has no cron code.

Beads: anchor `run-z85b`, plan questions `run-z85b.1`, dev questions `run-z85b.2`

Fixtures:
- pass: After the move to the app database, the schedule table keeps a unique constraint on schedule name, and the ported test proves a second active row with the same name is rejected.
- fail: After the move, the schedule table drops the unique name constraint, so two active rows with the same schedule name can exist.

## ANC-031

Anchor: When sc-cron's schedule rows move from a separate cron database file into the application's database, sc-cron MUST still refuse to add a schedule row once the row count reaches the row limit configured by the embedding application.

> Moving the cron table MUST NOT drop one-active-schedule-per-name, the row limit, or generation fencing; their tests are ported, not deleted.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 2.

Summary: cron keeps caller-supplied row limit

Scope: plan items that move or change cron storage, CronStorage, sc-cron's schedule store, or cron tests

Deterministic check to pair with the development gate: ported sc-cron row-limit admission tests pass against the app-database CronStorage

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-3`: d-3 only qualifies dependency ranges; the cron row-limit refusal on the app database is owned by d-9.
- `run-d-5`: d-5 defines the generic transaction; sc-cron's row-limit refusal over it is implemented and its tests ported in d-9.
- `run-d-6`: The cron row limit is sc-cron domain policy implemented over the generic transaction in d-9; d-6 has no cron schema.
- `run-d-7`: The cron row limit is sc-cron policy over the generic transaction, implemented in d-9; d-7 has no cron code.
- `run-d-8`: The cron row limit is sc-cron policy over the generic transaction, implemented in d-9; d-8 has no cron code.

Beads: anchor `run-9pf5`, plan questions `run-9pf5.1`, dev questions `run-9pf5.2`

Fixtures:
- pass: After the move, adding a schedule row when the configured row limit is reached is refused, and the ported row-limit test passes against app-database storage.
- fail: After the move, the row-limit check is removed and schedules can be added without limit.

## ANC-032

Anchor: The application storage backends for SQLite, Postgres and MySQL MUST all implement one shared Rust storage trait, Storage<T>.

> Storage is behind one trait. SQLite, Postgres and MySQL MUST implement the same storage trait.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 3.

Summary: SQLite/Postgres/MySQL implement one Storage<T>

Scope: plan items that define the storage trait or implement a SQLite, Postgres or MySQL storage backend

Deterministic check to pair with the development gate: grep: impl Storage<..> for each of the SQLite, Postgres and MySQL backends; same trait path

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-9`: d-9 consumes Storage<T> in sc-cron and implements no SQLite, Postgres or MySQL backend.
- `run-d-10`: d-10 consumes Storage<T> through the shared double and implements no engine backend.
- `run-d-12`: Backend Storage<T> conformance is delivered by d-6, d-7, d-8 and verified in d-11; d-12 publishes the crates.

Beads: anchor `run-7vxt`, plan questions `run-7vxt.1`, dev questions `run-7vxt.2`

Fixtures:
- pass: The SQLite, Postgres and MySQL backends each implement sc_storage::Storage<T>.
- fail: The Postgres backend implements Storage<T>, while the SQLite and MySQL backends keep their own separate per-table store functions.

## ADR-RUN-0011

Anchor: Production code under crates/ and template/ MUST NOT call Runtime::block_on, Handle::block_on or futures::executor::block_on, except Runtime::block_on on the SQLite writer's dedicated OS thread.

> Production code under crates/ and template/ MUST NOT call Runtime::block_on, Handle::block_on or futures::executor::block_on, except the runtime owned and driven on the SQLite writer's dedicated OS thread; SQLite reader threads receive with blocking_recv and need no exception.

Said by randlee, planning session, 2026-11-11. Status: locked. Version 4.

Summary: block_on banned except SQLite writer thread (ADR-RUN-0011 pt1, amended d9dd55e)

Scope: plan items that change async/blocking rules, the SQLite writer or reader pool, or code that bridges sync and async

Deterministic check to pair with the development gate: grep -rn 'block_on' crates/ template/ --include=*.rs excluding tests: only hit is inside the SQLite writer thread module

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-7`: d-7 adds async sqlx Postgres code only; it introduces no blocking bridge and does not touch the SQLite writer thread.
- `run-d-8`: d-8 adds async sqlx MySQL code only; it introduces no blocking bridge and does not touch the SQLite writer thread.
- `run-d-12`: d-12 changes release manifests, Python release/test wiring and the external beads-cache consumer, not production Rust blocking/runtime code under crates/ or template/; d-6 and d-9 own and satisfy that ADR behavior, verified by official coverage and the targeted relevance answer.

Beads: anchor `run-n3n7`, plan questions `run-n3n7.1`, dev questions `run-n3n7.2`

Fixtures:
- pass: Runtime::block_on appears only in the SQLite writer thread that owns its runtime; reader threads receive with blocking_recv.
- fail: Reader-pool threads each build a current-thread runtime and call Runtime::block_on to receive requests.

## ANC-033

Anchor: The storage contract MUST provide one generic operation that runs several row operations as a single atomic unit, implemented by each engine (SQLite, Postgres, MySQL).

> Storage MUST provide one generic way to run several row operations as a single atomic unit, supplied by each engine.

Said by randlee, storage contract direction: atomic transactions, cron feature withdrawn, 2026-11-12. Status: locked. Version 2.

Summary: generic atomic unit per engine

Scope: plan items that define or change the storage contract (Storage<T>, sc-storage) or implement a storage engine (SQLite writer, Postgres or MySQL store)

Deterministic check to pair with the development gate: sc-storage exposes the atomic-unit operation; SQLite, Postgres and MySQL implementations each pass one shared conformance test where a failing operation rolls back the earlier ones

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-10`: d-10 consumes the storage contract; the generic atomic operation is defined in d-5 and implemented by d-6, d-7 and d-8.
- `run-d-12`: The generic atomic operation is defined in d-5, implemented in d-6, d-7, d-8 and exercised in d-11; d-12 publishes the crates.
- `run-d-9`: d-9 only consumes AtomicStorage::transaction in sc-cron; the contract method is defined in d-5 and implemented per engine in d-6/d-7/d-8.

Beads: anchor `run-gar9`, plan questions `run-gar9.1`, dev questions `run-gar9.2`

Fixtures:
- pass: sc-storage defines Storage::transaction(name, producer), whose body reads and writes rows atomically; the SQLite writer runs it in one savepoint and the Postgres and MySQL stores run it in one sqlx begin/commit, all passing one shared rollback conformance test.
- fail: Storage<T> offers only read, read_all and write; callers that need several rows changed together issue separate writes, and only the SQLite backend gets a bespoke batch method.

## ANC-034

Anchor: Storage backend crates MUST NOT include any type, schema or SQL that is specific to one consumer's table (for example cron rows).

> No storage backend contains code for a specific consumer's table.

Said by randlee, storage contract direction: atomic transactions, cron feature withdrawn, 2026-11-12. Status: locked. Version 3.

Summary: no consumer table code in storage backends

Scope: plan items that change a storage backend crate (sc-runtime-db-sqlite, template store-sqlite/store-postgres/store-mysql)

Deterministic check to pair with the development gate: grep storage backend crates for CronRow, cron table DDL or cron SQL: no hits; cargo tree -p sc-runtime-db-sqlite --all-features has no sc-cron

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-10`: d-10 touches no storage backend crate; its application stores live in the application layer.
- `run-d-12`: Backend neutrality is owned by d-5 through d-8 and verified in d-11; d-12 publishes the crates.
- `run-d-3`: d-3 only edits dependency ranges and CI jobs; keeping consumer-specific code out of storage backend crates is owned by d-5 through d-8.

Beads: anchor `run-q03u`, plan questions `run-q03u.1`, dev questions `run-q03u.2`

Fixtures:
- pass: sc-runtime-db-sqlite implements Storage<T> generically over a row codec; CronRow, its table definition and the fencing SQL live in sc-cron, which uses the generic transaction.
- pass: sc-cron's compiled-hook blob table is sc-cron's own row type stored through Storage<T>; no backend gains code for it.
- fail: sc-runtime-db-sqlite adds an optional cron feature with a cron_jobs table definition and fenced record_run SQL for CronRow.

## ANC-035

Anchor: Public storage functions MUST NOT panic.

> public storage functions return `Result<T, E>` with a typed error and do not panic

Said by randlee, storage contract direction: atomic transactions, cron feature withdrawn, 2026-11-12. Status: locked. Version 3.

Summary: public storage functions do not panic

Scope: plan items that write or change public storage functions in this phase, in any crate: storage traits and engine backends (such as sc-storage and sc-runtime-db-sqlite), domain stores built on generic storage (such as sc-cron's CronStore/CronStorage), and template stores

Deterministic check to pair with the development gate: sc-lint panic rule (randlee/sc-lint#931) clean on changed storage crates

Beads: anchor `run-vjhq`, plan questions `run-vjhq.1`, dev questions `run-vjhq.2`

Fixtures:
- pass: Every public function in sc-storage and the SQLite writer returns a Result with a typed error; invalid input and closed channels become error variants, and the crates deny unwrap and expect.
- fail: The writer's public send() calls .expect("writer channel closed") and the store helper unwraps the decoded row.

## ANC-036

Anchor: If the SQLite writer thread panics, every caller waiting on it MUST receive an error rather than waiting indefinitely.

> a panic on the writer thread reaches every waiting caller as an error, never a hang

Said by randlee, storage contract direction: atomic transactions, cron feature withdrawn, 2026-11-12. Status: locked. Version 2.

Summary: writer panic reaches waiting callers as error

Scope: plan items that implement or change the SQLite writer thread, its request channel or its reply path

Deterministic check to pair with the development gate: deterministic test: a request that panics on the writer thread makes every pending caller receive an error; no await hangs (no sleeps)

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-11`: Writer-panic delivery to every waiting caller is implemented and tested in d-6; d-11 only composes the writer into generated apps.
- `run-d-12`: d-12 migrates beads-cache onto and publishes the 0.3.0 crates; SQLite writer behaviour is implemented and tested in d-6 and composed in d-11.
- `run-d-5`: d-5 defines the contract only; the SQLite writer thread and its panic delivery to waiting callers are implemented and tested in d-6.

Beads: anchor `run-cdtx`, plan questions `run-cdtx.1`, dev questions `run-cdtx.2`

Fixtures:
- pass: The writer thread runs each request under catch_unwind; on panic it fails the request and drains the channel, so every queued caller receives StorageError::WriterPanicked.
- fail: If a request panics, the writer thread exits and queued callers' oneshot replies are never sent, so their awaits never complete.

## ANC-037

Anchor: Registry dependency requirements in published crate and template manifests MUST NOT declare an upper version bound.

> Remove upper bound / If a security patch is tommorrow I won't be able to use it / The consumer can pin what they want. / All dependencies

Said by randlee, Phase D planning, m6 owner decision 2026-11-12, 2026-11-11. Status: locked. Version 2.

Summary: No upper bounds on any registry dependency

Scope: Adds, changes or specifies registry dependency requirements in published crate manifests or template manifests.

Deterministic check to pair with the development gate: Parse every published-crate and template Cargo.toml registry requirement; fail on any '<', '<=', '=', '~' or caret requirement (caret implies an upper bound).

Beads: anchor `run-svmi`, plan questions `run-svmi.1`, dev questions `run-svmi.2`

Fixtures:
- pass: Every registry requirement in crates/sc-storage/Cargo.toml and the template manifests is a floor such as rusqlite = ">=0.41" with no upper bound.
- fail: The template keeps rusqlite = "^0.41" and sc-observability = "=1.5.0", so consumers cannot select 0.42 or 1.6.

## ANC-038

Anchor: The generated application MUST record the release's tested dependency combination as metadata under .sc-runtime/, readable by tools that revert to it or upgrade from it at the next release.

> This should also be saved as metadata in .sc-runtime/ fit reference or tools to revert/upgrade on next release

Said by randlee, Phase D planning, m6 owner decision 2026-11-12, 2026-11-11. Status: locked. Version 4.

Summary: Tested dependency combination recorded in .sc-runtime/

Scope: Plans what the generated application writes under .sc-runtime/, or the generation step or tool that writes or reads the recorded tested dependency versions there. Work that only sets or tests dependency ranges is outside this scope.

Deterministic check to pair with the development gate: Generate an application; assert a .sc-runtime/ metadata file lists every locked dependency version and matches the shipped Cargo.lock.

Excluded from (not judged; the cell passes as `excluded`):
- `run-d-11`: d-11 composes application code and invokes the manifest checker; recommendation generation, template/.sc-runtime/** and the script that reads/writes tested-version metadata are entirely d-12-owned. d-11 does not implement a metadata-writing/reading generation step.
- `run-d-5`: d-5 defines storage contracts and registry floors, not .sc-runtime/ metadata or the generation step/tools that read/write recorded tested versions; d-12 owns all recommendation production.
- `run-d-10`: d-10 changes generic commands/transforms and registry floors, not .sc-runtime/ metadata or its generation reader/writer; d-12 owns the metadata path and recommendation script.
- `run-d-9`: d-9 implements cron policy, migration and registry dependency floors; it does not write/read .sc-runtime/ tested-version metadata or its generation tools. d-12 owns that metadata and recommendation production.

Beads: anchor `run-ongv`, plan questions `run-ongv.1`, dev questions `run-ongv.2`

Fixtures:
- pass: Generation writes .sc-runtime/tested-dependencies.json listing every dependency version the release tested, matching the shipped Cargo.lock, in a machine-readable form a later revert or upgrade tool can read.
- fail: The generated application records no tested dependency versions; upgrades rely only on cargo update.

## ANC-039

Anchor: The sc-monty crate MUST NOT have a normal or build dependency on any sc-* crate, tokio or serde_json.

> Depends on the pydantic Monty crates only. No sc-* crates, no tokio, no serde_json

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: sc-monty depends only on Monty crates

Scope: plan items that create or change the sc-monty crate or its Cargo.toml

Deterministic check to pair with the development gate: cargo tree -p sc-monty -e normal,build --all-features shows no sc-*, tokio or serde_json crate

Beads: anchor `run-yaji`, plan questions `run-yaji.1`, dev questions `run-yaji.2`

Fixtures:
- pass: sc-monty's Cargo.toml lists only the pydantic Monty crates as normal dependencies.
- fail: sc-monty adds tokio and serde_json so it can run hook ticks and parse JSON.

## ANC-040

Anchor: The sc-monty crate MUST NOT define cron types or cron-reserved names such as cron_tick and cron_stop.

> no cron types or cron-reserved names.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: sc-monty has no cron types or names

Scope: plan items that create or change the sc-monty crate's public types or names

Deterministic check to pair with the development gate: grep crates/sc-monty/src for Cron, cron_tick, cron_stop: no hits

Beads: anchor `run-4711`, plan questions `run-4711.1`, dev questions `run-4711.2`

Fixtures:
- pass: sc-monty exposes a compiled session and MontyInvoke with no cron-specific types; cron_tick and cron_stop are named only in sc-cron.
- fail: sc-monty defines CronHook with cron_tick and cron_stop entry points.

## ANC-041

Anchor: No crate other than sc-monty MUST compile Monty code, run Monty sessions or dispatch Monty host functions.

> the duplication of monty logic was the biggest red flag.  I expect single monty implementation

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: one Monty implementation, in sc-monty

Scope: plan items that create sc-monty or change Monty hosting in sc-cron, sc-command, the template service or any other crate

Deterministic check to pair with the development gate: grep crates and template sources outside crates/sc-monty for use of the monty/monty-types crates: no hits; sc-cron and the transform caller depend on sc-monty

Beads: anchor `run-7yi7`, plan questions `run-7yi7.1`, dev questions `run-7yi7.2`

Fixtures:
- pass: sc-cron's hooks and the d-10 command transform both call sc-monty; neither imports the monty crate directly.
- fail: The template service writes its own Monty host with its own compile and limit handling instead of calling sc-monty.

## ANC-042

Anchor: Moving sc-cron's Monty hooks onto sc-monty MUST NOT delete or weaken any existing hook behavior test.

> Hook behavior and tests are ported, not deleted or weakened

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: cron hook tests ported, not weakened

Scope: plan items that change sc-cron's Monty hooks or move Monty hosting out of sc-cron

Deterministic check to pair with the development gate: every pre-existing sc-cron Monty hook test name is present after the change and passes with unchanged assertions

Beads: anchor `run-yn8d`, plan questions `run-yn8d.1`, dev questions `run-yn8d.2`

Fixtures:
- pass: sc-cron calls sc-monty for hooks and every existing hook test is ported and passes unchanged.
- fail: sc-cron drops the hook tests that sc-monty's own tests are said to cover.

## ANC-043

Anchor: sc-cron MUST pass hook data to its Monty hooks as Monty values, not as JSON strings.

> JSON conversion and serde_json stay in sc-cron. Hook behavior and tests are ported, not deleted or weakened; hooks keep receiving values, not JSON strings.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: hooks receive values, not JSON strings

Scope: plan items that change how sc-cron passes data to Monty hooks

Deterministic check to pair with the development gate: sc-cron hook calls pass Monty values; no hook entry receives a str/String of JSON

Beads: anchor `run-zerf`, plan questions `run-zerf.1`, dev questions `run-zerf.2`

Fixtures:
- pass: sc-cron converts the schedule's JSON to Monty values and passes those to the hook.
- fail: sc-cron passes the hook its JSON data as a string for the script to parse.

## ANC-044

Anchor: sc-cron MUST save each successfully compiled hook session as a blob row in an sc-cron table stored through Storage<T>.

> When a hook's source compiles, sc-cron saves the compiled session as a blob in a table that is part of sc-cron, a row type through Storage<T>.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: compiled hook sessions saved as an sc-cron row

Scope: plan items that change sc-cron hook compilation or sc-cron's storage tables

Deterministic check to pair with the development gate: sc-cron defines a compiled-hook row type stored through Storage<T>; no backend crate gains code for it

Beads: anchor `run-te1w`, plan questions `run-te1w.1`, dev questions `run-te1w.2`

Fixtures:
- pass: After a hook compiles, sc-cron writes the dumped session as a CompiledHookRow through Storage<CompiledHookRow>.
- fail: sc-cron compiles each hook on the first run of every generation and keeps no compiled session.

## ANC-045

Anchor: sc-cron MUST key each stored compiled Monty hook session by the SHA-256 of the hook's source text, computed with the sc-sha crate, combined with Monty's dump format version.

> Key: SHA-256 of the source, computed with the sc-sha crate, plus Monty's dump version.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: blob key = sc-sha SHA-256 of source + dump version

Scope: plan items that change how sc-cron stores or looks up compiled Monty hook sessions (the saved result of compiling a cron hook script)

Deterministic check to pair with the development gate: the compiled-hook row key is computed with sc-sha over the hook source plus the Monty dump version; a test changes either and gets a miss

Beads: anchor `run-nd6j`, plan questions `run-nd6j.1`, dev questions `run-nd6j.2`

Fixtures:
- pass: The compiled-hook row is keyed by sc-sha's SHA-256 of the hook source plus Monty's dump version.
- fail: The compiled-hook row is keyed by schedule name only.

## ANC-046

Anchor: sc-cron MUST use a stored compiled hook session only when its key matches the hook source text in the schedule row.

> The source text in the schedule row stays the source of truth. A blob is used only when its key matches that source.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: blob used only when its key matches the source

Scope: plan items that change how sc-cron loads compiled hook sessions

Deterministic check to pair with the development gate: a test edits the schedule's hook source and shows the old blob is not loaded

Beads: anchor `run-n2ay`, plan questions `run-n2ay.1`, dev questions `run-n2ay.2`

Fixtures:
- pass: After a daemon restart, a schedule whose source is unchanged loads its blob and runs its next tick without recompiling; a changed source recompiles.
- fail: sc-cron loads the stored blob for the schedule name even after its source changed.

## ANC-047

Anchor: sc-cron MUST capture a compiled hook session's blob immediately after compile, before the hook runs.

> The blob is captured immediately after compile, before any run. No run state is persisted.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: blob captured right after compile, no run state

Scope: plan items that change when or what sc-cron stores for a compiled hook session

Deterministic check to pair with the development gate: the blob is written in the compile path before the first tick; stored bytes equal a fresh compile of the same source after runs

Beads: anchor `run-zjoo`, plan questions `run-zjoo.1`, dev questions `run-zjoo.2`

Fixtures:
- pass: sc-cron dumps the session right after compile, before the first tick, so the blob never holds run state.
- fail: sc-cron dumps the session after each tick so the next run resumes from saved state.

## ANC-048

Anchor: sc-cron MUST reject a schedule write whose hook source does not compile.

> Compile when the schedule is written, so a script that does not compile is rejected at submit instead of at its first tick.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: hooks compile when the schedule is written

Scope: plan items that change sc-cron schedule creation, replacement or hook compilation

Deterministic check to pair with the development gate: a test submits a schedule whose hook does not compile and gets a typed error with no row written

Beads: anchor `run-vi2g`, plan questions `run-vi2g.1`, dev questions `run-vi2g.2`

Fixtures:
- pass: Adding a schedule compiles its hook; a script with a syntax error is rejected at submit with a typed error and no row is written.
- fail: A schedule with a script that does not compile is accepted and fails at its first tick.

## ANC-049

Anchor: The source of a Monty command transform MUST be replaceable without rebuilding the application.

> the source of a Monty command transform is loaded at run time and can be replaced without rebuilding the app.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: Monty transform source is run-time data

Scope: plan items that add or change a Monty or Python command transform or its caller

Deterministic check to pair with the development gate: a test replaces the transform source at run time, without rebuilding, and the next command uses the new source

Beads: anchor `run-y0v1`, plan questions `run-y0v1.1`, dev questions `run-y0v1.2`

Fixtures:
- pass: The service loads the widget transform's Monty source at run time; replacing it changes the next command's result without a rebuild.
- fail: The Monty transform source is embedded in the binary with include_str!.

## ANC-050

Anchor: Documentation for Transform, storage_read and storage_write MUST state that they are not yet stable.

> Transform, storage_read and storage_write are documented as not yet stable.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: command seam documented as not yet stable

Scope: plan items that define, document or publish Transform, storage_read or storage_write

Deterministic check to pair with the development gate: sc-command rustdoc and README mark Transform, storage_read and storage_write as not yet stable

Beads: anchor `run-25z5`, plan questions `run-25z5.1`, dev questions `run-25z5.2`

Fixtures:
- pass: The sc-command README and rustdoc state that Transform, storage_read and storage_write are provisional in 0.3.0 and may change.
- fail: The sc-command README presents storage_write(store, transform, input) as the stable command API.

## ANC-051

Anchor: Phase D plan, ADR and REQ text MUST NOT prohibit later adding a second transform input, a no-write command outcome, or a transform inside an atomic transaction.

> Build none of these now. Forbid none of them.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: no prohibition of the three seam extensions

Scope: plan items, ADRs or REQs that define or constrain Transform, storage_read, storage_write or transaction bodies

Deterministic check to pair with the development gate: plan, ADR and REQ text has no rule forbidding a second transform input, a no-write outcome or a transform inside a transaction; such items read as not built in phase-d

Beads: anchor `run-dz6v`, plan questions `run-dz6v.1`, dev questions `run-dz6v.2`

Fixtures:
- pass: Phase D does not build a transform inside a transaction; that remains possible later.
- fail: Transaction bodies may not call the transform mechanism.

## ANC-052

Anchor: The signatures of Transform, storage_read and storage_write MUST allow a second transform input, a no-write outcome and a transform inside an atomic transaction to be added later without breaking consumers.

> Their shape must not prevent adding later, without breaking consumers: (a) a second input to the transform, (b) an outcome that writes nothing, (c) a transform that runs inside an atomic transaction.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: seam shape leaves room for three extensions

Scope: plan items that define the signatures of Transform, storage_read or storage_write

Deterministic check to pair with the development gate: design review of the Transform/storage_read/storage_write signatures shows each extension can be added as a new item without changing existing signatures

Beads: anchor `run-cz0z`, plan questions `run-cz0z.1`, dev questions `run-cz0z.2`

Fixtures:
- pass: storage_write returns a typed Result whose Ok can later carry a no-write outcome, and Transform is a trait that can gain a second-input form without changing existing impls.
- fail: storage_write(store, transform, input) is a fixed function that always writes, with no way to add a no-write outcome without a breaking change.

## ANC-053

Anchor: The plan MUST name which cron commands go through the shared command path.

> Name which cron commands go through the shared path

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: plan names cron commands on the shared path

Scope: plan items that change cron commands, sc-cron-commands or the shared command path

Deterministic check to pair with the development gate: the sprint text lists each cron command as on or off the shared path

Beads: anchor `run-1am3`, plan questions `run-1am3.1`, dev questions `run-1am3.2`

Fixtures:
- pass: Cron add and replace go through the shared path: command type in, transform, CronRow written through Storage<CronRow>.
- fail: Cron commands keep their own storage calls and the plan does not say whether any use the shared path.

## ANC-054

Anchor: The sprint text MUST state a reason for each cron command that does not go through the shared command path.

> Name which cron commands go through the shared path; for any that do not, state the reason in the sprint text.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: reason stated for cron commands off the path

Scope: plan items that change cron commands, sc-cron-commands or the shared command path

Deterministic check to pair with the development gate: each cron command listed off the shared path has a reason in the sprint text

Beads: anchor `run-6wp9`, plan questions `run-6wp9.1`, dev questions `run-6wp9.2`

Fixtures:
- pass: Cron run-now does not use the shared path because it writes no schedule row; the sprint says so.
- fail: Cron stop is handled outside the shared path with no reason given.

## ANC-055

Anchor: Before a cron schedule row is written, a consuming application MUST be able to modify or append to the JSON data the end user supplied with the schedule.

> an end user schedules cron with a Monty or Python script plus JSON data that is passed to the script. A consuming app can modify or append to that JSON before it is written. That step is the transform.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: apps can modify cron JSON before write

Scope: plan items that change cron schedule creation, cron commands or the command transform

Deterministic check to pair with the development gate: a test supplies an app transform that appends a field to the user's schedule JSON; the written CronRow contains it

Beads: anchor `run-qjv1`, plan questions `run-qjv1.1`, dev questions `run-qjv1.2`

Fixtures:
- pass: An app transform appends its own fields to the user's schedule JSON, and the CronRow is then written through Storage<CronRow>.
- fail: Cron writes the user's JSON unchanged; an app cannot add its own data before the row is written.

## ANC-056

Anchor: A command whose Tcli type converts directly to its Tsql row type MUST NOT need a hand-written transform.

> A command whose Tcli converts directly to Tsql needs no hand-written transform.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: direct conversions need no hand-written transform

Scope: plan items that define the command transform (Tcli is the type an interface speaks, Tsql the stored row), its default or the template's commands

Deterministic check to pair with the development gate: a test command whose Tcli and Tsql have the same fields runs through storage_write with the default transform and no app-written transform

Beads: anchor `run-w5ju`, plan questions `run-w5ju.1`, dev questions `run-w5ju.2`

Fixtures:
- pass: When Tcli converts into Tsql, the default transform is that conversion and the app writes no transform.
- fail: Every command must supply an application-written transform.

## ANC-057

Anchor: ADRs and REQs MUST NOT state that the command path must be hand-written.

> No ADR or REQ may state that the command path must be hand-written.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 3.

Summary: no REQ/ADR binds hand-writing

Scope: plan items that add or amend ADRs or REQs about the command path or transforms

Deterministic check to pair with the development gate: grep docs/requirements.md, docs/architecture.md and per-crate docs for wording requiring hand-written commands: none outside history notes

Beads: anchor `run-z29j`, plan questions `run-z29j.1`, dev questions `run-z29j.2`

Fixtures:
- pass: Every ADR and REQ, including ADR-RUN-0001 and ADR-RUN-0021, describes hand-written command code only as the interim form; none states that the command path must be hand-written.
- fail: An ADR states that every operation is hand-written in project code.

## ANC-058

Anchor: ADRs and REQs MUST NOT contain wording that forbids a consumer-side code generator for the command path.

> Scan the REQ/ADR set for any other wording that binds hand-writing or forbids generators.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: no REQ/ADR forbids a consumer-side generator

Scope: plan items that add or amend ADRs or REQs about code generation or the command path

Deterministic check to pair with the development gate: grep the REQ/ADR set for wording forbidding generators; any no-generator rule is limited to sc-runtime's own crates

Beads: anchor `run-6nsz`, plan questions `run-6nsz.1`, dev questions `run-6nsz.2`

Fixtures:
- pass: REQ-RUN-0318 says crates/sc-runtime ships no generator and does not restrict a generator in a consuming project.
- fail: A REQ states that command code must never be generated.

## ANC-059

Anchor: The plan MUST state which of the widget create, get and delete commands go through storage_read or storage_write.

> state which of the widget create/get/delete commands go through storage_read/storage_write.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 4.

Summary: plan names widget commands on the path

Scope: plan items that change the template's widget commands or service

Deterministic check to pair with the development gate: the sprint text names which of widget create/get/delete use storage_read or storage_write

Beads: anchor `run-8xhu`, plan questions `run-8xhu.1`, dev questions `run-8xhu.2`

Fixtures:
- pass: Of the widget commands, create goes through storage_write and get goes through storage_read; delete goes through neither and stays on the WidgetStorage extension.
- fail: The plan routes widget commands through WidgetStorage without saying which use storage_read or storage_write.

## ANC-060

Anchor: At least one write command of the template's widget example MUST go through storage_write.

> state which of the widget create/get/delete commands go through storage_read/storage_write. At least one write and one read must.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: a widget write uses storage_write

Scope: plan items that change the template's widget commands or service

Deterministic check to pair with the development gate: generated service tests show a widget write command calling storage_write

Beads: anchor `run-v3j3`, plan questions `run-v3j3.1`, dev questions `run-v3j3.2`

Fixtures:
- pass: Widget create goes through storage_write.
- fail: Widget create and delete both call WidgetStorage extension methods; no write uses storage_write.

## ANC-061

Anchor: Production release crates MUST NOT have a normal or build dependency on any crate from the sc-compose repository other than sc-sha, or on any ATM crate.

> sc-sha is published from the sc-compose repo. Owner ruling: sc-sha is permitted as a production dependency; ANC-012 covers the sc-compose composition crates and ATM crates.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: no sc-compose repo crates except sc-sha, no ATM crates

Scope: Changes normal/build dependency declarations or resolved production dependency graphs of published sc-runtime crates. Crates from the sc-compose repository include sc-compose, sc-composer, sc-composer-beads and sc-sha. ATM denotes the agent-team-mail workspace crates covered by ADR-RUN-0013, including agent-team-mail and atm-core. Dev-only test and benchmark references are excluded.

Deterministic check to pair with the development gate: cargo tree -e normal,build for each published crate and supported feature set shows no sc-compose, sc-composer, sc-composer-beads, agent-team-mail or atm-* crate; sc-sha may appear

Beads: anchor `run-sxwd`, plan questions `run-sxwd.1`, dev questions `run-sxwd.2`

Fixtures:
- pass: sc-cron depends on sc-sha for the hook source hash and has no other sc-compose repository crate or ATM crate in its normal or build graph.
- fail: A production release crate adds sc-composer as a normal dependency to render templates.

## ANC-062

Anchor: sc-monty MUST begin as an exact copy of sc-cron's current Monty code and its unit and integration tests, passing, before its API is extended.

> monty implementation should start w/ an exact copy of current code AND tests including integration tests.  It is important to preserve the current excellent work.  then monty api can be extended as necessary to addres the requirements

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: sc-monty starts as an exact copy of current Monty code and tests

Scope: plan items that create sc-monty or move Monty code or tests out of sc-cron

Deterministic check to pair with the development gate: the first sc-monty commit's Monty sources and tests match sc-cron's monty_run.rs/monty_host.rs and their unit and integration tests (diff limited to crate paths and names); they pass before any API change

Beads: anchor `run-7ejc`, plan questions `run-7ejc.1`, dev questions `run-7ejc.2`

Fixtures:
- pass: d-5 first copies sc-cron's Monty modules and every Monty unit and integration test into sc-monty unchanged and makes them pass; the MontyInvoke extension follows.
- fail: sc-monty is written fresh against a new API and only the tests it needs are rewritten.

## ANC-063

Anchor: At least one read command of the template's widget example MUST go through storage_read.

> state which of the widget create/get/delete commands go through storage_read/storage_write. At least one write and one read must.

Said by randlee, owner requirement changes against plan/phase-d 5cffe24, 2026-11-12, 2026-11-12. Status: locked. Version 2.

Summary: a widget read uses storage_read

Scope: plan items that change the template's widget commands or service

Deterministic check to pair with the development gate: generated service tests show a widget read command calling storage_read

Beads: anchor `run-50cw`, plan questions `run-50cw.1`, dev questions `run-50cw.2`

Fixtures:
- pass: Widget get goes through storage_read.
- fail: Widget get calls a WidgetStorage extension method; no read uses storage_read.
