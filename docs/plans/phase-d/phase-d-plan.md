---
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0005
  - REQ-RUN-0702
  - REQ-DBSQ-0001
  - REQ-CRON-0001
  - NFR-RUN-0007
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0013
  - ADR-RUN-0017
---

# Phase d — easy adoption and compatible dependency upgrades

**Intent:** [intent.md](intent.md). **Target:** all six public crates at 0.2.1.
**Plan branch:** `plan/phase-d` → `develop`. **Implementation branch:**
`integrate/phase-d`. **Beads:** `run-ajx`.

## Consumer outcome and scope

Unblock beads-cache on the published SQLite security fix (#132), clear the
rmcp advisory and remove avoidable dependency restrictions (#133), and document
the restored test helper from merged PR131. Consumers can build the supported
configuration and select compatible dependency fixes without another framework
release. README and template guidance describe the actual published product.

Use tested floors with normal Cargo-compatible ranges. Extra restrictions need
an observed API/native-link/toolchain incompatibility and an owning manifest
comment. Separately validate any broader compatibility envelope. A normal
caret is not an exact pin; arbitrary future breaking versions are not promised.
This is the approved clarification of #133's earlier blanket-`>=` wording.

## Boundary map

Read from `boundaries/<crate>/<crate>.toml`. Arrows below are the manifests'
allowed dependency/dependent sets, including external budgets; existing
feature-specific companion checks remain authoritative. No new boundary edge,
first-party trait, schema, wire field, or error contract is introduced.

| Crate / boundary manifest | Allowed dependencies | Allowed dependents | Phase change |
| --- | --- | --- | --- |
| sc-config / boundaries/sc-config/sc-config.toml | serde, serde_json | none | compatible floors only |
| sc-transport / boundaries/sc-transport/sc-transport.toml | tokio, reqwest, serde, serde_json | sc-runtime | compatible floors; preserve server feature gating |
| sc-command / boundaries/sc-command/sc-command.toml | serde, serde_json, sc-observability-types | sc-cron-commands, sc-runtime, sc-cron | rmcp/schema/floors; preserve optional server/schema contracts and ADR-RUN-0017 exception |
| sc-runtime / boundaries/sc-runtime/sc-runtime.toml | sc-observability-types, sc-transport, sc-command, axum, tokio, fd-lock, utoipa, utoipa-axum, serde, rmcp, tracing | none | floors and rmcp; remove unused direct tracing with matching NFR/boundary text |
| sc-cron / boundaries/sc-cron/sc-cron.toml | sc-command, sc-observability-types, async-trait, chrono, cron, getrandom, process-wrap, rusqlite, serde_json, serde, tempfile, tokio, tracing | sc-cron-commands | rusqlite/hooks/floors; optional Monty toolchain cap; remove unused direct tracing |
| sc-cron-commands / boundaries/sc-cron-commands/sc-cron-commands.toml | sc-command, sc-cron, serde, serde_json, schemars, utoipa | none | floors; preserve schema/scheduler feature graph |
| sc-runtime-db-sqlite / boundaries/sc-runtime-db-sqlite/sc-runtime-db-sqlite.toml | rusqlite | none | rusqlite/writer compatibility; ADR amendment; delete sqlx-resolve |

The generated template and separate consumer fixtures are outside these crate
boundaries and get their own dependency resolves. They preserve the same
compatible-update policy. The snapshot inventory is
[issue-133-dependency-sweep.md](issue-133-dependency-sweep.md).

## Sprint cut and wave table

No contract sprint is needed: there is no new first-party interface to split
between implementers/consumers. The maintenance defect must update shared
manifests, native-link identities, and CI together, so d-1 is one integration
sprint with an explicit vertical rationale. README content is substantial,
independent documentation work in d-2. d-3 consumes the qualified packages and
their README bodies to close the released-consumer capability.

The earlier d-1/d-1b split overlapped Cargo.toml/Cargo.lock and treated sections
of a file as independent paths. It is replaced by one manifest owner. The old
hybrid d-2 doc/publish wave is split into a docs sprint and a release sprint.
Only unmerged planning identifiers are renamed; merged history is unchanged.

| Wave | Track | Sprint / closure | Target | Branch and worktree | Owned paths summary |
| --- | --- | --- | --- | --- | --- |
| 1 | dependency-compatibility | [d-1](sprint-d-1-dependency-compatibility.md), integration | runtime-ecosystem-dependency-compatibility | sprint/d-1-dependency-compatibility | all existing manifests/locks, affected code/tests, CI/tooling, boundaries and architecture/NFR text |
| 1 | consumer-docs | [d-2](sprint-d-2-consumer-docs.md), docs | runtime-ecosystem-consumer-docs | sprint/d-2-consumer-docs | root and seven crate READMEs only |
| 2 | consumer-release | [d-3](sprint-d-3-release.md), integration | runtime-ecosystem-release-0-2-1 | sprint/d-3-release | registry consumer proof, release notes, project status and required release benchmark reports |

**Critical path:** 2 sprints. **Usable width:** 2. **Sprint count:** 3.
No sprint was added: the same three units now have disjoint file ownership,
correct numeric IDs, and one closure type each. Authoritative `owned_paths`
are frontmatter in each sprint doc; within a wave, no file matches two owners.
Single-layer development tracks can run as independent stacks; shared source
artifacts are integrated before the release stack. No shared-file serial edge
is manufactured. d-3's dependencies name actual API/manifest and README artifacts.

## Acceptance ownership

- d-1 closes source dependency/API/tooling qualification and required endpoint CI.
- d-2 closes README accuracy and consumer instructions.
- d-3 alone closes published security clearance, compatible registry updates,
  packaged README verification, published helper proof, and #132/#133.

Versions, readme manifest keys and publish-input source changes belong to d-1;
d-3 publishes the candidate without editing those files. A failed release
validation returns a defect to its source owner and invalidates affected checks.

## Limits and readiness

SQLx 0.9's SQLite link window cannot share a process with the upgraded rusqlite
writer or sc-cron scheduler. The architecture and consumer docs must identify
supported configurations and the unsupported combined graph. The beads-cache
registry proof must use its actual affected configuration; deleting a fixture
or auditing a source-path consumer is insufficient. SQLx major upgrades,
sc-dolt, and unrelated features are outside this maintenance phase.

The plan remains **planned**. Final human approval, normal plan hardening/QA,
and merge to develop precede implementation branch dispatch. Jev anchor
validation does not itself certify release readiness or publication.
The current alignment work produces the requested HTML diff for approval.
