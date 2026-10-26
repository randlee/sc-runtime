# Generation driver

`scripts/new_project.py` is the driver behind `just new`. It is not a JSON
API. Three entry modes supply one answers object, and every mode then uses
the same validation and pipeline.

`--var-file` never reads stdin, never prompts, and never starts a wizard.
Interactive and `--prefill` modes call `scripts/run_wizard.py`. That runner
is the only place that looks for Wyvern. The layout paths for this milestone
are `wizard/wizard.json`, `wizard/pages/`, and `scripts/run_wizard.py`.

`--var-file` stderr is one line:

```text
<code> <cause> — <recovery>
```

The dash in that line is U+2014. An agent selects the row by `code`. Wizard
modes may print `[run-wizard]` diagnostics first. The last line is still
that inventory row. There is no uncoded `ERROR:` line. `GEN.STEP_FAILED`
puts the failed pipeline step in the cause. Later steps do not run. Child
output from a failed step is copied to stdout so the agent can read the
compiler or lint diagnostic. The recovery tells the agent to fix the step
or remove the partial destination.

| Failure | Code | Cause | Recovery |
|---|---|---|---|
| schema-invalid answers | `GEN.SCHEMA_INVALID` | answers fail wizard/answers.schema.json | fix the file; cargo is not called; the destination is not created |
| reserved database | `GEN.DB_RESERVED` | db is postgres or both | set db to sqlite; the destination is not created |
| invalid project name | `APP.INVALID_NAME` | name fails the project-name pattern in wizard/answers.schema.json | fix the name to that pattern; the destination is not created |
| missing sc-compose | `GEN.TOOL_MISSING` | SC_COMPOSE unset and sc-compose not on PATH | set SC_COMPOSE or install sc-compose on PATH; cargo generate does not run |
| bad SC_COMPOSE override | `GEN.TOOL_MISSING` | SC_COMPOSE does not point at an executable file | set SC_COMPOSE to the sc-compose binary; cargo generate does not run |
| cargo missing | `GEN.TOOL_MISSING` | cargo is not on PATH | install cargo on PATH; later steps do not run |
| wrong cargo-generate version | `GEN.TOOL_MISSING` | cargo-generate is not 0.25.1 | install cargo-generate 0.25.1; cargo generate does not run |
| cargo-generate won't start | `GEN.TOOL_MISSING` | cargo generate could not start | install cargo-generate 0.25.1; cargo generate does not run |
| tool missing on PATH | `GEN.TOOL_MISSING` | the tool name is not on PATH | install that tool on PATH; later steps do not run |
| missing wyvern | `GEN.TOOL_MISSING` | WYVERN_BIN unset and wyvern not on PATH | set WYVERN_BIN or install wyvern on PATH; cargo generate does not run |
| unresolved WYVERN_BIN | `GEN.TOOL_MISSING` | WYVERN_BIN is set but does not resolve to an executable | set WYVERN_BIN to the wyvern binary path; cargo generate does not run |
| Wyvern runtime failure | `GEN.TOOL_MISSING` | wyvern exited with a non-zero status | fix Wyvern or the wizard UI; cargo generate does not run |
| missing jsonschema | `GEN.TOOL_MISSING` | jsonschema is not installed | pip install jsonschema==4.26.0; cargo generate does not run |
| wizard cancel or dismiss | `GEN.WIZARD_CANCELLED` | wizard cancelled or dismissed | finish the wizard; the destination is not created |
| destination conflict | `GEN.DEST_CONFLICT` | the destination path exists | choose an absent path |
| pipeline step | `GEN.STEP_FAILED` | step is `values`, `cargo-generate`, `sc-compose`, `setup`, `lint`, `test`, `build-cli`, or `build-daemon` | fix it or remove the partial destination; later steps do not run |

The project-name `pattern` on `wizard/answers.schema.json` is the only
character class. The driver reads it from that schema. `project_name` is not a
cargo-generate placeholder: `cargo generate` receives it as `--name`.

