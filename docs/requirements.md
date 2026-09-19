# sc-runtime Requirements

**ID Range:** REQ-RUN-0001 through REQ-RUN-0702; NFR-RUN-0001 through NFR-RUN-0009  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

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

## ID Ranges

| Range | Area | Scope |
|---|---|---|
| REQ-RUN-0001 through REQ-RUN-0005 | Repository | - |
| REQ-RUN-0101 through REQ-RUN-0103 | Spike: verifying the design's open facts | The design marks several facts as "Verify": they are believed true but nothing may be planned around them until a throwaway spike proves them. These requirements define that spike. |
| REQ-RUN-0201 through REQ-RUN-0205 | End-to-end behaviour | These are properties of the assembled system. No single crate can satisfy one alone, so they close in integration sprints. |
| REQ-RUN-0301 through REQ-RUN-0309 | Template | The template is what a project owns and edits after generation. It is kept thin: it contains the example operation and the wiring, and nothing that should improve across projects. |
| REQ-RUN-0401 through REQ-RUN-0403 | Answers contract | - |
| REQ-RUN-0501 through REQ-RUN-0503 | Driver | - |
| REQ-RUN-0601 through REQ-RUN-0603 | Wizard | - |
| REQ-RUN-0701 through REQ-RUN-0702 | Release | - |
| NFR-RUN-0001 through NFR-RUN-0009 | Non-functional requirements | - |

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

---

## REQ-RUN-0001: One repository, one workspace, four crates

**Status:** Active  

### Requirement Statement

The repository `randlee/sc-runtime` holds a Cargo workspace whose members are
`crates/sc-config`, `crates/sc-transport`, `crates/sc-command` and
`crates/sc-runtime`. Beside it sit `template/` (the `cargo-generate` template
root), `wizard/` (options contract, fixtures, wizard pages), `scripts/`
(generation driver), `tests/unit/` (contract tests for the generator) and
`examples/spike/`. `template/` is not a workspace member, because its files
contain placeholders and do not compile until rendered.

### Rationale

Keeping crates and template in one workspace means template CI
always tests the template against the crates at the same commit, so an API
change and the template change that follows it land together (decision 16).

### Success Criteria

The root `Cargo.toml` member list; `cargo build --workspace`
succeeds without touching `template/`.

---

## REQ-RUN-0002: Each crate is an independent deliverable

**Status:** Active  

### Requirement Statement

Each of the four crates builds and tests on its own (`cargo test -p <crate>`
from a clean checkout), is usable without the other three or the template,
and carries its own README, version, licence and complete crates.io metadata.

### Rationale

`sc-config`, `sc-transport` and `sc-command` are useful to programs
that are not sc-runtime daemons, and any of them may move to its own repo
later without an API change (decision 11).

### Success Criteria

`cargo test -p <crate>` and `cargo publish --dry-run -p
<crate>` for each crate; a README per crate that shows standalone use.

---

## REQ-RUN-0003: Generated projects depend on published versions

**Status:** Active  

### Requirement Statement

A generated project names the four crates by crates.io version. Before the
first publish, and when testing unreleased changes, a `[patch.crates-io]`
entry or a git tag stands in. A path dependency is never rendered into a
generated project.

### Rationale

A path dependency copied into each project has no upgrade path; this
replaces the earlier plan of copying crates into projects and extracting them
later (decision 16, and the rev 4 amendment recorded in the design).

### Success Criteria

The rendered `Cargo.toml` files for every fixture contain
version requirements and no `path =` entry for these crates.

---

## REQ-RUN-0004: Standard `just` commands

**Status:** Active  

### Requirement Statement

The repository exposes the standard SC base commands `just lint` and
`just test`, plus `just new <dest>` which runs the generation driver.

### Rationale

Every SC repo exposes the same base commands so agents, CI and
people use one vocabulary; the sc-lint repo is the source of truth for that
system (decision 35).

### Success Criteria

