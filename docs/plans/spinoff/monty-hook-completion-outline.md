# Outline: Full Monty hook implementation (post–Track B)

**Audience:** Codex / plan-hardening agent  
**Status:** Planning input only  
**Bead:** (create epic when plan exists)

## One-line goal

Take `HookTarget::Monty` from “works in lab + minimal production bridge” to a **shippable, ADR-aligned, CI-stable** Monty cron path: same reliability and operability as Rust/Python hooks, with compose host calls bounded and observable.

## What already exists (do not replan)

| Layer | Source | State on `develop` / integration |
|-------|--------|-----------------------------------|
| Monty schedule target | [sprint-b-9](../phase-b/sprint-b-9-sc-cron-monty-hooks.md) | On **`develop`**: `HookTarget::Monty`, REPL per schedule, `execute_feed`, worker stop/tick gating, `monty_hook.rs` integration tests |
| Host bridge | [sprint-b-10](../phase-b/sprint-b-10-sc-cron-monty-host-bridge.md) | On **`integrate/phase-b-monty`**: `render_template` via `feed_start` loop in `monty_run::execute_feed`, temp-root compose, `monty_host_bridge` tests |
| Benchmarks | [sprint-b-11](../phase-b/sprint-b-11-benchmark-monty-reports.md) | Campaign platform + `production_bridge` case (scheduler path, not lab `feed_start`) |
| Row limit | [sprint-b-12](../phase-b/sprint-b-12-sc-cron-schedule-row-limit.md) | `ScheduleLimits` / template default |
| Docs | `docs/sc-cron/hooks-sc-compose.md`, `docs/sc-cron/architecture.md` | Python + Monty compose patterns |

**Integration branch extras (verify at plan time):** `sc_compose_run(subcommand, args)` host function, benchmark `examples.*` blocks, template observability init (unrelated to Monty logic but same PR line).

## Gaps this spinoff should close (from phase-end QA on integration)

Planner should turn each bucket into deliverables + acceptance tests.

### 1. ADR and contract alignment (merge-blocker class)

- **ADR-RUN-0013 point 4** still describes only `render_template` on some revisions; code and **REQ-CRON-0003** also allow **`sc_compose_run`**. Sprint must either:
  - **Amend ADR** (preferred if product keeps both): name both host functions, deny all other names, temp-root rules for render paths, subcommand policy; update consequences and `docs/sc-cron/architecture.md` in lockstep; or
  - **Remove** `sc_compose_run` from production and docs (only if owner reverses REQ-CRON-0003).
- **Subcommand policy (security):** phase-end **ruthless-boundary-qa** flagged hook-chosen argv to `bead`/`init`/`extract` without temp-root isolation. Plan must specify allowed subcommands (likely **`render` only** through same staging as `render_template`) and `COMPOSE_SUBCOMMAND` behavior.
- **Mechanical gate:** unit test that production `snippet_inputs` registers exactly `__hook`, `render_template`, and (if ADR allows) `sc_compose_run`—no drift encoded in tests (see triage `RBQA-F003` on integration branch).

**Authority files:** `docs/architecture.md` (ADR-RUN-0013), `docs/sc-cron/requirements.md` (REQ-CRON-0003), `docs/plans/phase-b/sprint-b-10-sc-cron-monty-host-bridge.md` non-goals vs current product.

### 2. Operational error surfaces (rust-best-practices phase-end)

Monty compose/host paths use stable `CODE: …; recovery` strings in `monty_run`; **scheduler/worker** paths still record plain English `RunOutcome::Failed` for:

- tick returned stop action (`run_due.rs`)
- internal resolver mismatch
- Monty join panic/cancel (`run_due`, `worker_runs.rs`)

Plan should unify **RBP-001** style detail strings across all Monty outcomes that persist to `last_outcome`.

### 3. Test hardening (flaky-test-qa)

`crates/sc-cron/tests/monty_hook.rs` uses **fixed `tokio::time::sleep`** (300ms/500ms/100ms) under the **`sc-cron-monty`** CI feature gate. Replace with:

