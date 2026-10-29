---
id: b-18
phase: b
status: planned
closure_type: integration
target_boundary: generated-daemon-observability
track: observability
wave: 10
branch: sprint/b-18-template-observability-integration
worktree: sprint/b-18-template-observability-integration
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0003
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0309
  - REQ-RUN-0503
  - REQ-RUN-0701
  - REQ-RUN-0702
  - REQ-RT-0001
  - REQ-RT-0004
  - REQ-RT-0005
  - NFR-RT-0001
  - NFR-RT-0002
  - NFR-RT-0003
  - NFR-RT-0004
  - REQ-CRON-0003
  - REQ-CRON-0005
  - REQ-CRON-0019
  - NFR-CRON-0002
  - NFR-RUN-0005
  - NFR-RUN-0002
  - NFR-RUN-0008
  - NFR-RUN-0009
  - NFR-RUN-0010
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0004
  - ADR-RUN-0007
  - ADR-RUN-0011
  - ADR-RUN-0013
  - ADR-RUN-0014
  - ADR-RUN-0016
  - ADR-RT-0001
  - ADR-RT-0002
  - ADR-TRN-0006
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
must_follow: [b-14, b-17]
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
---

# Sprint b-18 — generated daemon observability integration

## Goal

Make the generated daemon the sole full-observability composition root. It
builds one logger, shares the same underlying instance with runtime and cron,
and proves canonical HTTP and cron events in one JSONL sink.

## Relations

`must_follow` b-14 and b-17 because this sprint consumes the runtime events
from b-14 and the completed cron environment/generated-daemon surface from
b-17. b-17 already follows b-15 and b-16, so cron events, injection, Cargo
features, and hook-environment wiring all arrive through that parent. Each
parent PR must merge into this branch before development begins and before
every fix round. There is no same-wave sibling.

## Consumed artifacts

- b-14: `DaemonBuilder::logger` and HTTP events.
- b-17: the daemon `cron` and `monty` Cargo features from b-16, the cron
  worker/due-run event taxonomy from b-15, and the shared Python/Monty hook
  environment contract. b-13 supplies `Scheduler::open_with_logger`,
  component propagation/events, and `cron.scheduler/logger.attached`. This
  sprint does not edit shared Cargo files. It also consumes the exact `0.1.1`
  registry pins and checkout-patch inventory that originated in b-16.

The current pin `16457e06ade6c0b003ff75addb7d694e9da41fe2` is not a valid
b-18 starting artifact because that kit package includes upstream sprint
history at `docs/sprints/fix-homebrew-formula-bundled-paths-tojson.md`.
b-18 is blocked until sc-publish publishes a later qualified exact revision
whose installer package contains no `docs/sprints/**`. This plan does not
invent that future SHA. Once available, b-18 updates
`release/sc-publish-pin.toml`, installs only from that exact revision, and
proves the installer destination inventory is complete and package-safe; the
leaked upstream sprint-history document is never copied or owned here.

## Fixed composition signatures

```rust
pub struct Observability {
    logger: std::sync::Arc<sc_observability::v2::Logger>,
    shared: sc_observability_types::SharedCanonicalLog,
}

impl Observability {
    pub fn logger(&self) -> sc_observability_types::SharedCanonicalLog;
    pub fn shutdown(self) -> Result<(), sc_observability::v2::ShutdownError>;
}

#[cfg(feature = "cron")]
pub fn open(
    config: &CronConfig,
    instance_root: &std::path::Path,
    logger: SharedCanonicalLog,
    hook_environment: HookEnvironmentHost,
) -> Result<Scheduler<SqliteScheduleStore, SystemClock, SystemJitter>, CronError>;
```

The wrapper resolves `database_path` below `instance_root`, maps config to
`ScheduleLimits`, and calls exactly:

```rust
let scheduler = Scheduler::open_with_logger(
    database_path,
    limits,
    Arc::new(SystemClock),
    logger,
)?
.with_hook_environment(hook_environment);

#[cfg(feature = "monty")]
let scheduler = scheduler.with_monty_host(monty_host);

Ok(scheduler)
```

