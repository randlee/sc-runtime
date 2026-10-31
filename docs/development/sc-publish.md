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

The previous kit pin was rejected because its actual installer destination
inventory included `docs/sprints/**`. The current pin is the qualified exact
revision `9677ab083e7dd3feb258235a342e321ca1b45d8d`, from unmerged upstream
[PR 137](https://github.com/randlee/sc-publish/pull/137). Qualification must enumerate every actual
copy/render/package destination, reject that class plus traversal, absolute or
duplicate paths and unsafe symlinks, and compare the inventory with real install
outputs. Upstream source history may remain in `docs/sprints/**`; checking source
checkout absence is not sufficient or required. Install from the exact qualified remote full SHA. Kit
preparation may proceed independently of runtime parents; final composition
consumes their approved merges. b-18's [validation recipe](../plans/phase-b/sprint-b-18-template-observability-integration.md#required-validation)
is authoritative for that install and inventory check.

The 2026-10-31 [initial release amendment](../plans/phase-b/initial-release-observability-shim.md)
and [ADR-RUN-0017](../architecture.md#adr-run-0017-initial-release-observability-shim)
replace the old unpublished-1.5.1 initial release prerequisite. `run-h0h`
prepares the same five 0.1.1 packages using actual registry logger/types
`=1.5.0`, the command-owned neutral contract and private generated adapter.
Delete local development packages/paths and obsolete generation-mode rewrites
before publication; current product code and guards await that implementation.
Strict guards must enforce this exact profile, all-five normal/build identity
closures and capability-gated cleanup. Full logger/OTLP and sc-compose/ATM SDK
exclusions remain. No extra crate is published.

Before any upload, select all five interdependent packages in one verified dry
run and inspect actual `.crate` normalized manifests/contents for registry
requirements and absence of local shim/adapter/path/git/patch leakage:

```sh
cargo +1.96.1 publish --dry-run --locked -p sc-config -p sc-transport -p sc-command -p sc-runtime -p sc-cron
```

Record source/toolchain identities, archive checksums and verification output.
Do not use `--no-verify` or list-only output as build proof. Keep the actual kit
publication order config, transport, command, runtime, cron; db-sqlite remains
unpublished. Credentials, human publication authorization, qualified kit and
controlled-host benchmarks remain required. No upload or dispatch is authorized
by this documentation change. Same-source publication, registry-generated proof
and tag `v0.1.1` remain binding.

[Issue 97](https://github.com/randlee/sc-runtime/issues/97) tracks upstream cleanup
outside the initial release epic. Refresh registry availability independently
of the pinned lock at readiness; indeterminate discovery fails closed. Actual
published matching upstream trait/Noop/error and direct Arc capability proof
requires removal of the contract/adapter/pins and restoration of cron's original
forbidden command edge before another shim release. Published incompatibility
blocks release for remediation. Do not treat version >=1.5.1 as sufficient or
promise nominal trait compatibility across old/new SDK selections. The plan
requires representative current-consumer and upstream probes and an explicit
migration version strategy.

Pre-publication consumer preflight runs static release source contracts and
its active dependency guard. It cannot execute the registry-only generated
proof before the five internal crates are published. The qualified kit has no
consumer command hook after the crate-publication stage. The later `run-h0h`
release owner must run the explicit proof after all five publications and
before accepting the same-source release/tag evidence:

```sh
SC_RUNTIME_REGISTRY_PROOF=1 python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_mode_uses_registry
```

Ordinary pre-publication discovery skips that unavailable proof. The opted-in
proof must actually execute and fail on unmet release prerequisites; it may
never report a skipped test as registry evidence.

## Install (a-7 or release maintainer)

```bash
# Fetch exactly the qualified consumer pin into a fresh isolated cache.
# Verify the pinned revision and inventory before installation.
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