The root `justfile`; CI calls only these recipes.

---

## REQ-RUN-0005: Boundary manifests for every crate

**Status:** Active  

### Requirement Statement

Each workspace crate has a boundary manifest under `boundaries/<crate>/` that
records its public facade, its allowed dependents and dependencies, and its
forbidden edges. `sc-lint-boundary` enforces the manifests through
`just lint`.

### Rationale

The crate graph in [`architecture.md`](architecture.md) (no edges
among the three standalone crates; nothing server-side reachable from a CLI)
is the property most likely to erode silently, so it is checked mechanically
and not by review. The manifests are also what sprint planning cuts parallel
sprints from.

### Success Criteria

A manifest exists for each crate; `just lint` runs
`sc-lint-boundary` and is clean.

---

## REQ-RUN-0101: One binary proves the stack coexists

**Status:** Active  

### Requirement Statement

`examples/spike` is a single binary containing axum, utoipa-axum, an rmcp
Streamable HTTP service in stateless mode mounted at `/mcp`, sqlx with
SQLite, and a UDS listener, plus a clap client that connects with reqwest
`unix_socket`. `curl --unix-socket`, the clap client and an MCP client all
reach the same single service function.

### Rationale

The whole architecture assumes these libraries share one router and
one listener. If they cannot, the crate split and the template are wrong, so
this is proven before anything is built on it.

### Success Criteria

A recorded run of the three clients against the spike.

---

## REQ-RUN-0102: Verify items are answered and recorded

**Status:** Active  

### Requirement Statement

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
facts ([ADR-RUN-0202](architecture.md), [ADR-RUN-0401](architecture.md)) move from Proposed to Active, amended where
the spike found otherwise.

### Rationale

These facts set dependency versions and template syntax for every
later sprint; guessing them would turn each into rework.

### Success Criteria

The evidence and pins written into `architecture.md`; no ADR
left Proposed when the spike sprint closes.

---

## REQ-RUN-0103: The spike is throwaway

**Status:** Active  

### Requirement Statement

`examples/spike` is rewritten on the four crates once they exist (see
[REQ-RUN-0204](requirements.md)) and deleted at `v0.1.0`.

### Rationale

It is evidence, not a product; leaving it would give projects a
second, unmaintained example to copy.

### Success Criteria

`examples/spike` absent at the `v0.1.0` tag.

---

## REQ-RUN-0201: One operation, three surfaces, one path to SQL

**Status:** Active  

### Requirement Statement

An operation is defined once, as a pair of `api-types` structs and one async
service function. It is reachable from REST, from MCP and from the CLI, and
all three reach SQL only by calling that service function.

### Rationale

The three surfaces drifting apart, or one of them growing its own
query, is the failure this architecture exists to prevent (decisions 3, 4).

### Success Criteria

A test that calls the example operation through each surface
against one daemon and observes the same stored result; no sqlx call outside
the store and service crates.

---

## REQ-RUN-0202: No daemon, no fallback

**Status:** Active  

### Requirement Statement

The CLI has no direct-database mode and does not start the daemon itself.
When the daemon is unreachable the CLI returns the typed error
`DAEMON.NOT_RUNNING` with `suggested_action: "run <app> daemon start"`.

### Rationale

The daemon is the only process allowed to open the local database;
a CLI bypass would reintroduce a second writer (decisions 1, 2). Auto-start
is an open question whose default is report-only (decision 24).

### Success Criteria

A test that runs a CLI command with no daemon and asserts
the error code, the suggested action and a non-zero exit.

---

## REQ-RUN-0203: One envelope everywhere

**Status:** Active  

### Requirement Statement

Every surface responds with the sc-ai-cli envelope
`{version, ok, data, error}`. The CLI's `--json` prints the envelope it
received, unchanged.

### Rationale

Agents consume these CLIs and APIs; one response shape with typed
errors and suggested actions is what lets them branch and recover
(decision 8).

