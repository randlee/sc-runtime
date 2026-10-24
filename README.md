# sc-runtime

Four MIT-licensed library crates and a `cargo-generate` template for a JSON CLI, an HTTP daemon, SQLite, and optional MCP.

## Crates

| Crate | Role |
| --- | --- |
| `sc-config` | JSON configuration with environment overrides |
| `sc-transport` | Local endpoint resolution and HTTP between a daemon and its CLI |
| `sc-command` | Response envelope and REST/MCP conversions |
| `sc-runtime` | Daemon assembly: instance lock, one router, graceful shutdown |

Each crate is version 0.1.0 and is published on its own. Publish order is `sc-config`, `sc-transport`, `sc-command`, then `sc-runtime`. The workspace `rust-version` is 1.94.1.

## Generate a project

The supported core path, from a checkout of this repository:

```bash
just new <dest> --var-file wizard/fixtures/sqlite-mcp.json
```

The driver checks the answers file against `wizard/answers.schema.json`, renders `template/` with `cargo generate`, renders the agent documents, then runs `just setup`, `just lint`, `just test`, `just build-cli`, and `just build-daemon` in the new project.

`wizard/fixtures/` holds the v0.1 combinations: `sqlite-mcp.json`, `sqlite-no-mcp.json`, and `sqlite-apache.json`.

On a development checkout the driver adds a `[patch.crates-io]` entry that points the generated project at `crates/` in this checkout. A release run keeps the crates.io version requirements and skips that patch. Pass `--release`, or set `SC_RUNTIME_RELEASE=1`. Tag runs of `.github/workflows/template.yml` set that variable.

`cargo generate --path template` is the renderer the driver calls. Plain `cargo generate` is the same template with a values file and, on a development checkout, the driver's checkout patch.

## v0.1.0

Release notes: [docs/releases/v0.1.0.md](docs/releases/v0.1.0.md).

This tag ships the four crates and the template path above. CLI auto-start, attribute-generated CLI tests, and the Wyvern wizard remain deferred. A separate minimum Cargo version, beyond Cargo from the 1.94.1 toolchain (including `cargo publish --workspace`), remains deferred.
