# sc-runtime Requirements

Repo-level requirements for `sc-runtime`. They are extracted from
[`sc-runtime-design.md`](sc-runtime-design.md) (final input for sprint planning,
2026-09-19); where a requirement comes from a numbered entry in that document's
decision log it says so as "decision N".

Crate-level requirements live beside each crate's architecture:

| Crate | Requirements | Architecture |
|---|---|---|
| `sc-config` | [requirements](sc-config/requirements.md) | [architecture](sc-config/architecture.md) |
| `sc-transport` | [requirements](sc-transport/requirements.md) | [architecture](sc-transport/architecture.md) |
| `sc-command` | [requirements](sc-command/requirements.md) | [architecture](sc-command/architecture.md) |
| `sc-runtime` | [requirements](sc-runtime/requirements.md) | [architecture](sc-runtime/architecture.md) |

## How to read this document

Entries follow the shared SC format in
`.claude/skills/plan-hardening/req-adr-format.md`.

- `REQ-<AREA>-nnn` is a functional requirement: something the product does.
  `NFR-<AREA>-nnn` is a non-functional requirement: a constraint on how it is
  built. Both are binding.
- Each entry gives the obligation, then **Why** (the problem it solves, so a
  reader can judge whether a change still honours it) and **Verified by** (the
  evidence `req-qa` accepts; "tests pass" alone is never evidence for an NFR).
- An id is never reused or renumbered. A requirement changes only by editing
  this file in a planned, reviewed change.
- This file owns behaviour that spans crates or lives outside them (template,
  generator, wizard, release). Behaviour of one crate is owned by that crate's
  file and is not restated here.

## Product purpose

Every SC project that needs a local service has so far hand-assembled the same
stack (Tokio, Axum, an MCP server, a Clap CLI, SQL) slightly differently.
`sc-runtime` removes that work without acquiring opinions about what a project
does. It is a `cargo-generate` template plus four small, independently
published library crates. A new project supplies an answers JSON file (written
by hand, by an agent, or by a Wyvern wizard) and gets a compiling workspace with
one example operation reachable from REST, MCP and the CLI.

Three principles decide what belongs here:

1. **The framework instantiates, the project wires.** `sc-runtime` builds the
   parts and hands them over. What routes do, which store they touch and what
   data moves where are project decisions in ordinary Rust.
2. **Thin template, versioned crates.** Generated code cannot be upgraded after
   birth; a dependency can. Anything that should improve across all projects
   lives in a published crate; anything a project will edit lives in the
   template.
3. **Standardise below the application layer with general code, not extra
   requirements.** A requirement is admitted only if it results in less code.

## Repository

- `REQ-REPO-001` **One repository, one workspace, four crates.**
  The repository `randlee/sc-runtime` holds a Cargo workspace whose members are
  `crates/sc-config`, `crates/sc-transport`, `crates/sc-command` and
  `crates/sc-runtime`. Beside it sit `template/` (the `cargo-generate` template
  root), `wizard/` (options contract, fixtures, wizard pages), `scripts/`
  (generation driver), `tests/unit/` (contract tests for the generator) and
  `examples/spike/`. `template/` is not a workspace member, because its files
  contain placeholders and do not compile until rendered.
  **Why:** keeping crates and template in one workspace means template CI
  always tests the template against the crates at the same commit, so an API
  change and the template change that follows it land together (decision 16).
  **Verified by:** the root `Cargo.toml` member list; `cargo build --workspace`
  succeeds without touching `template/`.

- `REQ-REPO-002` **Each crate is an independent deliverable.**
  Each of the four crates builds and tests on its own (`cargo test -p <crate>`
  from a clean checkout), is usable without the other three or the template,
  and carries its own README, version, licence and complete crates.io metadata.
  **Why:** `sc-config`, `sc-transport` and `sc-command` are useful to programs
  that are not sc-runtime daemons, and any of them may move to its own repo
  later without an API change (decision 11).
  **Verified by:** `cargo test -p <crate>` and `cargo publish --dry-run -p
  <crate>` for each crate; a README per crate that shows standalone use.

- `REQ-REPO-003` **Generated projects depend on published versions.**
  A generated project names the four crates by crates.io version. Before the
  first publish, and when testing unreleased changes, a `[patch.crates-io]`
  entry or a git tag stands in. A path dependency is never rendered into a
  generated project.
  **Why:** a path dependency copied into each project has no upgrade path; this
  replaces the earlier plan of copying crates into projects and extracting them
  later (decision 16, and the rev 4 amendment recorded in the design).
  **Verified by:** the rendered `Cargo.toml` files for every fixture contain
  version requirements and no `path =` entry for these crates.

