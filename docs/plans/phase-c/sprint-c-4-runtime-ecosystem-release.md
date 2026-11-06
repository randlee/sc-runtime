---
id: c-4
phase: c
status: in_progress
closure_type: integration
target_boundary: published-runtime-template-consumer
track: release
wave: 3
branch: sprint/c-4-runtime-ecosystem-release
worktree: sprint/c-4-runtime-ecosystem-release
requirements:
  - REQ-CRON-0005
  - REQ-CMD-0001
  - REQ-CMD-0002
  - NFR-CMD-0001
  - REQ-RUN-0002
  - REQ-RUN-0003
  - REQ-RUN-0301
  - REQ-RUN-0503
  - REQ-RUN-0701
  - REQ-RUN-0702
  - REQ-CRON-0003
  - REQ-CRON-0019
  - REQ-CRON-0020
  - REQ-CRON-0021
  - REQ-CRNC-0001
  - REQ-CRNC-0002
  - NFR-CRNC-0001
  - REQ-RUN-0332
  - NFR-RUN-0001
  - NFR-RUN-0010
  - NFR-RUN-0002
  - NFR-RUN-0005
  - NFR-RUN-0009
  - NFR-RUN-0003
  - NFR-RUN-0004
  - NFR-RUN-0006
  - NFR-RUN-0007
  - NFR-RUN-0008
adrs:
  - ADR-RUN-0004
  - ADR-RUN-0005
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0011
  - ADR-RUN-0015
  - ADR-RUN-0201
  - ADR-RUN-0302
  - ADR-RUN-0002
  - ADR-RUN-0017
  - ADR-RUN-0203
  - ADR-RUN-0401
  - ADR-RUN-0402
  - ADR-RUN-0013
  - ADR-RUN-0016
  - ADR-RUN-0018
  - ADR-RUN-0019
  - ADR-RUN-0202
  - ADR-CRNC-0001
  - ADR-RUN-0001
  - ADR-RUN-0003
  - ADR-RUN-0009
  - ADR-RUN-0008
  - ADR-RUN-0303
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CMD-0003
  - ADR-RT-0001
  - ADR-RT-0002
  - ADR-RT-0003
  - ADR-RT-0004
  - ADR-RT-0005