`Scheduler::open_with_logger` constructs
`SystemJitter::with_logger(logger.clone())` internally, so callers cannot
pair the store and `RunCoord` handle with a differently configured jitter.
`main` builds `hook_environment` from generated daemon config through b-17's
typed builder, attaches the application's reference resolver when it has one,
and passes the completed host policy into `cron::open`. It retains empty
defaults, daemon-name allowlist, authorized absolute dotenv roots, and the
resolver; `cron::open` does not recreate or overwrite that policy. Final
composition does not weaken it. `with_hook_environment` applies for
every `cron` build; `with_monty_host` is compiled and applied only when the
generated `monty` feature enables `sc-cron/monty`.

`Observability` creates `Arc<Logger>` once and derives `shared` by Arc
coercion; it never clones `Logger`. `main` injects clones of the shared handle,
keeps scheduler state alive for daemon lifetime, drains and joins scheduler
workers and runtime work, drops every scheduler/runtime/shared-handle owner,
then consumes `Observability` with `shutdown(self)`. No shared logger handle
remains when the underlying logger shuts down; `shutdown(self)` destructures
the owner, drops its internal `shared` Arc clone, and then invokes shutdown on
the retained original logger.

Generated agent-facing failures use the established exact
`CODE cause — recovery` form:

| Failure | Code | Recovery |
|---|---|---|
| logger build | `DAEMON.OBSERVABILITY` | fix logging configuration or sink paths and retry |
| scheduler open | `DAEMON.CRON_OPEN` | fix cron configuration or database path and retry |
| logger shutdown | `DAEMON.OBSERVABILITY_SHUTDOWN` | inspect sink health and retry shutdown |

Each row includes its secret-free cause. The generated path forbids bare
`Display`/`Debug`, `anyhow`, and `Box<dyn Error>` reporting. Primary/secondary
precedence is fixed. Existing config, transport, and runtime failures retain
their exact primary code, cause, and recovery strings; composition does not
rewrite them. A shutdown failure remains secondary whenever one of those
operations already failed:

One centralized `format_agent_error(code, cause, recovery)` function owns the
exact `format!("{code} {cause} — {recovery}")` layout. Callers select one of
the three literal codes above and do not duplicate the layout.

| Primary operation | Shutdown result | Returned/reporting result |
|---|---|---|
| logger build fails | unreachable | `DAEMON.OBSERVABILITY` primary |
| setup or scheduler open fails | succeeds or fails | setup/`DAEMON.CRON_OPEN` remains primary; shutdown failure is secondary |
| daemon run fails | succeeds or fails | daemon error remains primary; shutdown failure is secondary |
| daemon run succeeds | fails | `DAEMON.OBSERVABILITY_SHUTDOWN` becomes primary process failure |
| daemon run succeeds | succeeds | success |

## Required work and deliverables

1. REQ-RUN-0004 / REQ-RUN-0309: change generated `Observability` to Arc ownership and expose only the
   types-level shared handle to library consumers.
2. REQ-RT-0001 / REQ-RT-0004: inject that handle into `DaemonBuilder` for every layout.
3. REQ-CRON-0003 / REQ-CRON-0005: under the b-16 `cron` feature, use b-13's
   `Scheduler::open_with_logger`, which internally constructs the concrete
   logger-aware `SystemJitter`, to open the scheduler
   database below the instance root, apply configured limits, attach the same handle, and retain
   the scheduler. Map config `0` or `null` to unlimited (`None`) and every
   other accepted positive value to `ScheduleLimits` at scheduler open;
   sc-cron never loads config. Feature-off behavior is unchanged.
   REQ-CRON-0019 / NFR-CRON-0002 / ADR-RUN-0016: then apply b-17's
   prebuilt `HookEnvironmentHost`, preserving empty defaults and configured
   allowlists/resolver without reintroducing daemon environment inheritance in
   Python, Monty, or compose children. Generated `main` owns construction and
   resolver injection; `cron::open` accepts the completed policy as an argument.
