# sc-runtime Requirements

**ID Range:** REQ-RUN-0001 through REQ-RUN-0702; NFR-RUN-0001 through NFR-RUN-0010  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

**Authority (2026-11-11).** [PRD v2.0](prd/sc-runtime-prd.md) governs this document. Where a requirement below disagrees with the PRD, the PRD is right. Requirements changed to match it carry the date 2026-11-11 and a PRD id.

Repo-level requirements for `sc-runtime`. They were extracted from
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
| `sc-cron` | [requirements](sc-cron/requirements.md) | [architecture](sc-cron/architecture.md) |
| `sc-cron-commands` (Phase C planned) | [requirements](sc-cron-commands/requirements.md) | [architecture](sc-cron-commands/architecture.md) |

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

**Amended 2026-11-11 (PRD v2.0 §3, §4).** Principle 1 does not mean every project hand-writes the path between an interface and storage. That path is shared: a project supplies its types (`Tcli`, `Tsql`) and its transforms, and wires by choosing where its rules go. Hand-written commands are the interim form (PRD §9).

## ID Ranges

| Range | Area | Scope |
|---|---|---|
| REQ-RUN-0001 through REQ-RUN-0005 | Repository | - |
| REQ-RUN-0101 through REQ-RUN-0103 | Spike: verifying the design's open facts | The design marks several facts as "Verify": they are believed true but nothing may be planned around them until a throwaway spike proves them. These requirements define that spike. |
| REQ-RUN-0201 through REQ-RUN-0206 | End-to-end behaviour | These are properties of the assembled system. No single crate can satisfy one alone, so they close in integration sprints. |
| REQ-RUN-0301 through REQ-RUN-0331 | Template | The template is what a project owns and edits after generation. It is kept thin: it contains the example operation and the wiring, and nothing that should improve across projects. REQ-RUN-0313 through REQ-RUN-0325 are the generated store adaptor. REQ-RUN-0326 is the database layout of a new application. REQ-RUN-0327 and REQ-RUN-0328 are schema and trait pairing. REQ-RUN-0329 is the crates.io package name. REQ-RUN-0330 places row structs in the schema crate. REQ-RUN-0331 is the general database command surface. |
| REQ-RUN-0401 through REQ-RUN-0403 | Answers contract | - |
| REQ-RUN-0501 through REQ-RUN-0503 | Driver | - |
| REQ-RUN-0601 through REQ-RUN-0603 | Wizard | - |
| REQ-RUN-0701 through REQ-RUN-0702 | Release | - |
| NFR-RUN-0001 through NFR-RUN-0010 | Non-functional requirements | - |

## Out of scope for v0.1

Recorded so a sprint plan leaves them out deliberately. Each would add code, so
each waits until a project asks for it.

| Item | Status |
|---|---|
| Database layouts other than SQLite-only | accepted when that layout's store crate exists ([REQ-RUN-0326](requirements.md)); v0.1 generates SQLite only |
| `store-mysql` | when the first project needs it |
| SQL Server | never; sqlx has no driver |
| SQLite batched write actor | required: the atm-core writer is the SQLite implementation of `Storage<T>` (PRD-STO-3). The earlier "forbidden" entry is withdrawn, 2026-11-11 |
| Library rusqlite writer `sc-runtime-db-sqlite` | the atm-core writer behind `Storage<T>` (PRD-STO-3). The "not a template dependency" rule is withdrawn, 2026-11-11 |
| `sc-cron` | cron's table is a row struct stored through `Storage<CronRow>` in the application's database (PRD §6), 2026-11-11 |
| `tracing` bridge into `sc-observability` | undecided |
| Bearer token for TCP; Origin and Host checks on `/mcp` | ideas only; relevant only on TCP |
| Surface snapshots and adapter-drift tooling | owned by sc-lint |
| `sc-lint create` driver step | added when sc-lint ships that command |
| `sc-config` reload, change notification, async interop | later, possibly `sc-config-tokio` |
| stdio MCP clients | a stdio-to-HTTP shim is a later option |
| A virtual machine definition, colima configuration or provisioning scripts for daemon-process tests | not built in v0.1, in this repository or in the template; the safety rule remains in [NFR-RUN-0010](requirements.md), while generated-project wording is deferred with [REQ-RUN-0311](requirements.md) |
| The synchronization adaptor crate of a two-store layout | required (owner, 2026-11-12: "The synchronization adapter will be necessary in early products."); bound by REQ-RUN-0301 and REQ-RUN-0326 |
| Synchronization algorithm (forwarding, replication, an outbox, a routing policy, schedule, direction, conflict policy) | consumer-specific (owner, 2026-11-12: "Synchronization algorithm will always be specific."); no record binds one |
| A raw-SQL layer portable across database backends | never. `Storage<T>` is not one: it carries no SQL (PRD-STO-1, PRD-STO-2), 2026-11-11 |
| Web-application features: HTML templating, sessions, user authentication | never; this is not a web-app framework |
| Adopting an existing scaffold (loco-rs, rust-web-app, axum-postgres-template) | never; none has the daemon plus thin-client split, so they stay pattern references |
| Generating the prototype set of example repositories and handing it to the sc-lint team | first step after the MVP |
| Updating the sc-ai-cli Rust `.j2` templates to match this template | after the MVP, after `store-postgres` |
| Migrating atm-core onto the four crates | stretch goal after the MVP; also when its write actor would be ported back |

---

## REQ-RUN-0001: Repository layout: one workspace and its library crates

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-20  
**Owner:** randlee  

### Requirement Statement

Definitions: A workspace member is a Cargo package selected by the root `Cargo.toml` workspace members list. The member table is the table headed `Member path | Crate name | Purpose`. The path table is the separate table headed `Path | Contents`; it lists repository support paths, not member packages. A path-table entry is one value in the Path column of the path table, read without its trailing `/`; files named only in the Contents column are not path-table entries. A required path-table entry is any path-table entry, except `examples/spike/` at and after the `v0.1.0` tag.

The GitHub repository `randlee/sc-runtime` MUST contain one Cargo workspace,
declared in the root `Cargo.toml`.

The workspace members MUST be exactly the nine packages listed in the member table. Amended 2026-10-25 ([ADR-RUN-0012](architecture.md), [ADR-RUN-0013](architecture.md)): `sc-runtime-db-sqlite` and `sc-cron` are members. Amended 2026-11-11 (PRD-STO-3): `sc-runtime-db-sqlite` holds the atm-core SQLite writer that sits behind `Storage<T>`; the earlier rule that it is not a store the daemon opens and not a template dependency is withdrawn. The generated daemon may depend on `sc-cron` only through the optional `cron` feature fixed by [REQ-RUN-0301](requirements.md); feature-off generated builds do not link it.

| Member path | Crate name | Purpose |
|---|---|---|
| `crates/sc-config` | `sc-config` | JSON config files with environment overrides |
| `crates/sc-transport` | `sc-transport` | UDS/TCP listener and client connector |
| `crates/sc-command` | `sc-command` | response envelope, `OpError`, REST/MCP conversions and shared storage command helpers |
| `crates/sc-runtime` | `sc-runtime` | daemon assembly: `Daemon::builder()`, `daemon.lock`, `DaemonFixture` |
| `crates/sc-runtime-db-sqlite` | `sc-runtime-db-sqlite` | rusqlite write actor; one connection, one thread |
| `crates/sc-cron` | `sc-cron` | optional cron policy over `Storage<CronRow>` in the application database |
| `crates/sc-cron-commands` | `sc-cron-commands` | optional standard cron DTOs and scheduler adapter; ADR-RUN-0019 |
| `crates/sc-storage` | `sc-storage` | standard-library-only asynchronous typed storage contract |
| `crates/sc-monty` | `sc-monty` | single Monty interpreter implementation shared by cron and command transforms |

The Phase D member set is the nine public packages in REQ-RUN-0002.
The cron-command adapter remains optional for consumers.
Its default DTO graph and optional scheduler edge are defined by
REQ-RUN-0332 and NFR-CRNC-0001; no reverse core command/runtime edge is allowed.

| Path | Contents |
|---|---|
| `template/` | the `cargo-generate` template root, including `template/cargo-generate.toml`, `template/AGENTS.md.j2` and `template/CLAUDE.md.j2` |
| `wizard/` | `wizard.json`, `answers.schema.json` (the options contract), `pages/` (wizard HTML pages), `fixtures/` (one answers JSON file per tested variant) |
| `scripts/` | `new_project.py` (the generation driver) and `run_wizard.py` |
| `tests/unit/` | contract tests for the generator: schema validity, key-set match, invalid fixtures failing closed |
| `examples/spike/` | the throwaway spike; present only until the `v0.1.0` tag ([REQ-RUN-0103](requirements.md)) |
| `docs/` | requirements and architecture files; the ADRs are sections of the architecture files (see below) |
| `boundaries/` | one directory per library crate holding its `sc-lint-boundary` manifest ([REQ-RUN-0005](requirements.md)) |
| `Justfile` | the `just` recipes, including `just new <dest>` |
| `.github/workflows/` | crate CI and the template fixture-matrix CI |

Nothing under `wizard/` or `scripts/` may be placed inside `template/`, so
that no generated project ever contains a wizard or driver file. This item
owns that rule; items about the wizard or the driver refer to it.

Decided in this document: ADRs live in the architecture files (`docs/architecture.md` and `docs/<crate>/architecture.md`).

`examples/spike` is an excluded standalone package while it exists. It is not
an additional workspace member and it is not a Cargo example. Root `Cargo.toml`
`exclude` names `examples/spike`. The workspace member list is exactly the
library crates in the table above. Sprint a-5 deleted `crates/sc-command/lint-edge/`
when `crates/sc-runtime` landed. `sc-observability` and `sc-observability-otlp`
are not workspace members. `cargo tree` enforces those edges, because sc-lint
0.8.0 rejects a forbidden edge whose target is absent.

The files in `template/` contain `cargo-generate` placeholders and do not compile until rendered.

**MUST statements:**

- The root `Cargo.toml` of the GitHub repository `randlee/sc-runtime` MUST contain one `[workspace]` table.
- The workspace members MUST be exactly the nine packages listed in the member table.
- The output of `git ls-tree -r -t --name-only HEAD` MUST list each required path-table entry.
- Every string in the `[workspace] members` list of the root `Cargo.toml` MUST start with `crates/`.

**MUST NOT statements:**

- Each directory and file in the path table MUST NOT be a workspace member.
- The `[workspace] members` array in the root `Cargo.toml` MUST NOT contain a string that starts with `template`.
- The repository MUST NOT contain a `docs/adr/` directory.

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

1. Inspect the root `Cargo.toml`: `[workspace] members` is the glob `crates/*`.
   A missing directory is not a member. The resolved Phase D member set is
   exactly the nine library packages in the member table and REQ-RUN-0002.
   `cargo metadata --no-deps --format-version 1` lists those packages and
   no package whose manifest is under a path-table entry.
2. `cargo build --workspace` run at the repository root exits 0 and compiles
   no file under `template/`.
3. Each path in the second table exists at the repository root (with
   `examples/spike/` exempt at and after the `v0.1.0` tag).
4. `find template -path '*wizard*' -o -name 'new_project.py' -o -name
   'run_wizard.py'` prints nothing.
5. `test -d docs/adr` fails (the directory does not exist), and
   `grep -c '^## ADR-' docs/architecture.md` prints a number greater than 0.

---

# Requirement: Each crate is an independent deliverable

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-RUN-0002: Each crate is an independent deliverable

**Status:** Active  

### Requirement Statement

Definitions: A crate's README is the `README.md` file in that crate's directory. A doctest is a Rust code block in documentation that `cargo test --doc` compiles. The repository LICENSE file is the file `LICENSE` at the repository root. The MIT license text is the license text the Open Source Initiative publishes as the MIT License. Tested minimum versions are the lowest dependency versions the project has tested. CLI means command-line interface. JSON means JavaScript Object Notation. API means application programming interface. The pydantic Monty crates are the pydantic interpreter packages, including monty and monty-types. A production dependency is a normal or build dependency; dev-dependencies are excluded. `tokio` is the crates.io asynchronous runtime crate of that name. Cron types are scheduler, schedule-row and cron-policy types; reserved cron names are cron_stop and cron_tick. SDK means software development kit: the public crate set named in this record. Usable without the other SDK crates or generated template means usable as a standalone library selected alone from a clean checkout, including its own default-feature build and tests. The generated template is the project template in the repository's `template/` directory. Registry dependencies are dependencies resolved from crates.io. Interpreter behavior is running scripts through the pydantic Monty crates. Shared storage behavior is storing sc-cron's rows, such as `CronRow`, through sc-storage's `Storage<T>`. A crate consumes another crate for a behavior when it lists that crate as a production dependency and calls it for that behavior. sc-* means every package whose name starts with `sc-`. A description states what a crate provides when it names the crate's main function in one phrase; sc-command's `Lossless JSON CLI/API response envelope and REST/MCP conversions.` is an example.

The Phase D public package set is sc-config, sc-transport, sc-command, sc-runtime, sc-cron, sc-cron-commands, sc-runtime-db-sqlite, sc-storage and sc-monty, released together at 0.3.0.

**MUST statements:**

- When selected alone from a clean checkout, each public crate MUST build with its default features.
- When selected alone from a clean checkout, each public crate MUST pass its own default-feature tests.
- Each public crate MUST provide a README.
- Each public crate's README MUST contain a Rust usage example.
- Each public crate's README Rust usage example MUST compile as a doctest.
- Each public crate's `name` in `cargo metadata --no-deps` output MUST equal that crate's name in the public package set.
- Each public crate's `version` in `cargo metadata --no-deps` output MUST equal the version the public package set is released together at.
- Each public crate's `description` in `cargo metadata --no-deps` output MUST state what that crate provides.
- Each public crate's `license` in `cargo metadata --no-deps` output MUST be `MIT`.
- Each public crate's `repository` in `cargo metadata --no-deps` output MUST be `https://github.com/randlee/sc-runtime`.
- Each public crate's `readme` in `cargo metadata --no-deps` output MUST name that crate's README.
- Each public crate MUST be independently publishable to crates.io.
- Every published crate MUST declare the MIT license.
- The repository LICENSE file MUST contain the MIT license text.
- When selected alone from a clean checkout without the other SDK crates or generated template, `sc-config` MUST pass its own default-feature tests.
- When selected alone from a clean checkout without the other SDK crates or generated template, `sc-transport` MUST build with its default features.
- When selected alone from a clean checkout without the other SDK crates or generated template, `sc-transport` MUST pass its own default-feature tests.
- sc-storage MUST use only the Rust standard library.
- sc-monty production dependencies MUST be limited to pydantic Monty crates.
- sc-cron MUST consume sc-storage for shared storage behavior.
- sc-cron MUST consume sc-monty for interpreter behavior.
- Published registry dependencies MUST declare tested minimum versions.
- Template registry dependencies MUST declare tested minimum versions.

**MUST NOT statements:**

- Published registry dependencies MUST NOT declare an upper bound.
- Template registry dependencies MUST NOT declare an upper bound.
- `crates/sc-command/Cargo.toml` MUST NOT list `sc-runtime` in any dependency table.
- `crates/sc-command/Cargo.toml` MUST NOT list `sc-config` in any dependency table.
- `crates/sc-command/Cargo.toml` MUST NOT list `sc-transport` in any dependency table.
- `crates/sc-command/Cargo.toml` MUST NOT list a path inside the generated template.
- sc-monty MUST NOT have a production dependency on sc-*.
- sc-monty MUST NOT have a production dependency on the `tokio` crate.
- sc-monty MUST NOT have a production dependency on serde_json.
- sc-monty MUST NOT define cron types.
- sc-monty MUST NOT define reserved cron names.

### Rationale

`sc-config` (configuration), `sc-transport` (a local daemon and CLI pair) and
`sc-command` (a JSON-first response envelope) are useful to programs that are
not sc-runtime daemons, so each must stand alone. Independence also means any
crate can move to its own repository later without an API change, and lets a
CLI depend on `sc-transport` and `sc-command` without pulling in the daemon's
dependencies.

### Success Criteria

Normal per-crate tests, README doctests, package metadata inspection and package dry runs verify the public package set. Normal/build dependency graph inspection verifies each crate boundary for its supported features. Pre-publication archive rehearsal qualifies interdependent packages before registry-only proof.

### Implementation Notes

Generated project licensing remains an independent generation option. Consumers may pin dependencies in their own manifests and locks; this does not permit restrictions in SDK/template dependency declarations. sc-sha is permitted for the cron source key.

## REQ-RUN-0003: Generated projects depend on published versions

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

"Generated project" means the Cargo workspace that `cargo generate` renders
from `template/`. The public package set is `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`, `sc-runtime-db-sqlite`, `sc-storage` and `sc-monty` (REQ-RUN-0002).

Definitions: A "crates.io version requirement" is either a `version = "..."` key or the bare-string form `name = "..."`. A `Cargo.toml` depends on a crate when it lists that crate in a dependency table: `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, `[workspace.dependencies]` or a `[target.*]` form of these.

3. This obligation does not apply to a `[patch.crates-io]` table, whose entries are normally `path` or `git` entries. (Path dependencies between the generated project's own crates, such as `daemon` on `service`, are unaffected.)
4. Before a crate's first crates.io publish, and when testing unreleased
   crate changes, the dependency MAY instead be satisfied by a
   `[patch.crates-io]` entry or by a git dependency on a tag of
   `https://github.com/randlee/sc-runtime`. The version requirement in
   obligation 1 stays in place when a `[patch.crates-io]` entry is used. When
   the git form is used, the dependency names the repository URL and a `tag`
   in place of the version requirement.
5. Which generated crate depends on which of the public crates is not stated
   here. It is owned by [REQ-RUN-0301](requirements.md), which holds the
   dependency table of the generated workspace.

Checkout validation MAY satisfy that requirement with the same `[patch.crates-io]` mechanism above; release-mode generation MUST resolve all public crates in its dependency graph from the registry without a path or git override.

**MUST statements:**

- When a `Cargo.toml` rendered into a generated project depends on one of the public crates, it MUST name that crate by a crates.io version requirement.
- When the generated project's `daemon` crate (`crates/daemon/Cargo.toml`) enables its optional `cron` feature, every rendered `Cargo.toml` that depends on `sc-cron` MUST name `sc-cron` by a crates.io version requirement.

**MUST NOT statements:**

