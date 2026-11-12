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
| ANC-012 | locked | Generation and orchestration tools stay outside production graphs | 2026-11-11 |
| ANC-013 | retired | #131 consumer router docs | 2026-11-11 |
| ANC-014 | retired | Dedicated plan PR | 2026-11-11 |
| ANC-015 | locked | Skill checks accept versions above the minimum | 2026-11-11 |
| ANC-016 | locked | Consumers can take compatible dependency upgrades | 2026-11-11 |
| ANC-017 | locked | CI proves dependency range endpoints | 2026-11-11 |
| ANC-018 | retired | Advisory proof uses the released consumer graph | 2026-11-11 |
| ANC-019 | locked | Additional upper bounds explain their known break | 2026-11-11 |
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

Said by randlee, Phase D planning session 2026-11-11, 2026-11-11. Status: locked. Version 4.

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

Beads: anchor `run-tngz`, plan questions `run-tngz.1`, dev questions `run-tngz.2`

Fixtures:
- pass: The skill requires sc-compose >=1.8.0 and accepts installed sc-compose 1.8.1.
- fail: The skill rejects installed sc-compose 1.8.1 because its configured minimum is exactly 1.8.0.

## ANC-016

Anchor: Published crate and template dependency requirements MUST allow consumers to select newer Cargo SemVer-compatible dependency releases.

> we need to be following 'best practices' for a scaffolding framework which does not block consumers from building or upgrading dependencies w/ non-breaking changes/upgrades.

Said by randlee, User planning handover in this conversation, 2026-11-11; revised intent.md Purpose and Dependency policy direction. Proposed interpretation: normal Cargo-compatible ranges, with known incompatibilities justified separately; blanket unbounded >= is not implied., 2026-11-11. Status: locked. Version 4.

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

Said by randlee, randlee-authored GitHub issue #133, Requested change item 3: https://github.com/randlee/sc-runtime/issues/133; same method required in #132., 2026-11-11. Status: locked. Version 3.

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

Said by randlee, randlee-authored GitHub issue #133, Requested change item 2 and Done when: https://github.com/randlee/sc-runtime/issues/133. This candidate captures justification of extra restrictions; it does not adopt the older blanket >= prescription., 2026-11-11. Status: locked. Version 4.

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

Beads: anchor `run-kaze`, plan questions `run-kaze.1`, dev questions `run-kaze.2`

Fixtures:
- pass: The primary CI installation path fetches the selected sc-compose GitHub Release archive and verifies its checksums.txt entry.
- fail: The primary CI installation path recompiles sc-compose with cargo install against the current crates.io index.

## ANC-021

Anchor: In an app that has a SQLite database and enables cron, the default configuration MUST store cron rows in the app's SQLite database file.

> When an app has a SQLite database and enables cron, cron rows MUST by default be stored in that same SQLite database file.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: cron rows default to app SQLite DB

Scope: plan items that change sc-cron storage, cron persistence configuration, or the generated template's SQLite store or cron wiring

Deterministic check to pair with the development gate: generated SQLite+cron app: cron schedule table exists in the app database file and no other .db file is created under the instance root (test)

## ANC-022

Anchor: All writes to an app's SQLite database MUST be submitted to a single writer through an async channel.

> Every write to an app's SQLite database MUST go through one writer with an async channel input

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: all SQLite writes via one async-channel writer

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: grep: no SQLite write path outside the writer; writer API takes an async channel sender

## ANC-023

Anchor: The app's only SQLite write connection MUST be owned by the SQLite writer thread.

> a single thread that owns the only write connection

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: writer thread owns the only write connection

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: grep: one write-mode Connection::open in the writer thread; other connections opened read-only

## ANC-024

Anchor: The SQLite writer MUST reply to each write request only after the transaction containing that write has committed.

> Every write to an app's SQLite database MUST go through one writer with an async channel input, a single thread that owns the only write connection, and batched transactions that reply after commit. No caller blocks or uses spawn_blocking to reach it.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: writer replies after commit

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: deterministic test: reply observed only after commit hook fires

## ANC-025

Anchor: The SQLite writer MUST commit writes that are queued together in a single transaction.

> Every write to an app's SQLite database MUST go through one writer with an async channel input, a single thread that owns the only write connection, and batched transactions that reply after commit. No caller blocks or uses spawn_blocking to reach it.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: queued writes batched in one transaction

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: deterministic test: N writes queued before drain commit in one transaction (commit hook count 1)

## ANC-026

Anchor: Callers MUST NOT use spawn_blocking to submit writes to the SQLite writer.

