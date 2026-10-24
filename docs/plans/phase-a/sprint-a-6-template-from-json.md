---
id: a-6
phase: a
status: planned
closure_type: integration
target_boundary: template-from-json
vertical_rationale: "The generator and editable example do not exist yet; one vertical sprint creates the single JSON-to-project artifact and proves it without introducing a reusable application framework. Splitting the driver from the template would only serialize the same artifact."
track: generation
wave: 3
branch: sprint/a-6-template-from-json
worktree: sprint/a-6-template-from-json
recommended_agent: solar
recommended_model: sol
must_follow: [a-5]
parallel_safe: []
requirements:
  - REQ-RUN-0003
  - REQ-RUN-0004
  - REQ-RUN-0201
  - REQ-RUN-0202
  - REQ-RUN-0203
  - REQ-RUN-0301
  - REQ-RUN-0302
  - REQ-RUN-0303
  - REQ-RUN-0304
  - REQ-RUN-0305
  - REQ-RUN-0306
  - REQ-RUN-0307
  - REQ-RUN-0308
  - REQ-RUN-0309
  - REQ-RUN-0310
  - REQ-RUN-0401
  - REQ-RUN-0402
  - REQ-RUN-0403
  - REQ-RUN-0501
  - REQ-RUN-0502
  - REQ-RUN-0503
  - REQ-RUN-0701
  - NFR-RUN-0001
  - NFR-RUN-0002
  - NFR-RUN-0003
  - NFR-RUN-0004
  - NFR-RUN-0005
  - NFR-RUN-0006
  - NFR-RUN-0007
  - NFR-RUN-0008
  - NFR-RUN-0009
  - NFR-RUN-0010
adrs:
  - ADR-RUN-0001
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0004
  - ADR-RUN-0005
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0008
  - ADR-RUN-0009
  - ADR-RUN-0201
  - ADR-RUN-0202
  - ADR-RUN-0301
  - ADR-RUN-0302
  - ADR-RUN-0303
  - ADR-RUN-0401
  - ADR-RUN-0402
owned_paths:
  - "template/**"
  - "wizard/answers.schema.json"
  - "wizard/fixtures/**"
  - "tests/fixtures/refused/**"
  - "scripts/new_project.py"
  - "Justfile"
  - "docs/requirements.md"
  - "tests/unit/generation/**"
  - "tests/integration/generation/**"
  - ".github/workflows/template.yml"
  - "docs/generation.md"
  - "docs/validation/phase-a-generation.md"
---

# Sprint a-6 — template from JSON

## Outcome and closure

Generate one minimal, editable SQLite application from a validated answers JSON
file. The generated project proves the four libraries are useful together; it
is not a framework whose internal organization is imposed on future projects.
It proves the [`framework-charter`](../../framework-charter.md) consumer model:
published crates plus an editable generated tree for many independent teams.

## Decision prerequisites

Reconcile the deferred auto-start and prescribed CLI-test requirements before
dispatch. Settle only the answers keys/defaults, library version requirements,
sc-compose invocation, destination/name mapping, and minimal example operation
needed for this milestone.

Install the sc-lint 0.8.0 consumer contract into the template with
`sc-lint init --just`. That command is the installer REQ-RUN-0307 was waiting
for; do not keep a hand-written lint stand-in. Commit `template/sc-lint.toml`
with `minimum_version = "0.8.0"`, `template/.sc-lint/bootstrap.py`, and a
generated `Justfile` whose public recipes are `setup`, `lint`, `test`, and
`upgrade`. Keep consumer-owned `db-prepare` beside those four. Adding that
recipe edits the generated `Justfile`; a later `sc-lint init --just` must be
reviewed so `db-prepare` stays. The template still carries no lint policy of
its own: no `boundaries/`, no clippy/rustfmt/deny config, and no `just` import
(NFR-RUN-0006). The plain cargo-generate job still only builds. Fixture CI
that runs `just setup` needs Python 3.11, which the bootstrap requires. Do
not rewrite the a-5 source-maintainer root `Justfile` into the consumer
file, and do not replace or contradict that file's sc-lint init header.
The only root `Justfile` edit here is `just new` and generation-test
discovery, in the section `sc-lint init --just` does not regenerate.
This sprint amends only the REQ-RUN-0503 and REQ-RUN-0701 sections of
`docs/requirements.md`, so the Active green sequence is `just setup` before
`just lint`. No other section of that file changes.