- The source of the public crates MUST NOT be copied into a generated project.
- A rendered `Cargo.toml` MUST NOT contain a `path = ...` dependency on any of the public crates in a dependency table (`[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, `[workspace.dependencies]` or a `[target.*]` form of these).
### Rationale

Generated code cannot be upgraded after it is created; a versioned dependency
can, with `cargo update`. A path dependency or a copy of the crate source
inside each project would mean every fix has to be re-applied by hand in
every project. Everyone who owns a generated project is affected.
### Success Criteria

1. For every answers file in `wizard/fixtures/`, render the project and
   inspect every rendered `Cargo.toml`: each dependency on a public crate has a version requirement,
   or, where the git form of obligation 4 is in use, a `git` URL of
   `https://github.com/randlee/sc-runtime` with a `tag`.
2. In the same rendered files, `grep -n 'path *='` finds no line belonging
   to a dependency on any of the public crates outside a `[patch.crates-io]`
   table.
3. The rendered project contains no directory named for any public crate and no copy of its source.
4. A release-mode render with generated feature `cron` names `sc-cron` by
   version, contains no path/git override for any selected public crate, and
   resolves all public crates in its dependency graph from the registry.
### Implementation Notes

**Key Considerations:**

- OPEN: who writes the `[patch.crates-io]` entry in the pre-publish case
  (a template option, the driver `scripts/new_project.py`, or the CI workflow)
  is not decided. The same undecided mechanism is what would let template CI
  build a generated project against the crates of the checkout under test
  ([REQ-RUN-0701](requirements.md)).
- Registry requirements use tested minimum versions without upper bounds (REQ-RUN-0002).

---

## REQ-RUN-0004: Root `justfile` recipes `lint`, `test` and `new`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-20  
**Owner:** randlee  

### Requirement Statement

Definitions: CI means continuous integration. The root `Justfile` is the file named `Justfile` (capital J) at the root of the repository `randlee/sc-runtime`. A recipe is a named command defined in the root `Justfile`. A primary recipe is a recipe defined by its own header line in the root `Justfile`; its name is the name on that line. A just alias is a line of the form `alias <name> := <recipe>`, which gives another name to an existing recipe. The recipe table is the table below. The test command list is the numbered list of six items below; each item names one or more commands that run tests. The static check list is the lettered list of five checks below. The driver is the generation script `scripts/new_project.py`. The crate CI workflow is the GitHub Actions workflow named `Crate CI`, which runs lint and tests for the crates of this repository. The plain-generation job is the job `plain-cargo-generate` in `.github/workflows/template.yml`. A workflow step is one entry under `steps:` of a job in a file under `.github/workflows/`. A lint step is a workflow step outside the plain-generation job whose command lints code. A test step is a workflow step outside the plain-generation job whose command runs tests. A generation step is a workflow step outside the plain-generation job whose command generates a project from `template/`; that project is the step's project. A recipe step is a lint step, a test step or a generation step. A step invokes a program directly when its `run:` text calls that program itself instead of calling a `just` recipe. A lint tool is a program that performs a static check; `sc-lint`, `cargo clippy` and `cargo fmt --check` are lint tools.

The names `lint` and `test` are the recipe names that `sc-lint init --just` writes. The first line of the root `Justfile` stays the sc-lint initialization header.

| Recipe | Behaviour |
|---|---|
| `just lint` | Runs every static check for this repository and exits non-zero if any check fails. Its checks include the `sc-lint-boundary` check ([REQ-RUN-0005](requirements.md)) and the `sc-lint` source file length check, 1000 source lines not counting blank and comment-only lines ([NFR-RUN-0008](requirements.md)). |
| `just test` | Runs every command in the test command list, and exits non-zero if any of them fails. |
| `just new <dest>` | Runs the generation driver `scripts/new_project.py` with `<dest>` as the destination directory of the new project. Any further arguments (for example `--var-file <answers.json>` or `--prefill <json>`) are passed through to the driver unchanged. |

The test command list is specified in this record; the sc-runtime design names only the recipe.

1. `cargo test --workspace --exclude sc-cron` (Monty needs rustc 1.96; see items 2 and 6).
2. `cargo test -p sc-cron` with default features.
3. `cargo test -p sc-config`, `cargo test -p sc-transport`,
   `cargo test -p sc-command` and `cargo test -p sc-runtime`, each with
   default cargo features. These are needed because `cargo test --workspace`
   builds `sc-transport` and `sc-command` once with their `server` feature
   on (`sc-runtime` enables it, and cargo unifies features across the
   packages it builds together), so it never tests their default-feature
   build.
4. `cargo test -p sc-transport --features server` and
   `cargo test -p sc-command --features server`.
5. The generator contract tests under `tests/unit/`.
6. `cargo test -p sc-cron --features monty` on **rustc 1.96** (the `monty`
   1.4 crates require 1.96; workspace MSRV stays 1.94.1 for the other crates).

The static check list is what `just lint` with no profile argument runs, through `.sc-lint/bootstrap.py` and `sc-lint.toml`, as the full profile of sc-lint 0.8.6 or later:

- (a) `sc-lint lint sc-boundary` (the `boundary` lint).
- (b) line-counts, with the key `[line_counts] max_production_lines` set to `1000` for non-test files under `crates/*/src`.
- (c) `cargo fmt --all -- --check`.
- (d) `cargo clippy --workspace --exclude sc-cron --all-targets -- -D warnings`.
- (e) `cargo clippy -p sc-cron --all-targets --features monty -- -D warnings` on **rustc 1.96**.

`just lint gates` is the early profile of (a) plus (b). `template/**/*.rs` is not configured for line-counts in this sprint.

There is exactly one recorded exception to the recipe-step statements: the plain-generation job, required by [REQ-RUN-0306](requirements.md), proves that plain `cargo generate` works without the driver, and therefore runs `cargo generate --path template ...` and `cargo build --workspace` directly.

**MUST statements:**

1. The root `Justfile` MUST define a recipe named `lint`.
2. Running `just lint` MUST behave as the `just lint` row of the recipe table describes.
3. The root `Justfile` MUST define a recipe named `test`.
4. Running `just test` MUST behave as the `just test` row of the recipe table describes.
5. The root `Justfile` MUST define a recipe named `new`.
6. Running `just new <dest>` MUST behave as the `just new <dest>` row of the recipe table describes.
7. Running `just test` MUST run the whole test command list.
8. Each lint step MUST run its checks by calling `just lint`.
9. Each test step MUST run its tests by calling `just test`.
10. Each generation step MUST generate its project by calling `just new`.
11. The crate CI workflow MUST be defined in the file `.github/workflows/crates.yml`.
12. On every pull request, the crate CI workflow MUST run `just lint` at the repository root.
13. On every pull request, the crate CI workflow MUST run `just test` at the repository root.

**MUST NOT statements:**

14. A recipe step MUST NOT invoke `cargo` directly.
15. A recipe step MUST NOT invoke `python` directly.
16. A recipe step MUST NOT invoke a lint tool directly.
17. The root `Justfile` MUST NOT rename the primary recipe `lint`.
18. The root `Justfile` MUST NOT define `lint` as a just alias of a primary recipe with another name.
19. The root `Justfile` MUST NOT rename the primary recipe `test`.
20. The root `Justfile` MUST NOT define `test` as a just alias of a primary recipe with another name.

### Rationale

Every SC repository exposes the same base `just` commands so that agents, CI
and people use one vocabulary and need no per-repository knowledge to lint or
test. Routing CI through the same recipes means a green local `just lint` and
`just test` predicts a green CI run. The names `lint` and `test` stay primary recipe names because they are the names sc-lint writes; an alias or a renamed recipe would break that shared vocabulary.

### Success Criteria

1. Statements 1, 3 and 5: inspect the root `Justfile`; it has a recipe header line for each of `lint`, `test` and `new`, and `just --list` at the repository root shows all three.
2. Statement 2: `just lint` exits 0 on a clean checkout of the default branch; inspect the `lint` recipe body and `sc-lint.toml` to confirm each of checks (a) to (e) runs; introducing a formatting error makes `just lint` exit non-zero.
3. Statements 4 and 7: inspect the `test` recipe body; each of the six items of the test command list appears. `just test` exits 0 on a clean checkout of the default branch.
4. Statement 4: introducing a failing unit test in any workspace crate makes `just test` exit non-zero; introducing a failing test under `tests/unit/` does the same.
5. Statement 7: introducing, in `crates/sc-command`, a failing test gated with `#[cfg(not(feature = "server"))]` makes `just test` exit non-zero, and so does a failing test gated with `#[cfg(feature = "server")]`. This shows both feature configurations are tested.
6. Statement 6: inspect the `new` recipe body; `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` invokes `scripts/new_project.py` with that destination and that `--var-file` argument.
7. Statements 8, 9, 10, 14, 15 and 16: inspect every file in `.github/workflows/`; outside the plain-generation job, each lint step is a `just lint` invocation, each test step is a `just test` invocation, each generation step is a `just new` invocation, and no `run:` text of such a step calls `cargo`, `python` or a lint tool itself.
8. Statement 11: the file `.github/workflows/crates.yml` exists and its `name:` is `Crate CI`.
9. Statements 12 and 13: `.github/workflows/crates.yml` has a `pull_request` trigger and steps running `just lint` and `just test` at the repository root, and a pull request's check list shows a run of that workflow.
10. Statements 17 to 20: inspect the root `Justfile`; `lint` and `test` each appear as the name on a recipe header line, and no line starts with `alias lint :=` or `alias test :=`.

## REQ-RUN-0005: Boundary manifests for every crate

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: The public package set is `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`, `sc-runtime-db-sqlite`, `sc-storage` and `sc-monty`. The crate graph is the Phase D workspace dependency graph shown in the table below; it updates the earlier graph snapshots under REQ-RUN-0002 and ADR-RUN-0020/0021. A crate's additional dependency list is the list owned by its per-crate requirement named below. The crate graph table is the Crate graph table below. Its edges include allowed and forbidden edges with their feature and phase conditions. A crate's row is that crate's row of the crate graph table. The manifest format is the TOML key set listed below. The sc-compose crates are the packages `sc-compose` and `sc-compose-*`, and their Rust library names such as `sc_compose`. ATM tool crates are agent-team-mail and its atm-* tool packages and Rust imports. CLI means command-line interface. TOML is the Cargo manifest data format. ADR means architecture decision record. A boundary manifest is one or more TOML files under boundaries/<crate>/ read by sc-lint-boundary, where <crate> is the workspace crate name. ATM means agent-team-mail; its tool crates and imports are the exclusions named here. The additional dependency owners are: sc-config—NFR-CFG-0002; sc-transport—NFR-TRN-0001; sc-command—NFR-CMD-0001; sc-runtime—NFR-RT-0003; sc-cron—NFR-CRON-0001; sc-runtime-db-sqlite—REQ-DBSQ-0001; sc-cron-commands—NFR-CRNC-0001; sc-storage and sc-monty—REQ-RUN-0002. These are release boundaries, not permanent restrictions on future releases. An allowed dependent is a workspace crate permitted to depend on this crate. An allowed dependency is a package this crate may depend on. A forbidden edge is a prohibited dependency from the owner package to a target package. The facade is the set of Rust items exported by the crate. A manifest violation is a dependency or public export that fails the rules encoded in that manifest. The crate dependency table in ADR-RUN-0003 is the crate table in the Decision section of ADR-RUN-0003 in `docs/architecture.md`, titled "Crate dependency graph and the server cargo feature". The crate graph in `docs/architecture.md` is the crate table under that file's `## Crate graph` heading. `just` is the command runner that runs recipes from the root `Justfile`; `just lint` runs its `lint` recipe. `sc-lint` is this repository's static check tool, and `sc-lint lint sc-boundary` runs its boundary check, which reads the boundary manifests and reports each manifest violation.

The manifest format, as observed in the sc-lint repository, is TOML with these keys: `boundary_id`, `owner_package`, a `[public]` table with `facade`, an `[implementation]` table, a `[composition]` table with `roots`, a `[dependencies]` table with `allowed_dependents`, `allowed_dependencies` and `forbidden_edges` (each forbidden edge a string of the form `"a -> b"`), and a `[references]` table.

Crate graph table (Phase D scope):

| Crate | Workspace crates it depends on | Forbidden edges (`forbidden_edges`) |
   |---|---|---|
   | `sc-config` | none | `sc-transport`, `sc-command`, `sc-runtime`, `sc-runtime-db-sqlite`, `sc-cron`, `sc-observability`, `sc-observability-otlp`, `sc-observability-types`, `tokio` (Phase D only) |
   | `sc-transport` | none | `sc-config`, `sc-command`, `sc-runtime`, `sc-runtime-db-sqlite`, `sc-cron`, `sc-cron-commands`, `sc-storage`, `sc-monty`, `sc-observability`, `sc-observability-otlp`, `sc-observability-types` |
   | `sc-command` | `sc-storage` for the shared storage command helpers (ADR-RUN-0021); OPEN for owner decision: the feature selection carrying this edge and any sibling-free envelope selection (NFR-CMD-0002) | `sc-config`, `sc-transport`, `sc-runtime`, `sc-runtime-db-sqlite`, `sc-cron`, `sc-observability`, `sc-observability-otlp` |
   | `sc-runtime` | `sc-transport` with its `server` cargo feature, `sc-command` with its `server` cargo feature, and `sc-observability-types` for the injected canonical logger contract. The edge `sc-runtime` -> `sc-config` is absent | `sc-runtime-db-sqlite`, `sc-cron`, `sqlx` (cargo tree; not a workspace member), `sc-observability`, `sc-observability-otlp` |
| `sc-runtime-db-sqlite` | `sc-storage` for `Storage<T>` | `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`. `sqlx` is absent and is checked with `cargo tree`, because it is not a workspace member |
| `sc-cron` | `sc-storage`; optional `sc-monty` for interpreter execution; temporary defaults-off/no-features `sc-command` for the contract (ADR-RUN-0017). Canonical event values use registry `sc-observability-types` | `sc-config`, `sc-transport`, `sc-runtime`, `sc-runtime-db-sqlite`, `sc-observability`, `sc-observability-otlp`. `sqlx` is absent and is checked with `cargo tree` |
| `sc-cron-commands` | defaults-off `sc-command`; `sc-storage` for the shared command storage adapter; optional `sc-cron` under non-default scheduler feature | `sc-config`, `sc-transport`, `sc-runtime`, `sc-runtime-db-sqlite`, full observability/OTLP and production tool crates |
| `sc-storage` | none; Rust standard library only | every other workspace crate and every external crate |
| `sc-monty` | none; production dependencies limited to pydantic Monty crates | every `sc-*` production dependency, Tokio and serde_json |

Phase C also forbids sc-command→sc-cron-commands and
sc-runtime→sc-cron-commands. The adapter is an allowed dependent of command
and, only with scheduler enabled, cron. These planned edges are implemented
by c-1 boundary manifests and checked by c-6/c-4 isolated graph tests.

`sc-observability-types` is an allowed dependency of `sc-command`, `sc-runtime`, and `sc-cron`; the latter two import the temporary command contract and direct canonical values under ADR-RUN-0017. The full `sc-observability` runtime remains forbidden in every library crate. Boundary lint rejects production edges/imports; companion production cargo-tree checks cover renamed or transitive external packages that the workspace-only boundary tool cannot inspect. The rule applies to every supported production feature combination and to normal/build dependency edges. Dev-only test and benchmark references are permitted. Invoking either external CLI through an application-configured host adapter remains permitted and does not link its Rust crate. Package identity, rather than dependency-key spelling, governs this rule: known tool packages include `sc-compose`, `sc-compose-*`, `agent-team-mail`, `agent-team-mail-core`, and `atm-*`; imports include the corresponding Rust library names such as `sc_compose` and `atm_core`. Renaming a dependency must not bypass the production exclusion. Test fixtures exercise those names and renamed equivalents instead of relying only on an `atm` prefix match.

The table lists the full forbidden-edge policy. Manifest `forbidden_edges`
list only workspace-member targets in that lint tree; crates.io names use
criterion 5 `cargo tree` checks. sc-lint 0.8.0 cannot fail a crates.io edge,
so b-13's types-only rule is measured with direct-depth `cargo tree` checks
for `sc-runtime` and `sc-cron`; each may print `sc-observability-types ` but
must print no direct `sc-observability ` or `sc-observability-otlp ` line.
`allowed_dependents` records the reverse workspace edges of the table for the supported features. Thus sc-storage admits the listed command, SQLite and cron consumers; sc-monty admits cron when its interpreter feature is selected; command admits runtime, cron and the cron-command adapter. External consumer applications are not workspace dependents. The absent runtime -> config edge remains forbidden. Historical sprint a-1/a-5 evidence records the earlier graph, not a restriction on these Phase D edges.

**MUST statements:**

- For each crate in the public package set, the repository MUST contain a boundary manifest.
- Each boundary manifest MUST use the manifest format.
- Each boundary manifest MUST list the items its crate exports in the `facade` key of its `[public]` table.
- Each boundary manifest MUST list its crate's allowed dependents in the `allowed_dependents` key of its `[dependencies]` table.
- Each boundary manifest MUST list its crate's allowed dependencies in the `allowed_dependencies` key of its `[dependencies]` table.
- Each boundary manifest MUST list its crate's forbidden edges in the `forbidden_edges` key of its `[dependencies]` table.
- The boundary manifests MUST encode the crate set of the crate graph table.
- The `allowed_dependencies` keys of the boundary manifests MUST encode the allowed workspace edges of the crate graph table.
- The `forbidden_edges` keys of the boundary manifests MUST encode the forbidden workspace edges of the crate graph table.
- The crate dependency table in ADR-RUN-0003 MUST list the same crates as the crate graph table.
- The crate dependency table in ADR-RUN-0003 MUST list the same edges as the crate graph table.
- The crate graph in `docs/architecture.md` MUST list the same crates as the crate graph table.
- The crate graph in `docs/architecture.md` MUST list the same edges as the crate graph table.
- Every crate in the public package set MUST exclude sc-compose crate dependencies.
- Every crate in the public package set MUST exclude ATM crate dependencies.
- Every crate in the public package set MUST exclude sc-compose Rust imports.
- Every crate in the public package set MUST exclude ATM Rust imports.
- Each manifest's `allowed_dependencies` MUST equal the union of its crate graph row and its additional dependency list.
- The `just lint` recipe in the root `Justfile` MUST run `sc-lint lint sc-boundary` against the boundary manifests.
- `just lint` MUST fail when a manifest is violated.


### Rationale

The statements preserve the existing contract while separating independently observable obligations.

### Success Criteria

Inspect boundaries/<crate>/<crate>.toml for each public crate against the local format and graph, including reverse dependent entries and the per-crate additional dependency list. Compare the graph with docs/architecture.md and ADR-RUN-0003. Inspect production Rust imports and normal/build package closures, including renamed dependencies, for the tool exclusions. Inspect the root Justfile invocation; a scratch manifest violation produces nonzero exit from just lint.
### Related Documents

**Architecture Decisions:**

- ADR-RUN-0003 — crate dependency graph and the `server` cargo feature (`docs/architecture.md`)
- ADR-RUN-0017 — 2026-10-31 amendment

---

## REQ-RUN-0101: Spike `examples/spike` proves the stack coexists

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: MCP means Model Context Protocol, served here through rmcp. REST is the HTTP route interface; SQL is the database query language. The service function is the one async function used by all three clients to access SQLite. CLI means command-line interface. MCP means Model Context Protocol. HTTP means Hypertext Transfer Protocol. SQL means Structured Query Language. A custom connector is connector code written for the spike instead of using the documented reqwest Unix-socket connection API. A client reaches the one async SQLite service function when the result the client receives for its request is the result that function returned for that request. An MCP client is any program that sends MCP Streamable HTTP requests over the server's Unix socket. `tools/call` is the MCP request that runs one named tool.

`examples/spike` in the `randlee/sc-runtime` repository is a throwaway proof,
built before any of the four library crates exists, so it uses the third-party
crates directly.

   - REST routes registered through `utoipa-axum` 0.2 `OpenApiRouter` with
     the `routes!` macro;
   - an `rmcp` 3.x `StreamableHttpService`, configured in stateless mode and
     mounted with `Router::nest_service("/mcp", service)`;
   - `sqlx` 0.9 with the `sqlite` feature, opening a SQLite database;
   - a Unix domain socket listener (`tokio::net::UnixListener`) passed to
     `axum::serve`.

The spike operation may read SQLite, write SQLite, or do both; the operation remains unspecified.

**MUST statements:**

- The spike MUST contain one server binary.
- That server binary MUST run in a single process.
- That server process MUST use a single `axum` 0.8 `Router`.
- The spike server MUST register REST routes through `utoipa-axum` 0.2 `OpenApiRouter` with the `routes!` macro.
- The spike `Cargo.toml` MUST list `rmcp` 3.x.
- The spike server MUST serve its `rmcp` `StreamableHttpService` at the path `/mcp`.
- The spike server MUST mount that `StreamableHttpService` with `Router::nest_service`.
- That `StreamableHttpService` MUST use stateless mode.
- The spike `Cargo.toml` MUST list `sqlx` 0.9.
- The spike `Cargo.toml` MUST enable the `sqlite` feature of `sqlx`.
- The spike server MUST open its SQLite database through `sqlx`.
- The spike server MUST pass a Unix domain socket listener (`tokio::net::UnixListener`) to `axum::serve`.
- Every SQLite query in the spike MUST be run by the same single asynchronous service function.
- The REST handler MUST call the one async service function that reads or writes the SQLite database.
- The MCP tool MUST call the one async service function that reads or writes the SQLite database.
- The spike MUST contain a `clap` command-line client.
- The spike `clap` client MUST connect to the server socket using `reqwest` 0.13 `ClientBuilder::unix_socket(path)`.
- The spike `clap` client MUST call the REST route.
- Against one running spike server process, `curl --unix-socket <path>` calling the REST route MUST reach the one async SQLite service function.
- Against one running spike server process, the `clap` client MUST reach the one async SQLite service function.
- Against one running spike server process, an MCP `tools/call` request sent by an MCP client to `/mcp` MUST reach the one async SQLite service function.

**MUST NOT statements:**

- The REST handler MUST NOT contain SQL.
- The MCP tool MUST NOT contain SQL.
- The spike MUST NOT use the Rust crate named `hyperlocal`.
- The spike MUST NOT use a custom connector.

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
### Implementation Notes

**Key Considerations:**

- OPEN: what the spike's one operation does, its route path and its MCP
  tool name are not stated; any single operation that touches SQLite satisfies
  this requirement.
- OPEN: which MCP client is used for the proof is not stated; any client
  that speaks MCP Streamable HTTP over the Unix socket qualifies.

---

## REQ-RUN-0102: Spike answers the four unverified facts and records tested versions

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-23  
**Owner:** randlee  

### Requirement Statement

Definitions: MCP means Model Context Protocol. ADR means architecture decision record. Sprint a-1 is the spike sprint. An evidence row is a row of the first table below, named by its Row id (V1 to V4b). The surface snapshot fact is the former fifth verification fact: how to dump MCP `tools/list` for surface snapshots. A surface snapshot is the tools/list dump formerly proposed as the fifth verification fact. Snapshot tooling is not part of this repository. To record the exact version used for a crate means to write the `major.minor.patch` version the spike built with into `docs/architecture.md`. The spike evidence for an evidence row is the answer sprint a-1 recorded for that row (true, or false with the finding) together with the evidence named in that row's Evidence required column. The finding for an evidence row is what sprint a-1 observed in place of that row's fact. `docs/architecture.md` contains an item when the item is written in the text of that file itself, not only linked to from it. A tested version is the exact version recorded for one of the crates named in the MUST statements: the version sprint a-1 built and tested with. For `axum`, `utoipa`, `utoipa-axum`, `rmcp`, `sqlx` and `reqwest`, a tested version is the minimum version tested with; it does not pin or cap the version a consumer uses (PRD-ENG-9). For `cargo-generate`, the tested version is the version of the generation tool that sprint a-1 ran.

The sc-runtime design relies on four facts it has not verified.

This table is the only definition of that evidence. ADR-RUN-0203 and
ADR-RUN-0402, both Active after sprint a-1, refer to its rows by id. Any
text beside a row id in those ADRs is a non-normative summary; the rows of
this item govern.

| Row | Fact to verify | Evidence required |
|---|---|---|
| V1 | `rmcp` 3.x, `utoipa-axum` 0.2 and `axum` 0.8 coexist on one `axum::Router` with no dependency version conflict. | `examples/spike` compiles with all three in one binary; `cargo tree -i axum` run on the spike package shows exactly one `axum` version; with the spike running, `curl --unix-socket`, the `clap` client and an MCP client each reach the one service function through that one router ([REQ-RUN-0101](requirements.md)). |
| V2 | `rmcp`'s `Parameters<T>` accepts a struct `T` that derives both `schemars::JsonSchema` and `utoipa::ToSchema`, with no clash between the two derives or their schemas. | The spike's request struct derives `Serialize`, `Deserialize`, `schemars::JsonSchema` and `utoipa::ToSchema`; it is used as an `axum` `Json<T>` body and as `Parameters<T>` in a `#[tool]`; it compiles and both calls answer; MCP `tools/list` returns an input schema for that tool, and `openapi.json` contains the same struct as a component schema. |
| V3a | The `cargo-generate` version pinned by sprint a-1 accepts, in `cargo-generate.toml`, placeholders of type `string`, `bool` and `array`, a conditional `ignore` list keyed on a placeholder value, and an `exclude` list; the exact syntax of each is known. | A scratch `cargo-generate.toml` using every one of these constructs, copied into the record, that the pinned version parses without error. |
| V3b | A conditional `ignore` list includes or excludes a whole file. | Two runs of the scratch template that differ only in one `bool` value: the conditionally ignored file is present in one output and absent in the other. |
| V4a | `cargo generate --path <template> --template-values-file <file> --name <n> --silent` runs with no prompt, including when a value is of type `array`. | The command line used; run with stdin closed it exits 0 and prints no prompt, and the rendered output contains the `string`, `bool` and `array` values from the values file. |
| V4b | A file listed under `exclude` (for example `AGENTS.md.j2`) is copied byte-for-byte without Liquid rendering. | A `.j2` file containing `{{ }}` expressions is byte-identical in the template and in the generated output (`cmp` exits 0). |

ADR-RUN-0203 and ADR-RUN-0402 were the two `Proposed` ADRs, and each held
nothing but unverified facts. Sprint a-1 made both `Active`:

| ADR | Holds | Rests on rows |
|---|---|---|
| [ADR-RUN-0203](architecture.md) | that the tested `rmcp`, `utoipa-axum` and `axum` versions coexist, and that one struct is both the `Parameters<T>` input and a `ToSchema` type; the tested versions of the libraries | V1, V2 |
| [ADR-RUN-0402](architecture.md) | the `cargo-generate` mechanics: conditional `ignore`, `exclude`, the non-interactive values file, the `cargo-generate.toml` syntax and the `cargo-generate` version pin | V3a, V3b, V4a, V4b |

Sprint a-1 moved both from `Proposed` to `Active`. Every row was true, so the
Decision text did not have to be replaced by a different mechanism. The
working cargo-generate syntax, including the array-`default` constraint, is
recorded in ADR-RUN-0402.

The shape decisions do not wait for the spike and are already `Active`:
[ADR-RUN-0202](architecture.md) (REST, OpenAPI and MCP on one `axum` router
and one listener) and [ADR-RUN-0401](architecture.md) (the generation
pipeline and the `answers.schema.json` contract). If V1 or V2 is false, the
tested versions change, not the one-router shape.

Decided in this document: the sc-runtime design's Phase 0 exit gate "ADR
amendments written and accepted" is met by
[ADR-RUN-0001](architecture.md) (which amends SC scaffold ADR-013, the
single assembly point and command registry) and
[ADR-RUN-0201](architecture.md) (which carries SC scaffold ADR-014, the
daemon is always running) having Status `Active` in `docs/architecture.md`.
ADRs live in the architecture files; the design's `docs/adr/` directory is
not used.

**MUST statements:**

- Sprint a-1 MUST record whether evidence row V1 is true.
- If evidence row V1 is false, sprint a-1 MUST record the finding for evidence row V1.
- Sprint a-1 MUST record whether evidence row V2 is true.
- If evidence row V2 is false, sprint a-1 MUST record the finding for evidence row V2.
- Sprint a-1 MUST record whether evidence row V3a is true.
- If evidence row V3a is false, sprint a-1 MUST record the finding for evidence row V3a.
- Sprint a-1 MUST record whether evidence row V3b is true.
- If evidence row V3b is false, sprint a-1 MUST record the finding for evidence row V3b.
- Sprint a-1 MUST record whether evidence row V4a is true.
- If evidence row V4a is false, sprint a-1 MUST record the finding for evidence row V4a.
- Sprint a-1 MUST record whether evidence row V4b is true.
- If evidence row V4b is false, sprint a-1 MUST record the finding for evidence row V4b.
- The surface snapshot fact MUST be recorded as closed.
- Sprint a-1 MUST record the exact version used for `axum`.
- Sprint a-1 MUST record the exact version used for `utoipa`.
- Sprint a-1 MUST record the exact version used for `utoipa-axum`.
- Sprint a-1 MUST record the exact version used for `rmcp`.
- Sprint a-1 MUST record the exact version used for `sqlx`.
- Sprint a-1 MUST record the exact version used for `reqwest`.
- Sprint a-1 MUST record the exact version used for `cargo-generate`.
- `docs/architecture.md` MUST contain the spike evidence for every evidence row.
- `docs/architecture.md` MUST contain every tested version.

**MUST NOT statements:**

- The surface snapshot fact MUST NOT be investigated.

### Rationale

These facts fix the minimum dependency versions the crates and the template
are tested with (PRD-ENG-9) and the template syntax every template file uses. Planning later sprints around a guess would
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
   it (ADR-RUN-0203 or ADR-RUN-0402, in the table above) has been changed to
   match the finding.
6. The `**Status:**` lines of ADR-RUN-0001 and ADR-RUN-0201 in
   `docs/architecture.md` read `Active`.
7. The evidence tables of ADR-RUN-0203 and ADR-RUN-0402 cite rows of this
   item by id (V1 to V4b); any text beside the id states nothing the row
   does not, and is marked as a non-normative summary.

---

## REQ-RUN-0103: `examples/spike` is rewritten once, then deleted

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: The spike is the examples/spike proof project. Daemon::builder is the sc-runtime daemon assembly entry point; sc_transport::Client is the reusable client connector of the sc-transport crate; Envelope<T> is the sc-command response envelope. `clap` is the Rust command-line argument parser crate; the spike `clap` client is the spike's command-line client program built with `clap`, whose source is under `examples/spike/cli`. The client parses responses as `sc_command::Envelope<T>` when its source deserializes each server response body into the type `sc_command::Envelope<T>` and not into a type of its own. The rewritten spike is `examples/spike` after the rewrite onto the four library crates required below; its server main file is `examples/spike/server/src/main.rs`, and its line count is the number `wc -l` prints for that file, as [REQ-RUN-0204](requirements.md) also measures it. A client connects through `sc_transport::Client` when it opens its UDS or TCP connection to the spike server by calling `sc_transport::Client` and not by building its own socket or HTTP connection.

`examples/spike` in the `randlee/sc-runtime` repository is first written
directly on `axum`, `utoipa-axum`, `rmcp`, `sqlx` and `reqwest`, before the
four library crates exist.

1. The `Client` type and method names are illustrative until the contract sprint pins them ([REQ-TRN-0005](sc-transport/requirements.md)).
3. This item owns the deletion rule; the release requirement refers to it.
4. After deletion, none of the root `Cargo.toml`, the root `justfile`, the
   files under `.github/workflows/` and the root `README.md` may refer to
   `examples/spike`. Files under `docs/` may still name it, because the
   spike evidence of [REQ-RUN-0102](requirements.md) is recorded there.

**MUST statements:**

- Once `sc-config`, `sc-transport`, `sc-command` and `sc-runtime` exist, the spike server MUST be rewritten to assemble through `sc_runtime::Daemon::builder()`.
- Once `sc-config`, `sc-transport`, `sc-command` and `sc-runtime` exist, the spike server MUST be rewritten to load its configuration through `sc-config`.
- Once `sc-config`, `sc-transport`, `sc-command` and `sc-runtime` exist, the spike `clap` client MUST be rewritten to connect through `sc_transport::Client`.
- Once `sc-config`, `sc-transport`, `sc-command` and `sc-runtime` exist, the spike `clap` client MUST be rewritten to parse responses as `sc_command::Envelope<T>`.
- After the rewrite, `wc -l examples/spike/server/src/main.rs` MUST print a number below 30.
- The commit that deletes `examples/spike` MUST be an ancestor of the commit that the `v0.1.0` tag points at.

**MUST NOT statements:**

- The output of `git ls-tree -r --name-only v0.1.0` MUST NOT contain any path that starts with `examples/spike/`.

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

# Requirement: One operation, three surfaces, one service function

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-RUN-0201: One operation, three surfaces, one service function

**Status:** Active  

### Requirement Statement

Definitions: CLI means command-line interface. MCP means Model Context Protocol. SQL means Structured Query Language. JSON means JavaScript Object Notation. ADR means architecture decision record. `Storage<T>` is the storage trait that reads and writes `T`. A store crate is a crate of the generated project that provides `Storage<T>` for one database engine: `store-sqlite`, which uses the published crate `sc-runtime-db-sqlite`, or a sqlx store crate. A sqlx store crate is a store crate for Postgres or for MySQL and contains that engine's SQL. Only the service function calls a store crate; on a two-store layout the store adaptor of REQ-RUN-0301 calls the store crates and the service function calls the adaptor. An operation is one named action of a project generated from `template/`; the template's example is `widget.create`. The generated crates are the crates of that project. The three surfaces are REST (an HTTP route of the daemon), MCP (a tool at the daemon's MCP endpoint) and CLI (a command of the generated `cli` crate). An operation's service function is an async function in the generated `service` crate that carries out that operation. Pass-through code is code in a REST handler or MCP tool that converts the surface's input to the service function's input type, calls the service function, or converts its result to the surface's response. CLI client code is code that builds the request, sends it by HTTP to the operation's daemon REST route or presents the response. An operation is reachable through a surface when a request on that surface runs the operation's service function and returns its result; for REST the request is an HTTP `POST /ops/<operation>` to the daemon; for CLI the request is a run of the operation's command of the generated `cli` crate, which sends that REST request.

Command paths may be written directly in Phase D. Direct Tcli-to-Tsql conversion needs no custom transform. Later consumer-side generation against the delivered framework remains permitted.

A separately hand-written REST handler, MCP tool and CLI command per operation is the interim form of the command pattern, used because the generator does not exist yet. The target is one shared write path and one shared read path for all three interfaces.

This is a property of a project generated from `template/`, demonstrated by
the template's example operation `widget.create`.

1. For the example:



```rust
   // crates/api-types
   #[derive(Serialize, Deserialize, JsonSchema, ToSchema)]
   pub struct CreateWidget { pub name: String }

   // crates/service
   pub async fn create_widget(s: &Stores, input: CreateWidget)
       -> Result<Widget, OpError>;


```

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
   [REQ-RUN-0102](requirements.md)). Sprint a-1 made that ADR Active and
   proved V2, so these clauses stand. The rule that every surface calls one service
   function does not depend on it.

4. The `service` crate reaches SQL only through `Storage<T>`, provided by a store crate (PRD-STO-1, PRD-STO-2). A project whose only database is SQLite, or that has no database, has no sqlx store crate and need not depend on `sqlx`.
5. A project MAY give REST and MCP different JSON shapes at the edge, provided
   both are converted to the same service-function input type before the
   service function is called. The template example uses one struct for all
   three surfaces. Nothing in the `sc-runtime` crate enforces either choice.

**MUST statements:**

- An operation MUST define exactly one request struct in the generated `api-types` crate.
- An operation MUST define exactly one response struct in the generated `api-types` crate.
- An operation MUST have exactly one service function.
- Each operation MUST be reachable through REST.
- Each operation MUST be reachable through MCP.
- Each operation MUST be reachable through CLI.

**MUST NOT statements:**

- An operation's REST handler MUST NOT contain code other than pass-through code.
- An operation's MCP tool MUST NOT contain code other than pass-through code.
- An operation's CLI command MUST NOT contain code other than CLI client code.
- When `cargo tree -i sqlx --depth 1` runs in the generated project, its output MUST NOT name any generated crate that is not a sqlx store crate.

- REST handlers MUST NOT contain SQL.
- REST handlers MUST NOT call `sqlx`.
- REST handlers MUST NOT call a store crate directly.
- MCP tools MUST NOT contain SQL.
- MCP tools MUST NOT call `sqlx`.
- MCP tools MUST NOT call a store crate directly.
- CLI commands MUST NOT contain SQL.
- CLI commands MUST NOT call `sqlx`.
- CLI commands MUST NOT call a store crate directly.

### Rationale

The failure this architecture exists to prevent is the three surfaces
drifting apart: a CLI command that behaves differently from the REST route of
the same name, or an MCP tool that grows its own query. With one service
function as the only path to SQL, a fix or a validation rule is written once
and every client gets it.

### Success Criteria

1. A host-safe test in the generated project starts one daemon with
   `sc_runtime::testing::DaemonFixture` and creates three widgets with
   distinct names: one by `POST /ops/widget.create`, one by an MCP
   `tools/call` of `widget_create` sent to `/mcp`, and one through the CLI
   command path using the same parsed arguments, endpoint resolver,
   `sc_transport::Client`, and envelope renderer as production, but without
   spawning the CLI binary. The test does not prescribe a permanent helper
   function, trait, mock transport, or generated unit-test layout. It then
   reads all three back with `widget.get` and asserts each is returned with
   the name it was created with. One real CLI-binary smoke runs separately
   on an isolated runner under NFR-RUN-0010.
2. `cargo metadata` in the generated project shows `sqlx` as a direct
   dependency of `store-*` packages only.
3. `grep -rn 'sqlx' crates/api-types crates/service crates/daemon crates/cli`
   in the generated project prints nothing.
4. Inspect `crates/daemon/src/routes.rs` and `crates/daemon/src/mcp.rs`:
   each handler or tool body is a call to a `service::` function plus a
   conversion, with no other logic.

## REQ-RUN-0202: CLI reports `DAEMON.NOT_RUNNING`; no database fallback

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-20  
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. HTTP means Hypertext Transfer Protocol. SQL means Structured Query Language. JSON means JavaScript Object Notation. OpError is the sc-command operation error with code and suggested_action fields. <app> is the application name. `suggested_action` is the `OpError` wire field carrying remediation text; its Rust representation and wire form are defined by REQ-CMD-0002.

This applies to the `cli` crate of a project generated from `template/`.
`<app>` below is the project's application name.

2. The generated CLI provides an explicit `daemon start` subcommand that execs the daemon binary named `{app}-daemon` beside the CLI binary. That subcommand is not auto-start. Obligations 3 to 6 describe what a report-only command reports when the daemon cannot be reached. A later optional auto-start layer may amend launch behavior under deferred [REQ-RUN-0206](requirements.md). Transport does not carry the `suggested_action` string.
3. When the daemon cannot be reached (the socket file does not exist, or the connection is refused), the `sc-transport` client returns the typed error `TransportError::DaemonNotRunning` ([REQ-TRN-0006](sc-transport/requirements.md)). Whether `connect` or the first `get` or `post` call returns it is OPEN in [REQ-TRN-0005](sc-transport/requirements.md), and the client method names are illustrative until pinned there.

Decided in this document: `--json` prints one envelope on stdout and exits 1.
`DAEMON.NOT_RUNNING` uses kind `unavailable`, message `the daemon is not running`,
and `suggested_action` `run <app> daemon start`. Every other `TransportError`
uses kind `internal`, message `transport failure {code} at {endpoint}: {cause}`,
and `suggested_action` null. The mapping lives in the generated CLI. The code
is `sc_observability_types::ErrorCode` built from the transport `&'static str`.
Transport does not depend on that type and does not carry `suggested_action`.

| Variant | Kind | Code | Message | suggested_action |
|---|---|---|---|---|
| `InvalidName` | internal | `APP.INVALID_NAME` | `transport failure APP.INVALID_NAME at {endpoint}: {cause}` | null |
| `InvalidEndpoint` | internal | `TRANSPORT.INVALID_ENDPOINT` | `transport failure TRANSPORT.INVALID_ENDPOINT at {endpoint}: {cause}` | null |
| `MissingTcpPort` | internal | `TRANSPORT.MISSING_TCP_PORT` | `transport failure TRANSPORT.MISSING_TCP_PORT at {endpoint}: {cause}` | null |
| `InstanceRootUnavailable` | internal | `TRANSPORT.INSTANCE_ROOT_UNAVAILABLE` | `transport failure TRANSPORT.INSTANCE_ROOT_UNAVAILABLE at {endpoint}: {cause}` | null |
| `DaemonNotRunning` | unavailable | `DAEMON.NOT_RUNNING` | `the daemon is not running` | `run <app> daemon start` |
| `ResponseDecode` | internal | `TRANSPORT.RESPONSE_DECODE` | `transport failure TRANSPORT.RESPONSE_DECODE at {endpoint}: {cause}` | null |
| `TimedOut` | internal | `TRANSPORT.TIMED_OUT` | `transport failure TRANSPORT.TIMED_OUT at {endpoint}: {cause}` | null |
| `Request` | internal | `TRANSPORT.REQUEST` | `transport failure TRANSPORT.REQUEST at {endpoint}: {cause}` | null |
| `Bind` | internal | `TRANSPORT.BIND` | `transport failure TRANSPORT.BIND at {endpoint}: {cause}` | null |
| `EndpointBusy` | internal | `TRANSPORT.ENDPOINT_BUSY` | `transport failure TRANSPORT.ENDPOINT_BUSY at {endpoint}: {cause}` | null |
| `NotASocket` | internal | `TRANSPORT.NOT_A_SOCKET` | `transport failure TRANSPORT.NOT_A_SOCKET at {endpoint}: {cause}` | null |

The envelope is `{version, ok, data, error}` with `ok` false and `data` null.
`daemon start` exists and execs the daemon binary. Transport does not carry
`run <app> daemon start`.

NFR-RUN-0001 owns the rule that a CLI links no daemon-side dependency. The reusable transport client remains report-only.

**MUST statements:**

- The CLI MUST reach data only by sending HTTP requests to the daemon through `sc_transport::Client`.
- The CLI MUST map that error to an `OpError`.
- That `OpError` MUST have `code` equal to `"DAEMON.NOT_RUNNING"`.
- That `OpError` MUST have `suggested_action` equal to `"run <app> daemon start"`, with `<app>` replaced by the application name.
- For the `DAEMON.NOT_RUNNING` `OpError`, the `suggested_action` wire field MUST serialise as a JSON string, as defined by [REQ-CMD-0002](sc-command/requirements.md).
- When reporting `TransportError::DaemonNotRunning`, the CLI MUST exit with a non-zero status.
- When reporting `TransportError::DaemonNotRunning` with `--json`, the CLI MUST print exactly one response envelope to standard output.
- The response envelope printed by the CLI when reporting `TransportError::DaemonNotRunning` with `--json` MUST have a `version` key.
- That envelope MUST have an `ok` key.
- That envelope MUST have a `data` key.
- That envelope MUST have an `error` key.
- That envelope's `ok` MUST be `false`.
- That envelope's `data` MUST be `null`.
- That envelope's `error` MUST be the `OpError` specified by this record for `DAEMON.NOT_RUNNING`.

**MUST NOT statements:**

- The generated CLI MUST NOT open the application database.
- The `cli` crate MUST NOT have `sqlx` among its normal (non-dev) dependencies.
- The `cli` crate MUST NOT have any generated `store-*` crate among its normal (non-dev) dependencies.
- The `cli` crate MUST NOT have any generated `service` crate among its normal (non-dev) dependencies.
- Report-only commands MUST NOT start the daemon.
- The reusable transport client MUST NOT spawn any process.
- The reusable transport client MUST NOT retry for readiness.
- Decided in this document: any other transport failure (a timeout, a response body that cannot be decoded) MUST NOT be reported as `DAEMON.NOT_RUNNING`.
- A response that arrives with a non-2xx HTTP status proves a daemon is running, so it MUST NOT be reported as `DAEMON.NOT_RUNNING` either; how the `sc-transport` client presents a non-2xx response is OPEN in [REQ-TRN-0006](sc-transport/requirements.md).

### Rationale

The daemon is the only process allowed to open the local database, and it
holds an exclusive lock on `<instance-root>/daemon.lock` to stay the only
one. `<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](sc-transport/requirements.md)). A CLI that fell
back to opening the database itself would be a second
writer to a SQLite file and a second code path to SQL, and would behave
differently depending on whether a daemon happened to be up. Every CLI
command therefore needs a running daemon. In Phase A the absence of a daemon
gets a typed code an agent can branch on and a suggested action that says how
to fix it. Deferred REQ-RUN-0206 may later add an optional auto-start layer
without changing the transport or envelope contracts.
### Success Criteria

These criteria constrain externally observable behavior and do not prescribe
the CLI's internal helper-function or generated-test architecture. A host-safe
in-process test may exercise the CLI command path without spawning the binary;
criterion 2 runs the real binary only on an isolated runner.

1. A host-safe test gives the generated CLI command path
   `TransportError::DaemonNotRunning` with `--json` selected. It asserts:
   the exit status is non-zero; the output parses as JSON; `ok` is `false`;
   `data` is `null`; `error.code` is `"DAEMON.NOT_RUNNING"`;
   `error.suggested_action` is `"run <app> daemon start"` with the
   application name substituted.
2. On an isolated machine ([NFR-RUN-0010](requirements.md)), a test runs the
   real Phase A CLI binary with `--json` against an endpoint where no daemon is
   listening. It asserts the
   same envelope as criterion 1 and a non-zero exit status, and that
   afterwards the instance root contains no `daemon.lock`, no `daemon.sock`
   and no database file.
3. `cargo tree -p <project>-cli -e normal --prefix none` in the generated project
   prints no line beginning with `sqlx `, `<project>-service `, or a store package (the
   name followed by a space). Amended 2026-10-25. v0.1 rendered the package as `cli`.
4. A host-safe test gives the same CLI reporting path a transport failure that is not
   `DaemonNotRunning` (an undecodable response body) and asserts that
   `error.code` is not `"DAEMON.NOT_RUNNING"`. How a non-2xx response reaches
   the rendering function follows the OPEN in
   [REQ-TRN-0006](sc-transport/requirements.md).
### Implementation Notes

**Key Considerations:**

- OPEN: the format and stream of the message when `--json` is not given are
  not stated.

---

## REQ-RUN-0203: REST, MCP and CLI `--json` return the same envelope

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. MCP means Model Context Protocol. HTTP means Hypertext Transfer Protocol. JSON means JavaScript Object Notation. The envelope is the sc-ai-cli response object defined by [REQ-CMD-0001](sc-command/requirements.md), with the keys `version`, `ok`, `data` and `error`. `OpError` is the typed error type of the `sc-command` crate, defined in `crates/sc-command/src/error.rs`. CallToolResult is the rmcp operation tool result with content and the error flag `is_error`. An operation route is the REST route of one operation. A surface is REST, MCP or CLI `--json`. Two envelopes are equal as JSON values when they parse to the same JSON value; key order and whitespace do not count.

This is a property of a project generated from `template/`.

3. `OpError` has the five fields `kind`, `code`, `message`, `details` and `suggested_action`; which keys appear in its JSON form is defined by [REQ-CMD-0002](sc-command/requirements.md), and this item requires exactly the keys defined there.

**MUST statements:**

- Every operation response, on every surface, MUST be an envelope.
- The envelope MUST be a JSON object.
- The set of keys of the envelope MUST be exactly `version`, `ok`, `data` and `error`.
- The envelope MUST be produced by the type `Envelope<T>` in the `sc-command` crate ([REQ-CMD-0001](sc-command/requirements.md)).
- On success `ok` MUST be `true`.
- On success `data` MUST hold the operation's response struct.
- On success `error` MUST be `null`.
- On failure `ok` MUST be `false`.
- On failure `data` MUST be `null`.
- On failure `error` MUST be an `OpError` object.
- The HTTP response body of an operation route MUST be JSON text.
- The HTTP response body of an operation route MUST parse to the envelope.
- The `CallToolResult` of an operation MCP tool MUST carry that same envelope as its content.
- When the envelope's `ok` is `false`, the `CallToolResult` of the operation MCP tool MUST have `is_error` set to `Some(true)` (REQ-CMD-0005).
- When a CLI command is given `--json`, the CLI command MUST write the envelope to stdout.
- When a CLI command is given `--json`, the envelope that the CLI command prints MUST be the envelope that the CLI command received from the daemon, unchanged.
- When a CLI command is given `--json`, the text that the CLI command prints and the REST response body that the CLI command received from the daemon MUST be equal as JSON values.
- When a CLI command is given `--json`, the text that the CLI command prints MUST include any content inside `data` or `error.details` of the received REST response body that the CLI's own types do not define.
- For the same operation, the same input and the same daemon state, the envelope obtained through REST, through MCP and through CLI `--json` MUST be equal as JSON values.

### Rationale

Agents are the main consumers of these CLIs and APIs. One response shape, with
an explicit `ok`, a typed `code` to branch on and a `suggested_action` to
recover with, means an agent needs one parser and one error-handling path no
matter which surface it used, and can move between surfaces without relearning
anything.
### Success Criteria

1. A test against one `DaemonFixture` daemon calls `widget.get` for the same
   existing widget through REST, through an MCP `tools/call`, and through the
   production CLI command path with `--json` rendering: parsed arguments,
   endpoint resolution, `sc_transport::Client`, and unchanged JSON output are
   exercised inside the test process without running the CLI binary. This
   criterion does not prescribe the deferred helper or test-generation shape
   of [REQ-RUN-0312](requirements.md). It parses the three results as JSON and
   asserts all three are equal, with `ok` equal to `true` and `error` equal to
   `null`. Any real CLI-binary proof follows the process-isolation rule in
   [NFR-RUN-0010](requirements.md).
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
### Implementation Notes

**Key Considerations:**

- OPEN: the type and value of `version` are not stated.
- OPEN: whether `details` and `suggested_action` are present as `null` or
  omitted when they are empty is undecided; the owner of that question is
  [REQ-CMD-0002](sc-command/requirements.md).
- OPEN: obligations 1 to 7 cover responses whose content comes from a
  service function. Whether a response generated by a framework is also an
  envelope is not decided, and this item owns the question. The cases are: an
  unknown route, a request body that `axum` fails to deserialise, and an MCP
  protocol error returned as `Err(McpError)` by `rmcp`.

---

## REQ-RUN-0204: Rewritten spike `main.rs` is under 30 lines

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: The rewritten spike server main file is `examples/spike/server/src/main.rs`. Its startup sequence comprises configuration loading with `sc-config`, the `sc_runtime::Daemon::builder(...)` call chain and conversion of the run result into the process exit code. The configuration-loading call to `sc-config` is a call to the function `sc_config::load`, which reads the spike server's configuration and returns it as a value of the server's configuration type. The rewritten `examples/spike/server/src/main.rs` is that file after `examples/spike` is rewritten on the four library crates `sc-config`, `sc-transport`, `sc-command` and `sc-runtime`. The complete `sc_runtime::Daemon::builder(...)` call chain is one expression that starts with `sc_runtime::Daemon::builder(...)` and goes on through `.stores(...)`, `.routes(...)`, `.mcp(...)` and `.run().await`.

`examples/spike` is first written directly on `axum`, `utoipa-axum`, `rmcp`
and `sqlx`, then rewritten on the four library crates `sc-config`,
`sc-transport`, `sc-command` and `sc-runtime`.

2. Lines are counted with `wc -l`: every physical line counts, including
   blank lines, comments, `use` lines and attributes. This counting rule is
   decided in this document.
4. The router function, the MCP service, the `Stores` struct and the service
   function MAY live in other files of the spike; they are project wiring,
   not assembly.

The spike server `main.rs` is `examples/spike/server/src/main.rs`. The client
binary is `examples/spike/cli`. `examples/spike` is its own Cargo package,
excluded from the root workspace.

**MUST statements:**

- After the rewrite, the `main.rs` of the spike's server binary MUST be under 30 lines, that is 29 lines or fewer.
- The rewritten `examples/spike/server/src/main.rs` MUST contain the complete daemon startup sequence described in this record.
- The rewritten `examples/spike/server/src/main.rs` MUST contain a call to the function `sc_config::load`.
- The rewritten `examples/spike/server/src/main.rs` MUST contain the complete `sc_runtime::Daemon::builder(...)` call chain.
- The rewritten `examples/spike/server/src/main.rs` MUST itself turn the result of `.run().await` into the process exit code.

**MUST NOT statements:**

- It MUST NOT meet the limit by moving that sequence into another file.

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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-20  
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. MCP means Model Context Protocol. HTTP means Hypertext Transfer Protocol. TCP means Transmission Control Protocol. JSON means JavaScript Object Notation. The daemon is a process assembled with `sc_runtime::Daemon::builder()`. The daemon router is the `axum::Router` that the builder assembles. A listener is the Unix domain socket, or the TCP address and port, that `run()` binds. Serving the daemon router unchanged means each listener answers every request with the response the daemon router gives for that request.

This item states the end-to-end property; what the `sc-runtime` crate merges and mounts to achieve it is [REQ-RT-0004](sc-runtime/requirements.md).

| What | Source |
|---|---|
| The project's own routes (in the template: `POST /ops/widget.create` and `POST /ops/widget.get`) | the `utoipa_axum::OpenApiRouter` returned by the project's `.routes(...)` closure |
| The OpenAPI document `openapi.json` describing those routes | generated from the same `OpenApiRouter`; the project writes no separate spec |
| A health route | added by `sc-runtime` |
| The MCP endpoint at path `/mcp` | the `rmcp` `StreamableHttpService` returned by the project's `.mcp(...)` closure, mounted with `nest_service("/mcp", ...)` |

3. v0.1 binds exactly one listener. `TransportConfig` is `#[non_exhaustive]`
   and has no `additional_endpoints` field. `sc-transport` can bind either a
   Unix domain socket or a TCP address ([REQ-TRN-0003](sc-transport/requirements.md));
   `Daemon::builder()` serves the one endpoint that resolver returns. A later
   release may add a second listener without changing that v0.1 field list.

The OpenAPI document is `GET /openapi.json`. The health route is `GET /health`,
status 200, and the body is the `sc_command` success envelope from
`sc_command::Envelope`'s server-feature Axum `IntoResponse`, with data
`{ "status": "ok" }`.

**MUST statements:**

- The daemon MUST assemble exactly one daemon router.
- The daemon MUST serve the daemon router from one process.
- `run()` MUST serve the daemon router unchanged on every listener.
- The daemon router MUST serve the routes of the `OpenApiRouter` that the project's `.routes(...)` closure returns (in the template: `POST /ops/widget.create` and `POST /ops/widget.get`).
- The daemon router MUST serve the OpenAPI document generated from the same `OpenApiRouter`, as specified in the table above.
- The daemon router MUST serve the health route added by `sc-runtime`, as specified in the table above.
- The daemon router MUST serve the optional MCP endpoint at `/mcp` from the `.mcp(...)` closure, as specified in the table above.
- When the project gives no `.mcp(...)` closure, the project routes MUST still be served.
- When the project gives no `.mcp(...)` closure, the OpenAPI document MUST still be served.
- When the project gives no `.mcp(...)` closure, the health route MUST still be served.

**MUST NOT statements:**

- MCP MUST NOT run in a separate process.
- MCP MUST NOT run in a separate binary.
- The source of the generated `crates/daemon` MUST NOT bind a listener for the MCP endpoint other than the one listener that serves the project routes.
- When the project gives no `.mcp(...)` closure, `/mcp` MUST NOT be served.

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

## REQ-RUN-0206: CLI auto-starts the daemon with a clean, launchd-equivalent launch

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-09-20  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. JSON means JavaScript Object Notation. ADR means architecture decision record. A CLI command finds that the daemon is not running when the `sc-transport` client returns `TransportError::DaemonNotRunning` ([REQ-TRN-0006](sc-transport/requirements.md)). A clean launch is the launch from the service definition with that service's environment, working directory, standard streams and session, without inherited CLI state. CI means continuous integration. The winning daemon is the daemon that acquired daemon.lock; the losing daemon failed that singleton-lock acquisition. Explicit invocation values are values the daemon needs from the CLI invocation, including a resolved endpoint or instance root. The startup directory is the working directory of the daemon process at startup, held invariant across launchd and CLI starts. Retry without bound means retry indefinitely, without a finite stopping condition. A finite retry bound is a cap on readiness attempts or elapsed readiness-wait time, selected by the implementation. The requirement is to enforce a bound; it does not fix its numerical value. A permitted daemon argument is an argument of the service definition or an explicit invocation value. A value is passed to the daemon visibly when the value appears in the started daemon's arguments or configuration. An isolated machine is a virtual machine or an ephemeral CI runner ([NFR-RUN-0010](requirements.md)).

Decided by the owner on 2026-09-20. The sc-runtime design document records
report-only as the default and leaves auto-start undecided; this item
replaces that default.

This applies to the production code of the `cli` crate of a project
generated from `template/`: it describes how a real user's or agent's
command starts the real daemon. How tests may start daemons is a separate
rule ([NFR-RUN-0010](requirements.md)).
`<app>` is the project's application name. `<instance-root>` is the
per-application, per-user directory resolved by sc-transport, or an
explicitly supplied path; its default location is undecided
([REQ-TRN-0002](sc-transport/requirements.md)).

2. That exception is decided in this document: the owner decided the clean, launchd-equivalent launch and that the environment must not leak; the alternative that needs no exception is recorded in the first OPEN below.
3. No environment variable set in the shell, agent session or parent process that ran the CLI may appear in the daemon's environment because the CLI started it. This is the known failure mode of auto-start: variables leaking from whichever caller happened to start the daemon make the daemon behave differently from one start to the next, and the leak persists for the life of the daemon.
6. If the daemon that won is not reachable at the endpoint this CLI resolved (for example it was started on the default socket while this CLI was given another endpoint), obligation 7 applies.
9. On that machine they MAY use the default `<instance-root>`; tests that share one `<instance-root>` MUST NOT run at the same time.

The daemon.lock singleton (REQ-RT-0002) decides the winner.

Auto-start starts a real daemon process.

**MUST statements:**

- Auto-start MUST be the default.
- When a CLI command finds that the daemon is not running, the CLI MUST start the daemon.
- After starting the daemon, the CLI MUST wait until the daemon accepts a connection on the resolved endpoint.
- After the started daemon accepts a connection on the resolved endpoint, the CLI MUST run the command the user asked for.
- The daemon the CLI starts MUST be started the same way it starts when launchd (the macOS service manager) launches it cleanly from its service definition.
- The started daemon process MUST use the same program as a launchd launch.
- The started daemon process MUST use the environment a launchd launch gives it rather than the CLI process's environment.
- The started daemon process MUST use the working directory a launchd launch gives it rather than the CLI process's working directory.
- The started daemon process MUST use the standard input a launchd launch gives it rather than the CLI process's standard input.
- The started daemon process MUST use the standard output a launchd launch gives it rather than the CLI process's standard output.
- The started daemon process MUST use the standard error a launchd launch gives it rather than the CLI process's standard error.
- The started daemon process MUST use the session a launchd launch gives it rather than the CLI process's session.
- The CLI-started daemon's arguments MUST include every argument of the service definition (ADR-RUN-0202 point 4).
- Every argument of the CLI-started daemon MUST be a permitted daemon argument (ADR-RUN-0202 point 4).
- The daemon's startup directory (its working directory when it starts) MUST always be the same directory, whoever starts it and from wherever, including launchd starts, CLI starts and every CLI invocation directory.
- Every value the daemon needs from the CLI's invocation MUST be passed to the daemon explicitly.
- Every value the daemon needs from the CLI's invocation MUST be passed to the daemon visibly.
- In particular, an endpoint or instance root that the CLI resolved from `--endpoint` or from `SC_ENDPOINT` in its own environment ([REQ-RUN-0310](requirements.md)) MUST reach the daemon as an explicit argument or configuration value; otherwise the daemon, which does not see the CLI's `SC_ENDPOINT`, would bind somewhere else and the CLI would never reach it.
- The daemon MUST outlive the CLI command that started it.
- Two CLI commands that auto-start at the same time MUST end with exactly one daemon.
- When auto-start is disabled, the CLI MUST report the `DAEMON.NOT_RUNNING` result that REQ-RUN-0202 defines.
- When the daemon the CLI started does not become reachable, the CLI MUST report the `DAEMON.NOT_RUNNING` result that REQ-RUN-0202 defines.
- The CLI MUST still have no direct-database mode (REQ-RUN-0202).
- Starting the daemon MUST remain the only action the CLI takes about a missing daemon.
- Tests of this item MUST run only on an isolated machine (NFR-RUN-0010).
- The CLI whose daemon lost MUST connect to the daemon that won.
- In either case, readiness retries MUST enforce a finite retry bound.

**MUST NOT statements:**

- Tests of this item MUST NOT run on a developer's host.
- The daemon MUST NOT be stopped by the CLI process exiting.
- The daemon MUST NOT be stopped by its terminal closing.
- The daemon's startup directory MUST NOT be the CLI's current directory.
- The daemon MUST NOT inherit the environment of the CLI process.
- The CLI MUST NOT pass a value the daemon needs from the CLI's invocation through inherited environment.
- The CLI whose daemon lost the `daemon.lock` singleton race MUST NOT treat losing as a failure.

### Rationale

Every CLI command needs a running daemon, so a person or an agent who types a
command should get its result, not an instruction to start something first.
Auto-start is only safe if the daemon it produces is the same daemon every
time. A daemon that inherits the environment of whoever ran the first
command carries that caller's variables (credentials, proxy settings, tool
overrides, an agent session's variables) for as long as it lives, behaves
differently depending on who started it, and cannot be reproduced when it
misbehaves. Requiring the launch to be the one launchd performs gives one
definition of a clean start, used both at login and on demand, and makes the
CLI-started daemon indistinguishable from the service-started one.
### Success Criteria

Every criterion below starts a real daemon process, so all of them run only
on an isolated machine ([NFR-RUN-0010](requirements.md)), never on a
developer's host. Where a criterion says "a fresh tempdir instance root", the
test on that machine MAY use the default `<instance-root>` instead, emptied
before the test, when the binaries have no way to be given an explicit one
(that is OPEN in [REQ-RUN-0310](requirements.md)). Criterion 3 is
conditional on the non-default-endpoint OPEN and the carrier OPEN of this
item.

1. A test runs a generated CLI command with a fresh tempdir instance root
   and no daemon running. It asserts: the command succeeds and returns the
   operation's result; afterwards a daemon holds `daemon.lock` in that
   tempdir; a second command succeeds without starting another daemon.
2. The same test runs the first command with a sentinel environment variable
   set in the CLI's environment only (a name no service definition sets). It
   reads the started daemon's environment from outside the process (the
   operating system's process-inspection facility) and asserts the sentinel
   is absent.
3. A test sets `SC_ENDPOINT` in the CLI's environment only, to a socket path
   in the tempdir, and runs a command with no daemon running. It asserts the
   command succeeds, which shows the endpoint reached the daemon explicitly
   (obligation 4), and that the daemon's environment does not contain
   `SC_ENDPOINT`.
4. A test runs two CLI commands against one fresh tempdir instance root with no daemon running. Use explicit synchronization to hold both launched daemon attempts before singleton-lock acquisition, release them to contend for the same lock, and assert both CLI commands succeed with exactly one surviving daemon holding `daemon.lock`.
5. A test runs a CLI command from one directory, stops the daemon, and runs
   a CLI command from a different directory; each time it reads the started
   daemon's working directory from outside the process and asserts it is the
   same directory and is neither of the two directories the CLI was run
   from.
6. A test starts the daemon through a CLI command run under a
   pseudo-terminal, waits for the CLI process to exit, closes the terminal,
   and asserts the daemon still answers a request. It also asserts the
   daemon's standard input is not that terminal and its session id differs
   from the CLI's.
7. A test runs a command with auto-start disabled and no daemon running, and
   asserts the `DAEMON.NOT_RUNNING` result of
   [REQ-RUN-0202](requirements.md) and that no daemon was started (the
   tempdir instance root contains no `daemon.lock` and no `daemon.sock`).
   This criterion is conditional on the OPEN above on how auto-start is
   disabled.
8. A test runs the command with `--json` while driving the started daemon readiness failure with controlled attempt results and clock advancement to the implementation's existing finite bound. It asserts the CLI exits non-zero and prints the envelope of [REQ-RUN-0202](requirements.md): `ok` is `false`, `data` is `null`, `error.code` is `"DAEMON.NOT_RUNNING"` and `error.suggested_action` is `"run <app> daemon start"` with the application name substituted. No wall-clock duration is a pass/fail assertion.
9. Once the mechanism and service-definition OPENs are decided: on macOS, a
   test compares a CLI-started daemon with the same daemon started by launchd
   from its service definition, and asserts that the environment, working
   directory and standard streams are equal, that each daemon is the leader
   of its own session with no controlling terminal, and that the arguments
   are equal apart from the explicit values of obligation 4.
### Implementation Notes

**Key Considerations:**

- Status: Deferred — post-core application layer
- Disposition (2026-09-21): The owner-directed Phase A recut returns to the
  original five-milestone core plan. Auto-start is not required to prove or
  publish the reusable crates or the minimal JSON-driven template. The detailed
  design below is preserved as decision history and as input to a later optional
  application layer, but it is not binding on Phase A and no Phase A criterion
  may claim it complete. Its future design must be evaluated against generated
  applications after the core release.
- OPEN: which directory the fixed startup directory is (for example
  `<instance-root>`, or a directory the service definition names) is not
  decided. It matters beyond auto-start: `sc_config::load` reads the `config`
  directory relative to the current working directory
  ([REQ-CFG-0003](sc-config/requirements.md)), so the startup directory decides
  which configuration files the daemon reads unless the daemon is given the
  config directory explicitly.
- OPEN: what auto-start does when the CLI's resolved endpoint or instance
  root is not the default (an `--endpoint` value, `SC_ENDPOINT`, or an explicit
  instance root is in effect) is not decided. Either the daemon is started with
  those values passed explicitly, which is the exception in obligation 2; or
  auto-start applies only to the default endpoint and the CLI reports
  `DAEMON.NOT_RUNNING` whenever an override is in effect, which keeps the launch
  exactly equal to the launchd launch but leaves obligation 9's tests needing
  another way to reach a tempdir.
- OPEN: the carrier by which the CLI passes a resolved endpoint and
  instance root to the daemon it starts is not decided: command-line arguments
  of the daemon (the `--endpoint` option of
  [REQ-RUN-0310](requirements.md); whether the binaries accept an
  instance-root option is OPEN there), or a configuration value and where it is
  stored.
- OPEN: the mechanism that satisfies obligation 2 is not decided: asking
  launchd to start the daemon's registered job (so launchd itself performs the
  launch), or spawning the daemon directly with an environment, working
  directory, standard streams and session constructed to match a launchd
  launch. A job started from its registered definition takes no
  per-invocation values, so the first mechanism cannot satisfy obligation 4 or
  obligation 9 on its own; if it is chosen, the decision MUST also say how a
  non-default endpoint or instance root is handled.
