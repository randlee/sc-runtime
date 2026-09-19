# sc-runtime Requirements

Repo-level requirements. Source: [`sc-runtime-design.md`](sc-runtime-design.md)
(final input for sprint planning, 2026-09-19). Crate-level requirements live in
`docs/<crate>/requirements.md`:
[sc-config](sc-config/requirements.md),
[sc-transport](sc-transport/requirements.md),
[sc-command](sc-command/requirements.md),
[sc-runtime](sc-runtime/requirements.md).

Every `REQ-*` and `NFR-*` id here is binding. An id is never reused or
renumbered. A requirement changes only by editing this file in a planned,
reviewed change.

## Product Purpose

`sc-runtime` is a `cargo-generate` template plus four independently published
library crates that together instantiate a working daemon: Tokio, Axum, an MCP
server on the same router, a Clap CLI that talks to the daemon over HTTP, and
one or two sqlx stores. The framework instantiates; the project wires.

## Repository

- `REQ-REPO-001` One repository holds a Cargo workspace whose members are
  `crates/sc-config`, `crates/sc-transport`, `crates/sc-command` and
  `crates/sc-runtime`. `template/`, `wizard/`, `scripts/`, `tests/unit/` and
  `examples/spike/` sit beside it; `template/` is not a workspace member.
- `REQ-REPO-002` Each of the four crates is an independent deliverable: it
  builds and tests alone, and carries its own README, version and complete
  crates.io metadata.
- `REQ-REPO-003` Generated projects depend on the four crates by crates.io
  version. Before first publish, and for testing unreleased changes, a
  `[patch.crates-io]` entry or a git tag stands in. Path dependencies are
  never rendered into a generated project.
- `REQ-REPO-004` The repo exposes the standard base `just` commands
  (`just lint`, `just test`) plus `just new <dest>`.
- `REQ-REPO-005` Each workspace crate has a boundary manifest under
  `boundaries/<crate>/` enforced by `sc-lint-boundary` through `just lint`.

## Spike (decision verification)

- `REQ-SPIKE-001` `examples/spike` is one binary with axum, utoipa-axum, an
  rmcp stateless Streamable HTTP mount at `/mcp`, sqlx SQLite and a UDS
  listener, plus a clap client using reqwest `unix_socket`.
  `curl --unix-socket`, the client and an MCP client all reach one service
  function.
- `REQ-SPIKE-002` The spike answers design decisions 19, 20, 22 and 31 with
  recorded evidence, records exact version pins for axum, utoipa,
  utoipa-axum, rmcp, sqlx, reqwest and cargo-generate, and moves the ADRs that
  depend on them from `proposed` to `accepted` (or amends them). Decision 21
  is closed as not needed.
- `REQ-SPIKE-003` `examples/spike` is deleted at v0.1.0.

## End-to-end behaviour

- `REQ-E2E-001` One operation defined once (a pair of `api-types` structs and
  one async service function) is reachable from REST, MCP and the CLI, and all
  three reach SQL only through that service function.
- `REQ-E2E-002` The CLI has no direct-database mode and does not auto-start
  the daemon. When the daemon is unreachable the CLI returns the typed error
  `DAEMON.NOT_RUNNING` with `suggested_action: "run <app> daemon start"`.
- `REQ-E2E-003` Every surface responds with the sc-ai-cli envelope
  `{version, ok, data, error}`. The CLI's `--json` prints the envelope
  unchanged.
- `REQ-E2E-004` The spike, rewritten on the four crates, has a `main.rs` under
  30 lines.
- `REQ-E2E-005` The daemon serves `openapi.json` and a health route on the
  same listener as the project's routes and `/mcp`.

## Template

- `REQ-TEMPLATE-001` `template/` is a standard `cargo-generate` template that
  renders a Cargo workspace with crates `api-types`, `store-sqlite`,
  `service`, `daemon` and `cli`, `config/default.json`, a gitignored
  `config/local.json`, `AGENTS.md`, `CLAUDE.md`, a `Justfile` and
  `.github/workflows/ci.yml`, with the dependency edges in the design's
  generated-project table.
- `REQ-TEMPLATE-002` The template ships one example operation, `widget.create`
  and `widget.get`, wired through REST (`POST /ops/{name}`), MCP and the CLI,
  with a test that uses `DaemonFixture`.
- `REQ-TEMPLATE-003` v0.1 options are the project name, `db = sqlite` and
  `mcp` (bool). `db = postgres | both` are reserved in the schema and rejected
  by validation until `store-postgres` ships. Whole files are included or
  excluded by conditional `ignore` lists in `cargo-generate.toml`.
- `REQ-TEMPLATE-004` Liquid stays out of Rust logic: in-file conditional
  differences total under ten lines across the template.
- `REQ-TEMPLATE-005` `AGENTS.md.j2` and `CLAUDE.md.j2` are validated Jinja
  listed under `exclude` in `cargo-generate.toml`, rendered by `sc-compose`
  from the same answers; the `.j2` sources are removed from the result.
