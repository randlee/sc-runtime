# sc-runtime Requirements

**ID Range:** REQ-RUN-0001 through REQ-RUN-0702; NFR-RUN-0001 through NFR-RUN-0009  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

Repo-level requirements for `sc-runtime`. They are extracted from
[`sc-runtime-design.md`](sc-runtime-design.md) (final input for sprint planning,
2026-09-19). Each requirement states its facts in full, so it can be read
without that document.

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
| REQ-RUN-0301 through REQ-RUN-0310 | Template | The template is what a project owns and edits after generation. It is kept thin: it contains the example operation and the wiring, and nothing that should improve across projects. |
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
| `store-postgres`, `db = both`, and a Postgres service container in template CI | v0.2, after the MVP |
| `store-mysql` | when the first project needs it |
| SQL Server | never; sqlx has no driver |
| SQLite batched write actor | ported from atm-core when a project measures write contention |
| `tracing` bridge into `sc-observability` | undecided |
| CLI auto-starts the daemon | undecided; default report-only |
| Bearer token for TCP; Origin and Host checks on `/mcp` | ideas only; relevant only on TCP |
| Surface snapshots and adapter-drift tooling | owned by sc-lint |
| `sc-lint create` driver step | added when sc-lint ships that command |
| `sc-config` reload, change notification, async interop | later, possibly `sc-config-tokio` |
| stdio MCP clients | a stdio-to-HTTP shim is a later option |
| Forwarding, replication, an outbox or routing policy between stores | never; data flow between stores is project code |
| A query layer portable across database backends | never; each store crate has one fixed backend |
| Web-application features: HTML templating, sessions, user authentication | never; this is not a web-app framework |
| Adopting an existing scaffold (loco-rs, rust-web-app, axum-postgres-template) | never; none has the daemon plus thin-client split, so they stay pattern references |
| Generating the prototype set of example repositories and handing it to the sc-lint team | first step after the MVP |
| Updating the sc-ai-cli Rust `.j2` templates to match this template | after the MVP, after `store-postgres` |
| Migrating atm-core onto the four crates | stretch goal after the MVP; also when its write actor would be ported back |

---

## REQ-RUN-0001: Repository layout: one workspace, four library crates

**Status:** Active  

### Requirement Statement

The GitHub repository `randlee/sc-runtime` MUST contain one Cargo workspace,
declared in the root `Cargo.toml`.

The workspace members MUST be exactly these four library crates:

| Member path | Crate name | Purpose |
|---|---|---|
| `crates/sc-config` | `sc-config` | JSON config files with environment overrides |
| `crates/sc-transport` | `sc-transport` | UDS/TCP listener and client connector |
| `crates/sc-command` | `sc-command` | response envelope, `OpError`, REST and MCP conversions |
| `crates/sc-runtime` | `sc-runtime` | daemon assembly: `Daemon::builder()`, `daemon.lock`, `DaemonFixture` |

The repository root MUST also contain these directories and files, none of
which is a workspace member:

| Path | Contents |
|---|---|
| `template/` | the `cargo-generate` template root, including `template/cargo-generate.toml`, `template/AGENTS.md.j2` and `template/CLAUDE.md.j2` |
| `wizard/` | `wizard.json`, `answers.schema.json` (the options contract), `pages/` (wizard HTML pages), `fixtures/` (one answers JSON file per tested variant) |
| `scripts/` | `new_project.py` (the generation driver) and `run_wizard.py` |
| `tests/unit/` | contract tests for the generator: schema validity, key-set match, invalid fixtures failing closed |
| `examples/spike/` | the throwaway spike; present only until the `v0.1.0` tag ([REQ-RUN-0103](requirements.md)) |
| `docs/` | requirements and architecture files; the ADRs are sections of the architecture files (see below) |
| `boundaries/` | one directory per library crate holding its `sc-lint-boundary` manifest ([REQ-RUN-0005](requirements.md)) |
| `justfile` | the `just` recipes, including `just new <dest>` |
| `.github/workflows/` | crate CI and the template fixture-matrix CI |

`template/` MUST NOT be a workspace member and MUST NOT be matched by a
workspace `members` glob, because its files contain `cargo-generate`
placeholders and do not compile until rendered.

Nothing under `wizard/` or `scripts/` may be placed inside `template/`, so
that no generated project ever contains a wizard or driver file. This item
owns that rule; items about the wizard or the driver refer to it.

Decided in this document: ADRs live in the architecture files
(`docs/architecture.md` and `docs/<crate>/architecture.md`). The sc-runtime
design's `docs/adr/` directory is not used, and the repository MUST NOT
contain it.

**OPEN:** whether `examples/spike` is a fifth workspace member, an excluded
standalone package, or a Cargo example while it exists is not decided. The
"exactly four" rule above applies in full once it is deleted.

### Rationale

Keeping the crates and the template in one repository means a crate API
change and the template change that follows it land in one pull request, and
template CI can test the template against the crates at the same commit. How
a generated project is pointed at the checked-out crates is not decided here:
it is the `[patch.crates-io]` question recorded as OPEN in
[REQ-RUN-0003](requirements.md). Keeping `template/` out of the workspace
keeps `cargo build --workspace` working, because the template's Rust files
are not valid Rust until rendered.

### Success Criteria

1. Inspect the root `Cargo.toml`: `[workspace] members` resolves to exactly
   `crates/sc-config`, `crates/sc-transport`, `crates/sc-command` and
   `crates/sc-runtime`. `cargo metadata --no-deps --format-version 1` lists
   exactly those four packages while `examples/spike/` does not exist, and
   lists no package whose manifest path is under `template/`.
2. `cargo build --workspace` run at the repository root exits 0 and compiles
   no file under `template/`.
3. Each path in the second table exists at the repository root (with
   `examples/spike/` exempt at and after the `v0.1.0` tag).
4. `find template -path '*wizard*' -o -name 'new_project.py' -o -name
   'run_wizard.py'` prints nothing.
5. `test -d docs/adr` fails (the directory does not exist), and
   `grep -c '^## ADR-' docs/architecture.md` prints a number greater than 0.

---

## REQ-RUN-0002: Each crate is an independent deliverable

**Status:** Active  

### Requirement Statement

This requirement applies to each of the four workspace crates: `sc-config`,
`sc-transport`, `sc-command` and `sc-runtime`, each under `crates/<crate>/`.

1. Each crate MUST build and pass its own tests when selected alone, with its
   default cargo features: `cargo test -p <crate>` from a clean checkout
   exits 0. The `just test` recipe runs these four commands
   ([REQ-RUN-0004](requirements.md)).
2. `sc-config`, `sc-transport` and `sc-command` MUST each be usable by a
   program that depends on none of the other three crates and does not use
   the template. None of these three may depend on another of the four
   crates.
3. `sc-runtime` is the assembly crate. It is the only one of the four that
   MAY depend on the other three.
4. Each crate MUST have its own `crates/<crate>/README.md` containing a
   compiling usage example that uses that crate alone (for `sc-runtime`, the
   example may use the other three, since it assembles them). The sc-runtime
   design requires only that each crate has its own README; that the README
   holds an example, and that the example is compiled as a doctest, is
   decided in this document.
5. Each crate MUST have its own `version` in its `Cargo.toml` `[package]`
   table. The four versions are independent and need not be equal.
6. Each crate's `[package]` table MUST set `name`, `version`, `description`,
   `license`, `repository` and `readme`, so that the crate is publishable to
   crates.io as it stands. This field list is decided in this document; the
   design says only "crates.io metadata".
7. Each crate MUST be published to crates.io separately (one
   `cargo publish -p <crate>` per crate).

**OPEN:** the licence identifier to put in `license` (and whether a licence
file is shipped per crate) is not decided.

**OPEN:** the minimum cargo version this repository requires is not decided.
Success criterion 2 needs a cargo whose `cargo publish` accepts
`--workspace`.

### Rationale

`sc-config` (configuration), `sc-transport` (a local daemon and CLI pair) and
`sc-command` (a JSON-first response envelope) are useful to programs that are
not sc-runtime daemons, so each must stand alone. Independence also means any
crate can move to its own repository later without an API change, and lets a
CLI depend on `sc-transport` and `sc-command` without pulling in the daemon's
dependencies.

### Success Criteria

1. For each of the four crates, `cargo test -p <crate>` exits 0 from a clean
   checkout.
2. For each of `sc-config`, `sc-transport` and `sc-command`,
   `cargo publish --dry-run -p <crate>` exits 0 and prints no warning about
   missing manifest metadata. `sc-runtime` depends on those three, and a
   per-crate dry run resolves them from crates.io, where they do not exist
   before the first publish. So until all three are on crates.io,
   `sc-runtime` is checked with `cargo publish --dry-run --workspace`
   (exits 0, no metadata warning); after that, with
   `cargo publish --dry-run -p sc-runtime`. This split is decided in this
   document.
3. For each `<crate>` of `sc-config`, `sc-transport` and `sc-command`,
   `cargo tree -p <crate> -e normal --prefix none` prints no line beginning
   with `sc-config `, `sc-transport `, `sc-command ` or `sc-runtime ` (the
   name followed by a space) other than the line of `<crate>` itself. For
   `sc-transport` and `sc-command` the same holds with `--features server`
   added.
4. Each `crates/<crate>/README.md` exists and contains a Rust code block
   showing use of that crate; the code block is compiled as a doctest (for
   example through `#![doc = include_str!("../README.md")]`) so it cannot
   rot.
5. Each `crates/<crate>/Cargo.toml` sets the six `[package]` fields listed in
   obligation 6, and none of them is inherited from a value that would be
   wrong for that crate.

---

## REQ-RUN-0003: Generated projects depend on published versions

**Status:** Active  

### Requirement Statement

"Generated project" means the Cargo workspace that `cargo generate` renders
from `template/`. "The four crates" means `sc-config`, `sc-transport`,
`sc-command` and `sc-runtime`.

1. Every `Cargo.toml` rendered into a generated project that depends on one
   of the four crates MUST name it by a crates.io version requirement (a
   `version = "..."` key or the bare-string form).
2. The source of the four crates MUST NOT be copied into a generated project.
3. A rendered `Cargo.toml` MUST NOT contain a `path = ...` dependency on any
   of the four crates in a dependency table (`[dependencies]`,
   `[dev-dependencies]`, `[build-dependencies]`, `[workspace.dependencies]`
   or a `[target.*]` form of these). This obligation does not apply to a
   `[patch.crates-io]` table, whose entries are normally `path` or `git`
   entries. (Path dependencies between the generated project's own crates,
   such as `daemon` on `service`, are unaffected.)
4. Before a crate's first crates.io publish, and when testing unreleased
   crate changes, the dependency MAY instead be satisfied by a
   `[patch.crates-io]` entry or by a git dependency on a tag of
   `https://github.com/randlee/sc-runtime`. The version requirement in
   obligation 1 stays in place when a `[patch.crates-io]` entry is used. When
   the git form is used, the dependency names the repository URL and a `tag`
   in place of the version requirement.
5. Which generated crate depends on which of the four crates is not stated
   here. It is owned by [REQ-RUN-0301](requirements.md), which holds the
   dependency table of the generated workspace.

**OPEN:** who writes the `[patch.crates-io]` entry in the pre-publish case
(a template option, the driver `scripts/new_project.py`, or the CI workflow)
is not decided. The same undecided mechanism is what would let template CI
build a generated project against the crates of the checkout under test
([REQ-RUN-0701](requirements.md)).

**OPEN:** the form of the version requirement (for example caret on the
minor version) is not decided.

### Rationale

Generated code cannot be upgraded after it is created; a versioned dependency
can, with `cargo update`. A path dependency or a copy of the crate source
inside each project would mean every fix has to be re-applied by hand in
every project. Everyone who owns a generated project is affected.

### Success Criteria

1. For every answers file in `wizard/fixtures/`, render the project and
   inspect every rendered `Cargo.toml`: each dependency on `sc-config`,
   `sc-transport`, `sc-command` or `sc-runtime` has a version requirement,
   or, where the git form of obligation 4 is in use, a `git` URL of
   `https://github.com/randlee/sc-runtime` with a `tag`.
2. In the same rendered files, `grep -n 'path *='` finds no line belonging
   to a dependency on any of the four crates outside a `[patch.crates-io]`
   table.
3. The rendered project contains no directory named `sc-config`,
   `sc-transport`, `sc-command` or `sc-runtime`.

---

## REQ-RUN-0004: Root `justfile` recipes `lint`, `test` and `new`

**Status:** Active  

### Requirement Statement

The file `justfile` at the root of the `randlee/sc-runtime` repository MUST
define these three recipes:

| Recipe | Behaviour |
|---|---|
| `just lint` | Runs every static check for this repository and exits non-zero if any check fails. It MUST include the `sc-lint-boundary` check ([REQ-RUN-0005](requirements.md)) and the 1000-line source file limit ([NFR-RUN-0008](requirements.md)). |
| `just test` | Runs every command of the `just test` list below, and exits non-zero if any of them fails. |
| `just new <dest>` | Runs the generation driver `scripts/new_project.py` with `<dest>` as the destination directory of the new project. Any further arguments (for example `--var-file <answers.json>` or `--prefill <json>`) MUST be passed through to the driver unchanged. |

`just test` MUST run all of these. The list is decided in this document; the
sc-runtime design names only the recipe.

1. `cargo test --workspace`.
2. `cargo test -p sc-config`, `cargo test -p sc-transport`,
   `cargo test -p sc-command` and `cargo test -p sc-runtime`, each with
   default cargo features. These are needed because `cargo test --workspace`
   builds `sc-transport` and `sc-command` once with their `server` feature
   on (`sc-runtime` enables it, and cargo unifies features across the
   packages it builds together), so it never tests their default-feature
   build.
3. `cargo test -p sc-transport --features server` and
   `cargo test -p sc-command --features server`.
4. The generator contract tests under `tests/unit/`.

`lint` and `test` are the standard SC base command names; they MUST NOT be
renamed or aliased to other primary names.

Decided in this document: every job in `.github/workflows/` that lints, tests
or generates MUST do so by calling one of these recipes, not by calling
`cargo`, `python` or a lint tool directly. There is exactly one recorded
exception: the job required by [REQ-RUN-0306](requirements.md), which proves
that plain `cargo generate` works without the driver, and therefore runs
`cargo generate --path template ...` and `cargo build --workspace` directly.

A workflow file under `.github/workflows/` (the crate CI workflow) MUST run
`just lint` and `just test` at the repository root on every pull request.

**OPEN:** the full list of tools `just lint` runs in this repository (for
example formatting and clippy settings) is not stated; the standard SC `just`
system owned by the sc-lint repository is the source of truth for it.

### Rationale

Every SC repository exposes the same base `just` commands so that agents, CI
and people use one vocabulary and need no per-repository knowledge to lint or
test. Routing CI through the same recipes means a green local `just lint` and
`just test` predicts a green CI run.

### Success Criteria

1. `just --list` at the repository root shows `lint`, `test` and `new`.
2. `just lint` and `just test` each exit 0 on a clean checkout of the default
   branch.
3. Introducing a failing unit test in any workspace crate makes `just test`
   exit non-zero; introducing a failing test under `tests/unit/` does the
   same.
4. Introducing, in `crates/sc-command`, a failing test gated with
   `#[cfg(not(feature = "server"))]` makes `just test` exit non-zero, and so
   does a failing test gated with `#[cfg(feature = "server")]`. This shows
   both feature configurations are tested.
5. `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` invokes
   `scripts/new_project.py` with that destination and that `--var-file`
   argument (check by inspecting the recipe body).
6. Inspect every file in `.github/workflows/`: each lint, test or generate
   step is a `just lint`, `just test` or `just new` invocation, except the
   steps of the one plain-`cargo generate` job of
   [REQ-RUN-0306](requirements.md).
7. One workflow file under `.github/workflows/` has a `pull_request` trigger
   and steps running `just lint` and `just test` at the repository root, and
   a pull request's check list shows a run of that workflow.

---

## REQ-RUN-0005: Boundary manifests for every crate

**Status:** Active  

### Requirement Statement

This requirement is decided in this document; the sc-runtime design does not
state it.

1. For each of the four workspace crates (`sc-config`, `sc-transport`,
   `sc-command`, `sc-runtime`) the repository MUST contain a boundary
   manifest: one or more TOML files under `boundaries/<crate>/` at the
   repository root, in the format read by the `sc-lint-boundary` tool.
2. The manifest format, as observed in the sc-lint repository, is TOML with
   these keys: `boundary_id`, `owner_package`, a `[public]` table with
   `facade`, an `[implementation]` table, a `[composition]` table with
   `roots`, a `[dependencies]` table with `allowed_dependents`,
   `allowed_dependencies` and `forbidden_edges` (each forbidden edge a string
   of the form `"a -> b"`), and a `[references]` table. Each manifest MUST
   record the crate's public facade (the items it exports) in
   `[public] facade`, and its allowed dependents, allowed dependencies and
   forbidden edges in `[dependencies]`.
3. This item owns the edges among the four workspace crates and the
   forbidden edges. The manifests MUST encode exactly this table, and
   [ADR-RUN-0003](architecture.md) and the crate graph in
   `docs/architecture.md` mirror it:

   | Crate | Workspace crates it depends on | Forbidden edges (`forbidden_edges`) |
   |---|---|---|
   | `sc-config` | none | `sc-transport`, `sc-command`, `sc-runtime`, `sc-observability`, `sc-observability-otlp`, `tokio` |
   | `sc-transport` | none | `sc-config`, `sc-command`, `sc-runtime`, `sc-observability`, `sc-observability-otlp` |
   | `sc-command` | none | `sc-config`, `sc-transport`, `sc-runtime`, `sc-observability`, `sc-observability-otlp` |
   | `sc-runtime` | MUST: `sc-transport` with its `server` cargo feature, `sc-command` with its `server` cargo feature. MAY: `sc-config` (see the OPEN below) | `sqlx`, `sc-observability`, `sc-observability-otlp` |

4. This item does not list the third-party crates each crate may depend on.
   Each list is owned by that crate's own requirement, and the manifest's
   `allowed_dependencies` MUST equal the row above plus that list:
   [NFR-CFG-0002](sc-config/requirements.md) for `sc-config`,
   [NFR-TRN-0001](sc-transport/requirements.md) for `sc-transport`,
   [NFR-CMD-0001](sc-command/requirements.md) for `sc-command`, and
   [NFR-RT-0003](sc-runtime/requirements.md) for `sc-runtime`.