- OPEN: the equivalent on Linux (for example a systemd user unit) and on
  Windows is not decided; obligations 3 to 9 apply on every platform.
- OPEN: whether the generated project ships a launchd service definition
  (and the Linux and Windows equivalents), where it is installed, and what
  program path, arguments, environment and working directory it names, are not
  decided. Obligation 2 needs that definition to exist, because it is what
  "the same way" is measured against; until it and the mechanism are decided,
  the equality clause of obligation 2 cannot be checked, and the binding floor
  on every platform is obligation 3 together with: the daemon does not inherit
  the CLI's working directory, terminal, standard streams or session. A
  shipped definition is a rendered file, so it also changes the layout owned by
  [REQ-RUN-0301](requirements.md).
- OPEN: how auto-start is disabled (a command-line option, a configuration
  value, an environment variable, or more than one) is not decided.
- OPEN: how long the CLI waits for the started daemon to accept a
  connection, and how it polls, are not decided.
- OPEN: which crate holds the auto-start code is not decided. Each option
  has a cost. The generated `cli` crate conflicts with nothing, but the code is
  then copied into every project and cannot be upgraded
  ([ADR-RUN-0002](architecture.md)). `sc-transport` requires narrowing the
  crate-wide ban on process spawning in
  [REQ-TRN-0006](sc-transport/requirements.md) (its criterion on
  `std::process::Command` and `tokio::process`) to the `Client` code, and
  amending [NFR-TRN-0001](sc-transport/requirements.md) if a dependency is
  added. A new library crate requires amending
  [REQ-RUN-0001](requirements.md) (exactly four workspace crates),
  [REQ-RUN-0005](requirements.md), [ADR-RUN-0002](architecture.md) and
  [ADR-RUN-0003](architecture.md), and the crate must itself satisfy
  [NFR-RUN-0001](requirements.md).
- OPEN: whether the generated CLI also provides explicit `daemon start`
  and `daemon stop` subcommands is not decided
  ([REQ-RUN-0202](requirements.md)).

---

# Requirement: Layout of the workspace rendered from `template/`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-RUN-0301: Layout of the workspace rendered from `template/`

**Status:** Active  

### Requirement Statement

Definitions: CLI means command-line interface. MCP means Model Context Protocol. TCP means Transmission Control Protocol. SQL means Structured Query Language. The template is the cargo-generate source tree under `template/` in the `randlee/sc-runtime` repository. An answers file is a file in `wizard/fixtures/`. A rendered project is the directory that `just new <dest> --var-file <answers file>` writes. A layout is a database-store combination of [REQ-RUN-0326](requirements.md): a no-database layout has no store, a one-store layout has one store, and a two-store layout has `store-sqlite` and one remote-engine store. A role is the suffix in the package name `<project>-<role>` ([REQ-RUN-0329](requirements.md)); `api-types` names `<project>-types`, and every other bare role in backticks (`schema`, `service`, `daemon`, `cli`, `store-sqlite`, `store-postgres`, `store-mysql`, `store-adaptor`) names `<project>-<role>`. A store is a `store-sqlite`, `store-postgres` or `store-mysql` crate. A remote-engine store is `store-postgres` or `store-mysql`. The adaptor is the `store-adaptor` crate. A path that begins `crates/` names a file of a rendered project, and a statement about such a path applies to every rendered project. A manifest is a `Cargo.toml` file under `crates/`. A dependency entry for a crate is an entry of a manifest's `[dependencies]` table whose key or `package` value names the crate's package. A non-remote-engine manifest is a manifest outside `crates/store-postgres/` and `crates/store-mysql/`. A non-daemon manifest is a manifest outside `crates/daemon/`. A SQL crate is `sqlx` or `sc-runtime-db-sqlite`. The consumer set is `service`, `daemon` and `cli`. The CLI exclusion set is `<project>-service`, every `<project>-store-*` package, `<project>-daemon`, `sc-runtime`, `sqlx`, `axum` and `rmcp`. A generated feature is a `[features]` entry of the `daemon` manifest. The CLI tree is the output of `cargo tree -p <project>-cli -e normal --prefix none` in a rendered project. The public crates are the REQ-RUN-0002 package set: `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`, `sc-runtime-db-sqlite`, `sc-storage` and `sc-monty`. Non-test source is a crate's files outside `tests/` and outside `#[cfg(test)]` code. A numeric TCP port literal is a number in source or configuration text that names a TCP port. The local override file is `config/local.json`. v0.1 is the first release.

Scope: this layout is the interim Phase D wiring form. SQLite storage uses `sc-runtime-db-sqlite` and the shared `Storage<T>` command path (PRD §3/§4, ADR-RUN-0020/0021); the old SQLite-on-sqlx implementation is not required. Phase D command paths may be written directly, with no custom Tcli-to-Tsql transform; later consumer-side generation remains permitted. This record binds the adaptor crate and its dependency edges, and no synchronization algorithm, schedule, direction or conflict policy (owner, 2026-11-12). Which stores render follows [REQ-RUN-0326](requirements.md); v0.1 rendered `crates/store-sqlite` only. `AGENTS.md` and `CLAUDE.md` are produced by `sc-compose` from `template/AGENTS.md.j2` and `template/CLAUDE.md.j2` ([REQ-RUN-0305](requirements.md)). TCP ports come from project configuration; each project's port range is allocated in the synaptic-canvas port registry. This record and [ADR-RUN-0303](architecture.md) own the crate graph of the generated workspace. The layout tree:

```text
<project>/
  Cargo.toml                  # workspace
  crates/
    api-types/                # always
    schema/                   # when the layout has a store
    store-sqlite/             # when the layout has store-sqlite
    store-postgres/           # when the layout's remote engine is Postgres
    store-mysql/              # when the layout's remote engine is MySQL
    store-adaptor/            # when the layout has two stores
    service/                  # always
    daemon/                   # always; src/main.rs, src/routes.rs, src/mcp.rs
    cli/                      # always
  config/default.json
  Justfile
  AGENTS.md
  CLAUDE.md
  .gitignore
  .github/workflows/ci.yml
```

**MUST statements:**

- `cargo generate --path template` run in a checkout of `randlee/sc-runtime` MUST exit with status 0.
- The template MUST be configured by `template/cargo-generate.toml`.
- The root `Cargo.toml` of every rendered project MUST contain a `[workspace]` table.
- Every rendered project whose answers file sets `mcp` to `true` MUST contain `crates/daemon/src/mcp.rs`.
- The rendered `.gitignore` MUST contain the line `config/local.json`.
- `crates/api-types/Cargo.toml` MUST contain a dependency entry for `serde`.
- `crates/api-types/Cargo.toml` MUST contain a dependency entry for `schemars`.
- `crates/api-types/Cargo.toml` MUST contain a dependency entry for `utoipa`.
- When `crates/store-sqlite/` exists, `crates/store-sqlite/Cargo.toml` MUST contain a dependency entry for `sc-runtime-db-sqlite`.
- When `crates/store-postgres/` exists, the `sqlx` dependency entry of `crates/store-postgres/Cargo.toml` MUST enable the `postgres` feature.
- When `crates/store-mysql/` exists, the `sqlx` dependency entry of `crates/store-mysql/Cargo.toml` MUST enable the `mysql` feature.
- On a two-store layout, `crates/store-adaptor/Cargo.toml` MUST contain a dependency entry for `schema`.
- On a two-store layout, `crates/store-adaptor/Cargo.toml` MUST contain a dependency entry for `store-sqlite`.
- On a two-store layout, `crates/store-adaptor/Cargo.toml` MUST contain a dependency entry for the layout's remote-engine store.
- `crates/service/Cargo.toml` MUST contain a dependency entry for `api-types`.
- `crates/service/Cargo.toml` MUST contain a dependency entry for `sc-command`.
- On a one-store layout, `crates/service/Cargo.toml` MUST contain a dependency entry for the layout's store.
- On a two-store layout, `crates/service/Cargo.toml` MUST contain a dependency entry for `store-adaptor`.
- A file under `crates/service/` MUST contain the declaration `struct Stores`.
- `crates/daemon/Cargo.toml` MUST contain a dependency entry for `service`.
- `crates/daemon/Cargo.toml` MUST contain a dependency entry for `api-types`.
- `crates/daemon/Cargo.toml` MUST contain a dependency entry for `sc-runtime`.
- `crates/daemon/Cargo.toml` MUST contain a dependency entry for `sc-config`.
- `crates/daemon/Cargo.toml` MUST contain a dependency entry for `sc-transport`.
- `crates/daemon/Cargo.toml` MUST contain a dependency entry for `sc-observability`.
- `crates/daemon/Cargo.toml` MUST contain a dependency entry for `sc-observability-types`.
- The `sc-command` dependency entry of `crates/daemon/Cargo.toml` MUST enable the `server` feature.
- The `sc-cron` dependency entry of `crates/daemon/Cargo.toml` MUST set `optional = true`.
- The `sc-monty` dependency entry of `crates/daemon/Cargo.toml` MUST set `optional = true`.
- The `cron` list of the `[features]` table of `crates/daemon/Cargo.toml` MUST contain `dep:sc-cron`.
- The `monty` list of the `[features]` table of `crates/daemon/Cargo.toml` MUST contain `dep:sc-monty`.
- The output of `cargo tree -p <project>-daemon -e normal -f '{p} {f}' --features cron,monty` in a rendered project MUST show the `monty` feature on the `sc-cron` line.
- A file under `crates/daemon/src/` MUST contain the declaration `struct AppConfig`.
- `crates/cli/Cargo.toml` MUST contain a dependency entry for `api-types`.
- `crates/cli/Cargo.toml` MUST contain a dependency entry for `sc-transport`.
- `crates/cli/Cargo.toml` MUST contain a dependency entry for `sc-command`.
- `crates/cli/Cargo.toml` MUST contain a dependency entry for `sc-config`.

**MUST NOT statements:**

- On a no-database layout, `crates/schema/` MUST NOT exist.
- On a layout with fewer than two stores, `crates/store-adaptor/` MUST NOT exist.
- The `crates/` directory of a store that the layout does not have MUST NOT exist.
- A rendered project whose answers file sets `mcp` to `false` MUST NOT contain `crates/daemon/src/mcp.rs`.
- `crates/api-types/Cargo.toml` MUST NOT contain a dependency entry for a crate of the generated workspace.
- `crates/schema/Cargo.toml` MUST NOT contain a dependency entry for a SQL crate.
- `crates/schema/Cargo.toml` MUST NOT contain a dependency entry for a store.
- A non-remote-engine manifest MUST NOT contain a dependency entry for `sqlx`.
- `crates/store-sqlite/Cargo.toml` MUST NOT contain a dependency entry for a crate of the consumer set.
- The `sc-command` dependency entry of `crates/service/Cargo.toml` MUST NOT set `default-features = false`.
- On a no-database layout, `crates/service/Cargo.toml` MUST NOT contain a dependency entry for a crate of the generated workspace other than `api-types`.
- `crates/daemon/Cargo.toml` MUST NOT contain a dependency entry for a store.
- A manifest outside `crates/store-adaptor/` and `crates/service/` MUST NOT contain a dependency entry for a store.
- On a two-store layout, `crates/service/Cargo.toml` MUST NOT contain a dependency entry for a store.
- The output of `cargo tree -p <project>-daemon -e normal --prefix none --features monty` in a rendered project MUST NOT show a line beginning with `sc-cron `.
- The `default` list of the `[features]` table of `crates/daemon/Cargo.toml` MUST NOT contain `cron`.
- The `default` list of the `[features]` table of `crates/daemon/Cargo.toml` MUST NOT contain `monty`.
- The output of `cargo tree -p <project>-cli -e normal -f '{p} {f}'` in a rendered project MUST NOT show the `server` feature on the `sc-transport` line.
- The output of `cargo tree -p <project>-cli -e normal -f '{p} {f}'` in a rendered project MUST NOT show the `server` feature on the `sc-command` line.
- The CLI tree MUST NOT contain a line beginning with a crate name of the CLI exclusion set followed by a space.
- A non-daemon manifest MUST NOT contain a dependency entry for `sc-runtime`.
- A file under `template/` MUST NOT contain a numeric TCP port literal.
- The non-test source of a public crate MUST NOT contain a numeric TCP port literal.

### Rationale

`sqlx` needs one crate per database for its compile-time checks. The CLI must not link daemon dependencies, so `cli` and `daemon` are separate and share structs through `api-types`. `service`, whose functions take `&Stores`, is the one path to SQL for REST and MCP; it lists `sc-command` directly because it returns `OpError` and may not depend on `sc-runtime`. The `daemon` is the composition root: it names `SharedCanonicalLog` publicly, and `sc-runtime` does not re-export `sc-command`. Optional `cron` and `monty` are embedding policy. The adaptor is where sync code reaches both stores.

### Success Criteria

1. For each answers file, render the project with `just new <dest> --var-file <fixture>`: every layout-tree path that applies to the layout exists, no store or adaptor directory that does not apply exists, and `crates/daemon/src/mcp.rs` exists exactly when the fixture's `mcp` is `true`.
2. In each rendered project, `cargo metadata --format-version 1` shows exactly the packages `<project>-<role>` for the roles the layout renders: `types`, `schema`, each store, the adaptor on a two-store layout, `service`, `daemon` and `cli`.
3. In each rendered project, `cargo tree -p <project>-cli -e normal --prefix none` prints no line beginning with `sqlx `, `axum `, `rmcp `, `sc-runtime `, `<project>-service `, a store package or `<project>-daemon `.
4. In each rendered project, `sqlx` appears under `[dependencies]` only in remote-engine store manifests; the SQLite wiring crate depends on `sc-runtime-db-sqlite`. A two-store layout's adaptor depends on both stores and its `service` on the adaptor; a one-store layout's `service` depends on that store. No manifest other than the adaptor's and a one-store layout's `service` manifest has a dependency entry for a store.
5. The rendered `.gitignore` contains the line `config/local.json`.
6. The rendered project contains no file ending in `.j2` and no file from `wizard/` or `scripts/`.
7. `grep -rn 'struct Stores' crates/` in a rendered project matches only under `crates/service/`.
8. `template/` and the non-test source of the public crates contain no numeric TCP port literal (if the default-port question in Implementation Notes is decided for a default, the one registered value in `template/config/default.json` is exempt).
9. A rendered daemon manifest names `sc-config`, `sc-transport`, `sc-command` with `server`, `sc-runtime`, `sc-observability` and `sc-observability-types` directly. With default features its normal tree contains no `sc-cron`; `--features cron` adds `sc-cron` without Monty; `--features monty` adds `sc-monty` without `sc-cron`; `cron` and `monty` together enable `sc-cron/monty`.

### Implementation Notes

**Key Considerations:**

- Implementation scope: whether the template renders a daemon service definition (launchd property list on macOS, and the Linux and Windows equivalents), and where, is not decided; [REQ-RUN-0206](requirements.md) owns the question for the CLI's auto-start.
- Implementation scope: whether v0.1 renders a default TCP port into `config/default.json` is not decided. If it does, that one value in `template/config/default.json` is exempt from the port-literal prohibition and comes from the project's range in the synaptic-canvas port registry.
- Implementation scope: whether the template renders a `config/local.json` file or only its `.gitignore` entry, and the keys of `config/default.json`, are not stated.

# Requirement: Template example operations `widget.create`, `widget.get`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-RUN-0302: Template example operations `widget.create`, `widget.get`

**Status:** Active  

### Requirement Statement

Definitions: The generated project is the project rendered from `template/`. The worked example is the pair of operations `widget.create` and `widget.get`, with the pieces listed in the worked-example table. The worked-example table is the table in this requirement headed Piece, Where in the generated project, What it MUST contain. Its three surfaces are a REST handler, an MCP tool and a CLI command. A handler is a REST handler of the worked example. A tool is an MCP tool of the worked example. The service function is the async operation function shared by those surfaces. In this example, a command is the generated `clap` CLI command in the table; it reaches the service function through the matching daemon HTTP route. SQL means Structured Query Language, the database query text implemented by the storage layer. A piece matches a row of the worked-example table when it has every item that row's What it MUST contain cell lists for its operation. `rmcp` is the Rust crate that implements MCP, and `StreamableHttpService` is its HTTP service type. `initialize` and `tools/call` are MCP request methods; a session header is the HTTP header that identifies an MCP client session. Session state is server-owned state that tracks an MCP client session; an MCP session store is project code that keeps that state.

Command paths may be written directly in Phase D. Direct Tcli-to-Tsql conversion needs no custom transform. Later consumer-side generation against the delivered framework remains permitted.

The widget example is the interim, hand-written form of the command pattern. "Typed query functions" in `store-sqlite` and "one service function is the only path to SQL" describe that interim form; the target is `Tcli`, `Tsql`, a transform and `Storage<Tsql>`.

2026-10-25 by [REQ-RUN-0331](requirements.md). The widget pair remains the worked example. A layout with a store also has the general database commands in that requirement.

Related requirement: [REQ-RUN-0331](requirements.md). The widget pair remains the worked example. A layout with a store also has the general database commands in that requirement.

| Piece | Where in the generated project | What it MUST contain |
|---|---|---|
| Types | `crates/api-types` | `pub struct CreateWidget { pub name: String }` and a response struct `Widget`, each deriving `Serialize`, `Deserialize`, `schemars::JsonSchema` and `utoipa::ToSchema`; plus the request struct for `widget.get` |
| Storage | `crates/store-sqlite` | a migration under `migrations/` creating the widget table, and typed query functions to insert and to fetch a widget |
| Service | `crates/service` | `pub async fn create_widget(s: &Stores, input: CreateWidget) -> Result<Widget, OpError>` and the matching function for `widget.get` |
| REST | `crates/daemon/src/routes.rs` | `POST /ops/widget.create` and `POST /ops/widget.get`, each an `axum` handler annotated with `#[utoipa::path(...)]`, registered with `utoipa_axum` `routes!` on an `OpenApiRouter`, returning `Envelope<Widget>` |
| MCP | `crates/daemon/src/mcp.rs` (only when option `mcp` is `true`) | `rmcp` `#[tool]` methods named `widget_create` and `widget_get`, taking `Parameters<...>` of the same `api-types` request structs and returning through `.into_mcp()` |
| CLI | `crates/cli` | one `clap` command per operation; each builds the `api-types` request struct, sends it with `sc_transport::Client::post` (method name illustrative until pinned by [REQ-TRN-0005](sc-transport/requirements.md)) to the matching `/ops/...` path, and supports `--json` and the global `--endpoint` option ([REQ-RUN-0310](requirements.md)) |
| Test | the generated project's test suite | at least one test that starts a daemon with `sc_runtime::testing::DaemonFixture` and exercises the example through REST, through MCP (when `mcp` is `true`) and through the production CLI command path, composed inside the test process without running the CLI binary. The test runs on a developer's host, so it MUST keep its database under the fixture's temporary instance root ([NFR-RUN-0010](requirements.md)). This contract does not prescribe the deferred REQ-RUN-0312 helper or test-generation architecture. |

Which derives the struct carries, and that the same struct is the rmcp
`Parameters<T>` input, is conditional on
[ADR-RUN-0203](architecture.md) (spike row V2 of
[REQ-RUN-0102](requirements.md)). Sprint a-1 made that ADR Active and
proved V2, so these clauses stand. The rule that every surface calls one service
function does not depend on it.

2. The route shape `POST /ops/{name}` is the example's convention only.
   Nothing in the `sc-runtime` crate may depend on it.

REQ-RUN-0201 states the rule in full: one service function is the only path to SQL.

**MUST statements:**

- The generated project MUST contain exactly one worked example.
- The set of operations in the worked example MUST equal the pair `widget.create`, `widget.get`.
- `crates/daemon/src/routes.rs` in the generated project MUST contain a handler that matches the REST row of the worked-example table for operation `widget.create`.
- When option `mcp` is `true`, `crates/daemon/src/mcp.rs` in the generated project MUST contain a tool that matches the MCP row of the worked-example table for operation `widget.create`.
- `crates/cli` in the generated project MUST contain a `clap` command that matches the CLI row of the worked-example table for operation `widget.create`.
- `crates/daemon/src/routes.rs` in the generated project MUST contain a handler that matches the REST row of the worked-example table for operation `widget.get`.
- When option `mcp` is `true`, `crates/daemon/src/mcp.rs` in the generated project MUST contain a tool that matches the MCP row of the worked-example table for operation `widget.get`.
- When option `mcp` is `true`, `mcp::service` in `crates/daemon/src/mcp.rs` MUST return a `StreamableHttpService`.
- When option `mcp` is `true`, a `tools/call` sent to the service returned by `mcp::service` with no prior `initialize` and no session header MUST succeed.
- The generated project MUST contain a CLI command for `widget.get`, built as the CLI row of the worked-example table states.
- Every handler MUST call the service function of its operation.
- Every handler MUST contain no SQL.
- Every tool MUST call the service function of its operation.
- Every tool MUST contain no SQL.
- Every command MUST call the service function of its operation.
- Every command MUST contain no SQL.

**MUST NOT statements:**

- The template MUST NOT contain a second example operation.
- The template MUST NOT contain an MCP session store.

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
   and `/ops/widget.get`. With `mcp` on and the general database surface off,
   or with no database, an MCP `tools/list` returns exactly `widget_create`
   and `widget_get`. With `mcp` on and the general surface on, `tools/list`
   also returns the [REQ-RUN-0331](requirements.md) commands, including
   `delete row`. 4. `grep -rn 'ops/' crates/` in the `randlee/sc-runtime` repository (the
   public package set in REQ-RUN-0002) finds no match outside tests and documentation.
5. In a project generated with `mcp` equal to `true`, a test sends an MCP
   `tools/call` of `widget_create` to `/mcp` with no prior `initialize`
   request and no session header, and it succeeds.
6. Inspect `template/`: it contains no MCP session manager and no session
   store, and `mcp::service` builds the `StreamableHttpService` in stateless
   mode.

### Implementation Notes

**Key Considerations:**

- Implementation scope: which generated crate or directory hosts the example test, and how
  it reaches the `cli` crate's request-construction and rendering functions and
  the `daemon` crate's router (library targets in `cli` and `daemon`, or
  another arrangement), is not decided. Whichever crate hosts it needs
  `sc-runtime` as a dev-dependency; if that crate is `cli`, that is why the CLI
  dependency rules count normal dependencies only
  ([NFR-RUN-0001](requirements.md)).
- Implementation scope: the name of the example test function is not decided.
- Implementation scope: the fields of `Widget`, the name and fields of the `widget.get`
  request struct, the name of the `widget.get` service function, the widget
  table's columns, and the spelling of the two CLI commands and their
  arguments are not stated.

### Related Documents

**Requirements:**

- REQ-RUN-0331 — amended this requirement on 2026-10-25

## REQ-RUN-0303: Template options in v0.1: project name, `db`, `mcp`, licence

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: Whole-file selection means determining whether an entire file is present in a generated project for the selected option values. A Liquid control tag is a tag written `{% ... %}`, such as `{% if mcp %}` or `{% endif %}`. A conditional line is a line containing a Liquid control tag, or a line between an opening control tag (such as `{% if mcp %}`) and its closing tag (such as `{% endif %}`); each such line counts once. A Liquid value substitution is a tag written `{{ ... }}`; it does not make a line conditional. This counting rule is the one in [REQ-RUN-0304](requirements.md).

"v0.1" is the first release of the template. The generation options are
defined in `wizard/answers.schema.json` and mirrored as placeholders in
`template/cargo-generate.toml`.

1. The list of options (project name, a string; `db`, a string enum of
   `none`, `sqlite`, `postgres`, `mysql`, `sqlite-postgres`, and
   `sqlite-mysql`; `mcp`, a boolean; the licence, a string
   whose default is `MIT`), their types and the rule
   that the list is closed are owned by [REQ-RUN-0401](requirements.md).
   `both` is not a value. Amended 2026-10-25.
   This item owns what each option does to the rendered project:

   | Option | Effect on the rendered project |
   |---|---|
   | project name | names the generated project |
   | `db` | selects the database layout in [REQ-RUN-0326](requirements.md) |
   | `mcp` | `true` renders `crates/daemon/src/mcp.rs`, the `mod mcp;` line and the `.mcp(...)` line in `crates/daemon/src/main.rs`; `false` omits all three |
   | licence | sets the licence stated in the `Cargo.toml` of every crate of the generated project; `MIT` when not supplied. The rendered project MUST NOT state `MIT` when a different licence was supplied |

   **OPEN:** whether the template also renders a licence file into the
   generated project, and for which licence values, is not decided. The
   licence text of a project that is not MIT licensed is not something this
   template can supply.

2. [REQ-RUN-0326](requirements.md) owns the database layouts. v0.1 shipped
   the SQLite store only. Sprint `b-8` accepts the six `db` values above.
   A value outside that enum, including `both`, fails schema validation in
   `scripts/new_project.py` before `cargo generate` runs.
   [REQ-RUN-0329](requirements.md) names each generated package with the
   project name. [REQ-RUN-0330](requirements.md) puts row structs in
   `<project>-schema`. Amended 2026-10-25.