- `REQ-TEMPLATE-006` Plain `cargo generate --git <repo> template --name <n>`
  works without Wyvern or the driver, using placeholder prompts and defaults.
- `REQ-TEMPLATE-007` The generated `Justfile` is a minimal placeholder that
  provides `just lint` and `just test` (and `just db-prepare`); nothing else
  in the template depends on its contents.
- `REQ-TEMPLATE-008` The generated `store-sqlite` crate owns its pool
  (read pool plus a one-connection write pool, WAL on), embedded migrations,
  checked queries, `sqlx.toml` naming `SQLITE_DATABASE_URL`, and a checked-in
  `.sqlx/`. Nothing but `service` calls it.
- `REQ-TEMPLATE-009` The generated daemon uses `sc-observability` 1.2.x for
  logging: `main.rs` reads config, then initialises `sc-observability`
  directly with plain config values before assembling the daemon. A project
  adds the OTel export crate itself.

## Answers contract

- `REQ-ANSWERS-001` `wizard/answers.schema.json` is the single definition of
  every option: names, types, defaults, enums and conditional requirements. It
  carries a schema version and is a published contract also consumed by
  sc-lint.
- `REQ-ANSWERS-002` A unit test asserts that the schema's key set equals the
  placeholder key set in `template/cargo-generate.toml`.
- `REQ-ANSWERS-003` `wizard/fixtures/` holds one answers file per tested
  variant; unit tests assert valid fixtures validate and invalid fixtures fail
  closed.

## Driver

- `REQ-DRIVER-001` `scripts/new_project.py` validates the answers against the
  schema, writes them as a `[values]` TOML file, runs
  `cargo generate --path template --template-values-file <file> --name <n>`
  non-interactively, runs `sc-compose` on the `.j2` documents, then runs
  `just lint` and `just test` in the result. Any failing step fails the run.
- `REQ-DRIVER-002` `--var-file <answers.json>` runs fully non-interactively.
  `sc-compose` and `wyvern` are found through `$SC_COMPOSE` / `$WYVERN_BIN` or
  `PATH`.
- `REQ-DRIVER-003` `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json`
  produces a project where `just lint` and `just test` are green with no
  edits.

## Wizard

- `REQ-WIZARD-001` `wizard/wizard.json` and `wizard/pages/` are a Wyvern
  wizard built by copying the `p3-nuget-template` wizard and editing fields
  and lists to match the schema. Nothing under `wizard/` is copied into a
  generated project.
- `REQ-WIZARD-002` With no `--var-file` the driver runs the wizard;
  `--prefill <json>` merges agent-written guesses into the wizard's prefill.
- `REQ-WIZARD-003` Wizard output validates against the schema and equals the
  equivalent fixture.

## Release

- `REQ-RELEASE-001` Template CI is a fixture matrix: for each
  `wizard/fixtures/*.json` it runs the driver with `--var-file`, then
  `just lint` and `just test` in the result. Wyvern is not needed in CI.
- `REQ-RELEASE-002` The four crates are published to crates.io, the template
  depends on the published versions, the matrix is green against them, and
  the repo is tagged `v0.1.0`.

## Non-functional requirements

- `NFR-REPO-001` A CLI binary built from these crates links neither axum nor
  sqlx (nor rmcp). Proven from the dependency tree, not from manifests.
- `NFR-REPO-002` Everything is async end to end: handler, service function,
  sqlx pool. No blocking call on a runtime thread.
- `NFR-REPO-003` Every component is a standard, widely used crate used the
  documented way. The repo contains no command registry, macro system, code
  generator, portable query layer, or wrapper around an Axum, rmcp or sqlx
  type.
- `NFR-REPO-004` A requirement or boundary is admitted only if it results in
  less code to write. Nothing is built where a standard tool already does the
  job.
- `NFR-REPO-005` No production code hardcodes a temp or home path. Every test
  that touches the filesystem or starts a daemon uses its own tempdir instance
  root and runs in parallel with the others.
- `NFR-REPO-006` The template carries no lint configuration, boundary rules or
  `just` modules of its own; that infrastructure is owned and installed by
  sc-lint.
- `NFR-REPO-007` rmcp is pinned to a minor version in every manifest that
  names it.
- `NFR-REPO-008` No non-test source file exceeds 1000 lines.
- `NFR-REPO-009` No public library function panics on caller input or
  environment state; errors are typed values.

## Out of scope for v0.1

`store-postgres`, `store-mysql` and `db = both`; SQL Server; the SQLite batched
write actor; a `tracing` bridge into `sc-observability`; CLI auto-start; OTel
export wrapping; bearer tokens and Origin/Host checks; surface snapshots and
adapter-drift tooling (owned by sc-lint); the `sc-lint create` driver step;
`sc-config` reload, change notification and async interop; stdio MCP.
