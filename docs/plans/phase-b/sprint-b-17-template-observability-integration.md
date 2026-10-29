---
id: b-17
phase: b
status: planned
closure_type: integration
target_boundary: generated-daemon-observability
track: observability
wave: 9
branch: sprint/b-17-template-observability-integration
worktree: sprint/b-17-template-observability-integration
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0003
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0309
  - REQ-RUN-0702
  - REQ-RT-0001
  - REQ-RT-0004
  - REQ-CRON-0003
  - REQ-CRON-0005
  - NFR-RUN-0005
  - NFR-RUN-0002
  - NFR-RUN-0008
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0004
  - ADR-RUN-0007
  - ADR-RUN-0013
  - ADR-RT-0001
owned_paths:
  - template/crates/daemon/src/main.rs
  - template/crates/daemon/src/observability.rs
  - template/crates/daemon/src/cron.rs
  - tests/integration/generation/test_observability.py
  - release/sc-publish-consumer-input.json
  - release/sc-publish-pin.toml
  - release/publish-preflight.toml
  - release/publish-preflight.toml.j2
  - docs/development/sc-publish.md
  - release/publish-artifacts.toml
  - release/publish-artifacts.toml.j2
  - release/publish-channel-contracts.toml
  - release/publish-channel-contracts.toml.j2
  - release/sc-publish-pin.toml.example
  - release/homebrew/formula.rb.j2
  - release/scoop/manifest.json.j2
  - README.sc-publish.md
  - install.py
  - docs/consumer-preflight-and-docli.md
  - docs/npm-channel-compatibility.md
  - .claude/agents/crates-io-publisher.md
  - .claude/agents/github-release-publisher.md
  - .claude/agents/homebrew-publisher.md
  - .claude/agents/npm-publisher.md
  - .claude/agents/publisher-channel-protocol.md
  - .claude/agents/publisher.md
  - .claude/agents/pypi-publisher.md
  - .claude/agents/scoop-publisher.md
  - .claude/agents/winget-publisher.md
  - .claude/skills/prerelease/SKILL.md
  - .claude/skills/prerelease/install.md
  - .claude/skills/prerelease/list.md
  - .claude/skills/prerelease/publish.md
  - .claude/skills/prerelease/references/installation-and-troubleshooting.md
  - .claude/skills/prerelease/scripts/prerelease.py
  - .claude/skills/publishing/SKILL.md
  - .claude/skills/publishing/agents/openai.yaml
  - .claude/skills/publishing/evals/channel-name-inquiry.md
  - .claude/skills/publishing/evals/publisher-preflight.md
  - .claude/skills/publishing/evals/publisher-recovery.md
  - .claude/skills/publishing/preflight.xml.j2
  - .claude/skills/publishing/publish.xml.j2
  - .claude/skills/publishing/ref/channel-contracts.md
  - .claude/skills/publishing/ref/publish-kit-requirements.md
  - .claude/skills/publishing/ref/release-state-strategy.md
  - .claude/skills/publishing/ref/renderer-contract.md
  - .codex/skills/prerelease/SKILL.md
  - .cursor/agents/publisher.md
  - .cursor/commands/cursor-publish.md
  - .cursor/skills/cursor-publish/SKILL.md
  - .github/actions/extract-published-renderer/action.yml
  - .github/actions/install-linux-native-deps/action.yml
  - .github/actions/setup-lint-toolchain/action.yml
  - .github/actions/setup-python-release-build/action.yml
  - .github/actions/setup-renderer/action.yml
  - .github/actions/setup-sc-lint/action.yml
  - .github/actions/verify-published-release/action.yml
  - .github/scripts/bootstrap_sc_compose.py
  - .github/scripts/npm_release.py
  - .github/scripts/release-preflight-docli.sh
  - .github/scripts/release_artifacts.py
  - .github/scripts/release_credentials.py
  - .github/scripts/release_gate.sh
  - .github/scripts/release_immutability.py
  - .github/scripts/release_manifest.py
  - .github/scripts/release_python.py
  - .github/scripts/release_registry.py
  - .github/scripts/run-consumer-preflight.sh
  - .github/scripts/setup_sc_lint_source.py
  - .github/scripts/worker_result.py
  - .github/scripts/tests/test_fail_closed_probes.py
  - .github/scripts/tests/test_install.py
  - .github/scripts/tests/test_npm_release.py
  - .github/scripts/tests/test_preflight_regressions.py
  - .github/scripts/tests/test_prerelease.py
  - .github/scripts/tests/test_publish_kit_assets.py
  - .github/scripts/tests/test_publish_kit_scripts.py
  - .github/scripts/tests/test_release_artifacts.py
  - .github/scripts/tests/test_release_immutability.py
  - .github/scripts/tests/test_release_python.py
  - .github/scripts/tests/test_setup_sc_lint_source.py
  - .github/scripts/tests/test_worker_result.py
  - .github/workflows/crates-publish.yml
  - .github/workflows/homebrew-publish.yml
  - .github/workflows/npm-publish.yml
  - .github/workflows/prerelease-archive.yml
  - .github/workflows/pypi-publish.yml
  - .github/workflows/release-candidate.yml
  - .github/workflows/release-preflight.yml
  - .github/workflows/release.yml
  - .github/workflows/scoop-publish.yml
  - .github/workflows/template.yml
  - .github/workflows/winget-publish.yml
