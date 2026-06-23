# sc-runtime Architecture

This document defines the high-level product architecture for `sc-runtime`.

Related ADRs:
- [docs/prd/sc-runtime-prd.md](./prd/sc-runtime-prd.md) — authoritative PRD
- [docs/adr/](./adr/) — Architecture Decision Records

Crate-specific architecture documentation lives under the corresponding crate docs:
- [docs/sc-runtime-core/](./sc-runtime-core/)
- [docs/sc-runtime-cli/](./sc-runtime-cli/)
- [docs/sc-runtime-mcp-stdio/](./sc-runtime-mcp-stdio/)
- [docs/sc-runtime-db/](./sc-runtime-db/)
- [docs/sc-runtime-daemon/](./sc-runtime-daemon/)
- [docs/sc-runtime-web/](./sc-runtime-web/)

---

## Ecosystem Position

`sc-runtime` is a composable Rust infrastructure workspace. It provides the standard building blocks — CLI, MCP transport, storage, daemon lifecycle, and HTTP — that every Synaptic Canvas tool needs, without those tools rebuilding them from scratch.

```
sc-runtime  (infrastructure: CLI, MCP, storage, daemon, web)
├── atm-core        (domain: agent messaging)
├── Continuity      (domain: CI/PR monitoring)
├── ci              (domain: git/gh policy enforcement)
└── [future tools]
```

Dependency flow is strictly downward. `sc-runtime` has zero dependencies on any SC domain crate. Domain crates depend on `sc-runtime`; `sc-runtime` has no knowledge of agents, messages, or any domain concept.

---

## Composability Model

The core architectural principle is that all five capability axes are independent. A tool takes exactly what it needs, and no layer forces another as a dependency.

```
CLI axis:     always
MCP axis:     none │ stdio │ http/streaming  (http requires Daemon)
Storage axis: none │ sqlite │ frankensqlite │ sqlx
Daemon axis:  no   │ yes
Web axis:     no   │ yes   (foreground without Daemon; plugin-supervised with Daemon)
```

The storage axis is fully orthogonal to all other axes — any storage backend (or none) composes with any combination of CLI, MCP, daemon, and web choices.

### Composability Matrix

All valid configurations are shown below. Every row is valid with any storage column (none, sqlite, frankensqlite, sqlx) or any combination thereof.

| Configuration | CLI | stdio MCP | Daemon | Web | http MCP | Storage |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| Minimal CLI | ✓ | | | | | any |
| CLI + stdio MCP | ✓ | ✓ | | | | any |
| CLI + web (foreground) | ✓ | | | ✓ | | any |
| CLI + stdio MCP + web | ✓ | ✓ | | ✓ | | any |
| CLI + daemon | ✓ | | ✓ | | | any |
| CLI + daemon + http MCP | ✓ | | ✓ | | ✓ | any |
| CLI + daemon + web | ✓ | | ✓ | ✓ | | any |
| CLI + daemon + web + http MCP | ✓ | | ✓ | ✓ | ✓ | any |

`http MCP` requires daemon because SSE clients maintain persistent connections that must survive beyond a single CLI invocation. `stdio MCP` has no such requirement and carries no daemon dependency.

---

## Layered Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Consumers                                │
│  CLI binary   stdio MCP   HTTP client   daemon plugin        │
└────────────────────┬─────────────────────────────────────────┘
                     │
┌────────────────────▼─────────────────────────────────────────┐
│              sc-runtime (facade + typestate builder)          │
└───┬──────────┬──────────┬──────────┬──────────┬─────────────┘
    │          │          │          │          │