### Success Criteria

Tests comparing the REST body, the MCP tool result content
and the CLI `--json` output for the same call.

---

## REQ-RUN-0204: The crates make assembly small

**Status:** Active  

### Requirement Statement

The spike, rewritten on the four crates, has a `main.rs` under 30 lines.

### Rationale

This is the measurable form of "the framework instantiates": if
assembling a daemon still takes a page of code, the crates have not removed
the hand-assembly they exist to remove.

### Success Criteria

Line count of the rewritten spike's `main.rs`.

---

## REQ-RUN-0205: One listener serves everything

**Status:** Active  

### Requirement Statement

The daemon serves the project's routes, `openapi.json`, a health route and
`/mcp` on the same listener and port.

### Rationale

MCP is part of the daemon, same binary and same port (decision 3);
frontends generate their own TypeScript client from `openapi.json` with
whichever generator they prefer.

### Success Criteria

A fixture test requesting all four from one endpoint.

---

## REQ-RUN-0301: Generated workspace layout

**Status:** Active  

### Requirement Statement

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

### Rationale

The crate boundaries are mechanical, not stylistic: sqlx needs one
crate per database, and the CLI must not link the daemon's dependencies.

### Success Criteria

Rendering each fixture and inspecting `cargo metadata`.

---

## REQ-RUN-0302: One worked example, wired end to end

**Status:** Active  

### Requirement Statement

The template ships one example operation, `widget.create` and `widget.get`,
wired through REST (`POST /ops/{name}`), MCP (`widget_create`, `widget_get`)
and the CLI, with a test that uses `DaemonFixture`.

### Rationale

It is the pattern a project copies for its first real operation and
then deletes; an example that skips a surface would teach the wrong pattern.

### Success Criteria

`just test` in a generated project exercises all three
surfaces.

---

## REQ-RUN-0303: v0.1 options

**Status:** Active  

### Requirement Statement

The options are the project name, `db` and `mcp` (bool). In v0.1 the only
accepted `db` value is `sqlite`; `postgres` and `both` are present in the
schema's enum so the contract does not change shape later, and are rejected
by validation until `store-postgres` ships. Options include or exclude whole
files through conditional `ignore` lists in `cargo-generate.toml`
(for example `daemon/src/mcp.rs`).

### Rationale

Backends arrive in the order SQLite, Postgres, MySQL; reserving the
values keeps the published schema stable for sc-lint (decision 39).

### Success Criteria

Fixtures for `mcp` on and off generate and pass; a fixture
with `db = postgres` fails validation.

---

## REQ-RUN-0304: Liquid stays out of Rust logic

**Status:** Active  

### Requirement Statement

In-file conditional differences (the `Stores` struct, the `.mcp(...)` line)
total under ten lines across the whole template. Everything else varies by
including or excluding whole files.

### Rationale

Rust source threaded with template conditionals cannot be compiled,
linted or read as Rust, and every conditional is a variant nobody tests.

### Success Criteria

A count of Liquid control tags in `template/**/*.rs`.

---

## REQ-RUN-0305: Agent documents are rendered by `sc-compose`

**Status:** Active  

### Requirement Statement

`AGENTS.md.j2` and `CLAUDE.md.j2` are validated Jinja with frontmatter,
listed under `exclude` in `cargo-generate.toml` so `cargo-generate` copies
them without rendering. The driver renders them with `sc-compose` from the
same answers and removes the `.j2` sources from the result.

### Rationale

Liquid and Jinja both use `{{ }}`, so one engine cannot own both
file kinds; these documents match the sc-ai-cli templates, which are Jinja
(decisions 26, 28).

### Success Criteria

A generated project contains `AGENTS.md` and `CLAUDE.md` and
no `.j2` file.

---

## REQ-RUN-0306: Plain `cargo generate` still works

**Status:** Active  

### Requirement Statement