must_follow: [b-14, b-15, b-16]
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
---

# Sprint b-17 — generated daemon observability integration

## Goal

Make the generated daemon the sole full-observability composition root. It
builds one logger, shares the same underlying instance with runtime and cron,
and proves canonical HTTP and cron events in one JSONL sink.

## Relations

`must_follow` b-14, b-15, and b-16 because this sprint consumes the runtime
events, cron events/injection, and generated Cargo features respectively.
Each parent PR must merge into this branch before development begins and
before every fix round. There is no same-wave sibling.

## Consumed artifacts

- b-14: `DaemonBuilder::logger` and HTTP events.
- b-15: cron worker/due-run event taxonomy; b-13 supplies
  `Scheduler::open_with_logger`, component propagation/events, and
  `cron.scheduler/logger.attached`.
- b-16: the daemon `cron` and `monty` Cargo features. This sprint does not
  edit shared Cargo files. It also consumes the exact `0.1.1` registry pins
  and checkout-patch inventory owned by b-16.

The current pin `16457e06ade6c0b003ff75addb7d694e9da41fe2` is not a valid
b-17 starting artifact because that kit package includes upstream sprint
history at `docs/sprints/fix-homebrew-formula-bundled-paths-tojson.md`.
b-17 is blocked until sc-publish publishes a later qualified exact revision
whose installer package contains no `docs/sprints/**`. This plan does not
invent that future SHA. Once available, b-17 updates
`release/sc-publish-pin.toml`, installs only from that exact revision, and
proves its installed file inventory equals the explicit `owned_paths` set;
the leaked upstream sprint-history document is never copied or owned here.

## Fixed composition signatures

```rust
pub struct Observability {
    logger: std::sync::Arc<sc_observability::v2::Logger>,
    shared: sc_observability_types::SharedCanonicalLog,
}

impl Observability {
    pub fn logger(&self) -> sc_observability_types::SharedCanonicalLog;
    pub fn shutdown(&self) -> Result<(), sc_observability::v2::ShutdownError>;
}

#[cfg(feature = "cron")]
pub fn open(
    config: &CronConfig,
    instance_root: &std::path::Path,
    logger: SharedCanonicalLog,
) -> Result<Scheduler<SqliteScheduleStore, SystemClock, LoggedJitter<SystemJitter>>, CronError>;
```

The wrapper resolves `database_path` below `instance_root`, maps config to
`ScheduleLimits`, and calls exactly:

```rust
Scheduler::open_with_logger(
    database_path,
    limits,
    Arc::new(SystemClock),
    Arc::new(SystemJitter::with_logger(logger.clone())),
    logger,
)
```

`Observability` creates `Arc<Logger>` once and derives `shared` by Arc
coercion; it never clones `Logger`. `main` injects clones of the shared handle,
keeps scheduler state alive for daemon lifetime, drops runtime/scheduler work,
then calls shutdown. The reachable result precedence is fixed: logger
construction failure returns immediately; setup or scheduler-open failure
after logger construction remains primary and any shutdown failure is
secondary; daemon-run failure remains primary and any shutdown failure is
secondary; a successful run followed by shutdown failure returns process
failure mapped from `ShutdownError`; only successful run plus successful
shutdown returns success.

## Required work and deliverables

Every numbered item must land production-ready for integration closure. The
real generated composition, lifecycle ordering, sink behavior, and JSONL proof
must pass together; a fake-only or test-only wiring cannot close this sprint.

1. REQ-RUN-0004 / REQ-RUN-0309: change generated `Observability` to Arc ownership and expose only the
   types-level shared handle to library consumers.
2. REQ-RT-0001 / REQ-RT-0004: inject that handle into `DaemonBuilder` for every layout.
3. REQ-CRON-0003 / REQ-CRON-0005: under the b-16 `cron` feature, use b-13's
   `LoggedJitter` and `Scheduler::open_with_logger` to open the scheduler
   database below the instance root, apply configured limits, attach the same handle, and retain
   the scheduler. Feature-off behavior is unchanged.
4. NFR-RUN-0005 / NFR-RUN-0008: add a generated-project smoke test that polls with a deadline, sends a
   health request, enables cron, and parses JSONL. It must observe
   `http.server/request.*` plus the production
   `cron.scheduler/logger.attached` event with one service identity.
5. ADR-RUN-0007 / NFR-RUN-0002 / NFR-RUN-0009: prove the fixed reachable result/shutdown precedence and that
   logger shutdown happens after both library users stop. Do not add a
   post-construction production-logger fault injector: b-14 and b-15 already
   own rejecting-emitter contract tests for emission failure, while this
   sprint owns only the real production JSONL proof.