┌───▼──┐  ┌───▼──┐  ┌────▼───┐  ┌───▼──┐  ┌───▼──────┐
│ cli  │  │ mcp  │  │daemon  │  │ web  │  │  db/*    │
│      │  │stdio │  │        │  │      │  │          │
└───┬──┘  └──────┘  └───┬────┘  └──────┘  └──────────┘
    │                   │
    └──────────┬────────┘
               │
┌──────────────▼───────────────────────────────────────────────┐
│                     sc-runtime-core                           │
│   Plugin trait · types · observability wiring · errors        │
└──────────────────────────────────────────────────────────────┘
```

### Crate Inventory

```
sc-runtime/
├── Cargo.toml                   # workspace root, resolver = "2"
├── rust-toolchain.toml          # edition 2024, rust-version = 1.94.1
├── deny.toml                    # cargo-deny: licenses, duplicates, advisories
├── Justfile                     # lint, test, ci, xwin profiles
├── boundaries/                  # sc-lint boundary TOML definitions
│   ├── planning.toml
│   ├── sc-runtime-core/
│   ├── sc-runtime-cli/
│   ├── sc-runtime-db/
│   ├── sc-runtime-daemon/
│   └── sc-runtime-web/
├── crates/
│   ├── sc-runtime-core/         # Plugin trait, types, observability wiring
│   ├── sc-runtime-cli/          # sc-ai-cli framework: clap, JSON contract, router
│   ├── sc-runtime-mcp-stdio/    # stdio MCP transport (no daemon dependency)
│   ├── sc-runtime-db/           # StorageBackend trait only — no implementation
│   ├── sc-runtime-db-sqlite/    # rusqlite implementation
│   ├── sc-runtime-db-fsqlite/   # FrankenSQLite implementation (future)
│   ├── sc-runtime-db-sqlx/      # sqlx implementation (sqlite + postgres)
│   ├── sc-runtime-daemon/       # daemon lifecycle, PID, signals, RPC
│   ├── sc-runtime-web/          # axum HTTP + streaming MCP (MVP)
│   └── sc-runtime/              # facade + typestate ScRuntime builder
└── docs/
    └── adr/                     # Architecture Decision Records
```

---

## Crate Dependency Rules

Boundary enforcement is active in CI via `sc-lint-boundary`. The canonical rules live in `boundaries/`. The key invariants are:

| Rule | Constraint |
|---|---|
| `sc-runtime-core` | Zero SC domain crate dependencies. Foundation for all other crates. |
| `sc-runtime-mcp-stdio` | No dependency on `sc-runtime-daemon`. Hard boundary. |
| `sc-runtime-db` | No dependency on any `sc-runtime-db-*` implementation crate. Trait-only. |
| `sc-runtime-db-sqlite` | Implements `StorageBackend`. Does not appear in `sc-runtime-db-sqlx` dependency tree. |
| `sc-runtime-db-sqlx` | Implements `StorageBackend`. Does not appear in `sc-runtime-db-sqlite` dependency tree. |
| `sc-runtime-web` | Depends on `sc-runtime-daemon` only when the daemon feature is active. |
| `sc-runtime` (facade) | Depends on all layers. No SC domain crate dependencies. |

These rules are enforced at compile time by the module graph and verified by `sc-lint-boundary` in CI. Any violation fails the CI lint gate.

---

## Layer Roles

**`sc-runtime-core`** is the foundation. It defines the `Plugin` trait and its lifecycle, `PluginContext`, `ScRuntimeHome` (the `SC_RUNTIME_HOME`-aware home directory resolver), and `ScRuntimeError` (the structured error type used across all layers). Every other crate in the workspace depends on `sc-runtime-core`; `sc-runtime-core` depends on nothing in the workspace. See [docs/sc-runtime-core/](./sc-runtime-core/) for detail.

**`sc-runtime-cli`** implements the sc-ai-cli contract as a reusable infrastructure layer. It owns `CommandEnvelope<T>`, `CliError`, `CommandId`, the built-in lifecycle commands (`start`, `stop`, `restart`, `status`, `health`), and the command router. The router transparently forwards requests over IPC when a daemon socket is reachable, and executes directly when it is not. Machine contract is primary; human output is a rendering of the same data. See [docs/sc-runtime-cli/](./sc-runtime-cli/) for detail.

**`sc-runtime-mcp-stdio`** provides an MCP server over stdin/stdout. It reads JSON-RPC 2.0 from stdin, dispatches to the same operation layer as the CLI using identical request/response types, and writes responses to stdout. It carries no daemon dependency and requires no background process — the process lifetime is the MCP client lifetime. See [docs/sc-runtime-mcp-stdio/](./sc-runtime-mcp-stdio/) for detail.

**`sc-runtime-db`** defines the `StorageBackend` sealed trait and nothing else. Any crate that needs storage accepts `Arc<dyn StorageBackend>` without taking a dependency on any concrete backend. The trait is sealed within the workspace — external crates may hold it but may not implement it, preserving the ability to evolve the interface without a breaking-change surface. See [docs/sc-runtime-db/](./sc-runtime-db/) for detail.

**`sc-runtime-db-sqlite`**, **`sc-runtime-db-fsqlite`**, and **`sc-runtime-db-sqlx`** are the concrete `StorageBackend` implementations. The sqlite backend uses `rusqlite` with the bundled feature (zero system dependency), WAL journal mode, and an `r2d2` connection pool. The sqlx backend supports SQLite and PostgreSQL via connection URL and is the path to Postgres. The FrankenSQLite backend is deferred until native MVCC mode ships a stable toggle. See [docs/sc-runtime-db/](./sc-runtime-db/) for detail.

**`sc-runtime-daemon`** manages the plugin registry, daemon singleton lifecycle, cross-platform signal handling, and the RPC server. It orchestrates plugin init, concurrent execution under a `JoinSet`, graceful shutdown on SIGTERM, and PID file singleton enforcement. All plugin tasks run as region-owned tokio tasks — no detached tasks. See [docs/sc-runtime-daemon/](./sc-runtime-daemon/) for detail.

**`sc-runtime-web`** provides the HTTP transport layer. The MVP uses axum, which is a private implementation detail — no axum types appear in any public API. The web server can run in the foreground (no daemon required) or as a supervised plugin inside the daemon. HTTP MCP uses Server-Sent Events over JSON-RPC 2.0 and requires daemon because persistent SSE connections outlive any single CLI call. See [docs/sc-runtime-web/](./sc-runtime-web/) for detail.

**`sc-runtime`** (the facade) is the single consumer entry point. It exposes a typestate builder that encodes legal configurations at the type level — `mcp_http()` and supervised `web` are only available after `.daemon()`, making illegal combinations compile errors rather than runtime panics. See [docs/sc-runtime/](./sc-runtime/) for detail.

---

## Cross-Cutting Concerns

### sc-observability Injection Model

`sc-observability` (`1.2.0`) is the only allowed structured logging dependency across the workspace. Direct use of `tracing`, `log`, or `println!` for structured output is forbidden in all `sc-runtime` crates.

`sc-runtime` never constructs a `Logger<Running>`. The application constructs its logger via `LoggerBuilder` and injects it via `.logger(logger)` on the builder. This is a hard architectural requirement — tools must own their log root, service name, sinks, and retention policy. `sc-runtime-core` holds the logger reference in `PluginContext`; backend crates receive it through context injection, not through a global. This policy is consistent with ADR-008 in sc-lint.

Log events emitted by sc-runtime itself use stable `ActionName` values with the prefix `sc_runtime.*`, allowing consumers to filter infrastructure events from domain events.

### sc-ai-cli Contract

The sc-ai-cli JSON-first contract governs all CLI and transport surfaces. The machine contract is the primary product surface; MCP-ready from day one. Key points:

- Every command exposes `--json` producing a stable `CommandEnvelope<T>`.
- Request and response types are identical across CLI, daemon RPC, and HTTP — no reshaping at any transport boundary.
- `CliError` carries `kind`, `code`, `message`, `cause`, `details`, and `suggested_action`. No opaque string errors.
- Exit codes are CLI-owned: `0` success, `1` internal, `2` usage, `3` config, `4` capability, `5` backend failure, `6` backend protocol.
- Human output is a rendering of the JSON result — never richer than `--json`.
- The command router's transport is an injected trait, enabling simulator-backed integration tests without a live daemon.

The same JSON fixtures used in CLI `--json` tests must pass against the stdio MCP path and the HTTP MCP path with no contract reshaping.

### RPC Wire Format

The RPC protocol is identical on all transports:

| Platform | Transport | Path |
|---|---|---|
| macOS / Linux | Unix domain socket | `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.sock` |
| Windows | Named pipe | `\\.\pipe\sc-{tool}` |
| Fallback | TCP loopback | port registered in `{SC_RUNTIME_HOME}/.sc/runtime/{tool}.port` |

Wire format: newline-delimited JSON. The same `CommandEnvelope<T>` and `CliError` types as the CLI — no transport-specific DTOs. One request line per connection, one response line.

```json
// Request
{"version":1,"request_id":"uuid","command":"runtime.health","payload":{}}

// Response
{"version":1,"request_id":"uuid","ok":true,"command":"runtime.health","data":{"state":"running","plugins":[...]}}
```

---

## Consumer Repository Pattern

### Standalone Repos

`sc-runtime` is a standalone repo with its own workspace. Each SC tool is also a standalone repo. They never share a workspace.

```
github/
  sc-runtime/        ← own repo, own workspace, publishes to crates.io
  continuity/        ← own repo, [patch.crates-io] during migration sprint
  atm-core/          ← own repo, [patch.crates-io] during migration sprint
  ci/                ← own repo, [patch.crates-io] during migration sprint
```

### App-Types Crate Pattern

Every SC tool defines a `{tool}-types` crate with minimal dependencies (`sc-runtime-core` and `serde` only). This crate owns all tool-specific request/response structs, CLI command shapes, and error codes. The same types are used by the CLI binary, daemon RPC, and any MCP wrapper — zero reshaping across transports. It can be published to crates.io independently to support client SDKs.

```
continuity/
  crates/
    continuity-types/     # sc-runtime-core + serde only
    continuity-plugins/   # Plugin impls; depends on continuity-types
    continuity/           # binary; sc-runtime + continuity-types + continuity-plugins
```

### `[patch.crates-io]` Co-Development

While `sc-runtime` and a consumer are being developed in parallel, the consumer workspace pins the local clone via `[patch.crates-io]`:

```toml
# continuity/Cargo.toml (workspace root)
[patch.crates-io]
sc-runtime = { path = "../sc-runtime/crates/sc-runtime" }
sc-runtime-core = { path = "../sc-runtime/crates/sc-runtime-core" }
sc-runtime-db = { path = "../sc-runtime/crates/sc-runtime-db" }
```

When `sc-runtime` `0.1.0` publishes to crates.io, remove the `[patch]` block and update version pins. No structural changes to the consumer workspace are required.

---

## Technology Stack

| Crate | Version | Purpose |
|---|---|---|
| `tokio` | 1 (full) | Async runtime |
| `tokio-util` | 0.7 | `CancellationToken` |
| `clap` | 4 (derive) | CLI argument parsing |
| `serde` / `serde_json` | 1 | Request/response serialization |
| `thiserror` | 2 | Error type derivation |
| `uuid` | 1 | RPC `request_id` generation |
| `rusqlite` | 0.31 (bundled) | SQLite — `sc-runtime-db-sqlite` only |
| `r2d2-sqlite` | 0.23 | Connection pool — `sc-runtime-db-sqlite` only |
| `sqlx` | 0.8 | sqlx backend — `sc-runtime-db-sqlx` only |
| `axum` | 0.8 | HTTP server — `sc-runtime-web` only |
| `libc` | 0.2 | POSIX signals, pid liveness (unix) |
| `sc-observability` | 1.2.0 | Structured JSONL logging (injected) |
| `sc-observability-types` | 1.2.0 | Log types shared across crates |
| `sc-lint-attributes` | 0.3 | `#[sc_lint(...)]` annotations (dev) |
| `sc-lint` | — | Boundary, portability, runtime, xwin CI gates |

---

## Architecture Management

This file owns workspace-level architecture. Crate-specific design notes remain under the corresponding crate docs directories. ADRs governing specific decisions are in [docs/adr/](./adr/). As the crate implementations land, this document should be updated to reflect the implemented surfaces rather than the planned ones.
