---
id: d-1
phase: d
status: planned
closure_type: integration
target_boundary: runtime-ecosystem-dependency-compatibility
track: dependency-compatibility
wave: 1
branch: sprint/d-1-dependency-compatibility
worktree: sprint/d-1-dependency-compatibility
pr_target: integrate/phase-d
vertical_rationale: One maintenance defect spans the native SQLite link identity, MCP public dependency types, all manifest ranges, and their CI resolves. One owner updates shared manifests and lockfiles atomically; splitting these files across parallel boundary sprints creates conflicts rather than independent work. No new first-party contract is introduced.
recommended_agent: arch-ctm
recommended_model: terra
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0005
  - REQ-RUN-0301
  - REQ-RUN-0303
  - REQ-RUN-0701
  - REQ-CMD-0002
  - REQ-CMD-0005
  - REQ-DBSQ-0001
  - REQ-CRON-0001
  - NFR-RUN-0007
  - NFR-CFG-0002
  - NFR-TRN-0001
  - NFR-CMD-0001
  - NFR-CMD-0002
  - NFR-RT-0001
  - NFR-RT-0003
  - NFR-CRON-0001
  - NFR-CRNC-0001
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0013
  - ADR-RUN-0017
  - ADR-RUN-0203
  - ADR-CFG-0003
  - ADR-TRN-0001
  - ADR-TRN-0004
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CMD-0003
  - ADR-RT-0003
  - ADR-CRNC-0001
github_issues: [132, 133]
parallel_safe:
  - d-2
  - d-4
parallel_safe_rationale: d-2 owns README bodies; d-4 owns the generator driver, tool action and version checks. This sprint alone edits shared Justfile/crates.yml/template.yml; their existing action inputs stay unchanged. All owned paths are disjoint.
must_follow: []
owned_paths:
  - Cargo.toml
  - Cargo.lock
  - crates/**/Cargo.toml
  - crates/**/Cargo.lock
  - crates/**/src/**
  - crates/**/tests/**
  - template/Cargo.toml
  - template/workspace.template.toml
  - template/Cargo.lock
  - template/crates/**/Cargo.toml
  - template/crates/**/Cargo.*.toml
  - template/crates/**/Cargo.lock
  - template/crates/**/src/**
  - tests/contracts/**
  - tests/integration/**
  - tests/unit/generation/test_runtime_ecosystem_upgrade.py
  - Justfile
  - .github/workflows/crates.yml
  - .github/workflows/template.yml
  - boundaries/**
  - docs/architecture.md
  - docs/requirements.md
  - docs/sc-*/architecture.md
  - docs/sc-*/requirements.md
  - docs/development/ci.md
  - release/sc-publish-consumer-input.json
paths_to_delete:
  - crates/sc-runtime-db-sqlite/tests/fixtures/sqlx-resolve
---

# Sprint d-1 — dependency compatibility

## Goal and closure

Qualify the source dependency graph for 0.2.1 across the seven workspace crates,
generated applications, and separate consumer workspaces. This is an integration
sprint: it owns the shared Cargo files and cross-crate tests, rather than claiming
one boundary is independently closed. Published-artifact acceptance belongs to
[d-3](sprint-d-3-release.md). The boundary map is in the phase plan.

## Deliverables

| Deliverable | Requirement served |
| --- | --- |
| Sweep every existing repository Cargo.toml, preserving feature and dependency budgets; raise rmcp to a tested 3.9.0 floor and rusqlite to 0.41.0 minimum | REQ-RUN-0002, NFR-RUN-0007, all listed crate dependency NFRs; #132/#133 |
| Adapt affected hooks, interrupts, MCP conversions, and template call sites; retain public API/feature contracts | REQ-CRON-0001, REQ-DBSQ-0001, NFR-CMD-0001/0002, ADR-RT-0003 |
| Amend ADR-RUN-0012 and SQLite architecture for the native-link limitation; delete the obsolete sqlx-resolve fixture | REQ-DBSQ-0001, ADR-RUN-0012 |
| Required minimum/latest CI recipes for root, generated fixtures, separate consumers, Windows dependencies and optional Monty | REQ-RUN-0002, REQ-RUN-0301, NFR-RUN-0007 |
| Amend NFR-RUN-0007's old rmcp minor-pin requirement to the approved compatible-floor policy; align the sc-command bridge version sentence while retaining ADR-RUN-0017's temporary edge and upstream-proof obligation | NFR-RUN-0007, REQ-CMD-0002, ADR-RUN-0017 |
| Amend ADR-RUN-0203's frozen runtime matrix for the rmcp upgrade and repeat its runtime/schema qualification, preserving negative router/type-identity tests | ADR-RUN-0203, REQ-CMD-0005 |
| Remove unused direct tracing from sc-cron/sc-runtime with matching boundary/NFR text; set workspace/published crate and core template requirements to 0.2.1 with the explicit native-link constraint below; set sc-cron-commands readme key | REQ-RUN-0002/0005, NFR-CRON-0001, NFR-RT-0003 |