- `REQ-REPO-004` **Standard `just` commands.**
  The repository exposes the standard SC base commands `just lint` and
  `just test`, plus `just new <dest>` which runs the generation driver.
  **Why:** every SC repo exposes the same base commands so agents, CI and
  people use one vocabulary; the sc-lint repo is the source of truth for that
  system (decision 35).
  **Verified by:** the root `justfile`; CI calls only these recipes.

- `REQ-REPO-005` **Boundary manifests for every crate.**
  Each workspace crate has a boundary manifest under `boundaries/<crate>/` that
  records its public facade, its allowed dependents and dependencies, and its
  forbidden edges. `sc-lint-boundary` enforces the manifests through
  `just lint`.
  **Why:** the crate graph in [`architecture.md`](architecture.md) (no edges
  among the three standalone crates; nothing server-side reachable from a CLI)
  is the property most likely to erode silently, so it is checked mechanically
  and not by review. The manifests are also what sprint planning cuts parallel
  sprints from.
  **Verified by:** a manifest exists for each crate; `just lint` runs
  `sc-lint-boundary` and is clean.

## Spike: verifying the design's open facts

The design marks several facts as "Verify": they are believed true but nothing
may be planned around them until a throwaway spike proves them. These
requirements define that spike.

- `REQ-SPIKE-001` **One binary proves the stack coexists.**
  `examples/spike` is a single binary containing axum, utoipa-axum, an rmcp
  Streamable HTTP service in stateless mode mounted at `/mcp`, sqlx with
  SQLite, and a UDS listener, plus a clap client that connects with reqwest
  `unix_socket`. `curl --unix-socket`, the clap client and an MCP client all
  reach the same single service function.
  **Why:** the whole architecture assumes these libraries share one router and
  one listener. If they cannot, the crate split and the template are wrong, so
  this is proven before anything is built on it.
  **Verified by:** a recorded run of the three clients against the spike.

- `REQ-SPIKE-002` **Verify items are answered and recorded.**
  The spike answers, with recorded evidence: that rmcp 3.x, utoipa-axum 0.2 and
  axum 0.8 coexist on one router without version conflicts (decision 19); that
  rmcp `Parameters<T>` accepts a struct that also derives `ToSchema` with no
  schema clash (decision 20); the conditional `ignore`, `exclude` and
  placeholder syntax of the current `cargo-generate` release (decision 22); and
  that `cargo generate` runs fully non-interactively with
  `--template-values-file` and `--silent`, including `array` values, with
  `exclude` copying `.j2` files untouched (decision 31). Decision 21 is closed
  as not needed. Exact version pins for axum, utoipa, utoipa-axum, rmcp, sqlx,
  reqwest and cargo-generate are recorded, and the ADRs that depend on these
  facts (ADR-006, ADR-008) move from `proposed` to `accepted`, amended where
  the spike found otherwise.
  **Why:** these facts set dependency versions and template syntax for every
  later sprint; guessing them would turn each into rework.
  **Verified by:** the evidence and pins written into `architecture.md`; no ADR
  left `proposed` when the spike sprint closes.

- `REQ-SPIKE-003` **The spike is throwaway.**
  `examples/spike` is rewritten on the four crates once they exist (see
  `REQ-E2E-004`) and deleted at `v0.1.0`.
  **Why:** it is evidence, not a product; leaving it would give projects a
  second, unmaintained example to copy.
  **Verified by:** `examples/spike` absent at the `v0.1.0` tag.

## End-to-end behaviour

These are properties of the assembled system. No single crate can satisfy one
alone, so they close in integration sprints.

- `REQ-E2E-001` **One operation, three surfaces, one path to SQL.**
  An operation is defined once, as a pair of `api-types` structs and one async
  service function. It is reachable from REST, from MCP and from the CLI, and
  all three reach SQL only by calling that service function.
  **Why:** the three surfaces drifting apart, or one of them growing its own
  query, is the failure this architecture exists to prevent (decisions 3, 4).
  **Verified by:** a test that calls the example operation through each surface
  against one daemon and observes the same stored result; no sqlx call outside
  the store and service crates.

