---
id: b-3
phase: b
status: planned
closure_type: boundary
target_boundary: sc-cron
track: cron
wave: 2
branch: sprint/b-3-sc-cron
worktree: sprint/b-3-sc-cron
requirements:
  - REQ-RUN-0001
  - REQ-RUN-0005
  - NFR-RUN-0005
  - NFR-RUN-0009
  - REQ-CRON-0001
  - REQ-CRON-0002
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
adrs:
  - ADR-RUN-0013
  - ADR-RUN-0012
  - ADR-RUN-0003
  - ADR-RUN-0005
  - ADR-RUN-0006
  - ADR-RUN-0011
owned_paths:
  - crates/sc-cron/
  - boundaries/sc-cron/
  - boundaries/sc-runtime/sc-runtime.toml
  - boundaries/sc-config/sc-config.toml
  - boundaries/sc-transport/sc-transport.toml
  - boundaries/sc-command/sc-command.toml
  - boundaries/sc-runtime-db-sqlite/
  - Cargo.toml
  - Cargo.lock
  - docs/sc-cron/
must_follow: [b-1]
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One ported crate. The expression change and the scheduler live in the same files. Cargo.toml and the existing boundary manifests are edited only after b-1 has added the writer member.
---

# Sprint b-3 — port sc-cron

## Goal

Port beads-cache `crates/sc-cron` into this workspace as specified in [architecture.md](../../sc-cron/architecture.md). Five-field crontab strings parse as minute-first. The app registers the `Job`. The schedule file is not the writer database.

## Public types

Implement the types already in the beads-cache crate, with the expression rule in [docs/sc-cron/architecture.md](../../sc-cron/architecture.md): `CronExpr`, `CronError`, `ScheduleSpec`, `Schedule`, `Scheduler`, `SqliteScheduleStore`, `Job`, `RunOutcome`, `StopDecision`, `WorkerHandle`.

## Tasks

1. REQ-RUN-0001, REQ-CRON-0001: copy the crate into `crates/sc-cron` and add the workspace member.
2. REQ-CRON-0002: accept a five-field minute-first string, keep six-field seconds-first, and update the `BadExpression` recovery text.
3. REQ-CRON-0003, REQ-CRON-0004: keep `start_async`, `stop_async`, `spawn_worker`, and `SqliteScheduleStore::open` on the crate's own file. Add no HTTP route, MCP tool, or binary.
4. REQ-RUN-0005: add `boundaries/sc-cron/sc-cron.toml` with `allowed_dependents = []` and forbidden edges to every other workspace crate. Add `sc-cron` to the forbidden edges of `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, and `sc-runtime-db-sqlite`.
5. NFR-RUN-0005: tests open the schedule file under a `tempfile::TempDir`. They do not set process environment variables.

## Required validation

- `cargo test -p sc-cron` exits 0.
- `cargo build --workspace` exits 0.
- `sc-lint lint sc-boundary` exits 0 with the new manifest.
- `rg sc-cron template` prints nothing.
- `cargo tree -p sc-cron -e normal --prefix none` prints no line `sqlx` and no workspace crate.
- `CronExpr::parse("*/5 * * * *")` is covered by a test that expects success.

## Non-goals

No beads CLI client. No Postgres store. No store-adaptor generator. No edit to `sc-lint.toml` (sprint `b-2` owns that file). No edit to `crates/sc-runtime-db-sqlite/` source (sprint `b-1` owns that crate; this sprint only adds the forbidden edge in its manifest).
