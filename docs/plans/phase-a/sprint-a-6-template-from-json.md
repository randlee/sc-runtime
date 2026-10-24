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
  - "tests/fixtures/app-name-vectors.json"
  - "scripts/new_project.py"
  - "Justfile"
  - "docs/requirements.md"
  - "tests/unit/generation/**"
  - "tests/integration/generation/**"
  - ".github/workflows/template.yml"
  - "docs/generation.md"
---

# Sprint a-6 — template from JSON

## Outcome and closure

Generate one minimal, editable SQLite application from a validated answers JSON
file. The generated project proves the four libraries are useful together; it
is not a framework whose internal organization is imposed on future projects.
It proves the [`framework-charter`](../../framework-charter.md) consumer model:
published crates plus an editable generated tree for many independent teams.

## Decision prerequisites

Auto-start and the prescribed CLI-test shape stay deferred in the phase
deferred-scope table. This sprint does not reconcile them again. It does ship
`daemon start` as an explicit subcommand. Settle only the answers
keys/defaults, library version requirements, sc-compose invocation,
destination/name mapping, and the minimal example operation needed for this
milestone.

Install the sc-lint 0.8.0 consumer contract into the template with
`sc-lint init --just`. That command is the installer REQ-RUN-0307 was waiting
for; do not keep a hand-written lint stand-in. Commit `template/sc-lint.toml`
with `minimum_version = "0.8.0"`, `template/.sc-lint/bootstrap.py`, and a
generated `Justfile` whose public recipes are `setup`, `lint`, `test`, and
`upgrade`. Keep consumer-owned `db-prepare` and `build-cli` beside those
four. Adding those recipes edits the generated `Justfile`; a later
`sc-lint init --just` must be reviewed so both stay. The template still carries no lint policy of
its own: no `boundaries/`, no clippy/rustfmt/deny config, and no `just` import
(NFR-RUN-0006). The plain cargo-generate job still only builds. Fixture CI
that runs `just setup` needs Python 3.11, which the bootstrap requires. Do
not rewrite the a-5 source-maintainer root `Justfile` into the consumer
file, and do not replace or contradict that file's sc-lint init header.
The only root `Justfile` edit here is `just new` and generation-test
discovery, in the section `sc-lint init --just` does not regenerate.
This sprint amends only the REQ-RUN-0202, REQ-RUN-0503, and REQ-RUN-0701
sections of `docs/requirements.md`. The Active green sequence is `just new`,
then `just setup`, then `just lint`, then `just test`, then `just build-cli`.
No other section of that file changes.

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
crates are absent. The generated `Justfile` gains consumer-owned `just build-cli`,
which runs `cargo build -p cli --release`. Fixture CI runs that recipe.
A workspace-wide build is the documented exemption and is not the shipped
agent binary. This sprint does not split crates. Workspace-unification was
closed in a-1.
Unknown route, Axum body rejection, and rmcp `Err(McpError)` stay deferred
for v0.1. Do not extend ADR-CMD-0001 when the example widget operation is
added. Document the widget operation error inventory on the generated service.

## Dependencies and parallel safety

- `must_follow a-5`: consumes the real published-surface candidates and the
  rewritten spike's ordinary composition pattern.

No separate driver, template-tree, or phase-wide integration sprint is
created: those slices would serialize one small greenfield artifact and defer
its only meaningful proof.

## Deliverables

1. **REQ-RUN-0401 / REQ-RUN-0402 / REQ-RUN-0403** — One versioned schema and
   small valid/refused fixture set; invalid or unsupported input fails before
   destination creation. Refused inputs include the handoff `AppName` reject
   vectors (empty, `my app`, `my.app`, `my/app`, 65 `a` characters, `café`).
   The driver reports stable code `APP.INVALID_NAME` and does not create the
   destination. ADR-RUN-0010 is the predicate.
2. **REQ-RUN-0501 / REQ-RUN-0502** — A small `--var-file` driver sequences
   validation, cargo-generate, sc-compose, and the generated project's
   `just setup`, `just lint`, `just test`, and `just build-cli`. It does not include wizard
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
6. **REQ-RUN-0701** — Fixture CI discovers supported answers files and runs
   the amended sequence: `just new`, then `just setup`, then `just lint`,
   then `just test`, then `just build-cli`. No source file changes between
   generation and the end of that sequence. A workspace-wide build is not a
   matrix step. `cargo tree -p cli -e normal` stays under NFR-RUN-0001.
7. **REQ-RUN-0306** — Plain cargo-generate remains usable for the template tree
   with documented limitations around separately rendered Jinja documents.
