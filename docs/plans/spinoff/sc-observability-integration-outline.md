# Outline: sc-observability integration (generated projects + libraries)

**Audience:** Codex / plan-hardening agent  
**Status:** Planning input only  
**Bead:** **`run-nwk`** — *Inject sc-observability logger into sc-runtime and sc-cron*

## One-line goal

Generated daemons **construct** `sc-observability` 1.5.x once, retain sinks for process lifetime, and **inject** a shareable handle into **`sc-runtime`** and **`sc-cron`** so library code emits canonical **`LogEvent`** records (not orphaned `tracing` spans) into JSONL console/file sinks.

## Contract already written

Primary spec: [`docs/sc-runtime/observability-injection.md`](../../sc-runtime/observability-injection.md) (copied onto this plan branch from Track B; merge to `develop` with the observability sprint or earlier).

Summary:

| Concern | Rule |
|---------|------|
| Who builds logger | **Daemon / template `main` only** — no `LoggerBuilder` in library crates |
| Wire type | **`sc-observability-types`** trait (to be added), implemented by `sc_observability::v2::Logger` in project code only |
| Libraries | `sc-runtime`, `sc-cron` accept injected handle at build/open time |
| Boundaries | Libraries MUST NOT depend on **`sc-observability`** runtime crate until **ADR-RUN-0004** amended |
| Tests | `Noop` / in-memory fakes in library tests; full sink tests in **daemon** integration tests |
| OTel | **Non-goal** for template default (`sc-observability-otlp` opt-in on consumer daemon) |

## What already exists (partial — verify at plan time)

Track B template work (integration branch) typically includes:

- `template/crates/daemon/src/observability.rs` — centralizes logger build
- `template/config/default.json` — **file + console** sinks on
- `main.rs` refactor calling `Observability::init`
- **No** `.logger(...)` on `Daemon::builder` or `Scheduler::new` until library APIs exist

Libraries today: **`tracing`** in `sc-runtime` router and `sc-cron` with informal fields.

## Gaps this spinoff must plan

### 1. Architecture and requirements (gate first)

1. **Amend ADR-RUN-0004** — allow **`sc-observability-types`** (types + emit trait only) on:
   - `sc-runtime`
   - `sc-transport` (optional later)
   - `sc-cron`
   - Explicitly **forbid** `sc-observability` full crate on those boundaries.
2. **REQ-RUN-0005** / `boundaries/*/sc-*.toml` — update allowed dependencies; run **`sc-lint-boundary`**.
3. **REQ-RUN-0309** (ordering): config → log_root → `LoggerConfig` → build → inject → run daemon.
4. Resolve **tracing bridge** in root `docs/requirements.md`: plan migration path (dual-write vs big-bang) and removal of library `tracing` when done.

### 2. sc-observability-types (upstream or in-repo)

Planner must decide **version pin** (1.5.x) and whether trait lives in:

- published **`sc-observability-types`** patch release, or
- temporary trait in-repo (discouraged if consumers already use crates.io types).

Trait sketch (from contract doc):

- Object-safe or generic **`CanonicalLog`** / emit method taking **`LogEvent`** or builder
- **`TargetCategory`**, **`ActionName`** stable enums already in types crate
- **`Noop`** for tests

Deliverable: semver story, changelog, and sc-runtime dependency bump plan.

### 3. sc-runtime API

- **`Daemon::builder(...).logger(handle)`** (or equivalent assembly hook)
- HTTP lifecycle: request start/finish, errors → **`LogEvent`** with request id propagation (align with existing middleware)
- Keep **`tracing`** behind feature flag during migration if needed; acceptance = JSONL receives same events in integration test

**Owned paths (typical):** `crates/sc-runtime/src/`, `docs/sc-runtime/requirements.md`, `boundaries/sc-runtime/`

### 4. sc-cron API

- **`Scheduler::new(..., logger)`** or setter on builder used by template `daemon/src/cron.rs`
- Map existing cron actions to stable targets (e.g. `cron.tick`, `cron.stop`, `run_due`, Monty feed outcomes) — no ad hoc string keys
- Feature **`monty`**: optional logger on Monty paths without pulling observability into default build

**Owned paths:** `crates/sc-cron/src/`, `docs/sc-cron/requirements.md`, `boundaries/sc-cron/`

### 5. Template / generation

- Wire **`Observability::init`** return value into:
  - `Daemon::builder`
  - cron scheduler construction in `template/crates/daemon/src/cron.rs`
- **`log_root`**: explicit config vs **instance root** fallback (document in template schema)
- Generated **`config/default.json`** and **`cron.schema.json`** unchanged except documented logging fields
- Acceptance: **`cargo test`** in generated example or template integration test renders project and asserts log file created under instance root

### 6. Testing strategy

| Level | What |
|-------|------|
| Unit | Libraries with **`Noop`** logger — no `sc-observability` dev-dep |
| Template | Render + run daemon short-lived; assert JSONL line shape |
| Boundary | `sc-lint lint sc-boundary` after manifest edits |
| Regression | Existing phase A/B tests still pass without enabling file sink in CI if too heavy — planner chooses hermetic temp dir |

### 7. Release and consumer impact

- Workspace crate versions bump together
- **beads-cache / external consumers:** document that injection is **embedder responsibility**; libraries remain usable without logger (Noop default?)
- Publish order: **types patch** → **sc-runtime / sc-cron** → template

## Suggested sprint split (planner may merge)

| Candidate id | Focus | Depends on |
|--------------|-------|------------|
| **obs-1** | ADR-RUN-0004 + boundaries + types trait + Noop | develop |
| **obs-2** | sc-runtime injection + HTTP LogEvent migration | obs-1 |
| **obs-3** | sc-cron injection + cron target taxonomy | obs-1; parallel with obs-2 if paths disjoint |
| **obs-4** | Template wire-up + e2e log assertion | obs-2, obs-3 |

Single sprint is possible if one implementer and narrow scope (template + sc-runtime only, cron deferred)—planner should use **plan-scope-reviewer** if width > 2 parallel tracks.

## Acceptance themes

- ADR-RUN-0004 and REQ-RUN-0005 updated; boundary lint clean
- No library `Cargo.toml` lists `sc-observability` (runtime)
- Rendered daemon writes **`<log_root>/logs/<service>.log.jsonl`** with at least one HTTP and one cron-related event in smoke test
- **`run-nwk`** closable with link to sprint ids

## Non-goals

- OTLP exporter in default template
- Injecting into every workspace crate (`sc-config`, `sc-transport` silent unless promoted)
- Replacing consumer-owned logging outside generated daemon

## References

- [`docs/sc-runtime/observability-injection.md`](../../sc-runtime/observability-injection.md)
- ADR-RUN-0004, REQ-RUN-0309, NFR-RT-0002 in `docs/architecture.md` / `docs/requirements.md`
- `template/crates/daemon/src/observability.rs`, `main.rs`, `cron.rs`
- Phase B close doc: **`run-nwk`** listed as post–Track B follow-on ([phase-b-integration-close.md](../phase-b/phase-b-integration-close.md))

## Coordination with Monty spinoff

Orthogonal tracks: observability injection may touch `sc-cron` scheduler signatures same sprint as Monty hardening—planner should sequence **`owned_paths`** to avoid merge conflicts (observability API vs Monty error strings) or combine into one integration branch with explicit file ownership table.
