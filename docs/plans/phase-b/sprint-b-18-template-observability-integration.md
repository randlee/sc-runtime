---
id: b-18
phase: b
status: complete
closure_type: integration
target_boundary: generated-daemon-observability
track: observability
wave: 10
branch: codex/b-18-publishing-kit
worktree: /Volumes/Extreme Pro/github/sc-runtime-worktrees/codex/b-18-publishing-kit
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0003
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0301
  - REQ-RUN-0309
  - REQ-RUN-0503
  - REQ-RUN-0701
  - REQ-RUN-0702
  - REQ-RT-0001
  - REQ-RT-0002
  - REQ-RT-0004
  - REQ-RT-0005
  - NFR-RT-0001
  - NFR-RT-0002
  - NFR-RT-0003
  - NFR-RT-0004
  - REQ-CRON-0003
  - REQ-CRON-0004
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
  - ADR-RUN-0001
  - ADR-RUN-0003
  - ADR-RUN-0005
  - ADR-RUN-0008
  - ADR-RUN-0009
  - ADR-RUN-0015
  - ADR-RUN-0303
  - ADR-RUN-0401
  - ADR-RUN-0402
  - ADR-RT-0004
  - ADR-RT-0005
owned_paths:
  - scripts/check_observability_release.py
  - dev/observability-logger-adapter/Cargo.toml
  - dev/observability-logger-adapter/Cargo.lock
  - dev/observability-logger-adapter/src/lib.rs
  - dev/observability-logger-adapter/tests/real_logger.rs
  - Justfile
  - scripts/new_project.py
  - scripts/run_lifecycle_signal_test.py
  - template/config/default.json
  - template/config/cron.schema.json
  - template/crates/daemon/Cargo.toml
  - template/crates/daemon/src/lib.rs
  - template/crates/daemon/src/composition.rs
  - template/crates/daemon/src/agent_error.rs
  - template/crates/daemon/tests/observability.rs
  - template/crates/daemon/tests/cron_environment.rs
  - tests/unit/generation/test_development_observability.py
  - tests/unit/generation/test_monty_example.py
  - tests/unit/generation/test_cron_environment.py
  - tests/unit/generation/test_general_db_disable.py
  - tests/unit/generation/test_prefix_packages.py
  - tests/unit/generation/test_liquid.py
  - tests/integration/generation/snapshot_just.py
  - docs/sc-cron/monty-generated-example.md
  - docs/project-plan.md
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
  - crates/sc-runtime/src/builder.rs
  - crates/sc-runtime/src/lib.rs
  - crates/sc-runtime/src/tests.rs
  - crates/sc-runtime/src/tests/lifecycle.rs
  - crates/sc-runtime/README.md
  - docs/sc-runtime/architecture.md
  - docs/sc-runtime/requirements.md
  - crates/sc-cron/src/scheduler.rs
  - crates/sc-cron/src/worker.rs
  - crates/sc-cron/src/worker/worker_runs.rs
  - crates/sc-cron/src/environment/dotenv.rs
  - crates/sc-cron/src/environment/resolve.rs
  - crates/sc-cron/src/job.rs
  - crates/sc-cron/tests/worker.rs
  - crates/sc-cron/tests/worker/lifecycle.rs
  - crates/sc-cron/tests/observability_components.rs
  - crates/sc-cron/README.md
  - docs/sc-cron/architecture.md
  - tests/integration/generation/owned_process.py
  - tests/integration/generation/test_owned_process.py
  - tests/integration/generation/test_workflow_dispatch.py
  - .github/workflows/crates.yml
must_follow: [b-14, b-17]
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
---

# Sprint b-18 — generated daemon observability integration

## Goal

Make the generated daemon the sole full-observability composition root. It
builds one logger, shares the same underlying instance with runtime and cron,
and proves canonical HTTP and cron events in one JSONL sink. sc-runtime remains
a cargo-generate template plus separately published library crates.

## Owner timing amendment — 2026-10-30

The owner directed that Phase B finish before upstream `1.5.1` publication.
The governing timing clauses in [REQ-RUN-0702](../../requirements.md),
including Success Criteria 6–7, and [architecture](../../architecture.md)
(System shape and ADR-RUN-0004/0007) are amended with this plan to assign
production release acceptance to `run-h0h` after Phase B. Public interfaces,
version inventories, and every release safety predicate remain unchanged.
The timing amendment itself did not complete b-18. The sprint status records
implementation delivery; independent parent QA and CI remain required for
acceptance and do not follow from that metadata alone.