`postgres` and `both` stay in the schema enum. Phase b accepts every layout
in REQ-RUN-0326, including MySQL and no database. Until that sprint lands,
v0.1 refuses a layout other than `sqlite` with `GEN.DB_RESERVED` before it creates the
destination. The reserved values are an annotation on the schema, not a list
in the driver. Plain `cargo generate` uses the same SQLite-only render:
`template/cargo-generate.toml` lists `sqlite` as the only `db` choice, so a
values file or prompt cannot select `postgres` or `both`.

## Entry modes

| Mode | Command | What is validated |
|---|---|---|
| interactive | `just new <dest>` | The person finishes the wizard at `wizard/wizard.json`. |
| prefill | `just new <dest> --prefill <partial.json>` | `<partial.json>` is merged into the descriptor's `config.prefill` for review. The validated object is what the wizard returns after Finish. |
| non-interactive | `just new <dest> --var-file <answers.json>` | The file is the answers object. The wizard does not start. |

`--var-file` wins when it is combined with `--prefill`. The wizard does not
start and the prefill file is not read. That preserves the non-interactive
rule.

A prefill key that the schema does not define is dropped with a stderr
warning. The on-disk `wizard/wizard.json` is not modified. Prefill is a
suggestion: the cargo values come from the finished wizard, not from the
prefill file.

Wyvern is resolved from `WYVERN_BIN` when that variable is set, otherwise as
`wyvern` on `PATH`. An override that does not resolve does not fall through
to `PATH`. Neither script contains an install path. When the binary is
missing, the runner exits non-zero. The last stderr line is
`GEN.TOOL_MISSING` and names `wyvern`, `WYVERN_BIN`, and `PATH`. `cargo
generate` is not called, and the destination is not created.

Cancel and dismissed both mean the person did not finish. The runner writes
no answers file. The last stderr line is `GEN.WIZARD_CANCELLED`. The driver
does not call `cargo generate` and does not create the destination.

The pages collect each current schema property and no others. Defaults are
the schema defaults; `wizard/pages/wizard-answers.js` mirrors only those.
The pages do not copy the project-name `pattern`. The runner checks the
finished object against `wizard/answers.schema.json`, and the driver checks
it again before the pipeline. `postgres` and `both` stay in the schema enum,
so the wizard offers them. The driver still answers `GEN.DB_RESERVED` and
does not create the destination.

A finished wizard that makes the same choices as a file in `wizard/fixtures/`
produces an equal JSON value. Key order and whitespace do not matter. Equal
values call the same `generate()` path, so the project files match. An
automated test stubs Wyvern, runs that path once from the stub and once from
`--var-file`, and compares the trees. The driver writes no timestamps into
project source. Ignore these when you `diff -r` two generated trees:

- `target/` is build output.
- The `[patch.crates-io]` checkout patch records this checkout's absolute
  crate paths.
- `.sc-lint/venv/` is the local sc-lint install from `just setup`. Its files
  embed the destination path.
- `.sc-compose/logs/` is the generator log and contains timestamps.

`--var-file` failures stay one inventory line on stderr. A wizard that
cancels ends with `GEN.WIZARD_CANCELLED`. A missing Wyvern binary ends with
`GEN.TOOL_MISSING`, and that line names `wyvern`, `WYVERN_BIN`, and `PATH`.
A runner schema or project-name reject ends with `GEN.SCHEMA_INVALID` or
`APP.INVALID_NAME`. `[run-wizard]` diagnostics may precede the inventory
line. A schema-valid finish that the driver then rejects keeps those
diagnostics and then the driver's one inventory line.

## Development and release

Development and pull-request mode writes `[patch.crates-io]` for `sc-config`,
`sc-transport`, `sc-command`, and `sc-runtime` into the generated
`Cargo.toml` after `cargo generate` returns and before the first Cargo
command. The paths are the absolute crate directories in this checkout. The
template keeps the `0.1.0` version requirements either way.

