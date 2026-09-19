# sc-runtime Architecture

Repo-level architecture and ADRs. Source:
[`sc-runtime-design.md`](sc-runtime-design.md). Crate-level architecture and
ADRs live in `docs/<crate>/architecture.md`:
[sc-config](sc-config/architecture.md),
[sc-transport](sc-transport/architecture.md),
[sc-command](sc-command/architecture.md),
[sc-runtime](sc-runtime/architecture.md).

## Shape

One daemon process owns the stores. Every client (CLI, frontend, MCP client)
reaches it over HTTP through one Axum router on a UDS or TCP listener. REST
handlers and MCP tools are thin: each deserialises into an `api-types` struct
and calls the same async service function, which is the only path to SQL.

| Layer | Owner | Lives in |
|---|---|---|
| Config, transport, envelope | standalone crates | `crates/sc-config`, `crates/sc-transport`, `crates/sc-command` |
| Assembly: lock, router merge, serve, shutdown, test fixture | assembly crate | `crates/sc-runtime` |
| Operations, routes, MCP tools, CLI commands, stores, schemas | the generated project | `template/` |
| Options contract, generation | this repo's tooling | `wizard/`, `scripts/`, `tests/unit/` |

## Crate graph

```text
sc-config      (serde, serde_json)
sc-transport   (tokio, reqwest; axum behind `server`)
sc-command     (serde, sc-observability-types; axum, rmcp behind `server`)
sc-runtime  -> sc-config, sc-transport[server], sc-command[server], axum, tokio, fd-lock
```

There is no edge among `sc-config`, `sc-transport` and `sc-command`. Generated
`daemon` depends on `sc-runtime` and `sc-config`; generated `cli` depends on
`sc-transport`, `sc-command`, `sc-config` and `api-types` with no `server`
feature. Each crate's edges are recorded in `boundaries/<crate>/*.toml` and
enforced by `sc-lint-boundary`.

## ADR

Status is `accepted` unless stated. A `proposed` ADR is not yet binding and
names the sprint that must accept or amend it. An ADR changes only through a
later ADR that names the one it amends or supersedes.

### ADR-001 The framework instantiates, the project wires

Status: accepted. Amends SC scaffold ADR-013 (design decisions 6, 10).

Operations are shared async service functions in project code. `sc-runtime`
is the single assembly point and does assembly only: it owns no command
registry and wraps no Axum, rmcp or sqlx type. The project passes in an
ordinary router and an ordinary rmcp service. What routes do, which store they
touch and what data moves where are project decisions in ordinary Rust.

### ADR-002 Thin template, versioned crates

Status: accepted (decisions 11, 16).

Anything that should improve across all projects lives in a published crate;
anything a project will edit lives in the template. Crates and template share
one repo and workspace so template CI tests the crates at the same commit.
Crates are published separately to crates.io and generated projects depend on
crates.io versions, never on path dependencies.

### ADR-003 Daemon always running, HTTP only

Status: accepted. Carries SC scaffold ADR-014 (decisions 1, 2, 3, 24).

The daemon is the only process that opens the local database and holds a hard
singleton lock. All clients use HTTP. The CLI has no direct-database mode and
does not auto-start the daemon; an unreachable daemon is reported as
`DAEMON.NOT_RUNNING` with a suggested action.

### ADR-004 Crate graph and the `server` feature

Status: accepted (decision 11).

`sc-config`, `sc-transport` and `sc-command` are standalone and have no edges
among them. `sc-runtime` is the only crate that knows the others. Server-side
code in `sc-transport` and `sc-command` (anything needing axum or rmcp) sits
behind a `server` feature that is off by default, so a CLI never links axum,
rmcp or sqlx. `sc-runtime` is a daemon-only dependency.

### ADR-005 `sc-observability` is used directly, not wrapped

Status: accepted (decisions 8, 34).

`sc-observability` 1.2.x is the observability stack of every generated
project: the generated `main.rs` reads config, then initialises it directly
with plain values. The four library crates do not wrap it, re-export it, or
depend on it, and they define no logging wrapper functions, so it stays
independent of config and of assembly. The one observability dependency
inside the crates is `sc-observability-types` in `sc-command`, for the error
code and remediation types the envelope is defined on. Whether a `tracing`
bridge is acceptable is open (decision 23) and nothing may assume it.

### ADR-006 One router: REST, OpenAPI and MCP

Status: proposed; accepted or amended by sprint aa-1 (decisions 15, 18, 19, 20).

REST routes and `openapi.json` come from `utoipa-axum` `OpenApiRouter` with
`routes!`. MCP tools come from rmcp `#[tool]` / `#[tool_router]`, served by
`StreamableHttpService` in stateless mode and mounted with
`nest_service("/mcp", ...)` on the same router and listener. rmcp is pinned to
a minor version. Exact version pins are recorded here by aa-1.

### ADR-007 One store crate per backend

Status: accepted (decisions 4, 7, 12).

A store is one crate bound to one sqlx driver with its own pool, embedded
migrations, `sqlx.toml` URL variable and checked-in `.sqlx/`. No query is
written to run on more than one backend. Store crates are template code the
project owns; the framework only calls the project's stores closure. SQLite
uses a read pool plus a one-connection write pool with WAL on. Only `service`
calls store functions.

### ADR-008 Generation pipeline and the answers contract

Status: proposed; accepted or amended by sprint aa-1 (decisions 25 to 31, 39).

`wizard/answers.schema.json` is the single, versioned options contract.
`cargo-generate` renders the Rust workspace from a `[values]` file;
conditional `ignore` lists include or exclude whole files; `.j2` documents are
listed under `exclude` and rendered by `sc-compose`. The driver is a Python
script reusing p3's `run_wizard.py`. Template CI is an answers-fixture matrix
run through `--var-file`. The wizard and driver live beside `template/`, never
inside it.

### ADR-009 Lint and `just` infrastructure belong to sc-lint

Status: accepted (decisions 17, 35, 36, 37).

The template carries no lint configuration, boundary rules, adapter-drift
tooling or `just` modules; it ships a placeholder `Justfile` with the standard
command names until `sc-lint create` exists. This repo's own crates are linted
by the standard sc-lint setup, including `sc-lint-boundary` over
`boundaries/`. The only interface between the two projects is the answers JSON
and its schema.

### ADR-010 One `api-types` struct by default

Status: accepted (decisions 13, 14).

The template example derives `Serialize`, `Deserialize`, `JsonSchema` and
`ToSchema` on one `api-types` struct shared by REST, MCP and the CLI at
compile time; there is no generated Rust client. The same logical struct is
not defined twice. A project may diverge edge shapes provided both convert to
the same service-function input; nothing in these crates enforces either
choice.

### ADR-011 Errors are values

Status: accepted (decisions 32, 38).

Public library APIs return `Result<T, E>` with a typed error enum per crate.
No public function panics on caller input or environment state. Errors that
cross a process boundary are `OpError` inside the envelope.