> Every write to an app's SQLite database MUST go through one writer with an async channel input, a single thread that owns the only write connection, and batched transactions that reply after commit. No caller blocks or uses spawn_blocking to reach it.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: no spawn_blocking to reach writer

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: grep: no spawn_blocking in callers of the SQLite writer

## ANC-027

Anchor: Async callers MUST submit writes to the SQLite writer without blocking their thread while waiting for the reply.

> Every write to an app's SQLite database MUST go through one writer with an async channel input, a single thread that owns the only write connection, and batched transactions that reply after commit. No caller blocks or uses spawn_blocking to reach it.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: async callers do not block awaiting writer reply

Scope: plan items that implement or change a SQLite storage backend, the SQLite writer, or the generated template's SQLite store

Deterministic check to pair with the development gate: grep: writer submit API is async (await on oneshot), no blocking recv in async callers

## ANC-028

Anchor: The cargo tree of an app that uses SQLite MUST NOT include a sqlx SQLite driver crate.

> An app with SQLite MUST link exactly one SQLite client library: cargo tree shows rusqlite, no sqlx SQLite driver, and one libsqlite3-sys version.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: no sqlx SQLite driver in SQLite app tree

Scope: plan items that change SQLite-related dependencies, the generated template's SQLite store, or sc-cron's storage dependencies

Deterministic check to pair with the development gate: cargo tree -e normal on generated SQLite app: no sqlx-sqlite

## ANC-029

Anchor: Template generation MUST reject a project that enables cron with the no-database layout.

> Cron needs a database. Generation MUST reject cron with the no-database layout.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: generation rejects cron with no-database layout

Scope: plan items that change template generation options, layouts, or the cron feature

Deterministic check to pair with the development gate: generation test: cron enabled + no-database layout exits non-zero with a typed error; no project written

## ANC-030

Anchor: When sc-cron's schedule rows move from a separate cron database file into the application's database, sc-cron MUST still reject a second active schedule row that has the same schedule name as an active row.

> Moving the cron table MUST NOT drop one-active-schedule-per-name, the row limit, or generation fencing; their tests are ported, not deleted.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: cron keeps one active row per schedule name

Scope: plan items that move or change cron storage, CronStorage, sc-cron's schedule store, or cron tests

Deterministic check to pair with the development gate: ported sc-cron test: unique schedule-name constraint/behaviour test passes against the app-database CronStorage

## ANC-031

Anchor: When sc-cron's schedule rows move from a separate cron database file into the application's database, sc-cron MUST still refuse to add a schedule row once the row count reaches the row limit configured by the embedding application.

> Moving the cron table MUST NOT drop one-active-schedule-per-name, the row limit, or generation fencing; their tests are ported, not deleted.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: cron keeps caller-supplied row limit

Scope: plan items that move or change cron storage, CronStorage, sc-cron's schedule store, or cron tests

Deterministic check to pair with the development gate: ported sc-cron row-limit admission tests pass against the app-database CronStorage

## ANC-032

Anchor: The application storage backends for SQLite, Postgres and MySQL MUST all implement one shared Rust storage trait, Storage<T>.

> Storage is behind one trait. SQLite, Postgres and MySQL MUST implement the same storage trait.

Said by Rand Lee, Phase D planning session; option B storage decision, 2026-11-11. Status: locked. Version 1.

Summary: SQLite/Postgres/MySQL implement one Storage<T>

Scope: plan items that define the storage trait or implement a SQLite, Postgres or MySQL storage backend

Deterministic check to pair with the development gate: grep: impl Storage<..> for each of the SQLite, Postgres and MySQL backends; same trait path

## ADR-RUN-0011

Anchor: Production code under crates/ and template/ MUST NOT call Runtime::block_on, Handle::block_on or futures::executor::block_on, except the runtime owned and driven on the SQLite writer's dedicated OS thread; SQLite reader threads receive with blocking_recv and need no exception.

> Production code under crates/ and template/ MUST NOT call Runtime::block_on, Handle::block_on or futures::executor::block_on, except the runtime owned and driven on the SQLite writer's dedicated OS thread; SQLite reader threads receive with blocking_recv and need no exception.

Said by randlee, planning session, 2026-11-11. Status: locked. Version 1.

Summary: block_on banned except SQLite writer thread (ADR-RUN-0011 pt1, amended d9dd55e)

Scope: plan items that change async/blocking rules, the SQLite writer or reader pool, or code that bridges sync and async

Deterministic check to pair with the development gate: grep -rn 'block_on' crates/ template/ --include=*.rs excluding tests: only hit is inside the SQLite writer thread module