Pin generated `crates/daemon` to the `sc-observability` version a-1 recorded
(the latest release at spike time; 1.5.0 as of this plan) after a-1 has
amended REQ-RUN-0309 and ADR-RUN-0004 to that pair. `main` loads config,
initialises `sc-observability` with plain values from that config, then calls
`Daemon::builder`. At that application boundary, convert with `as_str()` into
`sc_config::AppName`, `sc_transport::AppName`, and `sc_runtime::AppName`.
Leave `sc-observability-otlp` out of the template. OpenTelemetry export stays
optional for a later project change.

This sprint is the sole owner of the NFR-RUN-0001 feature-level close.
Fixture CI runs `cargo tree -p cli -e normal`, and the named server-only
crates are absent. `cargo build -p cli` is the normative selection. A
workspace-wide build is the documented exemption. This sprint does not split
crates. Workspace-unification was closed in a-1; a-6 owns only the generated-project `cargo tree -p cli` check.
Unknown route, Axum body rejection, and rmcp `Err(McpError)` stay deferred
for v0.1. Do not extend ADR-CMD-0001 when the example widget operation is
added.

## Dependencies and parallel safety

- `must_follow a-5`: consumes the real published-surface candidates and the
  rewritten spike's ordinary composition pattern.

No separate driver, template-tree, or phase-wide integration sprint is
created: those slices would serialize one small greenfield artifact and defer
its only meaningful proof.

## Deliverables

1. **REQ-RUN-0401 / REQ-RUN-0402 / REQ-RUN-0403** — One versioned schema and
   small valid/refused fixture set; invalid or unsupported input fails before
   destination creation.
2. **REQ-RUN-0501 / REQ-RUN-0502** — A small `--var-file` driver sequences
   validation, cargo-generate, sc-compose, and the generated project's
   `just setup`, `just lint`, and `just test`. It does not include wizard
   logic yet.
   In development/PR mode the driver writes a generated-root
   `[patch.crates-io]` table pointing at the four crates in the checkout before
   its first Cargo invocation; release mode writes no patch and requires
   registry sources. The template always retains version requirements.
3. **REQ-RUN-0301 / REQ-RUN-0302** — An editable five-crate example with one
   create/get operation and thin REST, optional MCP, and CLI transport adapters.
4. **REQ-RUN-0303 through REQ-RUN-0310** — Minimal conditional generation,
   SQLite store and config wiring, agent-document rendering, and standard
   generated recipes needed for the example to compile and run. Daemon
   observability init is deliverable 9. Concrete example choices are
   documented as examples, not extension rules.
5. **REQ-RUN-0201 / REQ-RUN-0203** — One host-safe in-process smoke proves the
   example service through REST, optional MCP, and the CLI's transport path.
6. **REQ-RUN-0701** — Fixture CI discovers supported answers files and verifies
   no-edit generation, `just setup && just lint`, test, and build. The
   rendered project workflow runs lint and test only through `just`, after
   `just setup`.
7. **REQ-RUN-0306** — Plain cargo-generate remains usable for the template tree
   with documented limitations around separately rendered Jinja documents.
8. **REQ-RUN-0307 / NFR-RUN-0006** — The generated project is an
   sc-lint 0.8.0 consumer. `sc-lint init --just` supplies `sc-lint.toml`,
   `.sc-lint/bootstrap.py`, and public recipes `setup`, `lint`, `test`, and
   `upgrade`. `db-prepare` remains on that `Justfile` and must survive a later
   `sc-lint init --just`. Fixture CI, in each generated project,
   runs `just setup && just lint` before `just test` and build. `just setup`
   installs the consumer tool and is not a source edit. Boundary manifests
   stay in this repository.