3. With `mcp` equal to `false`, `crates/daemon/src/mcp.rs` is ignored. The `ignore` mechanism and its syntax are recorded in [ADR-RUN-0402](architecture.md), which sprint a-1 made Active.
5. That the schema's keys and the placeholders of
   `template/cargo-generate.toml` always match, so a new option is added to
   both together, is owned by [REQ-RUN-0402](requirements.md) (the key-set
   test).

`both` is outside the enum and fails schema validation
([REQ-RUN-0403](requirements.md)). The v0.1 reserved-database check is
removed by sprint `b-8`. Amended 2026-10-25.

**MUST statements:**

- Whole-file selection MUST be done only by the `ignore` lists in `template/cargo-generate.toml`.
- The total number of conditional lines across all `.rs` files under `template/` MUST be fewer than ten.
### Rationale

Phase b generates every layout in [REQ-RUN-0326](requirements.md), including
MySQL, in one phase with the pairing and the adaptor. v0.1 shipped SQLite
only. The answers schema is a published contract that the
sc-lint project also reads, so its shape should not change when Postgres
arrives; reserving the enum values now keeps it stable. Varying whole files,
not lines within files, keeps every template source file readable and
compilable as Rust.
### Success Criteria

1. `template/cargo-generate.toml` defines a placeholder named `db`, a
   placeholder named `mcp` and a placeholder for the licence (its name
   follows the licence OPEN of [REQ-RUN-0401](requirements.md)). Once the
   project-name question above is decided:
   it defines, or does not define, a project-name placeholder as decided.
   (The schema side is checked by [REQ-RUN-0401](requirements.md).)
2. Generating from `wizard/fixtures/sqlite-mcp.json` (`db` `sqlite`, `mcp`
   `true`) yields a project containing `crates/daemon/src/mcp.rs` in which
   `just lint` and `just test` exit 0.
3. Generating from the fixture with `db` `sqlite` and `mcp` `false` yields a
   project without `crates/daemon/src/mcp.rs`, whose
   `crates/daemon/src/main.rs` contains no `.mcp(` call, in which `just lint`
   and `just test` exit 0.
4. A unit test runs the driver with an answers file whose `db` is `both`.
   The run exits non-zero, does not invoke `cargo generate`, and creates no
   destination directory. `both` fails schema validation. Amended 2026-10-25.
5. `template/cargo-generate.toml` contains a conditional `ignore` entry for
   `crates/daemon/src/mcp.rs` keyed on `mcp`.
6. Generating from the fixture whose licence is not `MIT`
   ([REQ-RUN-0403](requirements.md)) yields a project in which every crate's
   `Cargo.toml` states that licence, and
   `grep -rn 'MIT' --include=Cargo.toml .` in the project prints nothing.
7. Generating with plain `cargo generate` and accepting the licence
   placeholder's default yields a project in which every crate's
   `Cargo.toml` states `MIT`.
### Implementation Notes

**Key Considerations:**

- OPEN: the key name of the project-name option, and whether it is a
  template placeholder or only `cargo generate`'s `--name` argument, are not
  decided ([REQ-RUN-0402](requirements.md) owns the question).
- OPEN: the defaults of `db` and `mcp` are not decided.
- OPEN: the file names of the `mcp`-off fixture and of the invalid
  `db = postgres` answers file are not decided; only
  `wizard/fixtures/sqlite-mcp.json` is named.

---

## REQ-RUN-0304: Fewer than ten Liquid conditional lines in template Rust

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: A Liquid control tag is a tag written `{% ... %}`. A conditional line is a line that contains a Liquid control tag, or a line that lies between an opening control tag and its closing tag. A value substitution is a tag written `{{ ... }}`. A generated variant is the project rendered from `template/` for one combination of option values. The three permitted locations are the locations listed in item 2.

`cargo-generate` renders files under `template/` with the Liquid template
language. Liquid control tags are written `{% ... %}` (for example
`{% if mcp %}` and `{% endif %}`); value substitutions are written
`{{ ... }}`.

1. A conditional line is a line that contains a Liquid control tag, or a line that lies between an opening control tag and its closing tag. This counting rule is decided in this document.
2. Liquid control tags in template `.rs` files are permitted in exactly
   these three locations and nowhere else:
   - around fields of the project's `Stores` struct in the `service` crate
     (they depend on option `db`);
   - around the `mod mcp;` line of `crates/daemon/src/main.rs` (it depends
     on option `mcp`, because `mcp.rs` is omitted when `mcp` is `false`);
   - around the `.mcp(...)` line of the `Daemon::builder()` chain in
     `crates/daemon/src/main.rs` (it depends on option `mcp`).
3. For example, the `ignore` list includes `crates/daemon/src/mcp.rs` when MCP is disabled. The `ignore` mechanism is conditional on [ADR-RUN-0402](architecture.md), which sprint a-1 made Active.
5. Value substitutions `{{ ... }}`, such as the project name, are not
   conditional lines and are not limited by this requirement.

**MUST statements:**

- Across all `.rs` files under `template/`, the number of conditional lines MUST be fewer than ten in total.
- A difference between generated variants that is made neither by a value substitution nor by a conditional line in one of the three permitted locations MUST be made by including or excluding a whole file through a conditional `ignore` list in `template/cargo-generate.toml`.

**MUST NOT statements:**

- A Liquid control tag MUST NOT appear in a `.rs` file under `template/` unless it belongs to a tag pair that encloses only one of the three permitted locations: fields of the `Stores` struct in the `service` crate, the `mod mcp;` line of `crates/daemon/src/main.rs`, or the `.mcp(...)` line of the `Daemon::builder()` chain in `crates/daemon/src/main.rs`.
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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions:
sc-compose is the external command-line tool that validates and renders Jinja templates that begin with frontmatter.
Frontmatter is the block delimited by `---` lines at the start of a template file.
The sc-ai-cli agent-document form is a Jinja file that begins with frontmatter and that `sc-compose validate --mode file --file <path>` accepts with exit status 0, as the sc-ai-cli project's `AGENTS.md.j2` and `CLAUDE.md.j2` do.
Liquid is the template language that `cargo-generate` renders.
Liquid syntax intended for cargo-generate is markup that `cargo-generate` substitutes during `cargo generate`, such as the `{{project-name}}` placeholder or a `{% elsif %}` tag.
`exclude` is the list in `template/cargo-generate.toml` whose files `cargo-generate` copies byte-for-byte without Liquid rendering, as [ADR-RUN-0402](architecture.md) records.
The driver is the generation script `scripts/new_project.py` that `just new` runs.
`<dest>` is the destination directory of a driver run, into which `cargo generate` writes the generated project.
The answers JSON of a driver run is the answers object of that run after the driver has validated it against `wizard/answers.schema.json` and filled in the schema defaults. The driver gives the values of that one object to `cargo generate` and to every `sc-compose` step.
JSON means JavaScript Object Notation.
ADR means architecture decision record.
The wait call is the call in `scripts/new_project.py` that waits for the `cargo generate` process to exit.
A step is one external command that the driver runs, such as `cargo generate` or `sc-compose render`.
The failed step is the step whose command exited with a non-zero status.
An `sc-compose` step is a step that runs the executable named by the `SC_COMPOSE` environment variable, or the `sc-compose` found on `PATH` when `SC_COMPOSE` is unset.
An `sc-compose` step renders a template into a file when that step writes the result of rendering that template to that file.
`widget-app` is the `project_name` value in `wizard/fixtures/sqlite-mcp.json`.
An agent document is `<dest>/AGENTS.md` or `<dest>/CLAUDE.md`.
The template-variable input of an `sc-compose` step is the set of values that the step substitutes for the variables of the template it renders.
The exact `sc-compose` command line, including how the answers JSON and the input and output paths are passed, is not decided by this record.
The driver reports a step as the failed step by naming that step in the driver's standard error output.
This record owns the driver's sc-compose step, the failure behaviour of that step and the removal of the `.j2` files.

**MUST statements:**

- The file `template/AGENTS.md.j2` MUST exist.
- The file `template/CLAUDE.md.j2` MUST exist.
- `sc-compose validate --mode file --file template/AGENTS.md.j2` MUST exit with status 0.
- `sc-compose validate --mode file --file template/CLAUDE.md.j2` MUST exit with status 0.
- `template/AGENTS.md.j2` MUST have the sc-ai-cli agent-document form.
- `template/CLAUDE.md.j2` MUST have the sc-ai-cli agent-document form.
- The `exclude` list in `template/cargo-generate.toml` MUST contain `AGENTS.md.j2`.
- The `exclude` list in `template/cargo-generate.toml` MUST contain `CLAUDE.md.j2`.
- In `scripts/new_project.py`, the wait call MUST appear before every `sc-compose render` call.
- An `sc-compose` step of the driver MUST render `<dest>/AGENTS.md.j2` into `<dest>/AGENTS.md`.
- An `sc-compose` step of the driver MUST render `<dest>/CLAUDE.md.j2` into `<dest>/CLAUDE.md`.
- Every `sc-compose` step that renders an agent document MUST receive the answers JSON of its driver run as its template-variable input.
- After `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` exits with status 0, `<dest>/AGENTS.md` MUST contain `widget-app`.
- After `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` exits with status 0, `<dest>/CLAUDE.md` MUST contain `widget-app`.
- If an `sc-compose` step exits with a non-zero status, the driver MUST exit with a non-zero status.
- If an `sc-compose` step exits with a non-zero status, the failed step that the driver reports MUST be `sc-compose`.

**MUST NOT statements:**

- `template/AGENTS.md.j2` MUST NOT contain Liquid syntax intended for cargo-generate.
- `template/CLAUDE.md.j2` MUST NOT contain Liquid syntax intended for cargo-generate.
- After a `just new <dest>` command exits with status 0, `test -e <dest>/AGENTS.md.j2` MUST NOT exit with status 0.
- After a `just new <dest>` command exits with status 0, `test -e <dest>/CLAUDE.md.j2` MUST NOT exit with status 0.

### Rationale

Liquid (used by `cargo-generate`) and Jinja (used by `sc-compose`) both write
substitutions as `{{ }}`, so one engine cannot safely process a file written
for the other. The agent documents stay validated Jinja because that is what
the sc-ai-cli templates and the `p3-nuget-template` pipeline already use;
each tool does the job it is built for and no render engine is written here.
### Success Criteria

1. `template/cargo-generate.toml` lists `AGENTS.md.j2` and `CLAUDE.md.j2`
   under `exclude`.
2. Checks the two render statements and the two `widget-app` statements by
   a command run: run `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json`
   with the real `sc-compose`; after it exits with status 0, the files
   `<dest>/AGENTS.md` and `<dest>/CLAUDE.md` exist and each contains
   `widget-app`, the fixture's `project_name` value. `cargo generate` copies
   the two `.j2` files without rendering them (they are in `exclude`), so
   `widget-app` in those files shows that an `sc-compose` step rendered them.
3. In the same `<dest>`, `find . -name '*.j2'` prints nothing.
4. In the same `<dest>`, `grep -n '{{\|{%' AGENTS.md CLAUDE.md` prints
   nothing, showing no unrendered template syntax remains.
5. Checks the two `sc-compose` failure statements: the `sc-compose` case of
   `test_each_pipeline_step_failure_stops` in
   `tests/integration/generation/test_driver.py` places an `sc-compose`
   executable that exits with status 1 on the driver's `PATH`, runs the
   driver, and asserts that the driver exits with a non-zero status and that
   its standard error output is the single line
   `GEN.STEP_FAILED step is sc-compose — <recovery>`, which names
   `sc-compose` as the failed step.
6. Checks the answers-input statement by reading `render_documents` in
   `scripts/new_project.py`: every `sc-compose render` command it builds
   passes, as the variable file, the JSON file that
   `write_answers_for_compose` writes from the run's validated answers. The
   `widget-app` check in criterion 2 supports this statement but does not by
   itself show that every answer reaches `sc-compose`.
### Implementation Notes

**Key Considerations:**

- OPEN: the exact `sc-compose` command line (how the answers JSON and the
  input and output paths are passed) is not stated.
- OPEN: the frontmatter keys these two files must carry are not stated;
  they follow whatever `sc-compose` validates.

---

## REQ-RUN-0306: Plain `cargo generate` works without wizard or driver

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: ADR means architecture decision record. The direct path is the cargo generate command below, run without the generation driver, together with the hook scripts it runs and `cargo build --workspace` in the default project. The template tree is template/, also used by scripts/new_project.py. This path means direct cargo generate from the template/ tree, without scripts/new_project.py or Wyvern. sc-compose is the agent-document renderer used by the driver, not by the direct cargo-generate path. The direct command is `cargo generate --git https://github.com/randlee/sc-runtime template --name <project>`. An option is a choice the template offers, declared as a placeholder in `template/cargo-generate.toml`. A `[placeholders.*]` table is a TOML table in `template/cargo-generate.toml` whose header line starts with `[placeholders.`, such as `[placeholders.db]`; each one declares one option. A `prompt` key is a `prompt = "..."` line in such a table; its text is the question the direct command shows for that option. A `default` key is a `default = ...` line in such a table; its value is the answer the direct command uses when the person accepts the default. Wyvern is the wizard that `just new` runs by default. Files that exist only for this path are files maintained exclusively for direct generation rather than shared with the driver's template tree. The generated project is the Cargo workspace rendered from this repository's template/ subtree. <project> is the project name supplied through cargo-generate's --name option. The project directory is the directory that cargo-generate creates for the generated project. A hook script is a file listed in the `[hooks]` table of `template/cargo-generate.toml`. The generation files are `template/cargo-generate.toml` together with the hook scripts. The default project is the project directory that the direct command creates when every option is left at its default. A branch is a git branch of `randlee/sc-runtime`. A file or branch exists only for the direct path when the direct command uses it and the driver `scripts/new_project.py` does not. The direct path requires a program or file when, with that program or file absent from the machine and the repository checkout, the direct command, a hook script or `cargo build --workspace` in the default project fails. Each MUST NOT statement on what the direct path requires is checked by a CLI run: the direct command and then `cargo build --workspace` are run with that program off `PATH` or that directory deleted from the checkout, and each must exit 0.

The normal way to generate a project is `just new <dest>`, which runs the
driver `scripts/new_project.py` and, by default, the Wyvern wizard. This
requirement covers a person who has neither.



```text
   cargo generate --git https://github.com/randlee/sc-runtime template --name <project>


```

   The second positional argument `template` tells `cargo-generate` to use
   the `template/` subfolder of the repository as the template root.
5. That this is the meaning of "works" is decided in this document.

A CI job accepts every default with no prompt by running
`cargo generate --path template --template-values-file <file> --name <project> --silent`
with stdin closed. Sprint a-1 proved that command, including an array value,
on cargo-generate 0.25.1. The syntax and pin are in Active
[ADR-RUN-0402](architecture.md). The destination directory must already exist.

**MUST statements:**

- The direct command MUST render the `template/` subtree as a Cargo workspace.
- The direct command MUST name the project directory `<project>`.
- The direct command MUST prompt in the terminal for each option.
- Every `[placeholders.*]` table in `template/cargo-generate.toml` MUST have a `prompt` key.
- Where an option has a default, its `[placeholders.*]` table in `template/cargo-generate.toml` MUST have a `default` key.
- Where an option has a default, its placeholder `default` in `template/cargo-generate.toml` MUST be identical to that option's default in `wizard/answers.schema.json`.
- The template tree MUST be the same one the driver uses.
- `cargo build --workspace` MUST exit 0 in the default project.

**MUST NOT statements:**

- The direct path MUST NOT require Wyvern.
- The direct path MUST NOT require `sc-compose`.
- The direct path MUST NOT require Python.
- The direct path MUST NOT require any file under the top-level `wizard/` directory.
- The direct path MUST NOT require any file under the top-level `scripts/` directory.
- There MUST NOT be a second template that exists only for the direct path.
- There MUST NOT be a branch that exists only for the direct path.
- The repository MUST NOT contain a file that exists only for the direct path.

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
6. Reading the generation files shows that none of them invokes Wyvern,
   `sc-compose` or Python, and none names a path under the top-level
   `wizard/` or `scripts/` directory. This reading supports the MUST NOT
   statements on what the direct path requires; it does not replace them.
7. A CLI run of `cargo generate --path template --name plain-defaults` with
   every default, then `cargo build --workspace` in the project it creates,
   exits 0 at each step when no `wyvern`, `sc-compose`, `python` or `python3`
   program is on `PATH` and the top-level `wizard/` and `scripts/` directories
   are deleted from the checkout.
### Implementation Notes

**Key Considerations:**

- OPEN: in this path nothing runs `sc-compose`, so the generated project
  keeps `AGENTS.md.j2` and `CLAUDE.md.j2` unrendered and has no `AGENTS.md` or
  `CLAUDE.md`. Whether that is accepted, or documented with a manual
  `sc-compose` step, is not decided.
- OPEN: the default values of the options are not decided, so which variant
  "every default" produces is not decided.

---

## REQ-RUN-0307: sc-lint 0.8.6 consumer install

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions: A generated project is the consumer repository rendered from template/.
sc-lint is the tool installing the common SC repository commands.
just is the command runner exposing named recipes.
A public recipe is a command listed by just --list in the generated project.
A recipe is a named command in the generated project’s Justfile.
db-prepare is the consumer-owned recipe regenerating checked-in sqlx offline query metadata for a selected sqlx store.
A selected sqlx store is the crates/store-postgres or crates/store-mysql crate in a generated project, whose sqlx offline query metadata is that crate's .sqlx/ directory.
CI means continuous integration.
A run step is a shell command in a GitHub Actions workflow run: entry.
A linting run step executes lint checks, including formatting or clippy analysis.
A testing run step executes automated tests.
The rendered workflow is the generated project’s .github/workflows/ci.yml.

The two recipe-name constraints are checked by running `just --list` in the generated project.

Every SC repository exposes the common base commands installed by sc-lint 0.8.6 through `sc-lint init --just`. The placeholder `Justfile` and prototype-handoff order are retired.

The template commits `template/sc-lint.toml` with `minimum_version` 0.8.6 and `template/.sc-lint/bootstrap.py`. `sc-lint.toml` records the consumer minimum version. The only additional invocation wiring is the three analyzer entries below.

The public recipes are `setup`, `lint`, `test` and `upgrade`; `lint` and `test` are the standard SC base command names. The consumer owns `db-prepare`. `AGENTS.md` and `CLAUDE.md` refer to `setup`, `lint`, `test`, `upgrade` and `db-prepare` by name only.

Tests starting a separate daemon process never execute on a developer’s host ([NFR-RUN-0010](requirements.md)); whether `just test` also runs such tests inside an isolated machine remains the question recorded there. Postgres and MySQL are where sqlx is used (PRD-STO-2); the SQLite store uses rusqlite, so `db-prepare` is not required to produce sqlx offline query metadata for `crates/store-sqlite`. This item owns recipe names and committed template files; a-6 owns generated consumer installation. The no-hand-written-lint-knowledge rule is [NFR-RUN-0006](requirements.md).

**MUST statements:**

- When a later `sc-lint init` completes in a generated project, `just --list` in that project MUST still list `db-prepare`.
- In a generated project with a selected sqlx store, `just db-prepare` MUST regenerate that store's `.sqlx/` directory.
- Every linting `run:` step in the rendered `.github/workflows/ci.yml` MUST invoke only `just lint`.
- Every testing `run:` step in the rendered `.github/workflows/ci.yml` MUST invoke only `just test`.

**MUST NOT statements:**

- The generated project’s public `lint` recipe MUST NOT be renamed.
- The generated project’s public `test` recipe MUST NOT be renamed.

### Rationale

sc-lint owns the `just` and lint install, so a generated project picks up
rule changes from sc-lint. The template commits the consumer pin and the
bootstrap file that `sc-lint init --just` expects, and it does not freeze a
hand-written lint policy. `db-prepare` is the project's own recipe and has to
remain after a later init.
### Success Criteria

1. `template/sc-lint.toml` sets `minimum_version` to `0.8.6`, and
   `template/.sc-lint/bootstrap.py` exists. `template/` contains no
   placeholder `Justfile` that stands in for this install.
2. In a project generated from `wizard/fixtures/sqlite-mcp.json`,
   `just --list` shows `setup`, `lint`, `test`, `upgrade`, and `db-prepare`.
3. In that project `just lint` and `just test` exit 0 with no edits.
4. In a project generated from `wizard/fixtures/postgres.json`, copy
   `crates/store-postgres/.sqlx/` to a directory outside the project, delete
   the original, and run `just db-prepare`: it recreates the directory, and
   `diff -r` of the copy and the recreated directory prints nothing. In the
   `wizard/fixtures/sqlite-mcp.json` project, after customization, repeated native `sc-lint init --just`
   and the public `just setup` and `just upgrade` commands succeed and preserve
   the SDK's custom profiles, `db-prepare`, and edited external boundary policy.
5. The rendered `.github/workflows/ci.yml` has at least one `run:` step that
   is `just lint` and at least one that is `just test`; every `run:` step
   that lints or tests is one of those two; the file contains no direct
   `cargo test`, `cargo clippy` or `cargo fmt` invocation.
6. No maintainer boundary manifests, clippy, rustfmt, deny, custom rules or
   `just` modules are copied. The sole fresh-install policy exception is
   configuration-derived application crate/edge inventory (ADR-RUN-0005).


Current invocation contract (sc-lint **0.8.6** for repository and consumer):

```toml
[[tool.sc-lint.lint]]
name = "sc-boundary"
command = ["sc-lint", "lint", "sc-boundary", "--root", ".", "--config", "sc-lint.toml"]
[[tool.sc-lint.lint]]
name = "sc-portability"
command = ["sc-lint", "lint", "sc-portability", "--root", ".", "--config", "sc-lint.toml"]
[[tool.sc-lint.lint]]
name = "sc-runtime"
command = ["sc-lint", "lint", "sc-runtime", "--root", ".", "--config", "sc-lint.toml"]
[tool.sc-lint.lint_profiles]
default = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]
full = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]
```

Fresh generation derives inventory only from selected application crates and
allowed direct edges (ADR-RUN-0303), omitting disabled store/MCP components.
After setup it is consumer-owned: later setup/upgrade and repeated
`sc-lint init --just` must preserve policy edits, analyzer declarations, both
profiles and `db-prepare`. Bootstrap execution receipts must identify all three
exact analyzer commands and zero statuses; TOML inspection alone is insufficient.
ADR-RUN-0005 records the active 0.8.6 contract. Native repeated initialization
must succeed while preserving customized consumer files. Historical Phase C
evidence records the 0.8.4 exit-3 behavior and remains unchanged.

---

## REQ-RUN-0308: Generated `store-sqlite` crate

**Status:** Superseded  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. MCP means Model Context Protocol. SQL means Structured Query Language. JSON means JavaScript Object Notation. API means application programming interface. The crate is the template SQLite store crate described here. A write actor is a dedicated worker accepting queued writes. A queue is an intermediate collection of pending operations. The two-pool arrangement is the reader pool and single-connection writer described in the table below. Custom concurrency code is code written in the crate to coordinate concurrent work, such as spawned tasks or threads, channels, or locks; the two `sqlx` pools are not custom concurrency code.

The SQLite store is the atm-core writer behind `Storage<T>`, not two sqlx pools with per-table query functions. The text below is kept for history and binds nothing.

`crates/store-sqlite` in a project rendered from `template/` is template code that the project owns. The `sc-runtime` crate constructs nothing in it.

| Aspect | Requirement |
|---|---|
| Driver | depends on `sqlx` 0.9 with the `sqlite` feature. Only a `store-*` crate has a direct `sqlx` dependency. Amended 2026-10-25: `store-postgres` and `store-mysql` are the other engine crates |
| Pools | two `sqlx` SQLite pools on the same database file: a read pool of N connections, and a write pool of exactly 1 connection; all writes go through the write pool |
| Journal mode | WAL (write-ahead logging) is turned on when the pools are opened |
| Migrations | SQL files in `crates/store-sqlite/migrations/`, embedded in the binary with `sqlx::migrate!` and run when the store is opened |
| Queries | written with `sqlx`'s compile-time-checked macros (`query!`, `query_as!`) |
| Database URL variable | `crates/store-sqlite/sqlx.toml` names `SQLITE_DATABASE_URL` as the variable the `sqlx` macros read, so a second store crate with a different variable can build in the same workspace later |
| Offline data | `crates/store-sqlite/.sqlx/` is produced by `just db-prepare`. It is checked in to git in this repository as `template/crates/store-sqlite/.sqlx/` and rendered into every generated project, which checks it in with its first commit |
| Public API | `open(cfg) -> Result<Store>`, which opens both pools and runs the migrations, plus typed async query functions (for the example: insert a widget, fetch a widget) |
| Callers | A two-store layout: the adaptor depends on each `store-*` crate whose rows it moves, and `service` depends on the adaptor. A one-store layout has no adaptor, and `service` depends on that store. `daemon` and `cli` do not depend on a store crate. REST handlers, MCP tools, and the CLI never call a store crate. Amended 2026-10-25 |

`open` takes `OpenConfig { url: String }` and returns `StoreError` with
`StoreCode` (`Connection`, `Query`, `Migration`, `NotFound`, `Conflict`,
`Unavailable`) and `message`. `StoreCode` is `non_exhaustive`. `Unavailable`
is retryable. `StoreCode::remediation` returns the recovery text.
`StoreError` implements `Display` and `std::error::Error`. `Display` includes
the code. The engine copies the driver error into `message`, and
`Error::source` stays empty. SQLite reads a file path from `url`. Postgres
and MySQL read a database URL. Crates other than the schema crate construct
the config with `OpenConfig::new`. The adaptor calls `read_rows` and
`write_rows` on `EngineStore`. Those calls reject more than 10_000 rows or a
payload over 1 MiB. `RowRecord.payload` is UTF-8 JSON whose keys are column
names. Amended 2026-10-26.
How the application chooses that path (a config key, or a default under
`<instance-root>`) is not stated. `<instance-root>` is the per-application,
per-user directory resolved by sc-transport, or an explicitly supplied path;
its default location is undecided
([REQ-TRN-0002](sc-transport/requirements.md)).

The two-pool arrangement is the whole mechanism.

**MUST statements:**

- The crate MUST conform to the Aspect and Requirement table in this record.

**MUST NOT statements:**

- The crate MUST NOT contain a write actor.
- The crate MUST NOT contain a queue.
- The crate MUST NOT contain any other custom concurrency code.

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
   `SQLX_OFFLINE=true cargo build -p <project>-store-sqlite` exits 0.
   The template workspace, before sprint `b-8` renders the package name,
   builds `-p store-sqlite --manifest-path template/Cargo.toml`. Amended
   2026-10-25.
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
   dependency of each `store-*` crate that layout renders, and of no other
   generated crate. Amended 2026-10-25. A layout that renders only
   `store-sqlite` still shows `sqlx` on that crate alone.
6. Inspect the source: the write pool is built with a maximum of 1
   connection, and every `INSERT`, `UPDATE` or `DELETE` query function uses
   it.
### Implementation Notes

**Key Considerations:**

- OPEN: the value of N (the read pool size) and whether it is configurable
  are not stated.

---

## REQ-RUN-0309: Generated daemon initialises `sc-observability` directly

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

This applies to `crates/daemon` of a project rendered from `template/`.
`sc-observability` is the SC logging crate. In this record, OpenTelemetry export is the telemetry-export functionality supplied by the `sc-observability-otlp` crate mentioned below. The template omits it; a consuming project can add it to its own daemon.

Definitions: The template is the `template/` directory; a rendered project is a project generated from it. `sc-config` is the SC configuration crate; `sc_config::load` reads a project's configuration into a struct. `AppConfig` is the configuration struct defined in the generated daemon crate; its `logging` field holds the logging settings. `sc-observability-types` is the SC crate of shared logging value types. A registry requirement is a Cargo dependency version requirement resolved from the crates.io registry, not from a path or git source. `daemon::observability` is the generated daemon crate's module that converts logging settings and calls `sc-observability`. `Observability` is the type in `daemon::observability` that owns the `sc-observability` logger; `Observability::build` creates it and `Observability::shutdown` consumes it. `sc_runtime::Daemon::builder` is the `sc-runtime` daemon assembly entry point; `.run().await` on the builder runs the daemon until it stops. A bridge from the `tracing` crate into `sc-observability` is any code that forwards events of the `tracing` crate to `sc-observability`.

1. `crates/daemon/Cargo.toml` MUST use registry requirements
   `sc-observability = ">=1.5.0"` and
   `sc-observability-types = ">=1.5.0"`, with no upper bound (ANC-037).
   An upper bound is any requirement that excludes a later version, including
   an exact `=` pin, a tilde requirement or an explicit `<` limit. The
   historical 0.1.1 release used exact `=1.5.0` pins; those pins do not govern
   Phase D. Phase D retains the command-owned neutral port and private generated
   LoggerAdapter. Normal build/tests check compatibility at the selected
   versions; no registry-availability, signature or forced-cleanup probe gates
   the release. ADR-RUN-0017's Phase D amendment governs panic/error behavior.

2. `crates/daemon/src/main.rs` MUST perform these three steps, in this order:
   1. load the project's configuration with `sc-config` into the project's
      own config struct; on error, report it and return a failure exit code
      without panicking;
   2. initialise `sc-observability` by calling it directly, passing plain
      values taken from the loaded config (its logging section); if the
      initialisation call returns a guard value, bind it to a variable that
      lives until `main` returns;
   3. assemble and run the daemon with `sc_runtime::Daemon::builder(...)`.

   The shape below fixes the ownership and lifetime, while the generated
   `daemon::observability` module owns the detailed settings conversion:



```rust
   let cfg: AppConfig = match sc_config::load("my-app") {
       Ok(cfg) => cfg,
       Err(e) => return report(e),
   };
   let app = sc_transport::AppName::new(APP_NAME)?;
   let log_root = resolve_log_root(&cfg, &app)?;
   let observability = daemon::observability::Observability::build(
       APP_NAME,
       log_root,
       &cfg.logging,
   )?; // generated main maps this to DAEMON.OBSERVABILITY
   let logger = observability.logger();
   let result = sc_runtime::Daemon::builder(&app, &cfg.daemon)
       .logger(logger.clone())
       /* .stores(...).routes(...).mcp(...) */
       .run()
       .await;
   drop(logger);
   let shutdown = observability.shutdown();
   prefer_run_error(result, shutdown)?;


```

3. No library crate may initialise, wrap or depend on
   `sc-observability` or its OpenTelemetry export crate
   `sc-observability-otlp`; the forbidden edges are owned by
   [REQ-RUN-0005](requirements.md). The only observability dependency inside
   library crates is registry `sc-observability-types` with requirement
   `>=1.5.0` and no upper bound; the recommended lock records the versions
   actually tested. Runtime/cron receive the command-owned neutral port;
   no library wraps or links the full logger. Types remain forbidden on
   config/transport and remediation stays on command only.
   `DaemonNotRunning` carries `code: &'static str`
   (`DAEMON.NOT_RUNNING`) plus the endpoint and the cause, and does not
   carry `sc_observability_types::ErrorCode`
   ([ADR-RUN-0004](architecture.md)). a-4's `sc-command` publishes the
   string-to-`OpError` constructor; a-6's generated CLI calls it.
4. A project that wants it adds `sc-observability-otlp` to its own `daemon` crate.
5. Phase B resolves the earlier open question in favor of direct injected `LogEvent` emission; no tracing bridge is added.

The Phase B composition contract fixes the concrete path: the generated
daemon crate defines `AppConfig`; its `observability` module builds
`LoggerConfig::default_for(ServiceName::new(APP_NAME)?, log_root)`, applies
the generated logging settings, and calls `LoggerBuilder::build`. It retains
the resulting `Arc<Logger>` until all injected handles have drained and then
consumes the generated `Observability` owner with `shutdown(self)`.

Root/generated lock evidence proves matching registry identity.

**MUST statements:**

- `crates/daemon/Cargo.toml` MUST declare the registry requirement `sc-observability = ">=1.5.0"`.
- `crates/daemon/Cargo.toml` MUST declare the registry requirement `sc-observability-types = ">=1.5.0"`.
- In `crates/daemon/src/main.rs`, the `sc_config::load` call that produces `AppConfig` MUST come before the `Observability::build` call.
- When the rendered daemon starts and `sc_config::load` returns an error, the daemon process MUST write the configuration error to standard error.
- When `sc_config::load` returns an error, `crates/daemon/src/main.rs` MUST return a non-zero process exit code.
- `crates/daemon/src/main.rs` MUST initialize `sc-observability` by calling `Observability::build`.
- The `Observability::build` call in `crates/daemon/src/main.rs` MUST receive the `logging` field of the loaded `AppConfig` as its logging-settings argument.
- `crates/daemon/src/main.rs` MUST keep the value returned by `Observability::build` alive until `.run().await` on the daemon builder completes.
- In `crates/daemon/src/main.rs`, the `sc_runtime::Daemon::builder` call MUST come after the `Observability::build` call.
- `crates/daemon/src/main.rs` MUST call `.run().await` on the builder returned by `sc_runtime::Daemon::builder`.

**MUST NOT statements:**

- When the rendered daemon starts with a configuration file that `sc_config::load` rejects, the daemon process MUST NOT print a Rust panic message to standard error.
- The `sc-observability` requirement in `crates/daemon/Cargo.toml` MUST NOT declare an upper bound.
- The `sc-observability-types` requirement in `crates/daemon/Cargo.toml` MUST NOT declare an upper bound.
- `Cargo.toml` files under `template/` MUST NOT declare a dependency on `sc-observability-otlp`.
- Files under `template/` MUST NOT contain a bridge from the `tracing` crate into `sc-observability`.

### Rationale

`sc-observability` is the SC logging standard, so every generated daemon uses
it. It is initialised by the project's own `main.rs`, not hidden inside
`sc-config` or `sc-runtime`, so that those two crates stay usable by programs
that do not use `sc-observability`, and so the observability crates can
release independently of them. Config is loaded first because logging
settings come from config; the daemon is assembled last so that its start-up
is logged.
### Success Criteria

1. Rendered `crates/daemon/Cargo.toml` declares `sc-observability = ">=1.5.0"`
   and `sc-observability-types = ">=1.5.0"` without upper bounds, lists no
   OpenTelemetry crate, and passes ordinary generated-daemon logging tests.

2. In rendered `crates/daemon/src/main.rs`, the `sc-config` load call comes
   before `daemon::observability::Observability::build`, which comes before
   `Daemon::builder`. The named `Observability` owner lives until runtime and
   scheduler workers have drained and every injected handle has been dropped;
   `shutdown(self)` is then called exactly once. There is no anonymous guard,
   library-owned construction, or second logger.
3. In the `randlee/sc-runtime` repository, for each library
   crates, `cargo tree -p <crate> -e normal --prefix none` prints no line
   beginning with `sc-observability ` or `sc-observability-otlp ` (the name
   followed by a space). A line beginning with `sc-observability-types ` is
   expected under `sc-command` and does not match those two names. The
   same command on `sc-config` and `sc-transport` prints no line beginning
   with `sc-observability-types `. `sc-runtime` and `sc-cron` follow the
   direct types-only rule in [REQ-RUN-0005](requirements.md) criterion 5.
4. `grep -rn 'tracing' template/crates/daemon` finds no bridge or subscriber
   set-up code.
5. Rendered `main.rs` contains no `unwrap`, `expect` or `panic!` on the
   config load path.

---

## REQ-RUN-0310: Generated `cli` and `daemon` accept `--endpoint`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-20  
**Owner:** randlee  

### Requirement Statement

Definitions: The template is the `cli` and `daemon` crates of a project rendered from `template/`; `cli` and `daemon` name those crates and their binaries. Template code is the code of those two crates. The endpoint is where the daemon listens: a Unix domain socket path or a TCP address. The resolver is the `sc-transport` endpoint resolver (REQ-TRN-0001). A global option is an option accepted with every subcommand of that binary. The instance root is the per-application, per-user directory resolved by sc-transport, or a directory path supplied explicitly ([REQ-TRN-0002](sc-transport/requirements.md)). An explicit instance root is an instance root path given to the binary instead of one sc-transport resolves. The project's configuration is the configuration the template loads at startup. A socket file name is the last component of a Unix domain socket path, such as `daemon.sock`. A crate's default features are the features listed under `default` in its `Cargo.toml`. A crate depends on another crate when `cargo tree` output for the first crate lists the second.

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

2. The resolver accepts the `SC_ENDPOINT` value as an explicit input; a convenience form of the resolver that reads it with `std::env::var_os` may be used instead.
3. Neither binary may compute an endpoint itself.
4. `cli` passes the resolved endpoint to the `sc-transport` client.
5. `daemon` does not bind a listener itself. `run()` then resolves the bind endpoint through the same `sc-transport` resolver ([REQ-RT-0008](sc-runtime/requirements.md)).