- `REQ-E2E-002` **No daemon, no fallback.**
  The CLI has no direct-database mode and does not start the daemon itself.
  When the daemon is unreachable the CLI returns the typed error
  `DAEMON.NOT_RUNNING` with `suggested_action: "run <app> daemon start"`.
  **Why:** the daemon is the only process allowed to open the local database;
  a CLI bypass would reintroduce a second writer (decisions 1, 2). Auto-start
  is an open question whose default is report-only (decision 24).
  **Verified by:** a test that runs a CLI command with no daemon and asserts
  the error code, the suggested action and a non-zero exit.

- `REQ-E2E-003` **One envelope everywhere.**
  Every surface responds with the sc-ai-cli envelope
  `{version, ok, data, error}`. The CLI's `--json` prints the envelope it
  received, unchanged.
  **Why:** agents consume these CLIs and APIs; one response shape with typed
  errors and suggested actions is what lets them branch and recover
  (decision 8).
  **Verified by:** tests comparing the REST body, the MCP tool result content
  and the CLI `--json` output for the same call.

- `REQ-E2E-004` **The crates make assembly small.**
  The spike, rewritten on the four crates, has a `main.rs` under 30 lines.
  **Why:** this is the measurable form of "the framework instantiates": if
  assembling a daemon still takes a page of code, the crates have not removed
  the hand-assembly they exist to remove.
  **Verified by:** line count of the rewritten spike's `main.rs`.

- `REQ-E2E-005` **One listener serves everything.**
  The daemon serves the project's routes, `openapi.json`, a health route and
  `/mcp` on the same listener and port.
  **Why:** MCP is part of the daemon, same binary and same port (decision 3);
  frontends generate their own TypeScript client from `openapi.json` with
  whichever generator they prefer.
  **Verified by:** a fixture test requesting all four from one endpoint.

## Template

The template is what a project owns and edits after generation. It is kept
thin: it contains the example operation and the wiring, and nothing that
should improve across projects.

- `REQ-TEMPLATE-001` **Generated workspace layout.**
  `template/` is a standard `cargo-generate` template that renders a Cargo
  workspace with the crates `api-types` (request and response structs),
  `store-sqlite` (pool, migrations, queries), `service` (async service
  functions), `daemon` (`main.rs`, `routes.rs`, and `mcp.rs` when MCP is on)
  and `cli` (clap commands), together with `config/default.json`, a gitignored
  `config/local.json`, `AGENTS.md`, `CLAUDE.md`, a `Justfile` and
  `.github/workflows/ci.yml`. Dependency edges are exactly those in the
  design's generated-project table: `service` depends on `api-types` and the
  store crates; `daemon` on `service`, `sc-runtime` and `sc-config`; `cli` on
  `api-types`, `sc-transport`, `sc-command` and `sc-config`.
  **Why:** the crate boundaries are mechanical, not stylistic: sqlx needs one
  crate per database, and the CLI must not link the daemon's dependencies.
  **Verified by:** rendering each fixture and inspecting `cargo metadata`.

- `REQ-TEMPLATE-002` **One worked example, wired end to end.**
  The template ships one example operation, `widget.create` and `widget.get`,
  wired through REST (`POST /ops/{name}`), MCP (`widget_create`, `widget_get`)
  and the CLI, with a test that uses `DaemonFixture`.
  **Why:** it is the pattern a project copies for its first real operation and
  then deletes; an example that skips a surface would teach the wrong pattern.
  **Verified by:** `just test` in a generated project exercises all three
  surfaces.

- `REQ-TEMPLATE-003` **v0.1 options.**
  The options are the project name, `db` and `mcp` (bool). In v0.1 the only
  accepted `db` value is `sqlite`; `postgres` and `both` are present in the
  schema's enum so the contract does not change shape later, and are rejected
  by validation until `store-postgres` ships. Options include or exclude whole
  files through conditional `ignore` lists in `cargo-generate.toml`
  (for example `daemon/src/mcp.rs`).
  **Why:** backends arrive in the order SQLite, Postgres, MySQL; reserving the
  values keeps the published schema stable for sc-lint (decision 39).
  **Verified by:** fixtures for `mcp` on and off generate and pass; a fixture
  with `db = postgres` fails validation.

