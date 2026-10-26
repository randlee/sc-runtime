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
blocked_by: beads-cache sprint b-8 (sprint/b-8-tick-python, REQ-CRON-0016, ADR-CRON-0009)
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One ported crate. The expression change and the scheduler live in the same files. Cargo.toml and the existing boundary manifests are edited only after b-1 has added the writer member.
---

# Sprint b-3 — port sc-cron

## Blocker

This sprint copies `crates/sc-cron` after beads-cache sprint b-8 merges (`sprint/b-8-tick-python`, deliverable 6, REQ-CRON-0016, ADR-CRON-0009). That merge is the tree this sprint copies.

- Each schedule row chooses a Rust job or a Python module, so both can sit in the one table.
- `StopDecision::Stop` is removed. `Delete` replaces it.
- The crate's rustdoc and test fixtures carry no beads-cache names.
- The crate gains no beads-cache API, protocol, environment policy, or option.

beads-cache sprint b-9 edits comments, provenance, and tests, and does not bump the version. It does not hold this port. Maturin and PyO3 bindings stay out of this port.

`must_follow` stays `b-1`. No later sprint in this phase waits on this one.

## Goal

Port beads-cache `crates/sc-cron` into this workspace as specified in [architecture.md](../../sc-cron/architecture.md). It is an optional SQLite cron crate: one configuration table, a normal schedule string of five fields minute first, and a hook that is either a Rust function or `python3 <module> cron.tick` or `cron.stop` with one JSON object on stdin.

## Public types

Implement the types in [docs/sc-cron/architecture.md](../../sc-cron/architecture.md): `CronExpr`, `CronError`, `ScheduleSpec`, `HookTarget`, `Scheduler`, `Job`, `RunOutcome`, `StopDecision`. `HookTarget` is `Rust` or `Python { module }`.

Fallible public functions return `CronError`, the crate's one error enum. The variants are `BadExpression`, `InvalidSpec`, `AlreadyActive`, `Store`, and `Fenced`, with the codes in [ADR-RUN-0013](../../architecture.md) point 6.

## Tasks

1. REQ-RUN-0001, REQ-CRON-0001: copy the crate into `crates/sc-cron` and add the workspace member.
2. REQ-CRON-0002: a sc-cron schedule is a normal crontab string of five fields: minute, hour, day of month, month, and day of week. `*/5 * * * *` means every fifth minute. A six-field seconds-first string stays parseable only as the ADR-RUN-0013 point 2 compatibility note. A normal schedule string remains five fields, minute first.
3. REQ-CRON-0003: a `Rust` row calls `Job`. A `Python` row runs `python3 <module> cron.tick` or `python3 <module> cron.stop` and writes one JSON object to stdin. The object includes the schedule name and the row payload. It does not include ATM team or phase fields. The crate exposes no HTTP route, MCP tool, or binary.
4. REQ-CRON-0004: the crate creates one schedule table. A test writes a row without constructing `SqliteWriter`. The crate does not migrate the consumer's other tables.
5. REQ-RUN-0005: add `boundaries/sc-cron/sc-cron.toml` with `allowed_dependents = []` and forbidden edges to every other workspace crate. Add `sc-cron` to the forbidden edges of `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, and `sc-runtime-db-sqlite`.
6. NFR-RUN-0005: tests open the schedule file under a `tempfile::TempDir`. They do not set process environment variables.

## Required validation

- `cargo test -p sc-cron` exits 0.
- `cargo build --workspace` exits 0.
- `sc-lint lint sc-boundary` exits 0 with the new manifest.
- `rg sc-cron template` prints nothing.
- `cargo tree -p sc-cron -e normal --prefix none` prints no line `sqlx` and no workspace crate.
- `CronExpr::parse("*/5 * * * *")` is covered by a test that expects success.
- A test runs a Python hook with a stub interpreter and checks the argv is `python3`, the module, and `cron.tick` or `cron.stop`, and that stdin is one JSON object that includes the schedule name and the row payload and does not include ATM team or phase fields.
- A test writes a schedule row without constructing `SqliteWriter`. The crate's migrations create only the schedule table.

## Non-goals

No beads CLI client. No Postgres store. No store-adaptor generator. No HTTP, MCP, or CLI API on this crate. No edit to `sc-lint.toml` (sprint `b-2` owns that file). No edit to `crates/sc-runtime-db-sqlite/` source (sprint `b-1` owns that crate; this sprint only adds the forbidden edge in its manifest).