5. The manifest format has no key for cargo features, so `sc-lint-boundary`
   enforces only the inter-crate edges and the public facade list. The three
   kinds of rule are enforced as follows:

   | Rule | Enforced by |
   |---|---|
   | inter-crate edges and forbidden edges; public facade list | `sc-lint-boundary`, run by `just lint` |
   | dependencies allowed only behind the `server` cargo feature (`axum` in `sc-transport`; `axum` and `rmcp` in `sc-command`) | the `cargo tree` criteria of [NFR-TRN-0001](sc-transport/requirements.md), [NFR-CMD-0001](sc-command/requirements.md) and [NFR-RUN-0001](requirements.md) |
   | `sc-runtime` wraps no `axum` or `rmcp` type ([NFR-RT-0001](sc-runtime/requirements.md)) | architecture review of the pull request |

6. The `just lint` recipe in the root `justfile` MUST run `sc-lint-boundary`
   against these manifests, and `just lint` MUST fail when a manifest is
   violated.

**OPEN:** the manifest file name(s) inside `boundaries/<crate>/` are not
stated here; `sc-lint-boundary` defines them.

**OPEN:** the sc-runtime design lists `sc-config` as a dependency of
`sc-runtime` but names no use for it, and `sc-runtime` MUST NOT load
configuration (the project loads it and passes `&cfg.daemon` in). Whether
the edge `sc-runtime -> sc-config` exists in v0.1 is therefore not decided;
the manifest allows it and does not require it.

### Rationale

Two properties of the crate graph are the ones most likely to erode without
anyone noticing: there is no dependency edge among `sc-config`,
`sc-transport` and `sc-command`, and nothing server-side (`axum`, `rmcp`,
`sqlx`) is reachable from a CLI build. One convenient `use` breaks either, so
they are checked mechanically and not left to review: the first by
`sc-lint-boundary` on every lint run, the second by `cargo tree` criteria,
because the manifest format cannot express a cargo feature. Keeping the edge
table in one item stops the copies in the ADR and in the crate files from
drifting apart. The manifests also tell sprint planning which crates can be
built in parallel without touching each other.

### Success Criteria

1. `boundaries/sc-config/`, `boundaries/sc-transport/`,
   `boundaries/sc-command/` and `boundaries/sc-runtime/` each exist and
   contain at least one `.toml` file.
2. Each manifest's `forbidden_edges` holds one `"<crate> -> <name>"` string
   for every name in the crate's "Forbidden edges" cell above, and no
   manifest of `sc-config`, `sc-transport` or `sc-command` lists another
   workspace crate under `allowed_dependencies`.
3. `just lint` invokes `sc-lint-boundary` (inspect the recipe) and exits 0 on
   the default branch.
4. Negative check: adding `sc-transport` as a dependency of `sc-command` in a
   scratch branch makes `just lint` exit non-zero with a boundary violation
   naming that edge.
5. For each of the four crates,
   `cargo tree -p <crate> -e normal --prefix none` prints no line beginning
   with `<name> ` (the name followed by a space) for any `<name>` in that
   crate's "Forbidden edges" cell. For `sc-transport` and `sc-command` the
   same holds with `--features server` added.
6. The table in obligation 3, the crate dependency table of
   [ADR-RUN-0003](architecture.md) and the crate graph section of
   `docs/architecture.md` name the same workspace edges and the same
   forbidden edges.

---

## REQ-RUN-0101: Spike `examples/spike` proves the stack coexists

**Status:** Active  

### Requirement Statement

`examples/spike` in the `randlee/sc-runtime` repository is a throwaway proof,
built before any of the four library crates exists, so it uses the third-party
crates directly.

1. The spike MUST contain one server binary that, in a single process and on
   a single `axum` 0.8 `Router`, combines all of:
   - REST routes registered through `utoipa-axum` 0.2 `OpenApiRouter` with
     the `routes!` macro;
   - an `rmcp` 3.x `StreamableHttpService`, configured in stateless mode and
     mounted with `Router::nest_service("/mcp", service)`;
   - `sqlx` 0.9 with the `sqlite` feature, opening a SQLite database;
   - a Unix domain socket listener (`tokio::net::UnixListener`) passed to
     `axum::serve`.
2. The spike MUST contain exactly one async service function that reads or
   writes the SQLite database. The REST handler and the MCP tool MUST both
   call that function and MUST NOT contain SQL themselves.
3. The spike MUST contain a `clap` command-line client that connects to the
   server's socket using `reqwest` 0.13 `ClientBuilder::unix_socket(path)`
   and calls the REST route. It MUST NOT use `hyperlocal` or a custom
   connector.
4. Three clients MUST each reach that one service function against one
   running server process: `curl --unix-socket <path>` calling the REST
   route, the `clap` client, and an MCP client issuing `tools/call` on
   `/mcp`.

**OPEN:** what the spike's one operation does, its route path and its MCP
tool name are not stated; any single operation that touches SQLite satisfies
this requirement.

**OPEN:** which MCP client is used for the proof is not stated; any client
that speaks MCP Streamable HTTP over the Unix socket qualifies.

### Rationale

The whole architecture assumes that `axum`, `utoipa-axum`, `rmcp` and `sqlx`
share one router, one listener and one process, and that a CLI can reach that
listener over a Unix socket with stock `reqwest`. If any of that is false the
crate split and the template are wrong. Half a day of spike proves it before
four crates and a template are built on the assumption.

### Success Criteria

1. `cargo build` of `examples/spike` exits 0 with `axum`, `utoipa-axum`,
   `rmcp`, `sqlx` and `reqwest` in one dependency graph; the output of
   `cargo tree -i axum` for the spike package is recorded and shows exactly
   one `axum` version.
2. With the spike server running, `curl --unix-socket <socket> ...` on the
   REST route returns HTTP 200 and the operation's result.
3. With the same server process, the `clap` client returns the same
   operation's result and exits 0.
4. With the same server process, an MCP `tools/call` request to `/mcp`
   returns the same operation's result.
5. Inspect the spike source: one service function contains the only SQL; the
   REST handler and the MCP tool each call it.
6. The commands run for criteria 2 to 4 and their output are recorded in
   `docs/architecture.md` together with the evidence required by
   [REQ-RUN-0102](requirements.md).

---

## REQ-RUN-0102: Spike answers the four unverified facts and pins versions

**Status:** Active  

### Requirement Statement

The sc-runtime design relies on four facts it has not verified. The spike
sprint, named sprint aa-1 (the work in `examples/spike` plus a scratch
`cargo-generate` template), MUST answer each one with recorded evidence, as
true, or as false together with what was found instead.

This table is the only definition of that evidence. The two Proposed ADRs
named below refer to its rows by id. Any text beside a row id in those ADRs is
a non-normative summary; the rows of this item govern.

| Row | Fact to verify | Evidence required |
|---|---|---|
| V1 | `rmcp` 3.x, `utoipa-axum` 0.2 and `axum` 0.8 coexist on one `axum::Router` with no dependency version conflict. | `examples/spike` compiles with all three in one binary; `cargo tree -i axum` run on the spike package shows exactly one `axum` version; with the spike running, `curl --unix-socket`, the `clap` client and an MCP client each reach the one service function through that one router ([REQ-RUN-0101](requirements.md)). |
| V2 | `rmcp`'s `Parameters<T>` accepts a struct `T` that derives both `schemars::JsonSchema` and `utoipa::ToSchema`, with no clash between the two derives or their schemas. | The spike's request struct derives `Serialize`, `Deserialize`, `schemars::JsonSchema` and `utoipa::ToSchema`; it is used as an `axum` `Json<T>` body and as `Parameters<T>` in a `#[tool]`; it compiles and both calls answer; MCP `tools/list` returns an input schema for that tool, and `openapi.json` contains the same struct as a component schema. |
| V3a | The `cargo-generate` version pinned by sprint aa-1 accepts, in `cargo-generate.toml`, placeholders of type `string`, `bool` and `array`, a conditional `ignore` list keyed on a placeholder value, and an `exclude` list; the exact syntax of each is known. | A scratch `cargo-generate.toml` using every one of these constructs, copied into the record, that the pinned version parses without error. |
| V3b | A conditional `ignore` list includes or excludes a whole file. | Two runs of the scratch template that differ only in one `bool` value: the conditionally ignored file is present in one output and absent in the other. |
| V4a | `cargo generate --path <template> --template-values-file <file> --name <n> --silent` runs with no prompt, including when a value is of type `array`. | The command line used; run with stdin closed it exits 0 and prints no prompt, and the rendered output contains the `string`, `bool` and `array` values from the values file. |
| V4b | A file listed under `exclude` (for example `AGENTS.md.j2`) is copied byte-for-byte without Liquid rendering. | A `.j2` file containing `{{ }}` expressions is byte-identical in the template and in the generated output (`cmp` exits 0). |

A fifth fact once listed for verification, how to dump MCP `tools/list` for
surface snapshots, MUST be recorded as closed without investigation, because
snapshot tooling is not part of this repository.

The spike sprint MUST also record the exact version used for each of `axum`,
`utoipa`, `utoipa-axum`, `rmcp`, `sqlx`, `reqwest` and `cargo-generate`.

All evidence and version pins MUST be written into `docs/architecture.md`.

Only two ADRs are `Proposed`, and each holds nothing but unverified facts:

| ADR | Holds | Rests on rows |
|---|---|---|
| [ADR-RUN-0203](architecture.md) | that the pinned `rmcp`, `utoipa-axum` and `axum` versions coexist, and that one struct is both the `Parameters<T>` input and a `ToSchema` type; the version pins | V1, V2 |
| [ADR-RUN-0402](architecture.md) | the `cargo-generate` mechanics: conditional `ignore`, `exclude`, the non-interactive values file, the `cargo-generate.toml` syntax and the `cargo-generate` version pin | V3a, V3b, V4a, V4b |

Both MUST be moved from `Proposed` to `Active` when the spike sprint closes.
Where the spike finds a fact false, that ADR MUST be amended to what was
found before it is made `Active`.

The shape decisions do not wait for the spike and are already `Active`:
[ADR-RUN-0202](architecture.md) (REST, OpenAPI and MCP on one `axum` router
and one listener) and [ADR-RUN-0401](architecture.md) (the generation
pipeline and the `answers.schema.json` contract). If V1 or V2 is false, the
pinned versions change, not the one-router shape.

Decided in this document: the sc-runtime design's Phase 0 exit gate "ADR
amendments written and accepted" is met by
[ADR-RUN-0001](architecture.md) (which amends SC scaffold ADR-013, the
single assembly point and command registry) and
[ADR-RUN-0201](architecture.md) (which carries SC scaffold ADR-014, the
daemon is always running) having Status `Active` in `docs/architecture.md`.
ADRs live in the architecture files; the design's `docs/adr/` directory is
not used.

### Rationale

These facts fix the dependency versions every crate uses and the template
syntax every template file uses. Planning later sprints around a guess would
turn each wrong guess into rework across four crates and the template, so
nothing may be planned around them as facts until the spike has answered
them.

### Success Criteria

1. `docs/architecture.md` contains, for each of the six rows V1, V2, V3a,
   V3b, V4a and V4b, the answer (true, or false with the finding) and the
   evidence named in that row.
2. `docs/architecture.md` records that the MCP `tools/list` dump question is
   closed as not needed.
3. `docs/architecture.md` lists an exact version (`major.minor.patch`) for
   each of `axum`, `utoipa`, `utoipa-axum`, `rmcp`, `sqlx`, `reqwest` and
   `cargo-generate`.
4. When the spike sprint closes, the `**Status:**` line of ADR-RUN-0203 and
   of ADR-RUN-0402 in `docs/architecture.md` reads `Active`, and
   `grep -n '^\*\*Status:\*\* Proposed' docs/architecture.md` prints
   nothing.
5. For each row answered false, the Decision text of the ADR that rests on
   it (see the table of the two Proposed ADRs) has been changed to match the
   finding.
6. The `**Status:**` lines of ADR-RUN-0001 and ADR-RUN-0201 in
   `docs/architecture.md` read `Active`.
7. The evidence tables of ADR-RUN-0203 and ADR-RUN-0402 cite rows of this
   item by id (V1 to V4b); any text beside the id states nothing the row
   does not, and is marked as a non-normative summary.

---

## REQ-RUN-0103: `examples/spike` is rewritten once, then deleted

**Status:** Active  

### Requirement Statement

`examples/spike` in the `randlee/sc-runtime` repository is first written
directly on `axum`, `utoipa-axum`, `rmcp`, `sqlx` and `reqwest`, before the
four library crates exist.

1. Once `sc-config`, `sc-transport`, `sc-command` and `sc-runtime` exist, the
   spike MUST be rewritten to use them: its server is assembled with
   `sc_runtime::Daemon::builder()` and loads its configuration with
   `sc-config`, and its `clap` client connects with `sc_transport::Client`
   and parses responses as `sc_command::Envelope<T>`. The `Client` type and
   method names are illustrative until the contract sprint pins them
   ([REQ-TRN-0005](sc-transport/requirements.md)).
2. The rewritten spike's server `main.rs` MUST be under 30 lines; that
   measurement is [REQ-RUN-0204](requirements.md).
3. The directory `examples/spike` MUST be deleted before the `v0.1.0` tag is
   created. The tree at the `v0.1.0` tag MUST NOT contain it. This item owns
   the deletion rule; the release requirement refers to it.
4. After deletion, none of the root `Cargo.toml`, the root `justfile`, the
   files under `.github/workflows/` and the root `README.md` may refer to
   `examples/spike`. Files under `docs/` may still name it, because the
   spike evidence of [REQ-RUN-0102](requirements.md) is recorded there.

### Rationale

The spike is evidence that the stack works and that the crates make assembly
small; it is not a product. The template's widget example is the one example
projects are meant to copy. Leaving the spike in a release would give users a
second example that nobody maintains.

### Success Criteria

1. Before deletion, `examples/spike`'s `Cargo.toml` depends on `sc-runtime`,
   `sc-config`, `sc-transport` and `sc-command`, and its server source calls
   `Daemon::builder`.
2. `git ls-tree -r --name-only v0.1.0 | grep '^examples/spike/'` prints
   nothing.
3. At the `v0.1.0` tag, `grep -rn 'examples/spike' Cargo.toml justfile
   .github README.md` prints nothing.
4. At the `v0.1.0` tag, `cargo build --workspace` and `just test` exit 0,
   showing nothing depended on the spike.

---

## REQ-RUN-0201: One operation, three surfaces, one service function

**Status:** Active  

### Requirement Statement

This is a property of a project generated from `template/`, demonstrated by
the template's example operation `widget.create`.

1. An operation MUST be defined once, as a request struct and a response
   struct in the generated `api-types` crate plus one async service function
   in the generated `service` crate. For the example:

   ```rust
   // crates/api-types
   #[derive(Serialize, Deserialize, JsonSchema, ToSchema)]
   pub struct CreateWidget { pub name: String }

   // crates/service
   pub async fn create_widget(s: &Stores, input: CreateWidget)
       -> Result<Widget, OpError>;
   ```

2. The operation MUST be reachable through three surfaces, and each surface
   MUST do nothing but convert its input into the `api-types` request struct,
   call the service function, and convert the result:

   | Surface | Where | How it reaches the service function |
   |---|---|---|
   | REST | `crates/daemon/src/routes.rs`, `POST /ops/widget.create` | `axum` handler takes `Json<CreateWidget>`, calls `service::create_widget`, returns `Envelope<Widget>` through `.into()` |
   | MCP | `crates/daemon/src/mcp.rs`, tool `widget_create` | `rmcp` `#[tool]` method takes `Parameters<CreateWidget>`, calls `service::create_widget`, returns through `.into_mcp()` |
   | CLI | `crates/cli` | `clap` command builds a `CreateWidget`, sends it with `sc_transport::Client::post("/ops/widget.create", &input)`; the daemon's REST handler then calls the service function |

   The `sc_transport::Client` method names are illustrative until the
   contract sprint pins them ([REQ-TRN-0005](sc-transport/requirements.md)).
   `OpError` is defined in `sc-command`, so the `service` crate uses
   `sc-command`; `Stores` is defined in the `service` crate. Both facts, and
   the rest of the generated crate graph, are owned by
   [REQ-RUN-0301](requirements.md).

   Which derives the struct carries, and that the same struct is the rmcp
   `Parameters<T>` input, is conditional on
   [ADR-RUN-0203](architecture.md) (spike row V2 of
   [REQ-RUN-0102](requirements.md)), which stays Proposed until the spike
   proves it. If V2 is false, these clauses are amended in the same change to
   what the spike found. The rule that every surface calls one service
   function does not depend on it.

3. REST handlers, MCP tools and CLI commands MUST NOT contain SQL, MUST NOT
   call `sqlx`, and MUST NOT call a store crate directly.
4. `sqlx` MUST be a dependency of the generated `store-*` crates only. The
   `service` crate reaches SQL only by calling the typed query functions
   that a store crate exports.
5. A project MAY give REST and MCP different JSON shapes at the edge, provided
   both are converted to the same service-function input type before the
   service function is called. The template example uses one struct for all
   three surfaces. Nothing in the `sc-runtime` crate enforces either choice.

### Rationale

The failure this architecture exists to prevent is the three surfaces
drifting apart: a CLI command that behaves differently from the REST route of
the same name, or an MCP tool that grows its own query. With one service
function as the only path to SQL, a fix or a validation rule is written once
and every client gets it.

### Success Criteria

1. A test in the generated project starts one daemon with
   `sc_runtime::testing::DaemonFixture` and creates three widgets with
   distinct names: one by `POST /ops/widget.create`, one by an MCP
   `tools/call` of `widget_create` sent to `/mcp`, and one by running the
   generated CLI binary against the fixture's endpoint. The fixture exposes
   its endpoint as a string in the form `--endpoint` and `SC_ENDPOINT`
   accept ([REQ-RT-0006](sc-runtime/requirements.md)); the test passes that
   string to the CLI with `--endpoint` or `SC_ENDPOINT`
   ([REQ-RUN-0310](requirements.md)). It then reads all three back with
   `widget.get` and asserts each is returned with the name it was created
   with.
2. `cargo metadata` in the generated project shows `sqlx` as a direct
   dependency of `store-*` packages only.
3. `grep -rn 'sqlx' crates/api-types crates/service crates/daemon crates/cli`
   in the generated project prints nothing.
4. Inspect `crates/daemon/src/routes.rs` and `crates/daemon/src/mcp.rs`:
   each handler or tool body is a call to a `service::` function plus a
   conversion, with no other logic.

---

## REQ-RUN-0202: CLI reports `DAEMON.NOT_RUNNING`; no fallback

**Status:** Active  

### Requirement Statement

This applies to the `cli` crate of a project generated from `template/`.
`<app>` below is the project's application name.