- `REQ-TEMPLATE-004` **Liquid stays out of Rust logic.**
  In-file conditional differences (the `Stores` struct, the `.mcp(...)` line)
  total under ten lines across the whole template. Everything else varies by
  including or excluding whole files.
  **Why:** Rust source threaded with template conditionals cannot be compiled,
  linted or read as Rust, and every conditional is a variant nobody tests.
  **Verified by:** a count of Liquid control tags in `template/**/*.rs`.

- `REQ-TEMPLATE-005` **Agent documents are rendered by `sc-compose`.**
  `AGENTS.md.j2` and `CLAUDE.md.j2` are validated Jinja with frontmatter,
  listed under `exclude` in `cargo-generate.toml` so `cargo-generate` copies
  them without rendering. The driver renders them with `sc-compose` from the
  same answers and removes the `.j2` sources from the result.
  **Why:** Liquid and Jinja both use `{{ }}`, so one engine cannot own both
  file kinds; these documents match the sc-ai-cli templates, which are Jinja
  (decisions 26, 28).
  **Verified by:** a generated project contains `AGENTS.md` and `CLAUDE.md` and
  no `.j2` file.

- `REQ-TEMPLATE-006` **Plain `cargo generate` still works.**
  `cargo generate --git https://github.com/randlee/sc-runtime template --name
  <n>` works without Wyvern or the driver, using the placeholders' own prompts
  and defaults.
  **Why:** the wizard and driver are conveniences; someone without them must
  not be locked out, and the template tree must be the same in both paths.
  **Verified by:** a CI job that runs plain `cargo generate` with defaults.

- `REQ-TEMPLATE-007` **Placeholder `Justfile`.**
  The generated `Justfile` is a minimal placeholder providing `just lint`,
  `just test` and `just db-prepare`. Nothing else in the template depends on
  its contents, so `sc-lint create` can later replace the file without touching
  a workflow or document.
  **Why:** the standard `just` and lint infrastructure is owned and installed
  by sc-lint, but prototype repos and CI need stable gates before that command
  exists (decisions 36, 37).
  **Verified by:** the generated CI workflow calls only those recipe names.

- `REQ-TEMPLATE-008` **The SQLite store crate.**
  The generated `store-sqlite` crate owns its pool (a read pool of N
  connections plus a one-connection write pool, WAL on), migrations embedded
  with `sqlx::migrate!`, compile-time-checked queries, a `sqlx.toml` naming
  `SQLITE_DATABASE_URL`, and a checked-in `.sqlx/` produced by
  `just db-prepare`. It exposes `open(cfg) -> Result<Store>` and typed query
  functions. Only `service` calls it.
  **Why:** SQLite allows one writer at a time; many readers plus one write
  connection is the standard sqlx arrangement and is correct under concurrency
  with no custom code. The per-crate URL variable is what lets a second
  backend build in the same workspace later.
  **Verified by:** the generated crate builds offline from `.sqlx/`; a
  concurrent-write test passes; no other generated crate depends on sqlx.

- `REQ-TEMPLATE-009` **The generated daemon uses `sc-observability`.**
  The generated daemon uses `sc-observability` 1.2.x for logging. Its `main.rs`
  reads config, then initialises `sc-observability` directly with plain config
  values, then assembles the daemon. A project that wants OTel export adds the
  export crate itself.
  **Why:** `sc-observability` is the SC logging standard (decision 8), and it
  is deliberately initialised by the project rather than hidden inside
  `sc-config` or `sc-runtime`, so the observability crates stay independent of
  both (decision 34).
  **Verified by:** the generated `daemon/src/main.rs` and its `Cargo.toml`.

## Answers contract

- `REQ-ANSWERS-001` **One schema defines every option.**
  `wizard/answers.schema.json` is the single definition of every generation
  option: names, types, defaults, enums and conditional requirements. It
  carries a schema version and is a published contract that sc-lint also
  consumes.
  **Why:** the wizard, the driver, `cargo-generate.toml` and sc-lint all need
  the option set; one definition is the only way they cannot disagree. It is
  the entire interface between this project and sc-lint (decisions 27, 39).
  **Verified by:** the schema validates as JSON Schema and carries a version.

- `REQ-ANSWERS-002` **Schema and template cannot drift.**
  A unit test asserts that the schema's key set equals the placeholder key set
  in `template/cargo-generate.toml`.
  **Why:** an option present in one and missing from the other produces a
  template that silently ignores an answer.
  **Verified by:** `tests/unit/`, run by `just test`.

