# sc-runtime Requirements

This document defines the canonical workspace-level requirements for `sc-runtime`.

Related documents:
- [docs/architecture.md](./architecture.md) — workspace architecture
- [docs/prd/sc-runtime-prd.md](./prd/sc-runtime-prd.md) — authoritative PRD (source of record)
- [docs/sc-runtime-core/requirements.md](./sc-runtime-core/requirements.md)
- [docs/sc-runtime-cli/requirements.md](./sc-runtime-cli/requirements.md)
- [docs/sc-runtime-mcp-stdio/requirements.md](./sc-runtime-mcp-stdio/requirements.md)
- [docs/sc-runtime-db/requirements.md](./sc-runtime-db/requirements.md)
- [docs/sc-runtime-daemon/requirements.md](./sc-runtime-daemon/requirements.md)
- [docs/sc-runtime-web/requirements.md](./sc-runtime-web/requirements.md)

---

## Purpose

This file states the workspace-level requirements for `sc-runtime`. It is the single source of truth for functional and non-functional requirements that span the workspace as a whole. Crate-specific requirements — rule details, trait contracts, internal invariants — live in the crate docs directories linked above and are not repeated here.

The PRD ([docs/prd/sc-runtime-prd.md](./prd/sc-runtime-prd.md)) is the authoritative upstream source. Requirements here are derived from and traceable to the PRD. When there is ambiguity, the PRD governs.

---

## Functional Requirements

| ID | Requirement |
|---|---|
| FR-01 | Every command exposes `--json` producing a stable `CommandEnvelope<T>`. Human output is a rendering of the same data. |
| FR-02 | Request and response types are identical across CLI, daemon RPC, and HTTP transport. No reshaping at any transport boundary. |
| FR-03 | stdio MCP server operates without a daemon. Process starts, serves JSON-RPC over stdin/stdout, exits. |
| FR-04 | HTTP MCP and supervised web require daemon. `mcp_http()` on a non-daemon builder is a compile error. |
| FR-05 | `StorageBackend` is a sealed trait. All implementations live in sc-runtime workspace crates. Consumer selects backend at construction time. |
| FR-06 | `SC_RUNTIME_DB` environment variable overrides the database path/URL on all storage backends for test isolation. |
| FR-07 | Daemon enforces singleton via PID file. A second instance detects the running daemon and exits with `DAEMON.ALREADY_RUNNING`. Stale PID files from dead processes are reclaimed automatically. |
| FR-08 | SIGTERM triggers graceful shutdown: `CancellationToken` propagated to all plugins; `shutdown()` called in reverse-init order. |
| FR-09 | SIGUSR1 sends a wake notification to all plugins without terminating the daemon. Windows equivalent: named event. |
| FR-10 | RPC protocol is identical on all transports (Unix socket, named pipe, TCP loopback): newline-delimited JSON, version field, `request_id` for correlation. |
| FR-11 | CLI command router: if daemon socket/pipe is reachable, route via IPC; otherwise execute directly. Transparent to caller. |
| FR-12 | `PluginContext` provides: injected `Logger<Running>`, `Arc<dyn StorageBackend>` (if configured), `CancellationToken`. |
| FR-13 | All sc-lint gates (boundary, portability, runtime, xwin) pass in CI from Sprint 1. |
| FR-14 | sc-runtime never panics in normal operation. All error paths return `Result`. |
| FR-15 | sc-observability logger is injected by the consumer — sc-runtime never constructs a logger internally. |
| FR-16 | `ScRuntime::run()` returns `Result<(), ScRuntimeError>` — the entry point is fallible, not `!`. |

---

## Non-Functional Requirements

| ID | Requirement |
|---|---|
| NF-01 | `sc-runtime-core` has zero SC domain crate dependencies. Enforced by `sc-lint-boundary` CI gate. |
| NF-02 | `sc-runtime-mcp-stdio` has no dependency on `sc-runtime-daemon`. Enforced by boundary definition. |
| NF-03 | Cross-platform: macOS, Linux, Windows. Every `#[cfg(unix)]` block has a Windows companion or portable fallback (PORT-010 compliant). |
| NF-04 | rusqlite bundled feature in `sc-runtime-db-sqlite`. Zero system SQLite dependency. |
| NF-05 | `unsafe_code = "forbid"` as a workspace lint. No unsafe code without a documented, justified override. |
| NF-06 | Pedantic and nursery Clippy lints at deny level, matching sc-lint workspace policy. |
| NF-07 | `cargo deny` checks for license compatibility, duplicate dependencies, and security advisories. |
| NF-08 | Edition 2024, rust-version = 1.94.1. Matches sc-lint and sc-observability. |
| NF-09 | Starting version 0.1.0 across all workspace crates. |
| NF-10 | sc-lint `full` profile (including xwin) must pass before any release tag. |