1. The CLI MUST reach data only by sending HTTP requests to the daemon
   through `sc_transport::Client`. It MUST NOT open the database. The `cli`
   crate MUST NOT have `sqlx` or any generated `store-*` or `service` crate
   among its normal (non-dev) dependencies; the rule that a CLI links no
   daemon-side dependency is owned by [NFR-RUN-0001](requirements.md).
2. The CLI MUST NOT start the daemon, neither automatically nor as a retry.
3. When the daemon cannot be reached (the socket file does not exist, or the
   connection is refused), the `sc-transport` client returns the typed error
   `TransportError::DaemonNotRunning`
   ([REQ-TRN-0006](sc-transport/requirements.md)). Whether `connect` or the
   first `get` or `post` call returns it is OPEN in
   [REQ-TRN-0005](sc-transport/requirements.md), and the client method names
   are illustrative until pinned there. The CLI MUST map that error to an
   `OpError` with `code` equal to `"DAEMON.NOT_RUNNING"` and
   `suggested_action` equal to `"run <app> daemon start"`, with `<app>`
   replaced by the application name. `suggested_action` MUST serialise as a
   JSON string; its Rust type is OPEN in
   [REQ-CMD-0002](sc-command/requirements.md), constrained to serialise as
   a string.
4. In that case the CLI MUST exit with a non-zero status.
5. In that case, when the command was given `--json`, the CLI MUST print to
   stdout one envelope `{version, ok, data, error}` with `ok` equal to
   `false`, `data` equal to `null`, and `error` equal to the `OpError` of
   obligation 3.
6. Decided in this document: any other transport failure (a timeout, a
   response body that cannot be decoded) MUST NOT be reported as
   `DAEMON.NOT_RUNNING`. A response that arrives with a non-2xx HTTP status
   proves a daemon is running, so it MUST NOT be reported as
   `DAEMON.NOT_RUNNING` either; how the `sc-transport` client presents a
   non-2xx response is OPEN in
   [REQ-TRN-0006](sc-transport/requirements.md).

**OPEN:** the `code` and the envelope the CLI emits for a transport failure
that is not `DAEMON.NOT_RUNNING` are not stated.

**OPEN:** the values of the `OpError` fields `kind` and `message` for this
error are not stated.

**OPEN:** the exact non-zero exit status, and the format and stream (stdout
or stderr) of the message when `--json` is not given, are not stated.

**OPEN:** the suggested action names a command `<app> daemon start`; whether
the generated CLI provides a `daemon start` subcommand is not stated. The
string is fixed either way.

**OPEN:** whether the CLI may auto-start the daemon in a later version is
undecided. Until it is decided the behaviour is report-only, and nothing may
be built that assumes otherwise.

### Rationale