6. ADR-RUN-0002 / REQ-RUN-0002 / REQ-RUN-0003 / REQ-RUN-0702: extend the
   existing sc-publish consumer input for the
   `0.1.1` release. Preserve publish order `sc-config` (1), `sc-transport` (2),
   `sc-command` (3), `sc-runtime` (4), then publish `sc-cron` (5);
   `sc-runtime-db-sqlite` remains unpublished. Advance the current rejected
   pin to a later qualified exact kit revision with no `docs/sprints/**`.
   `test_pinned_publish_kit_inventory` must enumerate installer destination
   paths and assert exact equality with the installed-output subset of
   `owned_paths`; any `docs/sprints/**` destination fails closed.
   From a clean checkout, fetch
   the repository and exact revision in `release/sc-publish-pin.toml`, detach
   at `FETCH_HEAD`, bootstrap the pinned renderer, run the real installer,
   and commit every exact installed output listed in `owned_paths`. Require a
   clean repeat `--dry-run` and git drift check. Add the release-mode generated
   contract preflight and document that
   a human release owner with crates.io credentials must authorize and observe
   the generated release workflow. This repository neither publishes nor
   claims authority over the upstream observability `1.5.1` pair.
7. REQ-RUN-0702: after the five internal `0.1.1` crates are published, render
   with `scripts/new_project.py --release`, assert there is no
   `[patch.crates-io]` or path override, and build/test the generated daemon
   against the registry with `cron` and `monty`. The pre-publication source
   contract lives in
   `GeneratedObservabilityIntegration.test_release_contract_sources`; the
   registry-only proof lives in
   `GeneratedObservabilityIntegration.test_release_mode_uses_registry`.

## Acceptance criteria

1. ADR-RUN-0004: config -> log root -> logger -> shared injection -> run ->
   shutdown order is explicit and tested.
2. `REQ-RT-0004`: the rendered daemon writes correlated canonical HTTP events
   to its configured JSONL file.
3. `REQ-CRON-0003`: opening the real generated scheduler with the injected
   handle writes the real cron attachment event; no fake emitter stands in for
   the production logger in this integration proof.
4. ADR-RUN-0007 / NFR-RUN-0002 / NFR-RUN-0009: b-14/b-15
   rejecting-emitter contract tests remain the authoritative immediate,
   best-effort emission proof; this integration test verifies only reachable
   generated-process result/shutdown precedence and production JSONL output.
5. No generated library crate depends on the full observability runtime.
6. REQ-RUN-0702: the current bad pin has been replaced by a qualified exact
   later revision; its package inventory contains no `docs/sprints/**`. The
   real install from that exact pinned kit revision owns and
   synchronizes every listed manifest, workflow, script, action, agent, skill,
   and kit document; its repeat dry-run and git drift checks are clean,
   generated preflight passes, the human-authorized publish follows orders 1 through 5,
   and a subsequent `--release` generated build resolves all internal crates
   from the registry with no checkout patch.

## Required validation

```sh
python3 -m unittest tests.integration.generation.test_observability
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_contract_sources
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_pinned_publish_kit_inventory
SC_PUBLISH_REPOSITORY="$(sed -n 's/^repository = "\([^"]*\)"/\1/p' release/sc-publish-pin.toml)"
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
test -z "$(find .sc-publish-kit/plugins/sc-publish/docs/sprints -type f -print -quit 2>/dev/null)"
python3 .sc-publish-kit/plugins/sc-publish/.github/scripts/bootstrap_sc_compose.py --venv .sc-publish-venv
.sc-publish-venv/bin/python .sc-publish-kit/plugins/sc-publish/install.py --input release/sc-publish-consumer-input.json .
.sc-publish-venv/bin/python .sc-publish-kit/plugins/sc-publish/install.py --dry-run --input release/sc-publish-consumer-input.json .
git diff --check
git diff --exit-code
test -z "$(git status --porcelain)"
bash .github/scripts/run-consumer-preflight.sh
just lint
just test
just new generated/test-observability --var-file wizard/fixtures/sqlite-mcp.json
cargo test --manifest-path generated/test-observability/Cargo.toml --workspace
cargo test --manifest-path generated/test-observability/Cargo.toml -p widget-app-daemon --features cron
cargo +1.96.1 check --manifest-path generated/test-observability/Cargo.toml -p widget-app-daemon --features monty
cargo tree --manifest-path generated/test-observability/Cargo.toml -p widget-app-daemon -e normal
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_mode_uses_registry
just lint gates
```

The integration test parses every JSONL line as `LogEvent`, asserts both
targets and one service name, and uses an isolated temporary log root. The
install/drift block runs first from a clean sprint checkout; its cache and
virtual environment are ignored. The registry-only test runs only after the
human-authorized `0.1.1` publication. The release workflow generated from the
installed sc-publish kit remains the publication authority, so this sprint
edits its consumer input and regenerates every listed kit-owned output rather
than hand-editing installed files.

## This sprint does not close

- OTLP defaults, library-owned sink lifecycle, a cron workload example, or
  observability injection into config/transport/command/store crates.