---

## Scope Rules

The following are outside the scope of `sc-runtime`. Tools that need these must implement them in the application layer.

- Domain logic of any kind — tools own this entirely.
- Agent messaging, ATM protocol, inbox/outbox — sc-runtime has no knowledge of these.
- HTTP authentication, rate limiting, and multi-tenancy.
- GUI or TUI surfaces.
- Stable FrankenSQLite native MVCC — deferred until native mode ships.
- Dynamic loadable plugins — all plugins are compiled in; no `dlopen`.

`sc-runtime` also makes three explicit assumptions: the host OS provides a filesystem (for PID files and SQLite storage); the caller injects a valid `Logger<Running>` before calling `run()`; and plugin implementations respect the `CancellationToken` — sc-runtime cannot forcibly terminate a plugin that ignores cancellation.

---

## sc-lint Gates

### Workspace-Level Gates

These gates apply to the entire workspace and are active in CI from Sprint 1.

**`sc-lint-boundary`** enforces the layered dependency rules via TOML definitions in `boundaries/`. The critical workspace-level rules are that `sc-runtime-core` carries zero SC domain crate dependencies, `sc-runtime-mcp-stdio` carries no dependency on `sc-runtime-daemon`, the `sc-runtime-db` trait crate carries no dependency on any implementation crate, and `sc-runtime-web` depends on `sc-runtime-daemon` only under the daemon feature.

**`sc-lint-portability`** enforces cross-platform correctness across all crates. The rules directly relevant at workspace level are PORT-001 through PORT-005 (path and separator checks), PORT-008 (ungated `HOME`/`USER`/`XDG_*` lookups — use `ScRuntimeHome` instead), PORT-009 (no `Command::new("sh")` or `Command::new("bash")`), and PORT-010 (every `#[cfg(unix)]` branch must have a Windows companion or portable fallback, enforcing the Unix socket / named pipe transport pair).

**`sc-lint-runtime`** applies SCB-RUNTIME-001 and SCB-RUNTIME-002 to the plugin registry, PID file handling, and RPC dispatch code in `sc-runtime-daemon`.

**xwin preflight** (`sc-lint check xwin` / `sc-lint clippy xwin`) runs `cargo xwin check` and `cargo xwin clippy` targeting `x86_64-pc-windows-msvc`. Included in the `full` lint profile. Catches platform-specific compilation failures for the named-pipe transport and any `#[cfg(windows)]` code before Windows CI.

### Crate-Level Gates

`sc-lint-tokio` (reserved, not yet implemented) is the planned home for Tokio-specific rules covering plugin task supervision, cancellation propagation, and `spawn_blocking` usage in storage backends. When it ships, sc-runtime is a primary enforcement target.

### Justfile Targets

```
just lint        → sc-lint lint full
just lint fast   → sc-lint lint fast
just lint ci     → sc-lint lint ci
just ci          → sc-lint ci
```

The `fast` profile excludes xwin to preserve low-latency local feedback. The `full` profile includes xwin when `cargo xwin` is installed. The `ci` lint profile stays aligned to real CI and does not depend on xwin. The top-level `just ci` command runs lint plus tests; `just lint ci` is lint-only.

---

## Semver Policy

`sc-runtime` is shared infrastructure — a breaking change in `sc-runtime-core::Plugin` or `StorageBackend` breaks every consumer simultaneously. `sc-lint check interfaces` (`sc-lint-version`, Phase C) is a required release gate before any crates.io publish. No crates.io publish proceeds without a clean interface version check.

`sc-lint-version` is the highest-priority release gate in the workspace. When it ships, it must be wired as a blocking CI check on every release tag. The rationale is stated in ADR-14: silent breaking changes in a shared infrastructure crate have broad blast radius across the entire tool family.

---

## Requirement Management

This file owns workspace-level requirements. Detailed crate requirements, rule mechanics, and contract specifications remain under the corresponding crate docs directories linked at the top of this document. As new crates are added to the workspace, crate-specific requirements should be added to their own docs and linked here.
