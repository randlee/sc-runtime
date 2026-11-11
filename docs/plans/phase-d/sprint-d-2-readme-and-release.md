---
id: d-2
phase: d
status: planned
closure_type: integration
target_boundary: runtime-ecosystem-release-0-2-1
track: supply-chain
wave: 2
branch: sprint/d-2-readme-and-release
worktree: ../sc-runtime-worktrees/sprint/d-2-readme-and-release
pr_target: integrate/phase-d
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0702
  - REQ-RUN-0701
  - REQ-RUN-0005
  - NFR-RT-0001
adrs:
  - ADR-RT-0005
  - ADR-RUN-0012
github_issues:
  - 131
  - 132
must_follow:
  - d-1
recommended_agent: rust-developer
recommended_model: grok-4.7-high-fast
---

# Sprint d-2 — README truth and 0.2.1 release

## Goal

Integration closure for issues **#131** and **#132**: consumer-facing docs,
registry truth, and authorized **0.2.1** publication after d-1 merges to
`integrate/phase-d`.

Authority: [registry-doc-audit.md](registry-doc-audit.md).

## Deliverables

| Deliverable | Serves |
| --- | --- |
| Root `README.md` — six crates, 0.2.x, wizard, MSRV 1.94.1 | REQ-RUN-0701 |
| Each published crate README — **Part of sc-runtime** scaffolding blurb | REQ-RUN-0702 |
| `sc-cron-commands/Cargo.toml` — `readme = "README.md"` | REQ-RUN-0002 |
| `sc-runtime` README — `test-util` / `instrument_router` (#131) | NFR-RT-0001 |
| `docs/releases/v0.2.1.md` | REQ-RUN-0702 |
| sc-publish kit run → crates.io 0.2.1 | REQ-RUN-0702, #132 done-when |
| Close GitHub #131, #132 | — |

### README inventory

| Path |
| --- |
| `README.md` |
| `crates/sc-config/README.md` |
| `crates/sc-transport/README.md` |
| `crates/sc-command/README.md` |
| `crates/sc-runtime/README.md` |
| `crates/sc-runtime-db-sqlite/README.md` (workspace-only; no publish claim) |
| `crates/sc-cron/README.md` |
| `crates/sc-cron-commands/README.md` |

## Acceptance criteria

1. **req:131** — `sc-runtime` README documents `instrument_router` behind `test-util`.
2. **req:132** — Published 0.2.1 clears RUSTSEC-2026-0459 for a documented consumer path.
3. **req:RUN-0702** — All six crates.io READMEs include the standard scaffolding section from the audit.
4. **integration:publish** — Kit publish completes; `release/sc-publish-consumer-input.json` unchanged in intent except versions produced by kit.
5. **integration:qa** — Sprint QA-1 PASS, `reviewer_spawn_gate: pass`, required CI green.

## Owned paths

- `README.md`
- `crates/sc-config/README.md`
- `crates/sc-transport/README.md`
- `crates/sc-command/README.md`
- `crates/sc-runtime/README.md`
- `crates/sc-runtime-db-sqlite/README.md`
- `crates/sc-cron/README.md`
- `crates/sc-cron-commands/README.md`
- `crates/sc-cron-commands/Cargo.toml`
- `docs/releases/v0.2.1.md`
- `docs/project-plan.md` (phase d status only)
- `docs/plans/phase-d/registry-doc-audit.md` (mark closed)
- Release kit outputs per `docs/development/sc-publish.md` (no manual registry bypass)

## This sprint does not close

- `rusqlite` code changes (**d-1**).
- sc-dolt planning.

## Validation

- Post-publish spot-check each `https://crates.io/crates/<name>` README render.
- `cargo deny` smoke documented in release notes.