`cargo generate --git https://github.com/randlee/sc-runtime template --name
<n>` works without Wyvern or the driver, using the placeholders' own prompts
and defaults.

### Rationale

The wizard and driver are conveniences; someone without them must
not be locked out, and the template tree must be the same in both paths.

### Success Criteria

A CI job that runs plain `cargo generate` with defaults.

---

## REQ-RUN-0307: Placeholder `Justfile`

**Status:** Active  

### Requirement Statement

The generated `Justfile` is a minimal placeholder providing `just lint`,
`just test` and `just db-prepare`. Nothing else in the template depends on
its contents, so `sc-lint create` can later replace the file without touching
a workflow or document.

### Rationale

The standard `just` and lint infrastructure is owned and installed
by sc-lint, but prototype repos and CI need stable gates before that command
exists (decisions 36, 37).

### Success Criteria

The generated CI workflow calls only those recipe names.

---

## REQ-RUN-0308: The SQLite store crate

**Status:** Active  

### Requirement Statement

The generated `store-sqlite` crate owns its pool (a read pool of N
connections plus a one-connection write pool, WAL on), migrations embedded
with `sqlx::migrate!`, compile-time-checked queries, a `sqlx.toml` naming
`SQLITE_DATABASE_URL`, and a checked-in `.sqlx/` produced by
`just db-prepare`. It exposes `open(cfg) -> Result<Store>` and typed query
functions. Only `service` calls it.

### Rationale

SQLite allows one writer at a time; many readers plus one write
connection is the standard sqlx arrangement and is correct under concurrency
with no custom code. The per-crate URL variable is what lets a second
backend build in the same workspace later.

### Success Criteria

The generated crate builds offline from `.sqlx/`; a
concurrent-write test passes; no other generated crate depends on sqlx.

---

## REQ-RUN-0309: The generated daemon uses `sc-observability`

**Status:** Active  

### Requirement Statement

The generated daemon uses `sc-observability` 1.2.x for logging. Its `main.rs`
reads config, then initialises `sc-observability` directly with plain config
values, then assembles the daemon. A project that wants OTel export adds the
export crate itself.

### Rationale

`sc-observability` is the SC logging standard (decision 8), and it
is deliberately initialised by the project rather than hidden inside
`sc-config` or `sc-runtime`, so the observability crates stay independent of
both (decision 34).

### Success Criteria

The generated `daemon/src/main.rs` and its `Cargo.toml`.

---

## REQ-RUN-0401: One schema defines every option

**Status:** Active  

### Requirement Statement

`wizard/answers.schema.json` is the single definition of every generation
option: names, types, defaults, enums and conditional requirements. It
carries a schema version and is a published contract that sc-lint also
consumes.

### Rationale

The wizard, the driver, `cargo-generate.toml` and sc-lint all need
the option set; one definition is the only way they cannot disagree. It is
the entire interface between this project and sc-lint (decisions 27, 39).

### Success Criteria

The schema validates as JSON Schema and carries a version.

---

## REQ-RUN-0402: Schema and template cannot drift

**Status:** Active  

### Requirement Statement

A unit test asserts that the schema's key set equals the placeholder key set
in `template/cargo-generate.toml`.

### Rationale

An option present in one and missing from the other produces a
template that silently ignores an answer.

### Success Criteria

`tests/unit/`, run by `just test`.

---

## REQ-RUN-0403: Fixtures, valid and invalid

**Status:** Active  

### Requirement Statement

`wizard/fixtures/` holds one answers file per tested variant. Unit tests
assert that valid fixtures validate and that invalid fixtures fail closed:
validation errors stop generation before `cargo generate` runs.

### Rationale

The fixtures are the template's test matrix and one of the three
things handed to the sc-lint team (schema, fixtures, prototype repos).

### Success Criteria

`tests/unit/`.

---

## REQ-RUN-0501: The generation pipeline

**Status:** Active  

### Requirement Statement

