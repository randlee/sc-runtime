---
id: d-1
phase: d
status: planned
closure_type: boundary
target_boundary: sc-cron-sc-runtime-db-sqlite-rusqlite
track: supply-chain
wave: 1
branch: sprint/d-1-rusqlite-0-41
worktree: ../sc-runtime-worktrees/sprint/d-1-rusqlite-0-41
pr_target: integrate/phase-d
vertical_rationale: Issue #132 requires one coherent lockfile and advisory fix across sc-cron, the sqlite writer helper, and the cron-consumer integration harness before any 0.2.1 publish.
recommended_agent: rust-developer
recommended_model: grok-4.7-high-fast
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0005
  - REQ-DBSQ-0001
  - REQ-DBSQ-0002
  - REQ-DBSQ-0005
  - REQ-CRON-0001
  - NFR-RUN-0007
  - NFR-RUN-0010
  - NFR-CRON-0001
adrs:
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0013
github_issues:
  - 132
parallel_safe: []
must_follow: []
---

# Sprint d-1 — rusqlite 0.41 advisory floor

## Goal

Implement [issue #132](https://github.com/randlee/sc-runtime/issues/132) through
boundary-rooted dependency and code changes. Publish and README work stay in d-2.

## Deliverables

| Deliverable | Serves |
| --- | --- |
| `rusqlite` 0.41 base + `>=` floor in `sc-cron`, `sc-runtime-db-sqlite`, `cron-consumer` | REQ-CRON-0001, REQ-DBSQ-0001, REQ-DBSQ-0005 |
| `trace_v2` API port (0.41) in owned `sc-cron` sources | REQ-CRON-0001 |
| Documented sqlx / `libsqlite3-sys` coexistence ruling | REQ-DBSQ-0002 |
| CI proof: minimal-versions and latest-compatible `rusqlite` | NFR-RUN-0007, NFR-RUN-0010 |
| `Cargo.lock` consistent with ruling | REQ-RUN-0002 |

## Acceptance criteria

1. **boundary:sc-cron** — `cargo test -p sc-cron` passes at lowest and highest allowed `rusqlite` in the sprint matrix.
2. **boundary:sc-runtime-db-sqlite** — writer tests pass; sqlx-resolve policy matches documented REQ-DBSQ choice.
3. **req:132-advisory** — workspace lock does not force `libsqlite3-sys` < 0.39; `cargo deny` procedure documented in PR.
4. **boundary:workspace** — `just lint` and `just test` green on PR head (scoped jobs if matrix is split).

## Owned paths

- `crates/sc-cron/Cargo.toml`
- `crates/sc-cron/src/**` (only if `trace_v2` port required)
- `crates/sc-runtime-db-sqlite/**`
- `tests/integration/cron-consumer/**`
- `crates/sc-runtime-db-sqlite/tests/fixtures/sqlx-resolve/**` (only if ruling amends test)
- `docs/sc-runtime-db-sqlite/requirements.md`
- `docs/sc-runtime-db-sqlite/architecture.md` (only if ADR/RULING text required)
- `.github/workflows/**` and/or `Justfile` (dual-version hooks)
- `Cargo.lock`

## This sprint does not close

- crates.io publish / git tag (**d-2**).
- Root or crate README updates (**d-2**).
- Issue #131 consumer docs (**d-2**).

## Validation

- `just lint` and `just test` on integration branch head.
- Documented `cargo deny check advisories` smoke for RUSTSEC-2026-0459 (command in PR body).