The daemon is the only process allowed to open the local database, and it
holds an exclusive lock on `<instance-root>/daemon.lock` to stay the only
one. `<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](sc-transport/requirements.md)). A CLI that fell
back to opening the database itself would be a second
writer to a SQLite file and a second code path to SQL, and would behave
differently depending on whether a daemon happened to be up. Because every
CLI command therefore needs a running daemon, its absence is the most common
CLI failure, so it gets a typed code an agent can branch on and a suggested
action that says how to fix it.

### Success Criteria

1. A test runs a generated CLI command with `--json` against an endpoint
   where no daemon is listening (a socket path inside a fresh tempdir). It
   asserts: exit status is non-zero; stdout parses as JSON; `ok` is `false`;
   `data` is `null`; `error.code` is `"DAEMON.NOT_RUNNING"`;
   `error.suggested_action` is `"run <app> daemon start"` with the
   application name substituted.
2. The same test uses the fresh tempdir as the instance root and asserts
   that after the command the tempdir contains no `daemon.lock`, no
   `daemon.sock` and no database file, and that connecting to the endpoint
   still fails.
3. `cargo tree -p cli -e normal --prefix none` in the generated project
   prints no line beginning with `sqlx `, `service ` or `store-sqlite ` (the
   name followed by a space).
4. A test that points the CLI at a listener answering HTTP 500 asserts that
   `error.code` is not `"DAEMON.NOT_RUNNING"`.

---

## REQ-RUN-0203: REST, MCP and CLI `--json` return the same envelope

**Status:** Active  

### Requirement Statement

This is a property of a project generated from `template/`.

1. Every operation response, on every surface, MUST be the sc-ai-cli envelope:
   a JSON object with exactly the four keys `version`, `ok`, `data` and
   `error`, implemented by the type `Envelope<T>` in the `sc-command` crate
   ([REQ-CMD-0001](sc-command/requirements.md)).
2. On success `ok` MUST be `true`, `data` MUST hold the operation's response
   struct, and `error` MUST be `null`.
3. On failure `ok` MUST be `false`, `data` MUST be `null`, and `error` MUST be
   an `OpError` object. `OpError` has the five fields `kind`, `code`,
   `message`, `details` and `suggested_action`; which keys appear in its
   JSON form is defined by [REQ-CMD-0002](sc-command/requirements.md), and
   this item requires exactly the keys defined there.
4. REST: the HTTP response body of an operation route MUST be that envelope
   as JSON.
5. MCP: the `CallToolResult` of an operation tool MUST carry that same
   envelope as its content, and MUST be flagged as an error result when `ok`
   is `false` ([REQ-CMD-0005](sc-command/requirements.md)).
6. CLI: a command given `--json` MUST print to stdout the envelope it
   received from the daemon, unchanged: parsing the printed text yields a
   JSON value equal to the REST response body, including any content inside
   `data` or `error.details` that the CLI's own types do not know about.
7. For the same operation, the same input and the same daemon state, the
   envelope obtained through REST, through MCP and through CLI `--json` MUST
   be equal as JSON values.

**OPEN:** the type and value of `version` are not stated.

**OPEN:** whether `details` and `suggested_action` are present as `null` or
omitted when they are empty is undecided; the owner of that question is
[REQ-CMD-0002](sc-command/requirements.md).

**OPEN:** obligations 1 to 7 cover responses whose content comes from a
service function. Whether a response generated by a framework is also an
envelope is not decided, and this item owns the question. The cases are: an
unknown route, a request body that `axum` fails to deserialise, and an MCP
protocol error returned as `Err(McpError)` by `rmcp`.

### Rationale

Agents are the main consumers of these CLIs and APIs. One response shape, with
an explicit `ok`, a typed `code` to branch on and a `suggested_action` to
recover with, means an agent needs one parser and one error-handling path no
matter which surface it used, and can move between surfaces without relearning
anything.

### Success Criteria

1. A test against one `DaemonFixture` daemon calls `widget.get` for the same
   existing widget through REST, through an MCP `tools/call`, and through the
   CLI with `--json`; it parses the three results as JSON and asserts all
   three are equal, with `ok` equal to `true` and `error` equal to `null`.
2. The same test repeats the three calls for a widget that does not exist and
   asserts the three envelopes are equal, with `ok` equal to `false`, `data`
   equal to `null`, and `error` containing `kind`, `code` and `message`.
   Once the present-or-omitted question of
   [REQ-CMD-0002](sc-command/requirements.md) is decided: `error` holds
   exactly the keys defined there. It also asserts the MCP result is flagged
   as an error.
3. Each envelope in criteria 1 and 2 has exactly the keys `version`, `ok`,
   `data`, `error`.
4. A test feeds the CLI a daemon response whose `data` contains a field
   unknown to the CLI's `api-types` struct and asserts the field is present
   in the `--json` output.

---

## REQ-RUN-0204: Rewritten spike `main.rs` is under 30 lines

**Status:** Active  

### Requirement Statement

`examples/spike` is first written directly on `axum`, `utoipa-axum`, `rmcp`
and `sqlx`, then rewritten on the four library crates `sc-config`,
`sc-transport`, `sc-command` and `sc-runtime`.

1. After the rewrite, the `main.rs` of the spike's server binary MUST be
   under 30 lines, that is 29 lines or fewer.
2. Lines are counted with `wc -l`: every physical line counts, including
   blank lines, comments, `use` lines and attributes. This counting rule is
   decided in this document.
3. That `main.rs` MUST itself contain the whole start-up sequence: loading
   configuration with `sc-config`, the call chain
   `sc_runtime::Daemon::builder(...)` with `.stores(...)`, `.routes(...)`,
   `.mcp(...)` and `.run().await`, and turning the result into the process
   exit code. It MUST NOT meet the limit by moving that sequence into another
   file.
4. The router function, the MCP service, the `Stores` struct and the service
   function MAY live in other files of the spike; they are project wiring,
   not assembly.

**OPEN:** the path of that `main.rs` inside `examples/spike` depends on how
the spike lays out its server and client binaries, which is not stated.

### Rationale

The product's first principle is that the framework instantiates the parts
and the project only wires them. This requirement is the measurable form of
that claim: if assembling a daemon with lock, stores, router, OpenAPI, MCP,
listener and graceful shutdown still takes a page of code, the crates have
not removed the hand-assembly they exist to remove.

### Success Criteria

1. `wc -l` on the rewritten spike server's `main.rs` prints a number less
   than 30.
2. Inspect that `main.rs`: it contains the `sc-config` load call,
   `Daemon::builder`, `.stores(`, `.routes(`, `.mcp(`, `.run()` and the exit
   code mapping.
3. The rewritten spike still passes the three-client check of
   [REQ-RUN-0101](requirements.md): `curl --unix-socket`, the `clap` client
   and an MCP client each reach the one service function.

---

## REQ-RUN-0205: One listener serves routes, OpenAPI, health and `/mcp`

**Status:** Active  

### Requirement Statement

A daemon assembled with `sc_runtime::Daemon::builder()` MUST serve all of the
following from one `axum::Router`, served unchanged on every listener `run()`
binds (a Unix domain socket, or a TCP address and port), in one process. This
item states the end-to-end property; what the `sc-runtime` crate merges and
mounts to achieve it is [REQ-RT-0004](sc-runtime/requirements.md).

| What | Source |
|---|---|
| The project's own routes (in the template: `POST /ops/widget.create` and `POST /ops/widget.get`) | the `utoipa_axum::OpenApiRouter` returned by the project's `.routes(...)` closure |
| The OpenAPI document `openapi.json` describing those routes | generated from the same `OpenApiRouter`; the project writes no separate spec |
| A health route | added by `sc-runtime` |
| The MCP endpoint at path `/mcp` | the `rmcp` `StreamableHttpService` returned by the project's `.mcp(...)` closure, mounted with `nest_service("/mcp", ...)` |

1. MCP MUST NOT run in a separate process, binary or port.
2. When the project gives no `.mcp(...)` closure, `/mcp` MUST NOT be served,
   and the other three MUST still be served.
3. A daemon MAY serve a Unix socket and a TCP listener at the same time (a
   browser frontend needs TCP while the CLI stays on the socket);
   `sc-transport` provides the listener binding for both
   ([REQ-TRN-0003](sc-transport/requirements.md)).

**OPEN:** whether `Daemon::builder()` can bind a Unix socket and a TCP
listener at once in v0.1 is not decided. Once it is decided that it can: a
test asserts one daemon answers the same route on both listeners.

**OPEN:** the URL path at which `openapi.json` is served is not stated.

**OPEN:** the URL path of the health route, and its response status and
body, are not stated.

### Rationale

A project's CLI, its browser frontend and its MCP clients all talk to the
same daemon; one listener means one endpoint to configure, one thing to
secure and one process to supervise. Serving `openapi.json` from the daemon
lets a frontend generate its own TypeScript client with whichever generator
it prefers, without this project choosing one.

### Success Criteria

1. A test starts a daemon with `sc_runtime::testing::DaemonFixture`, with an
   MCP closure. The fixture exposes its resolved endpoint
   ([REQ-RT-0006](sc-runtime/requirements.md)), which the test needs because
   the MCP request is Streamable HTTP and cannot be sent with the JSON
   `get`/`post` client of `sc-transport`. Through that single endpoint the
   test: posts to
   `/ops/widget.create` and gets an envelope back; fetches `openapi.json` and
   asserts it parses as JSON, has an `openapi` field, and its `paths` contain
   `/ops/widget.create`; requests the health route and gets a success
   status; sends an MCP `tools/list` request to `/mcp` and gets a result
   listing `widget_create`.
2. A test starts a daemon without an MCP closure and asserts a request to
   `/mcp` returns HTTP 404 while the other three still answer.
3. Inspect the generated `crates/daemon`: it starts one process and no code
   in it binds a second listener for MCP.

---

## REQ-RUN-0301: Layout of the workspace rendered from `template/`

**Status:** Active  

### Requirement Statement

`template/` in the `randlee/sc-runtime` repository MUST be a standard
`cargo-generate` template (configured by `template/cargo-generate.toml`)
that renders a Cargo workspace with this layout. "v0.1" means the first
release, in which the only database option is SQLite.

```text
<project>/
  Cargo.toml                  # workspace
  crates/
    api-types/                # request and response structs
    store-sqlite/             # pool, migrations/, queries, sqlx.toml, .sqlx/
    service/                  # async service functions
    daemon/                   # src/main.rs, src/routes.rs, src/mcp.rs
    cli/                      # clap commands
  config/
    default.json
  Justfile
  AGENTS.md
  CLAUDE.md
  .gitignore                  # lists config/local.json
  .github/workflows/ci.yml
```

1. `crates/daemon/src/mcp.rs` MUST be rendered only when the option `mcp` is
   `true`.
2. `crates/store-sqlite` MUST be rendered when the option `db` is `sqlite`
   (the only value accepted in v0.1).
3. `config/local.json` is the developer's local override file. It MUST be
   listed in the rendered `.gitignore`.
4. `AGENTS.md` and `CLAUDE.md` are produced from `template/AGENTS.md.j2` and
   `template/CLAUDE.md.j2` by `sc-compose`, not by `cargo-generate`
   ([REQ-RUN-0305](requirements.md)).
5. This item and [ADR-RUN-0303](architecture.md) own the crate graph of the
   generated workspace. The rendered crates MUST have these dependencies:

   | Crate | MUST depend on | Why it is a separate crate |
   |---|---|---|
   | `api-types` | `serde`, `schemars`, `utoipa` | shared by `daemon` and `cli`, so both compile against the same structs and no client code generator is needed |
   | `store-sqlite` | `sqlx` with the `sqlite` feature | `sqlx` 0.9 checks queries against one database per crate |
   | `service` | `api-types`, `store-sqlite`; the `sc-command` items `OpError` (default features, see below) | the single code path to SQL, called by REST handlers and MCP tools |
   | `daemon` | `service`, `api-types`, `sc-runtime`, `sc-config`; the `sc-command` items `Envelope<T>` and `into_mcp()`, which need its `server` cargo feature (see below) | owns the wiring: which routes exist and what they call |
   | `cli` | `api-types`, `sc-transport`, `sc-command`, `sc-config` | thin HTTP client |

   The sc-runtime design's dependency table omits three of these edges. They
   follow from its own code: service functions return `OpError`, which is
   defined in `sc-command`; `daemon` handlers name `Envelope<T>`, call
   `into_mcp()`, and take `api-types` structs as input.
6. The project's `Stores` struct MUST be defined in the `service` crate.
   This follows from two rules: service functions take `&Stores`, and only
   `service` may depend on a store crate.
7. These dependency edges MUST NOT exist:
   - `cli` on `service`, `store-sqlite`, `daemon`, `sc-runtime`, `sqlx`,
     `axum` or `rmcp` as a normal (non-dev) dependency, directly or
     transitively. The rule that a CLI links no daemon-side dependency, and
     the build it is defined for, are owned by
     [NFR-RUN-0001](requirements.md);
   - `api-types` on any other crate of the generated workspace;
   - any crate other than a `store-*` crate on `sqlx` directly;
   - any crate other than `service` on a `store-*` crate;
   - `store-sqlite` on `service`, `daemon` or `cli`;
   - any crate other than `daemon` on `sc-runtime` as a normal (non-dev)
     dependency.
8. `cli` MUST use `sc-transport` and `sc-command` without their `server`
   cargo feature.
9. TCP ports come from project configuration. Port ranges are allocated per
   project in the synaptic-canvas port registry. No file under `template/`
   and none of the four library crates may hard-code a TCP port.

`service` MUST take `sc-command` (default features) as a direct dependency:
by rule 7 it may not have a dependency on `sc-runtime`, so a re-export from
`sc-runtime` cannot reach it.

**OPEN:** whether `daemon` takes `sc-command` as a direct dependency with
`features = ["server"]`, or reaches it through a re-export from `sc-runtime`,
is not decided. A re-export would have to be added to the public surface of
`sc-runtime`, which [NFR-RT-0001](sc-runtime/requirements.md) limits.

**OPEN:** which generated crate defines the project's config struct
`AppConfig`, and whether `cli` and `daemon` load the same type, is not
decided. The constraints are: `cli` cannot take it from `daemon` or
`service` (forbidden edges above), and `api-types` may depend only on
`serde`, `schemars` and `utoipa`.

**OPEN:** whether v0.1 renders a default TCP port into
`config/default.json` is not decided. If it does, the port MUST come from
the range registered for the project in the synaptic-canvas port registry.

**OPEN:** whether the template renders a `config/local.json` file, or only
the `.gitignore` entry for one, is not stated. The keys of
`config/default.json` are not stated either.

### Rationale

The crate boundaries exist for mechanical reasons, not style. `sqlx` requires
one crate per database for its compile-time query checks. The CLI must not
link the daemon's dependencies, which forces `cli` and `daemon` apart and
puts the structs they share into `api-types`. `service` is separate so that
REST and MCP have exactly one place to call.

### Success Criteria

1. For each answers file in `wizard/fixtures/`, render the project with the
   driver (`just new <dest> --var-file <fixture>`) and assert
   every path in the layout above exists, with `crates/daemon/src/mcp.rs`
   present exactly when the fixture's `mcp` is `true`.
2. In each rendered project, `cargo metadata --format-version 1` shows the
   workspace members `api-types`, `store-sqlite`, `service`, `daemon`, `cli`
   and no others, and every "MUST depend on" edge of the table other than
   the `sc-command` edge of `daemon`; this includes the direct edge from
   `service` to `sc-command`. Once the direct dependency or re-export
   question for `daemon` is decided: it shows that edge in the decided
   form.
3. In each rendered project,
   `cargo tree -p cli -e normal --prefix none` prints no line beginning with
   `sqlx `, `axum `, `rmcp `, `sc-runtime `, `service `, `store-sqlite ` or
   `daemon ` (the name followed by a space).
4. In each rendered project, `sqlx` appears under `[dependencies]` only in
   `crates/store-sqlite/Cargo.toml`, and `store-sqlite` appears under
   `[dependencies]` only in `crates/service/Cargo.toml`.
5. The rendered `.gitignore` contains the line `config/local.json`.
6. The rendered project contains no file ending in `.j2` and no file from
   `wizard/` or `scripts/`.
7. `grep -rn 'struct Stores' crates/` in a rendered project matches only
   under `crates/service/`.
8. Inspect `template/` and the non-test source of the four library crates
   in the `randlee/sc-runtime` repository: no numeric TCP port literal
   appears (once the default-port question above is decided in favour of a
   default, the one registered value in `template/config/default.json` is
   exempt).

---

## REQ-RUN-0302: Template example operations `widget.create`, `widget.get`

**Status:** Active  

### Requirement Statement

The project rendered from `template/` MUST contain exactly one worked
example, made of two operations named `widget.create` and `widget.get`, each
wired through all three surfaces:

| Piece | Where in the generated project | What it MUST contain |
|---|---|---|
| Types | `crates/api-types` | `pub struct CreateWidget { pub name: String }` and a response struct `Widget`, each deriving `Serialize`, `Deserialize`, `schemars::JsonSchema` and `utoipa::ToSchema`; plus the request struct for `widget.get` |
| Storage | `crates/store-sqlite` | a migration under `migrations/` creating the widget table, and typed query functions to insert and to fetch a widget |
| Service | `crates/service` | `pub async fn create_widget(s: &Stores, input: CreateWidget) -> Result<Widget, OpError>` and the matching function for `widget.get` |
| REST | `crates/daemon/src/routes.rs` | `POST /ops/widget.create` and `POST /ops/widget.get`, each an `axum` handler annotated with `#[utoipa::path(...)]`, registered with `utoipa_axum` `routes!` on an `OpenApiRouter`, returning `Envelope<Widget>` |
| MCP | `crates/daemon/src/mcp.rs` (only when option `mcp` is `true`) | `rmcp` `#[tool]` methods named `widget_create` and `widget_get`, taking `Parameters<...>` of the same `api-types` request structs and returning through `.into_mcp()`. The service returned by `mcp::service` MUST be an `rmcp` `StreamableHttpService` configured in stateless mode, and the template MUST NOT contain an MCP session store |
| CLI | `crates/cli` | one `clap` command per operation; each builds the `api-types` request struct, sends it with `sc_transport::Client::post` (method name illustrative until pinned by [REQ-TRN-0005](sc-transport/requirements.md)) to the matching `/ops/...` path, and supports `--json` and the global `--endpoint` option ([REQ-RUN-0310](requirements.md)) |
| Test | the generated project's test suite | at least one test that starts a daemon with `sc_runtime::testing::DaemonFixture` and exercises the example through REST, through MCP (when `mcp` is `true`) and through the CLI |

Which derives the struct carries, and that the same struct is the rmcp
`Parameters<T>` input, is conditional on
[ADR-RUN-0203](architecture.md) (spike row V2 of
[REQ-RUN-0102](requirements.md)), which stays Proposed until the spike
proves it. If V2 is false, these clauses are amended in the same change to
what the spike found. The rule that every surface calls one service
function does not depend on it.

1. Every handler, tool and command MUST call the service function and
   contain no SQL ([REQ-RUN-0201](requirements.md) states this rule in full:
   one service function is the only path to SQL).
2. The route shape `POST /ops/{name}` is the example's convention only.
   Nothing in the `sc-runtime` crate may depend on it.
3. The template MUST NOT contain a second example operation.

**OPEN:** which generated crate or directory hosts the example test, and how
that test obtains the built `cli` binary, is not decided. If it lives in
`cli`, it needs `sc-runtime` as a dev-dependency, which is why the CLI
dependency rules count normal dependencies only
([NFR-RUN-0001](requirements.md)).

**OPEN:** the name of the example test function is not decided.

**OPEN:** the fields of `Widget`, the name and fields of the `widget.get`
request struct, the name of the `widget.get` service function, the widget
table's columns, and the spelling of the two CLI commands and their
arguments are not stated.

### Rationale

This example is the pattern a project copies for its first real operation and
then deletes. An example that skipped a surface, or reached SQL some other
way, would teach the wrong pattern to every project. Keeping it to one
example keeps the template thin and leaves little to delete.

### Success Criteria

1. In a project generated from `wizard/fixtures/sqlite-mcp.json`, `just test`
   exits 0. Inspect the example test's source: it starts a `DaemonFixture`
   and calls `widget.create` and `widget.get` through REST, MCP and the CLI.
   Once the test's name is decided: the `just test` output contains
   `<name> ... ok`.
2. In a project generated with `mcp` equal to `false`, `just test` exits 0,
   `crates/daemon/src/mcp.rs` does not exist, and the example test covers
   REST and the CLI.
3. The generated daemon's `openapi.json` lists the paths `/ops/widget.create`
   and `/ops/widget.get`, and with `mcp` on an MCP `tools/list` returns
   exactly `widget_create` and `widget_get`.
4. `grep -rn 'ops/' crates/` in the `randlee/sc-runtime` repository (the
   four library crates) finds no match outside tests and documentation.
5. In a project generated with `mcp` equal to `true`, a test sends an MCP
   `tools/call` of `widget_create` to `/mcp` with no prior `initialize`
   request and no session header, and it succeeds.
6. Inspect `template/`: it contains no MCP session manager and no session
   store, and `mcp::service` builds the `StreamableHttpService` in stateless
   mode.

---

## REQ-RUN-0303: Template options in v0.1: project name, `db`, `mcp`

**Status:** Active  

### Requirement Statement

"v0.1" is the first release of the template. The generation options are
defined in `wizard/answers.schema.json` and mirrored as placeholders in
`template/cargo-generate.toml`.

1. The list of v0.1 options (project name, a string; `db`, a string enum of
   `sqlite`, `postgres`, `both`; `mcp`, a boolean), their types and the rule
   that the list is closed are owned by [REQ-RUN-0401](requirements.md).
   This item owns what each option does to the rendered project:

   | Option | Effect on the rendered project |
   |---|---|
   | project name | names the generated project |
   | `db` | selects which `store-*` crates are rendered |
   | `mcp` | `true` renders `crates/daemon/src/mcp.rs`, the `mod mcp;` line and the `.mcp(...)` line in `crates/daemon/src/main.rs`; `false` omits all three |

2. In v0.1 the only accepted value of `db` is `sqlite`. The values `postgres`
   and `both` MUST be present in the schema's enum, so the published schema
   does not change shape when they become available, and an answers file
   using either MUST fail the validation step of the driver
   `scripts/new_project.py`, before `cargo generate` runs, until the
   `store-postgres` crate ships.
   Reserving the two values this way is decided in this document.
3. An option MUST include or exclude whole files through a conditional
   `ignore` list in `template/cargo-generate.toml`. With `mcp` equal to
   `false`, `crates/daemon/src/mcp.rs` is ignored. The `ignore` mechanism
   and its syntax are conditional on [ADR-RUN-0402](architecture.md), which
   stays Proposed until the spike has verified them.
4. Differences inside a file MUST stay within the limit set by
   [REQ-RUN-0304](requirements.md): fewer than ten conditional lines across
   all template `.rs` files.
5. That the schema's keys and the placeholders of
   `template/cargo-generate.toml` always match, so a new option is added to
   both together, is owned by [REQ-RUN-0402](requirements.md) (the key-set
   test).

**OPEN:** the key name of the project-name option, and whether it is a
template placeholder or only `cargo generate`'s `--name` argument, are not
decided ([REQ-RUN-0402](requirements.md) owns the question).

**OPEN:** the defaults of `db` and `mcp` are not decided.

**OPEN:** the mechanism that rejects `db` equal to `postgres` or `both` in
v0.1 is not decided: the schema lists both values, so the rejection must be
either an additional schema constraint or a check in the driver. A check in
the driver would be option knowledge held outside the schema, which
[REQ-RUN-0401](requirements.md) forbids (its criterion that
`scripts/new_project.py` holds no option list of its own); choosing that
branch requires relaxing that criterion for this one check. It also decides
whether such answers files are schema-invalid or schema-valid but rejected
([REQ-RUN-0403](requirements.md) classes the invalid fixtures). What plain
`cargo generate` (without the driver) does when given those values is not
decided either.

**OPEN:** the file names of the `mcp`-off fixture and of the invalid
`db = postgres` answers file are not decided; only
`wizard/fixtures/sqlite-mcp.json` is named.

### Rationale

Database backends arrive in the order SQLite, then Postgres, then MySQL, and
v0.1 ships SQLite only. The answers schema is a published contract that the
sc-lint project also reads, so its shape should not change when Postgres
arrives; reserving the enum values now keeps it stable. Varying whole files,
not lines within files, keeps every template source file readable and
compilable as Rust.

### Success Criteria

1. `template/cargo-generate.toml` defines a placeholder named `db` and a
   placeholder named `mcp`. Once the project-name question above is decided:
   it defines, or does not define, a project-name placeholder as decided.
   (The schema side is checked by [REQ-RUN-0401](requirements.md).)
2. Generating from `wizard/fixtures/sqlite-mcp.json` (`db` `sqlite`, `mcp`
   `true`) yields a project containing `crates/daemon/src/mcp.rs` in which
   `just lint` and `just test` exit 0.
3. Generating from the fixture with `db` `sqlite` and `mcp` `false` yields a
   project without `crates/daemon/src/mcp.rs`, whose
   `crates/daemon/src/main.rs` contains no `.mcp(` call, in which `just lint`
   and `just test` exit 0.
4. A unit test runs the driver with an answers file whose `db` is `postgres`
   and another whose `db` is `both`; each run exits non-zero, does not invoke
   `cargo generate`, and creates no destination directory.
5. `template/cargo-generate.toml` contains a conditional `ignore` entry for
   `crates/daemon/src/mcp.rs` keyed on `mcp`.

---

## REQ-RUN-0304: Fewer than ten Liquid conditional lines in template Rust

**Status:** Active  

### Requirement Statement

`cargo-generate` renders files under `template/` with the Liquid template
language. Liquid control tags are written `{% ... %}` (for example
`{% if mcp %}` and `{% endif %}`); value substitutions are written
`{{ ... }}`.

1. Across all `.rs` files under `template/`, the number of conditional lines
   MUST be fewer than ten in total. A conditional line is a line that
   contains a Liquid control tag, or a line that lies between an opening
   control tag and its closing tag. This counting rule is decided in this
   document.
2. Liquid control tags in template `.rs` files are permitted in exactly
   these three locations and nowhere else:
   - around fields of the project's `Stores` struct in the `service` crate
     (they depend on option `db`);
   - around the `mod mcp;` line of `crates/daemon/src/main.rs` (it depends
     on option `mcp`, because `mcp.rs` is omitted when `mcp` is `false`);
   - around the `.mcp(...)` line of the `Daemon::builder()` chain in
     `crates/daemon/src/main.rs` (it depends on option `mcp`).
3. Every other difference between generated variants MUST be made by
   including or excluding a whole file through a conditional `ignore` list
   in `template/cargo-generate.toml` (for example
   `crates/daemon/src/mcp.rs`). The `ignore` mechanism is conditional on
   [ADR-RUN-0402](architecture.md), which stays Proposed until the spike has
   verified it.
4. Liquid control tags MUST NOT be used to vary the body of a function, a
   handler, a query or a test in a `.rs` file.
5. Value substitutions `{{ ... }}`, such as the project name, are not
   conditional lines and are not limited by this requirement.

### Rationale

Rust source threaded with template conditionals cannot be compiled, formatted,
linted or read as Rust while it sits in `template/`, and every conditional
creates a variant that no test may be exercising. Whole-file inclusion keeps
each template `.rs` file valid Rust and makes the set of variants equal to
the set of fixtures CI already generates and tests.

### Success Criteria

1. A unit test under `tests/unit/` (or a `just lint` step) scans
   `template/**/*.rs`, counts conditional lines as defined in obligation 1,
   and fails when the total is ten or more.
2. For every match of `grep -rn '{%' template --include='*.rs'`, the tag
   pair encloses only one of: fields of the `Stores` struct, the line
   `mod mcp;`, or the `.mcp(...)` line, as listed in obligation 2. Any other
   match fails this criterion.
3. Every file that exists in one generated variant and not in another is
   named in a conditional `ignore` list in `template/cargo-generate.toml`.

---

## REQ-RUN-0305: `AGENTS.md` and `CLAUDE.md` are rendered by `sc-compose`

**Status:** Active  

### Requirement Statement

1. The template MUST contain `template/AGENTS.md.j2` and
   `template/CLAUDE.md.j2`. Both MUST be Jinja templates with frontmatter, in
   the form the `sc-compose` tool validates and renders, matching the
   agent-document templates of the sc-ai-cli project.
2. Both files MUST be listed under `exclude` in
   `template/cargo-generate.toml`. `exclude` makes `cargo-generate` copy a
   file into the generated project byte-for-byte without Liquid rendering.
   That behaviour is conditional on [ADR-RUN-0402](architecture.md), which
   stays Proposed until the spike has verified it.
3. After `cargo generate` finishes, the generation driver
   `scripts/new_project.py` MUST run `sc-compose` on the two copied `.j2`
   files, giving it the same answers JSON that drove `cargo generate`, to
   produce `AGENTS.md` and `CLAUDE.md` at the root of the generated project.
4. The driver MUST then delete `AGENTS.md.j2` and `CLAUDE.md.j2` from the
   generated project.
5. If `sc-compose` fails, the driver MUST fail the run and name the
   `sc-compose` step. This failure behaviour is decided in this document.
6. This item owns the `sc-compose` step and the removal of the `.j2` files;
   the driver pipeline requirement refers to it.
7. These two files MUST NOT contain Liquid syntax intended for
   `cargo-generate`.

**OPEN:** the exact `sc-compose` command line (how the answers JSON and the
input and output paths are passed) is not stated.

**OPEN:** the frontmatter keys these two files must carry are not stated;
they follow whatever `sc-compose` validates.

### Rationale

Liquid (used by `cargo-generate`) and Jinja (used by `sc-compose`) both write
substitutions as `{{ }}`, so one engine cannot safely process a file written
for the other. The agent documents stay validated Jinja because that is what
the sc-ai-cli templates and the `p3-nuget-template` pipeline already use;
each tool does the job it is built for and no render engine is written here.

### Success Criteria

1. `template/cargo-generate.toml` lists `AGENTS.md.j2` and `CLAUDE.md.j2`
   under `exclude`.
2. After `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json`,
   `<dest>/AGENTS.md` and `<dest>/CLAUDE.md` exist and are non-empty.
3. In the same `<dest>`, `find . -name '*.j2'` prints nothing.
4. In the same `<dest>`, `grep -n '{{\|{%' AGENTS.md CLAUDE.md` prints
   nothing, showing no unrendered template syntax remains.
5. A driver test with a stub `sc-compose` that exits 1 asserts the driver
   exits non-zero and its error output names `sc-compose`.

---

## REQ-RUN-0306: Plain `cargo generate` works without wizard or driver

**Status:** Active  

### Requirement Statement

The normal way to generate a project is `just new <dest>`, which runs the
driver `scripts/new_project.py` and, by default, the Wyvern wizard. This
requirement covers a person who has neither.

1. This command MUST generate a project, with `cargo-generate` prompting in
   the terminal for each option:

   ```text
   cargo generate --git https://github.com/randlee/sc-runtime template --name <project>
   ```

   The second positional argument `template` tells `cargo-generate` to use
   the `template/` subfolder of the repository as the template root.
2. It MUST NOT require Wyvern, `sc-compose`, Python, or any file under
   `wizard/` or `scripts/`.
3. Every placeholder in `template/cargo-generate.toml` MUST therefore carry
   its own `prompt` text and, where the option has a default, its own
   `default`, identical to the default in `wizard/answers.schema.json`.
4. The template tree MUST be the same one the driver uses. There MUST NOT be
   a second template, a branch, or files that exist only for this path.
5. Accepting every default MUST produce a project in which
   `cargo build --workspace` exits 0. That this is the meaning of "works" is
   decided in this document.

**OPEN:** in this path nothing runs `sc-compose`, so the generated project
keeps `AGENTS.md.j2` and `CLAUDE.md.j2` unrendered and has no `AGENTS.md` or
`CLAUDE.md`. Whether that is accepted, or documented with a manual
`sc-compose` step, is not decided.

**OPEN:** the default values of the options are not decided, so which variant
"every default" produces is not decided.

**OPEN:** how a CI job accepts every default with no prompt (for example
`--silent`, with or without a values file) depends on rows V4a and V3a of
the spike evidence ([REQ-RUN-0102](requirements.md)) and is conditional on
[ADR-RUN-0402](architecture.md), which stays Proposed until then.

### Rationale

The wizard and the driver are conveniences layered on a standard
`cargo-generate` template. Someone without Wyvern or Python must not be locked
out, and keeping one template tree for both paths means the path most people
use and the path CI tests cannot drift apart.

### Success Criteria

1. A CI job runs `cargo generate --path template --name plain-defaults` with
   every placeholder left at its default and no prompt answered by hand; it
   exits 0. The job has no Wyvern, no `sc-compose` and does not call
   `scripts/new_project.py`. This job is the one recorded exception to the
   rule that CI calls `cargo` only through `just` recipes
   ([REQ-RUN-0004](requirements.md)). Once the no-prompt mechanism above is
   decided: the job uses it.
2. In the project that job produced, `cargo build --workspace` exits 0.
3. Every placeholder in `template/cargo-generate.toml` has a `prompt` key; a
   unit test asserts each placeholder default equals the schema default for
   the same key.
4. The repository contains exactly one `cargo-generate.toml`.
5. A recorded manual check, made once before the `v0.1.0` tag and again when
   `template/cargo-generate.toml` changes: the `--git` command of obligation
   1, run against the pushed repository, prompts for each option and renders
   a project in which `cargo build --workspace` exits 0. The command, date
   and result are recorded in the release pull request.

---

## REQ-RUN-0307: Generated placeholder `Justfile`: `lint`, `test`, `db-prepare`

**Status:** Active  

### Requirement Statement

Every SC repository exposes the same base `just` commands. The full `just`
and lint infrastructure for a generated project will be installed by a
command of the sc-lint project, of the form
`sc-lint create --vars <answers.json>`, which does not exist yet. Until it
does, the template supplies a stand-in.

1. `template/` MUST render a file named `Justfile` at the root of the
   generated project.
2. That `Justfile` MUST define exactly these recipes:

   | Recipe | Behaviour |
   |---|---|
   | `just lint` | runs the project's static checks; exits non-zero on any finding |
   | `just test` | runs the generated workspace's tests, including the `DaemonFixture` example test; exits non-zero on any failure |
   | `just db-prepare` | regenerates the checked-in `sqlx` offline query data in `crates/store-sqlite/.sqlx/` |

3. The recipe names `lint` and `test` are the standard SC base command names
   and MUST NOT be changed.
4. The `Justfile` MUST be minimal: no `just` modules, no imports, no lint
   configuration and no crate-boundary rules.
5. Nothing else rendered from `template/` may depend on the contents of the
   `Justfile`; other files may use only the recipe names. In particular the
   rendered `.github/workflows/ci.yml` MUST contain at least one step that
   runs `just lint` and at least one step that runs `just test`, MUST run
   lint and tests only through those two commands, and `AGENTS.md` and
   `CLAUDE.md` MUST refer to those commands by name only.
6. As a result, when `sc-lint create` later replaces the `Justfile`, no
   workflow and no document in the generated project needs to change.
7. This item owns the contents of the placeholder `Justfile`; the rule that
   the template carries no lint knowledge refers to it.

**OPEN:** the commands inside the `lint` and `test` recipes (for example
which `cargo` invocations and flags) and the exact `db-prepare` command are
not stated.

### Rationale

The standard `just` and lint infrastructure is owned and installed by
sc-lint, so that when its rules change generated projects pick the change up
from sc-lint and this template does not change. But sc-lint builds its
install packages from prototype projects this template generates, so those
prototypes, and template CI, need stable `just lint` and `just test` gates
before sc-lint's command exists. A placeholder with the final command names
gives that without putting lint knowledge in the template.

### Success Criteria

1. In a project generated from `wizard/fixtures/sqlite-mcp.json`,
   `just --list` shows exactly `lint`, `test` and `db-prepare`.
2. In that project `just lint` and `just test` exit 0 with no edits.
3. In that project, copy `crates/store-sqlite/.sqlx/` to a directory outside
   the project, delete the original, and run `just db-prepare`: it recreates
   the directory, and `diff -r` of the copy and the recreated directory
   prints nothing.
4. The rendered `.github/workflows/ci.yml` has at least one `run:` step that
   is `just lint` and at least one that is `just test`; every `run:` step
   that lints or tests is one of those two; the file contains no direct
   `cargo test`, `cargo clippy` or `cargo fmt` invocation.
5. The rendered `Justfile` contains no `mod` or `import` statement, and
   `template/` contains no lint configuration file.

---

## REQ-RUN-0308: Generated `store-sqlite` crate

**Status:** Active  

### Requirement Statement

`crates/store-sqlite` in a project rendered from `template/` is template
code that the project owns. The `sc-runtime` crate constructs nothing in it.
It MUST have these properties:

| Aspect | Requirement |
|---|---|
| Driver | depends on `sqlx` 0.9 with the `sqlite` feature; it is the only generated crate with a direct `sqlx` dependency |
| Pools | two `sqlx` SQLite pools on the same database file: a read pool of N connections, and a write pool of exactly 1 connection; all writes go through the write pool |
| Journal mode | WAL (write-ahead logging) is turned on when the pools are opened |
| Migrations | SQL files in `crates/store-sqlite/migrations/`, embedded in the binary with `sqlx::migrate!` and run when the store is opened |
| Queries | written with `sqlx`'s compile-time-checked macros (`query!`, `query_as!`) |
| Database URL variable | `crates/store-sqlite/sqlx.toml` names `SQLITE_DATABASE_URL` as the variable the `sqlx` macros read, so a second store crate with a different variable can build in the same workspace later |
| Offline data | `crates/store-sqlite/.sqlx/` is produced by `just db-prepare`. It is checked in to git in this repository as `template/crates/store-sqlite/.sqlx/` and rendered into every generated project, which checks it in with its first commit |
| Public API | `open(cfg) -> Result<Store>`, which opens both pools and runs the migrations, plus typed async query functions (for the example: insert a widget, fetch a widget) |
| Callers | only the generated `service` crate depends on `store-sqlite`; REST handlers, MCP tools and the CLI never call it |

The crate MUST NOT contain a write actor, a queue or any other custom
concurrency code; the two-pool arrangement is the whole mechanism.

**OPEN:** the value of N (the read pool size) and whether it is configurable
are not stated.

**OPEN:** the type of `cfg`, the error type inside `Result<Store>`, and how
the database file path is chosen (a config key, or a default under
`<instance-root>`) are not stated. `<instance-root>` is the per-application,
per-user directory resolved by sc-transport, or an explicitly supplied path;
its default location is undecided
([REQ-TRN-0002](sc-transport/requirements.md)).

### Rationale

SQLite allows one writer at a time. Many read connections plus one write
connection, with WAL on, is the standard `sqlx` arrangement: it is correct
under concurrent requests with no custom code, where a single shared pool
can produce "database is locked" errors. A batched write actor is the known
upgrade for a project that measures write contention and is deliberately not
in v0.1. One crate per database with its own URL variable is what `sqlx` 0.9
requires for checked queries, and is what lets `store-postgres` be added
beside this crate later. Offline data in git lets the project build in CI
and on a fresh clone without a database.

### Success Criteria

1. In a generated project, with `SQLITE_DATABASE_URL` unset,
   `SQLX_OFFLINE=true cargo build -p store-sqlite` exits 0.
2. A test opens a `Store` on a tempdir database and issues at least 100
   inserts from concurrently running tasks; every insert returns `Ok`, none
   fails with a busy or locked error, and a count query then returns the
   number inserted.
3. A test opens a `Store` on an empty tempdir database and asserts the widget
   table exists afterwards (migrations ran), and that
   `PRAGMA journal_mode` returns `wal`.
4. Rendered `crates/store-sqlite/sqlx.toml` names `SQLITE_DATABASE_URL`. In
   the `randlee/sc-runtime` repository,
   `git ls-files template/crates/store-sqlite/.sqlx` lists at least one
   file, and a freshly rendered project contains the same files under
   `crates/store-sqlite/.sqlx/` (`diff -r` prints nothing). A freshly
   rendered project has no commit, so `git ls-files` inside it proves
   nothing and is not used.
5. `cargo metadata` in the generated project shows `sqlx` as a direct
   dependency of `store-sqlite` only, and `store-sqlite` as a dependency of
   `service` only.
6. Inspect the source: the write pool is built with a maximum of 1
   connection, and every `INSERT`, `UPDATE` or `DELETE` query function uses
   it.

---

## REQ-RUN-0309: Generated daemon initialises `sc-observability` directly

**Status:** Active  

### Requirement Statement

This applies to `crates/daemon` of a project rendered from `template/`.
`sc-observability` is the SC logging crate.

1. `crates/daemon/Cargo.toml` MUST depend on `sc-observability` with a
   version requirement that admits only 1.2.x releases.
2. `crates/daemon/src/main.rs` MUST perform these three steps, in this order:
   1. load the project's configuration with `sc-config` into the project's
      own config struct; on error, report it and return a failure exit code
      without panicking;
   2. initialise `sc-observability` by calling it directly, passing plain
      values taken from the loaded config (its logging section); if the
      initialisation call returns a guard value, bind it to a variable that
      lives until `main` returns;
   3. assemble and run the daemon with `sc_runtime::Daemon::builder(...)`.

   The shape, with illustrative function names:

   ```rust
   let cfg: AppConfig = match sc_config::load("my-app") {
       Ok(cfg) => cfg,
       Err(e) => return report(e),
   };
   let _obs = sc_observability::init(cfg.logging.clone());
   let result = sc_runtime::Daemon::builder("my-app", &cfg.daemon)
       /* .stores(...).routes(...).mcp(...) */
       .run()
       .await;
   ```

3. None of the four library crates may initialise, wrap or depend on
   `sc-observability` or its OpenTelemetry export crate
   `sc-observability-otlp`; the forbidden edges are owned by
   [REQ-RUN-0005](requirements.md). The only observability dependency
   inside the four library crates is `sc-observability-types`, used by
   `sc-command` for error code and remediation types.
4. The template MUST NOT add OpenTelemetry export. A project that wants it
   adds `sc-observability-otlp` to its own `daemon` crate.
5. The template MUST NOT contain a bridge from the `tracing` crate into
   `sc-observability`; whether such a bridge is acceptable is undecided.

**OPEN:** the exact `sc-observability` initialisation function, its argument
type, whether it returns a guard that must be kept alive, and the keys of
the config's logging section are not stated; the names in the sketch are
illustrative.

**OPEN:** which generated crate defines `AppConfig` is not decided
([REQ-RUN-0301](requirements.md) owns the question).

### Rationale

`sc-observability` is the SC logging standard, so every generated daemon uses
it. It is initialised by the project's own `main.rs`, not hidden inside
`sc-config` or `sc-runtime`, so that those two crates stay usable by programs
that do not use `sc-observability`, and so the observability crates can
release independently of them. Config is loaded first because logging
settings come from config; the daemon is assembled last so that its start-up
is logged.

### Success Criteria

1. Rendered `crates/daemon/Cargo.toml` lists `sc-observability` with a
   requirement matching only 1.2.x, and lists no OpenTelemetry crate.
2. In rendered `crates/daemon/src/main.rs`, the `sc-config` load call comes
   before the `sc_observability` initialisation call, which comes before
   `Daemon::builder`. Once it is confirmed that the initialisation call
   returns a guard: that value is bound to a named variable (not `_`) that
   lives to the end of `main`.
3. In the `randlee/sc-runtime` repository, for each of the four library
   crates, `cargo tree -p <crate> -e normal --prefix none` prints no line
   beginning with `sc-observability ` or `sc-observability-otlp ` (the name
   followed by a space). A line beginning with `sc-observability-types ` is
   expected under `sc-command` and does not match.
4. `grep -rn 'tracing' template/crates/daemon` finds no bridge or subscriber
   set-up code.
5. Rendered `main.rs` contains no `unwrap`, `expect` or `panic!` on the
   config load path.

---

## REQ-RUN-0310: Generated `cli` and `daemon` accept `--endpoint`

**Status:** Active  

### Requirement Statement

This applies to the `cli` and `daemon` crates of a project rendered from
`template/`. The "endpoint" is where the daemon listens: a Unix domain socket
path or a TCP address. The `sc-transport` crate exposes one endpoint resolver
([REQ-TRN-0001](sc-transport/requirements.md)) that picks the first of these
sources that supplies a value:

| Order | Source |
|---|---|
| 1 | the value of the `--endpoint` command-line option |
| 2 | the value of the `SC_ENDPOINT` environment variable |
| 3 | the endpoint in the project's configuration |
| 4 | the platform default: on macOS and Linux the socket `<instance-root>/daemon.sock`; on Windows TCP on `127.0.0.1` with the port from configuration |

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](sc-transport/requirements.md)).

1. The generated `cli` binary and the generated `daemon` binary MUST each
   accept a global command-line option `--endpoint <value>`. "Global" means
   it is accepted with every subcommand of that binary.
2. Each binary MUST obtain its endpoint only from the `sc-transport`
   resolver. It MUST give the resolver the `--endpoint` value (or "absent"),
   the `SC_ENDPOINT` value, and the endpoint from its loaded configuration,
   so that the resolver applies the order above. The resolver accepts the
   `SC_ENDPOINT` value as an explicit input; a convenience form of the
   resolver that reads it with `std::env::var_os` may be used instead.
3. Neither binary may compute an endpoint itself. Template code MUST NOT
   join a socket file name onto a directory, choose a default port, or
   apply its own precedence between the sources.
4. `cli` passes the resolved endpoint to the `sc-transport` client.
5. `daemon` does not bind a listener itself. It MUST place the `--endpoint`
   value into the `DaemonConfig` value it passes to
   `sc_runtime::Daemon::builder(...)`, together with the configured endpoint
   and any explicit instance root. `run()` then resolves the bind endpoint
   through the same `sc-transport` resolver
   ([REQ-RT-0008](sc-runtime/requirements.md)).
6. The endpoint and instance-root part of the project's configuration MUST
   be a `Deserialize` type exported by `sc-transport` with default features.
   `cli` uses that type in its configuration, because `cli` MUST NOT depend
   on `sc-runtime` and so cannot name `DaemonConfig`.

This requirement is decided in this document. The sc-runtime design names the
override ("`--endpoint` flag or `SC_ENDPOINT`") and shows `cfg.endpoint` in
its CLI sketch, but does not oblige the template to implement them.

**OPEN:** the names of the `DaemonConfig` fields that carry the `--endpoint`
value, the configured endpoint and the explicit instance root are not
decided ([REQ-RT-0001](sc-runtime/requirements.md)).

**OPEN:** the name of the `sc-transport` configuration type of obligation 6,
the resolver's name and parameter list, and the string syntax of an endpoint
value are not decided ([REQ-TRN-0001](sc-transport/requirements.md)).

**OPEN:** whether the binaries also accept an option for an explicit instance
root is not decided.

### Rationale

The daemon and the CLI are separate binaries with no run-time coordination.
If either computes the endpoint its own way, they stop meeting as soon as
one source is set. Requiring both to go through the one resolver makes that
impossible. The override also is what lets a test point the generated CLI at
an isolated `DaemonFixture` daemon: the three-surface tests of
[REQ-RUN-0201](requirements.md), [REQ-RUN-0202](requirements.md) and
[REQ-RUN-0203](requirements.md) all depend on it. Everyone who runs more
than one instance of a generated daemon, and every test author, is affected.

### Success Criteria

1. A test starts a daemon with `sc_runtime::testing::DaemonFixture`, takes
   the fixture's endpoint string (the form accepted by `--endpoint` and
   `SC_ENDPOINT`, [REQ-RT-0006](sc-runtime/requirements.md)), and runs the
   generated CLI's `widget.create` command with `--endpoint <string>` and
   `--json`, with `SC_ENDPOINT` unset. It asserts exit status 0 and `ok`
   equal to `true`.
