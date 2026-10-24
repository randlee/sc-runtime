# Generation driver

`scripts/new_project.py` is the `--var-file` driver behind `just new`. It is
not a JSON API. A wizard UI is a later sprint. This driver never reads stdin,
never prompts, and never starts a wizard.

Stderr is exactly one line:

```text
<code> <cause> — <recovery>
```

The dash in that line is U+2014. An agent selects the row by `code`.
`GEN.STEP_FAILED` puts the failed pipeline step in the cause. Later steps do
not run.

| Failure | Code | Cause | Recovery |
|---|---|---|---|
| schema-invalid answers | `GEN.SCHEMA_INVALID` | answers fail wizard/answers.schema.json | fix the file; cargo is not called; the destination is not created |
| reserved database | `GEN.DB_RESERVED` | db is postgres or both | set db to sqlite; the destination is not created |
| invalid project name | `APP.INVALID_NAME` | name fails the project-name pattern in wizard/answers.schema.json | fix the name to that pattern; the destination is not created |
| missing sc-compose | `GEN.TOOL_MISSING` | SC_COMPOSE unset and sc-compose not on PATH | set SC_COMPOSE or install sc-compose on PATH; cargo generate does not run |
| destination conflict | `GEN.DEST_CONFLICT` | the destination path exists | choose an absent path |
| pipeline step | `GEN.STEP_FAILED` | step is `values`, `cargo-generate`, `sc-compose`, `setup`, `lint`, `test`, `build-cli`, or `build-daemon` | stderr names that step; fix it; later steps do not run |

The project-name `pattern` on `wizard/answers.schema.json` is the only
character class. The driver reads it from that schema. `project_name` is not a
cargo-generate placeholder: `cargo generate` receives it as `--name`.

`postgres` and `both` stay in the schema enum so a later release can accept
them. v0.1 refuses them with `GEN.DB_RESERVED` before it creates the
destination. The reserved values are an annotation on the schema, not a list
in the driver.

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
3. Resolve `sc-compose` from `SC_COMPOSE`, then from `PATH`.
4. Write `values.toml` (no `project_name` key; schema defaults applied).
5. Create the absent destination and run `cargo generate --init` into it.
   Then run `cargo fmt --all` so a removed Liquid line does not leave a
   blank that `cargo fmt --check` would reject.
6. Write the checkout patch unless release mode is on.
7. Render `AGENTS.md.j2` and `CLAUDE.md.j2` with `sc-compose` and delete the
   `.j2` files.
8. Run `just setup`, `just lint`, `just test`, `just build-cli`, and
   `just build-daemon` in the generated project.

`SC_COMPOSE` wins over a `sc-compose` binary on `PATH`. When both are absent
the driver exits with `GEN.TOOL_MISSING` and does not run `cargo generate`.

## Plain cargo-generate

`cargo generate --path template` still builds a workspace without this driver
and without a wizard. Placeholders supply `db`, `mcp`, and `licence`. The
project name is `--name`.

Limitations, stated honestly:

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