owned_paths:
  - docs/requirements.md
  - template/scripts/consumer_checks.py
  - template/scripts/dependency_policy.py
  - tests/unit/generation/test_daemon_scratch_policy.py
  - docs/development/sc-publish.md
  - docs/consumer-preflight-and-docli.md
  - docs/getting-started.md
  - tests/integration/generation/test_observability.py
  - tests/unit/generation/test_cron_environment.py
  - tests/unit/generation/test_monty_example.py
  - tests/unit/generation/test_liquid.py
  - template/crates/cli/Cargo.toml
  - scripts/run_generation_tests.py
  - template/crates/daemon/Cargo.toml
  - install.py
  - README.sc-publish.md
  - .github/workflows/*release*.yml
  - .github/workflows/*publish*.yml
  - .github/actions/
  - .github/scripts/
  - .claude/agents/publisher.md
  - .claude/skills/prerelease/
  - .codex/agents/publisher.md
  - .codex/skills/prerelease/
  - .cursor/commands/cursor-publish.md
  - .cursor/skills/prerelease/
  - release/
  - tests/integration/release/test_published_runtime_ecosystem.py
  - tests/integration/release/test_standard_cron_commands.py
  - template/crates/daemon/src/cron_commands.rs
  - template/crates/daemon/src/composition.rs
  - template/crates/daemon/src/lib.rs
  - template/crates/daemon/src/main.rs
  - template/crates/daemon/src/routes.rs
  - template/crates/daemon/src/mcp.rs
  - template/crates/cli/src/lib.rs
  - template/crates/cli/src/main.rs
  - template/crates/service/src/cron_commands.rs
  - template/crates/service/src/lib.rs
must_follow: [c-2, c-3, c-5, c-6]
parallel_safe: []
recommended_agent: solar
recommended_model: sol
vertical_rationale: Final selected template binding consumes c-3 generic UI projections, c-5 real Scheduler/CronOperations and c-6 StandardCronExecutor. The small composition roots, prepublication behavior, authorized release and registry-only rerun close that one feature/release track; no contract stub can supply those real artifacts.
---

# Sprint c-4 — runtime ecosystem authorized release

The final generated fixture consumes c-2’s full bootstrap profile; c-2 is
therefore a named dependency before fixture execution. Parent-authorized daemon
manifest ownership is limited to forwarding scheduler/cron and optional Monty
features, with no dependency or version changes.

Dependency execution: consume c-3's ecosystem/template implementation, c-5's
real tracked query/lifecycle and c-6's concrete shared command handler by
merging each parent development push before every development/fix
round. Parent QA is not a start barrier; the parent PR merges before this
PR completes, for all three parents. Publication retains the release
authorization gate below. These real implementations cannot be contract stubs.

## Goal

Publish `sc-runtime 0.2.0` only after c-5's dedicated compatibility sprint
and c-3/c-6 implementation are complete and release is authorized. Then prove
a newly generated consumer resolves it from crates.io without a checkout patch.

## Deliverables

1. **Before any publication**, require completed c-3 runtime/ecosystem and
   fixture work, c-5's dedicated cron compatibility work, and c-6's command
   adapter work, with their required validation. Complete this sprint's
   checkout-patched composition, schema, shared-UI, lifecycle and custom-store
   proofs specified below. All must pass before publication; completed parent
   work alone does not replace c-4's own prepublication proofs.
2. **Authorized publication:** after stage 1, use the existing authorization
   and publication workflow for `sc-runtime 0.2.0` and every public package
   frozen by c-1, all six at the shared 0.2.0 version. Change only canonical
   `release/sc-publish-consumer-input.json` and the qualified exact
   `release/sc-publish-pin.toml`, then rerun its installer. Include its actual
   destination inventory in this sprint's release-owned paths; any additional
   generated destination requires an explicit ownership update before writing.
   Do not hand-edit vendored workflows/scripts. Use the existing pinned kit
   to publish all six public crates at 0.2.0 in dependency order, including
   config/transport and the adapter; command precedes cron/runtime and cron
   precedes adapter. Check each exact version's registry visibility before its
   dependent. The internal publish=false helper stays private. `v0.2.0` names
   the shared-version public release according to c-1's tag contract;
   do not infer credentials or publish authority from this plan.
3. **Registry-only rerun:** after authorized dependency-ordered publication
   and registry visibility, generate the `sqlite-mcp` release-mode
   fixture with no `[patch.crates-io]` entry and run its standard no-edit
   sequence: `just setup`, `just lint`, `just test`, `just build-cli`, and
   `just build-daemon`.
   Run locked `cargo metadata --format-version 1 --all-features` **only in the
   no-patch generated fixture**. Starting at the rendered `daemon` resolve ID,
   follow normal dependency edges and enforce exactly one reachable
   `(name, version, source)` tuple each for `rmcp`, `utoipa`, and
   `utoipa-axum`. Record all reachable public workspace packages at exactly 0.2.0, all from
   crates.io rather than path, git or patch sources. Check visibility of all
   six public packages and ensure no stale command RMCP type reaches the daemon.

Store the authorization reference, publication reference, visibility evidence,
no-patch assertion and package-identity receipt in the existing release PR/check
output. No credentials or publish authority are granted by this plan.

## Acceptance criteria

- In the existing real selected composition fixture, MCP tools/list input
  schemas and OpenAPI request/data schemas agree with c-1's same adapter DTO
  and strict version/tag/required/target wire fixtures; typed canonical
  Envelope<CronResponse>/OpError response schemas match serialized success/
  failure/nullability cases. The actual schema
  features use the frozen compatible identities; canonical Envelope/OpError
  wrapping stays unchanged. Exercise positive/negative structural examples
  and real commands, repeating packaged/registry composition; no generic
  object schema or duplicate DTO can stand in for the typed public contract.

REQ-RUN-0503 is a supporting registry-fixture constraint here; c-2 alone owns
its phase requirement root. Retain all existing real registry proofs.

Additional sole feature-level integration deliverables and criteria, rooted
at REQ-RUN-0332, REQ-CRNC-0001/0002 and REQ-CRON-0003/0020/0021:

- Before publication, implement actual selected template root binding in the
  listed module/call sites: Arc<Scheduler<...>> implementing CronOperations,
  real ApplicationTargetBindings and StandardCronExecutor feed c-3's generic
  projections. This takes the explicit later-wave source handoff from c-3;
  no same-wave sibling writes those paths or supplies a fake production factory.
  Compose c-5 scheduler/SQLite operations and c-6 executor in the release fixture
  venue with checkout patches. Actual CLI, HTTP and selected MCP use the same
  canonical JSON start/end/list command contract across Rust, Python and explicit
  Monty targets. The same request yields equivalent normalized data/error
  across actual UI projections; transport wrappers may differ.
  Exercise real schedule mutation/execution and generation conflicts.
- Follow c-6's `crates/sc-cron-commands/README.md` selected-default recipe and
  exercise c-5's `crates/sc-cron/README.md` public custom-store/schema example
  and application-owned-field proof in the existing
  generated and registry-only consumer fixtures using documented
  options/public APIs, without manual framework-source edits. Exercise selected
  cron/commands, bundled store, target registration, explicit worker startup,
  real commands and stop/physical join/outcomes; record the recipe revision.
  Verify these guides map ScheduleStore replacement to consumer schema,
  validation, generation fencing and persistence truth; CronOperations to
  application operations/shutdown, cancellation and physical joins; and
  CronExecutor to command authorization, dispatch and error mapping. Overrides
  remain optional and consumer-owned, with accurate errors/outcomes under their
  selected policy. Release acceptance requires both named C-5/C-6 guides to
  document a concrete supported public-API integration path, correct use of the
  core libraries, and the consumer override responsibilities listed above.
  C-5/C-6 retain guide authorship; c-4 executes and verifies those documented
  paths and responsibility assertions through its existing consumer proof.
- Prove default-off generation has no standard commands/adapter worker,
  DTO-only CLI graphs remain server/cron/SQLite/Monty free, scheduler/Monty
  enable only approved edges, disabled MCP stays disabled, and core runtime
  still excludes cron/adapter. Standard list covers active/ended/all with
  bounded stable continuation; no mocked list_active-only substitute.
- In this same real mixed-UI fixture, fill c-1's default64 shared Scheduler
  command capacity while store work is held. Calls through multiple executors
  and CLI/HTTP/MCP projections cannot multiply the limit; the next operation
  receives the fixed typed unavailable refusal before registered/submitted work.
  Drop admitted transport waiters and prove slots remain held through physical
  completion. Race deadline/cancellation and stop while full; independent
  cleanup/stop still progress and accepted work is physically joined with
  accurate outcomes. Exercise a valid finite configuration override; default-off
  generation and custom executor policy remain independent. No new receipt or
  fixture venue is needed.
- Packaged lifecycle/command proof consumes c-1/c-5 typed admission/persistence
  outcomes. A cutoff prohibits new admission; an already admitted late commit
  or unknown result is never reported as rollback, not-persisted or bounded
  success. Preserve independent stop/physical ownership, no replay of uncertain
  mutations and no effects after actual release. Registry proof repeats this
  public contract; it does not claim a universal fsync/physical-2T guarantee.
- Publish all six public packages at the exact shared 0.2.0 version:
  command first, then cron, then cron-commands; runtime after command. Check
  each version's registry visibility before dependent publication. Extend the
  qualified canonical all-crates kit input/pin, not hand-edited vendored outputs.
- After visibility, repeat the same no-source-edit opt-in UI/target/graph
  suite and public logging-facade consumer against exact registry artifacts
  with no path/git/patch sources. Earlier prepublication proof is not this
  registry result. Actual consumer-product adoption retains its own QA.

The release fixture's lint smoke uses the already existing bootstrap surface;
c-2's additional analyzer receipts are independently owned and are not a
registry-release start prerequisite. If a future fixture recipe actually
consumes a changed c-2 artifact, update the named dependency before execution.

Root: `req:REQ-RUN-0002`, `req:REQ-RUN-0702`,
`adr:ADR-RUN-0203`.

- An authorized release record identifies `sc-runtime 0.2.0`; the release
  workflow and per-public-package registry visibility checks succeed in
  dependency order. Canonical installer repeat dry-run is clean, version
  checks accept the shared version of all six public crates; only the internal
  publish=false helper is excluded, without bypassing validation.
- The generated no-patch fixture resolves that crates.io package and completes
  the full standard command sequence without a generated-source edit.
- The release fixture’s locked metadata closure has one identity each for
  `rmcp`, `utoipa`, and `utoipa-axum`; any duplicate reaching the rendered
  daemon fails the test. Both `sc-command` and `sc-runtime` are exactly 0.2.0
  from crates.io, along with every other reachable public workspace package
  at the same 0.2.0 version; no checkout supplies a public release package.
- The fixture uses ADR-RUN-0401/0402's established noninteractive generation
  driver and values-file contract; it introduces no alternate renderer.

## Required validation

- Existing release preflight and publication checks, plus the no-patch fixture
  and exact package-identity assertions after visibility.
- Root `just lint && just test` on the composed integration head.

## This sprint does not close

- Dependency/API migration (`c-3`), consumer lint (`c-2`), cron core (`c-5`)
  or adapter boundary (`c-6`) implementation. This sprint owns their real
  command/release composition and the explicitly listed final template binding
  paths, not the cron/adapter implementation source.

## Prepublication development checkpoint — 2026-11-05

The selected application root now shares one native Scheduler and
StandardCronExecutor across HTTP, MCP and CLI. Public hooks cover registered
Python/Monty targets, Rust jobs, a consumer-owned Scheduler/store/clock,
command policy, finite capacity, independent stop, custom physical cleanup and
checked consumer shutdown budgets. Python registration stores a live script
path; its private working directory is separate from the script location.
Monty registration persists source. These hooks do not schedule jobs by
construction or transfer custom persistence/cleanup obligations to the core.

The official generated consumer proof passed eight real Rust tests: all three
command projections across all three target kinds; real deterministic target
execution; typed schemas and pagination; native default64 shared capacity and
retained slots after dropped waiters; committed-before-cutoff persistence truth;
finite override validation; consumer executor/physical cleanup; and the default
file-opening factory. The existing C5 application-schema example also passed
its ten macOS tests inside that generated consumer. Exact package closure,
DTO-only CLI and default-off/core-runtime graphs passed. The exact pinned
publishing kit's installer repeat dry-run was clean.

The untouched-consumer sequence passed setup with sc-lint0.8.4, full lint,
workspace tests and release CLI build. Its daemon compiled, but the existing
whole-tree tempfile guard failed because production Python cron uses tempfile.
REQ-RUN-0503 criterion6 and REQ-RUN-0701 command6/criterion1 explicitly forbid
that transitive package; a requirement decision remains pending. The guard and
requirements remain unchanged. The resumed generated proof also passed the
consumer-budget API with finite cutoffs and expired cutoffs while admitted work
remained held; all eight generated Rust tests and ten unchanged application-schema
tests passed. C3's diagnostic-redaction fix was merge-forwarded without product
conflicts, and its real generated projection regression passed on this composition.
Final composed `just lint && just test`, remaining parent fixes/QA/CI,
publication and registry-only acceptance remain open. This is a development
checkpoint, not full sprint qualification or closure.

C4 QA1 source corrections preserve the SDK's primary startup/serve failure
when cron cleanup also fails, emit safe secondary canonical diagnostics, and
retain native cron codes, reconciliation advice and typed persistence facts.
The isolated logging consumer now targets 0.2.0, and registry qualification
rejects every noncanonical crates.io source. Focused generated regressions,
strict selected-daemon Clippy, the existing eight composition tests and ten
unchanged C5 schema/ownership tests passed. Reintroducing cleanup-first
precedence caused the intended regression assertion to fail. Canonical C5/C6
owner/error corrections are consumed. Independent review remains required.
The tempfile decision, upstream publication-visibility adoption, final composed
gates, controlled-source benchmark qualification, publication and registry-only
proofs remain open; no release authority or closure is inferred.

The CI follow-up adopts merged sc-publish PR163 at exact source
`1a494e7adb067d93c57807f443ac136a8775c7d8` through the canonical installer.
All six public crate declarations require positive exact-version visibility
before dependent publication in both normal and retry paths. The full old-pin
delta was reviewed; all 89 unique installer destinations match canonical source
and repeat installation reports no drift. Installed visibility tests cover
positive predecessor checks and stopping on absent or indeterminate results.
The native startup regression now checks the safe native INVALID_SPEC contract;
startup errors are not duplicated as cleanup failures. MCP feature dispatch lives
in Rust behind one Liquid registration, and Monty assertions retain optional
forwarding/default-off semantics and the production-only dependency boundary.
The tempfile guard and all required final release gates remain unchanged.


### Approved Python scratch policy — 2026-11-06

The human-approved clarification now permits only the qualified direct
`sc-cron -> tempfile` dependency for private Python working directories.
The daemon guard projects Cargo's active normal tree onto exact metadata
package identities; workspace dev-feature unification cannot create a false
normal edge. Direct daemon, alternate/transitive tempfile ancestry (including a
second path to the same package) and every normal tempdir remain rejected.
Script locations and daemon instance/database path rules are unchanged.

All eighteen graph regressions and the real selected/default-off graph checks
passed. The fresh untouched generated consumer completed `setup`, `lint`,
`test`, `build-cli` and `build-daemon`; strict workspace Clippy and formatting
also passed. These results supersede the earlier scratch-policy blocker;
independent QA, final composed/source-qualified benchmark, publication and
registry-only acceptance remain open. The approval is recorded in
`reviews/authority-approvals/phase-c-stage-policy-and-calibration-approved-2026-11-06.json`.