This requirement is decided in this document. The sc-runtime design names the
override ("`--endpoint` flag or `SC_ENDPOINT`") and shows `cfg.endpoint` in
its CLI sketch, but does not oblige the template to implement them.

The `DaemonConfig` fields that carry these values are decided.
`transport` is flattened `sc_transport::TransportConfig` and holds the
configured `endpoint`, `instance_root`, and `tcp_port`. `endpoint_flag` is
`Option<String>` for the `--endpoint` text the project's `main` already
parsed ([REQ-RT-0001](sc-runtime/requirements.md),
[ADR-RT-0001](sc-runtime/architecture.md)).

The `sc-transport` configuration type is `TransportConfig`.
The resolver is
`resolve_endpoint(&AppName, flag: Option<&OsStr>, environment: Option<&OsStr>, &TransportConfig)`
and `resolve_endpoint_from_env`, which reads only
`std::env::var_os("SC_ENDPOINT")`. Endpoint text is `uds:<path>` and
`tcp:<socket-addr>` ([REQ-TRN-0001](sc-transport/requirements.md)).

The resolver applies the source order above.

`cli` uses that type in its configuration because it cannot name `DaemonConfig`.

**MUST statements:**

- The generated `cli` argument parser MUST parse `<subcommand> --endpoint <value>` without error for every `cli` subcommand.
- When given the argument sequence `<subcommand> --endpoint <value>` for any `daemon` subcommand, the generated `daemon` argument parser MUST return a successful parse result.
- Each binary MUST obtain its endpoint only from the `sc-transport` resolver.
- When `--endpoint` is given, the template MUST give its value to the resolver.
- When `--endpoint` is not given, the template MUST give the resolver no flag value.
- The template MUST give the resolver the `SC_ENDPOINT` value.
- The template MUST give the resolver the endpoint from its loaded configuration.
- The `daemon` binary MUST place the `--endpoint` value in the `endpoint_flag` field of the `DaemonConfig` value it passes to `sc_runtime::Daemon::builder(...)`.
- The `daemon` binary MUST place the configured endpoint in the `transport` field of the `DaemonConfig` value it passes to `sc_runtime::Daemon::builder(...)`.
- When the `daemon` binary is given an explicit instance root, the `daemon` binary MUST place that path in the `transport` field of the `DaemonConfig` value it passes to `sc_runtime::Daemon::builder(...)`.
- The endpoint in the project's configuration MUST be a field of its `sc_transport::TransportConfig` value.
- The instance root in the project's configuration MUST be a field of its `sc_transport::TransportConfig` value.
- `crates/sc-transport/src/config.rs` MUST implement `serde::Deserialize` for `TransportConfig`.
- `sc_transport::TransportConfig` MUST be public when `sc-transport` is built with only its default features.

**MUST NOT statements:**

- Template code MUST NOT join a socket file name onto a directory.
- Template code MUST NOT choose a default port.
- Template code MUST NOT apply its own precedence between endpoint sources.
- The template's `cli` crate MUST NOT depend on the `sc-runtime` crate.

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

Criteria that start a daemon in a child process run only on an isolated
machine, never on a developer's host
([NFR-RUN-0010](requirements.md)).

Criteria 1 to 3 are focused, host-safe parser/resolver tests with no daemon
and no child process. They assert the boundary between generated argument
parsing and `sc-transport` without prescribing the CLI's internal helper
functions or test-generation architecture. The precedence logic itself is
tested in `sc-transport` ([REQ-TRN-0001](sc-transport/requirements.md)).

1. A unit test in the generated `cli` crate parses a command line containing
   `--endpoint <value>` and asserts that the inputs the CLI hands to the
   `sc-transport` resolver carry that value as the explicit flag value.
2. A unit test gives the CLI's resolver call an `SC_ENDPOINT` value through
   an explicit environment input, with no `--endpoint` option, and asserts
   the resolved endpoint is that value; a third case gives both and asserts
   the `--endpoint` value wins.
3. The same three cases are asserted for the generated `daemon` binary's
   argument handling: the `DaemonConfig` it builds carries the `--endpoint`
   value.
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
### Implementation Notes

**Key Considerations:**

- OPEN: whether the binaries also accept an option for an explicit instance
  root is not decided.

---

## REQ-RUN-0311: Generated agent documents state the daemon-testing rule

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-09-20  
**Last Updated:** 2026-09-20  
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. An agent document is the `AGENTS.md` or `CLAUDE.md` file of a project generated from `template/`, rendered from `template/AGENTS.md.j2` or `template/CLAUDE.md.j2` ([REQ-RUN-0305](requirements.md)). The daemon-testing section is the section of an agent document on testing the daemon. A daemon-process test is an integration test that runs the daemon as a separate process, uses the default instance root, or involves the service manager or the CLI's auto-start. An isolated machine is a virtual machine, for example one run by colima, or an ephemeral CI runner. The instance root is the per-application, per-user directory resolved by sc-transport, or a directory the caller supplies explicitly ([REQ-RT-0002](sc-runtime/requirements.md)). A temporary instance root is a temporary directory a test supplies as the instance root. The daemon singleton is the operating-system exclusive lock on the file `daemon.lock` in the instance root, which lets only one daemon run per instance root ([REQ-RT-0002](sc-runtime/requirements.md)). A daemon collides with another daemon when both daemons use the same `daemon.lock`, endpoint or database files. A test is safe on the host when the test cannot collide with a daemon the developer is running and cannot leave a daemon running after the test ends. Request construction and result rendering are the CLI command functions defined in [REQ-RUN-0312](requirements.md). The CLI binary is the `cli` executable of a generated project. A host-run test is a test that may run on a developer's host ([REQ-RUN-0312](requirements.md)). A test daemon is a crate, binary or cargo feature that is mostly the daemon but built for tests. colima is the virtual machine tool named as the example in [NFR-RUN-0010](requirements.md). A virtual machine definition is a file that defines a virtual machine, such as a `Vagrantfile` or a `*.lima.yaml` file. A colima configuration is a file that configures colima. A provisioning script is a script that creates a virtual machine or installs software on one. v0.1 is release version 0.1 of sc-runtime.

Decided by the owner on 2026-09-20. [NFR-RUN-0010](requirements.md) states the daemon-testing rule in full; this record requires the rule to reach generated projects as plain-language guidance, not as tooling.

**MUST statements:**

- The daemon-testing section of every agent document MUST state that a daemon-process test runs only on an isolated machine.
- The daemon-testing section of every agent document MUST state that a daemon-process test does not run on a developer's host.
- The daemon-testing section of every agent document MUST state that the project's own daemon may be running on a developer's host.
- The daemon-testing section of every agent document MUST state that a daemon which a test starts on a developer's host can collide with a daemon the developer is running.
- The daemon-testing section of every agent document MUST state that a daemon which a test starts can keep running after the test ends.
- The daemon-testing section of every agent document MUST state that a test which cannot be made safe on the host runs in a virtual machine.
- The daemon-testing section of every agent document MUST state that the project does not create a test daemon.
- The daemon-testing section of every agent document MUST state that the project does not weaken the daemon singleton.
- The daemon-testing section of every agent document MUST state that the project does not add test-only switches to daemon code.
- The daemon-testing section of every agent document MUST state that a test running the daemon in-process through `sc_runtime::testing::DaemonFixture` on a temporary instance root is safe on the host when the test keeps the test's database under the temporary instance root.
- The daemon-testing section of every agent document MUST state that running the daemon in-process through `sc_runtime::testing::DaemonFixture` is the default way to test an operation.
- The daemon-testing section of every agent document MUST state that a host-run test never executes the CLI binary.
- The daemon-testing section of every agent document MUST state that the CLI starts a real daemon by default when the CLI cannot reach a daemon.
- The daemon-testing section of every agent document MUST state that the behaviour of a CLI command is verified by unit tests of the command's request construction and result rendering ([REQ-RUN-0312](requirements.md)).
- The daemon-testing section of every agent document MUST state that unit tests of request construction and result rendering run without a daemon.

**MUST NOT statements:**

- In v0.1, the `template/` directory MUST NOT contain a virtual machine definition.
- In v0.1, the `template/` directory MUST NOT contain a colima configuration.
- In v0.1, the `template/` directory MUST NOT contain a provisioning script.

### Rationale

The rule protects against a mistake that is cheap to make and expensive to
undo, and the people most likely to make it are agents and developers working
in a generated project long after generation, who will never read this
repository's requirements. Putting the rule in the documents they do read is
what makes it reach them. It is stated as guidance and not as tooling because
the owner decided that building a virtual machine is not part of v0.1.
### Success Criteria

1. In a project generated from each fixture, `AGENTS.md` and `CLAUDE.md` each
   contain a daemon-testing section that states the content of every MUST
   statement; an inspection checks each statement against the rendered text.
2. `grep -il "colima" AGENTS.md CLAUDE.md` in the generated project lists both
   files, and `grep -il "test daemon" AGENTS.md CLAUDE.md` lists both files.
3. `find . -iname '*colima*' -o -iname 'Vagrantfile' -o -iname '*.lima.yaml'`
   in the generated project prints nothing.
### Implementation Notes

**Key Considerations:**

- Status: Deferred — follows generated test architecture
- Disposition (2026-09-21): The process-isolation safety rule remains Active
  in NFR-RUN-0010, but prescribing generated agent-document wording is deferred
  with the generated CLI-test layer. Phase A may document the safety rule in
  this repository; it does not freeze generated application guidance before the
  post-core attribute-generation work is evaluated.

---

## REQ-RUN-0312: Generated CLI commands are proven by unit tests with no daemon

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-09-20  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. HTTP means Hypertext Transfer Protocol. JSON means JavaScript Object Notation. ADR means architecture decision record. A mock transport is a test substitute for the real network transport. A transport trait is an interface introduced to substitute transport implementations. Request construction is the function from a command's command-line arguments to the HTTP method, route path and JSON body to be posted. Result rendering is the function from the received `sc_command::Envelope<T>` or `sc_transport::TransportError` to stdout text and process exit status, for both the `--json` and plain forms. A pure function performs no I/O, does not read the environment and does not call the network. A split CLI command is a CLI command written as two pure functions, request construction and result rendering, with the HTTP call made between them. An example command is the CLI command for `widget.create` or `widget.get`. The unit-test cases are the five cases listed in this requirement, each beginning "given". The template is the `template/` directory. A host-run test is a test that may run on a developer's host ([NFR-RUN-0010](requirements.md)). The example unit tests are the unit tests of the example commands, in `template/crates/cli`. A test is for a unit-test case when it sets up the input the case gives and asserts the result the case states. A call to a function includes a call through other functions. A function opens a socket when it creates a TCP, UDP or Unix-domain socket. A function uses the network when it sends or receives data over a socket, for example through `sc_transport::Client`. A function starts a child process when it runs `std::process::Command` or `tokio::process::Command`. A testability-only abstraction is a trait whose only implementors are a real implementation and a test double, or a type or module that only test code uses.

Direction given by the owner on 2026-09-20: one purpose of sc-runtime is to
reduce the complexity of testing a CLI that posts and gets over HTTP, and
because the CLI code and its tests are generated, a command's input and
output can be proven in a simple unit test without a daemon. The shape below
is decided in this document to carry out that direction, and is pending the
owner's confirmation.

This applies to the `cli` crate of a project generated from `template/`, and
to the example commands `widget.create` and `widget.get`
([REQ-RUN-0302](requirements.md)), which are the pattern a project copies.

   - request construction: from the command-line arguments to the HTTP
     method, the route path and the JSON body that would be posted (the
     serialised `api-types` request value). In the owner's words: the command
     `my-cli <args>` posts `<expected JSON>` over HTTP, and that is what the
     test asserts;
   - result rendering: from what came back, either the received
     `sc_command::Envelope<T>` or the `sc_transport::TransportError`, to the
     text written to stdout and the process exit status, for both the
     `--json` form and the plain form.
   Neither function may perform I/O, read the environment, or call the
   network.
   - given the command-line arguments of a command, request construction
     yields the expected HTTP method and route path, and a JSON body equal,
     compared as JSON, to the expected document written in the test;
   - given a success envelope as JSON text, rendering with `--json` writes
     that envelope unchanged and the exit status is 0;
   - given a failure envelope as JSON text, rendering with `--json` writes it
     unchanged and the exit status is non-zero;
   - given `TransportError::DaemonNotRunning`, rendering produces the
     `DAEMON.NOT_RUNNING` envelope of [REQ-RUN-0202](requirements.md) and a
     non-zero exit status;
   - given any other `TransportError`, rendering produces a failure whose
     code is not `DAEMON.NOT_RUNNING` and a non-zero exit status.
3. These unit tests, together with three things that already exist, are the
   proof that a command works: the HTTP hop is tested in `sc-transport`
   ([REQ-TRN-0005](sc-transport/requirements.md)); the route and its handler
   are tested against a `DaemonFixture` daemon through
   `sc_transport::Client` ([REQ-RUN-0201](requirements.md)); and the request
   and response types are the same `api-types` structs on both sides, checked
   by the compiler ([ADR-RUN-0302](architecture.md)).
4. Every test that executes the CLI binary is a daemon-process test and runs only on an isolated machine ([NFR-RUN-0010](requirements.md)), because the binary auto-starts a daemon by default. Where a host-run test needs the CLI's path to a fixture daemon, it calls the command's request-construction function, sends the result with `sc_transport::Client`, and passes the reply to the rendering function, all inside the test process. This never reaches the CLI's auto-start ([REQ-RUN-0206](requirements.md)), so it cannot start a daemon on the developer's machine ([NFR-RUN-0010](requirements.md)).
6. One end-to-end run of the real `cli` binary against the real `daemon`
   binary is a daemon-process test and runs only on an isolated machine
   ([NFR-RUN-0010](requirements.md)).

The split into two functions is the whole testability mechanism.

**MUST statements:**

- Each CLI command MUST be a split CLI command.
- `template/crates/cli` MUST contain unit tests for each example command.
- The example unit tests MUST pass when no daemon process is running.
- `template/crates/cli` MUST contain a unit test for every pair of an example command and a unit-test case.

**MUST NOT statements:**

- The example unit tests MUST NOT call any function that opens a socket.
- The example unit tests MUST NOT call any function that uses the network.
- The example unit tests MUST NOT call any function that starts a child process.
- A host-run test MUST NOT execute the CLI binary.
- The template MUST NOT add a mock transport.
- The template MUST NOT add a transport trait.
- The template MUST NOT add a testability-only abstraction.

### Rationale

Testing a CLI by running its binary against a running service is slow,
needs a daemon, and on a development machine risks colliding with or leaking
daemons. Almost none of that test is about the CLI: the CLI's own work is
turning arguments into a request and a reply into output. Those two steps are
pure, so a unit test proves them completely, and because the code is
generated the tests are generated with it and every project starts with them.
The parts a unit test does not reach are already proven elsewhere: the
transport by its own crate's tests, the server by fixture tests, and the
agreement between client and server types by the compiler. Splitting into two
functions costs nothing and needs no mocking layer. The ban on testability-only
abstractions applies [NFR-RUN-0004](requirements.md).
### Success Criteria

1. In a project generated from each fixture, `cargo test -p cli` passes with
   no daemon running and with networking unavailable. Once the test names
   are pinned (the OPEN above): its output lists, for `widget.create` and for
   `widget.get`, the five tests of obligation 2.
2. Inspection of `template/crates/cli` finds, for each command, one
   request-construction function and one rendering function, neither of
   which calls `sc_transport`, reads `std::env`, or writes to stdout
   directly; the command's entry point is the only code that does.
3. `grep -rn 'Command::new\|process::Command' template/crates/cli` prints
   nothing outside the auto-start code of
   [REQ-RUN-0206](requirements.md), and no host-run test in the template
   spawns the `cli` binary for any purpose.
4. Inspection of `template/` finds no mock transport and no trait whose only
   implementors are a real transport and a test double.
5. The host-run example test ([REQ-RUN-0302](requirements.md)) exercises the
   CLI leg as obligation 4 describes and passes under `just test`.
### Implementation Notes

**Key Considerations:**

- Status: Deferred — architecture reopened
- Disposition (2026-09-21): The two-pure-function shape was pending owner
  confirmation and is not part of the recut Phase A. The intended direction is
  attribute-generated CLI testing layered after the reusable core and minimal
  generated application exist. The detailed proposal below is preserved as
  decision history, not as a binding Phase A implementation contract. Phase A
  must still prove the CLI's externally observable interoperability, but it must
  not introduce either this helper shape or a speculative attribute framework.
- OPEN: how the CLI and the daemon are kept to the same route path is not
  decided. The request and response types are shared through `api-types`, but
  the route string (`/ops/widget.create`) is written once in the daemon's
  routes and once in the CLI; either it becomes a shared constant, or the
  host-run test of obligation 4 and the end-to-end run of obligation 6 are
  what catch a mismatch.
- OPEN: the names and signatures of the two functions, and the names of the
  unit tests, are not decided; they are pinned with the rest of the template's
  example code.

---

## REQ-RUN-0313: The store relationship lives in a consumer adaptor

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: A consumer repository is an application project using the framework. A store holds one database engine's rows and SQL. The remote store is the store of the layout's one sqlx engine, which is Postgres or MySQL. A copy adaptor pairs local SQLite with the remote schema; a host-aggregate adaptor pairs host SQLite with a remote aggregate. The adaptor crate is the crate that `just new` (`scripts/new_project.py`) generates into the consumer repository for the copy or host-aggregate adaptor. SQL means Structured Query Language. The statements below remain subject to the explicit deferral in this record.

This record is deferred and does not apply to Phase D; it creates no implementation or validation obligation until reopened.

Synchronization between databases is deferred. This requirement binds nothing until it is taken up again.

**MUST statements:**

- The Rust code that moves rows between the local SQLite store and the remote store MUST be in the adaptor crate.

### Rationale

SC projects pair a local SQLite database with a remote Postgres or SQL database. The local database is a cache. A cache moves rows between the two stores. Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. A generated repository that selects the copy pattern or the host-aggregate pattern contains an adaptor crate that names both stores.

## REQ-RUN-0314: The copy adaptor matches the remote schema

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: The copy adaptor is the generated adaptor crate that keeps a local SQLite copy of the remote database.
The remote schema is the table definitions of the application's remote database, each table with its columns; it does not include rows.
Initializing the local SQLite database is the copy adaptor's setup of the local SQLite database before the copy adaptor first reads or writes rows in it.
The local SQLite database is the SQLite database that the copy adaptor creates.

This Draft is deferred and creates no implementation or validation obligation until reopened. Database synchronization is deferred.

**MUST statements:**

- When the copy adaptor initializes the local SQLite database, the copy adaptor MUST create the remote schema in the local SQLite database.

### Rationale

The local copy gives faster or offline access. Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. A generated copy adaptor creates the shared remote tables in SQLite.

## REQ-RUN-0315: The host-aggregate adaptor collects many hosts

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions: A consumer repository is an application project using the framework.
The store adaptor template is the crate `template/crates/store-adaptor`, which generation renders into a consumer repository.
`aggregate_host` is the public function in `template/crates/store-adaptor/src/shared.rs` that moves one table's rows from a host database to an aggregate database.
A row is one `RowRecord`, which holds a table name and the payload of one table row.
A host database is the one SQLite database of the host that runs `aggregate_host`.
A Postgres aggregate is one Postgres database that receives rows from the host databases of many hosts.
A MySQL aggregate is one MySQL database that receives rows from the host databases of many hosts.
The remote engine is the layout's one sqlx engine, Postgres or MySQL.
The rollup is the consumer-owned `Rollup` implementation that `aggregate_host` calls to map host rows to aggregate rows.
sqlx is the Rust SQL access crate used for the remote engines.
`Ok` is the Rust `Result::Ok` variant that an `aggregate_host` call returns when it succeeds.
SQL means Structured Query Language.

This Draft record is deferred and does not apply to Phase D. It creates no implementation or validation obligation until reopened. Synchronization between databases is deferred.

**MUST statements:**

- Every row that an `aggregate_host` call passes to the rollup MUST be a row that the call read from the host database.
- When Postgres is the remote engine, an `aggregate_host` call that returns `Ok` MUST leave in the Postgres aggregate every row that the rollup returns for the host database's rows of the table passed to the call.
- When MySQL is the remote engine, an `aggregate_host` call that returns `Ok` MUST leave in the MySQL aggregate every row that the rollup returns for the host database's rows of the table passed to the call.

### Rationale

SC projects pair a local SQLite database with a remote Postgres or SQL database; the local database is a cache. Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. A generated host-aggregate adaptor documents one SQLite database per host and one Postgres aggregate.

## REQ-RUN-0316: The consumer defines the host rollup

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: A consumer repository is an application project using the framework.
A host row is a row in a host SQLite database.
A Postgres aggregate is a PostgreSQL database collecting data from multiple hosts.
A MySQL aggregate is a MySQL database collecting data from multiple hosts.
A rollup function is an application-owned mapping from host rows to aggregate rows.
The remote engine is the selected Postgres or MySQL database engine.

This Draft record is deferred and does not apply to Phase D. It creates no implementation or validation obligation until reopened. Database synchronization is deferred.

**MUST statements:**

- The consumer repository MUST define a rollup function mapping host rows into the Postgres aggregate.
- When MySQL is the remote engine, the consumer repository MUST define a rollup function mapping host rows into the MySQL aggregate.

### Rationale

Generated projects own operations, routes, MCP tools, CLI commands, stores, and schemas (framework charter). Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. The host-aggregate adaptor has a consumer-owned function that maps host rows into the Postgres aggregate.

## REQ-RUN-0317: `just new` generates the adaptor

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: A consumer repository is an application project using the framework. A new repository is the consumer repository that one run of `just new` (`scripts/new_project.py`) writes. A layout is a database-store combination accepted by the generation options (REQ-RUN-0301). A two-store layout is the `SQLite and Postgres` or the `SQLite and MySQL` layout. A store holds one database engine's rows and SQL. A copy adaptor pairs local SQLite with the remote schema; a host-aggregate adaptor pairs host SQLite with a remote aggregate. The adaptor crate is the `<project>-store-adaptor` crate in directory `crates/store-adaptor/`, whose Rust code holds the copy adaptor and the host-aggregate adaptor. The adaptor models are the row structs that the adaptor crate reads and writes, defined in the `<project>-schema` crate. SQL means Structured Query Language.

Synchronization between databases is deferred. This record does not apply to Phase D and creates no implementation or validation obligation until reopened.

**MUST statements:**

- After `just new` runs with a two-store layout, file `crates/store-adaptor/src/lib.rs` of the new repository MUST define the copy adaptor.
- After `just new` runs with a two-store layout, file `crates/store-adaptor/src/lib.rs` of the new repository MUST define the host-aggregate adaptor.
- After `just new` runs with a two-store layout, file `crates/schema/src/lib.rs` of the new repository MUST define the adaptor models.

### Rationale

Leaving the pairing for each consumer to invent was rejected: a common pattern has one generated implementation (ADR-RUN-0305). Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. `just new` writes the adaptor crate and its models into the new repository.

# Requirement: The `sc-runtime` library crate does not generate the adaptor

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-RUN-0318: The `sc-runtime` library crate does not generate the adaptor

**Status:** Draft  

### Requirement Statement

Definitions:
- The `sc-runtime` library crate is the Rust library crate whose files are under `crates/sc-runtime`.
- A consumer repository is an application project generated from the framework.
- A store holds one database engine's rows and SQL (Structured Query Language) schema.
- A store adaptor is a crate under `template/crates/store-adaptor*` that moves rows between a local SQLite store and one remote SQL engine.
- A store-adaptor generator is source code that writes the source files of a store adaptor.

Deferral: this record is deferred and does not apply to Phase D; it creates no implementation or validation obligation until reopened. Synchronization between databases is deferred. Command-path code may be written directly as the interim form; no command path is required to be hand-written. A command whose Tcli converts directly to Tsql uses the default conversion without a custom transform. Consumer-side generators are follow-up work running in a consumer repository against delivered sc-runtime; this record does not forbid them. No generator is built in Phase D.

**MUST NOT statements:**

- The files under `crates/sc-runtime` MUST NOT contain a store-adaptor generator.

### Rationale

The framework instantiates; each project wires. Published crates own daemon lifecycle, transport, envelopes, and assembly. Generated projects own operations, routes, MCP tools, CLI commands, stores, and schemas (framework charter). The storage plan is a generated store adaptor in the consumer repository.

### Success Criteria

1. `crates/sc-runtime` contains no generator.

## REQ-RUN-0319: Generation flags name one-sided tables and columns

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: A two-store layout pairs a local SQLite store with a SQL-server store.
A store is a database containing tables and rows.
Generation flags are project-generation inputs that name tables or columns absent from one store in a two-store layout.
A SQLite-only table exists in the SQLite store but not in the SQL-server store.
A SQL-server-only table exists in the SQL-server store but not in the SQLite store.
A one-sided column exists in a shared table in only one store.

This Draft record is deferred and does not apply to Phase D. It creates no implementation or validation obligation until reopened. Database synchronization is deferred.

**MUST statements:**

- The project generator MUST accept without error a flag naming a table present in only one store of a two-store layout.
- The project generator MUST accept without error a flag naming a column present in only one store of a two-store layout.
- The project generator MUST accept without error a flag naming a SQLite-only table.
- The project generator MUST accept without error a flag naming a SQL-server-only table.

### Rationale

Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. Generation input can name a SQLite-only table, a SQL-server-only table, and a column that exists on only one side of a shared table.

## REQ-RUN-0320: One generated implementation per common pattern

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: A consumer repository is an application project using the framework. A store holds one database engine's rows and SQL. A copy adaptor pairs local SQLite with the remote schema; a host-aggregate adaptor pairs host SQLite with a remote aggregate. SQL means Structured Query Language. The statements below remain subject to the explicit deferral in this record.

This record is deferred and does not apply to Phase D; it creates no implementation or validation obligation until reopened.

Synchronization between databases is deferred. This requirement binds nothing until it is taken up again.

Every consumer repository uses the same generated implementation of the copy pattern and the same generated implementation of the host-aggregate pattern.

**MUST statements:**

- Every consumer repository uses the same generated implementation of the copy pattern.
- Every consumer repository uses the same generated implementation of the host-aggregate pattern.

### Rationale

Leaving the pairing for each consumer to invent was rejected: a common pattern has one generated implementation (ADR-RUN-0305). Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. Two generated repositories that select the same pattern contain the same adaptor implementation.

## REQ-RUN-0321: Each database engine has its own generated code

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: Database engines are SQLite, Postgres and MySQL. An engine's SQL is the SQL statement text that an engine implementation sends to that database engine. `sc-runtime-db-sqlite` is the published crate in `crates/sc-runtime-db-sqlite` that implements `Storage<T>` for SQLite. Phase D is the project phase that publishes `sc-storage` and `sc-runtime-db-sqlite` at version 0.3.0 (ADR-RUN-0020 point 36). An engine implementation is the code that implements `Storage<T>` for one database engine. A generated engine implementation is an engine implementation that the project template generates into the consumer repository. Consumer row types are the row structs that a consumer repository defines for its own tables.

Each database engine's SQL is in that engine's implementation. How `T` maps to a table and who owns migrations are not decided by this requirement; they remain open under ADR-RUN-0020 point 4 (PRD section 9).

**MUST statements:**

- SQLite's SQL MUST reside in the crate `sc-runtime-db-sqlite`.
- In Phase D, Postgres's SQL MUST reside in the generated Postgres engine implementation.
- In Phase D, MySQL's SQL MUST reside in the generated MySQL engine implementation.
- Consumer row types MUST remain outside the SQLite engine implementation.
- Consumer row types MUST remain outside the Postgres engine implementation.
- Consumer row types MUST remain outside the MySQL engine implementation.

### Rationale

A query written for both engines is forbidden by ADR-RUN-0301 point 4. One SQL string executed on both SQLite and Postgres was rejected: each engine has its own SQL (ADR-RUN-0305). The framework has no portable query layer (framework charter). The SQLite implementation behind `Storage<T>` is the atm-core writer (PRD-STO-3), published as `sc-runtime-db-sqlite`; Postgres and MySQL use sqlx (PRD-STO-2) and stay generated in the template in Phase D (ADR-RUN-0020 point 37). Consumer row types stay outside generic engines (ADR-RUN-0020 point 5).
### Success Criteria

1. `crates/sc-runtime-db-sqlite` contains no Postgres or MySQL SQL, and the generated Postgres and MySQL engine implementations contain no SQLite SQL.
2. No SQLite, Postgres or MySQL engine implementation names a consumer row type.

## REQ-RUN-0322: An advanced adaptor is consumer code

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: A consumer repository is an application project using the framework. A store is the crate that holds one database engine's rows and SQL. SQL means Structured Query Language. An adaptor is a crate in a consumer repository that moves rows between a SQLite store and a Postgres or MySQL store. The copy adaptor is the generated adaptor that keeps a local SQLite database with the remote schema. The host-aggregate adaptor is the generated adaptor that pairs one SQLite database for this host with a remote database that aggregates many hosts. An advanced adaptor is an adaptor that is neither the copy adaptor nor the host-aggregate adaptor; the consumer writes an advanced adaptor from the store building blocks. The consumer is the owner of a consumer repository. The store building blocks are the store crates `store-sqlite`, `store-postgres` and `store-mysql` and their SQL operations. The project generator is `just new` (`scripts/new_project.py`).

This record is deferred and does not apply to Phase D; it creates no implementation or validation obligation until reopened.

Synchronization between databases is deferred. This requirement binds nothing until it is taken up again.

**MUST NOT statements:**

- The files that the project generator writes into a consumer repository MUST NOT contain an advanced adaptor.

### Rationale

The framework instantiates; each project wires (framework charter). Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. A third adaptor is a file the consumer wrote.

## REQ-RUN-0323: The application schedules store synchronization

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: A consumer repository is an application project using the framework. A store holds one database engine's rows and SQL. SQL means Structured Query Language. The consumer adaptor is the store adaptor crate generated from `template/crates/store-adaptor` into the consumer repository. Store synchronization is copying rows from one store to another store through `read_rows` and `write_rows`. A schedule is code that decides when store synchronization runs. A timer is code that waits for elapsed time or repeats on an interval. Application code is code in the consumer repository outside the consumer adaptor.

This record is deferred, does not apply to Phase D, and creates no implementation or validation obligation until reopened.

**MUST statements:**

- The schedule for store synchronization MUST be in application code.

**MUST NOT statements:**

- The consumer adaptor MUST NOT start a timer.

### Rationale

Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. The adaptor crate starts no timer. A schedule is registered by application code.

## REQ-RUN-0324: The consumer adaptor moves rows between stores

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: The consumer adaptor is the application-owned adaptor crate.
The local store is the application's SQLite database.
The remote store is the application's remote database.

This Draft is deferred and creates no implementation or validation obligation until reopened. Database synchronization is deferred.

**MUST statements:**

- The consumer adaptor crate MUST contain code that copies rows from one store to the other store.

### Rationale

A cache moves rows between the two stores. Store-to-store cache sync inside the sc-runtime crates is out of scope; that forwarding lives in the consumer adaptor (phase-b intent). Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. The adaptor crate contains the copy from one store to the other.

## REQ-RUN-0325: The sc-runtime crates do not move rows between stores

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: sc-runtime is the framework repository `randlee/sc-runtime`. The sc-runtime crates are the framework library crates in its top-level `crates/` directory. A store is one database: one SQLite database file, or one Postgres or MySQL database; connections to the same database are the same store. Reading or writing a store means issuing SQL (Structured Query Language) to that database through any client library or driver, including sqlx and the `sc-runtime-db-sqlite` writer. A row is one record of a database table. A function, method or closure moves rows between stores when it reads rows from one store and writes those rows, changed or unchanged, to another store, either in its own body or through the functions it calls. A consumer repository is an application project using the framework. The store adaptor is the `store-adaptor` crate in `template/crates/` that is generated into a consumer repository, outside the sc-runtime crates.

Deferral: Synchronization between databases is deferred. This record does not apply to Phase D and creates no implementation or validation obligation until reopened.

**MUST NOT statements:**

- Files under the top-level `crates/` directory of the `randlee/sc-runtime` repository MUST NOT contain a function, method or closure that moves rows between stores.

### Rationale

Store-to-store cache sync inside the sc-runtime crates is out of scope; that forwarding lives in the consumer adaptor (phase-b intent). Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. `crates/` contains no store-to-store copy, outbox, or routing policy.

## REQ-RUN-0326: Database layouts of a new application

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: A layout is a database-store combination in the table below. The adaptor is the `store-adaptor` crate that a two-store layout renders (REQ-RUN-0301). The synchronization algorithm is the consumer's code inside that crate; no record binds it. Storage<T> is the shared storage trait for application-defined row type T. SQL means Structured Query Language. The generation options are the template choices specified by `wizard/answers.schema.json`. Generating a layout means rendering `template/` with options that select that layout. A store is a `store-sqlite`, `store-postgres` or `store-mysql` crate; the adaptor is not a store.

A new application is created with no database, with SQLite only, with one sqlx SQL engine only, or with SQLite plus one sqlx SQL engine.

Postgres and MySQL are the SQL engines and use sqlx. SQLite uses the writer of `sc-runtime-db-sqlite`, the atm-core design (PRD-STO-3). Every store implements `Storage<T>` (PRD-STO-2). Amended 2026-11-11. The layouts are:

| Layout | `db` value | Stores |
|---|---|---|
| no database | `none` | none |
| SQLite only | `sqlite` | `store-sqlite` |
| Postgres only | `postgres` | `store-postgres` |
| MySQL only | `mysql` | `store-mysql` |
| SQLite and Postgres | `sqlite-postgres` | `store-sqlite` and `store-postgres` |
| SQLite and MySQL | `sqlite-mysql` | `store-sqlite` and `store-mysql` |

A layout renders the stores in its row. A layout with two stores also renders the adaptor (owner, 2026-11-12: "The synchronization adapter will be necessary in early products."). A layout with one store renders that store and no adaptor. A layout with no database renders no store crate and no adaptor.

Phase b's adaptor crate is the adaptor a two-store layout renders. The row-moving functions in it are template code that the consumer owns and replaces; no record requires a particular algorithm, schedule, direction or conflict policy (owner, 2026-11-12: "Synchronization algorithm will always be specific.").

**MUST statements:**

- The generation options MUST accept the `no database` layout from the table above.
- The generation options MUST accept the `SQLite only` layout from the table above.
- The generation options MUST accept the `Postgres only` layout from the table above.
- The `db` enum in `wizard/answers.schema.json` MUST contain `mysql`.
- The generation options MUST accept the `SQLite and Postgres` layout from the table above.
- The generation options MUST accept the `SQLite and MySQL` layout from the table above.
- Generating a layout MUST render the stores in its table row.
- Generating the `no database` layout MUST render no store crate.
- A generation request selecting cron with the `no database` layout MUST be rejected before template rendering.
- Generating a two-store layout MUST render the adaptor crate.
- Generating a layout with fewer than two stores MUST render no adaptor crate.
- `store-postgres` MUST use `sqlx` for PostgreSQL storage.
- `store-mysql` MUST use `sqlx` for MySQL storage.
- `store-sqlite` MUST use the SQLite writer of `sc-runtime-db-sqlite`.
- Every store MUST implement `Storage<T>` for each row type it stores.

### Rationale

sc-runtime is a set of crates and a template for deploying a new application that can use its database immediately (phase-b intent).
### Success Criteria

1. `wizard/answers.schema.json` accepts each layout in the table.
2. Generating each layout renders the stores in that row, and renders the adaptor only when the row has two stores.
3. A layout whose store crate does not exist yet fails in `scripts/new_project.py` before `cargo generate` runs.
4. Cron selected with the no-database layout is rejected before generation (PRD-CRON-4, 2026-11-11).

## REQ-RUN-0327: Paired stores use the same schema or a similar schema

**Status:** Draft  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions: The project generator is the `just new` command (`scripts/new_project.py`).
A store is a database containing tables and rows.
A two-store layout pairs a local SQLite store with a server store.
A server store is a Postgres or MySQL store.
Store migrations are the SQL files in a generated store crate's `migrations` directory.
A store schema is the set of tables and columns that a store's migrations create.
A copy-pattern project is a generated project whose two-store layout gives the SQLite store the server store's schema.
A host-aggregate project is a generated project whose two-store layout pairs one host's SQLite store with a server store that aggregates many hosts.
A generation flag is a project generator input that assigns one table or one column to one store.
A one-sided table is a table that a generation flag assigns to only one store.
A one-sided column is a column of a shared table that a generation flag assigns to only one store.

This Draft record is deferred and does not apply to Phase D. It creates no implementation or validation obligation until reopened. Database synchronization is deferred.

**MUST statements:**

- For a copy-pattern project, the project generator MUST write SQLite store migrations that create the server store schema.
- For a host-aggregate project, the project generator MUST create each one-sided table only in the store migrations of the store named by its generation flag.
- For a host-aggregate project, the project generator MUST create each one-sided column only in the store migrations of the store named by its generation flag.

