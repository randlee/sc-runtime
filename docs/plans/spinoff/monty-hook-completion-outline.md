# Outline: Full Monty hook implementation (post–Track B)

**Audience:** Codex / plan-hardening agent  
**Status:** Planning input only  
**Bead:** Provenance only; do not create an epic or sprint from this outline.

> Historical discovery snapshot: Track B Monty is now on `develop` at merge
> `240e445`. For binding host behavior use the landed b-10/b-11 documents,
> ADR-RUN-0013 section 4, and REQ-CRON-0003; this outline is not authority.
> The hardened disposition is already represented by b-13 through b-18.
> Historical candidate ids below must not be dispatched as additional work.

## One-line goal

Take `HookTarget::Monty` from “works in lab + minimal production bridge” to a **shippable, ADR-aligned, CI-stable** Monty cron path: same reliability and operability as Rust/Python hooks, with compose host calls bounded and observable.

## What already exists (do not replan)

| Layer | Source | State on `develop` / integration |
|-------|--------|-----------------------------------|
| Monty schedule target | [sprint-b-9](../phase-b/sprint-b-9-sc-cron-monty-hooks.md) | On **`develop`**: `HookTarget::Monty`, REPL per schedule, `execute_feed`, worker stop/tick gating, `monty_hook.rs` integration tests |
| Host bridge | [sprint-b-10](../phase-b/sprint-b-10-sc-cron-monty-host-bridge.md) | Landed implementation: hard-coded `render_template` and render-only `sc_compose_run`; b-16 must generalize this pre-release interface for application-owned host functions |
| Benchmarks | [sprint-b-11](../phase-b/sprint-b-11-benchmark-monty-reports.md) | Campaign platform + `production_bridge` case (scheduler path, not lab `feed_start`) |
| Row limit | [sprint-b-12](../phase-b/sprint-b-12-sc-cron-schedule-row-limit.md) | `ScheduleLimits` / template default |
| Docs | `docs/sc-cron/hooks-sc-compose.md`, `docs/sc-cron/architecture.md` | Python + Monty compose patterns |

**Reconciled integration facts:** b-10 proves host suspension and resume, but
its fixed two-function registry is not the final general-library contract.
`sc_compose_run(subcommand, args)` and `atm_run(subcommand, args)` are supplied
as opt-in adapters through the shared extension interface;
`render_template` is benchmark convenience, not a product requirement.

## Historical QA findings and reconciled disposition

These findings explain why the hardened plan has its present acceptance tests.
They are not a queue from which to create more sprints.

### 1. ADR and contract alignment — live, owned by b-16

The landed b-10 code and documents agree with each other, but the hard-coded
allowlist does not satisfy the owner requirement that `sc-cron` be reusable by
applications with their own Monty host calls. Resolve this inside existing
sprint b-16 rather than adding another sprint:

- Amend **ADR-RUN-0013 point 4**, **REQ-CRON-0003**, and
  `docs/sc-cron/architecture.md` in lockstep around an embedder-owned registry
  of validated Monty host names and synchronous handlers.
- Default to no registered application calls. Deny names that the embedding
  application did not register; hook source can invoke but cannot register or
  replace handlers.
- Supply `sc_compose_run` and `atm_run` as opt-in CLI adapters supporting any
  command/subcommand. They share argv forwarding and process supervision;
  no forced render-only or temp-root policy narrows the general interface.
  Neither tool's Rust crates may be linked/imported by any production release
  crate; boundary lint enforces this while allowing tests and benchmarks.
- MVP source is application-supplied. Script signing and a separate approval
  mechanism have no consumer-facing MVP configuration option.
- Remove `render_template` as a mandatory product host and migrate the
  production-bridge benchmark/example to `sc_compose_run("render", ...)`.
  Internal helpers and lab benchmark references may retain the name.
- Replace the fixed-name registration assertion with tests for empty-default,
  custom registration/call/result/error, invalid/duplicate/reserved names,
  unregistered denial, and the optional compose adapter.

This does not create a new b-13-style host sprint. b-16 already owns the Monty
product surface and is widened to close the general-library interface before
the environment work in b-17 consumes it.

**Authority files:** `docs/architecture.md` (ADR-RUN-0013), `docs/sc-cron/requirements.md` (REQ-CRON-0003), `docs/plans/phase-b/sprint-b-10-sc-cron-monty-host-bridge.md` non-goals vs current product.

### 2. Operational error surfaces — owned by b-15