`scripts/new_project.py` validates the answers against the schema, writes
them as a `[values]` TOML file, runs `cargo generate --path template
--template-values-file <file> --name <n>` non-interactively, runs
`sc-compose` on the `.j2` documents, then runs `just lint` and `just test`
inside the result. A failing step fails the run with that step named.

### Rationale

The pipeline follows the one proven in `p3-nuget-template`, with
`cargo-generate` replacing that repo's custom render engine (decision 29).
Running lint and test last means a generated project is known good at birth.

### Success Criteria

The driver's tests and [REQ-RUN-0503](requirements.md).

---

## REQ-RUN-0502: Fully non-interactive mode and tool discovery

**Status:** Active  

### Requirement Statement

`--var-file <answers.json>` runs with no prompt of any kind. `sc-compose` and
`wyvern` are found through `$SC_COMPOSE` / `$WYVERN_BIN`, then `PATH`; a
missing tool is reported by name with how to supply it.

### Rationale

CI and agents generate projects; neither can answer a prompt.

### Success Criteria

The driver run in CI with stdin closed.

---

## REQ-RUN-0503: A generated project is green with no edits

**Status:** Active  

### Requirement Statement

`just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` produces a
project where `just lint` and `just test` pass with no edits.

### Rationale

This is the MVP's exit gate for the template: the first thing a new
project does must not be fixing the scaffold.

### Success Criteria

That command, run in CI.

---

## REQ-RUN-0601: A Wyvern wizard built by copy and edit

**Status:** Active  

### Requirement Statement

`wizard/wizard.json` and `wizard/pages/` are a Wyvern wizard made by copying
the `p3-nuget-template` wizard (descriptor, pages, `wizard-answers.js`,
stylesheet) and editing its fields and lists to match the schema. Nothing
under `wizard/` is copied into a generated project.

### Rationale

The p3 wizard is a working, schema-driven example, so the work is an
edit, not a build (decision 25). Living beside `template/` means nothing
needs deleting from generated projects.

### Success Criteria

The wizard runs under Wyvern; generated projects contain no
wizard file.

---

## REQ-RUN-0602: Three entry modes

**Status:** Active  

### Requirement Statement

With no `--var-file` the driver runs the wizard. `--prefill <json>` merges
agent-written guesses into the wizard's prefill so a person reviews rather
than types. `--var-file` skips the wizard entirely.

### Rationale

These are the three modes proven in p3: interactive, agent-guesses
with human review, and fully non-interactive.

### Success Criteria

Driver tests for each mode, with the wizard stubbed.

---

## REQ-RUN-0603: The wizard produces the same contract

**Status:** Active  

### Requirement Statement

Wizard output validates against the schema and, for the same choices, equals
the equivalent fixture.

### Rationale

Because both routes produce the same answers JSON, nothing else in
the system depends on which one was used, and the wizard can slip or be
reshaped without touching anything else.

### Success Criteria

A test comparing wizard output with the matching fixture.

---

## REQ-RUN-0701: Fixture-matrix CI

**Status:** Active  

### Requirement Statement

Template CI runs, for each `wizard/fixtures/*.json`, the driver with
`--var-file`, then `just lint` and `just test` in the result. Wyvern is not
needed in CI.

### Rationale

Every supported option combination is generated and tested on every
change, which is what catches an rmcp release or a crate API change breaking
the template (decision 30).

### Success Criteria

The workflow under `.github/workflows/`.

---

## REQ-RUN-0702: v0.1.0

**Status:** Active  

### Requirement Statement

The four crates are published to crates.io, the template depends on the
published versions, the fixture matrix is green against them, and the
repository is tagged `v0.1.0`.

### Rationale

Until the crates are published, [REQ-RUN-0003](requirements.md) is satisfied only by
a patch entry; the release is what makes generated projects upgradeable.

### Success Criteria

Crates.io listings, the tag, and the matrix run on it.

---

## NFR-RUN-0001: A CLI never links server-side dependencies

**Status:** Active  

