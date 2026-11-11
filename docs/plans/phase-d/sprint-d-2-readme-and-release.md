---
id: d-2
phase: d
status: planned
closure_type: release
target_boundary: ecosystem-documentation
branch: sprint/d-2-readme-and-release
pr_target: integrate/phase-d
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0702
  - REQ-RUN-0701
  - NFR-RT-0001
adrs:
  - ADR-RT-0005
github_issues:
  - 131
  - 132
must_follow: [d-1]
---

# Sprint d-2 — README truth and 0.2.1 release

## Goal

1. Close documentation for **[issue #131](https://github.com/randlee/sc-runtime/issues/131)**
   (`sc_runtime::testing::instrument_router`, `test-util` feature) in user-facing
   READMEs.
2. Close **[issue #132](https://github.com/randlee/sc-runtime/issues/132)** via
   authorized **0.2.1** publication after d-1 lands on `integrate/phase-d`.
3. Align **root and crate READMEs** with the shipped 0.2.x ecosystem.

## Deliverables

Registry audit input: [registry-doc-audit.md](registry-doc-audit.md) (crates.io +
PyPI scope, framework blurb template, `sc-cron-commands` readme key).

### README updates (all required)

| Path | Must reflect |
| --- | --- |
| `README.md` | Six public crates at 0.2.x, `sc-cron` + `sc-cron-commands`, template wizard, MSRV 1.94.1, link to 0.2.0/0.2.1 release notes |
| `crates/sc-config/README.md` | Role, features, publish name |
| `crates/sc-transport/README.md` | Client/server feature split |
| `crates/sc-command/README.md` | Envelope, `server` feature |
| `crates/sc-runtime/README.md` | Daemon builder, **`test-util` / `instrument_router`** pointer to `docs/plans/maintenance/router-test-helper.md` |
| `crates/sc-runtime-db-sqlite/README.md` | Writer actor, rusqlite floor from d-1 |
| `crates/sc-cron/README.md` | rusqlite floor, observability-types edge |
| `crates/sc-cron-commands/README.md` | Optional adapter crate, 0.2.0 status |

Remove stale "0.1.0 only" or "four crates" claims unless marked historical in a
**Prior releases** subsection.

Every **published** crate README must include a **Part of sc-runtime** section
(scaffolding framework blurb per audit). Add `readme = "README.md"` to
`crates/sc-cron-commands/Cargo.toml`. Refresh `sc-runtime-db-sqlite/README.md`
as workspace-only (no false crates.io publish claim).

### Release

- `docs/releases/v0.2.1.md` (or project release doc pattern used for 0.2.0).
- sc-publish kit run per `docs/development/sc-publish.md` after QA PASS + green CI.
- Close GitHub issues #131 and #132 with release reference.

## Acceptance criteria

1. Every row in the README table updated; `git diff` shows no contradictory
   version text between root and crate READMEs.
2. `sc-runtime` README documents `instrument_router` behind `test-util`.
3. Published **0.2.1** includes d-1 rusqlite changes; consumer advisory check
   passes per issue #132 "Done when".
4. QA-1 PASS, `reviewer_spawn_gate: pass`, CI green on release PR.

## Owned paths

- `README.md`
- `crates/*/README.md` (seven workspace crates listed above)
- `crates/sc-cron-commands/Cargo.toml` (`readme` key)
- `docs/plans/phase-d/registry-doc-audit.md` (close audit when d-2 lands)
- `docs/releases/v0.2.1.md`
- `docs/project-plan.md` (phase d sprint status)
- Publish manifests / kit inputs only as required by sc-publish (no hand-written
  registry bypass)

## This sprint does not close

- rusqlite code changes (d-1).
- sc-dolt planning.

## Validation

- Spot-check generated project docs still link correctly from root README.
- Post-release: `cargo deny` smoke from `tests/integration/cron-consumer` or
  documented consumer fixture.
