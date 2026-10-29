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
  - REQ-CRON-0002
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
  - NFR-CRON-0001
  - REQ-RUN-0005
  - NFR-RUN-0005
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0004
  - ADR-RUN-0007
  - ADR-RUN-0011
  - ADR-RUN-0013
  - ADR-RUN-0014
  - ADR-RUN-0401
  - ADR-RUN-0402
owned_paths:
  - crates/sc-cron/src/scheduler.rs
  - crates/sc-cron/src/sqlite.rs
  - crates/sc-cron/src/limits.rs
  - crates/sc-cron/tests/row_limit.rs
  - template/config/cron.schema.json
  - template/cargo-generate.toml
  - template/crates/daemon/src/cron.rs
  - template/crates/daemon/src/observability.rs
  - template/crates/daemon/src/main.rs
must_follow: [b-9]
parallel_safe: [b-10, b-11]
recommended_agent: rust-developer
recommended_model: terra
vertical_rationale: One optional limit parameter on the existing scheduler and store; no Monty or benchmark paths.
---

# Sprint b-12 — embedder-supplied schedule row limit

Implements [ADR-RUN-0014](../../architecture.md) and the [owner ruling](decision-sync-scheduling-at-api.md#owner-ruling-embedder-supplied-row-limit-2026-10-27).

PR target: **`integrate/phase-b-monty`**. Phase-end QA on that integration branch; **`develop`** receives the integration PR after phase-end PASS and green CI.

## Relations

`must_follow` **`b-9`** (scheduler and store on `develop`, merged before this branch). **`parallel_safe`** with **`b-10`** and **`b-11`**: `owned_paths` here do not intersect theirs. This sprint edits `Scheduler`, schedule store, and template cron config only; it does not touch `monty_host`, `monty_run`, `sc-compose`, or `scripts/benchmark/`. Cut the sprint worktree from **`integrate/phase-b-monty`** (same integration line as Monty sprints).

**Shared files (`b-10` owns the file; this sprint edits named slices only):**

- `crates/sc-cron/src/lib.rs` — re-exports for `ScheduleLimits` and store open signatures only.
- `docs/sc-cron/requirements.md` — [REQ-CRON-0001](../../sc-cron/requirements.md) manifest-dependency sentence (template may name `sc-cron` in comments/schema without a Cargo dep); row-limit sentences under REQ-CRON-0003 / REQ-CRON-0004 only.
- `docs/sc-cron/architecture.md` — row-limit, `ScheduleLimits`, and template wiring sections only (not Monty host-bridge text).
- `docs/architecture.md` — [ADR-RUN-0014](../../architecture.md) row-limit summary cross-link; [ADR-RUN-0013](../../architecture.md) decision point 1 template-dependency sentence only (`b-10` owns point 4 and Monty host-bridge text).

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
3. **ADR-RUN-0004 observability init:** `template/crates/daemon/src/observability.rs` and the `LoggerBuilder` call in `main.rs` construct `sc-observability` in the **generated daemon binary**, not in a reusable library crate, per ADR-RUN-0004 and ADR-RUN-0002 point 6 for new template Rust outside store/MCP/CLI/configuration paths.

Template `Cargo.toml` files do not depend on `sc-cron`. Under [REQ-CRON-0001](../../sc-cron/requirements.md), "name" means that Cargo or manifest dependency. Comments in `template/crates/daemon/src/cron.rs` and the description in `template/config/cron.schema.json` may name `sc-cron` and `sc_cron` as documented wiring.

sc-cron crate does **not** depend on `sc-config`.

## Docs

Slice edits only on the shared files listed under Relations (Monty and host-bridge prose stays with **`b-10`**):

- **`docs/sc-cron/requirements.md`**: one sentence under REQ-CRON-0003 / REQ-CRON-0004 for optional embedder row limit.
- **`docs/sc-cron/architecture.md`**: cross-link ADR-RUN-0014 (already summarized; align with implementation).

## Tests

- `crates/sc-cron/tests/row_limit.rs`: at limit, new name → `InvalidSpec`; replace existing name ok; delete then add ok; unlimited (`None`) unchanged behavior. Tests use `tempfile::TempDir` per [NFR-RUN-0005](../../requirements.md).

## This sprint does not close

- beads-cache REQ-BDC-0720 / CLI (beads-cache repo; bead `run-zif.2`).
- Monty, benchmarks, or `render_template`.
- A global in-flight lock or two-row mutex.

## Acceptance

1. ADR-RUN-0014 behavior is implemented and tested.
2. Template default **10**, **`0` → unlimited**, and JSON **`null` → unlimited** are documented in schema and wired in generated daemon cron setup.
3. `just lint && just test` for the touched crates.