### Requirement Statement

A CLI binary built from these crates links none of axum, rmcp or sqlx.

### Rationale

The CLI is a thin HTTP client; linking the daemon's stack would
multiply its build time and binary size and would make a direct-database
shortcut possible.

### Success Criteria

`cargo tree` for the generated `cli` crate shows none of the
three. A manifest that merely looks right is not evidence.

---

## NFR-RUN-0002: Async end to end

**Status:** Active  

### Requirement Statement

Handler, service function and sqlx pool are all async. No blocking call runs
on a runtime thread.

### Rationale

Carried unchanged from the draft's confirmed decisions; one blocking
call in a handler stalls every request on that worker.

### Success Criteria

Review of handlers and service functions; no `block_on` or
synchronous I/O in request paths.

---

## NFR-RUN-0003: Standard crates, used the documented way

**Status:** Active  

### Requirement Statement

Every component is a standard, widely used crate used as its documentation
shows. The repository contains no command registry, macro system, code
generator, portable query layer, or wrapper around an Axum, rmcp or sqlx
type.

### Rationale

These are the design's non-goals. Each would be code this project
has to maintain and each project has to learn (decision 5).

### Success Criteria

`arch-qa` review against [ADR-RUN-0001](architecture.md); absence of proc-macro
crates and wrapper types.

---

## NFR-RUN-0004: Requirements must mean less code

**Status:** Active  

### Requirement Statement

A requirement or boundary is admitted only if it results in less code to
write. Nothing is built where a standard tool already does the job.

### Rationale

Decision 38. This is the test applied to every proposed addition,
including additions to this file.

### Success Criteria

Plan review; a deliverable that adds custom code where a
standard tool exists is a finding.

---

## NFR-RUN-0005: Isolated, parallel tests; no fixed paths

**Status:** Active  

### Requirement Statement

No production code hardcodes a temp or home path. Every test that touches the
filesystem or starts a daemon uses its own tempdir instance root and runs in
parallel with the others.

### Rationale

A fixed path or shared instance root makes tests interfere with each
other and with a developer's real daemon; `DaemonFixture` exists so this is
the easy path.

### Success Criteria

The test suite passes under the default parallel runner and
when run twice concurrently; no literal `/tmp` or home path in `crates/`.

---

## NFR-RUN-0006: The template carries no lint knowledge

**Status:** Active  

### Requirement Statement

The template contains no lint configuration, boundary rules or `just`
modules of its own.

### Rationale

That infrastructure is owned and installed by sc-lint, so when its
rules change generated projects pick the change up from sc-lint and nothing
here changes (decisions 17, 36). This does not restrict how this repository
lints itself ([REQ-RUN-0005](requirements.md)).

### Success Criteria

Contents of `template/`.

---

## NFR-RUN-0007: rmcp is pinned to a minor version

**Status:** Active  

### Requirement Statement

Every manifest that names rmcp, in crates and template, pins a minor version.

### Rationale

Rmcp shipped five versions in the 30 days before the design was
written; an unpinned dependency would break generated projects at random.

### Success Criteria

The manifests; the fixture matrix catches breakage on bump.

---

## NFR-RUN-0008: File size limit

**Status:** Active  

### Requirement Statement

No non-test source file exceeds 1000 lines.

### Rationale

Carried from the SC architectural rules; a file that large is a
module boundary that was not drawn.

### Success Criteria

A line count in `just lint`.

---

## NFR-RUN-0009: Library errors are values

**Status:** Active  

### Requirement Statement

No public library function panics on caller input or environment state;
errors are typed values.

### Rationale

These crates run inside long-lived daemons and inside CLIs driven by
agents; a panic is an outage or an unparseable failure (decision 32
generalised; see [ADR-RUN-0006](architecture.md)).

### Success Criteria

No `unwrap`, `expect`, `panic!` or panicking index reachable
from a public function; per-crate NFRs restate this where it applies.