### Rationale

One SQL string executed on both engines was rejected: each engine has its own SQL, and the schemas are the same or similar (ADR-RUN-0305). Owner, 2026-10-25 (ADR-RUN-0305): the storage plan is a generated adaptor in the consumer repository.
### Success Criteria

1. The copy pattern generates the same schema on both stores. The host-aggregate pattern generates a similar schema, with tables or columns that exist on only one side.

## REQ-RUN-0328: Every store implements `Storage<T>`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: Storage<T> is the shared asynchronous storage trait for application-defined row type T. A store is the SQLite, Postgres or MySQL implementation in the application layout.

Every store in an application implements the same storage trait, `Storage<T>` (PRD-STO-2). Amended 2026-11-11: the earlier alternative of a different trait per store is withdrawn.

**MUST statements:**

- Every store in an application MUST implement the same storage trait, `Storage<T>`, where `T` is the application-defined row type.

### Rationale

Owner, PRD section 4: "I want sqlite/postgres/sql to be interchangeable behind traits" (PRD-STO-2). Each engine keeps its own SQL behind that trait; `Storage<T>` carries no SQL (PRD-STO-1).
### Success Criteria

1. In each generated layout that renders a store, the source file of every rendered store contains an impl of `Storage<T>`.

## REQ-RUN-0329: A generated package name carries the project name

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: The project name is the application name chosen during generation. A generated crate is a crate directory under `crates/` in a project generated from `template/`. A crate's role is its directory name under `crates/`, except that `api-types` has the role `types`. A package name is the `[package] name` field of that crate's `Cargo.toml`.

A generated crate's package name is the project name, a hyphen, and its role. Every generated package that is published uses that prefix. `<project>-types` is the non-database types crate. `<project>-schema` is the schema crate.

**MUST statements:**

- In a generated project, each `crates/<directory>/Cargo.toml` `[package] name` MUST be the project name, a hyphen and that crate's role.
- Every published generated package MUST use that project-name prefix.

### Rationale

Owner, 2026-10-25 (ADR-RUN-0305 point 12): a generated package name is the project name, a hyphen, and its role.
### Success Criteria

1. Two projects generated with different project names produce no package name in common.

## REQ-RUN-0330: Row structs live in the schema crate

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: A consumer repository is an application project generated from the template. A store is the implementation for one database engine. A consumer adaptor is code in that application repository that moves rows between its local SQLite store and a remote Postgres or MySQL store. The schema crate is the generated <project>-schema package holding database row structs. The types crate holds non-database API types.

Row type structs are in the schema crate. The types crate is for non-database types.

The adaptor depends on `<project>-schema`. Engine implementations remain table-neutral. How a row type maps to a table and who owns migrations remain open under ADR-RUN-0020 point 4; this record does not prescribe SQL generation from the schema crate.

**MUST statements:**

- Database row type structs MUST reside in the schema crate.
- The types crate MUST contain no database row struct.
- The adaptor MUST depend on `<project>-schema`.

### Rationale

Owner, 2026-10-25 (ADR-RUN-0305 point 13): row type structs are in `<project>-schema`; `<project>-types` is for non-database types; the adaptor depends on `<project>-schema`. Owner, 2026-11-12: "The synchronization adapter will be necessary in early products."
### Success Criteria

1. The schema crate contains the row structs.
2. The types crate contains no database row struct.
3. On a two-store layout, the adaptor crate depends on the schema crate.

## REQ-RUN-0331: General database commands are a disableable bootstrap

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. MCP means Model Context Protocol. HTTP means Hypertext Transfer Protocol. A store layout is a generated project whose database selection names at least one of SQLite, Postgres and MySQL. A no-database layout is a generated project whose database selection is `none`. General database commands are the generated commands that take a table name as an argument, including `delete row <table> <id>`. The `general-db` feature is the Cargo feature of the generated CLI, daemon and service crates that compiles in the general database commands. A default build is a build of the generated project with its default Cargo features. The application delete command is the generated delete command for the `widgets` table only: the CLI command `widget delete <id>`, the MCP tool `widget_delete` and the HTTP route `POST /ops/widget.delete`. The backend delete operation is the service function `delete_row`. The OpenAPI document is the HTTP API description that the generated daemon builds from its HTTP router.

**MUST statements:**

- In a default build of a store layout, the generated CLI MUST accept the command `delete row <table> <id>`.
- In a default build of a store layout, the generated daemon MUST return a tool list that contains `delete_row` when sent an MCP `tools/list` request.
- In a default build of a store layout, the generated daemon's OpenAPI document MUST list the path `/ops/delete.row`.
- In a build of a store layout without the `general-db` feature, the generated CLI MUST reject the command `delete row <table> <id>` as unknown.
- In a default build of a store layout, the CLI command `widget delete <id>` MUST perform its delete by calling the backend delete operation with table `widgets`.
- In a default build of a store layout, the MCP tool `widget_delete` MUST perform its delete by calling the backend delete operation with table `widgets`.
- In a default build of a store layout, the HTTP route `POST /ops/widget.delete` MUST perform its delete by calling the backend delete operation with table `widgets`.
- In a default build of a no-database layout, the generated CLI MUST reject each general database command as unknown.

**MUST NOT statements:**

- In a build of a store layout without the `general-db` feature, the generated daemon MUST NOT return a tool list that contains `delete_row` when sent an MCP `tools/list` request.
- In a build of a store layout without the `general-db` feature, the generated daemon's OpenAPI document MUST NOT list the path `/ops/delete.row`.
- In a default build of a no-database layout, the generated daemon MUST NOT return a tool list that contains a general database command, such as `delete_row`, when sent an MCP `tools/list` request.
- In a default build of a no-database layout, the generated daemon's OpenAPI document MUST NOT list the path of a general database command, such as `/ops/delete.row`.

### Rationale

General database commands are the bootstrap. The consumer disables that public surface and adds application commands that call the same backend operations (phase-b intent).
### Success Criteria

1. A generated project with a store registers `delete row <table> <id>` on the CLI, on MCP, and over HTTP.
2. A flag removes those three commands.
3. An application command calls the same delete operation through its own CLI command, MCP tool, and HTTP route.

## REQ-RUN-0332: Optional shared cron command projections

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions:

- DTO means data transfer object.
- MCP means Model Context Protocol.
- The standard start/end/list DTO and handler are the cron start, end and list request types and the shared handler of the `sc-cron-commands` crate ([REQ-CRNC-0001](sc-cron-commands/requirements.md)).
- Selected generation means generation with `cron_commands` set to true and cron selected ([REQ-CRNC-0002](sc-cron-commands/requirements.md)).
- A feature-off build is a build with the cron feature disabled, of a core crate or of a generated project.
- A server dependency is a Cargo dependency on a crate that implements the HTTP or MCP server.
- CLI means command-line interface.
- HTTP means Hypertext Transfer Protocol.
- Generation renders template/ from the selected answers. cron_commands is the Boolean answer selecting standard cron commands; cron selects the scheduler.
- A scheduler dependency is a Cargo dependency on the sc-cron scheduler implementation.
- The core crates are the reusable library crates.
- A selected interface is CLI, HTTP, or MCP when its generation option is enabled.
- The CLI reaches the shared handler through the daemon HTTP route; using a handler does not mean linking its implementation into the CLI.
- Generation reads option defaults from `wizard/answers.schema.json` ([REQ-RUN-0401](requirements.md)).
- Cron is enabled when the `cron` generation answer is true. MCP is enabled when the `mcp` generation answer is true.
- The sc-cron-commands start, end and list request types are the start, end and list request types of the standard start/end/list DTO. The shared sc-cron-commands handler is the standard handler.
- To wire the standard handler into an interface is to make that interface's cron start, end and list projections call it.
- An interface's cron start, end or list projection is the generated code that exposes the cron start, end or list command on that interface: a CLI subcommand, an HTTP route or an MCP tool.

**MUST statements:**

- In `wizard/answers.schema.json`, the `default` of the `cron_commands` property MUST be `false`.
- When cron_commands=true and cron is enabled, the generated CLI cron start projection MUST use the sc-cron-commands start request type.
- When cron_commands=true and cron is enabled, the generated CLI cron start projection MUST invoke the shared sc-cron-commands handler.
- When cron_commands=true and cron is enabled, the generated CLI cron end projection MUST use the sc-cron-commands end request type.
- When cron_commands=true and cron is enabled, the generated CLI cron end projection MUST invoke the shared sc-cron-commands handler.
- When cron_commands=true and cron is enabled, the generated CLI cron list projection MUST use the sc-cron-commands list request type.
- When cron_commands=true and cron is enabled, the generated CLI cron list projection MUST invoke the shared sc-cron-commands handler.
- When cron_commands=true and cron is enabled, the generated HTTP cron start projection MUST use the sc-cron-commands start request type.
- When cron_commands=true and cron is enabled, the generated HTTP cron start projection MUST invoke the shared sc-cron-commands handler.
- When cron_commands=true and cron is enabled, the generated HTTP cron end projection MUST use the sc-cron-commands end request type.
- When cron_commands=true and cron is enabled, the generated HTTP cron end projection MUST invoke the shared sc-cron-commands handler.
- When cron_commands=true and cron is enabled, the generated HTTP cron list projection MUST use the sc-cron-commands list request type.
- When cron_commands=true and cron is enabled, the generated HTTP cron list projection MUST invoke the shared sc-cron-commands handler.
- When cron_commands=true and cron is enabled and MCP is enabled, the generated MCP cron start projection MUST use the sc-cron-commands start request type.
- When cron_commands=true and cron is enabled and MCP is enabled, the generated MCP cron start projection MUST invoke the shared sc-cron-commands handler.
- When cron_commands=true and cron is enabled and MCP is enabled, the generated MCP cron end projection MUST use the sc-cron-commands end request type.
- When cron_commands=true and cron is enabled and MCP is enabled, the generated MCP cron end projection MUST invoke the shared sc-cron-commands handler.
- When cron_commands=true and cron is enabled and MCP is enabled, the generated MCP cron list projection MUST use the sc-cron-commands list request type.
- When cron_commands=true and cron is enabled and MCP is enabled, the generated MCP cron list projection MUST invoke the shared sc-cron-commands handler.
- When cron_commands=true and cron is enabled, generation MUST wire the standard handler into the generated CLI.
- When cron_commands=true and cron is enabled, generation MUST wire the standard handler into the generated HTTP interface.
- When cron_commands=true and cron is enabled and MCP is enabled, generation MUST wire the standard handler into the generated MCP interface.

**MUST NOT statements:**

- Using the request types in a core build with cron disabled MUST NOT add a scheduler dependency.
- Using the request types in a generated build with cron disabled MUST NOT add a scheduler dependency.
- Using the sc-cron-commands request types in a feature-off build MUST NOT add a server dependency.


### Rationale

The statements preserve the existing contract while separating independently observable obligations.

### Success Criteria

Generate default-off and selected-cron projects with MCP both enabled and disabled. Inspect their source call paths and request types. Inspect the default option value and feature-off Cargo graphs for the stated dependency exclusions.

## REQ-RUN-0401: Options contract `wizard/answers.schema.json`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions: The v0.1 option set consists of project name, db, mcp and licence. The Phase C option set is the v0.1 option set plus cron and cron_commands. An option is a property of the generation answers object; JSON Schema specifies that object. A meta-schema is a JSON Schema used to validate another JSON Schema; the `$schema` value names it. The project-name option is the project name row in the Option table; its key spelling remains undecided. The licence option is the licence row in the Option table. A generation-option property is the entry for a generation option in the top-level `properties` object of `wizard/answers.schema.json`. The v0.1 answers schema is `wizard/answers.schema.json` as released for v0.1; the Phase C answers schema is `wizard/answers.schema.json` as released by Phase C. A conditional requirement between options makes one option depend on another, such as `cron_commands = true` requiring `cron = true`. The schema version is the integer at the root key `x-schema-version` of `wizard/answers.schema.json`; it identifies the version of the published answers schema (ADR-RUN-0401).

An answers object is the JSON file that drives project generation; it is written by hand, by an agent, or by the Wyvern wizard under `wizard/`.

No other file in the repository may introduce an option that is absent from the schema.

This item owns the generation option list. The v0.1 list was closed; Phase C adds `cron` and `cron_commands` as required by [REQ-RUN-0332](requirements.md) and [REQ-CRNC-0002](sc-cron-commands/requirements.md).

| Option | JSON type | Values the schema lists |
|---|---|---|
| project name | string | any non-empty string |
| `db` | string | `none`, `sqlite`, `postgres`, `mysql`, `sqlite-postgres`, `sqlite-mysql` |
| `mcp` | boolean | `true`, `false` |
| licence | string | the licence of the generated project's crates; default `MIT` when the answers object does not supply it |
| `cron` (Phase C) | boolean | `true`, `false`; default `false` |
| `cron_commands` (Phase C) | boolean | `true`, `false`; default `false`; `true` requires `cron = true` |

The licence option is decided by the owner on 2026-09-20. `MIT` is only the default.

What each option does to the rendered project is owned by
[REQ-RUN-0303](requirements.md). Sprint `b-8` accepts the six `db` values.
`both` is not in the enum. Amended 2026-10-25.

The consumers of the schema are: the wizard (emits an object of this shape), the driver `scripts/new_project.py` (validates against it), the placeholders in `template/cargo-generate.toml` (use the same key names), and sc-lint.

Placeholders in `template/cargo-generate.toml` are `[placeholders.<name>]`
tables whose `type` is `string`, `bool`, or `array`. Values reach them from
a `[values]` file passed with `--template-values-file`. That syntax is
recorded in Active [ADR-RUN-0402](architecture.md). Sprint a-1 proved it
(rows V3a and V4a of [REQ-RUN-0102](requirements.md)).

The generator is used for projects that are not MIT licensed and not open source.

**MUST statements:**

- The repository MUST contain the file `wizard/answers.schema.json`.
- `wizard/answers.schema.json` MUST validate against the meta-schema identified by its own `$schema` value.
- The top-level `type` in `wizard/answers.schema.json` MUST be `object`.
- The schema MUST be the only definition of the generation options.
- Every generation option MUST appear as a key of the top-level `properties` object in `wizard/answers.schema.json`.
- Every generation-option property MUST have a `type` keyword equal to the JSON type in the Option table row for the option.
- When an Option table row gives a default, the `default` keyword of the generation-option property for the row MUST equal the default in the row.
- For each generation option whose Option table row lists a fixed set of string values, the `enum` keyword of the option's property in `wizard/answers.schema.json` MUST list exactly the values in the row.
- The schema MUST encode every defined conditional requirement between generation options.
- The project-name generation-option property in the v0.1 answers schema MUST match the project name row of the Option table.
- The `db` generation-option property in the v0.1 answers schema MUST match the `db` row of the Option table.
- The `mcp` generation-option property in the v0.1 answers schema MUST match the `mcp` row of the Option table.
- The licence generation-option property in the v0.1 answers schema MUST match the licence row of the Option table.
- The `cron` generation-option property in the Phase C answers schema MUST match the `cron` row of the Option table.

- The `cron_commands` generation-option property in the Phase C answers schema MUST match the `cron_commands` row of the Option table.
- The schema MUST require `cron = true` when `cron_commands = true`.
- The answers object MUST accept a consumer-supplied licence for the generated project.
- `wizard/answers.schema.json` MUST carry the schema version.
- The driver MUST read option names from the schema.
- The driver MUST read option defaults from the schema.

**MUST NOT statements:**

- The v0.1 answers schema MUST NOT define a generation option outside the v0.1 option set.
- The Phase C answers schema MUST NOT define a generation option outside the Phase C option set.
- A file under `template/` MUST NOT set the licence of a generated project to a literal value instead of the licence value from the answers object.
- The driver MUST NOT hold its own list of option names.
- The driver MUST NOT hold its own list of option defaults.

### Rationale

The schema is a published contract: the sc-lint project reads the same answers object and schema to install lint and `just` infrastructure into a generated project.

The wizard, the driver, `template/cargo-generate.toml` and sc-lint all need
the option set. One definition is the only way they cannot disagree. The
answers object and this schema are the entire interface between this project
and sc-lint, so the schema is versioned like any other published contract.
### Success Criteria

1. A unit test under `tests/unit/` loads `wizard/answers.schema.json` and
   asserts that it is a valid schema under the meta-schema named by its own
   `$schema` value.
2. The same test asserts that the top-level `type` is `object`, that `db` is
   a string property whose `enum` is exactly `none`, `sqlite`, `postgres`,
   `mysql`, `sqlite-postgres`, `sqlite-mysql`,
   that `mcp` is a boolean property, that a string property for the project
   name exists, and that a string property for the licence exists whose
   default is `MIT` (its key name follows the licence OPEN above). Amended
   2026-10-25.
3. For v0.1, the test asserts that the top-level `properties` object has exactly
   four option keys: `db`, `mcp`, the project-name key and the licence key.
   For Phase C, it asserts those four keys plus `cron` and `cron_commands`,
   and checks both defaults and the `cron_commands` dependency on `cron`.
   If the schema version is carried as a property, its key is the only
   additional permitted key.
4. The same test asserts that the schema version is present and non-empty.
5. Inspection: `scripts/new_project.py` contains no list of option names or
   option defaults of its own; it reads both from the schema. Sprint `b-8`
   removes the v0.1 reserved-database check. The driver holds no extra `db`
   rejection list. Amended 2026-10-25.
### Implementation Notes

**Key Considerations:**