Phase completion requires the real development composition, shared ownership,
HTTP+cron JSONL, shutdown precedence, isolated-CI signal lifecycle, qualified
kit installation, and pre-publication source contracts below. It may use the
approved real registry `1.5.0` logger through the unpublished development
adapter. All eight acceptance criteria remain applicable with criterion 6
split explicitly between phase implementation and later release evidence.

Release epic `run-h0h` owns matching published `1.5.1` contracts and external
prerequisite `run-03z.7`, development facade/adapter/path removal, direct upstream
Arc coercion, the five internal `0.1.1` publications, registry-only generated
proof, and the same-source `v0.1.1` tag. Those are mandatory release conditions,
not Phase B completion conditions. The phase-end readiness review after fixes
must report these remaining release limits explicitly. The active release
guard continues to reject development resolution.

## Relations

`must_follow` b-14 and b-17 because this sprint consumes the runtime events
from b-14 and the completed cron environment/generated-daemon surface from
b-17. b-17 already follows b-15 and b-16, so cron events, injection, Cargo
features, and hook-environment wiring all arrive through that parent. Each
parent PR must merge into this branch before final runtime composition and before
every composition fix round. Kit preparation and qualification may progress
independently of those parent merges. There is no same-wave sibling.

## Consumed artifacts

- b-14: `DaemonBuilder::logger` and HTTP events.
- b-17: the daemon `cron` and `monty` Cargo features from b-16, the cron
  worker/due-run event taxonomy from b-15, and the shared Python/Monty hook
  environment contract. b-13 supplies `Scheduler::open_with_logger`,
  component propagation/events, and `cron.scheduler/logger.attached`. This
  sprint does not edit shared Cargo files. It also consumes the exact `0.1.1`
  registry pins and checkout-patch inventory that originated in b-16.
  All production release crates exclude sc-compose and ATM crate dependencies
  and Rust imports, enforced by boundary lint; test/benchmark references are
  permitted (ANC-031).

b-13 may deliver library development against the owner-authorized unpublished
`sc-observability-types` `1.5.1` facade in `dev/observability-types-shim`, which
reexports exact registry `1.5.0` values/errors and adds the fixed injection
trait, Noop/shared handle, and test-util doubles. Development applications
also use the unpublished `sc-observability-dev-adapter` `0.0.0` newtype in
`dev/observability-logger-adapter` to inject a real registry
`sc-observability` `1.5.0` `Arc<v2::Logger>`. It forwards the actual
`service_name`/`try_log` APIs with no fake queue or sinks; its tests exercise
real queue/sink, saturation, and shutdown behavior. The application retains
construction, flush, and shutdown ownership. Neither local package claims
to be the upstream `1.5.1` adapter. Before release acceptance, follow the
[removal procedure](../../../dev/observability-types-shim/README.md): remove
the `path` key from the root `sc-observability-types` workspace dependency
and the local adapter dependency, remove the development packages, and
resolve the real matching `sc-observability-types` /
`sc-observability` `1.5.1` registry pair, and prove the published Logger
adapter contract and generated JSONL behavior. Replace local newtype
construction with direct coercion of an Arc clone of the real Logger to
`SharedCanonicalLog` once the upstream trait implementation is available.
Release epic `run-h0h` owns the serial successor teardown paths and root
`Cargo.toml` / `Cargo.lock` transition after Phase B; b-18 preserves their
source contracts without performing teardown. Optimized development/benchmark
profiles remain
permitted; publication stays fenced by `publish = false` and the active
release guard. `scripts/check_observability_release.py` must reject a
development observability workspace path override, local adapter,
observability registry patch, or non-registry/wrong-version release
resolution; removing or disabling the guard is not a substitute for removing
the shim.

