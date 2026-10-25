# sc-runtime v0.1.0 — Core Runtime, Cargo-Generate Template, and Wyvern Wizard

**Released:** October 25, 2026 · **Tag:** `v0.1.0` · **Milestone:** first release

[Release notes](https://github.com/randlee/sc-runtime/blob/v0.1.0/docs/releases/v0.1.0.md) · [Architecture](https://github.com/randlee/sc-runtime/blob/v0.1.0/docs/architecture.md) · [Requirements](https://github.com/randlee/sc-runtime/blob/v0.1.0/docs/requirements.md)

---

## Service Author — building a JSON CLI backed by an HTTP daemon

**As a service author, I want a template that generates a working JSON CLI, HTTP daemon, SQLite store, and optional MCP surface, so that I can start from a licensed, linted, tested skeleton instead of assembling transport, config, and envelope plumbing myself.**

v0.1.0 is the first release of that skeleton. One command renders a full project, then runs its own `just setup`, `just lint`, `just test`, `just build-cli`, and `just build-daemon` before handing it back:

```bash
just new <dest> --var-file wizard/fixtures/sqlite-mcp.json
```

Three fixtures cover the v0.1 combinations: `sqlite-mcp.json` (SQLite plus an MCP surface), `sqlite-no-mcp.json` (SQLite only), and `sqlite-apache.json` (Apache-fronted deployment). The driver validates your answers file against `wizard/answers.schema.json` before it renders anything, so a typo in a variable name fails at the schema instead of halfway through a build.

The Wyvern wizard ships on this tree too — `wizard/wizard.json`, `wizard/pages/`, and `scripts/run_wizard.py` — for teams that would rather answer pages than hand-edit JSON. Its tests run as part of the generation suite. On a development checkout the generated project points at `crates/` in this repository so you can iterate on the runtime and the service together; pass `--release` (or set `SC_RUNTIME_RELEASE=1`) and the generated manifest keeps crates.io version requirements instead.

## Library Integrator — consuming the four crates directly

**As a Rust integrator, I want the runtime's layers as separate, independently publishable crates, so that I can take the transport or the envelope without adopting the daemon assembly.**

The workspace is exactly four MIT-licensed crates at 0.1.0, each with its own `README.md` and `LICENSE`:

- **`sc-config`** — JSON configuration with environment-variable overrides.
- **`sc-transport`** — local endpoint resolution plus HTTP between a daemon and its CLI.
- **`sc-command`** — the response envelope, with REST and MCP conversions.
- **`sc-runtime`** — daemon assembly: instance lock, one router, graceful shutdown.

Publish order is the three leaves, then `sc-runtime`. crates.io publication has not happened yet — the registry still returns 404 for all four names — so the working dependency form for v0.1.0 is the tag:

```toml
sc-config = { git = "https://github.com/randlee/sc-runtime", tag = "v0.1.0" }
sc-transport = { git = "https://github.com/randlee/sc-runtime", tag = "v0.1.0" }
sc-command = { git = "https://github.com/randlee/sc-runtime", tag = "v0.1.0" }
sc-runtime = { git = "https://github.com/randlee/sc-runtime", tag = "v0.1.0" }
```

The workspace `rust-version` is 1.94.1 and release preflight uses that toolchain.

## Release Engineer — publishing the tree through the sc-publish kit

**As a release engineer, I want publishing to be kit-generated rather than hand-authored, so that a release cannot drift from the channel contracts it is supposed to satisfy.**

Release workflows and `release/publish-*.toml` are produced by the sc-publish kit's `install.py`; the pin lives in `release/sc-publish-pin.toml`, and hand-authored publish workflows are forbidden (ADR-RUN-0002). The GitHub Release for `v0.1.0` carries the workspace source archive `sc-runtime-<version>-source.tar.gz`; `release_binaries` is empty, so this phase ships no binaries.

Note that this tag was cut on the integrated tree. Registry publication of the four crates and the registry-backed fixture matrix at the tag remain open gates that need release authorization and registry credentials.

## Not this release

- **End users of an installed binary** — no release binaries ship in v0.1.0 (`release_binaries` is empty), so there is no installer, Homebrew, or Scoop path to document yet. Skipped by design, not by omission.
- **CLI auto-start** (REQ-RUN-0206) — the CLI still reports `DAEMON.NOT_RUNNING` when the daemon is unreachable; an auto-start layer is a later decision.
- **Attribute-generated CLI tests** (REQ-RUN-0312) — the template proves the example operation through the daemon fixture instead.

## Get started

```bash
git clone https://github.com/randlee/sc-runtime && cd sc-runtime
git checkout v0.1.0
just new "$dest" --var-file wizard/fixtures/sqlite-mcp.json --release
```