9. **REQ-RUN-0309 / ADR-RUN-0004** — Generated `crates/daemon` depends on
   `sc-observability` at the exact version a-1 recorded. Rendered `main`
   loads config with `sc-config`, initialises `sc-observability`, then calls
   `Daemon::builder`. The template does not depend on
   `sc-observability-otlp`. The four library crates still do not depend on
   the logging runtime.
10. **REQ-RUN-0503 / REQ-RUN-0701** — Amend only those two sections of
    `docs/requirements.md`. The green sequence becomes `just new`, then
    `just setup`, then `just lint`, then `just test`. `just setup` installs
    the sc-lint 0.8.0 consumer and is not a source edit: no generated source
    file is added, changed, or deleted between generation and the end of
    `just test`. REQ-RUN-0503 success criterion 2, and REQ-RUN-0701's command
    list and success criterion 1, name `just setup` immediately before
    `just lint`. Do not amend any other requirement in that file.
11. **NFR-RUN-0008** — After `template/**/*.rs` exists, root `just lint`'s
    sc-lint line-counts check covers those files. a-5 closed only
    `crates/*/src`. This sprint does not add a line-counting script. Files
    under a `tests/` directory stay exempt.

## Acceptance criteria

1. `req:REQ-RUN-0501` — `just new <dest> --var-file <fixture>` generates into
   an absent destination and stops at the first named failing standard tool.
2. `req:REQ-RUN-0201` — the generated example creates and gets one widget via
   the shared service path from REST, optional MCP, and a real transport client.
   This proves connectivity, not permanent application architecture.
3. `req:REQ-RUN-0202` — report-only `--json` against an unreachable endpoint
   returns a nonzero status and the exact `DAEMON.NOT_RUNNING` envelope without
   creating daemon, lock, socket, or database artifacts. The real CLI-binary
   proof runs only on an isolated CI runner; a host-safe proof exercises the
   same reporting path without prescribing helper functions.
4. `req:REQ-RUN-0203` — success and operation-failure envelopes are equal as
   JSON across REST, MCP and CLI `--json`; unknown `data` and `error.details`
   content and the received version survive CLI output unchanged. Unknown
   route, Axum body rejection, and rmcp `Err(McpError)` are deferred for v0.1
   in the phase deferred-scope table. This criterion does not close that OPEN.
5. `req:REQ-RUN-0301` / `req:REQ-RUN-0302` — generation produces the minimal
   editable api-types/store-sqlite/service/daemon/cli example, and the create/get
   operation reaches one service path through all enabled surfaces.
6. `req:REQ-RUN-0310` — CLI and daemon accept the same endpoint text and use
   only `sc-transport` resolution; an override points both at the same fixture
   endpoint and no template code computes its own default.
7. `req:REQ-RUN-0303` — MCP on/off and project-licence fixtures generate the
   expected files; unsupported database choices fail before generation.
8. `req:REQ-RUN-0306` — plain cargo-generate creates the default project
   without Python or Wyvern. Before publication, its separate CI smoke applies
   the same checkout patch immediately after generation and before the first
   Cargo command; after publication it uses registry mode. Jinja document
   handling is documented honestly.
9. `req:REQ-RUN-0701` — fixture CI generates, then runs `just setup && just lint`,
   then tests and builds, without modifying the result between generation and
   validation.
   PR-mode lockfiles resolve all four libraries to the checkout paths; release
   lockfiles resolve all four to the crates.io registry.
10. `req:NFR-RUN-0001` — This sprint is the feature-level close. Fixture CI
    runs `cargo tree -p cli -e normal` on the generated project, and the named
    server-only crates are absent. `cargo build -p cli` is the normative
    selection. A workspace-wide build is the documented exemption. This
    criterion does not split crates and does not depend on a-5 closing
    ADR-RUN-0003.