2. The same test runs the CLI again with no `--endpoint` option and with
   `SC_ENDPOINT` set to that string in the child process environment only,
   and makes the same assertions.
3. A test runs the CLI with `--endpoint` set to the fixture's endpoint
   string and `SC_ENDPOINT` set to a socket path where nothing listens; the
   command succeeds, showing the option takes precedence.
4. Conditional on the OPEN above on whether the binaries accept an option
   for an explicit instance root: a Unix-only test starts the generated `daemon`
   binary with a fresh tempdir as its instance root and with
   `--endpoint <socket path inside that tempdir>`, waits until that socket
   file exists, and runs the CLI with the same `--endpoint` value; the
   command reaches the daemon, and `daemon.lock` exists inside that tempdir.
   The test then stops the daemon. Until that OPEN is decided this criterion
   is not run, because starting the binary without an explicit instance root
   would lock the developer's real one
   ([NFR-RUN-0005](requirements.md)).
5. `<cli> --help` and `<daemon> --help` each list `--endpoint`.
6. `grep -rn 'daemon\.sock' template/crates` prints nothing, and inspection
   of `template/crates/cli` and `template/crates/daemon` finds every endpoint
   value produced by a call into `sc_transport`.

---

## REQ-RUN-0401: Options contract `wizard/answers.schema.json`

**Status:** Active  

### Requirement Statement

The repository MUST contain the file `wizard/answers.schema.json`. It MUST be
a valid JSON Schema document whose top level describes one JSON object, called
here the answers object. An answers object is the JSON file that drives
project generation; it is written by hand, by an agent, or by the Wyvern
wizard under `wizard/`.

The schema MUST be the only definition of the generation options. For every
option it MUST state the option's name (the property key), its JSON type, its
default where it has one, its allowed values where it is an enum, and any
conditional requirement between options. No other file in the repository may
introduce an option that is absent from the schema.

This item owns the list of v0.1 options. In v0.1 the schema MUST define these
three options and MUST NOT define any other. That the v0.1 list is closed is
decided in this document.

| Option | JSON type | Values the schema lists |
|---|---|---|
| project name | string | any non-empty string |
| `db` | string | `sqlite`, `postgres`, `both` |
| `mcp` | boolean | `true`, `false` |

What each option does to the rendered project, and the rule that `postgres`
and `both` are listed in the enum but not accepted in v0.1, are owned by
[REQ-RUN-0303](requirements.md).

The schema MUST carry a schema version, because it is a published contract:
the sc-lint project reads the same answers object and the same schema to
install lint and `just` infrastructure into a generated project.

The consumers of the schema are: the wizard (emits an object of this shape),
the driver `scripts/new_project.py` (validates against it), the placeholders
in `template/cargo-generate.toml` (use the same key names), and sc-lint.
The driver MUST NOT hold a list of option names or option defaults of its
own; it MUST read both from the schema.

The clause about placeholders in `template/cargo-generate.toml` is conditional
on [ADR-RUN-0402](architecture.md), which is Proposed: how `cargo-generate`
placeholders are declared and how values reach them is unverified until the
spike has recorded the working syntax. The rest of this item does not depend
on that ADR.

**OPEN:** the property key for the project-name option is not decided.
Whether that key also appears as a placeholder in
`template/cargo-generate.toml` is not decided either, because
`cargo generate` takes the project name through `--name`
([REQ-RUN-0402](requirements.md) owns that question).  
**OPEN:** the defaults of `db` and `mcp` are not decided.  
**OPEN:** how the schema version is carried (which key, which format) and what
kind of schema change requires a new version are not decided.  
**OPEN:** the JSON Schema draft (the `$schema` value) is not decided.  
**OPEN:** the language and test runner of `tests/unit/` are not decided (the
driver is Python).  
**OPEN:** how an answers object with `db` equal to `postgres` or `both` is
rejected in v0.1 is not decided ([REQ-RUN-0303](requirements.md) owns the
question): an extra constraint in this schema, or a check in the driver. A
check in the driver would be option knowledge held outside the schema, which
the rule above forbids; if that branch is chosen, this item must be amended
to allow that one check.

### Rationale

The wizard, the driver, `template/cargo-generate.toml` and sc-lint all need
the option set. One definition is the only way they cannot disagree. The
answers object and this schema are the entire interface between this project
and sc-lint, so the schema is versioned like any other published contract.

### Success Criteria

1. A unit test under `tests/unit/` loads `wizard/answers.schema.json` and
   asserts that it is a valid schema under the meta-schema named by its own
   `$schema` value.
2. The same test asserts that the top-level `type` is `object`, that `db` is
   a string property whose `enum` is exactly `sqlite`, `postgres`, `both`,
   that `mcp` is a boolean property, and that a string property for the
   project name exists.
3. The same test asserts that the top-level `properties` object has exactly
   three keys: `db`, `mcp` and the project-name key. Once the way the schema
   version is carried is decided: if it is carried as a property, that key is
   the only permitted fourth key.
4. The same test asserts that the schema version is present and non-empty.
5. Inspection: `scripts/new_project.py` contains no list of option names or
   option defaults of its own; it reads both from the schema. Once the
   rejection mechanism for `db` equal to `postgres` or `both` is decided: if
   it is a driver check, that check is the single permitted exception and
   this criterion is amended to name it.

---

## REQ-RUN-0402: Key-set test: schema versus template placeholders