The former pin `16457e06ade6c0b003ff75addb7d694e9da41fe2` was not a valid
b-18 starting artifact because that kit package includes upstream sprint
history at `docs/sprints/fix-homebrew-formula-bundled-paths-tojson.md`.
The kit install required a later qualified exact revision with no
`docs/sprints/**` installer/package destination. This does not block independent
kit preparation or runtime composition after approved parent merges. Upstream
source may retain its own sprint history; source-checkout absence is not the
package-safety predicate. The installed qualified revision is `9677ab083e7dd3feb258235a342e321ca1b45d8d`. b-18 updated
`release/sc-publish-pin.toml`, installed only from that exact revision, and
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
    #[cfg(feature = "monty")] monty_hosts: sc_cron::MontyHostRegistry,
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
let scheduler = scheduler.with_monty_hosts(monty_hosts);

Ok(scheduler)
```

`Scheduler::open_with_logger` constructs
`SystemJitter::with_logger(logger.clone())` internally, so callers cannot
pair the store and `RunCoord` handle with a differently configured jitter.
`main` calls the application-owned production `composition::run` path. Its
`CronServices` builds `hook_environment` through b-17's typed config builder,
attaches the optional application-owned resolver, and passes that exact policy
and application registry into `cron::open` unchanged. Defaults choose no
built-in reference provider and an empty registry.
ANC-034/035: Python and Monty use identical schedule-environment settings and
resolution; daemon values enter only through names explicitly allowed by
daemon config. Load scheduled dotenv sources by absolute path only when
authorized by `cron.environment.allow_dotenv_files` or `allow_dotenv_roots`;
both default to empty. Preserve the application's resolver.
Each schedule's explicit Set and Clear operations survive;
intentional Set values, including secrets, remain plaintext SQLite schedule
configuration rather than being forbidden. `with_hook_environment` applies for
every `cron` build; `with_monty_hosts` is compiled and applied only when the
generated `monty` feature enables `sc-cron/monty`. The application supplies
that completed registry, empty by default or explicitly registering b-16's
compose/ATM adapters and its own handlers. `cron::open` forwards it unchanged:
applications add handlers without editing sc-cron, and both sc_compose_run and
atm_run support any CLI command/subcommand. Neither CLI is enabled implicitly.

`Observability` creates `Arc<Logger>` once and never clones `Logger`. During
development it derives `shared` through
`Arc::new(LoggerAdapter::new(Arc::clone(&logger)))`, using the approved real
registry `1.5.0` adapter. The final release replaces that expression with
direct upstream Arc coercion after the published `1.5.1` implementation exists.
`main` injects clones of the shared handle,
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

## Governing ADR applicability

The binding inventory contains 21 decisions: the existing ten, the ten QA1
trace corrections, and the explicit QA2 amendment ADR-RT-0005. Their concrete
scope is:

- ADR-RUN-0001 and ADR-RUN-0008 govern application-owned assembly and standard
  tools. The generated `composition` module supplies the resolver/registry
  extension seam; the SDK does not choose a provider or import application cron.
- ADR-RUN-0002, ADR-RUN-0003 and ADR-RUN-0303 govern published crate identities,
  the types-only library graph, daemon-only full logger, and default-off cron
  and Monty features. No compose/ATM release dependency is introduced.
- ADR-RUN-0004 and ADR-RUN-0007 govern the one shared logger, direct owner
  emission, structured code/message/cause/recovery, and final owner shutdown.
- ADR-RUN-0005 governs standard sc-lint/just infrastructure and routing the
  generated-feature workflow checks through the root recipes.
- ADR-RUN-0009 and ADR-TRN-0006 govern real daemon-process signals and endpoint
  cleanup only on isolated ephemeral CI; no local real-signal path is enabled.
- ADR-RUN-0011 governs async cleanup, owned blocking operations and child joins,
  without a synchronous guard across await or a production blocking bridge.
- ADR-RUN-0013, ADR-RUN-0014 and ADR-RUN-0016 govern cron ownership, mapped row
  limits, the unchanged prebuilt environment/resolver and host registry contract,
  beneath-instance database paths, shared snapshots and redaction.
- ADR-RUN-0015 remains the controlled-host benchmark gate for later authorized
  release acceptance. This sprint retains its preflight and claims no new
  benchmark campaign or release acceptance.
- ADR-RUN-0401 and ADR-RUN-0402 govern cargo-generate/sc-compose mechanics,
  explicit development generation, fixture admission and source-unchanged setup,
  lint, test and build sequences.
- ADR-RT-0005 explicitly amends ADR-RT-0001 and ADR-RT-0002 for the SDK cleanup
  extension while preserving their historical decisions: the
  ready builder consumes a general application cleanup future after server
  disposal, before stores/instance-lock/logger teardown, without importing cron.
  REQ-RT-0002 requires lock acquisition before the stores callback and retention
  through that cleanup; REQ-RT-0005 retains the graceful runtime signal path.
- ADR-RT-0004 governs B18's real in-process fixture proof, including local
  request/JSONL and lock-held cleanup regressions. It does not authorize local
  process-signal execution or replace the isolated daemon-process proof.

ADR-RT-0003 remains outside the phase plan's binding inventory: project Stores,
router and rmcp service still pass through unchanged, with no wrapper or field
inspection. Retaining the successful Stores owner through the new general
cleanup callback changes lifecycle ownership under ADR-RT-0005, amending
ADR-RT-0001/0002, not the
Stores/router/service pass-through contract. No referenced-crate ADR is copied
merely because that crate is consumed.

## Required work and deliverables

QA2 corrective deliverable: Active ADR-RT-0005 explicitly amends ADR-RT-0001
and ADR-RT-0002 for the implemented Ready-only cleanup boundary. Historical
decision text is retained with amendment references; production behavior and
existing lifecycle regressions remain unchanged. QA-002 updates the three
existing generation assertions to verify the workflow selectors and their exact
root recipe commands, plus the current small Liquid composition bodies.

1. ADR-RUN-0004 / REQ-RUN-0004 / REQ-RUN-0309: change generated `Observability` to Arc ownership and expose only the
   types-level shared handle to library consumers.
2. ADR-RT-0001 / REQ-RT-0001 / REQ-RT-0004: inject that handle into `DaemonBuilder` for every layout.
3. ADR-RUN-0013 / REQ-RUN-0301 / REQ-CRON-0003/0004/0005/0019 /
   NFR-CRON-0002 / ADR-RUN-0016: implement the fixed `cron::open` contract above.
   At scheduler open, map config `0` or `null` to unlimited (`None`) and other
   accepted positive values to `ScheduleLimits`; sc-cron never loads config.
   `main` constructs the prebuilt policy/resolver, injects it, and retains the
   scheduler. Feature-off behavior is unchanged.
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
   Phase B delivers the qualified install, consumer input, and pre-publication
   source/preflight contracts. Actual workflow dispatch/publication and its
   evidence belong to later release epic `run-h0h`. The publication evidence,
   registry-only proof, and `v0.1.1` tag bind to the same source commit; no
   manifest or source change is allowed between them.
8. REQ-RUN-0702: retain the following later-release acceptance in `run-h0h`;
   it does not gate Phase B implementation completion. After the five internal
   `0.1.1` crates are published, render
   with `scripts/new_project.py --release`, assert there is no
   `[patch.crates-io]` or path override, and build/test the generated daemon
   against the registry with `cron` and `monty`. The pre-publication source
   contract lives in
   `GeneratedObservabilityIntegration.test_release_contract_sources`; the
   registry-only proof lives in
   `GeneratedObservabilityIntegration.test_release_mode_uses_registry`.
   Before publication, remove the development workspace path overrides and
   local adapter, switch injection to direct upstream `Arc<Logger>` coercion,
   and replace the intentional registry `1.5.0` plus local `1.5.1`
   development resolution with
   real matching registry `1.5.1` releases in the root and generated release
   locks. The release guard must pass without weakening its predicate, and
   published adapter tests must prove immediate queue-full/disconnected
   rejection. Shim doubles or a local Logger substitute cannot satisfy this
   production proof. The local adapter's real `1.5.0` tests remain development
   evidence, not published `1.5.1` adapter evidence.
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
   explicit and observed. [roots: `req:REQ-RUN-0004`, `req:REQ-RUN-0309`]
2. REQ-RT-0001 / REQ-RT-0004 / NFR-RT-0001 / NFR-RT-0002 /
   NFR-RT-0003 / NFR-RT-0004: `test_observability` passes only when the
   rendered daemon injects through unchanged builder typestate and writes
   correlated canonical HTTP events to its configured JSONL file.
3. REQ-CRON-0004 / REQ-CRON-0005 / ADR-RUN-0014: `test_schedule_limit_mapping` and the full
   `test_observability` suite pass only when the real generated scheduler
   writes the real cron attachment event and maps `0`/`null`/positive limits
   exactly, without a fake emitter in this integration proof.
   REQ-CRON-0019 / NFR-CRON-0002 / ADR-RUN-0016:
   `test_observability` carries a recording resolver through generated
   `main`'s builder and the real `cron::open` argument, proving typed policy,
   empty administrator defaults, preserved Set/Clear and intentional plaintext
   Set values, authorized absolute dotenv loading without persisting resolved
   file values, and identical Python/Monty snapshots. Monty top-level getenv
   reads that snapshot during source initialization before hooks run.
   The same integration test registers and invokes an application-defined
   Monty closure through the real `cron::open` registry argument without
   changing sc-cron, proving custom handlers survive composition.
   Exact-file/root authorization and redaction also survive composition. b-17's
   `test_cron_environment` reruns unchanged as a regression root rather than
   becoming a b-18-owned test file. [roots: `req:REQ-CRON-0004`]
4. ADR-RUN-0007 / NFR-RUN-0002 / NFR-RUN-0009: b-14/b-15
   rejecting-emitter contract tests remain the authoritative
   immediate, best-effort proof; `test_observability` passes only for reachable
   generated-process result/shutdown precedence and production JSONL output. [roots: `req:NFR-RUN-0009`]
5. REQ-RUN-0005: `cargo tree` for the generated daemon and b-13's direct-tree
   checks pass only when workspace libraries exclude the full observability
   runtime and the generated daemon remains its sole composition root.
6. REQ-RUN-0002 / REQ-RUN-0003 / REQ-RUN-0503 / REQ-RUN-0701 /
   REQ-RUN-0702: `test_pinned_publish_kit_inventory`, real install/dry-run/drift,
   consumer preflight, and the registry-only test enforce deliverables 6–8:
   **Phase implementation:** qualified exact pin with complete package-safe
   installer inventory and no `docs/sprints/**` destination, real install and
   repeat dry-run/drift proof, consumer preflight source contracts preserving
   publish order 1–5, and one CI entry per fixture with the sqlite-mcp no-edits
   sequence. The source contract test remains
   `GeneratedObservabilityIntegration.test_release_contract_sources`.
   **Later release (`run-h0h`):** human-authorized publication in order 1–5,
   registry resolution without checkout patches, and `v0.1.1` at the
   publication/proof commit. This release gate also requires development
   path/adapter removal, direct upstream Arc coercion, real matching registry
   `1.5.1` root/generated lock resolution, the unchanged active release guard,
   and the published production Logger adapter's non-blocking proof. The
   registry-only test is required later; an unmet release prerequisite is
   recorded as deferred release work, never as a passing registry proof. [roots: `req:REQ-RUN-0002`, `req:REQ-RUN-0003`, `req:REQ-RUN-0503`, `req:REQ-RUN-0701`, `req:REQ-RUN-0702`]
7. REQ-RT-0002 (inherited constraint) / REQ-RT-0005 / NFR-RUN-0010 / ADR-RT-0002 / ADR-TRN-0006:
   `test_signal_shutdown_isolated_ci` passes in the dedicated GitHub-hosted
   ephemeral job for both real SIGINT and SIGTERM paths with
   temporary instance roots, released instance/endpoint locks, removed
   sockets, and no leaked process or shared logger handle. [roots: `req:NFR-RUN-0005`, `req:NFR-RUN-0010`, `req:REQ-RT-0005`]
8. NFR-RUN-0008: `just lint gates` exits zero with the line-count gate clean
   for all generated production Rust sources; no behavior claim is attached. [roots: `req:NFR-RUN-0008`]

## Required validation

Phase implementation validation uses the explicit development generation mode.
Select the repository or generated project's own `Justfile` explicitly so a
recipe cannot resolve from a parent checkout. The full suite must distinguish
unavailable later-release proof from executed development evidence. The signal
block runs only in the dedicated ephemeral CI job, never on a developer host.

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
just --justfile Justfile new "$SC_RUNTIME_FIXTURE_ROOT/sqlite-mcp" --var-file wizard/fixtures/sqlite-mcp.json --development-observability
(cd "$SC_RUNTIME_FIXTURE_ROOT/sqlite-mcp" && just --justfile Justfile setup && just --justfile Justfile lint && just --justfile Justfile test && just --justfile Justfile build-cli && just --justfile Justfile build-daemon)
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
# Enumerate the pinned installer's complete actual destination inventory,
# including all package/copy/render outputs; upstream source history may exist.
# Reject docs/sprints/**, traversal, absolute/duplicate destinations and unsafe
# symlinks; prove the inventory covers every output from the real install.
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_pinned_publish_kit_inventory
python3 .sc-publish-kit/plugins/sc-publish/.github/scripts/bootstrap_sc_compose.py --venv .sc-publish-venv
.sc-publish-venv/bin/python .sc-publish-kit/plugins/sc-publish/install.py --input release/sc-publish-consumer-input.json .
.sc-publish-venv/bin/python .sc-publish-kit/plugins/sc-publish/install.py --dry-run --input release/sc-publish-consumer-input.json .
git diff --check
git diff --exit-code
test -z "$(git status --porcelain)"
# Verify consumer preflight source wiring now; execution of release preflight
# is required later, because its active guard rejects this development bridge.
just --justfile Justfile lint
just --justfile Justfile test
just --justfile Justfile new generated/test-observability --var-file wizard/fixtures/sqlite-mcp.json --development-observability
cargo test --manifest-path generated/test-observability/Cargo.toml --workspace
cargo check --manifest-path generated/test-observability/Cargo.toml -p widget-app-daemon --features cron
cargo test --manifest-path generated/test-observability/Cargo.toml -p widget-app-daemon --features cron
cargo +1.96.1 check --manifest-path generated/test-observability/Cargo.toml -p widget-app-daemon --features monty
cargo tree --manifest-path generated/test-observability/Cargo.toml -p widget-app-daemon -e normal
just --justfile Justfile lint gates
```

Dedicated ephemeral CI validation only (the job sets the marker; local recipes
must not set it):

```sh
test "${GITHUB_ACTIONS:-}" = true
test "${RUNNER_ENVIRONMENT:-}" = github-hosted
test "${SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST:-}" = 1
just --justfile Justfile test lifecycle-signal
```

Deferred release validation (`run-h0h`), after matching upstream publication,
bridge removal, and authorized internal publication; these commands are not
claimed to pass during Phase B:

```sh
bash .github/scripts/run-consumer-preflight.sh
python3 scripts/check_observability_release.py
SC_RUNTIME_REGISTRY_PROOF=1 python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_mode_uses_registry
test "$(git rev-list -n 1 v0.1.1)" = "$(git rev-parse HEAD)"
```

The integration test parses every JSONL line as `LogEvent`, asserts both
targets and one service name, and uses an isolated temporary log root. The
install/drift block runs first from a clean sprint checkout; its cache and
virtual environment are ignored. The registry-only test runs only after the
human-authorized `0.1.1` publication. The release workflow generated from the
installed sc-publish kit remains the later release publication authority, so
this sprint
edits its consumer input and regenerates every listed kit-owned output rather
than hand-editing installed files.

## This sprint does not close

- OTLP defaults, library-owned sink lifecycle, a cron workload example, or
  observability injection into config/transport/command/store crates.


## Implementation scope clarification

The owner authorized necessary generation glue alongside the Rust composition:
`new_project.py` substitutes the approved real `1.5.0` adapter and adds its
normal daemon dependency only in explicit development generation. Canonical
release source retains direct upstream Arc coercion and has no adapter
dependency. The daemon's `async-trait` normal dependency is optional and enabled
only by `cron`; runtime fixture and temporary directory dependencies are test-only.
The two B17 generation assertions that named the former parameter/local feature
list were updated narrowly for the fixed B18 signature and optional worker
implementation dependency. Environment behavior remains covered by the B17
regression suite plus the generated Python/Monty resolver/registry proof.

The standard root `just test lifecycle-signal` selector shares the integration
suite's marker predicate and refuses a developer host before invoking the
real-signal test. No recipe enables any marker. Tests use production logger
ownership and the existing signal-free `DaemonFixture` for local HTTP/cron
JSONL proof; synthetic typed shutdown error values exercise result selection
without injecting faults into a running production logger.


The qualified kit exposes pre-publication consumer checks but no generic
post-publication consumer command hook. The actual registry-only proof must
therefore remain a mandatory explicit release-owner action in `run-h0h`, after
all five internal publications, using `SC_RUNTIME_REGISTRY_PROOF=1` as shown
above. Pre-publication consumer checks retain the static source contract and
strict dependency guard; they neither invoke nor skip a production registry
proof. Ordinary phase discovery skips the unavailable later proof. No current
registry success is claimed.
Cron workers start inside the runtime stores callback only after the runtime
holds `daemon.lock` and application stores open. Runtime invokes the general
cleanup callback before releasing that lock, so application drain cannot
overlap a replacement worker. A losing concurrent daemon cannot run schedules
before its instance-lock failure.


## Implementation delivered

The generated daemon owns one real Arc logger and one shared canonical handle.
Every layout injects it through the unchanged runtime builder. The optional
cron path maps config limits, preserves prebuilt environment policy/resolver
and the application Monty registry, and retains the scheduler throughout the
runtime. Worker startup occurs only inside the post-lock successful stores
callback. After runtime drain, the owner receives and joins the worker, drops
scheduler/shared consumers, and consumes observability shutdown. Centralized
agent formatting preserves primary config/transport/runtime errors while
reporting logger shutdown failures secondarily.

Generated Rust proofs cover typed production HTTP+cron JSONL with one service
and correlated HTTP request IDs, consuming handle release, real database
limits, early runtime failure before worker startup, all result/shutdown
precedence cases, and identical Python/Monty environment snapshots through the
application resolver/custom registry. The Python integration entrypoints
require actual Rust test execution, enforce the fixture matrix and a complete
source snapshot across the sqlite-mcp no-edits sequence, and provide the
three-marker isolated-GitHub-hosted signal lifecycle job. Real signals are
not local validation. The qualified publishing inventory remains all 85 exact
safe outputs from revision `9677ab083e7dd3feb258235a342e321ca1b45d8d`.

Root `Cargo.toml` and `Cargo.lock` were supplied by approved parent merges and
remain unchanged by this sprint. Their release transition belongs to
`run-h0h`. The final source is submitted for full maintainer gates, independent
QA, and CI; this implementation record does not claim those later review
results or the deferred production registry proof.


## QA1 corrective implementation

All ten published QA1 findings are implemented on the corrective descendant;
independent QA2 and current hosted CI remain acceptance gates. Application main
and the recording-resolver/real-logger proofs share `composition::run` and its
`CronServices::open_scheduler` production assembly. The application may supply
its resolver and completed registry; defaults choose neither a built-in provider
nor implicit CLI adapters. Cron database paths reject absolute/parent components
before creating the instance root and reject symlink traversal before opening a
file outside it. Schema and negative regressions match that contract.

Runtime and transport failures retain structured library code, message/cause
and original recovery without embedding Display's repeated code/recovery. The
reachable early runtime failure checks the entire resulting agent line.
Previously verified recurring schedule rows are disabled before the next
real-clock denied/redaction case; no timing retry or result filtering is used.

The generic SDK cleanup callback joins application workers while the singleton
lock and successful Stores owner remain alive. Worker shutdown joins active
blocking-store and child-run ownership even after its cooperative deadline;
a false deadline result does not detach native work. Private dotenv blocking
reads capture only file/root authorization, not a resolver that might retain
an injected logger. Canceled futures and noncooperative native code retain their
existing documented limits; no forced thread-preemption guarantee is claimed.

Workflow feature checks now use root just selectors. The common test command
owner cleans ordinary process groups within its pinned POSIX session, waits for
Windows job active-process count zero, and bounds direct-child reap/output.
Readiness witnesses cover timeout, failure, nested ownership and a normal child
creating a new process group. Windows runs all seven witnesses through an early
standard selector with an explicit nonzero-test-count guard before the existing
three environment roots. Deliberately detached new sessions remain outside this
MVP; cleanup does not hunt unrelated sessions or introduce a process framework.
The new helpers load under both package-based unittest and the repository's
standard top-level discovery shape.

Root Cargo manifests/lock, the actual development Logger adapter, qualified kit
revision and every later-publication predicate remain unchanged. No local real
daemon signals or premature registry proof form part of these corrective proofs.