8. **REQ-RUN-0307 / NFR-RUN-0006** — The generated project is an
   sc-lint 0.8.0 consumer. `sc-lint init --just` supplies `sc-lint.toml`,
   `.sc-lint/bootstrap.py`, and public recipes `setup`, `lint`, `test`, and
   `upgrade`. `db-prepare` remains on that `Justfile` and must survive a later
   `sc-lint init --just`. Fixture CI, in each generated project,
   runs the amended REQ-RUN-0701 sequence. `just setup`
   installs the consumer tool and is not a source edit. Boundary manifests
   stay in this repository.
9. **REQ-RUN-0309 / ADR-RUN-0004** — Generated `crates/daemon` depends on
   `sc-observability` at the exact version a-1 recorded. Rendered `main`
   loads config with `sc-config`, initialises `sc-observability`, then calls
   `Daemon::builder`. The template does not depend on
   `sc-observability-otlp`. The four library crates still do not depend on
   the logging runtime.
10. **REQ-RUN-0202 / REQ-RUN-0503 / REQ-RUN-0701** — Amend only those three
    sections of `docs/requirements.md`. REQ-RUN-0701's command list and
    success criterion 1 become `just new`, then `just setup`, then `just lint`,
    then `just test`, then `just build-cli`. `just setup` installs the
    sc-lint 0.8.0 consumer and is not a source edit. REQ-RUN-0503 success
    criterion 2 names `just setup` immediately before `just lint`.
    REQ-RUN-0202 closes the OPEN on `daemon start`: the generated CLI provides
    that subcommand, it execs the daemon binary, and the suggested-action
    string matches it. The same section records the `--json` mapping for
    every `TransportError` variant a-3 ships. Do not amend any other
    requirement in that file.
11. **NFR-RUN-0008** — After `template/**/*.rs` exists, root `just lint`'s
    sc-lint line-counts check covers those files. a-5 closed only
    `crates/*/src`. This sprint does not add a line-counting script. Files
    under a `tests/` directory stay exempt.
12. **REQ-RUN-0202** — Generated CLI `daemon start` execs the daemon binary.
    Report-only commands do not start a daemon. The suggested action
    `run <app> daemon start` is that subcommand.
13. **REQ-RUN-0202** — Generated CLI `--json` mapping table for every
    `TransportError` variant in a-3's error inventory: `OpError` kind, code,
    message, `suggested_action`, and the envelope. The mapping lives in the
    generated CLI. It does not add an `sc-command` → `sc-transport` edge.
    One host-safe vector per variant.
14. **NFR-RUN-0001** — Consumer-owned `just build-cli` runs
    `cargo build -p cli --release`. Fixture CI runs it. The recipe warns that
    `cargo build --workspace` is the exemption and is not the shipped agent
    binary.
15. **ADR-RUN-0010** — `tests/fixtures/app-name-vectors.json` is the only copy
    of the accept and reject vectors at the generation seam. Fixture CI, in
    the generated project, constructs `sc_config::AppName`,
    `sc_transport::AppName`, and `sc_runtime::AppName` from that file and
    fails if any constructor diverges.
16. **REQ-RUN-0001** — After generation, the tree contains no `wizard/`
    directory, no `scripts/new_project.py`, no `boundaries/` directory, and
    no `sc-*` crate directory. The root workspace `members` list does not
    match `template/`. `template/` is not a workspace member.
17. **REQ-RUN-0203** — The generated service documents an operation error
    inventory for widget create and get: failure mode, stable code, cause,
    `suggested_action`, and recovery. Golden failure vectors cover those
    rows. The rows are not added to ADR-CMD-0001.

## Acceptance criteria

1. `req:REQ-RUN-0501` — `just new <dest> --var-file <fixture>` generates into
   an absent destination and stops at the first named failing standard tool.
2. `req:REQ-RUN-0201` — the generated example creates and gets one widget via
   the shared service path from REST, optional MCP, and a real transport client.
   This proves connectivity, not permanent application architecture.
3. `req:REQ-RUN-0202` — report-only `--json` against an unreachable endpoint
   returns a nonzero status and the exact `DAEMON.NOT_RUNNING` envelope without
   creating daemon, lock, socket, or database artifacts. The suggested action
   is `run <app> daemon start`. `daemon start` is a generated subcommand that
   execs the daemon binary and does not run on other commands. The REQ-RUN-0202
   OPEN that asked whether that subcommand exists is closed by this amendment.
   The real CLI-binary proof runs only on an isolated CI runner; a host-safe
   proof exercises the same reporting path without prescribing helper functions.
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
9. `req:REQ-RUN-0701` — fixture CI runs the amended sequence and nothing else
   in the matrix: `just new`, then `just setup`, then `just lint`, then
   `just test`, then `just build-cli`, without modifying the result between
   generation and the end of that sequence. A workspace-wide build is not a
   step. PR-mode lockfiles resolve all four libraries to the checkout paths;
   release lockfiles resolve all four to the crates.io registry.