**Status:** Active  

### Requirement Statement

`tests/unit/` MUST contain a unit test that compares two sets of names:

| Set | Where it is read from |
|---|---|
| schema keys | the keys of the top-level `properties` object in `wizard/answers.schema.json` |
| placeholder keys | the names of the template-defined placeholders declared in `template/cargo-generate.toml` |

The test MUST fail when the two sets differ in either direction: a schema key
with no placeholder, or a placeholder with no schema key. The failure message
SHOULD list the names found in only one of the two sets.

The test MUST read both files from the working tree at test time. It MUST NOT
compare against a hard-coded list of names.

The test MUST run as part of `just test` at the repository root.

This test is the single enforcement of the rule that a generation option is
added to `wizard/answers.schema.json` and to `template/cargo-generate.toml`
together.

How placeholders are declared in `template/cargo-generate.toml` (the table
the test reads, and the syntax of `string`, `bool` and `array` placeholders)
is a `cargo-generate` mechanic that is unverified until the spike records it;
it is held in [ADR-RUN-0402](architecture.md), which is Proposed. The part of
this item that says where placeholder keys are read from is conditional on
that ADR. The obligation that the two key sets are compared, in both
directions, from the working tree, is not conditional.

**OPEN:** `cargo generate` receives the project name through its `--name`
argument, not through a template-defined placeholder. Whether the schema's
project-name key is exempt from this comparison, or is also declared as a
placeholder, is not decided. Until it is, "the two sets are equal" is not
defined for that one key; it is defined for every other key.  
**OPEN:** the language and test runner of `tests/unit/` are not decided (the
driver is Python).

### Rationale

The generation driver turns each key of the answers JSON into a value passed
to `cargo generate`, and the template reads those values by placeholder name.
An option present in the schema and missing from the template produces a
template that silently ignores an answer. A placeholder missing from the
schema can never be set by the wizard or by an answers file. The wizard, the
driver and sc-lint all trust the schema, so the drift has to be caught
mechanically.

### Success Criteria

1. `just test` at the repository root runs the key-set test and it passes on
   an unmodified checkout.
2. Adding a property named `zz_probe` to `properties` in
   `wizard/answers.schema.json`, with no other change, makes the test fail.
3. Adding a placeholder named `zz_probe` to `template/cargo-generate.toml`,
   with no other change, makes the test fail.
4. Inspection: the test source contains no literal list of option names.
5. Once the project-name question above is decided: if the key is exempt,
   the test source exempts exactly that one key, read from a single named
   constant, and criterion 4 allows that constant; if the key is a
   placeholder, the test exempts nothing.

---

## REQ-RUN-0403: Answers fixtures and fail-closed validation tests

**Status:** Active  

### Requirement Statement

The directory `wizard/fixtures/` MUST hold one answers JSON file per tested
variant of the template. A variant is one combination of option values that
the template supports. Every file matching `wizard/fixtures/*.json` MUST be a
complete answers object that validates against
`wizard/answers.schema.json`, because template CI generates a project from
every file matching that glob.

`wizard/fixtures/` MUST contain `sqlite-mcp.json`, the variant with
`db = sqlite` and `mcp = true`. It MUST also contain a variant with
`db = sqlite` and `mcp = false`.

The repository MUST also hold answers files that must be refused, used only
by unit tests. None of them may match the glob `wizard/fixtures/*.json`.
There are two classes:

| Class | What the file is | What must refuse it |
|---|---|---|
| schema-invalid | Breaks the schema in one way: a missing required key, a value of the wrong type, a `db` value outside the enum, or a key the schema does not define. | schema validation |
| reserved in v0.1 | A complete answers object whose `db` is `postgres`, and another whose `db` is `both`. Both values are inside the schema's enum, and neither is accepted in v0.1. | the driver's validate step; whether that is schema validation or an extra driver check is undecided (see the OPEN below) |

`tests/unit/` MUST contain tests that assert:

- every file matching `wizard/fixtures/*.json` validates against the schema;
- every schema-invalid file fails schema validation;
- the driver `scripts/new_project.py`, given a schema-invalid file through
  `--var-file`, fails closed. Fail closed means: it exits with a non-zero
  status, it does not run `cargo generate`, and it does not create the
  destination directory.

The test that the driver fails closed on the two reserved-in-v0.1 files is
owned by [REQ-RUN-0303](requirements.md), which owns the rule that those
values are not accepted; this item only requires the two files to exist.

No clause of this item depends on the unverified `cargo-generate` mechanics
held in [ADR-RUN-0402](architecture.md): the fail-closed test uses a stub
`cargo` and only asserts that it was not called.

**OPEN:** the directory that holds the refused answers files is not decided.  
**OPEN:** the file name of the `mcp = false` fixture, and the file names of
the refused answers files, are not decided.  
**OPEN:** whether the reserved-in-v0.1 files fail schema validation is not
decided. It depends on how `db` equal to `postgres` or `both` is rejected
(an extra schema constraint, or a driver check), which
[REQ-RUN-0303](requirements.md) holds open. Once it is decided: if the
rejection is a schema constraint, the reserved files join the schema-invalid
class and criterion 3 covers them.  
**OPEN:** the language and test runner of `tests/unit/` are not decided (the
driver is Python).

### Rationale

The fixtures are the template's test matrix: each one becomes a generated
project that CI lints and tests. They are also one of the three things handed
to the sc-lint team as the reference for its install packages, the other two
being the schema and the prototype repositories generated from the fixtures.
Failing closed matters because a project rendered from bad answers looks
finished and is wrong.

### Success Criteria

1. A unit test validates every file matching `wizard/fixtures/*.json` against
   `wizard/answers.schema.json` and passes.
2. `wizard/fixtures/sqlite-mcp.json` exists and contains `db` equal to
   `"sqlite"` and `mcp` equal to `true`. Another file in the directory
   contains `db` equal to `"sqlite"` and `mcp` equal to `false`.
3. A unit test validates every schema-invalid answers file and asserts that
   each one is rejected. The set contains at least one file for each of the
   four ways of breaking the schema named in the table.
4. A unit test runs the driver for a destination `<dest>` with
   `--var-file <schema-invalid file>` and a stub `cargo` first on `PATH` that
   records any call. It asserts a
   non-zero exit status, that the stub recorded no call, and that `<dest>`
   does not exist afterwards.
5. `ls wizard/fixtures/*.json` lists no schema-invalid file and no
   reserved-in-v0.1 file.
6. Outside `wizard/fixtures/`, the repository holds one complete answers
   file with `db` equal to `"postgres"` and one with `db` equal to `"both"`.

---

## REQ-RUN-0501: Generation driver `scripts/new_project.py` pipeline

**Status:** Active  

### Requirement Statement

The repository MUST contain a Python script `scripts/new_project.py`, called
here the driver. The root `justfile` MUST expose it as `just new <dest>`,
where `<dest>` is where the new project is created. The driver and
`scripts/run_wizard.py` live under `scripts/`, outside `template/`, so they
are never copied into a generated project; the repository layout that
guarantees this is owned by [REQ-RUN-0001](requirements.md).

The driver MUST obtain one answers JSON object, either from the file given
with `--var-file` or from the Wyvern wizard, and then MUST run these steps in
this order:

| # | Step | What the driver does |
|---|---|---|
| 1 | validate | Validates the answers object against `wizard/answers.schema.json`. |
| 2 | values file | Writes the validated answers to a TOML file as one `[values]` table, one entry per answers key. `string`, `bool` and `array` values are supported. |
| 3 | `cargo generate` | Runs `cargo generate --path template --template-values-file <values file> --name <project>` (plus `--silent` if the spike finds it is needed; see the OPEN below) with no terminal prompt. |
| 4 | `sc-compose` | Runs `sc-compose` so that the new project holds `AGENTS.md` and `CLAUDE.md` and no `.j2` file. What is rendered, from which inputs, and the removal of the two `.j2` files are owned by [REQ-RUN-0305](requirements.md); this item owns only the position of the step in the order. |
| 5 | `just lint` | Runs `just lint` in the new project. |
| 6 | `just test` | Runs `just test` in the new project. |

The driver MUST stop at the first step that fails. It MUST then exit with a
non-zero status and MUST write a message to stderr that identifies which of
the six steps failed. It MUST NOT run any later step. When every step
succeeds the driver MUST exit with status 0. Stopping at the first failed
step and naming it on stderr is decided in this document. This paragraph is
the single owner of that behaviour for all six steps, the `sc-compose` step
included.

The driver MUST NOT render any template file itself; rendering is done only
by `cargo generate` and `sc-compose`.

`scripts/run_wizard.py` SHOULD be reused from the `p3-nuget-template`
repository (`.scaffold/scripts/run_wizard.py`) with as few changes as
possible, because that script is already driven by the schema file.

The mechanism clauses of steps 2 and 3 are conditional on
[ADR-RUN-0402](architecture.md), which is Proposed. Those clauses are: the
`[values]` table form of the values file, the supported value types, the
`--template-values-file` and `--silent` flags, and the claim that the command
runs with no prompt. They are unverified until the spike has run them on the
pinned `cargo-generate` version. If the spike finds a different mechanism,
steps 2 and 3 change to it; the six steps, their order and the rule that the
driver stops at the first failure do not.

**OPEN:** how `<dest>` maps onto `cargo generate`'s `--name` and output
directory arguments is not decided.  
**OPEN:** the exact `sc-compose` command line (how the answers and the two
`.j2` paths are passed) is not decided.  
**OPEN:** where the values TOML file is written, and whether it is deleted
afterwards, is not decided.  
**OPEN:** whether step 3 also passes `--silent` depends on the spike result
for non-interactive `cargo generate` runs.  
**OPEN:** whether the project-name answer is also written into the `[values]`
table, or is passed only through `--name`, is not decided
([REQ-RUN-0402](requirements.md) owns the question of whether the
project-name key is a placeholder).

### Rationale

The pipeline follows the one already proven in the `p3-nuget-template`
repository: a wizard emits an answers JSON, a schema validates it, a driver
renders the project. Here `cargo-generate` takes the place of that
repository's custom render engine, so no render engine is written. Running
lint and test last means a generated project is known good at birth, and
naming the failed step tells a person or an agent where to look.

### Success Criteria

1. `just --list` at the repository root shows a `new` recipe, and the recipe
   body invokes `scripts/new_project.py`.
2. A driver test runs the driver with `--var-file` on a valid fixture and
   stub executables for `cargo`, `sc-compose` and `just` that record their
   calls. It asserts the recorded order is `cargo generate`, `sc-compose`,
   `just lint`, `just test`. Conditional on ADR-RUN-0402 being confirmed: it
   also asserts that the `cargo generate` call contains `--path template`,
   `--template-values-file` and `--name`.
3. Conditional on ADR-RUN-0402 being confirmed: the same test reads the file
   passed to `--template-values-file` and asserts it parses as TOML with a
   single `[values]` table. Once the project-name question above is decided:
   the table's keys equal the keys of the answers object, less the
   project-name key if that key is passed only through `--name`.
4. For each of steps 3 to 6, a driver test makes that step's stub exit
   non-zero and asserts: the driver exits non-zero, stderr identifies that
   step, and no later stub was called.
5. A driver test with an answers file that fails validation asserts that
   stderr identifies the validate step and that no stub was called.
6. The end-to-end run in [REQ-RUN-0503](requirements.md), which generates a
   real project from `wizard/fixtures/sqlite-mcp.json`, exits 0. That the
   project then holds `AGENTS.md` and `CLAUDE.md` and no `.j2` file is
   checked by the criteria of [REQ-RUN-0305](requirements.md).

---

## REQ-RUN-0502: Fully non-interactive mode and tool discovery

**Status:** Active  

### Requirement Statement

The generation driver `scripts/new_project.py` (run as `just new <dest>`)
MUST accept `--var-file <answers.json>`, where the file is a complete answers
JSON object for `wizard/answers.schema.json`.

With `--var-file` the driver MUST NOT read from stdin, MUST NOT show any
prompt, and MUST NOT start the Wyvern wizard. It MUST NOT look for or require
the `wyvern` executable in this mode. Every tool it starts (`cargo generate`
in particular) MUST be started so that it cannot prompt. This item is the
single owner of the rule that `--var-file` never runs the wizard.

How `cargo generate` is made unable to prompt (a values file given with
`--template-values-file`, and possibly `--silent`) is a `cargo-generate`
mechanic held in [ADR-RUN-0402](architecture.md), which is Proposed and
unverified until the spike has run it with stdin closed. The obligation that
nothing prompts is not conditional; the flags that achieve it are.

The driver MUST locate its two external tools in this order:

| Tool | First | Then |
|---|---|---|
| `sc-compose` | the executable named by the environment variable `SC_COMPOSE`, when set | `sc-compose` on `PATH` |
| `wyvern` | the executable named by the environment variable `WYVERN_BIN`, when set | `wyvern` on `PATH` |

When a required tool is found in neither place the driver MUST exit with a
non-zero status. It MUST do so before running `cargo generate`, so that no
half-generated project is left behind (the timing is decided in this
document). Its stderr message MUST name
the missing tool and MUST state both ways to supply it: the environment
variable and `PATH`.

The driver MUST NOT contain a hard-coded install path for either tool.

### Rationale

CI and agents generate projects, and neither can answer a prompt or open a
wizard window. Wyvern is a desktop UI tool that CI does not install, so the
non-interactive mode must not depend on it. An environment-variable override
lets a machine point at an install that is not on `PATH` without the script
guessing install prefixes.

### Success Criteria

1. CI runs `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` with
   stdin closed (`< /dev/null`) on a runner with no `wyvern` installed, and
   the command exits 0.
2. A driver test runs `--var-file` mode with `WYVERN_BIN` unset and no
   `wyvern` on `PATH` and asserts that the run does not fail for that reason.
3. A driver test runs `--var-file` mode with `WYVERN_BIN` set to a stub
   executable that records any call, and asserts that the stub was never
   called.
4. A driver test sets `SC_COMPOSE` to a stub executable, puts a different
   stub named `sc-compose` on `PATH`, and asserts that the one named by
   `SC_COMPOSE` was called.
5. A driver test with `SC_COMPOSE` unset and no `sc-compose` on `PATH`
   asserts a non-zero exit, that stderr contains `sc-compose`, `SC_COMPOSE`
   and `PATH`, and that `cargo generate` was not called.
6. A driver test in wizard mode (no `--var-file`) with `WYVERN_BIN` unset and
   no `wyvern` on `PATH` asserts a non-zero exit and that stderr contains
   `wyvern`, `WYVERN_BIN` and `PATH`.
7. Inspection: `scripts/new_project.py` and `scripts/run_wizard.py` contain
   no absolute path to `wyvern` or `sc-compose`.

---

## REQ-RUN-0503: A generated project is green with no edits

**Status:** Active  

### Requirement Statement

This item owns the definition of "green with no edits" and applies it to one
answers fixture. [REQ-RUN-0701](requirements.md) applies the same sequence to
every fixture in CI, and [REQ-RUN-0702](requirements.md) requires it at the
release commit; neither restates it.

Running `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` from the
root of a clean checkout of this repository MUST exit 0 and MUST create a
project at `<dest>`. `wizard/fixtures/sqlite-mcp.json` is the answers fixture
with `db = sqlite` and `mcp = true`.

In that project, with no file added, changed or deleted after generation,
`just lint` MUST exit 0 and `just test` MUST exit 0. This sequence (`just new`
with `--var-file`, then `just lint`, then `just test`, with nothing written
to the project in between) is what "green with no edits" means.

`just test` in the generated project MUST include the template's example
operation test, which calls `widget.create` and `widget.get` through REST,
MCP and the CLI against a daemon started by
`sc_runtime::testing::DaemonFixture`.

The generated project MUST resolve the crates `sc-config`, `sc-transport`,
`sc-command` and `sc-runtime` without any manual step. Before those crates
are published to crates.io, a `[patch.crates-io]` entry or a git tag supplies
them; that allowance, and the undecided question of who writes the
`[patch.crates-io]` entry, are owned by [REQ-RUN-0003](requirements.md).

`just new` renders the project with `cargo generate`. The `cargo-generate`
mechanics it relies on (conditional `ignore`, `exclude`, a values file with
no prompt) are held in [ADR-RUN-0402](architecture.md), which is Proposed
until the spike has verified them. This item states the result that must
hold whatever mechanism is confirmed, so none of its criteria is conditional.

### Rationale

This is the exit gate for the template in the minimum viable product. The
first thing a new project does must not be fixing the scaffold, and an agent
that generates a project must be able to trust that a red build is its own
doing.

### Success Criteria

These criteria are met by the `sqlite-mcp` entry of the fixture-matrix
workflow of [REQ-RUN-0701](requirements.md); no separate CI job is required.

1. A CI job runs `just new "$RUNNER_TEMP/app" --var-file
   wizard/fixtures/sqlite-mcp.json` and the command exits 0.
2. The same job then runs `just lint` in `$RUNNER_TEMP/app` with no step in
   between that writes to that directory, and it exits 0.
3. The same job then runs `just test` in `$RUNNER_TEMP/app`, and it exits 0.
4. The `just test` output lists the example operation test as run and passed,
   not skipped or filtered out.

---

## REQ-RUN-0601: A Wyvern wizard built by copy and edit

**Status:** Active  

### Requirement Statement

The repository MUST contain a Wyvern wizard that collects the generation
options from a person. Wyvern is the tool that shows a wizard's HTML pages
and prints the collected data as JSON on stdout when the person finishes. The
wizard consists of:

| Path in this repository | What it is |
|---|---|
| `wizard/wizard.json` | the Wyvern wizard descriptor |
| `wizard/pages/*.html` | the wizard pages |
| `wizard/pages/wizard-answers.js` | the helper script shared by the pages |
| `wizard/pages/wizard-page.css` | the stylesheet shared by the pages |

The wizard MUST be made by copying the working wizard of the private
repository `Radiant-Vision-Systems/p3-nuget-template` (read at commit
`9ef4d03`) and editing it. The files to copy are
`.scaffold/wizard/wizard.json`, `.scaffold/wizard/pages/*.html`,
`.scaffold/wizard/pages/wizard-answers.js` and
`.scaffold/wizard/pages/wizard-page.css`. The edit MUST change the fields and
lists on the pages so that they match the options defined in
`wizard/answers.schema.json` as that schema stands when the wizard is built.
The wizard MUST NOT be written from scratch.

The number of pages and their file names are not constrained.

The wizard MUST offer an input for every option in the schema and MUST NOT
offer an input for anything the schema does not define.

The wizard lives under `wizard/`, outside `template/`, so no wizard file is
rendered into a generated project. The repository layout that guarantees this
is owned by [REQ-RUN-0001](requirements.md); this item checks the result for
the wizard's files only (criterion 5).

