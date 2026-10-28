---
id: b-12
phase: b
status: complete
closure_type: boundary
target_boundary: sc-cron
track: cron
wave: 6
branch: sprint/b-12-sc-cron-schedule-row-limit
worktree: sprint/b-12-sc-cron-schedule-row-limit
integration_branch: integrate/phase-b-monty
requirements:
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
adrs:
  - ADR-RUN-0013
  - ADR-RUN-0014
  - ADR-RUN-0007
  - ADR-RUN-0011
owned_paths:
  - crates/sc-cron/src/scheduler.rs
  - crates/sc-cron/src/sqlite.rs
  - crates/sc-cron/src/limits.rs
  - crates/sc-cron/tests/row_limit.rs
  - docs/sc-cron/requirements.md
  - docs/sc-cron/architecture.md
  - template/config/cron.schema.json
  - template/cargo-generate.toml
must_follow: [b-9]
parallel_safe: [b-10, b-11]
recommended_agent: rust-developer
recommended_model: terra
vertical_rationale: One optional limit parameter on the existing scheduler and store; no Monty or benchmark paths.
---

# Sprint b-12 — embedder-supplied schedule row limit

Plan only until the sprint branch opens. Implements [ADR-RUN-0014](../../architecture.md) and the [owner ruling](decision-sync-scheduling-at-api.md#owner-ruling-embedder-supplied-row-limit-2026-10-27).

PR target: **`integrate/phase-b-monty`**. Phase-end QA on that integration branch; **`develop`** receives the integration PR after phase-end PASS and green CI.

## Relations

`must_follow` **`b-9`** (scheduler and store on `develop`, merged before this branch). **`parallel_safe`** with **`b-10`** and **`b-11`**: this sprint edits `Scheduler` / schedule store only; it does not touch `monty_host`, `sc-compose`, or `scripts/benchmark/`. Cut the sprint worktree from **`integrate/phase-b-monty`** (same integration line as Monty sprints).

## Goal

The embedder can pass an optional maximum schedule row count when opening the scheduler. Generated applications default to **10** rows via sc-config; operators can raise or remove the cap without recompiling. sc-cron never reads sc-config.

## API (names fixed in this sprint)

1. **`ScheduleLimits`** (or equivalent options struct) with **`max_schedule_rows: Option<NonZeroUsize>`**. **`None`** = unlimited (library default).
2. **`Scheduler::open`** and the store constructors accept `ScheduleLimits`. **`Scheduler::new`** reads that ceiling from the store (`ScheduleStore::limits`) and does not keep a second copy. Existing call sites open the store with **`ScheduleLimits::default()`** (unlimited) unless the embedder sets a limit.
3. **Count:** enabled **and** disabled rows in the schedule table. **`replace`** on an existing name does not change the count. **Delete** frees a slot.
4. **Enforcement:** in the same store transaction as an insert of a **new** name. Over limit → **`CronError::InvalidSpec`** with detail naming the limit and current count. No new `CronError` variant.
5. **Lowering the limit** below the current row count does not delete rows; only **new** names are rejected until count is under the limit.

**v1:** limit applied at scheduler open (daemon startup). **v2 (optional, same sprint only if trivial):** method to update the limit when the embedder reloads config.

## Generated template

1. **`template/config/cron.schema.json`**: key **`cron.max_schedule_rows`**, integer or JSON null, default **`10`**. **`0`** or **`null`** means unlimited. A missing key stays **`10`**. The schema allows the other keys in `config/default.json`. cargo-generate `exclude` copies a file without Liquid rendering and does not omit it. This schema is on `[template] ignore`, so a generated project does not contain it. Generated projects carry the key in `config/default.json`.
2. **Generated daemon wiring:** wherever the layout opens `Scheduler` (new module or existing startup path when sc-cron is enabled for the layout), read config → map **`0`** or JSON **`null`** to `None`, a missing key to **`10`**, else `Some(NonZeroUsize::new(n))` → pass into `ScheduleLimits`. If no layout enables sc-cron yet, add the schema key and document the wiring contract in `docs/sc-cron/` for the next layout that adds cron.

sc-cron crate does **not** depend on `sc-config`.

## Docs

- **`docs/sc-cron/requirements.md`**: one sentence under REQ-CRON-0003 / REQ-CRON-0004 for optional embedder row limit.
- **`docs/sc-cron/architecture.md`**: cross-link ADR-RUN-0014 (already summarized; align with implementation).

## Tests

- `crates/sc-cron/tests/row_limit.rs`: at limit, new name → `InvalidSpec`; replace existing name ok; delete then add ok; unlimited (`None`) unchanged behavior.

## This sprint does not close

- beads-cache REQ-BDC-0720 / CLI (beads-cache repo; bead `run-zif.2`).
- Monty, benchmarks, or `render_template`.
- A global in-flight lock or two-row mutex.

## Acceptance

1. ADR-RUN-0014 behavior is implemented and tested.
2. Template default **10**, **`0` → unlimited**, and JSON **`null` → unlimited** are documented in schema and wired in generated daemon cron setup.
3. `just lint && just test` for the touched crates.