10. `req:NFR-RUN-0001` — This sprint is the feature-level close. Fixture CI
    runs `cargo tree -p cli -e normal` on the generated project, and the named
    server-only crates are absent. `just build-cli` runs
    `cargo build -p cli --release` and fixture CI runs that recipe. A
    workspace-wide build is the documented exemption and is not the shipped
    agent binary. This criterion does not split crates and does not depend on
    a-5 closing ADR-RUN-0003.
11. `ADR-RUN-0303` — inspection confirms that reusable runtime behavior remains
   in the published crates while routes, schema, queries, and application
   composition remain editable template code.
12. `req:REQ-RUN-0307` / `req:NFR-RUN-0006` — a project generated from
    `wizard/fixtures/sqlite-mcp.json` lists `setup`, `lint`, `test`,
    `upgrade`, `db-prepare`, and `build-cli`. Criterion 9 exits 0 with no
    edits. `template/` contains `sc-lint.toml` and `.sc-lint/bootstrap.py`
    from sc-lint 0.8.0, and it contains no `boundaries/` directory, no
    clippy/rustfmt/deny config, and no `mod` or `import` in the `Justfile`.
    `minimum_version` is `0.8.0`.
13. `req:REQ-RUN-0309` / `ADR-RUN-0004` — rendered `crates/daemon/Cargo.toml`
    requires the a-1 `sc-observability` version and has no
    `sc-observability-otlp` requirement. Rendered `main.rs` orders config
    load, then `sc-observability` init, then `Daemon::builder`.
    Library logging edges stay on the crate manifests and leaf `cargo tree`
    checks. This criterion does not add a four-crate tree in the generated
    project.
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
    `wizard/fixtures/*.json`. The same driver tests refuse the handoff
    `AppName` reject vectors before destination creation and emit
    `APP.INVALID_NAME`.
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
    the `open` config type are recorded as example choices in the template
    comments.
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
22. `req:REQ-RUN-0202` / `req:REQ-RUN-0503` / `req:REQ-RUN-0701` — the only
    edit this sprint makes to `docs/requirements.md` amends these three
    requirements. The Active green sequence is `just new`, then `just setup`,
    then `just lint`, then `just test`, then `just build-cli`. `just setup`
    installs the sc-lint 0.8.0 consumer and is not a source edit.
    REQ-RUN-0503 success criterion 2 names `just setup` immediately before
    `just lint`. REQ-RUN-0701's command list and success criterion 1 name
    that full sequence. REQ-RUN-0202 states that `daemon start` exists and
    records the transport `--json` mapping. No other requirement section
    changes. Criterion 9 is the fixture run of that amended sequence.
23. `req:REQ-RUN-0202` — one host-safe `--json` vector exists for every
    `TransportError` variant a-3 ships. Each vector checks kind, code,
    message, `suggested_action`, and the envelope. The mapping is generated
    CLI code.
24. `ADR-RUN-0010` — fixture CI reads `tests/fixtures/app-name-vectors.json`
    and runs the accept and reject vectors through `sc_config::AppName`,
    `sc_transport::AppName`, and `sc_runtime::AppName`. A divergence fails
    the job.
25. `req:REQ-RUN-0001` — a generated tree contains no `wizard/` directory, no
    `scripts/new_project.py`, no `boundaries/` directory, and no `sc-*`
    directory. Root `Cargo.toml` `members` does not match `template/`.
26. `req:REQ-RUN-0203` — the generated service's operation error inventory
    lists each widget create/get failure with a stable code, cause,
    `suggested_action`, and recovery. Golden vectors match those rows.
    ADR-CMD-0001 gains no widget row.

## This sprint does not close

It ships `daemon start` and does not implement CLI auto-start. It does not
add service-manager definitions, prescribed CLI helper/test shapes, attribute
macros, attribute-generated tests, process daemon tests, Postgres, or the
wizard. It does not claim the example's internal file/crate organization is
mandatory for generated applications. It does not replace the
source-maintainer sc-lint install owned by a-5. It does not map unknown
routes, Axum body rejections, or rmcp `Err(McpError)` into the envelope. It
does not add widget operation codes to ADR-CMD-0001. It does not amend
`docs/requirements.md` outside REQ-RUN-0202, REQ-RUN-0503, and REQ-RUN-0701.
It does not re-close the `crates/*/src` half of NFR-RUN-0008.

## Paths to delete

Generated test destinations and temporary values files are removed by their
tests. No repository source path is deleted.

## Required validation

Run the acceptance criteria. In each generated project the matrix sequence is
`just setup`, `just lint`, `just test`, and `just build-cli` after `just new`.
No daemon child process runs on the host. CI output is the record.