- OPEN: the property key of the licence option, the form of its value (an
  SPDX licence expression as Cargo's `license` field takes, or something else),
  and how a project that is not open source states its licence (Cargo offers a
  `license-file` field for non-SPDX licences) are not decided.
- OPEN: the property key for the project-name option is not decided.
  Whether that key also appears as a placeholder in
  `template/cargo-generate.toml` is not decided either, because
  `cargo generate` takes the project name through `--name`
  ([REQ-RUN-0402](requirements.md) owns that question).
- OPEN: the defaults of `db` and `mcp` are not decided.
- The schema version is carried as the root integer `x-schema-version` and is raised when a breaking option change ships (ADR-RUN-0401).
- OPEN: the JSON Schema draft (the `$schema` value) is not decided.
- OPEN: the language and test runner of `tests/unit/` are not decided (the
  driver is Python).  
  A `db` value outside the six strings, including `both`, fails schema
  validation. Sprint `b-8` removes the v0.1 reserved-database driver check.
  Amended 2026-10-25.

---

## REQ-RUN-0402: Key-set test: schema versus template placeholders

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: Schema keys are the keys of the top-level properties object in wizard/answers.schema.json. Placeholder keys are the names of the template-defined placeholders declared in template/cargo-generate.toml. The key-set test is the unit test in `tests/unit/` that compares the schema keys with the placeholder keys. `just test` is the `test` recipe of the root `Justfile`, defined in [REQ-RUN-0004](requirements.md).

| Set | Where it is read from |
|---|---|
| schema keys | the keys of the top-level `properties` object in `wizard/answers.schema.json` |
| placeholder keys | the names of the template-defined placeholders declared in `template/cargo-generate.toml` |

This test is the single enforcement of the rule that a generation option is
added to `wizard/answers.schema.json` and to `template/cargo-generate.toml`
together.

Placeholders are declared in `template/cargo-generate.toml` as
`[placeholders.<name>]` tables. The key-set test reads those names. A
placeholder's `type` is `string`, `bool`, or `array`. That syntax is
recorded in Active [ADR-RUN-0402](architecture.md), which sprint a-1 proved
(row V3a of [REQ-RUN-0102](requirements.md)). The test still compares the
two key sets in both directions and reads both files from the working tree.

**MUST statements:**

- `tests/unit/` MUST contain a unit test that compares the schema keys with the placeholder keys.
- The key-set test MUST fail when a schema key has no placeholder key of the same name.
- The key-set test MUST fail when a placeholder key has no schema key of the same name.
- The key-set test MUST read `wizard/answers.schema.json` from the working tree at test time.
- The key-set test MUST read `template/cargo-generate.toml` from the working tree at test time.
- `just test` at the repository root MUST run the key-set test.

**SHOULD statements:**

- The failure message SHOULD list the names found in only one of the two sets.

**MUST NOT statements:**

- The key-set test MUST NOT compare against a hard-coded list of names.

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
### Implementation Notes

**Key Considerations:**

- OPEN: `cargo generate` receives the project name through its `--name`
  argument, not through a template-defined placeholder. Whether the schema's
  project-name key is exempt from this comparison, or is also declared as a
  placeholder, is not decided. Until it is, "the two sets are equal" is not
  defined for that one key; it is defined for every other key.
- OPEN: the language and test runner of `tests/unit/` are not decided (the
  driver is Python).

---

## REQ-RUN-0403: Answers fixtures and fail-closed validation tests

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: An answers object is a JSON object of generation options validated by `wizard/answers.schema.json`. The schema keys are the keys listed under `properties` in `wizard/answers.schema.json`. A fixture is a file matching `wizard/fixtures/*.json`. The complete-answers schema is `wizard/answers.schema.json` with every schema key listed in `required`. The licence-required schema is `wizard/answers.schema.json` with `licence` listed in `required`. A variant is the set of values a fixture gives to every schema key except `project_name`. CI means continuous integration. Template CI is the CI workflow `.github/workflows/template.yml`, which generates a project from every fixture. The `mcp` key selects whether the generated project contains MCP (Model Context Protocol) code. A schema-invalid answers file is a JSON answers file in `tests/fixtures/refused/` that fails validation against `wizard/answers.schema.json`. The driver is `scripts/new_project.py`. The driver fails closed when it exits with a non-zero status, does not call `cargo generate`, and does not create the destination directory.

The default variant is exercised by [REQ-RUN-0303](requirements.md) criterion 7, not by a fixture. The test that the driver fails closed on `db` equal to `both` is owned by [REQ-RUN-0303](requirements.md) criterion 4. Answers objects whose `db` is `postgres`, `mysql`, `sqlite-postgres` or `sqlite-mysql` pass schema validation since sprint `b-8`; v0.1 reserved them. Amended 2026-10-25. No clause of this item depends on the unverified `cargo-generate` mechanics held in [ADR-RUN-0402](architecture.md).

**MUST statements:**

- Every fixture MUST pass validation against `wizard/answers.schema.json`.
- Every fixture MUST pass validation against the complete-answers schema.
- Every fixture MUST pass validation against the licence-required schema.
- Each fixture's variant MUST differ from the variant of every other fixture.
- `wizard/fixtures/sqlite-mcp.json` MUST set `db` to `"sqlite"`.
- `wizard/fixtures/sqlite-mcp.json` MUST set `mcp` to `true`.
- `wizard/fixtures/sqlite-no-mcp.json` MUST set `db` to `"sqlite"`.
- `wizard/fixtures/sqlite-no-mcp.json` MUST set `mcp` to `false`.
- `wizard/fixtures/` MUST contain a fixture whose `licence` value is not `"MIT"`.
- `tests/fixtures/refused/missing-key.json` MUST omit the required key `project_name`.
- `tests/fixtures/refused/wrong-type.json` MUST set `mcp` to `"yes"`.
- `tests/fixtures/refused/db-outside-enum.json` MUST set `db` to `"sqlserver"`.
- `tests/fixtures/refused/unknown-key.json` MUST contain the top-level key `extra`.
- `tests/fixtures/refused/both.json` MUST set `db` to `"both"`.
- `tests/unit/` MUST contain a unit test that asserts every fixture passes validation against `wizard/answers.schema.json`.
- `tests/unit/` MUST contain a unit test that asserts every schema-invalid answers file fails validation against `wizard/answers.schema.json`.
- A unit test in `tests/unit/` MUST assert that, when the driver is run with `--var-file` set to a schema-invalid answers file, the driver exits with a non-zero status.
- A unit test in `tests/unit/` MUST assert that, when the driver is run with `--var-file` set to a schema-invalid answers file, the driver does not call `cargo generate`.
- A unit test in `tests/unit/` MUST assert that, when the driver is run with `--var-file` set to a schema-invalid answers file, the driver does not create the destination directory.

**MUST NOT statements:**

- `wizard/answers.schema.json` MUST NOT reject any file in `wizard/fixtures/`.
- The `db` enum of `wizard/answers.schema.json` MUST NOT list `"sqlserver"`.
- The `properties` of `wizard/answers.schema.json` MUST NOT list `extra`.
- Code outside `tests/unit/` MUST NOT read files in `tests/fixtures/refused/`.

### Rationale

The fixtures are the template's test matrix: each one becomes a generated
project that CI lints and tests. They are also one of the three things handed
to the sc-lint team as the reference for its install packages, the other two
being the schema and the prototype repositories generated from the fixtures.
Failing closed matters because a project rendered from bad answers looks
finished and is wrong. The licence is an option because some projects are not
MIT licensed. Every fixture states the licence, including fixtures that use `MIT`, so that a fixture and the
wizard's output for the same choices are equal JSON
([REQ-RUN-0603](requirements.md)). The fail-closed test uses a stub `cargo`
and only asserts that it was not called.
### Success Criteria

1. A unit test validates every file matching `wizard/fixtures/*.json` against
   `wizard/answers.schema.json` and passes.
2. `wizard/fixtures/sqlite-mcp.json` exists and contains `db` equal to
   `"sqlite"` and `mcp` equal to `true`. Another file in the directory
   contains `db` equal to `"sqlite"` and `mcp` equal to `false`.
3. A unit test validates every schema-invalid answers file and asserts that
   each one is rejected. The set contains at least one file for each of the
   four ways of breaking the schema named in the MUST statements.
4. A unit test runs the driver for a destination `<dest>` with
   `--var-file <schema-invalid file>` and a stub `cargo` first on `PATH` that
   records any call. It asserts a
   non-zero exit status, that the stub recorded no call, and that `<dest>`
   does not exist afterwards.
5. `ls wizard/fixtures/*.json` lists no schema-invalid file.
6. Outside `wizard/fixtures/`, the repository holds one schema-invalid
   answers file with `db` equal to `"both"`. A fixture with `db` equal to
   `"postgres"` validates against the schema. Amended 2026-10-25.
### Implementation Notes

**Key Considerations:**

- The schema-invalid answers files are in `tests/fixtures/refused/`. The
  `mcp = false` fixture is `wizard/fixtures/sqlite-no-mcp.json`.
- `both` is schema-invalid. `postgres` is accepted by sprint `b-8` and is not
  a refused file. Amended 2026-10-25.
- OPEN: the test runner of `tests/unit/` is not decided (the driver is Python).

---

## REQ-RUN-0501: Generation driver `scripts/new_project.py` pipeline

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: The driver is the Python script `scripts/new_project.py`. The base driver is the driver at the template milestone, before the wizard extension of [REQ-RUN-0602](requirements.md). `<dest>` is the directory where the new project is created. Wizard answer acquisition is obtaining the answers object from the wizard instead of from `--var-file` ([REQ-RUN-0602](requirements.md)). An answers object is the generation-options JSON object validated by wizard/answers.schema.json. The six-step pipeline is the ordered table below. A failed step is a pipeline step that does not complete the action in its table row; for steps 3 to 6 this includes its command exiting with a non-zero status. A step's name is its entry in the Step column of the table. Running the six-step pipeline means running its six steps one after another in table order. The wizard extension is the change of [REQ-RUN-0602](requirements.md) that adds wizard answer acquisition.

The driver and `scripts/run_wizard.py` live under `scripts/`, outside `template/`, so they are never copied into a generated project; the repository layout that guarantees this is owned by [REQ-RUN-0001](requirements.md).

| # | Step | What the driver does |
|---|---|---|
| 1 | validate | Validates the answers object against `wizard/answers.schema.json`. |
| 2 | values file | Writes the validated answers to a TOML file as one `[values]` table, one entry per answers key. `string`, `bool` and `array` values are supported. |
| 3 | `cargo generate` | Runs `cargo generate --path template --template-values-file <values file> --name <project> --silent` with no terminal prompt. |
| 4 | `sc-compose` | Runs `sc-compose` so that the new project holds `AGENTS.md` and `CLAUDE.md` and no `.j2` file. What is rendered, from which inputs, and the removal of the two `.j2` files are owned by [REQ-RUN-0305](requirements.md); this item owns only the position of the step in the order. |
| 5 | `just lint` | Runs `just lint` in the new project. |
| 6 | `just test` | Runs `just test` in the new project. |

Template files are rendered only by `cargo generate` (step 3) and `sc-compose` (step 4).

Stopping at the first failed step and naming it on stderr is decided in this document. This paragraph is the single owner of that behaviour for all six steps, the `sc-compose` step included.

The later wizard milestone extends answer acquisition under [REQ-RUN-0602](requirements.md).

Steps 2 and 3 use the mechanism recorded in Active
[ADR-RUN-0402](architecture.md): the values file is one `[values]` table,
the supported types are `string`, `bool`, and `array`, and step 3 passes
`--template-values-file` and `--silent` so the command runs with no prompt.
Sprint a-1 proved that mechanism on cargo-generate 0.25.1 (rows V3a, V4a,
and V4b of [REQ-RUN-0102](requirements.md)). The six steps, their order, and
the rule that the driver stops at the first failure are unchanged.

**MUST statements:**

- The repository MUST contain a Python script `scripts/new_project.py`, called here the driver.
- The root `justfile` MUST expose the driver as `just new <dest>`.
- The base driver MUST obtain one complete answers JSON object from the file given with `--var-file`.
- The base driver MUST run the six-step pipeline on the answers object it obtained from `--var-file`.
- When a step fails, the driver MUST stop.
- If any step fails, the driver exit status MUST be non-zero.
- When a step fails, the driver MUST write that step's name to stderr.
- When every step succeeds, running `scripts/new_project.py` MUST return exit status 0.
- When wizard answer acquisition supplies the answers object, the driver MUST run the six-step pipeline on it.

**MUST NOT statements:**

- The wizard extension MUST NOT add a second generation driver.
- The wizard extension MUST NOT add a second owner for the six generation steps.
- After a step fails, the driver MUST NOT run any later step.
- The driver MUST NOT render any template file itself.

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
   `just lint`, `just test`, and that the `cargo generate` call contains
   `--path template`, `--template-values-file`, `--name`, and `--silent`.
3. The same test reads the file passed to `--template-values-file` and
   asserts it parses as TOML with a single `[values]` table. Once the
   project-name question above is decided, the table's keys equal the keys
   of the answers object, less the project-name key if that key is passed
   only through `--name`.
4. For each of steps 3 to 6, a driver test makes that step's stub exit
   non-zero and asserts: the driver exits non-zero, stderr identifies that
   step, and no later stub was called.
5. A driver test with an answers file that fails validation asserts that
   stderr identifies the validate step and that no stub was called.
6. The end-to-end run in [REQ-RUN-0503](requirements.md), which generates a
   real project from `wizard/fixtures/sqlite-mcp.json`, exits 0. That the
   project then holds `AGENTS.md` and `CLAUDE.md` and no `.j2` file is
   checked by the criteria of [REQ-RUN-0305](requirements.md).
### Implementation Notes

**Key Considerations:**

- OPEN: how `<dest>` maps onto `cargo generate`'s `--name` and output
  directory arguments is not decided.
- OPEN: the exact `sc-compose` command line (how the answers and the two
  `.j2` paths are passed) is not decided.
- OPEN: where the values TOML file is written, and whether it is deleted
  afterwards, is not decided.
- OPEN: whether the project-name answer is also written into the `[values]`
  table, or is passed only through `--name`, is not decided
  ([REQ-RUN-0402](requirements.md) owns the question of whether the
  project-name key is a placeholder).

---

## REQ-RUN-0502: Fully non-interactive mode and tool discovery

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: JSON means JavaScript Object Notation. ADR means architecture decision record. The driver is the generation script `scripts/new_project.py`, which the command `just new <dest>` runs. An answers file is a file that holds a complete answers JSON object for `wizard/answers.schema.json`. The wizard is the Wyvern answers editor that `scripts/run_wizard.py` starts. A prompt is text written to stdout or stderr that asks the user for input. sc-compose is the executable that validates and renders the agent-document Jinja templates. sc-compose is missing when `SC_COMPOSE` is unset and no directory on `PATH` holds an executable named `sc-compose`. A generation call is a `subprocess.run` call in `scripts/new_project.py` that does not start `scripts/run_wizard.py`. To start a program is to run it as a child process by any means, `subprocess.run` included. A stub is an executable put in place of a program that records each time it is started and the arguments it was given, so a stub's record shows whether and how that program was started. A driver test is an integration test that runs `scripts/new_project.py` as a child process with stubs first on `PATH`, then reads the driver's exit status, its stderr and the stubs' records. To start `cargo generate` is to start the program `cargo` with `generate` as its first argument; this holds whether `cargo` is the real program or a stub. A command accepts an option when a run given that option does not fail because of that option. A machine has no `wyvern` executable when `WYVERN_BIN` is unset and no directory on `PATH` holds an executable named `wyvern`. In `--var-file` mode, the absence of a `wyvern` executable is not a failure condition, whatever diagnostic or error code a failure is reported with. `GEN.TOOL_MISSING` naming `wyvern` is the missing-Wyvern diagnostic that [REQ-RUN-0602](requirements.md) specifies for the interactive and prefill modes; it does not limit which `--var-file` runs exit with a non-zero status because `wyvern` is absent.

This item is the single owner of the rule that `--var-file` never runs the wizard.

`cargo generate` is kept from prompting by `--template-values-file` and
`--silent`, as recorded in Active [ADR-RUN-0402](architecture.md). Sprint
a-1 proved the two options with stdin closed on cargo-generate 0.25.1 (row V4a of
[REQ-RUN-0102](requirements.md)).

| Tool | First | Then |
|---|---|---|
| `sc-compose` | the executable named by the environment variable `SC_COMPOSE`, when set | `sc-compose` on `PATH` |

Wyvern discovery belongs exclusively to [REQ-RUN-0602](requirements.md).

**MUST statements:**

- `just new <dest>` MUST accept the option `--var-file <answers file>`.
- In `scripts/new_project.py`, the `subprocess.run` call that starts `cargo generate` MUST pass `stdin=subprocess.DEVNULL`.
- Every generation call MUST pass `stdin=subprocess.DEVNULL`.
- When `--var-file` is given, the standard input of every program that the driver starts MUST be `/dev/null`.
- The `cargo generate` argument list in `scripts/new_project.py` MUST contain the option `--template-values-file`.
- The `cargo generate` argument list in `scripts/new_project.py` MUST contain the option `--silent`.
- When `SC_COMPOSE` is set, the sc-compose program that the driver starts MUST be the file named by `SC_COMPOSE`.
- When `SC_COMPOSE` is unset, the sc-compose program that the driver starts MUST be the first executable named `sc-compose` on `PATH`.
- When sc-compose is missing, `just new <dest>` MUST exit with a non-zero status.
- When sc-compose is missing, the stderr of `just new <dest>` MUST contain the text `sc-compose`.
- When sc-compose is missing, the stderr of `just new <dest>` MUST contain the text `SC_COMPOSE`.
- When sc-compose is missing, the stderr of `just new <dest>` MUST contain the text `PATH`.

**MUST NOT statements:**

- When sc-compose is missing, the driver MUST NOT start `cargo generate`.
- `scripts/new_project.py` MUST NOT contain code that reads standard input.
- When `--var-file` is given, `just new <dest>` MUST NOT write a prompt.
- When `--var-file` is given, the driver MUST NOT start `scripts/run_wizard.py`.
- `scripts/new_project.py` MUST NOT contain code that searches for the `wyvern` executable.
- On a machine with no `wyvern` executable, `just new <dest> --var-file <answers file>` MUST NOT exit with a non-zero status because `wyvern` is absent.
- `scripts/new_project.py` MUST NOT contain an absolute path to `sc-compose`.

### Rationale

CI and agents generate projects, and neither can answer a prompt or open a
wizard window. The non-interactive mode therefore has no Wyvern dependency.
An environment-variable override lets a machine point at sc-compose when it
is not on `PATH` without the script guessing install prefixes.
### Success Criteria

1. CI runs `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` with
   stdin closed (`< /dev/null`) on a runner with no `wyvern` installed, and
   the command exits 0, which shows that `just new <dest>` accepts
   `--var-file <answers file>`.
2. A driver test runs `--var-file` mode with `WYVERN_BIN` unset and no
   `wyvern` on `PATH` and asserts that the run does not fail for that reason.
3. A driver test runs `--var-file` mode with `WYVERN_BIN` set to a stub
   executable that records any call, and asserts that the stub was never
   called.
4. A driver test sets `SC_COMPOSE` to a stub executable, puts a different
   stub named `sc-compose` on `PATH`, and asserts that the one named by
   `SC_COMPOSE` was called.
5. A driver test with `SC_COMPOSE` unset, no `sc-compose` on `PATH` and a
   stub named `cargo` first on `PATH` asserts a non-zero exit and that stderr
   contains `sc-compose`, `SC_COMPOSE` and `PATH`.
6. When sc-compose is missing, the driver does not start `cargo generate`:
   a driver test with `SC_COMPOSE` unset, no `sc-compose` on `PATH` and a
   stub named `cargo` first on `PATH` asserts that the stub's record shows
   no start of `cargo` with `generate` as its first argument.
7. Inspection: `scripts/new_project.py` contains no absolute path to
   `sc-compose` and no Wyvern invocation or discovery logic, and every
   generation call in it passes `stdin=subprocess.DEVNULL`.
8. A driver test runs `--var-file` mode with `SC_COMPOSE` set to a stub and a
   stub named `cargo` first on `PATH`, each stub recording the file its
   standard input is connected to, and asserts that every recorded file is
   `/dev/null`.
9. When `--var-file` is given, `just new <dest>` writes no prompt: CI runs
   `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` with stdin
   closed (`< /dev/null`), captures its stdout and stderr, and asserts that
   neither contains text that asks the user for input.
10. When sc-compose is missing, `just new <dest>` exits with a non-zero
   status: CI runs `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json`
   with `SC_COMPOSE` unset and no `sc-compose` on `PATH`, and asserts a
   non-zero exit status and that stderr contains `sc-compose`, `SC_COMPOSE`
   and `PATH`.
11. On a machine with no `wyvern` executable, `just new <dest> --var-file
   <answers file>` does not exit with a non-zero status because `wyvern` is
   absent: the CI run of criterion 1 has valid answers, `WYVERN_BIN` unset, no
   `wyvern` on `PATH` and every other program the run needs available
   (sc-compose and `cargo generate`), and it must exit with status 0. As a
   supplementary check, that run's stderr does not contain the missing-Wyvern
   diagnostic `GEN.TOOL_MISSING` naming `wyvern`.

---

## REQ-RUN-0503: A generated project is green with no edits

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-05  
**Owner:** randlee  

### Requirement Statement

Definitions: The public package set is `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`, `sc-runtime-db-sqlite`, `sc-storage` and `sc-monty` (REQ-RUN-0002). CLI means command-line interface. MCP means Model Context Protocol. API means application programming interface. ADR means architecture decision record. REST means representational state transfer. A clean checkout is a checkout of this repository in which `git status --porcelain` prints nothing. `<dest>` is the directory where `just new` creates the project. The generated project is the project created at `<dest>`. The green sequence is `just setup`, then `just lint`, then `just test`, then `just build-cli`, then `just build-daemon`, run in that order in the generated project with no file added, changed or deleted after generation. The example operation test is the template's test of `widget.create` and `widget.get` (REQ-RUN-0302). The selected dependency graph is the dependency graph of the generated project for the fixture's answers `db = sqlite` and `mcp = true`.

This item owns the definition of "green with no edits" and applies it to one
answers fixture. [REQ-RUN-0701](requirements.md) applies the same sequence to
every fixture in CI, and [REQ-RUN-0702](requirements.md) requires it at the
release commit; neither restates it.

`wizard/fixtures/sqlite-mcp.json` is the answers fixture with `db = sqlite` and `mcp = true`.

`just setup` installs the sc-lint 0.8.4 consumer and is not a source edit. This sequence (`just new` with `--var-file`, then `just setup`, then `just lint`, then `just test`, then `just build-cli`, then `just build-daemon`, with nothing written to the project in between) is what "green with no edits" means.

Before those crates are published to crates.io, a `[patch.crates-io]` entry or a git tag supplies them; that allowance, and the undecided question of who writes the `[patch.crates-io]` entry, are owned by [REQ-RUN-0003](requirements.md).

`just new` renders the project with `cargo generate`. Conditional `ignore`,
`exclude`, and a values file that runs with no prompt are recorded in Active
[ADR-RUN-0402](architecture.md). Sprint a-1 proved them (rows V3a, V3b, V4a,
and V4b of [REQ-RUN-0102](requirements.md)).

**MUST statements:**

- Running `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` from the root of a clean checkout MUST exit 0.
- Running `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json` MUST create a project at `<dest>`.
- Every command of the green sequence MUST exit 0.
- `just test` in the generated project MUST run the example operation test.
- The example operation test MUST call `widget.create` through REST.
- The example operation test MUST call `widget.create` through MCP.
- The example operation test MUST call `widget.create` through the CLI.
- The example operation test MUST call `widget.get` through REST.
- The example operation test MUST call `widget.get` through MCP.
- The example operation test MUST call `widget.get` through the CLI.
- The example operation test MUST run against a daemon started by `sc_runtime::testing::DaemonFixture`.
- Every crate of the public package set that is in the selected dependency graph MUST resolve during the green sequence.

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
2. The same job then runs `just setup` immediately before `just lint` in
   `$RUNNER_TEMP/app`. `just setup` installs the consumer tool and does not
   edit source. It exits 0.
3. The same job then runs `just lint` in `$RUNNER_TEMP/app` with no step in
   between that writes to that directory, and it exits 0.
4. The same job then runs `just test` in `$RUNNER_TEMP/app`, and it exits 0.
   `just test` includes the observability grep: `crates/daemon` requires
   `sc-observability` with the registry requirement `>=1.5.0` and no upper bound under ADR-RUN-0017, does not require `sc-observability-otlp`, and
   the logging-crate token does not match `sc-observability-types`.
5. The same job then runs `just build-cli` in `$RUNNER_TEMP/app`, and it
   exits 0. That recipe includes `cargo tree -p <project>-cli -e normal --prefix none`.
   Amended 2026-10-25.
6. The same job then runs `just build-daemon` in `$RUNNER_TEMP/app`, and it
   exits 0. That recipe includes `cargo tree -p <project>-daemon -e normal --prefix none`
   and an automated normal-dependency ancestry check. `tempdir` is forbidden.
   `tempfile` is allowed only as the existing direct normal dependency of the
   qualified `sc-cron` package for private Python-hook scratch directories;
   every normal path from the daemon to that package must pass through `sc-cron`.
   A direct daemon dependency or any other direct/transitive ancestry fails.
   Daemon instance roots, endpoints, locks and application database paths must
   still never use a temporary-directory API in production.
   Amended by the approved selected-cron clarification of 2026-11-06.
7. The `just test` output lists the example operation test as run and passed,
   not skipped or filtered out.

---

## REQ-RUN-0601: A Wyvern wizard built by copy and edit

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions:

- The wizard is the Wyvern HTML user interface that collects generation options; its files are listed in the path table.
- The path table is the table in this record whose first column is headed `Path in this repository`.
- Generation options are the properties defined in `wizard/answers.schema.json` in the same commit as the wizard pages.
- The source wizard is the files `.scaffold/wizard/wizard.json`, `.scaffold/wizard/pages/*.html`, `.scaffold/wizard/pages/wizard-answers.js` and `.scaffold/wizard/pages/wizard-page.css` in the repository `Radiant-Vision-Systems/p3-nuget-template` at commit `9ef4d03`.
- The corresponding source wizard file of a path-table file is the source wizard file whose path relative to `.scaffold/wizard/` equals the path-table file's path relative to `wizard/`.
- The copied wizard is the source wizard copied into this repository's `wizard/` directory.
- An option-input field is an HTML element in `wizard/pages/*.html` that carries a `data-answer` attribute.
- The schema property of an option-input field is the `wizard/answers.schema.json` property whose key equals the field's `data-answer` value.

Wyvern is the tool that shows a wizard's HTML pages and prints the collected data as JSON on stdout when the person finishes. The wizard consists of:

| Path in this repository | What it is |
|---|---|
| `wizard/wizard.json` | the Wyvern wizard descriptor |
| `wizard/pages/*.html` | the wizard pages |
| `wizard/pages/wizard-answers.js` | the helper script shared by the pages |
| `wizard/pages/wizard-page.css` | the stylesheet shared by the pages |

The number of pages and their file names are not constrained.

The wizard lives under `wizard/`, outside `template/`, so no wizard file is
rendered into a generated project. The repository layout that guarantees this
is owned by [REQ-RUN-0001](requirements.md); this item checks the result for
the wizard's files only (criterion 5).

**MUST statements:**

- This repository MUST contain a file for every path in the path table.
- When a person runs the wizard in Wyvern, the wizard MUST collect the person's choice for each generation option through its HTML pages.
- The path-table files MUST be created by copying the corresponding source wizard files.
- The copied wizard MUST be edited to produce this repository's wizard.
- The `data-answer` attribute values in `wizard/pages/*.html` MUST name the same options as the `properties` keys of `wizard/answers.schema.json`.
- The `<option>` `value` attributes inside each option-input field in `wizard/pages/*.html` MUST name the same values as the `enum` of that field's schema property in `wizard/answers.schema.json`.
- When a person runs `just new <dest>` without `--var-file`, the wizard pages that Wyvern displays MUST show one input for every generation option.

**MUST NOT statements:**

- When a person runs `just new <dest>` without `--var-file`, the wizard pages that Wyvern displays MUST NOT show an input for anything other than a generation option.
- The wizard MUST NOT be written from scratch.

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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: An answers object is a JSON object of generation-option values. The driver is the generation driver `scripts/new_project.py`, run as `just new <dest>`. The three modes are interactive, prefill and non-interactive; the mode table in this requirement gives each mode's command line and behaviour. Wyvern is the program that runs the wizard. The generation steps are validation, values-file creation, cargo generate, sc-compose, just lint and just test. The destination is the new project output directory `<dest>` passed to `just new <dest>`.

| Mode | Command line | Behaviour |
|---|---|---|
| interactive | neither `--var-file` nor `--prefill` | The driver runs the Wyvern wizard described by `wizard/wizard.json`. The person types every answer. |
| prefill | `--prefill <json>` | `<json>` is a partial answers object, typically written by an agent as its best guess. The driver merges it into the `config.prefill` object of the wizard descriptor and then runs the wizard, so the person reviews and corrects the values and does not type them. |
| non-interactive | `--var-file <json>` | `<json>` is a complete answers object. The wizard is not run in this mode; that rule and its test are owned by [REQ-RUN-0502](requirements.md). |

No later step may behave differently depending on which mode supplied the answers.

Wizard acquisition is an extension owned by this requirement. It MAY add entry-mode selection to `scripts/new_project.py` and SHOULD reuse `scripts/run_wizard.py` from `p3-nuget-template` with as few changes as possible. Neither script may contain a hard-coded Wyvern install path.

Prefill values are suggestions only. The answers that are validated and used
are the ones the wizard returns after the person finishes it.

This rule is decided in this document.

When `--prefill` and `--var-file` are both given, `--var-file` wins. The
wizard does not start and the prefill file is not read.

A `--prefill` key that the schema does not define is dropped, with a warning
on stderr. The on-disk `wizard/wizard.json` is not modified. The reference
script in `p3-nuget-template` drops such keys the same way.

When the person cancels or dismisses the wizard, the process exits non-zero.
The last stderr line is `GEN.WIZARD_CANCELLED`. The driver does not run
`cargo generate` and does not create the destination. A missing Wyvern binary
is `GEN.TOOL_MISSING`; the cause names `wyvern`, `WYVERN_BIN`, and `PATH`.
A runner-side schema reject is `GEN.SCHEMA_INVALID`. A runner-side
project-name reject is `APP.INVALID_NAME`. These are the same inventory rows
as [ADR-RUN-0401](architecture.md).

**MUST statements:**

- The driver MUST support the interactive mode of obtaining the answers object.
- The driver MUST support the prefill mode of obtaining the answers object.
- The driver MUST support the non-interactive mode of obtaining the answers object.
- In interactive mode, the driver MUST behave as the Behaviour column of the mode table states for the interactive row.
- In prefill mode, the driver MUST behave as the Behaviour column of the mode table states for the prefill row.
- In non-interactive mode, the driver MUST behave as the Behaviour column of the mode table states for the non-interactive row.
- In all three answer-acquisition modes, the answers object MUST be validated against `wizard/answers.schema.json`.
- In all three answer-acquisition modes, the validated answers object MUST pass through the same generation steps.
- When `WYVERN_BIN` is set, wizard code MUST look for Wyvern through `WYVERN_BIN` before it looks for `wyvern` on `PATH`.
- When `WYVERN_BIN` is unset or does not supply Wyvern, wizard code MUST look for Wyvern as `wyvern` on `PATH`.
- When `WYVERN_BIN` is unset and `wyvern` is not on `PATH`, `just new <dest>` in wizard mode MUST exit before it runs `cargo generate`.
- If neither location supplies Wyvern, wizard code MUST name `wyvern` on stderr.
- If neither location supplies Wyvern, `just new <dest>` in interactive or prefill mode MUST write the text `WYVERN_BIN` to stderr.
- If neither location supplies Wyvern, wizard code MUST name `PATH` on stderr.
- If neither location supplies Wyvern, wizard code MUST leave no destination.

**MUST NOT statements:**

- `scripts/new_project.py` MUST NOT contain code for any mode of obtaining the answers object other than interactive, prefill and non-interactive.
- When the wizard ends without the person finishing it, the driver MUST NOT run `cargo generate`.

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
   `cargo generate` was not called and that the last stderr line is
   `GEN.WIZARD_CANCELLED`.
6. With `WYVERN_BIN` unset and no `wyvern` on `PATH`, interactive mode fails
   before `cargo generate`, the last stderr line is `GEN.TOOL_MISSING`, that
   line names `wyvern`, `WYVERN_BIN`, and `PATH`, and no destination is
   created.
7. Inspection finds no absolute Wyvern path in either driver script.

---

## REQ-RUN-0603: Wizard output equals the equivalent answers fixture

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: Wyvern is the program that shows a wizard's HTML pages and prints the collected answers as JSON on stdout when the person finishes. The wizard is the Wyvern wizard described by `wizard/wizard.json`. The driver is the generation driver `scripts/new_project.py`, run as `just new <dest>`. A fixture is one answers JSON file under `wizard/fixtures/`. The wizard output file is the saved file holding the JSON object that Wyvern prints on stdout when a person finishes the wizard. The matching fixture is the fixture whose values are the choices the person made in the wizard. Two JSON objects are equal when, after parsing, they have the same keys and each key has the same value with the same JSON type; key order and whitespace do not matter. The wizard tree is the project directory that the driver generates from the wizard output file. The fixture tree is the project directory that the driver generates from the matching fixture with `--var-file`. The checkout patch is the `[patch.crates-io]` section of the generated root `Cargo.toml`, which maps SDK crate names to local checkout paths. The excluded paths are `target/`, `.sc-lint/venv/` (the local sc-lint install embeds the destination path) and `.sc-compose/logs/` (the generator log contains timestamps).

In CI, a stub that prints `wizard/fixtures/sqlite-mcp.json` replaces Wyvern, so CI does not need the real binary; the driver runs once with the stub and once with `--var-file` on the same fixture. Success criteria 1 to 3 check, on a machine with Wyvern, that the wizard prints the matching fixture.

**MUST statements:**

- Validation of the wizard output file against `wizard/answers.schema.json` MUST report no error.
- A JSON equality check of the wizard output file against the matching fixture MUST report no difference.
- A recursive diff of the wizard tree against the fixture tree, ignoring the excluded paths, MUST report no file path present in only one tree.
- A recursive diff of the wizard tree against the fixture tree, ignoring the excluded paths and the checkout patch, MUST report no file whose bytes differ.

**MUST NOT statements:**

- Validation of the wizard output file against `wizard/answers.schema.json` MUST NOT report a key that the schema does not define.
- Validation of the wizard output file against `wizard/answers.schema.json` MUST NOT report a missing required key.

### Rationale

Because the wizard and a hand-written answers file produce the same JSON,
nothing else in the system depends on which one was used. That is what lets
the wizard be built last, slip, or be reshaped without touching the schema,
the template, the driver or CI.
### Success Criteria

1. On a machine with Wyvern installed, run the wizard and choose the project
   name, `db = sqlite`, `mcp = true` and the licence exactly as
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
5. An automated test stubs Wyvern, runs the driver once from that stub with
   the answers in `wizard/fixtures/sqlite-mcp.json` and once with
   `--var-file` on that fixture, and compares the generated trees. Paths
   under `target/`, `.sc-lint/venv/`, and `.sc-compose/logs/` are ignored.
   The `[patch.crates-io]` checkout patch is ignored. Every other file
   matches.

---

## REQ-RUN-0701: Template CI: answers-fixture matrix workflow

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-30  
**Owner:** randlee  

### Requirement Statement

Definitions: CI means continuous integration. JSON means JavaScript Object Notation. ADR means architecture decision record. The workflow is the one GitHub Actions workflow that this record requires, and the workflow file is the YAML file that defines it. A fixture is one answers JSON file matching `wizard/fixtures/*.json`. The fixture name is the fixture's file name without the `.json` suffix. A generated project is the directory tree at `<dest>` that command 1 of the numbered list below renders from `template/` for one fixture. The fixture sequence is commands 1 to 6 of the numbered list below, in that order, and command 7 once the selection question of NFR-RUN-0010 is decided. A matrix entry is one job instance that the `strategy.matrix` key of the workflow file produces. The matrix job is the job in the workflow file that carries that `strategy.matrix` key. The fixture list is the set of files that match `wizard/fixtures/*.json` in the checkout under test, listed when the workflow runs. A branch filter is a `branches` or `branches-ignore` key. A path filter is a `paths` or `paths-ignore` key. The public package set is `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`, `sc-runtime-db-sqlite`, `sc-storage` and `sc-monty`. The selected dependency graph of a generated project is the set of packages listed in its `Cargo.lock`. The checkout under test is the commit the workflow run checked out. A crate is resolved from the checkout under test when its `Cargo.lock` entry has no `source` line; a crate taken from crates.io or from another commit is not resolved from it. The release run is the run of the workflow that REQ-RUN-0702 requires for a release. A test run is any run of the workflow other than the release run. A tested crate of a generated project is a crate of the public package set that is in the project's selected dependency graph. Postgres is the PostgreSQL database engine. A Postgres service container is an entry under the `services:` key of the matrix job that starts a Postgres server for the generated project. The `db` value of a fixture is the value of the `db` key in its JSON object. Wyvern is the optional interactive wizard program, whose executable is named `wyvern`.

1. `just new <dest> --var-file <fixture>` from the repository root, which runs the generation driver `scripts/new_project.py` without the wizard.
2. `just setup` inside the generated project at `<dest>`. It installs the sc-lint 0.8.4 consumer and is not a source edit.
3. `just lint` inside the generated project at `<dest>`.
4. `just test` inside the generated project at `<dest>`. It includes the observability grep: rendered `crates/daemon` requires `sc-observability` with the registry requirement `>=1.5.0` and no upper bound under ADR-RUN-0017, does not require `sc-observability-otlp`, and the logging-crate token does not match `sc-observability-types`.
5. `just build-cli` inside the generated project at `<dest>`. It includes `cargo tree -p <project>-cli -e normal --prefix none`. Amended 2026-10-25.
6. `just build-daemon` inside the generated project at `<dest>`. It includes `cargo tree -p <project>-daemon -e normal --prefix none` and the same normal-dependency ancestry check required by REQ-RUN-0503 criterion 6. It fails for any `tempdir`, any direct daemon `tempfile` edge, or any `tempfile` ancestry outside the qualified `sc-cron` private Python scratch path. Amended by the explicitly approved selected-cron clarification of 2026-11-06.
7. Once the selection question of [NFR-RUN-0010](requirements.md) is decided: the generated project's daemon-process tests (the tests that start the generated `daemon` binary), selected the way that decision says. The CI runner is the isolated machine those tests need.

No step of the workflow may write to `<dest>` between those commands. This is the "green with no edits" sequence that [REQ-RUN-0503](requirements.md) defines for one fixture; this record owns running it for every fixture in CI.

**MUST statements:**

- The workflow file MUST be located in the directory `.github/workflows/` of this repository.
- The matrix job MUST contain a step that runs command 1 of the numbered list.
- Each matrix entry MUST run the fixture sequence on its fixture.
- A run of the workflow MUST fail when a command of the fixture sequence exits with a non-zero status for any fixture.
- The workflow MUST run each fixture in its own matrix entry.
- The job name of each matrix entry MUST contain its fixture name.
- The workflow MUST take the values of its matrix from the fixture list.
- The `on:` key of the workflow file MUST list the `pull_request` event.
- In every test run, each tested crate of a generated project MUST be resolved from the checkout under test.
- From the release that adds the `store-postgres` crate, the workflow MUST give a Postgres service container to every fixture whose `db` value is `postgres` or `sqlite-postgres`.

**MUST NOT statements:**

- The `pull_request` entry of the `on:` key MUST NOT contain a branch filter.
- The `pull_request` entry of the `on:` key MUST NOT contain a path filter.
- The workflow file MUST NOT contain the file name of any fixture.
- The workflow file MUST NOT contain a step that installs Wyvern.
- The workflow file MUST NOT contain a step that invokes Wyvern.
- The matrix job MUST NOT contain a `uses:` line that names the `cargo-generate-action` GitHub Action.

### Rationale

Every supported option combination is generated, linted and tested on every change. That catches a new `rmcp` release, a crate change or a template edit breaking generated projects before a user meets it. Running the real driver with `--var-file` tests the same path that agents and CI users take; the driver is what is tested. Keeping the crates and the template in one repository is only useful if CI tests them together at one commit. After the first publish a generated project would otherwise resolve the published crates, and a pull request would be tested against the old crate code. The four matrix rules (one entry per fixture, the list derived from the directory, no workflow edit for a new fixture, every pull request) are decided in this record. The Postgres statement places no obligation on v0.1, because the only accepted `db` value in v0.1 is `sqlite`. Open: the workflow file name is not decided, and who writes the `[patch.crates-io]` entries into the generated project is owned by REQ-RUN-0003.

### Success Criteria

1. Inspection: exactly one file under `.github/workflows/` contains a job with a `strategy.matrix` key whose steps run `just new` with `--var-file`; that file is the workflow file (statements 1 and 2).
2. Inspection: the steps of the matrix job run `just new <dest> --var-file <fixture>`, `just setup`, `just lint`, `just test`, `just build-cli` and `just build-daemon` in that order, with no step between them that writes to `<dest>` (statement 3).
3. Inspection: no step of the fixture sequence carries `continue-on-error: true`, and the matrix job does not carry it either (statement 4).
4. Inspection: the matrix has one dimension whose values are the fixtures, and the `name:` of the matrix job interpolates the fixture name (statements 5 and 6).
5. Inspection: a step or job of the workflow file lists `wizard/fixtures/*.json` at run time and passes the result to `strategy.matrix` (statement 7).
6. Inspection: the `on:` key has a `pull_request` entry with no `branches`, `branches-ignore`, `paths` or `paths-ignore` key (statements 8, 11 and 12).
7. Inspection of a pull-request run's artifacts: the `Cargo.lock` of each generated project lists every public package selected by that fixture with no `source` line (statement 9).
8. Inspection, once `store-postgres` exists: the matrix job has a `services:` entry for Postgres that applies to every fixture whose `db` value is `postgres` or `sqlite-postgres` (statement 10).
9. `ls wizard/fixtures/` piped to `grep -F -f - <workflow file>` prints nothing (statement 13).
10. `grep -ri "wyvern" .github/workflows/` prints nothing (statements 14 and 15).
11. `grep -n "cargo-generate-action" <workflow file>` prints nothing (statement 16).
### Implementation Notes

**Key Considerations:**

- OPEN: the workflow file name is not decided.
- OPEN: who writes the `[patch.crates-io]` entries into the generated
  project (a template option, the driver `scripts/new_project.py`, or this
  workflow) is not decided; [REQ-RUN-0003](requirements.md) owns the question.
  Writing them after generation must not count as an edit that breaks "green
  with no edits"; how that is reconciled is part of the same question.

---

# Requirement: Coordinated public release and registry consumers

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-RUN-0702: Coordinated public release and registry consumers

**Status:** Active  

### Requirement Statement

Definitions for the following statements: The SDK (software development kit) is the nine public crates of REQ-RUN-0002: `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`, `sc-runtime-db-sqlite`, `sc-storage` and `sc-monty`. The qualified publishing kit is sc-publish at the revision pinned in `release/sc-publish-pin.toml`. A populated repository is a generated project with its selected template rendered. ADR means architecture decision record. The first publication wave is the first group of packages published in the dependency order. sc-monty is the Monty interpreter crate; sc-storage is the shared storage crate. The released template is the `template/` directory at the release tag. The registry proof run is the workflow run that performs the registry proof; the registry proof commit is the commit it checks out. The rehearsed archives are the packaged archives that passed archive rehearsal. In the rehearsed archives, the source files and Cargo manifests of a package are the files unpacked from its `.crate` archive; in the registry proof commit, they are the files under the package directory of that SDK crate, at the same relative paths. Two sets of files differ when a path is in one set only or the bytes at a path are not identical in both, that is, when `diff -r` of the two directories prints any line. Human authorization and registry credentials are the existing release permission and credential requirements. The release run is the run of the Template workflow (`.github/workflows/template.yml`) started by pushing the release tag, in which every generated project resolves the SDK from crates.io with no checkout patch. Registry proof is the build and tests of a consumer whose Cargo.lock gives a crates.io source for every SDK crate, with no checkout path, Git or patch substitute. An actual SDK consumer is a project that depends on SDK crates published to crates.io, such as a project generated from the released template. The controlled-host benchmark check is the publish preflight run of `scripts/benchmark/check_release_evidence.py` on committed controlled-host benchmark evidence for the release tag commit, required by ADR-RUN-0015. A fixture matrix entry is the Template workflow job for one answers JSON file under `wizard/fixtures/`. The daemon-process tests are the tests that NFR-RUN-0010 requires to run on an isolated machine. For this release, sc-storage has no dependency on another package in the release; sc-monty likewise has no dependency on another package in the release. JSON means JavaScript Object Notation. A checkout-path dependency resolves to a local checkout; a patched source overrides the registry package source. The preflight checks and package checks are the respective checks supplied by the qualified publishing kit. This record governs the coordinated 0.3.0 release of the SDK and delivery of the generated template; it is not a permanent restriction on consumers choosing different dependency versions.





Definitions for the following statements: Newest-tested Cargo.lock records the dependency versions most recently tested for the delivered release; .sc-runtime/tested-dependencies.json is the matching tested-dependency metadata. The dependency order is the topological order of the published package dependencies in the final Cargo manifests. The Cargo manifests of a package are its `Cargo.toml` files; its source files are its other files, apart from the files that `cargo package` adds (`Cargo.toml.orig`, `Cargo.lock` and `.cargo_vcs_info.json`). The release lock is the newest-tested Cargo.lock. A crates.io source is a Cargo.lock `source` value of `registry+https://github.com/rust-lang/crates.io-index`. Git is the source version-control system; Cargo is the Rust package manager.








Definitions for the following statements: Archive rehearsal is the pre-upload build and tests of the exact packaged archives that the release uploads.












Definitions for the following statements: The release tag is the Git tag `v0.3.0` of this repository.

**MUST statements:**

- The crates.io page of each SDK crate MUST list version 0.3.0.
- The crates-publish workflow (`.github/workflows/crates-publish.yml`) MUST upload the packages in the dependency order.
- The first publication wave MUST include sc-storage.
- The first publication wave MUST include sc-monty.
- Every SDK crate entry in the Cargo.lock of a project generated from the released template MUST have a crates.io source.
- The job of every fixture matrix entry in the release run MUST finish with conclusion success.
- The crates-publish workflow run that uploads the published packages MUST check out the commit that the release tag points to.
- The checkout step log of the registry proof run MUST show the commit that the release tag points to.
- The release MUST use the qualified publishing kit.
- Before upload, the release MUST pass the preflight checks of the qualified publishing kit.
- Before upload, the release MUST pass the package checks of the qualified publishing kit.
- Archive rehearsal MUST exit with status 0.
- After publication, the registry proof of each actual SDK consumer MUST exit with status 0.
- Publication MUST preserve the existing human-authorization requirements.
- Publication MUST preserve the existing registry-credential requirements.
- The controlled-host benchmark check MUST pass before the release uploads any package.
- Before upload, the daemon-process tests MUST pass for the release tag commit.
- Each delivered template release MUST include its newest-tested Cargo.lock.
- Each delivered template release MUST include its tested-dependency metadata.
- Generated populated repositories MUST receive the release lock.
- Generated populated repositories MUST receive the tested-dependency metadata.

**MUST NOT statements:**

- The released template MUST NOT substitute a checkout-path dependency for a registry SDK dependency.
- The released template MUST NOT substitute a Git dependency for a registry SDK dependency.
- The released template MUST NOT substitute a patched source for a registry SDK dependency.
- Checkout-patched consumer tests MUST NOT be reported as registry proof.
- The source files of the registry proof commit MUST NOT differ from the source files of the rehearsed archives.
- The Cargo manifests of the registry proof commit MUST NOT differ from the Cargo manifests of the rehearsed archives.


### Rationale

Published registry packages permit consuming projects to upgrade the SDK through Cargo. A tested lock and dependency metadata recommend a known combination without restricting consumer dependency choices.

### Success Criteria

Inspect .github/workflows/crates-publish.yml and its release run for dependency order and checkout identity. Inspect the release artifacts and consumer Cargo.lock sources. Check the recorded exit statuses of rehearsal and registry proof, pre-upload check results and fixture conclusions. Compare archive source/manifests by path and bytes. Confirm unchanged authorization and registry-credential requirements. Inspect a delivered template release and a generated populated repository for the newest-tested Cargo.lock and `.sc-runtime/tested-dependencies.json`.

### Implementation Notes

ADR-RUN-0017 governs the retained observability shim. Phase D uses open dependency floors under ANC-037; obsolete earlier-release pins and forced-cleanup probes do not govern this release.

### Related Documents

**Architecture Decisions:**

- ADR-RUN-0017 — Initial release amendment — 2026-10-31

## NFR-RUN-0001: A CLI never links server-side dependencies

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. `axum` is the Rust HTTP server crate. `rmcp` is the Rust crate that serves MCP (Model Context Protocol). `sqlx` is the Rust SQL database driver crate. The generated `cli` crate is the package rendered at `crates/cli` of a generated project (`<project>-cli`, REQ-RUN-0329). `api-types`, `service` and `daemon` name the generated packages `<project>-types`, `<project>-service` and `<project>-daemon`; a `store-*` crate is a generated package whose role begins `store-` (REQ-RUN-0301). The baseline CLI allowlist is `api-types`, `sc-transport`, `sc-command` and `sc-config`, plus `sc-cron-commands` when standard cron commands are selected (ADR-RUN-0019). `server` is the cargo feature of that name in `sc-transport` and `sc-command`. The normal dependency graph of a package is the set of crates, direct and transitive, that `cargo tree -p <package> -e normal` prints in the workspace holding that package; dev-dependencies and build-dependencies are not in it. A client crate is a command-line client crate with a direct normal dependency on `sc-transport` or `sc-command`; the generated `cli` crate is a client crate. A normal dependency graph contains a crate when `cargo tree -p <package> -e normal --prefix none` prints a line beginning with that crate's name followed by a space. A client crate enables a feature of a crate when `cargo tree -p <client crate> -e normal -f '{p} {f}'` lists that feature on that crate's line. A project crate is a package of the generated project's workspace or a package of the sc-runtime public package set (`sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`, `sc-runtime-db-sqlite`, `sc-storage`, `sc-monty`).

Phase C planned amendment, owned by c-1: the baseline CLI allowlist
includes defaults-only sc-cron-commands DTOs when standard
cron commands are explicitly selected, directly or through api-types. This is
the sole shared CronRequest identity, not a copied wire DTO. The exception
introduces no scheduler/server/SQLite/Monty dependency through the adapter
edge and changes
none of the existing forbidden normal edges. C-3's isolated CLI graph proof
and c-4's registry repetition verify the amended rule; default-off generation
is unchanged. REQ-CRNC-0002/NFR-CRNC-0001 and ADR-RUN-0019 govern that narrow
optional adapter edge.

This item is the single owner of the rule that a command-line client does not
depend on the daemon's stack. Other items that touch the rule reference this
one and use its criteria.

Dev-dependencies and build-dependencies are outside the rule, so a test inside the `cli` crate may use server-side crates. The rule applies to the `cli` crate of every generated project (`crates/cli` in the generated workspace).

The rule is defined for a CLI built as its own cargo selection:
`cargo build -p <cli package>`. In that build cargo resolves features for
the CLI's own dependency graph only.

To make that possible:

- The feature name comes from the sc-runtime design; that it is off by default is decided in this document. The per-crate statement of this, the allowed third-party dependencies and the criteria on the two crates are owned by [NFR-TRN-0001](sc-transport/requirements.md) and [NFR-CMD-0001](sc-command/requirements.md).

Sprint a-1 measured both selections and closed this question. The normative
selection is `cargo build -p <project>-cli`. v0.1 rendered that package as
`cli`, and a-6 measured `cargo tree -p cli -e normal` on that package. Sprint
`b-8` renders the package as `<project>-cli` and the daemon package as
`<project>-daemon` ([REQ-RUN-0329](requirements.md)). Amended 2026-10-25. A
workspace-wide build is exempt: under `cargo build --workspace` the CLI graph
may contain `axum`, `rmcp`, and `sqlx` because cargo unifies the `server`
feature. Splitting `sc-transport` and `sc-command` into client and server
crates is rejected for v0.1. a-5 does not run that command. The measurement
is the sprint a-1 evidence record in `docs/architecture.md`.

**MUST statements:**

- In `crates/sc-transport/Cargo.toml`, `dep:axum` MUST appear only in the `server` entry of `[features]`.
- In `crates/sc-command/Cargo.toml`, `dep:axum` MUST appear only in the `server` entry of `[features]`.
- In `crates/sc-command/Cargo.toml`, `dep:rmcp` MUST appear only in the `server` entry of `[features]`.

**MUST NOT statements:**

- The `[dependencies]` table of the generated `crates/cli/Cargo.toml` MUST NOT name a project crate outside the baseline CLI allowlist.
- In `crates/sc-transport/Cargo.toml`, the `default` entry of `[features]` MUST NOT list `server`.
- In `crates/sc-command/Cargo.toml`, the `default` entry of `[features]` MUST NOT list `server`.
- The normal dependency graph of a client crate MUST NOT contain `axum`.
- The normal dependency graph of a client crate MUST NOT contain `rmcp`.
- The normal dependency graph of a client crate MUST NOT contain `sqlx`.
- A client crate MUST NOT enable the `server` feature of `sc-transport`.
- A client crate MUST NOT enable the `server` feature of `sc-command`.
- The normal dependency graph of the generated `cli` crate MUST NOT contain the `sc-runtime` crate.
- The normal dependency graph of the generated `cli` crate MUST NOT contain the `service` crate.
- The normal dependency graph of the generated `cli` crate MUST NOT contain the `daemon` crate.
- The normal dependency graph of the generated `cli` crate MUST NOT contain any `store-*` crate.

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
5. Workspace-wide builds are exempt, as stated in the Requirement Statement.
   No criterion inspects the `cli` binary produced by `cargo build --workspace`.
   a-6 owns `cargo tree -p <cli package> -e normal`. a-5 does not run it.


Selected `cron_commands` consumers may depend on defaults-only
sc-cron-commands DTOs directly or through api-types. That edge must introduce
no scheduler, SQLite, Monty or server dependency; isolated CLI graph
proofs and all other forbidden edges remain binding. `cron_commands=false`
is the generation default; true requires cron and preserves independently
disabled MCP. Schema activation belongs only on selected daemon/service
projection edges, never shared CLI defaults.
### Implementation Notes

**Key Considerations:**

- OPEN: the recipe or command by which a generated project builds and
  releases its `cli` binary (so that it is built with `-p`) is not decided. The
  public recipes of the sc-lint 0.8.4 install do not include a build recipe.
### Related Documents

**Architecture Decisions:**

- ADR-RUN-0019 — Phase C planned amendment, owned by c-1

---

## NFR-RUN-0002: Async end to end

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: MCP means Model Context Protocol. HTTP means Hypertext Transfer Protocol. ADR means architecture decision record. Tokio is the asynchronous Rust runtime that runs daemon request handling. The request path is the code that runs from accepting a connection to writing that connection's response, in a generated project's daemon and in the `sc-runtime` and `sc-transport` library crates. A generated project is a project created from this repository's `template` directory. A REST handler is a function in a generated project's `crates/daemon/src/routes.rs` that serves an HTTP request. An MCP tool handler is a function in a generated project's `crates/daemon/src/mcp.rs` that serves an MCP tool call. A client request method is a public method of `sc_transport::Client` in `crates/sc-transport/src/client.rs` that sends a request to a daemon and returns the daemon's response, such as `Client::get` and `Client::post`. A service function is a `pub fn` in a generated project's `crates/service` crate. A store call is a request-serving method of the `Storage<T>` implementation in a generated project. A pure function is a constructor, parser or conversion that performs no I/O and never waits. A connection-serving function is a function, other than a pure function, that runs between accepting a connection and writing that connection's response.

Storage and other I/O reached by async callers must not block their Tokio worker or use a sync-over-async bridge. This applies beyond HTTP request handlers. `sc-config` remains synchronous, with no new reload/async work (PRD-ENG-5/6); pure constructors, parsers and conversions do not become async. The bind/shutdown filesystem carve-out below remains. SQLite work runs on dedicated threads: only the writer thread drives its own runtime with `Runtime::block_on` under ADR-RUN-0011. Reader workers receive with `blocking_recv`, have no runtime and need no exception; callers still enqueue and await without blocking. A SQLite store call awaits the writer's channel reply; a Postgres or MySQL store call awaits an `sqlx::Pool` (PRD-STO-3, amended 2026-11-11).

2026-10-31 clarification: ADR-RUN-0017 permits only application-adapter
try_log admission, which has no capacity wait/sink I/O but takes internal locks.
This is not a wait-free guarantee; all other request-path rules below apply.

The sc-runtime design states only that handler, service function and sqlx pool are async; this list of prohibited calls is decided in this document; it is not stated by the sc-runtime design.

Extending the rule to the library crates is decided in this document; it is not stated by the sc-runtime design.

Bind-time and shutdown-time file operations in the library crates are outside
the request path and are not restricted by this item. Those are the
operations that run once, before the first connection is accepted or after
the last one is closed: removing a stale socket file and setting the socket
file's permissions when `sc-transport` binds
([REQ-TRN-0004](sc-transport/requirements.md)), taking `daemon.lock`, and
removing the socket file during the `sc-runtime` shutdown sequence
([REQ-RT-0005](sc-runtime/requirements.md)). They MAY use `std::fs`.

`sc-config` stays synchronous under PRD-ENG-5/6. Loading at start-up describes
the current use; it does not forbid a consumer from choosing a later reload.
No watcher, reload or async configuration work follows from this exception.
The nonblocking Tokio path, with the exceptions above, is recorded in
[ADR-RUN-0007](architecture.md).

**MUST statements:**

- Every REST handler MUST be declared `async fn`.
- Every MCP tool handler MUST be declared `async fn`.
- Every service function MUST be declared `async fn`.
- Every store call MUST be declared `async fn`.
- Every connection-serving function in `crates/sc-runtime/src` MUST be declared `async fn`.
- Every connection-serving function in `crates/sc-transport/src` MUST be declared `async fn`.
- Every client request method MUST be declared `async fn`.

**MUST NOT statements:**

- Request-path code MUST NOT block a Tokio runtime worker thread.
- Request-path code MUST NOT call `block_on`.
- Request-path code MUST NOT call `std::thread::sleep`.
- Request-path code MUST NOT call blocking `std::fs` I/O.
- Request-path code MUST NOT call blocking `std::net` I/O.
- Request-path code MUST NOT call `reqwest::blocking`.

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
   `async fn`. Sprint a-5 reviewed criteria 4 and 5. `Client::get` and
   `Client::post` are `async fn`. Outside `#[cfg(test)]`, `std::fs` in
   `crates/sc-runtime/src/lock.rs` runs while preparing the instance root and
   taking `daemon.lock` at start-up. `block_on` in
   `crates/sc-runtime/src/testing.rs` drives `run` on the fixture's own thread
   and is not called from a connection handler. `std::fs` and `std::net` in
   `crates/sc-transport/src` resolve an endpoint or bind, replace, or remove a
   socket and its bind lock. `crates/sc-transport/src/client.rs` classifies a
   failed connect from the I/O error kind and the error text. It does not
   call `std::fs` on `Client` request methods. `Client::get` and
   `Client::post` stay `async fn`.
### Related Documents

**Architecture Decisions:**

- ADR-RUN-0017 — 2026-10-31 clarification

---

# Requirement: Standard crates, used the documented way

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## NFR-RUN-0003: Standard crates, used the documented way

**Status:** Active  

### Requirement Statement

Definitions: CLI means command-line interface. MCP means Model Context Protocol. HTTP means Hypertext Transfer Protocol. SQL means Structured Query Language. API means application programming interface. ADR means architecture decision record. A building block is a crate used to provide one of the jobs listed in the Job/Crate table. Job code is Rust code in `crates/` or `template/` that serves HTTP routes, serves MCP tools, generates OpenAPI, runs SQL, parses CLI arguments, sends HTTP requests or renders the project template. A documented API is a public item shown in the listed crate's rustdoc documentation. A minimum tested version is the lowest version of a crate with which the project has tested; a version listed in the Job/Crate table is a minimum tested version and sets no upper bound (PRD-ENG-9).

No command path must be hand-written. A later consumer-side generator may produce that path against the delivered sc-runtime. Phase D builds no generator.

The shared command path is the one shared write path and one shared read path of PRD §3, to which a project supplies `Tcli`, `Tsql` and the transform. A command or operation registry is any structure in which a project registers operations for the framework to dispatch; the shared command path is not a registry under this record. A portable query layer is any abstraction that lets one SQL query run on more than one database backend; `Storage<T>` (PRD-STO-1, PRD-STO-2) carries no SQL and is not a portable query layer. An operation-defining declarative macro is a `macro_rules!` macro that defines operations, routes or tools. A wrapper type in this scope is an added facade whose only purpose is to hide a chosen library API. The `Storage<T>` engines required by PRD-STO-2, including a Postgres or MySQL engine holding its `sqlx` pool, supply storage behavior rather than such a facade. The project passes `sc_runtime::Daemon::builder()` an ordinary `utoipa_axum::OpenApiRouter` and an ordinary `rmcp` service.

Code generation for the command pattern, including an sc-compose or syn-based generator, a procedural or declarative macro, and a generated Rust HTTP client, is undecided (PRD §9): not built and not forbidden. Phase D builds no crate with `proc-macro = true` and no operation-defining declarative macro; this is Phase D scope, checked by success criteria 1 and 4, and forbids neither.

Phase D adds no command or operation registry, portable query layer or extra library-hiding wrapper in `crates/` or `template/`. The shared command path, `Storage<T>` and its SQLite, Postgres and MySQL implementations remain required. This describes what is built in Phase D; it does not forbid a later consumer option and creates no work to implement or test one.

The Job/Crate table lists one chosen crate for each job:

| Job | Crate |
|---|---|
| HTTP server and router | `axum` (minimum tested version 0.8) |
| MCP server | `rmcp` (tested floor owned by NFR-RUN-0007; `StreamableHttpService`, `#[tool]`, `#[tool_router]`) |
| OpenAPI | `utoipa` and `utoipa-axum` (`OpenApiRouter`, `routes!`) |
| SQL in the Postgres and MySQL engines (PRD-STO-2) | `sqlx` (minimum tested version 0.9) |
| SQL in the SQLite writer (PRD-STO-3) | `rusqlite` |
| CLI parsing | `clap` derive |
| HTTP client | `reqwest` (minimum tested version 0.13) |
| template rendering | `cargo-generate` |

**MUST statements:**

- Job code MUST call the documented API of the crate that the Job/Crate table lists for that code's job.

### Rationale

Each additional framework-specific registry, query layer or facade would be code this project has to maintain
and every generated project has to learn, where a standard crate already does
the job and is already documented. Standard designs are preferred over
hand-rolled pieces throughout. The decision that standard designs are
preferred, and that this item and [NFR-RUN-0004](requirements.md) enforce it,
is recorded in [ADR-RUN-0008](architecture.md).

### Success Criteria

1. Phase D scope check: at the end of Phase D,
   `grep -rn "proc-macro" --include=Cargo.toml crates template` prints
   nothing.
2. Phase D scope check: the `arch-qa` architecture review checks the public API of each library
   crate in `crates/` against [ADR-RUN-0001](architecture.md), which states
   that the framework builds the parts and the project wires them in ordinary
   Rust. It reports no extra registry, portable query layer, library-hiding facade
   or code generator built in Phase D. The shared command path and required
   typed storage engines are not rejected by this check. It imposes no
   implementation, testing or prohibition on later consumer options.
3. Inspection of the template: the example routes are registered with
   `utoipa_axum::routes!` on an `OpenApiRouter`, the example MCP tools use
   `#[tool]` and `#[tool_router]`, and the CLI uses `clap` derive.
4. Phase D scope check: at the end of Phase D,
   `grep -rn "macro_rules!" crates/*/src template` prints nothing, or every
   match is reviewed and found not to define operations, routes or tools.

## NFR-RUN-0004: Requirements must mean less code

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-20  
**Owner:** randlee  

### Requirement Statement

sc-runtime standardises build, test, reporting and runtime, which are the same in every project. This rule governs what may be added to the sc-runtime requirements, architecture and sprint plans.

Definitions:

- An addition is a new or changed requirement in the `docs/` requirements files, or a new crate boundary.
- A crate boundary is the split of code into a separate Rust crate.
- A consuming project is a project that uses sc-runtime.
- A reviewer is the person or agent who reviews a sprint plan or a requirement change.
- Admitting an addition means the reviewer approves the addition for merge.
- A standard tool is an existing tool or crate that already performs a job; the standard tools here and their jobs include template rendering (`cargo-generate`), agent-document rendering (`sc-compose`), the options UI (Wyvern), OpenAPI generation (`utoipa`), MCP serving (`rmcp`), and lint and `just` infrastructure (sc-lint).
- Custom code is code, including an adapter, written for a job that a standard tool already performs.
- A sprint deliverable is a sprint plan item that adds code.
- The application layer is the project-specific operations, service functions, routes, tools, CLI commands, schema and data flow that legitimately differ between projects.
- MCP means Model Context Protocol. ADR means architecture decision record.

**MUST statements:**

- An addition MUST be admitted only if the reviewer names code that consuming projects would have to write if the addition were omitted.

**MUST NOT statements:**

- An addition that requires custom code MUST NOT be admitted.
- An addition MUST NOT place a requirement on the application layer.
- A sprint deliverable MUST NOT contain custom code.

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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: Test code is the code in `#[cfg(test)]` modules and in `tests/` directories under `crates/` and `template/`. Non-test code is all other code under `crates/` and `template/`. A test's temporary directory is a directory that the test creates through a temporary-directory API or `DaemonFixture`. The default instance root is the instance root that sc-transport resolves when no path is supplied; a daemon the developer runs uses the default instance root. `just test` is `cargo test --workspace` plus the per-crate default-feature and `server` feature runs defined by [REQ-RUN-0004](requirements.md). A copy of the whole suite is one `just test` run from its own checkout. TCP means Transmission Control Protocol. A literal temporary-directory path is a string literal naming a system temporary directory, such as `/tmp`. A literal home-directory path is a string literal naming a user's home directory, such as `/Users/<name>`, `/home/<name>` or `C:\Users\<name>`. A file-write call is a call that creates, changes or deletes a file or directory, such as `std::fs::write` or `std::fs::create_dir`.

The instance root is the directory that holds a daemon's `daemon.lock` and `daemon.sock`. `<instance-root>` is the per-application, per-user directory resolved by sc-transport, or an explicitly supplied path; its default location is undecided ([REQ-TRN-0002](sc-transport/requirements.md)). Tests that start no daemon, such as the `sc-config` tests, have no instance root and need only the temporary directory.

The sc-runtime design states that tests are isolated and parallel and that
`DaemonFixture` gives each test its own instance root. The specific rules
here are decided in this document: no literal temporary-directory or
home-directory path in non-test code, no fixed TCP port, and two copies of
the whole suite passing at the same time on one machine.

This item governs tests that may run on a developer's host. A test that
starts a daemon as a separate process does not run there at all
([NFR-RUN-0010](requirements.md)).

**MUST statements:**

- Test code that writes to the filesystem MUST create a temporary directory for the same test.
- Test code that starts a daemon MUST use a temporary directory of the same test as the daemon's instance root.
- `just test` MUST exit with status 0.
- Every test MUST pass when two copies of the whole suite run at the same time on one machine.

**SHOULD statements:**

- Tests that start a daemon SHOULD use `sc_runtime::testing::DaemonFixture`, which starts a daemon on a fresh temporary instance root for one test.

**MUST NOT statements:**

- Non-test code MUST NOT contain a literal temporary-directory path.
- Non-test code MUST NOT contain a literal home-directory path.
- Test code MUST NOT contain a file-write call to a path outside a temporary directory of the same test.
- Test code MUST NOT use an instance root other than a temporary directory of the same test.
- Test code MUST NOT use the default instance root.
- The `just test` recipes MUST NOT set `--test-threads=1`.
- A test MUST NOT bind a fixed TCP port.

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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-23  
**Owner:** randlee  

### Requirement Statement

Definitions: The template is the `cargo-generate` template in the `template/` directory of this repository. The sc-lint consumer install is `template/sc-lint.toml` (`minimum_version` 0.8.6), `template/.sc-lint/bootstrap.py` and the `template/Justfile` recipes installed by `sc-lint init --just`, all owned by REQ-RUN-0307. Lint configuration is a lint rule, lint level or formatting option set outside the sc-lint consumer install, including a `clippy.toml`, `.clippy.toml`, `rustfmt.toml`, `.rustfmt.toml` or `deny.toml` file, a `[lints]` or `[workspace.lints]` table, and a custom lint rule. A maintainer boundary manifest is a crate-boundary rule or manifest copied from this repository's `boundaries/<crate>/` directories. The configuration-derived inventory is the application crate and allowed-edge list that `template/boundaries/render.py` generates once from the selected crates, as defined in ADR-RUN-0005 and ADR-RUN-0303. A `just` module is a `*.just` file or a `just` file pulled in by `import` or `mod`. A surface snapshot is a stored copy of `openapi.json`, of the MCP `tools/list` result or of the clap command model. A snapshot test is test code that compares a generated `openapi.json`, MCP `tools/list` result or clap command model against a surface snapshot.

The template commits the sc-lint consumer install and does not ship a placeholder `Justfile`. The public recipes are `setup`, `lint`, `test` and `upgrade`; the consumer-owned recipe `db-prepare` survives a later init. The configuration-derived inventory includes only selected crates and allowed edges, omits disabled store and MCP components, and becomes consumer-owned after setup. All other excluded infrastructure is owned by the sc-lint project.

This requirement restricts `template/` only. The sc-runtime repository lints itself with boundary manifests under `boundaries/<crate>/` enforced through `just lint` ([REQ-RUN-0005](requirements.md)).

**MUST NOT statements:**

- Files under `template/` MUST NOT contain lint configuration.
- Files under `template/` MUST NOT contain a maintainer boundary manifest.
- Files under `template/` MUST NOT contain a `just` module.
- Files under `template/` MUST NOT contain a surface snapshot.
- Files under `template/` MUST NOT contain a snapshot test.

### Rationale

If the template carried lint rules, every generated project would freeze the
rules of its birth date, and every change to the SC lint standard would need
a template change here. With sc-lint as the owner, generated projects pick up
rule changes from sc-lint and nothing in this repository changes.
### Success Criteria

1. `find template` lists none of: `clippy.toml`, `.clippy.toml`,
   `rustfmt.toml`, `.rustfmt.toml`, `deny.toml`, copied maintainer manifests,
   any `*.just` file.
2. `grep -rn "^\[lints\|^\[workspace.lints" template` prints nothing.
3. `template/sc-lint.toml` sets `minimum_version` to `0.8.6`, and
   `template/.sc-lint/bootstrap.py` exists. `template/` contains no
   placeholder `Justfile`. Recipe names are checked by REQ-RUN-0307.
4. `template/` contains no snapshot file of `openapi.json`, of MCP
   `tools/list` or of the clap command model, and no test that compares
   against one.

---

## NFR-RUN-0007: rmcp uses a tested floor with no upper bound

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: rmcp is the Model Context Protocol server crate. A covered manifest is a published crate manifest under `crates/` or a manifest under `template/`. A registry requirement is a dependency entry in a Cargo.toml that resolves the crate from crates.io. A tested floor is a `>=` version requirement naming the lowest dependency version the project has tested. An upper bound is a version requirement part that excludes a later version. The selected `rmcp` version is the `rmcp` version recorded in the sc-runtime `Cargo.lock`. The normal test suite is `just test` run in the sc-runtime repository. The template CI fixture matrix generates a project per answers fixture and runs `just lint` and `just test` in it. The release-tested dependency combination is the set of locked dependency versions with which the normal test suite and the template CI fixture matrix passed for the delivered release. A generated project is a project rendered from `template/`.

This requirement covers published crate manifests under `crates/` and manifests under `template/` rendered into generated projects. Phase A used a single minor-version pin; Phase D supersedes that policy. sc-runtime does not pin a generated project's dependency versions (PRD-ENG-9); consumers may pin versions in their own manifests and lockfiles.


**MUST statements:**


- Each `rmcp` registry requirement in a covered manifest MUST declare a tested floor of at least 3.9.

- The selected `rmcp` version MUST pass the normal test suite.

- The selected `rmcp` version MUST pass the template CI fixture matrix.

- The `.sc-runtime/tested-dependencies.json` file of each generated project MUST list the release-tested dependency combination.



**MUST NOT statements:**


- Each `rmcp` registry requirement in a covered manifest MUST NOT declare an upper bound.


### Rationale


Open tested floors let consumers take compatible dependency updates without an SDK-imposed upper bound. Normal tests and the template CI fixture matrix check the versions selected for the release.

### Success Criteria


1. Inspect published crate and template manifests that name `rmcp`: each registry requirement declares a tested floor of at least 3.9 and no upper bound.
2. The normal tests and template CI fixture matrix pass with the selected versions.
3. The tested-dependencies record of a generated project identifies the combination tested for the release.

---

## NFR-RUN-0008: Source file length limit (enforced by sc-lint)

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: `sc-lint` is the external lint package whose `line-counts` check is the source of truth for this limit; sc-runtime adopts the rule and does not define it. A source line is a line that is neither blank (whitespace only) nor made up only of a comment, where a comment is the comment syntax of the file's language (in Rust, `//` and `/* */`). A test file is a file under a `tests/` directory. A production Rust file is a `.rs` file that is not a test file. `just lint` is the `lint` recipe of the file named `Justfile` at the repository root. `sc-lint.toml` is the `sc-lint` configuration file at the repository root. The `line-counts` check is the `[[tool.sc-lint.lint]]` entry named `line-counts` in `sc-lint.toml`. Line-counting code is a command, script, recipe or program that reads a file and computes its number of source lines; a command that only invokes `sc-lint` is not line-counting code.

**MUST statements:**

- `just lint` run at the repository root MUST exit non-zero when a production Rust file under `crates/*/src/` has more than 1000 source lines.
- `just lint` run at the repository root MUST exit non-zero when a production Rust file under `template/` has more than 1000 source lines.
- When a production Rust file has more than 1000 source lines, the `just lint` output MUST contain the path of the production Rust file.
- When a production Rust file has more than 1000 source lines, the `just lint` output MUST contain the source-line count of the production Rust file.
- The `line-counts` check command in `sc-lint.toml` MUST start with `sc-lint lint line-counts`.
- The `[line_counts]` table in `sc-lint.toml` MUST set `max_production_lines = 1000`.

**MUST NOT statements:**

- Files committed to this repository MUST NOT contain line-counting code.

### Rationale

The limit is a standing rule across SC projects, defined and enforced by
`sc-lint` so that every repository measures it the same way. A file with more
than 1000 lines of actual source is a module boundary that was not drawn: it
is slow to review, and agents working on it spend their context on code
unrelated to the change. Counting only source lines means documentation
comments and spacing are never a reason to split a file or to strip comments.

Related: [REQ-RUN-0004](requirements.md) defines `just lint`; [NFR-RUN-0004](requirements.md) and [ADR-RUN-0008](architecture.md) forbid repository-local tooling that duplicates `sc-lint`.
### Success Criteria

1. `just lint` on an unmodified checkout exits 0, and its output shows that
   the `sc-lint` source-length check ran.
2. Adding a `.rs` file with 1001 source lines under `crates/sc-config/src/`
   makes `just lint` exit non-zero with a message that names that file.
3. Adding a `.rs` file with 900 source lines plus 200 blank and comment-only
   lines (1100 physical lines) under `crates/sc-config/src/` does not make
   `just lint` fail.
4. Inspection of the root `Justfile`, `scripts/` and `.github/workflows/`
   finds no line-counting command or script for this rule other than the
   call into `sc-lint`.
### Implementation Notes

**Key Considerations:**

- OPEN: whether the limit also applies to non-Rust source (for example
  `scripts/new_project.py` and the wizard's HTML and JavaScript) is not
  decided; it follows what the `sc-lint` rule covers.
- OPEN: whether a Rust file that holds only a `#[cfg(test)]` module inside
  `src/` counts as a test file is not decided; it follows what the `sc-lint`
  rule does.

---

# Requirement: Library errors are values

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## NFR-RUN-0009: Library errors are values

**Status:** Active  

### Requirement Statement

Definitions: CLI means command-line interface. MCP means Model Context Protocol. JSON means JavaScript Object Notation. API means application programming interface. ADR means architecture decision record. A covered crate is one of the nine Phase D crates listed in REQ-RUN-0002: sc-config, sc-transport, sc-command, sc-runtime, sc-cron, sc-cron-commands, sc-runtime-db-sqlite, sc-storage and sc-monty. A public function is a function or method that a covered crate exports in its public API. Public-reachable code is every non-test function body in a covered crate's `src` directory that a public function of that crate calls directly or indirectly. A crate-originated error is an error that the code of a covered crate itself detects. A crate result type is `Result<T, E>` where `E` is the one public typed error enum defined by the same covered crate. `[]` indexing is an `[]` index or range expression on a slice, `Vec`, string or map. A service function is a function that implements an operation and that the REST handler, the MCP tool and the CLI command all call. An OpError envelope is the `sc-command` JSON response envelope whose `ok` is `false` and whose `error` is an `OpError` object with a stable `code` and the keys `kind`, `code`, `message`, `details` and `suggested_action`. `CallToolResult` is the rmcp tool result type, whose content carries the response envelope. An opaque error type is an error type whose variants a caller cannot match, such as `anyhow::Error` or `Box<dyn std::error::Error>`.

This item is the single owner of the error rule and the panic rule for the public API of every covered crate. Each crate's own non-functional requirement references this item and adds only strictness specific to that crate. The decision is recorded in [ADR-RUN-0006](architecture.md); the 2026-10-25 amendment added `sc-runtime-db-sqlite` and `sc-cron`.

sc-monty returns typed compile, session, invoke, dump and load failures. Cron maps invalid cached bytes, version or hook names to recompilation from source, and rejects invalid source at schedule submission.

ADR-RUN-0017 permits the fixed Noop fallback literal invariant, validated against the selected tested types version, and the foreign `CanonicalLog::emit` `EventError` signature. ADR-RUN-0017 also defines application-only `try_log` unwind containment and its abort and hook limits. The fixed literal invariant is the only exception to the banned-construct statements below, and remains bounded as ADR-RUN-0017 writes it.

The sc-runtime design states the rule ("discriminated union; no panics") for `sc-config` only. Extending the rule to the other covered crates, the one-enum-per-crate rule, the ban on opaque error types and the list of banned constructs are decided in this document.

**Panic rule.** No public function may panic because of caller input or environment state. Environment state includes a missing or unreadable file, malformed JSON, an unset or malformed environment variable, an endpoint nobody is listening on, a lock already held by another process, and a failed bind. A comment saying why a use cannot fail is not an exception.

**Error rule.**

| Crate | Error enum |
|---|---|
| `sc-config` | `ConfigError` |
| `sc-transport` | `TransportError` |
| `sc-runtime` | `RuntimeError` |
| `sc-runtime-db-sqlite` | `SqliteWriterError` in the legacy public writer; `sc-storage::StorageError` for the Phase D `Storage<T>` contract |
| `sc-cron` | `CronError` |
| `sc-command` | undecided; see the scope notes below |

Recorded exception: a signature dictated by a third-party trait or macro keeps the types that contract requires, and the shared `Storage<T>` contract requires `StorageError` for storage operations. `sc_command::IntoMcp::into_mcp` returns `Result<CallToolResult, McpError>`, where `McpError` is `rmcp`'s error type, because an `rmcp` `#[tool]` function must return that type. The `axum::response::IntoResponse` implementation for `Envelope<T>` is infallible, because `into_response` returns a `Response`, not a `Result`. Neither is an error the crate itself originates.

**Errors returned by a service function.** The criteria for this paragraph are those of [REQ-RUN-0203](requirements.md) (REST, MCP and CLI `--json` return the same envelope); they are not repeated here.

**MUST statements:**

- Unless a third-party trait or macro signature or the shared `Storage<T>` contract fixes its error type, each public function that can return a crate-originated error MUST return its crate result type.
- Each covered crate MUST define at most one public error enum.
- When a service function returns an error, the stdout of the CLI command run with `--json` for the operation that the service function implements MUST be an OpError envelope.
- When a service function returns an error, the content of the `CallToolResult` returned by the MCP tool that called it MUST be an OpError envelope.

**MUST NOT statements:**

- Public-reachable code MUST NOT call `unwrap`.
- Public-reachable code MUST NOT call `expect`.
- Public-reachable code MUST NOT invoke `panic!`.
- Public-reachable code MUST NOT invoke `unreachable!`.
- Public-reachable code MUST NOT invoke `todo!`.
- Public-reachable code MUST NOT invoke `unimplemented!`.
- Public-reachable code MUST NOT use `[]` indexing.
- A public function signature MUST NOT name an opaque error type.

### Rationale

These crates run inside long-lived daemons and inside CLIs driven by agents.
A panic in a daemon is an outage. A panic in a CLI is a failure no agent can
parse or recover from, where a typed error with a code and a suggested action
lets it branch. One owner for the rule keeps the nine Phase D crates from drifting
into different lists of banned constructs.

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
4. Inspect public `sc-command` signatures: foreign error types are permitted
   only where required by the named third-party trait or macro contracts,
   including `IntoMcp::into_mcp` returning `McpError` and `CanonicalLog::emit`
   returning `EventError`, or by the shared `Storage<T>` contract. Other
   fallible public operations return the crate-owned typed error required
   by this record. `IntoResponse` remains infallible.
5. In all nine Phase D crates no public signature names `anyhow::Error` or
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

8. Inspect public fallible storage operations in `sc-storage` and the SQLite `Storage<T>` implementation: they return the typed `StorageError` required by the shared contract. Inspect the public APIs of all nine Phase D crates for typed errors and the no-panic rule.

### Implementation Notes

**Key Considerations:**

- Implementation scope: whether `sc-command` needs an error enum of its own is not decided.
  Its one fallible public function known so far is `into_mcp`, which is covered
  by the exception above.
- Implementation scope: whether a response the framework produces before any service
  function runs is an envelope is not decided
  ([REQ-RUN-0203](requirements.md) owns the question). Such responses are: an
  `axum` response for an unknown route, an `axum` rejection of a body that
  fails to deserialise, and an `rmcp` protocol error, which reaches the MCP
  client as `Err(McpError)`. Nothing in this item asserts that they are
  `OpError` values.

### Related Documents

**Architecture Decisions:**

- ADR-RUN-0017 — 2026-10-31 bounded amendment

# Requirement: Daemon-process tests run on an isolated machine; no test daemon

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-20  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## NFR-RUN-0010: Daemon-process tests run on an isolated machine; no test daemon

**Status:** Active  

### Requirement Statement

This rule applies to this repository and, through
[REQ-RUN-0311](requirements.md), to every generated project.

Definitions:

- `<instance-root>` is the per-application, per-user directory resolved by sc-transport, or an explicitly supplied path; its default location is undecided ([REQ-TRN-0002](sc-transport/requirements.md)).
- A developer host is a developer's own machine, where the application's own daemon may be running.
- An isolated machine is a virtual machine (for example one run by colima) or an ephemeral CI runner; in v0.1 the isolated machine for this repository's own tests is the CI runner.
- An isolated machine is available to `just test` when `just test` can start a virtual machine to run tests in.
- `DaemonFixture` is `sc_runtime::testing::DaemonFixture`, which runs the daemon in-process through the same `Daemon::builder()` path as production ([REQ-RT-0006](sc-runtime/requirements.md)).
- A fixture resource is the `daemon.lock` file, the endpoint or a database file that a `DaemonFixture` daemon opens.
- A host-safe test is a test that runs the daemon only in-process through `DaemonFixture` on a fresh temporary `<instance-root>`, opens every fixture resource under that `<instance-root>`, and starts no daemon or `cli` binary as a separate process.
- A daemon-process test is any test that is not host-safe, including a test that starts the daemon binary or the generated `cli` binary as a separate operating-system process, a test that uses the default `<instance-root>`, and a `DaemonFixture` test whose endpoint is not under its `<instance-root>` (the Windows TCP default, OPEN in [REQ-RT-0006](sc-runtime/requirements.md)).
- The template's example test is the generated project's test that starts a daemon with `DaemonFixture` and exercises the example ([REQ-RUN-0302](requirements.md)).
- The signal-test job is the dedicated `ubuntu-latest` GitHub-hosted job that runs the Phase B generated SIGINT/SIGTERM tests.
- The isolation variables are the three environment variable values `SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST=1`, `GITHUB_ACTIONS=true` and `RUNNER_ENVIRONMENT=github-hosted`.
- The isolation check passes when all three isolation variables are set to those values, and fails otherwise.
- A process-spawning call is a call that starts an operating-system process.
- An isolation guard is a statement that returns from the test when the isolation check fails.
- A daemon-starting call is a process-spawning call that starts a daemon, or a call that starts a `DaemonFixture` daemon.
- A host check is a statement that returns from the test when the test is not running on an isolated machine; an isolation guard is a host check.
- A test runner is a GitHub Actions workflow job or a `just` recipe.
- Production daemon code is the source compiled into the daemon binary, including `crates/sc-runtime/src`.
- A test variant of the daemon is a crate, binary target, cargo feature, `cfg` flag or build configuration whose purpose is the daemon for tests: a daemon-like executable, a stub daemon or a daemon with parts switched off.
- A test-only switch is a code path in production daemon code that makes the daemon behave differently under test: a skipped lock, a different start-up order, stubbed stores or stubbed transport.
- The `testing::DaemonFixture` module of `sc-runtime`, its crate-private shutdown trigger (which starts the same shutdown sequence a signal starts, [REQ-RT-0006](sc-runtime/requirements.md)) and a cargo feature that only gates that module (OPEN in [NFR-RT-0003](sc-runtime/requirements.md)) are neither a test variant of the daemon nor a test-only switch.
- Weakening the `daemon.lock` singleton means any change that lets a second daemon start while another daemon holds the instance lock of the same `<instance-root>` ([REQ-RT-0002](sc-runtime/requirements.md)).
- CLI means command-line interface. TCP means Transmission Control Protocol.

Scope:

- The rule governs where a daemon-process test executes, not which command starts it. `just test` MAY run daemon-process tests inside an isolated machine that `just test` starts. What `just test` covers belongs to the standard SC `just` system, whose source of truth is the sc-lint project.
- Daemon-process tests exist in this repository's crate tests and in tests of the generated `cli` and `daemon` binaries that live in `template/` ([REQ-RUN-0310](requirements.md) criterion 4).
- Phase B selects the generated real-daemon SIGINT/SIGTERM tests with the signal-test job. Selection of other generated daemon-process tests remains OPEN.
- The host-safe classification of `DaemonFixture` tests is an interpretation of the owner's decision that the owner has not yet confirmed.
- Phase A proves CLI behavior primarily through host-safe tests and runs the minimal real-binary smoke test on an isolated CI runner. This rule does not prescribe helper functions, attributes or a generated unit-test layout.
- Building, provisioning or scripting a virtual machine is not part of v0.1 of this repository.

**MUST statements:**

- A test MUST start a daemon only through the production `Daemon::builder()` path.
- The definition of every test runner that runs a daemon-process test MUST target an isolated machine.
- When no isolated machine is available to `just test`, `just test` MUST print a line stating that the daemon-process tests were not run.
- The template's example test MUST place its database files under the temporary `<instance-root>` of its `DaemonFixture`.
- The signal-test job MUST be the only test runner that sets `SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST=1`.

**MUST NOT statements:**

- The workspace MUST NOT contain a test variant of the daemon.
- Production daemon code MUST NOT contain a test-only switch.
- A daemon-process test that runs on a developer host MUST NOT start a daemon.
- The source of a daemon-process test MUST NOT contain a daemon-starting call that is not preceded by a host check.
- The source of a daemon-process test MUST NOT contain a process-spawning call that is not preceded by its isolation guard.
- The `just test` recipe MUST NOT contain a command that runs a daemon-process test on the host.
- Production daemon code MUST NOT weaken the `daemon.lock` singleton.

### Rationale

Integration tests want to run the daemon. On a development machine the
application's real daemon is usually running too, so a test daemon either
collides with it or has to be made different from it. Both paths have been
taken in another SC project, with a known result: thousands of daemons left
running by tests, and a singleton gate that had to be strengthened to stop
them. The tempting fix is a test daemon, an executable that is mostly the
daemon but safe to run in tests. It drifts from the real daemon, it doubles
the code to maintain, and the tests stop proving anything about production.
A virtual machine removes the collision instead of working around it: the
real daemon, the real singleton and the real service manager can all be
exercised with nothing on the developer's machine at stake. Naming the
virtual machine as the escape path keeps the next person from designing the
test daemon.

### Success Criteria

1. Inspection of the workspace finds no crate, binary target, cargo feature
   or `cfg` flag whose purpose is a test variant of the daemon, and no branch
   in `crates/sc-runtime/src` that changes the daemon's behaviour under test.
   The `testing` module, its crate-private shutdown trigger and a feature
   that only gates that module are not such branches (Definitions).
2. Every test, in this repository and in a generated project, that starts a
   daemon as a separate process
   (the child-process tests of [REQ-RT-0005](sc-runtime/requirements.md),
   [REQ-RT-0008](sc-runtime/requirements.md) and
   [REQ-RUN-0310](requirements.md), plus any future tests introduced with
   deferred [REQ-RUN-0206](requirements.md)) is
   a daemon-process test; none of them executes on a
   developer host, whichever command started the run.
3. The Phase B generated SIGINT/SIGTERM tests exit without starting a daemon
   unless all three fixed values are present:
   `SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST=1`, `GITHUB_ACTIONS=true`, and
   `RUNNER_ENVIRONMENT=github-hosted`. Other daemon-process tests likewise
   refuse a non-isolated host, with their broader selection mechanism still
   OPEN (Scope).
4. This repository's crate CI workflow runs this repository's daemon-process
   tests. A dedicated `ubuntu-latest` GitHub-hosted job runs the Phase
   B generated SIGINT/SIGTERM tests and sets their dedicated marker. Once the
   broader selection OPEN is decided, the fixture-matrix workflow also runs
   the remaining generated-project daemon-process tests for every fixture.
5. After the full test suite has run on a developer's host, no daemon
   process started by a test is still running. The same holds after a run in
   which a fixture was made to fail.
6. Inspection of this repository's tests and of `template/` finds no test
   that spawns the generated `cli` binary other than daemon-process tests on an
   isolated machine.

The production tool-crate exclusion is enforced across the five release crates
by the `production-tool-boundary` lint companion in `sc-lint.toml`: package
identity checks cover normal/build, renamed and target-specific dependencies,
production Rust imports, and Cargo normal/build transitive trees for supported
production features. Dev-only fixtures remain permitted. The Monty registry
and optional adapters are documented in [the generated example](sc-cron/monty-generated-example.md).

## REQ-RUN-0703: Skill checks use the declared sc-compose minimum

**Status:** Active  
**Version:** 1.0.0  

### Requirement Statement

Definitions: Repository skills are the checked-in .claude, .codex and .cursor skill instructions and scripts. The declared minimum is the sc-compose version required by the invoking skill. A skill version check is the function in a repository skill script that compares a given sc-compose version with the declared minimum; it accepts a version when it returns success for that version. A malformed version is a version string that is not two or three dot-separated decimal numbers.

**MUST statements:**

- A skill version check MUST accept a version at or above the declared minimum.
- A skill version check MUST reject a version older than the declared minimum.
- A skill version check MUST reject a malformed version.
### Rationale

The owner requires skills to tolerate sc-compose >= a floor threshold (Phase D anchor ANC-015 in docs/plans/phase-d/anchors.md, not the Phase B or Phase C anchor of that number). The PRD does not address skill version checks; a lower bound is consistent with PRD-ENG-9, which states minimum versions rather than pins. Reproducible CI binary selection remains independent.
### Success Criteria

**Acceptance Criteria:**

- [ ] Controlled version inputs at the declared minimum and a newer version are accepted by each changed skill check.
- [ ] Controlled older and malformed version inputs are rejected by each changed skill check.
### Product Applicability

**Applies To:**

- Repository skill instructions and scripts that check sc-compose versions.

**Does Not Apply To:**

- Selection of an immutable CI binary or Cargo registry dependency requirements.
### Test Strategy

**Test Types:**

- Deterministic unit tests and real command smoke tests
