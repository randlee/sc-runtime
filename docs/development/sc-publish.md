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

**Original Phase B hold (b-1 through b-12):** `integrate/phase-b` does not retag `v0.1.0`, does not publish,
and keeps `sc-cron` and `sc-runtime-db-sqlite` out of the consumer input
(`publish = false` on those crates). Merge to `develop` does not trigger a
crates.io release of the new workspace members.

`release_binaries` is empty. Phase A ships no release binaries. The single
linux `release_targets` row (`x86_64-unknown-linux-gnu`, `ubuntu-latest`,
`tar.gz`) is the row [ADR-RUN-0002](../architecture.md) names for the
workspace source archive `sc-runtime-{version}-source.tar.gz` on the
`v0.1.0` GitHub Release. The release toolchain is Rust `1.94.1`, the same
`rust-version` as the workspace. Publish order is `sc-config`,
`sc-transport`, `sc-command`, then `sc-runtime`.

## Phase B continuation and later release

The owner directed on 2026-10-30 that Phase B finish before observability
`1.5.1`. b-18 prepares the `0.1.1` consumer input/order and release/preflight
source contracts, installs a qualified exact kit revision, and proves complete
output ownership plus repeat dry-run/drift. It also delivers the real development
composition, shared logger, HTTP+cron JSONL, shutdown precedence, and isolated-CI
signal lifecycle using the approved real registry `1.5.0` adapter.

The current kit pin is rejected because its actual installer destination
inventory includes `docs/sprints/**`. Qualification must enumerate every actual
copy/render/package destination, reject that class plus traversal, absolute or
duplicate paths and unsafe symlinks, and compare the inventory with real install
outputs. Upstream source history may remain in `docs/sprints/**`; checking source
checkout absence is not sufficient or required. Select an exact qualified remote
full SHA before installing; this timing amendment does not change the pin. Kit
preparation may proceed independently of runtime parents; final composition
consumes their approved merges. b-18's [validation recipe](../plans/phase-b/sprint-b-18-template-observability-integration.md#required-validation)
is authoritative for that install and inventory check.

The governing timing amendment in [REQ-RUN-0702](../requirements.md),
including Success Criteria 6–7, and [architecture](../architecture.md)
assigns actual publication and release acceptance after Phase B to `run-h0h`:
matching published observability `1.5.1` and external prerequisite `run-03z.7`, bridge/path
removal and direct upstream Arc coercion, then human-authorized publication of
`sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, and `sc-cron` in that
order. `sc-runtime-db-sqlite` remains unpublished. Registry-only generation
(`scripts/new_project.py --release`), consumer release preflight, and
`test_release_mode_uses_registry` must pass later, without patches/paths or a
weakened `scripts/check_observability_release.py`. Publication/proof and tag
`v0.1.1` bind to one unchanged source commit. Phase completion does not claim
these deferred release checks passed; readiness review after phase-ending fixes
must state the limits explicitly. Canonical `1.5.1` release inventories remain.

## Install (a-7 or release maintainer)

```bash
# Fetch exactly the qualified consumer pin into a fresh isolated cache.
# The current rejected pin must be advanced by b-18 before this install.
SC_PUBLISH_REPOSITORY="$(sed -n 's/^repository = "\([^" ]*\)"/\1/p' release/sc-publish-pin.toml)"
SC_PUBLISH_REVISION="$(sed -n 's/^revision = "\([0-9a-f]*\)"/\1/p' release/sc-publish-pin.toml)"
test -n "$SC_PUBLISH_REPOSITORY"
test "${#SC_PUBLISH_REVISION}" -eq 40
test "$SC_PUBLISH_REVISION" != "16457e06ade6c0b003ff75addb7d694e9da41fe2"
mkdir -p .sc-publish-kit
git -C .sc-publish-kit init
git -C .sc-publish-kit remote add origin "$SC_PUBLISH_REPOSITORY"
git -C .sc-publish-kit fetch --depth 1 origin "$SC_PUBLISH_REVISION"
git -C .sc-publish-kit checkout --detach FETCH_HEAD
test "$(git -C .sc-publish-kit rev-parse HEAD)" = "$SC_PUBLISH_REVISION"
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_pinned_publish_kit_inventory

python3 .sc-publish-kit/plugins/sc-publish/.github/scripts/bootstrap_sc_compose.py --venv .sc-publish-venv
.sc-publish-venv/bin/python .sc-publish-kit/plugins/sc-publish/install.py \
  --input release/sc-publish-consumer-input.json .
.sc-publish-venv/bin/python .sc-publish-kit/plugins/sc-publish/install.py \
  --dry-run --input release/sc-publish-consumer-input.json .
```

Add `.sc-publish-kit/` and `.sc-publish-venv/` to local gitignore habits; kit
docs recommend gitignoring the kit cache.

## Cursor vs ATM

After install, the later authorized release uses **`/cursor-publish`** (foreground session,
`.cursor/agents/publisher.md`) when working in Cursor, or the ATM publisher
teammate when using codex-orchestration. Both read the same manifests.

Human release authorization still applies for production `release.yml` dispatch
and crates.io credentials in GitHub environments.