Release mode writes no patch. Pass `--release` or set `SC_RUNTIME_RELEASE=1`.
The four crates then resolve from the registry.

`scripts/new_project.py --patch-checkout DEST` writes that same table and
exits. The plain `cargo generate` CI job uses it so the patch is not
duplicated. That flag does not run the rest of the pipeline.

## Pipeline

1. Validate the answers file.
2. Refuse a destination that already exists.
3. Resolve `sc-compose` from `SC_COMPOSE`, then from `PATH`. A missing `just`
   or `cargo` binary is `GEN.TOOL_MISSING` and does not create the destination.
4. Write `values.toml` (no `project_name` key; schema defaults applied).
   Validation uses the `jsonschema` package (`requirements-generation.txt`,
   pinned at 4.26.0) against Draft 2020-12.
5. Require `cargo-generate` 0.25.1 (`cargo generate --version`), then create
   the absent destination and run `cargo generate --init` into it.
   Then run `cargo fmt --all` so a removed Liquid line does not leave a
   blank that `cargo fmt --check` would reject.
6. Write the checkout patch unless release mode is on. The patched crates are
   the `sc-*` keys in `template/Cargo.toml` `[workspace.dependencies]` that
   exist under `crates/`.
7. Render `AGENTS.md.j2` and `CLAUDE.md.j2` with `sc-compose` and delete the
   `.j2` files.
8. Run `just setup`, `just lint`, `just test`, `just build-cli`, and
   `just build-daemon` in the generated project.

`SC_COMPOSE` wins over a `sc-compose` binary on `PATH`. When both are absent
the driver exits with `GEN.TOOL_MISSING` and does not run `cargo generate`.
The same code covers a missing `just` or `cargo`, a `cargo-generate` version
other than 0.25.1, and an `SC_COMPOSE` path that is not an executable file.
The cause names the tool.

ADR-RUN-0401 point 6 is this driver pipeline. Point 8 requires template CI to
run `just new` and then the same consumer recipes again. That second pass is
the independent proof. It is not a second owner of the pipeline.

## Plain cargo-generate

`cargo generate --path template` still builds a workspace without this driver
and without a wizard. Placeholders supply `db`, `mcp`, and `licence`. The
project name is `--name`. `db` accepts only `sqlite`.

Limitations, stated honestly:

- `postgres` and `both` stay in `wizard/answers.schema.json`. Phase b is
  the plan that accepts every REQ-RUN-0326 layout. Until sprint `b-8`, the
  driver refuses a layout other than `sqlite` with `GEN.DB_RESERVED` before
  `cargo generate`. Plain `cargo generate` rejects them because they are
  not `db` choices. v0.1 does not render a Postgres store.
- `AGENTS.md.j2` and `CLAUDE.md.j2` are excluded from Liquid and are copied
  byte-for-byte. They still contain Jinja delimiters and frontmatter until
  `sc-compose` renders them. The driver does that render and deletes the
  `.j2` files. Plain `cargo generate` does not.
- Plain `cargo generate` does not write `[patch.crates-io]`. Before the
  crates are published, a build of that tree needs the checkout patch (the
  CI smoke applies `--patch-checkout` before the first Cargo command). After
  publication, release mode uses the registry and needs no patch.
- Plain `cargo generate` does not run `just setup`, `just lint`, `just test`,
  `just build-cli`, or `just build-daemon`.

## sc-lint line counts

Root `sc-lint.toml` records a `paths` glob of `template/**/*.rs` for the
line-counts check. sc-lint 0.8.0's line-counts implementation walks workspace
members and ignores unknown keys, and `template/` is not a workspace member.
The glob is the configuration this sprint is allowed to add. It does not add
a line-counting script and it does not make `template/` a member.
