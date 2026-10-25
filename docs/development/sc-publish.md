# sc-publish (GitHub + crates.io)

Phase A core release (**sprint a-7**) uses the **vendored [sc-publish](https://github.com/randlee/sc-publish) kit** for:

- **GitHub Releases** (tag, release gate, immutable assets)
- **crates.io** publication of `sc-config`, `sc-transport`, `sc-command`, and
  `sc-runtime` in dependency order

Hand-editing installed kit files (workflows under `.github/workflows/`,
`.github/scripts/release_*.py`, publisher agents) is not supported. Change
`release/sc-publish-consumer-input.json` or advance the kit pin, then re-run
`install.py`.

## Consumer files (this repo)

| File | Role |
| --- | --- |
| `release/sc-publish-pin.toml` | Exact upstream kit SHA |
| `release/sc-publish-consumer-input.json` | Crates, project metadata, release targets |
| `release/publish-artifacts.toml` | Rendered by install (do not hand-edit) |
| `release/publish-channel-contracts.toml` | Rendered by install (do not hand-edit) |
| `release/publish-preflight.toml` | Rendered by install (do not hand-edit) |

Phase A uses **crates + GitHub release only** (`channels: {}` in consumer
input). Homebrew, npm, PyPI, scoop, and winget are out of scope until added
to the consumer input and re-installed.

`release_binaries` is empty. Phase A ships no release binaries. The single
linux `release_targets` row (`x86_64-unknown-linux-gnu`, `ubuntu-latest`,
`tar.gz`) is the row [ADR-RUN-0002](../architecture.md) names for the
workspace source archive `sc-runtime-{version}-source.tar.gz` on the
`v0.1.0` GitHub Release. The release toolchain is Rust `1.94.1`, the same
`rust-version` as the workspace. Publish order is `sc-config`,
`sc-transport`, `sc-command`, then `sc-runtime`.

## Install (a-7 or release maintainer)

```bash
# 1. Clone kit at pinned revision (never use a shared mutable sibling checkout)
mkdir -p .sc-publish-kit
git -C .sc-publish-kit init 2>/dev/null || true
git -C .sc-publish-kit fetch --depth 1 \
  "$(grep revision release/sc-publish-pin.toml | cut -d'"' -f2)" \
  https://github.com/randlee/sc-publish \
  "$(grep revision release/sc-publish-pin.toml | awk -F'"' '{print $2}')"
# Prefer: clone fresh at pin per README.sc-publish.md

python3 .sc-publish-kit/plugins/sc-publish/.github/scripts/bootstrap_sc_compose.py --venv .sc-publish-venv
.sc-publish-venv/bin/python .sc-publish-kit/plugins/sc-publish/install.py \
  --input release/sc-publish-consumer-input.json .
.sc-publish-venv/bin/python .sc-publish-kit/plugins/sc-publish/install.py \
  --dry-run --input release/sc-publish-consumer-input.json .
```

Add `.sc-publish-kit/` and `.sc-publish-venv/` to local gitignore habits; kit
docs recommend gitignoring the kit cache.

## Cursor vs ATM

After install, phase-end release uses **`/cursor-publish`** (foreground session,
`.cursor/agents/publisher.md`) when working in Cursor, or the ATM publisher
teammate when using codex-orchestration. Both read the same manifests.

Human release authorization still applies for production `release.yml` dispatch
and crates.io credentials in GitHub environments.