4. NFR-RUN-0005: add a generated-project smoke test that polls with a deadline, sends a
   health request, enables cron, and parses JSONL. It must observe
   `http.server/request.*` plus the production
   `cron.scheduler/logger.attached` event with one service identity.
5. ADR-RUN-0007 / ADR-RUN-0011 / NFR-RUN-0002 / NFR-RUN-0009: prove the fixed reachable result/shutdown precedence and that
   logger shutdown happens after both library users drain and join, every
   shared handle is dropped, and `shutdown(self)` consumes the owner. Add no
   production sleep/`block_on` or `std::sync::Mutex` guard across `.await`.
   Do not add a
   post-construction production-logger fault injector: b-14 and b-15 already
   own rejecting-emitter contract tests for emission failure, while this
   sprint owns only the real production JSONL proof.
6. REQ-RUN-0503 / REQ-RUN-0701: make the PR template workflow create one
   matrix entry for every `wizard/fixtures/*.json` file. The `sqlite-mcp`
   entry must execute exactly `just new` -> generated `just setup` ->
   `just lint` -> `just test` -> `just build-cli` -> `just build-daemon`, in
   that order, with no generated source edit between steps. Contract tests
   compare the fixture set with the matrix entries and preserve a generated
   source-tree snapshot across the entire no-edits sequence.
7. ADR-RUN-0002 / REQ-RUN-0002 / REQ-RUN-0003 / REQ-RUN-0702: extend the existing sc-publish consumer input for the
   `0.1.1` release. Preserve publish order `sc-config` (1), `sc-transport` (2),
   `sc-command` (3), `sc-runtime` (4), then publish `sc-cron` (5);
   `sc-runtime-db-sqlite` remains unpublished. Advance the current rejected
   pin to a later qualified exact kit revision with no `docs/sprints/**`.
   `test_pinned_publish_kit_inventory` must enumerate installer destination
   paths, reject path traversal and package-unsafe destinations, and fail
   closed on any `docs/sprints/**` destination. It must not read sprint
   frontmatter or couple the product installer inventory to `owned_paths`.
   From a clean checkout, fetch
   the repository and exact revision in `release/sc-publish-pin.toml`, detach
   at `FETCH_HEAD`, bootstrap the pinned renderer, run the real installer,
   and commit every exact installed output produced by that install. Require a
   clean repeat `--dry-run` and git drift check. Add the release-mode generated
   contract preflight and document that
   a human release owner with crates.io credentials must authorize and observe
   the generated release workflow. This repository neither publishes nor
   claims authority over the upstream observability `1.5.1` pair.
8. REQ-RUN-0702: after the five internal `0.1.1` crates are published, render
   with `scripts/new_project.py --release`, assert there is no
   `[patch.crates-io]` or path override, and build/test the generated daemon
   against the registry with `cron` and `monty`. The pre-publication source
   contract lives in
   `GeneratedObservabilityIntegration.test_release_contract_sources`; the
   registry-only proof lives in
   `GeneratedObservabilityIntegration.test_release_mode_uses_registry`.
9. REQ-RT-0005 / NFR-RUN-0010 / ADR-RT-0002 / ADR-TRN-0006: on an isolated ephemeral CI runner only,
   launch the real generated daemon binary twice with a fresh temporary
   instance root, deliver SIGINT in one case and SIGTERM in the other, and
   assert bounded drain/join, process exit, lock/socket cleanup, and logger
   shutdown. A dedicated `ubuntu-latest` GitHub-hosted job, and no local
   recipe, sets `SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST=1`. The test fails
   closed unless that marker, `GITHUB_ACTIONS=true`, and
   `RUNNER_ENVIRONMENT=github-hosted` are all present; setting the dedicated
   marker alone on a developer host cannot enable it.
10. NFR-RUN-0008: keep every generated production Rust file below the existing
   1000-source-line gate; the ID governs only line counts.

## Acceptance criteria