11. `ADR-RUN-0303` — inspection confirms that reusable runtime behavior remains
   in the published crates while routes, schema, queries, and application
   composition remain editable template code.
12. `req:REQ-RUN-0307` / `req:NFR-RUN-0006` — a project generated from
    `wizard/fixtures/sqlite-mcp.json` lists `setup`, `lint`, `test`,
    `upgrade`, and `db-prepare`. `just setup && just lint` exits 0 with no
    edits. `template/` contains `sc-lint.toml` and `.sc-lint/bootstrap.py`
    from sc-lint 0.8.0, and it contains no `boundaries/` directory, no
    clippy/rustfmt/deny config, and no `mod` or `import` in the `Justfile`.
    `minimum_version` is `0.8.0`.
13. `req:REQ-RUN-0309` / `ADR-RUN-0004` — rendered `crates/daemon/Cargo.toml`
    requires the a-1 `sc-observability` version and has no
    `sc-observability-otlp` requirement. Rendered `main.rs` orders config
    load, then `sc-observability` init, then `Daemon::builder`.
    `cargo tree` of `sc-config`, `sc-transport`, `sc-command`, and
    `sc-runtime` in the generated project has no `sc-observability ` or
    `sc-observability-otlp ` line (crate name, then a space).
14. `req:REQ-RUN-0401` — `wizard/answers.schema.json` is the only options
    definition. A unit test loads it as a valid schema under its own
    `$schema`, asserts top-level type `object`, asserts the `db` enum is
    exactly `sqlite`, `postgres`, and `both`, asserts `mcp` is boolean,
    asserts a project-name string property and a licence string property
    whose default is `MIT`, asserts `properties` has exactly those four keys
    (plus the schema-version key only if version is a property), and asserts
    the schema version is present and non-empty. `scripts/new_project.py`
    holds no list of option names or defaults. This sprint settles the
    property keys, defaults, `$schema` draft, and version carrier before
    generation, and records the `postgres` / `both` rejection so a driver
    check, if chosen, is the single named exception in that test.
15. `req:REQ-RUN-0402` — `just test` runs a key-set test that reads schema
    property keys and `template/cargo-generate.toml` placeholder keys from
    the working tree and fails when either set has a name the other lacks.
    The test source has no literal list of option names. The project-name
    key is either exempted by one named constant or declared as a
    placeholder. The test matches that decision and exempts nothing else.
16. `req:REQ-RUN-0403` — `wizard/fixtures/sqlite-mcp.json` has `db` `sqlite`
    and `mcp` true. Another fixture has `db` `sqlite` and `mcp` false. One
    fixture states a licence other than `MIT`, and every fixture states its
    licence. Unit tests accept every `wizard/fixtures/*.json` file and reject
    schema-invalid files for a missing required key, a wrong JSON type, a
    `db` value outside the enum, and an unknown key. A driver test with
    `--var-file` on a schema-invalid file exits non-zero, does not call
    `cargo`, and does not create the destination. Refused files, including
    one `postgres` answers file and one `both` answers file, live outside
    `wizard/fixtures/*.json`.
17. `req:REQ-RUN-0304` — a test counts Liquid conditional lines in
    `template/**/*.rs` and fails when the total is ten or more. Every `{%`
    match in those files encloses only `Stores` fields, `mod mcp;`, or the
    `.mcp(...)` builder line. A file present in one variant and absent in
    another is named in a conditional `ignore` list in
    `template/cargo-generate.toml`. No conditional tag varies a function,
    handler, query, or test body.
18. `req:REQ-RUN-0305` — `template/cargo-generate.toml` lists `AGENTS.md.j2`
    and `CLAUDE.md.j2` under `exclude`. After
    `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json`, `<dest>`
    contains non-empty `AGENTS.md` and `CLAUDE.md`, contains no `.j2` file,
    and those two documents contain no `{{` or `{%`. A stub `sc-compose`
    that exits 1 makes the driver exit non-zero and name `sc-compose`.
