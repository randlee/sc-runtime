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
4. **[Issue #133](https://github.com/randlee/sc-runtime/issues/133)** — dependency
   `>=` tested floors (not `~` / `=` pins); **rmcp ≥ 3.9** and dual lockfile CI.
   Sweep inventory: [issue-133-dependency-sweep.md](issue-133-dependency-sweep.md)
   (including **ELIMINATE** rows for unused `tracing` on `sc-cron` / `sc-runtime`;
   reqwest feature-split stays deferred).
   **Blocked upgrades** (sqlx major, rusqlite vs sqlx same-process) stay explicit in
   that doc and d-1 ruling — not every dep can move to crates.io latest in 0.2.1.

## Boundary map (phase touch set)

| Boundary / crate | Manifest | Manifest edges (this phase) | Contract change |
| --- | --- | --- | --- |
| `sc-cron` | `boundaries/sc-cron/sc-cron.toml` | `allowed_dependencies`: `rusqlite`, workspace crates as today | `rusqlite` 0.41; hook/interrupt call sites if needed |
| `sc-runtime-db-sqlite` | `boundaries/sc-runtime-db-sqlite/sc-runtime-db-sqlite.toml` | `allowed_dependencies`: `rusqlite` only; `allowed_dependents`: `[]` | `rusqlite` 0.41; **delete** sqlx-resolve fixture; ADR: no sqlx 0.9 same-process writer |
| `cron-consumer-proof` | (integration member) | Depends on `sc-cron` / path pins | Align test manifest to rusqlite 0.41 |
| Published README surfaces | — | — | Doc-only; six crates.io READMEs + root |
| Release kit | `release/sc-publish-consumer-input.json` | — | 0.2.1 publish; no new crates |

No new workspace members. d-1 **requires** ADR-RUN-0012 / crate-architecture
amendment and removal of the sqlx-resolve fixture (ruling frozen in
[sprint-d-1](sprint-d-1-rusqlite-0-41.md)).

## Tracks and wave table

One track, two **parallel_safe** wave-1 sprints; publish is the serial tail on d-2.

| Wave | Sprint | `closure_type` | `target_boundary` | Branch | `owned_paths` (summary) |
| --- | --- | --- | --- | --- | --- |
| 1 | [d-1](sprint-d-1-rusqlite-0-41.md) | `boundary` | `BOUNDARY-sc-cron` (+ vertical sqlite) | `sprint/d-1-rusqlite-0-41` | rusqlite 0.41, sqlite writer, cron-consumer-proof, ADR |
| 1 | [d-1b](sprint-d-1b-dependency-ranges.md) | `boundary` | `BOUNDARY-sc-command` (+ runtime/template MCP) | `sprint/d-1b-dependency-ranges` | rmcp ≥3.9, utoipa/schemars ranges, template daemon, `Justfile` minimal-versions |
| 1 | [d-2](sprint-d-2-readme-and-release.md) doc wave | `integration` | `runtime-ecosystem-release-0-2-1` | `sprint/d-2-readme-and-release` | README surfaces, `#131` doc |
| 2 | [d-2](sprint-d-2-readme-and-release.md) publish tail | `integration` | same | same | 0.2.1 versions, `Cargo.lock`, kit publish (#133 done-when) |

| Metric | Value | Note |
| --- | --- | --- |
| **critical_path** | 2 | d-2 publish tail waits on d-1 + d-1b lockfile (merge-forward) |
| **width** | 3 | d-1, d-1b, d-2 doc wave in parallel; coordinate `Cargo.lock` |
| **sprint count** | 3 | d-1b added for #133; publish stays in d-2 |

## Out of scope

- sc-dolt (`plan/sc-dolt` on hold).
- sqlx major bump beyond #132 coexistence decision.
- Template layout or sc-lint pin changes unless lockfile-only fallout from d-1.

## Plan anchoring

Phase d intent and locked anchors: [intent.md](intent.md) (`plan_dir`:
`docs/plans/phase-d`). Captures **registry consumer floors**, **CI hermetic
external CLIs**, and **downstream tool/version independence**; gates run after
plan hardening per `/plan-anchoring`.

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