1. REQ-RUN-0309 / REQ-RUN-0004: `test_observability` passes only when config
   -> log root -> logger -> shared injection -> run -> shutdown order is
   explicit and observed.
2. REQ-RT-0001 / REQ-RT-0004 / NFR-RT-0001 / NFR-RT-0002 /
   NFR-RT-0003 / NFR-RT-0004: `test_observability` passes only when the
   rendered daemon injects through unchanged builder typestate and writes
   correlated canonical HTTP events to its configured JSONL file.
3. REQ-CRON-0005 / ADR-RUN-0014: `test_schedule_limit_mapping` and the full
   `test_observability` suite pass only when the real generated scheduler
   writes the real cron attachment event and maps `0`/`null`/positive limits
   exactly, without a fake emitter in this integration proof.
   REQ-CRON-0019 / NFR-CRON-0002 / ADR-RUN-0016:
   `test_cron_environment` reruns against final `cron::open` composition and
   carries a recording resolver through generated `main`'s builder and the real
   `cron::open` argument, proving typed policy, empty defaults, and redaction
   wiring survive observability integration.
4. ADR-RUN-0007 / NFR-RUN-0002 / NFR-RUN-0009: b-14/b-15
   rejecting-emitter contract tests remain the authoritative
   immediate, best-effort proof; `test_observability` passes only for reachable
   generated-process result/shutdown precedence and production JSONL output.
5. REQ-RUN-0005: `cargo tree` for the generated daemon and b-13's direct-tree
   checks pass only when workspace libraries exclude the full observability
   runtime and the generated daemon remains its sole composition root.
6. REQ-RUN-0002 / REQ-RUN-0003 / REQ-RUN-0503 / REQ-RUN-0701 /
   REQ-RUN-0702: the current bad pin has been replaced by a qualified exact
   later revision; its package inventory contains no `docs/sprints/**`. The
   `test_pinned_publish_kit_inventory`, the real install/dry-run/drift block,
   consumer preflight, and the registry-only test pass. Together they prove
   the exact pinned kit synchronizes its complete package-safe destination
   inventory, the human-authorized publish follows orders 1 through 5,
   and a subsequent `--release` generated build resolves all internal crates
   from the registry with no checkout patch. The PR workflow has exactly one
   matrix entry per answers fixture, and the sqlite-mcp entry closes the
   exact generate -> setup -> lint -> test -> build-cli -> build-daemon
   no-source-edits sequence.
7. REQ-RT-0005 / NFR-RUN-0010 / ADR-RT-0002 / ADR-TRN-0006:
   `test_signal_shutdown_isolated_ci` passes in the dedicated GitHub-hosted
   ephemeral job for both real SIGINT and SIGTERM paths with
   temporary instance roots, released instance/endpoint locks, removed
   sockets, and no leaked process or shared logger handle.
8. NFR-RUN-0008: `just lint gates` exits zero with the line-count gate clean
   for all generated production Rust sources; no behavior claim is attached.

## Required validation

```sh
python3 -m unittest tests.integration.generation.test_observability
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_contract_sources
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_pinned_publish_kit_inventory
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_schedule_limit_mapping
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_agent_error_inventory
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_sqlite_mcp_no_edits_sequence
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_pr_fixture_matrix_one_entry_per_fixture
python3 -m unittest tests.unit.generation.test_cron_environment
SC_RUNTIME_FIXTURE_ROOT="$(mktemp -d)"
just new "$SC_RUNTIME_FIXTURE_ROOT/sqlite-mcp" --var-file wizard/fixtures/sqlite-mcp.json
(cd "$SC_RUNTIME_FIXTURE_ROOT/sqlite-mcp" && just setup && just lint && just test && just build-cli && just build-daemon)
test "${GITHUB_ACTIONS:-}" = true
test "${RUNNER_ENVIRONMENT:-}" = github-hosted
test "${SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST:-}" = 1
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_signal_shutdown_isolated_ci
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
cargo check --manifest-path generated/test-observability/Cargo.toml -p widget-app-daemon --features cron
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
