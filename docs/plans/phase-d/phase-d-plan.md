---
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0702
  - REQ-RUN-0701
  - REQ-DBSQ-0001
  - REQ-DBSQ-0005
  - NFR-RUN-0007
  - NFR-RUN-0010
  - NFR-RT-0001
  - REQ-CRON-0001
  - NFR-CRON-0001
adrs:
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0013
  - ADR-RT-0005
---

# Phase d — 0.2.1 advisory fix and documentation truth

## Phase id

`d`. Integration branch: **`integrate/phase-d`**. Plan branch: `plan/phase-d`.

**Readiness:** plan only until `/plan-hardening` and plan QA-1 PASS. No
implementation sprints start until the phase plan merges to `develop`.

## Objective

Close maintenance gaps after Phase C **0.2.0**:

1. **[GitHub issue #132](https://github.com/randlee/sc-runtime/issues/132)** —
   lift `rusqlite` to **0.41** with a tested `>=` floor so dependents can link
   `libsqlite3-sys` ≥ 0.39 (RUSTSEC-2026-0459 / CVE-2026-119375). Ship **0.2.1**.
2. **[GitHub issue #131](https://github.com/randlee/sc-runtime/issues/131)** —
   already merged via PR #131 (`testing::instrument_router` behind `test-util`).
   Phase d documents the public test helper in crate/root READMEs and release
   notes for 0.2.1 consumers; no further API work unless QA finds a doc gap.
3. **README truth** — root `README.md` and every workspace crate README under
   `crates/*` describe the **six public 0.2.x crates**, template path, MSRV
   1.94.1, and accurate version/publish story (replace stale 0.1.0-only text).

## Out of scope

- sc-dolt (`plan/sc-dolt` stays on hold).
- sqlx major upgrades beyond what issue #132 requires for the coexistence
  decision.
- New product features, template layout changes, or sc-lint pin changes unless
  issue #132 forces a lockfile-only touch.

## Sprint cut

| Sprint | Issue / theme | Branch | Integration PR target |
| --- | --- | --- | --- |
| [d-1](sprint-d-1-rusqlite-0-41.md) | #132 rusqlite 0.41 + CI dual-version | `sprint/d-1-rusqlite-0-41` | `integrate/phase-d` |
| [d-2](sprint-d-2-readme-and-release.md) | READMEs + #131 doc + 0.2.1 publish | `sprint/d-2-readme-and-release` | `integrate/phase-d` |

**Order:** d-1 then d-2 (release sprint consumes bumped manifests and qualification
from d-1).

## Beads

Parent epic suggestion: **`run-phase-d`** with children mirroring d-1 and d-2.
File beads before first dev spawn if epic does not exist.

## Orchestration

- Worktrees: `/sc-git-worktree` from `integrate/phase-d` (create integration
  branch from `develop` when implementation starts).
- QA: parent spawns plan reviewers on this phase doc, then **`cursor-quality-mgr`**
  per `.cursor/skills/cursor-orchestration/SKILL.md`.
- Implementation QA-1 reviewer set: full implementation set per quality policy.
- Phase-end: optional single sprint if both merges land without separate
  phase-end gate; otherwise run phase-end QA after d-2 merges to `develop`.

## Risks

| Risk | Mitigation |
| --- | --- |
| sqlx 0.9 vs `libsqlite3-sys` 0.39+ mutual exclusion in `sqlx-resolve` fixture | d-1 owns explicit ADR/requirement amendment: relax, replace, or scope-drop the coexistence test per owner decision in sprint doc |
| `trace_v2` API change on rusqlite 0.41 | Port per beads-cache#114 pattern in d-1 |
| README drift across seven files | d-2 owns a checklist table mapping crate → publish status → doc section |

## Authority

GitHub issues **#131** (closed/MERGED) and **#132** (open, label `0.2.1`) are the
product authority for closure criteria. This phase plan does not replace them;
sprint acceptance cites both.
