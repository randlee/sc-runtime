---
id: d-1
phase: d
status: planned
closure_type: integration
target_boundary: rusqlite-consumer-safety
branch: sprint/d-1-rusqlite-0-41
pr_target: integrate/phase-d
requirements:
  - REQ-RUN-0002
  - REQ-DBSQ-0001
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
---

# Sprint d-1 — rusqlite 0.41 advisory floor

## Goal

Implement [issue #132](https://github.com/randlee/sc-runtime/issues/132): move
`rusqlite` to **0.41** base with a declared `>=` range tested at **minimum** and
**latest** versions in CI. Unblock dependents on `libsqlite3-sys` ≥ 0.39.

## Deliverables

- Update `rusqlite` dependency in:
  - `crates/sc-cron/Cargo.toml`
  - `crates/sc-runtime-db-sqlite/Cargo.toml`
  - `tests/integration/cron-consumer/Cargo.toml`
- Adapt `Connection::trace_v2` call sites to the 0.41 `Result` + callback typing.
- Record the **`sqlx-resolve` / libsqlite3-sys coexistence** decision in
  `docs/sc-runtime-db-sqlite/requirements.md` or architecture: amend test or
  documented guarantee per issue #132 "Known conflict" section.
- CI jobs or `just` recipe entries proving:
  - `cargo +nightly update -Z minimal-versions` (lowest allowed), and
  - `cargo update` (latest compatible) both pass scoped workspace tests.
- `Cargo.lock` refreshed with consistent `libsqlite3-sys`.

## Acceptance criteria

1. No published-bound crate pins block `libsqlite3-sys` < 0.39.
2. Lowest and latest allowed `rusqlite` versions pass the sprint's CI matrix.
3. Issue #132 checklist items 1–3 satisfied; item 4 (publish) deferred to d-2.
4. `just lint` and `just test` green on PR head.

## Owned paths

- `crates/sc-cron/Cargo.toml`, `crates/sc-cron/src/**` (trace_v2 only if needed)
- `crates/sc-runtime-db-sqlite/**`
- `tests/integration/cron-consumer/**`
- `crates/sc-runtime-db-sqlite/tests/fixtures/sqlx-resolve/**` (if amended)
- `docs/sc-runtime-db-sqlite/requirements.md` (coexistence wording)
- `.github/workflows/**` or `Justfile` (dual-version CI hook)

## This sprint does not close

- crates.io publish / git tag (d-2).
- Root or crate README version prose (d-2).
- Issue #131 documentation (d-2).

## Validation

- Reproduce consumer `cargo deny check advisories` clean for RUSTSEC-2026-0459
  against the workspace lock produced here (document command in PR body).