### Rationale

The `p3-nuget-template` wizard is a working example that is already driven by
an answers schema, so the work here is an edit of about one to two hours, not
a build. Keeping `wizard/` beside `template/` and not inside it means nothing
has to be deleted from a generated project after rendering.

### Success Criteria

1. `wizard/wizard.json`, at least one `wizard/pages/*.html`,
   `wizard/pages/wizard-answers.js` and `wizard/pages/wizard-page.css` exist.
2. Every page file referenced from `wizard/wizard.json` or from another page
   exists under `wizard/pages/`.
3. Manual check on a machine with Wyvern installed: `just new <dest>` with no
   `--var-file` opens the wizard, every schema option can be set, and
   finishing the wizard generates a project at `<dest>`.
4. `grep -ri "nuget" wizard/wizard.json wizard/pages/` prints nothing, which
   shows the copied fields were edited.
5. In a project generated from every file in `wizard/fixtures/*.json`,
   `find <dest> -name 'wizard.json' -o -name 'answers.schema.json' -o -name
   'wizard-answers.js' -o -name 'wizard-page.css'` prints nothing.

---

## REQ-RUN-0602: Driver entry modes: wizard, `--prefill`, `--var-file`

**Status:** Active  

### Requirement Statement

The generation driver `scripts/new_project.py` (run as `just new <dest>`)
MUST support exactly three ways of obtaining the answers JSON object:

| Mode | Command line | Behaviour |
|---|---|---|
| interactive | neither `--var-file` nor `--prefill` | The driver runs the Wyvern wizard described by `wizard/wizard.json`. The person types every answer. |
| prefill | `--prefill <json>` | `<json>` is a partial answers object, typically written by an agent as its best guess. The driver merges it into the `config.prefill` object of the wizard descriptor and then runs the wizard, so the person reviews and corrects the values and does not type them. |
| non-interactive | `--var-file <json>` | `<json>` is a complete answers object. The wizard is not run in this mode; that rule and its test are owned by [REQ-RUN-0502](requirements.md). |

In all three modes the answers object MUST then be validated against
`wizard/answers.schema.json` and passed through the same generation steps.
No later step may behave differently depending on which mode supplied the
answers.

Prefill values are suggestions only. The answers that are validated and used
are the ones the wizard returns after the person finishes it.

When the wizard ends without the person finishing it, the driver MUST NOT run
`cargo generate`. This rule is decided in this document.

**OPEN:** the behaviour when `--prefill` and `--var-file` are both given is
not decided.  
**OPEN:** the handling of a `--prefill` key that the schema does not define is
not decided. The reference script `run_wizard.py` in `p3-nuget-template` drops
such keys with a warning on stderr.  
**OPEN:** the exit status when the person cancels the wizard is not decided.

### Rationale

These are the three modes already proven in the `p3-nuget-template`
repository: a person answers, an agent guesses and a person reviews, or an
agent or CI supplies everything with no person present. All three produce the
same answers JSON, which is why nothing after validation needs to know which
one was used.

### Success Criteria

Each test replaces Wyvern with a stub executable named through the
`WYVERN_BIN` environment variable. The stub records the descriptor file it
was given and prints a canned finished-wizard result.

1. Interactive mode: a driver test runs with no `--var-file` and no
   `--prefill`. It asserts that the stub was called once and that the answers
   passed to `cargo generate` equal the answers in the stub's canned result.
2. Prefill mode: a driver test runs with `--prefill p.json`, where `p.json`
   sets one option. It asserts that the descriptor the stub received has that
   option and value under `config.prefill`.
3. Prefill mode: the same test asserts that the answers passed to
   `cargo generate` equal the stub's canned result, not the prefill file.
4. Non-interactive mode: a driver test runs with `--var-file` on a valid
   fixture. It asserts that the answers passed to `cargo generate` equal the
   fixture. (That the Wyvern stub is never called in this mode is asserted by
   the criteria of REQ-RUN-0502.)
5. A driver test whose stub prints a cancelled-wizard result asserts that
   `cargo generate` was not called.

---

## REQ-RUN-0603: Wizard output equals the equivalent answers fixture

**Status:** Active  

### Requirement Statement

The answers JSON object produced by the Wyvern wizard under `wizard/` MUST
validate against `wizard/answers.schema.json`.

When a person makes the same choices in the wizard as an answers fixture in
`wizard/fixtures/` contains, the wizard's answers object MUST equal that
fixture. Equal means the two parsed JSON values are equal: the same keys,
each with the same value and JSON type. Key order and whitespace do not
matter.

The wizard MUST NOT emit a key that the schema does not define. It MUST NOT
omit a key that the schema requires.

A project generated from the wizard's answers MUST be identical, file for
file, to the project generated from the equal fixture with `--var-file`.

**OPEN:** whether this comparison is an automated test or a recorded manual
check is not decided. Wyvern is not installed in CI, so an automated test
cannot drive the real wizard there.

### Rationale

Because the wizard and a hand-written answers file produce the same JSON,
nothing else in the system depends on which one was used. That is what lets
the wizard be built last, slip, or be reshaped without touching the schema,
the template, the driver or CI.

### Success Criteria

1. On a machine with Wyvern installed, run the wizard and choose the project
   name, `db = sqlite` and `mcp = true` exactly as
   `wizard/fixtures/sqlite-mcp.json` has them. Save the resulting answers
   object to `out.json`.
2. `out.json` validates against `wizard/answers.schema.json`.
3. A comparison of the parsed JSON of `out.json` and
   `wizard/fixtures/sqlite-mcp.json` reports them equal (for example
   `python3 -c "import json,sys; sys.exit(json.load(open('out.json')) !=
   json.load(open('wizard/fixtures/sqlite-mcp.json')))"` exits 0).
4. Generate one project from `out.json` and one from the fixture, both with
   `--var-file`, into two directories. `diff -r` of the two directories,
   ignoring build output, reports no difference.

---

## REQ-RUN-0701: Template CI: answers-fixture matrix workflow

**Status:** Active  

### Requirement Statement

A GitHub Actions workflow under `.github/workflows/` in this repository MUST
test the template by generating real projects. For every file matching
`wizard/fixtures/*.json`, each of which is one complete answers JSON file for
one supported option combination, the workflow MUST run these commands in
this order and MUST fail when any of them exits non-zero:

1. `just new <dest> --var-file <fixture>` from the repository root, which
   runs the generation driver `scripts/new_project.py` without the wizard.
2. `just lint` inside the generated project at `<dest>`.
3. `just test` inside the generated project at `<dest>`.

No step of the workflow may write to `<dest>` between those commands. This
is the "green with no edits" sequence that
[REQ-RUN-0503](requirements.md) defines for one fixture; this item owns
running it for every fixture in CI.

Each fixture MUST be a separate matrix entry, so that one failing variant is
reported by its fixture name. The list of fixtures MUST be derived from the
files present in `wizard/fixtures/`. Adding a fixture file MUST NOT require
an edit to the workflow. The workflow MUST run on every pull request. These
four rules (one entry per fixture, the list derived from the directory, no
workflow edit for a new fixture, every pull request) are decided in this
document.

In every run of the workflow except the release run required by
[REQ-RUN-0702](requirements.md), each generated project MUST resolve the four
crates `sc-config`, `sc-transport`, `sc-command` and `sc-runtime` from the
checkout under test, not from crates.io and not from another commit. This is
what makes template CI test the template against the crates at the same
commit, so that a pull request changing a crate API and the template together
is tested as one change. The means follows from
[REQ-RUN-0003](requirements.md), which permits two stand-ins for testing
unreleased crate changes: a git tag, or a `[patch.crates-io]` entry. A git
tag cannot name the commit of an open pull request, which leaves a
`[patch.crates-io]` entry with a `path` to each crate directory of the
checkout.

The workflow MUST NOT install or invoke Wyvern.

From the release that adds the `store-postgres` crate, a fixture whose
project needs PostgreSQL MUST be given a Postgres service container by the
workflow. This sentence places no obligation on v0.1: the only accepted `db`
value in v0.1 is `sqlite`, so no v0.1 fixture needs the container.

The workflow MUST NOT use the `cargo-generate-action` GitHub Action as its
matrix mechanism. The driver is what is tested.

`just new` renders each project with `cargo generate`. The `cargo-generate`
mechanics it relies on (conditional `ignore`, `exclude`, a values file with
no prompt) are held in [ADR-RUN-0402](architecture.md), which is Proposed
until the spike has verified them. This item names only `just` commands, so
none of its clauses changes if the spike finds a different mechanism.

**OPEN:** the workflow file name is not decided.  
**OPEN:** who writes the `[patch.crates-io]` entries into the generated
project (a template option, the driver `scripts/new_project.py`, or this
workflow) is not decided; [REQ-RUN-0003](requirements.md) owns the question.
Writing them after generation must not count as an edit that breaks "green
with no edits"; how that is reconciled is part of the same question.

### Rationale

Every supported option combination is generated, linted and tested on every
change. That is what catches a new `rmcp` release, a crate API change or a
template edit breaking generated projects before a user meets it. Running the
real driver with `--var-file` tests the same path agents and CI users take.
Keeping the crates and the template in one repository is only useful if CI
tests them together at one commit; after the first publish a generated
project would otherwise resolve the published crates, and a pull request
would be tested against the old crate code.

### Success Criteria

1. A workflow file under `.github/workflows/` contains a job that runs
   `just new` with `--var-file`, then `just lint`, then `just test` in the
   generated directory.
2. A run of that workflow shows one matrix entry per file in
   `wizard/fixtures/*.json`, each labelled with its fixture name, all green.
3. A pull request that adds a new valid fixture file and changes nothing else
   produces a run with one more matrix entry.
4. A pull request that breaks the template for `mcp = false` only turns that
   variant's entry red and leaves `sqlite-mcp` green.
5. `grep -ri "wyvern" .github/workflows/` prints nothing, and the matrix job
   contains no `uses:` line naming `cargo-generate-action`.
6. In a pull-request run, the `Cargo.lock` of each generated project lists
   the packages `sc-config`, `sc-transport`, `sc-command` and `sc-runtime`
   with no `source` line (a package resolved from a path has none; one
   resolved from crates.io has
   `source = "registry+https://github.com/rust-lang/crates.io-index"`).
7. A pull request that renames a public function in one of the four crates
   and updates the template's call to it, with no other change, produces a
   green matrix.

---

## REQ-RUN-0702: Release `v0.1.0`: crates published, repository tagged

**Status:** Active  

### Requirement Statement

The release `v0.1.0` of the repository `randlee/sc-runtime` is complete only
when all of the following hold.

1. The four workspace crates `sc-config`, `sc-transport`, `sc-command` and
   `sc-runtime` MUST each be published to crates.io. `sc-runtime` depends on
   the other three, so it MUST be published after them.
2. Every `Cargo.toml` under `template/` that names one of the four crates
   MUST name it by a crates.io version requirement that the published version
   satisfies. At the tagged commit the template MUST NOT contain a `path =`
   or `git =` dependency on any of the four crates and MUST NOT contain a
   `[patch.crates-io]` entry for any of them.
3. The template CI fixture matrix, which generates, lints and tests one
   project per file in `wizard/fixtures/*.json` and is owned by
   [REQ-RUN-0701](requirements.md), MUST be green at the tagged commit. This
   item adds one thing to it: in this run the generated projects MUST resolve
   the four crates from crates.io, not from the checkout.
4. The commit that satisfies points 2 and 3 MUST carry the git tag `v0.1.0`.
5. `examples/spike`, the throwaway spike that proved the stack before the
   crates existed, MUST be absent at the tagged commit. The obligation to
   delete it is owned by [REQ-RUN-0103](requirements.md); this point and
   criterion 5 only check the result at the tag.

The version number of each published crate is not constrained by this
requirement; each crate carries its own version.

### Rationale

A generated project must depend on the four crates by crates.io version
([REQ-RUN-0003](requirements.md)), because a path dependency copied into each
project can never be upgraded. Until the crates are published that rule can
only be met through a patch entry or a git tag standing in. The release is
the point at which generated projects become upgradeable by a normal
`cargo update`.

### Success Criteria

1. `cargo info sc-config`, `cargo info sc-transport`, `cargo info sc-command`
   and `cargo info sc-runtime` each report a version published on crates.io.
2. `git tag -l v0.1.0` prints `v0.1.0`.
3. At the tagged commit, `grep -rn "path *=\|git *=\|patch.crates-io"
   template --include='Cargo.toml*'` prints no line that refers to
   `sc-config`, `sc-transport`, `sc-command` or `sc-runtime`.
4. The fixture-matrix workflow run for the tagged commit is green, and the
   `Cargo.lock` of a project generated in that run lists the four crates with
   source `registry+https://github.com/rust-lang/crates.io-index`.
5. `git ls-tree -d v0.1.0 examples/spike` prints nothing.

---

## NFR-RUN-0001: A CLI never links server-side dependencies

**Status:** Active  

### Requirement Statement

This item is the single owner of the rule that a command-line client does not
depend on the daemon's stack. Other items that touch the rule reference this
one and use its criteria.

A command-line client built on these crates MUST NOT have `axum`, `rmcp` or
`sqlx` anywhere in its normal dependency graph, directly or transitively.
"Normal" means the edges `cargo tree -e normal` follows: dev-dependencies and
build-dependencies are outside the rule, so a test inside the `cli` crate may
use server-side crates. The rule applies to the `cli` crate of every
generated project (`crates/cli` in the generated workspace).

The rule is defined for a CLI built as its own cargo selection:
`cargo build -p <cli package>`. In that build cargo resolves features for
the CLI's own dependency graph only.

To make that possible:

- `sc-transport` and `sc-command` MUST keep their server-side dependencies
  (`axum` for both, and also `rmcp` for `sc-command`) behind a cargo feature
  named `server`, and `server` MUST be off by default. The feature name comes
  from the sc-runtime design; that it is off by default is decided in this
  document. The per-crate statement of this, the allowed third-party
  dependencies and the criteria on the two crates are owned by
  [NFR-TRN-0001](sc-transport/requirements.md) and
  [NFR-CMD-0001](sc-command/requirements.md).
- The generated `cli` crate MUST depend only on `api-types`, `sc-transport`,
  `sc-command` and `sc-config` among workspace and sc-runtime crates. It MUST
  NOT enable the `server` feature of `sc-transport` or `sc-command`.
- The generated `cli` crate MUST NOT have a normal dependency on the
  `sc-runtime` crate, the `service` crate, the `daemon` crate or any
  `store-*` crate.

**OPEN:** cargo unifies features across all packages selected for one build.
Under `cargo build --workspace` (or `cargo test --workspace`) in a generated
project, `daemon` enables `server` on `sc-transport` and `sc-command` through
`sc-runtime`, those two crates are compiled once with `server` on, and the
`cli` binary of that build is linked against them. Whether workspace-wide
builds are exempt from this rule, or the rule must hold for them too and
`sc-transport` and `sc-command` must therefore be split into client and
server crates in place of the feature, is not decided. The spike MUST measure
it: build a two-package workspace both ways and record whether the `cli`
binary of the workspace-wide build links `axum`. The same OPEN is recorded in
[ADR-RUN-0003](architecture.md).  
**OPEN:** the recipe or command by which a generated project builds and
releases its `cli` binary (so that it is built with `-p`) is not decided; the
generated `Justfile` of v0.1 has no build recipe.

### Rationale

The CLI is a thin HTTP client of the daemon. Linking the daemon's stack would
multiply its build time and binary size. Having `sqlx` available in the CLI
would also make a direct-database shortcut possible, and the daemon must stay
the only process that opens the local database.

### Success Criteria

`<cli package>` is the package name of the generated `crates/cli`.

1. In a project generated from every file in `wizard/fixtures/*.json`,
   `cargo tree -p <cli package> -e normal --prefix none` prints no line
   beginning with `axum ` (the name, then a space), none beginning with
   `rmcp `, none beginning with `sqlx `, and none beginning with
   `sc-runtime `. The resolved tree is the evidence; a manifest that merely
   looks right is not.
2. In the same projects, `cargo build -p <cli package> --release` exits 0,
   and `cargo tree -p <cli package> -e normal --prefix none -f '{p} {f}'`
   prints the lines for `sc-transport` and `sc-command` with no `server` in
   their feature lists.
3. The generated `crates/cli/Cargo.toml` names `sc-transport` and
   `sc-command` without `features = ["server"]`, and names none of
   `sc-runtime`, `service`, `daemon`, `store-sqlite`, `store-postgres` under
   `[dependencies]`.
4. The default-feature trees of `sc-transport` and `sc-command` in this
   repository are checked by the criteria of NFR-TRN-0001 and NFR-CMD-0001,
   in the same `cargo tree` form as criterion 1; they are not repeated here.
5. Once the workspace-wide-build question above is decided: if such builds
   are not exempt, a criterion is added that inspects the `cli` binary
   produced by `cargo build --workspace`; if they are exempt, the exemption
   is stated in the Requirement Statement.

---

## NFR-RUN-0002: Async end to end

**Status:** Active  

### Requirement Statement

Every step on the path of a request through a daemon built with these crates
MUST be asynchronous on the Tokio runtime:

| Step | Where it lives in a generated project | Obligation |
|---|---|---|
| REST handler | `crates/daemon/src/routes.rs` | MUST be an `async fn` |
| MCP tool | `crates/daemon/src/mcp.rs` | MUST be an `async fn` |
| service function | `crates/service` | MUST be an `async fn` |
| store query function | `crates/store-sqlite`, `crates/store-postgres` | MUST be an `async fn` that awaits an `sqlx::Pool` |

Code on a request path MUST NOT block a Tokio runtime thread. It MUST NOT
call `block_on`, `std::thread::sleep`, blocking `std::fs` or `std::net` I/O,
or `reqwest::blocking`. The sc-runtime design states only that handler,
service function and sqlx pool are async; this list of prohibited calls is
decided in this document; it is not stated by the sc-runtime design.

The same rule holds for the request path inside the library crates: the code
of `sc-runtime` and `sc-transport` that runs between accepting a connection
and writing its response, and the client `sc_transport::Client`, whose
request methods MUST be `async fn`. Extending the rule to the library crates
is decided in this document; it is not stated by the sc-runtime design.

Bind-time and shutdown-time file operations in the library crates are outside
the request path and are not restricted by this item. Those are the
operations that run once, before the first connection is accepted or after
the last one is closed: removing a stale socket file and setting the socket
file's permissions when `sc-transport` binds
([REQ-TRN-0004](sc-transport/requirements.md)), taking `daemon.lock`, and
removing the socket file during the `sc-runtime` shutdown sequence
([REQ-RT-0005](sc-runtime/requirements.md)). They MAY use `std::fs`.