- `REQ-ANSWERS-003` **Fixtures, valid and invalid.**
  `wizard/fixtures/` holds one answers file per tested variant. Unit tests
  assert that valid fixtures validate and that invalid fixtures fail closed:
  validation errors stop generation before `cargo generate` runs.
  **Why:** the fixtures are the template's test matrix and one of the three
  things handed to the sc-lint team (schema, fixtures, prototype repos).
  **Verified by:** `tests/unit/`.

## Driver

- `REQ-DRIVER-001` **The generation pipeline.**
  `scripts/new_project.py` validates the answers against the schema, writes
  them as a `[values]` TOML file, runs `cargo generate --path template
  --template-values-file <file> --name <n>` non-interactively, runs
  `sc-compose` on the `.j2` documents, then runs `just lint` and `just test`
  inside the result. A failing step fails the run with that step named.
  **Why:** the pipeline follows the one proven in `p3-nuget-template`, with
  `cargo-generate` replacing that repo's custom render engine (decision 29).
  Running lint and test last means a generated project is known good at birth.
  **Verified by:** the driver's tests and `REQ-DRIVER-003`.

- `REQ-DRIVER-002` **Fully non-interactive mode and tool discovery.**
  `--var-file <answers.json>` runs with no prompt of any kind. `sc-compose` and
  `wyvern` are found through `$SC_COMPOSE` / `$WYVERN_BIN`, then `PATH`; a
  missing tool is reported by name with how to supply it.
  **Why:** CI and agents generate projects; neither can answer a prompt.
  **Verified by:** the driver run in CI with stdin closed.

- `REQ-DRIVER-003` **A generated project is green with no edits.**
  `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` produces a
  project where `just lint` and `just test` pass with no edits.
  **Why:** this is the MVP's exit gate for the template: the first thing a new
  project does must not be fixing the scaffold.
  **Verified by:** that command, run in CI.

## Wizard

- `REQ-WIZARD-001` **A Wyvern wizard built by copy and edit.**
  `wizard/wizard.json` and `wizard/pages/` are a Wyvern wizard made by copying
  the `p3-nuget-template` wizard (descriptor, pages, `wizard-answers.js`,
  stylesheet) and editing its fields and lists to match the schema. Nothing
  under `wizard/` is copied into a generated project.
  **Why:** the p3 wizard is a working, schema-driven example, so the work is an
  edit, not a build (decision 25). Living beside `template/` means nothing
  needs deleting from generated projects.
  **Verified by:** the wizard runs under Wyvern; generated projects contain no
  wizard file.

- `REQ-WIZARD-002` **Three entry modes.**
  With no `--var-file` the driver runs the wizard. `--prefill <json>` merges
  agent-written guesses into the wizard's prefill so a person reviews rather
  than types. `--var-file` skips the wizard entirely.
  **Why:** these are the three modes proven in p3: interactive, agent-guesses
  with human review, and fully non-interactive.
  **Verified by:** driver tests for each mode, with the wizard stubbed.

- `REQ-WIZARD-003` **The wizard produces the same contract.**
  Wizard output validates against the schema and, for the same choices, equals
  the equivalent fixture.
  **Why:** because both routes produce the same answers JSON, nothing else in
  the system depends on which one was used, and the wizard can slip or be
  reshaped without touching anything else.
  **Verified by:** a test comparing wizard output with the matching fixture.

## Release

- `REQ-RELEASE-001` **Fixture-matrix CI.**
  Template CI runs, for each `wizard/fixtures/*.json`, the driver with
  `--var-file`, then `just lint` and `just test` in the result. Wyvern is not
  needed in CI.
  **Why:** every supported option combination is generated and tested on every
  change, which is what catches an rmcp release or a crate API change breaking
  the template (decision 30).
  **Verified by:** the workflow under `.github/workflows/`.

- `REQ-RELEASE-002` **v0.1.0.**
  The four crates are published to crates.io, the template depends on the
  published versions, the fixture matrix is green against them, and the
  repository is tagged `v0.1.0`.
  **Why:** until the crates are published, `REQ-REPO-003` is satisfied only by
  a patch entry; the release is what makes generated projects upgradeable.
  **Verified by:** crates.io listings, the tag, and the matrix run on it.

## Non-functional requirements

- `NFR-REPO-001` **A CLI never links server-side dependencies.**
  A CLI binary built from these crates links none of axum, rmcp or sqlx.
  **Why:** the CLI is a thin HTTP client; linking the daemon's stack would
  multiply its build time and binary size and would make a direct-database
  shortcut possible.
  **Verified by:** `cargo tree` for the generated `cli` crate shows none of the
  three. A manifest that merely looks right is not evidence.