The [dependency snapshot](issue-133-dependency-sweep.md) identifies existing
manifests and observed blockers. It is evidence, not a second acceptance list.
Do not add a dependency, public API, or generated feature merely to perform this sweep.
The generated root comes from `template/workspace.template.toml`, not just the
reference `template/Cargo.toml`; update both and the adaptor manifest variants.
Update existing generation regression contracts, including
`tests/unit/generation/test_runtime_ecosystem_upgrade.py`, for the approved
0.2.1/0.41 matrix and native-link exception without weakening negative assertions.

## Dependency contract

Published crate and template requirements permit every newer Cargo
SemVer-compatible dependency release under their supported API, native-link,
and Rust-toolchain contract. Consumers can use `cargo update` without another
sc-runtime release (ANC-016). Normal requirements such as `rmcp = "3.9.0"`
already allow compatible 3.x updates; blanket support for future breaking
versions is not promised. Preserve existing dependency features.

Qualify rusqlite 0.41.0 and separately qualify the snapshot's 0.42.0 through
hook/writer/consumer tests. Every 0.41.x compatible patch remains permitted.
If both lines qualify, use `>=0.41.0, <0.43`; this widens the normal 0.41
range to include the separately tested 0.42 line. If 0.42 proves incompatible,
retain the normal `"0.41.0"` range and explain the demonstrated incompatibility.
Use one native SQLite identity per resolve. Test lockfile selections are not
production manifest pins.

Each additional upper bound tighter than the normal compatible range has a
TOML comment immediately beside the requirement naming the concrete API,
native-link, or toolchain break that requires that specific bound (ANC-019).
The actual optional Monty requirements retain their optional status:

```toml
# Monty 1.6 requires Rust 1.98; supported Monty CI uses Rust 1.96.1.
monty = { version = ">=1.4.0, <1.6", optional = true }
# monty-types 1.6 requires Rust 1.98; supported Monty CI uses Rust 1.96.1.
monty-types = { version = ">=1.4.0, <1.6", optional = true }
```

Exact/tilde restrictions need the same demonstrated incompatibility rationale.
Intentional negative compatibility fixtures retain their incompatible test
identity with a manifest comment; they do not constrain production consumers.
SQLx 0.9 cannot share its SQLite link graph with rusqlite 0.41/0.42. The limitation
applies to both the writer and sc-cron's scheduler in the same process. Preserve
supported non-conflicting configurations and make unsupported combinations
explicit; do not claim deleting sqlx-resolve proves consumer compatibility.

Generated SQLx-SQLite projects retain the compatible sc-cron/sc-cron-commands
0.2.0 floors, with the native-link reason beside those requirements, so Cargo
can retain the older link-compatible graph. Other supported generated graphs
qualify 0.2.1 floors. Optional dependencies participate in [Cargo lockfile resolution](https://doc.rust-lang.org/cargo/reference/resolver.html#features);
`cron=false` alone does not prove the conflict is absent. The legacy SQLx graph
does not receive the rusqlite advisory fix and must be documented as such.
The direct released-0.2.1 beads-cache proof in d-3 must use its actual
non-conflicting graph; it must not substitute this older template graph.

## Acceptance and validation

1. **req:REQ-RUN-0002/ranges** — Every existing Cargo.toml dependency requirement and rendered manifest input has a tested compatible range
   or a concrete in-manifest incompatibility exception. No unexplained exact
   pin, tilde restriction or extra cap remains in published/template requirements.
   All feature-gated dependency budgets and production-tool exclusions remain enforced.
2. **req:NFR-RUN-0007/endpoints** — Required CI runs minimum and latest allowed
   resolves plus affected tests for root and each changed separate workspace.
   Execute full minimal-versions first; a named transitive-floor failure may
   justify direct-minimal-versions with actual direct floors checked. Install
   prerequisites in CI; a documented command or missing prerequisite is not a pass.
   Run all affected generated fixture features, including SQLx-SQLite with cron
   disabled and the supported legacy-floor configuration, Windows jobs and Monty toolchain
   tests. Keep low/high locks isolated and restore the qualified high lockfiles.
   Source CI has no registry mode; d-3 owns its registry workflow and consumer.
3. **req:REQ-DBSQ-0001/sqlite** — Root and cron-consumer tests pass at both qualified
   endpoints; hook/interrupt behavior and one libsqlite3-sys identity hold.
   Architecture and supported-configuration guidance match the actual graph.
4. **req:REQ-RUN-0002/source-candidate** — `just lint && just test`, workspace build,
   and `sc-lint lint sc-boundary --root . --config sc-lint.toml` pass at 0.2.1.
   Production normal/build dependency graphs exclude sc-compose and ATM crates,
   including external/transitive identities (ANC-012). No stubs or bypasses remain.

These CI steps are consumed by maintainers and d-3; they gate supported dependency
updates and prevent the observed rusqlite/rmcp advisory lockout. Replace nightly
minimum-resolution machinery when stable Cargo provides the equivalent checks;
retain ordinary endpoint coverage. Use controlled inputs and causal synchronization;
no sleep, race, load or event-loop-order correctness assertions.

## This sprint does not close

Generator CLI selection, release installation and skill floors are d-4.
README content is d-2. Registry publication, released-consumer advisory clearance,
published helper proof, and final #132/#133 closure are d-3. Source tests do not
claim those published outcomes. No SQLx major upgrade or unrelated feature is added.
