---
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0005
  - REQ-RUN-0702
  - REQ-DBSQ-0001
  - REQ-CRON-0001
  - NFR-CRON-0001
  - NFR-RT-0001
adrs:
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0013
  - ADR-RUN-0002
---

# Phase d — 0.2.1 advisory fix and documentation truth

## Phase id

`d`. Integration branch: **`integrate/phase-d`**. Plan branch: `plan/phase-d`.

**Readiness:** **hardening in flight.** `/plan-hardening` steps 1–6 and plan QA-1
must PASS before implementation. No sprint branches until this plan merges to
`develop`.

**Beads:** epic `run-ajx` (Phase D).

## Objective

Close maintenance gaps after Phase C **0.2.0**:

1. **[Issue #132](https://github.com/randlee/sc-runtime/issues/132)** — `rusqlite`
   0.41+ with tested minimum/latest range; RUSTSEC-2026-0459 clearance.
2. **[Issue #131](https://github.com/randlee/sc-runtime/issues/131)** — code merged;
   document `test-util` / `instrument_router` for consumers.
3. **Registry documentation** — root and crate READMEs match 0.2.x reality and
   [registry-doc-audit.md](registry-doc-audit.md) (crates.io framework blurb; no PyPI).

## Boundary map (phase touch set)

| Boundary / crate | Manifest | Contract change this phase |
| --- | --- | --- |
| `sc-cron` | `boundaries/sc-cron/sc-cron.toml` | `rusqlite` version range; `trace_v2` call sites |
| `sc-runtime-db-sqlite` | `BOUNDARY-sc-runtime-db-sqlite` | `rusqlite` 0.41; **delete** sqlx-resolve fixture; ADR documents no sqlx 0.9 same-process writer |
| `cron-consumer` integration | test crate only | Align test dependency with published `sc-cron` |
| Published README surfaces | none | Doc-only; six crates.io READMEs + root |
| Release kit | `release/sc-publish-consumer-input.json` | 0.2.1 publish; no new crates |

No new workspace members. d-1 **requires** ADR-RUN-0012 / crate-architecture
amendment and removal of the sqlx-resolve fixture (ruling frozen in
[sprint-d-1](sprint-d-1-rusqlite-0-41.md)).

## Tracks and wave table

Single maintenance track — dependency/doc closure must ship as one **0.2.1**
release artifact.

| Wave | Sprint | `closure_type` | `target_boundary` | Branch |
| --- | --- | --- | --- | --- |
| 1 | [d-1](sprint-d-1-rusqlite-0-41.md) | `boundary` | `BOUNDARY-sc-cron` (+ vertical `BOUNDARY-sc-runtime-db-sqlite`) | `sprint/d-1-rusqlite-0-41` |
| 2 | [d-2](sprint-d-2-readme-and-release.md) | `integration` | `runtime-ecosystem-release-0-2-1` | `sprint/d-2-readme-and-release` |

| Metric | Value | Note |
| --- | --- | --- |
| **critical_path** | 2 | d-2 `must_follow` d-1 (lockfile + advisory fix before publish) |
| **width** | 1 | Serial track; no parallel layer sprints |
| **sprint count** | 2 | README work folded into integration sprint, not a thin third sprint |

## Out of scope

- sc-dolt (`plan/sc-dolt` on hold).
- sqlx major bump beyond #132 coexistence decision.
- Template layout or sc-lint pin changes unless lockfile-only fallout from d-1.

## Orchestration

- Plan hardening: [plan-hardening-rounds.md](plan-hardening-rounds.md).
- Implementation: `.cursor/skills/cursor-orchestration/SKILL.md`; enforcer
  **`cursor-quality-mgr`** only.
- Beads epic **`run-ajx`** (children filed at dev dispatch).

## Risks

| Risk | Owner sprint |
| --- | --- |
| sqlx 0.9 vs rusqlite 0.41 same-process | **Ruled in d-1** — not supported on 0.2.1; sqlx bump out of phase |
| Dual-version CI cost | d-1 |
| README / crates.io drift | d-2 + [registry-doc-audit.md](registry-doc-audit.md) |