Monty compose/host paths use stable `CODE: …; recovery` strings in `monty_run`; **scheduler/worker** paths still record plain English `RunOutcome::Failed` for:

- tick returned stop action (`run_due.rs`)
- internal resolver mismatch
- Monty join panic/cancel (`run_due`, `worker_runs.rs`)

Sprint b-15 owns the stable origin diagnostics and persisted failure shapes;
this outline creates no separate error-surface sprint.

### 3. Test hardening — owned by b-15

The phase-end finding was that `crates/sc-cron/tests/monty_hook.rs` used fixed
`tokio::time::sleep` calls under the `sc-cron-monty` CI feature gate. Sprint
b-15 owns the condition/deadline coordination and the no-fixed-sleep gate:

- `RecordingStore` / watch-channel waits
- `common::WAIT` + `timeout` on conditions tied to feed completion, disable, delete, shutdown
- Avoid coupling correctness to wall-clock upper bounds in unit tests (`monty_compose.rs` deadline test slack)

Its acceptance requires no fixed sleeps in `monty_hook.rs` and a green Monty
job under load. No additional test-hardening sprint is implied.

### 4. Compose stall / timeout — resolved baseline, verified by b-15

The later Track B baseline includes the process-group deadline/kill behavior
that FTQ-003 requested. The phase plan explicitly forbids reopening that
implementation; b-15 verifies the landed deadline, kill, output-cap, and
the landed baseline while adding observability. b-16 then generalizes the
host policy without reimplementing process limits.

### 5. Product and template surfaces — owned by b-16

- **Template:** b-16 owns the checked `cron_tick` / `cron_stop` example and
  opt-in generated Cargo feature.
- **Operator docs:** b-16 owns the Monty/Python/Rust guidance and security
  boundary.
- **beads-cache:** remains an explicit non-goal; Monty stays sc-runtime-only
  ([decision-sync-scheduling-at-api.md](../phase-b/decision-sync-scheduling-at-api.md)).

### 6. CI and release discipline — existing sprint acceptance

- Keep **`required-features = ["monty"]`** on `monty_hook`, `monty_host_bridge`, `sc_compose_bench` (`#[ignore]` bench).
- Rust **1.96.1** for Monty tests; default workspace **1.94.1** without feature.
- No new workspace deps for the landed host bridge. The continuation sprint
  files contain their own exact dependency, boundary, and release gates.

### 7. Shared Python/Monty hook environment (owner-promoted 2026-10-28)

The daemon process environment is not the invoking agent environment. A daemon
started under one agent's `ATM_IDENTITY` / `ATM_TEAM` must not leak that
identity into another agent's schedule. The hardened plan therefore adds a
shared Python/Monty environment boundary before release:

- daemon configuration allowlists process variables by name for application
  hooks (for example `JEV_API_KEY`), with empty defaults;
- each schedule may load authorized full-path `.env` files and then `set`,
  resolve a provider `reference`, or `clear` individual names;
- one resolved snapshot is used for Python and Monty stop/tick; Python and
  both compose/ATM CLI children receive it without full daemon inheritance;
- Monty permits its existing `Getenv` / `GetEnviron` calls and denies every
  other OS call; source initialization can read the snapshot;
- explicit `set` values are deliberate plaintext registry data, while daemon,
  file, and reference values are never persisted or logged;
- Rust jobs remain trusted application code and keep their existing interface;
- because no binary or package schema has shipped, the development v1-v4
  migration ladder is replaced by one first-release schema
  rather than extended to v5.

Binding authority is [ADR-RUN-0016](../../architecture.md),
[REQ-CRON-0019](../../sc-cron/requirements.md), and
[sprint b-17](../phase-b/sprint-b-17-sc-cron-hook-environment.md).

## Reconciled ownership — not a sprint generator

| Discovery bucket | Binding owner | Disposition |
|------------------|---------------|-------------|
| General host registry + `sc_compose_run` and `atm_run` adapters | **b-16** | Amend the b-10 baseline; no new sprint |
| Canonical cron errors and events | **b-13 / b-15** | Existing continuation ownership |
| Generated Monty example and feature surface | **b-16** | Existing continuation ownership |
| Shared Python/Monty environment and release-v1 schema | **b-17** | Existing continuation ownership |
| Generated observability composition and release proof | **b-18** | Existing continuation ownership |

The authoritative phase plan already assigns the listed REQ/NFR/ADR ids and
`owned_paths`. Do not append another sprint from this historical split.

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
