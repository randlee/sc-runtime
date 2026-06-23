# sc-runtime Project Plan

This document is the master project plan for the `sc-runtime` workspace. It sits between the PRD and the phase execution docs, providing the phase breakdown, sequencing rationale, and entry/exit states. Each phase doc contains the full sprint breakdown, crate delivery table, and phase-specific exit criteria.

Related documents:
- [docs/prd/sc-runtime-prd.md](../prd/sc-runtime-prd.md) — authoritative PRD
- [docs/architecture.md](../architecture.md) — crate layers and dependencies
- [docs/requirements.md](../requirements.md) — FR-01–16, NF-01–10

---

## Phase Overview

| Phase | Name | Key Deliverable | Entry State | Exit State |
|-------|------|-----------------|-------------|------------|
| A | Foundation | Working CLI + stdio MCP that tool consumers can use | Empty workspace | sc-runtime 0.1.0-alpha.1 |
| B | Storage | `StorageBackend` sealed trait + rusqlite + sqlx backends | Phase A done | 0.1.0-alpha.2 |
| C | Daemon + RPC | Full daemon lifecycle, signals, cross-platform RPC | Phase B done | 0.1.0-alpha.3 |
| D | Web + HTTP MCP | axum web server, HTTP MCP SSE endpoint | Phase C done | 0.1.0-beta.1 |
| E | Release | sc-lint-version gate, xwin CI, crates.io publish | Phase D done | 0.1.0 |

---

## Sequencing

Phases are strictly sequential. Each phase gate must pass before the next phase begins. This is not a planning preference — it is an architectural constraint. Phase B builds on Phase A's `PluginContext` and builder state machine. Phase C's daemon depends on the CLI router transport abstraction established in Phase A. Phase D's supervised web mode requires the daemon's `JoinSet` region and `Plugin` trait wiring from Phase C. Phase E's semver gate requires a stable public API surface, which only exists after Phase D stabilizes it.

Parallel sprints within a phase are permissible where there are no intra-phase dependencies, and the phase docs call out where this is safe. Cross-phase parallelism is not.

---

## Incremental Facade

The `sc-runtime` facade crate is a consumer's single dependency point. It grows each phase to expose the newly available capability, but the crate itself is present from Sprint A-1. Consumers always import `sc-runtime`, not sub-crates directly.

The typestate builder transitions track phase deliverables directly:

| Phase | New builder methods unlocked |
|-------|------------------------------|
| A | `.cli()`, `.mcp_stdio()`, `.logger()`, `.build()`, `.run()` |
| B | `.db(Arc<dyn StorageBackend>)` |
| C | `.daemon()`, `.plugin()` |
| D | `.web()` (both states), `.mcp_http()` (HasDaemon only) |
| E | No new methods — semver stability declared |

---

## sc-lint from Sprint 1

All sc-lint gates are active from the very first sprint:

- `sc-lint-boundary`: boundary TOML files written in Sprint A-1 alongside the workspace scaffold, CI gate active immediately
- `sc-lint-portability`: PORT-001–005, PORT-008, PORT-009, PORT-010 enforced from first code commit
- `sc-lint-runtime`: SCB-RUNTIME-001 and SCB-RUNTIME-002 enforced on registry and concurrency code from first relevant implementation
- `sc-lint check xwin`: added in Phase E; the `full` lint profile includes it but the `fast` and `ci` profiles used during Phases A–D do not require it

The rationale for early gate activation: boundary violations introduced early are architectural debt that compounds. A boundary violation caught in Sprint A-1 is a one-line fix; the same violation caught in Sprint C-3 requires refactoring across multiple crates.

---

## Consumer Integration Test

Beginning in Sprint A-5, a `sc-runtime-example` crate lives in the workspace. It is not published to crates.io. Its purpose is to exercise all currently available composable configurations in a realistic consumer pattern.

After Phase A, `sc-runtime-example` uses `ScRuntimeBuilder<HasCli>` with a minimal command set, verifying that the same `CommandEnvelope` round-trips identically via `--json` and stdio MCP. After each subsequent phase, the example crate is extended to exercise the new capabilities. The example crate also serves as the canonical reference implementation for the consumer documentation written in Phase E.

---

## Phase Documents

- [Phase A: Foundation](./phase-A/phase-A-plan.md) — workspace scaffold, sc-runtime-core, sc-runtime-cli, sc-runtime-mcp-stdio, facade NoCli→HasCli
- [Phase B: Storage](./phase-B/phase-B-plan.md) — StorageBackend sealed trait, rusqlite backend, sqlx backend, facade .db()
- [Phase C: Daemon + RPC](./phase-C/phase-C-plan.md) — daemon lifecycle, signals, cross-platform RPC, facade HasCli→HasDaemon
- [Phase D: Web + HTTP MCP](./phase-D/phase-D-plan.md) — axum server, HttpRouteRegistry, HTTP MCP SSE, facade .web() and .mcp_http()
- [Phase E: Release](./phase-E/phase-E-plan.md) — sc-lint-version gate, xwin CI, Windows CI (compilation + unit tests via `windows-latest` runner; full Windows daemon integration tests are post-0.1.0), API surface review, crates.io publish sequence