`sc-config` is synchronous by design and is not on a request path: a process
loads its configuration once at start-up. The decision that everything else
is async on Tokio, with `sc-config` as the exception, is recorded in
[ADR-RUN-0007](architecture.md).

### Rationale

A Tokio worker thread serves many requests. One blocking call in a handler or
a service function stalls every other request scheduled on that worker, and
the symptom (latency spikes under load) is hard to trace back to its cause.

### Success Criteria

1. Inspection of the template: every function in `crates/daemon/src/routes.rs`
   and `crates/daemon/src/mcp.rs` that serves a request, every public function
   in `crates/service`, and every public query function in the store crate is
   declared `async fn`.
2. `grep -rn "block_on\|thread::sleep\|reqwest::blocking" crates/*/src
   template` prints no line in non-test code.
3. Review of the same directories finds no `std::fs` or `std::net` call
   reachable from a request handler.
4. Library request paths: `grep -rn "std::fs\|std::net\|block_on\|thread::sleep"
   crates/sc-transport/src crates/sc-runtime/src` is reviewed, and every
   match outside `#[cfg(test)]` code is in a function that runs only at bind
   time, at start-up or during shutdown; none is reachable from the code that
   serves an accepted connection or from a `sc_transport::Client` request
   method.
5. Every public request method of `sc_transport::Client` is declared
   `async fn`.

---

## NFR-RUN-0003: Standard crates, used the documented way

**Status:** Active  

### Requirement Statement

Every building block of the four library crates and of the template MUST be a
standard, widely used crate, used the way its own documentation shows. The
chosen crates are:

| Job | Crate |
|---|---|
| HTTP server and router | `axum` 0.8 |
| MCP server | `rmcp` 3.x (`StreamableHttpService`, `#[tool]`, `#[tool_router]`) |
| OpenAPI | `utoipa` and `utoipa-axum` (`OpenApiRouter`, `routes!`) |
| SQL | `sqlx` 0.9 |
| CLI parsing | `clap` derive |
| HTTP client | `reqwest` 0.13 |
| template rendering | `cargo-generate` |

The repository MUST NOT contain any of the following, in `crates/` or in
`template/`:

- a command or operation registry;
- a macro system of its own: no crate with `proc-macro = true`, and no
  `macro_rules!` macro that defines operations, routes or tools;
- a code generator of its own, including a generated Rust HTTP client;
- a portable query layer, meaning any abstraction that lets one query run on
  more than one database backend;
- a wrapper type around an `axum`, `rmcp` or `sqlx` type in a public API. The
  project passes `sc_runtime::Daemon::builder()` an ordinary
  `utoipa_axum::OpenApiRouter` and an ordinary `rmcp` service.

### Rationale

Each item in the prohibited list would be code this project has to maintain
and every generated project has to learn, where a standard crate already does
the job and is already documented. Standard designs are preferred over
hand-rolled pieces throughout. The decision that standard designs are
preferred, and that this item and [NFR-RUN-0004](requirements.md) enforce it,
is recorded in [ADR-RUN-0008](architecture.md).

### Success Criteria

1. `grep -rn "proc-macro" --include=Cargo.toml crates template` prints
   nothing.
2. The `arch-qa` architecture review checks the public API of each of the
   four crates against [ADR-RUN-0001](architecture.md), which states that the
   framework builds the parts and the project wires them in ordinary Rust. It
   reports no registry type, no generator and no public newtype or trait
   whose only purpose is to hide an `axum`, `rmcp` or `sqlx` type.
3. Inspection of the template: the example routes are registered with
   `utoipa_axum::routes!` on an `OpenApiRouter`, the example MCP tools use
   `#[tool]` and `#[tool_router]`, and the CLI uses `clap` derive.
4. `grep -rn "macro_rules!" crates/*/src template` prints nothing, or every
   match is reviewed and found not to define operations, routes or tools.

---

## NFR-RUN-0004: Requirements must mean less code

**Status:** Active  

### Requirement Statement

This is a rule for what may be added to the sc-runtime requirements,
architecture and sprint plans.

A requirement or a crate boundary MUST be admitted only if it results in less
code to write across the projects that use sc-runtime. A requirement that can
only be met by writing custom code or an adapter, where a standard tool or
crate already does the job, MUST be rejected or rewritten to use that tool.

sc-runtime standardises build, test, reporting and runtime, which are the
same in every project. It MUST NOT add requirements at the application layer,
where projects legitimately differ.

A sprint deliverable MUST NOT introduce custom code for a job that a standard
tool already does. Examples of jobs already covered by standard tools here:
template rendering (`cargo-generate`), agent-document rendering
(`sc-compose`), the options UI (Wyvern), OpenAPI generation (`utoipa`),
MCP serving (`rmcp`), lint and `just` infrastructure (sc-lint).

### Rationale

The purpose of sc-runtime is to remove hand-assembled code from projects.
A framework that grows its own requirements, adapters and tooling moves that
work into the framework and makes every project learn it. This test is
applied to every proposed addition, including additions to this file. The
decision is recorded in [ADR-RUN-0008](architecture.md), which names this
item and [NFR-RUN-0003](requirements.md) as its enforcement.

### Success Criteria

1. Review of each sprint plan: every deliverable that adds code names the
   standard tool that was considered and why it does not cover the job. A
   deliverable with custom code where a standard tool exists is recorded as a
   finding.
2. Review of each new or changed requirement in the `docs/` requirements
   files: the reviewer can state what code the requirement saves a project
   from writing. A requirement for which no such statement can be made is
   recorded as a finding.

---

## NFR-RUN-0005: Isolated, parallel tests; no fixed paths

**Status:** Active  

### Requirement Statement

Non-test code in `crates/` and in `template/` MUST NOT contain a literal
temporary-directory path (such as `/tmp`) or a literal home-directory path
(such as `/Users/<name>`, `/home/<name>` or `C:\Users\<name>`).

Every test that writes to the filesystem MUST create its own temporary
directory and write only inside it. Every test that starts a daemon MUST
also use that temporary directory as the daemon's instance root. The
instance root is the directory that holds a daemon's `daemon.lock` and
`daemon.sock`. `<instance-root>` is the per-application, per-user directory
resolved by sc-transport, or an explicitly supplied path; its default
location is undecided ([REQ-TRN-0002](sc-transport/requirements.md)). A test
MUST NOT use the instance root of another test or of a daemon the developer
is really running. Tests that start no daemon, such as the `sc-config` tests,
have no instance root and need only the temporary directory.

Tests that start a daemon SHOULD use `sc_runtime::testing::DaemonFixture`,
which starts a daemon on a fresh temporary instance root for one test.

Every test MUST pass when run in parallel with every other test in the same
suite, and when two copies of the whole suite run at the same time on one
machine. A test MUST NOT bind a fixed TCP port.

The sc-runtime design states that tests are isolated and parallel and that
`DaemonFixture` gives each test its own instance root. The specific rules
here are decided in this document: no literal temporary-directory or
home-directory path in non-test code, no fixed TCP port, and two copies of
the whole suite passing at the same time on one machine.

### Rationale

A fixed path or a shared instance root makes tests interfere with each other
and with a developer's real daemon, whose singleton `daemon.lock` would make
the test daemon refuse to start. `DaemonFixture` exists so that isolation is
the easy path and test suites stay parallel and fast.

### Success Criteria

1. `just test` in this repository passes with the default parallel test
   runner (no `--test-threads=1` in any recipe). `just test` is
   `cargo test --workspace` plus the per-crate default-feature and `server`
   feature runs defined by [REQ-RUN-0004](requirements.md).
2. Two simultaneous `just test` runs from two checkouts on the same machine
   both pass.
3. `just test` in a generated project passes while a daemon of the same
   application is running under the developer's normal instance root.
4. `grep -rn '"/tmp\|/Users/\|/home/\|C:\\\\Users' crates/*/src template`
   prints no line in non-test code.
5. Inspection: every test that starts a daemon obtains its instance root from
   a temporary-directory API or from `DaemonFixture`.

---

## NFR-RUN-0006: The template carries no lint knowledge

**Status:** Active  

### Requirement Statement

The `cargo-generate` template under `template/` MUST NOT contain:

- lint configuration of its own;
- crate-boundary rules or boundary manifests;
- `just` modules or imported `just` files;
- tooling that checks that the REST, MCP and CLI surfaces stay in step, such
  as snapshots of `openapi.json`, of the MCP `tools/list` result or of the
  clap command model.

The one `just` file the template MAY contain is the minimal placeholder
`Justfile` with the recipes `lint`, `test` and `db-prepare`. Its contents and
the criteria on its recipes are owned by [REQ-RUN-0307](requirements.md);
this item only forbids it from carrying `just` modules or imports.

All of the excluded infrastructure is owned by the sc-lint project. sc-lint
installs it into a generated project from the same answers JSON the project
was generated from, and its installer later replaces the placeholder
`Justfile` without changing the recipe names.

This requirement restricts `template/` only. It does not restrict how the
sc-runtime repository lints itself, which includes boundary manifests under
`boundaries/<crate>/` enforced through `just lint`
([REQ-RUN-0005](requirements.md)).

The concrete file list in Success Criteria 1 is decided in this document.

### Rationale

If the template carried lint rules, every generated project would freeze the
rules of its birth date, and every change to the SC lint standard would need
a template change here. With sc-lint as the owner, generated projects pick up
rule changes from sc-lint and nothing in this repository changes.

### Success Criteria

1. `find template` lists none of: `clippy.toml`, `.clippy.toml`,
   `rustfmt.toml`, `.rustfmt.toml`, `deny.toml`, a `boundaries` directory,
   any `*.just` file.
2. `grep -rn "^\[lints\|^\[workspace.lints" template` prints nothing.
3. `template/Justfile` contains no `mod` or `import` statement. (Which
   recipes it defines is checked by the criteria of REQ-RUN-0307.)
4. `template/` contains no snapshot file of `openapi.json`, of MCP
   `tools/list` or of the clap command model, and no test that compares
   against one.

---

## NFR-RUN-0007: rmcp is pinned to a minor version

**Status:** Active  

### Requirement Statement

Every `Cargo.toml` in this repository that names the `rmcp` crate as a
dependency MUST pin it to one minor version series. This covers the manifests
under `crates/`, the manifests under `template/` that are rendered into
generated projects, and the manifest of the throwaway spike under
`examples/spike` for as long as that directory exists.

Pinned to one minor version series means the version requirement admits
patch releases of a single `MAJOR.MINOR` only. `~3.4` and
`>=3.4.0, <3.5.0` are pinned in this sense. `"3"`, `"3.4"` (which Cargo reads
as `^3.4` and which admits 3.5) and `"*"` are not.

All manifests MUST pin the same minor version series.

Moving to a new `rmcp` minor version MUST be a deliberate change to those
manifests, validated by the template CI fixture matrix, which generates a
project per answers fixture and runs `just lint` and `just test` in it.

**OPEN:** which `rmcp` minor version is pinned is not decided; the throwaway
spike under `examples/spike` records the exact version pins. Only the major
version is fixed so far: `rmcp` 3.x, with 3.4.0 current on 2026-09-18.

### Rationale

`rmcp` shipped five versions in the 30 days to 2026-09-15. A generated
project with an open-ended requirement would pick up a new minor version on
its first build and could break at random, on a day nobody changed anything.
A pin turns every upgrade into a reviewed change that CI has tested.

### Success Criteria

1. `grep -rn "rmcp" --include='Cargo.toml*' crates template examples` lists
   every manifest that names `rmcp` (a missing `examples` directory is not a
   failure). In each, the version requirement is a tilde
   requirement on `MAJOR.MINOR` or an equivalent bounded range.
2. All of those manifests show the same `MAJOR.MINOR`.
3. A pull request that changes the pinned minor version runs the template CI
   fixture matrix, and the matrix result is visible on that pull request.

---

## NFR-RUN-0008: File size limit

**Status:** Active  

### Requirement Statement

This item is decided in this document; not stated by the sc-runtime design.

No non-test source file in this repository may exceed 1000 lines. Lines are
physical lines as `wc -l` counts them, including blank lines and comments.

The rule MUST apply at least to every `.rs` file under `crates/*/src/` and to
every `.rs` file under `template/`. Files under a `tests/` directory are test
files and are exempt.

`just lint` at the repository root MUST perform this check and MUST fail,
naming the file and its line count, when a file exceeds the limit. The check
MUST be made with standard tools already on a developer machine (`find`,
`wc`, `awk`, as in Success Criteria 1) inside the `lint` recipe. It MUST NOT
be a program written for this repository.

**OPEN:** the written source of the 1000-line rule is not identified. It is
applied across SC projects as a standing architectural rule, but no document
that states it has been named; until one is, this item is the only statement
of the rule for this repository.  
**OPEN:** whether the limit also applies to non-Rust source (for example
`scripts/new_project.py` and the wizard's HTML and JavaScript) is not
decided.  
**OPEN:** whether a Rust file that holds only a `#[cfg(test)]` module inside
`src/` counts as a test file is not decided.

### Rationale

The limit is a rule applied across SC projects and is adopted here by this
document. A file that large is a
module boundary that was not drawn: it is slow to review, and agents working
on it spend their context on code unrelated to the change.

### Success Criteria

1. `find crates/*/src template -name '*.rs' -not -path '*/tests/*' -exec wc
   -l {} + | awk '$2 != "total" && $1 > 1000'` prints nothing.
2. `just lint` on an unmodified checkout exits 0.
3. Adding a 1001-line `.rs` file under `crates/sc-config/src/` makes
   `just lint` exit non-zero with a message that names that file.

---

## NFR-RUN-0009: Library errors are values

**Status:** Active  

### Requirement Statement

This item is the single owner of the error rule and the panic rule for the
public API of the four library crates `sc-config`, `sc-transport`,
`sc-command` and `sc-runtime`. Each crate's own non-functional requirement
references this item and adds only strictness specific to that crate. The
decision is recorded in [ADR-RUN-0006](architecture.md).

The sc-runtime design states the rule ("discriminated union; no panics") for
`sc-config` only. Extending it to `sc-transport`, `sc-command` and
`sc-runtime`, the rule of one error enum per crate, the ban on opaque error
types, and the list of banned constructs below are decided in this document.

**Panic rule.** No public function or method may panic because of the input a
caller passes or because of the state of the environment. Environment state
includes a missing or unreadable file, malformed JSON, an unset or malformed
environment variable, an endpoint nobody is listening on, a lock already held
by another process, and a failed bind.

Code reachable from a public function MUST NOT use any of: `unwrap`,
`expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, panicking `[]`
indexing. There is no exception for a use that carries a comment saying why
it cannot fail. Test code is exempt.

**Error rule.** Every public operation that can fail with an error the crate
itself originates MUST return `Result<T, E>`, where `E` is a typed error enum
owned by that crate, with one enum per crate:

| Crate | Error enum |
|---|---|
| `sc-config` | `ConfigError` |
| `sc-transport` | `TransportError` |
| `sc-runtime` | `RuntimeError` |
| `sc-command` | undecided; see the OPEN below |

A public API MUST NOT return an opaque error type such as `anyhow::Error` or
`Box<dyn std::error::Error>`.

Recorded exception: a signature dictated by a third-party trait or macro
contract keeps the types that contract requires. There are two.
`sc_command::IntoMcp::into_mcp` returns
`Result<CallToolResult, McpError>`, where `McpError` is `rmcp`'s error type,
because an `rmcp` `#[tool]` function must return that type. The
`axum::response::IntoResponse` implementation for `Envelope<T>` is
infallible, because `into_response` returns a `Response`, not a `Result`.
Neither is an error the crate itself originates.

**Errors returned by a service function.** When a service function returns an
error and that error crosses a process boundary (daemon to CLI, daemon to MCP
client), it MUST be an `OpError` from `sc-command` inside the response
envelope, carrying a stable `code` and the other keys defined by
[REQ-CMD-0002](sc-command/requirements.md). The criteria for this paragraph
are those of [REQ-RUN-0203](requirements.md) (REST, MCP and CLI `--json`
return the same envelope); they are not repeated here.

**OPEN:** whether `sc-command` needs an error enum of its own is not decided.
Its one fallible public function known so far is `into_mcp`, which is covered
by the exception above.  
**OPEN:** whether a response the framework produces before any service
function runs is an envelope is not decided
([REQ-RUN-0203](requirements.md) owns the question). Such responses are: an
`axum` response for an unknown route, an `axum` rejection of a body that
fails to deserialise, and an `rmcp` protocol error, which reaches the MCP
client as `Err(McpError)`. Nothing in this item asserts that they are
`OpError` values.

### Rationale

These crates run inside long-lived daemons and inside CLIs driven by agents.
A panic in a daemon is an outage. A panic in a CLI is a failure no agent can
parse or recover from, where a typed error with a code and a suggested action
lets it branch. One owner for the rule keeps the four crates from drifting
into four different lists of banned constructs.

### Success Criteria

1. `grep -rn "unwrap()\|\.expect(\|panic!\|unreachable!\|todo!\|unimplemented!"
   crates/*/src` prints no line outside `#[cfg(test)]` code, or each
   remaining match is shown by review to be unreachable from every public
   function. A comment on the line is not such a showing.
2. Review of `crates/*/src` finds no `[]` index expression on a slice, `Vec`,
   string or map that is reachable from a public function and can panic;
   `get` or pattern matching is used.
3. Inspection of the public API of `sc-config`, `sc-transport` and
   `sc-runtime`: every fallible public function returns `Result<_, E>` with
   `E` equal to `ConfigError`, `TransportError` and `RuntimeError`
   respectively.
4. Inspection of the public API of `sc-command`: the only public signatures
   that name an error type not defined in `sc-command` are
   `IntoMcp::into_mcp` (returning `McpError`) and the infallible
   `IntoResponse` implementation. Once the `sc-command` enum question above
   is decided: any other fallible public function returns that enum, or none
   exists.
5. In all four crates no public signature names `anyhow::Error` or
   `Box<dyn Error>`
   (`grep -rn "anyhow::Error\|Box<dyn .*Error" crates/*/src` prints no line
   in a `pub` signature).
6. Each crate has tests that pass bad input or a bad environment to its
   public functions (at least: missing file, malformed JSON, unreachable
   endpoint, lock already held, as applicable to the crate) and assert the
   specific `Err` variant returned.
7. Each crate's non-functional requirement on errors and panics
   ([NFR-CFG-0001](sc-config/requirements.md),
   [NFR-TRN-0004](sc-transport/requirements.md),
   [NFR-CMD-0003](sc-command/requirements.md),
   [NFR-RT-0004](sc-runtime/requirements.md)) references this item and does
   not restate a different list of banned constructs or a weaker rule.