- `RecordingStore` / watch-channel waits
- `common::WAIT` + `timeout` on conditions tied to feed completion, disable, delete, shutdown
- Avoid coupling correctness to wall-clock upper bounds in unit tests (`monty_compose.rs` deadline test slack)

Acceptance: no fixed sleeps in `monty_hook.rs` (or justify each with injected clock); Monty job green under load.

### 4. Compose stall / timeout (deferred from b-10)

**NFR-RUN-0004 / b-10 non-goals:** production waits on `Child::wait` without killing compose at row budget; **FTQ-003** asked for production-side kill of stalled `sc-compose`. Plan whether this spinoff:

- documents accepted risk only, or
- adds process-group kill after host deadline (without `block_on` / `supervise` on blocking pool—see b-10 architecture note), with tests on Unix.

### 5. Product and template surfaces

- **Template:** example Monty `cron_tick` / `cron_stop` source in generated projects (schema + `default.json` or docs-only first).
- **Operator docs:** when to use Monty vs Python vs Rust; feature `monty` in generated daemon `Cargo.toml` behind flag.
- **beads-cache:** explicit **non-goal**—Monty stays sc-runtime-only ([decision-sync-scheduling-at-api.md](../phase-b/decision-sync-scheduling-at-api.md)).

### 6. CI and release discipline

- Keep **`required-features = ["monty"]`** on `monty_hook`, `monty_host_bridge`, `sc_compose_bench` (`#[ignore]` bench).
- Rust **1.96.1** for Monty tests; default workspace **1.94.1** without feature.
- No new workspace deps; boundaries unchanged unless ADR amends host contract only.

## Suggested sprint split (planner may merge/split)

| Candidate id | Focus | Depends on |
|--------------|-------|------------|
| **b-13** (name TBD) | ADR + `sc_compose_run` policy + host-fn lint test + doc sync | PR #84 / Track B merged |
| **b-14** (name TBD) | RBP error strings on scheduler/worker Monty paths | b-13 or parallel if touch disjoint files |
| **b-15** (name TBD) | `monty_hook` flake fixes + optional compose-kill | b-13 |
| **b-16** (name TBD) | Template Monty cron examples + generation flag | template + sc-cron schema |

Planner should assign **REQ-CRON-0003**, **NFR-RUN-0004/0008/0009**, **ADR-RUN-0013**, **REQ-RUN-0005**, **NFR-CRON-0001** and owned_paths under `crates/sc-cron/`, `docs/sc-cron/`, `docs/architecture.md`, `template/`.

## Acceptance themes (for hardened plan)

- `cargo test -p sc-cron --features monty` green on 1.96.1; default `cargo test -p sc-cron` unchanged.
- `arch-qa` + `ruthless-boundary-qa` PASS on ADR/code alignment for host functions.
- `flaky-test-qa` no **important** findings on `monty_hook.rs`.
- `rg` sleep gate in `crates/sc-cron/src` still passes; test sleeps addressed separately in `tests/`.
- Boundary manifest unchanged except if planner adds a **lint/test** gate file (prefer in-crate test over new tool).

## References to read before planning

- `crates/sc-cron/src/monty_run.rs`, `monty_host.rs`, `monty_compose.rs`, `scheduler/run_due.rs`, `worker/worker_runs.rs`
- `crates/sc-cron/tests/monty_hook.rs`, `monty_host_bridge.rs`, `sc_compose_bench.rs`
- Phase-end transcripts: arch-qa **ARCH-001/002**, ruthless-boundary **RBQA-F001–F003**, rust-best-practices **RBP-F001–F004**, flaky-test **FTQ-001–F005** (integration branch @ ~`a0cc583` / `28bf417`)
- [phase-b-integration-close.md](../phase-b/phase-b-integration-close.md) Track B serial pipeline

## Non-goals (unless owner promotes)

- `monty-pool`, sandbox, HTTP/MCP for cron
- beads-cache Monty hooks
- Benchmark platform changes (owned by b-11 maintenance only)