- `NFR-REPO-002` **Async end to end.**
  Handler, service function and sqlx pool are all async. No blocking call runs
  on a runtime thread.
  **Why:** carried unchanged from the draft's confirmed decisions; one blocking
  call in a handler stalls every request on that worker.
  **Verified by:** review of handlers and service functions; no `block_on` or
  synchronous I/O in request paths.

- `NFR-REPO-003` **Standard crates, used the documented way.**
  Every component is a standard, widely used crate used as its documentation
  shows. The repository contains no command registry, macro system, code
  generator, portable query layer, or wrapper around an Axum, rmcp or sqlx
  type.
  **Why:** these are the design's non-goals. Each would be code this project
  has to maintain and each project has to learn (decision 5).
  **Verified by:** `arch-qa` review against ADR-001; absence of proc-macro
  crates and wrapper types.

- `NFR-REPO-004` **Requirements must mean less code.**
  A requirement or boundary is admitted only if it results in less code to
  write. Nothing is built where a standard tool already does the job.
  **Why:** decision 38. This is the test applied to every proposed addition,
  including additions to this file.
  **Verified by:** plan review; a deliverable that adds custom code where a
  standard tool exists is a finding.

- `NFR-REPO-005` **Isolated, parallel tests; no fixed paths.**
  No production code hardcodes a temp or home path. Every test that touches the
  filesystem or starts a daemon uses its own tempdir instance root and runs in
  parallel with the others.
  **Why:** a fixed path or shared instance root makes tests interfere with each
  other and with a developer's real daemon; `DaemonFixture` exists so this is
  the easy path.
  **Verified by:** the test suite passes under the default parallel runner and
  when run twice concurrently; no literal `/tmp` or home path in `crates/`.

- `NFR-REPO-006` **The template carries no lint knowledge.**
  The template contains no lint configuration, boundary rules or `just`
  modules of its own.
  **Why:** that infrastructure is owned and installed by sc-lint, so when its
  rules change generated projects pick the change up from sc-lint and nothing
  here changes (decisions 17, 36). This does not restrict how this repository
  lints itself (`REQ-REPO-005`).
  **Verified by:** contents of `template/`.

- `NFR-REPO-007` **rmcp is pinned to a minor version.**
  Every manifest that names rmcp, in crates and template, pins a minor version.
  **Why:** rmcp shipped five versions in the 30 days before the design was
  written; an unpinned dependency would break generated projects at random.
  **Verified by:** the manifests; the fixture matrix catches breakage on bump.

- `NFR-REPO-008` **File size limit.**
  No non-test source file exceeds 1000 lines.
  **Why:** carried from the SC architectural rules; a file that large is a
  module boundary that was not drawn.
  **Verified by:** a line count in `just lint`.

- `NFR-REPO-009` **Library errors are values.**
  No public library function panics on caller input or environment state;
  errors are typed values.
  **Why:** these crates run inside long-lived daemons and inside CLIs driven by
  agents; a panic is an outage or an unparseable failure (decision 32
  generalised; see ADR-011).
  **Verified by:** no `unwrap`, `expect`, `panic!` or panicking index reachable
  from a public function; per-crate NFRs restate this where it applies.

## Out of scope for v0.1

Recorded so a sprint plan leaves them out deliberately. Each would add code, so
each waits until a project asks for it.

| Item | Status |
|---|---|
| `store-postgres`, `db = both` | first follow-on after the MVP |
| `store-mysql` | when the first project needs it |
| SQL Server | never; sqlx has no driver |
| SQLite batched write actor | ported from atm-core when a project measures write contention |
| `tracing` bridge into `sc-observability` | undecided since rev 2 (decision 23) |
| CLI auto-starts the daemon | undecided since rev 4 (decision 24); default report-only |
| Bearer token for TCP; Origin and Host checks on `/mcp` | ideas only; relevant only on TCP |
| Surface snapshots and adapter-drift tooling | owned by sc-lint (decision 17) |
| `sc-lint create` driver step | added when sc-lint ships that command |
| `sc-config` reload, change notification, async interop | later, possibly `sc-config-tokio` (decision 33) |
| stdio MCP clients | a stdio-to-HTTP shim is a later option |