19. `req:REQ-RUN-0308` — generated `store-sqlite` depends on `sqlx` 0.9 with
    the `sqlite` feature and is the only generated crate with a direct `sqlx`
    dependency, and only `service` depends on `store-sqlite`. With
    `SQLITE_DATABASE_URL` unset, `SQLX_OFFLINE=true cargo build -p store-sqlite`
    exits 0. A concurrency test inserts through the write pool without a busy
    or locked error. Opening a store on an empty database runs migrations and
    `PRAGMA journal_mode` returns `wal`. `sqlx.toml` names
    `SQLITE_DATABASE_URL`. Checked-in `template/crates/store-sqlite/.sqlx`
    renders into the project. The write-pool maximum is 1 and every mutating
    query uses it. The crate has no write actor or queue. Read-pool size and
    the `open` config type are recorded as example choices in the sprint
    validation note.
20. `req:REQ-RUN-0502` — `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json`
    with stdin closed exits 0 on a runner that has no `wyvern`. Driver tests
    show `--var-file` does not call a Wyvern stub, prefers `SC_COMPOSE` over
    a `PATH` `sc-compose`, and, when both are absent, exits non-zero before
    `cargo generate` with stderr that names `sc-compose`, `SC_COMPOSE`, and
    `PATH`. `scripts/new_project.py` contains no absolute `sc-compose` path
    and no Wyvern discovery.
21. `req:NFR-RUN-0008` — this criterion closes `template/**/*.rs` only. a-5
    closed `crates/*/src`. After the template tree exists, root `just lint`
    runs the sc-lint source-length check over `template/**/*.rs` and exits 0
    on the unmodified tree. A non-test `.rs` file under `template/` with 1001
    source lines makes that check fail and name the file. A file under a
    `tests/` directory is exempt. The repository has no line-counting script
    of its own.
22. `req:REQ-RUN-0503` / `req:REQ-RUN-0701` — the only edit this sprint makes
    to `docs/requirements.md` amends these two requirements. The Active green
    sequence is `just new`, then `just setup`, then `just lint`, then
    `just test`. `just setup` installs the sc-lint 0.8.0 consumer and is not
    a source edit. REQ-RUN-0503 success criterion 2, and REQ-RUN-0701's
    command list and success criterion 1, name `just setup` immediately
    before `just lint`. No other requirement section changes. Criterion 9 is
    the fixture run of that amended sequence.

## This sprint does not close

It does not implement CLI auto-start, service-manager definitions, prescribed
CLI helper/test shapes, attribute macros, attribute-generated tests, process
daemon tests, Postgres, or the wizard. It does not claim the example's internal
file/crate organization is mandatory for generated applications. It does not
replace the source-maintainer sc-lint install owned by a-5, and it does not
hand a prototype repository to the sc-lint team. It does not map unknown
routes, Axum body rejections, or rmcp `Err(McpError)` into the envelope. It
does not add widget operation codes to ADR-CMD-0001. It does not amend
`docs/requirements.md` outside REQ-RUN-0503 and REQ-RUN-0701. It does not
re-close the `crates/*/src` half of NFR-RUN-0008.

## Paths to delete

Generated test destinations and temporary values files are removed by their
tests. No repository source path is deleted.

## Required validation

Run root lint/test, all fixture generations, and in each generated project
`just setup && just lint` followed by `just test` and the workspace build.
Also run the plain cargo-generate smoke, schema/refused-input tests,
`cargo tree -p cli -e normal` in a generated project, static checks for
unrendered Jinja/Liquid, the root line-counts check over `template/**/*.rs`,
and a diff check that `docs/requirements.md` changes only REQ-RUN-0503 and
REQ-RUN-0701. No daemon child process runs on
the host. Record the supported variants, the sc-lint 0.8.0 pin, the
`sc-observability` pin copied from a-1, and results in
`docs/validation/phase-a-generation.md`.
