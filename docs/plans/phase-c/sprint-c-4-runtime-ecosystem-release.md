---
id: c-4
phase: c
status: planned
closure_type: integration
target_boundary: published-runtime-template-consumer
track: release
wave: 3
branch: sprint/c-4-runtime-ecosystem-release
worktree: sprint/c-4-runtime-ecosystem-release
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0003
  - REQ-RUN-0301
  - REQ-RUN-0503
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
adrs:
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
owned_paths:
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
must_follow: [c-3, c-5, c-6]
parallel_safe: []
recommended_agent: solar
recommended_model: sol
vertical_rationale: Final selected template binding consumes c-3 generic UI projections, c-5 real Scheduler/CronOperations and c-6 StandardCronExecutor. The small composition roots, prepublication behavior, authorized release and registry-only rerun close that one feature/release track; no contract stub can supply those real artifacts.
---

# Sprint c-4 — runtime ecosystem authorized release

Dependency execution: consume c-3's ecosystem/template implementation, c-5's
real tracked query/lifecycle and c-6's concrete shared command handler by
merging each parent development push before every development/fix
round. Parent QA is not a start barrier; the parent PR merges before this
PR completes, for all three parents. Publication retains the release
authorization gate below. These real implementations cannot be contract stubs.

## Goal

Close the production release contract left deliberately open by `c-3`:
publish the approved `sc-runtime 0.2.0` only with release authorization, then
prove a newly generated consumer resolves it from crates.io without a checkout
patch.

## Deliverables

1. Use the existing authorization and publication workflow for `sc-runtime
   0.2.0` and every affected package frozen by c-1. Change only canonical
   `release/sc-publish-consumer-input.json` and the qualified exact
   `release/sc-publish-pin.toml`, then rerun its installer. Include its actual
   destination inventory in this sprint's release-owned paths; any additional
   generated destination requires an explicit ownership update before writing.
   Do not hand-edit vendored workflows/scripts. Use the c-1 proven selective
   contract to publish sc-command 0.2.0 before dependent cron/runtime packages,
   checking each exact version's registry visibility before its dependent.
   Unaffected packages stay at 0.1.1 and are not republished. `v0.2.0` names
   the qualified affected-package release according to c-1's tag contract;
   do not infer credentials or publish authority from this plan.
2. Wait for registry visibility, then generate the `sqlite-mcp` release-mode
   fixture with no `[patch.crates-io]` entry and run its standard no-edit
   sequence: `just setup`, `just lint`, `just test`, `just build-cli`, and
   `just build-daemon`.
3. Run locked `cargo metadata --format-version 1 --all-features` **only in the
   no-patch generated fixture**. Starting at the rendered `daemon` resolve ID,
   follow normal dependency edges and enforce exactly one reachable
   `(name, version, source)` tuple each for `rmcp`, `utoipa`, and
   `utoipa-axum`. Record sc-command and sc-runtime at exactly 0.2.0 and any
   affected cron version frozen by c-1, all from crates.io rather than path,
   git or patch sources. Assert unaffected packages retain their approved
   0.1.1 versions and no stale command RMCP type reaches the daemon.
4. Store the authorization reference, publication reference, visibility
   evidence, no-patch assertion, and package-identity receipt in the release
   PR/check output.

## Acceptance criteria

Additional sole feature-level integration deliverables and criteria, rooted
at REQ-RUN-0332, REQ-CRNC-0001/0002 and REQ-CRON-0003/0020/0021:

- Before publication, implement actual selected template root binding in the
  listed module/call sites: Arc<Scheduler<...>> implementing CronOperations,
  real ApplicationTargetBindings and StandardCronExecutor feed c-3's generic
  projections. This takes the explicit later-wave source handoff from c-3;
  no same-wave sibling writes those paths or supplies a fake production factory.
  Compose c-5 scheduler/SQLite operations and c-6 executor in the release fixture
  venue with checkout patches. Exercise start/end/list for Rust, Python and
  explicit Monty through CLI JSON, HTTP and selected MCP. Same request yields
  equivalent normalized data/error across all projections; actual schedule
  mutation/execution and generation conflicts occur, not just DTO serialization.
- Prove default-off generation has no standard commands/adapter worker,
  DTO-only CLI graphs remain server/cron/SQLite/Monty free, scheduler/Monty
  enable only approved edges, disabled MCP stays disabled, and core runtime
  still excludes cron/adapter. Standard list covers active/ended/all with
  bounded stable continuation; no mocked list_active-only substitute.
- Publish the exact frozen affected closure including cron and new adapter:
  command first, then cron, then cron-commands; runtime after command. Check
  each version's registry visibility before dependent publication. Extend the
  qualified selective kit input/pin, not hand-edited vendored outputs.
- After visibility, repeat the same no-source-edit opt-in UI/target/graph
  suite and public logging-facade consumer against exact registry artifacts
  with no path/git/patch sources. Earlier prepublication proof is not this
  registry result. Actual consumer-product adoption retains its own QA.

The release fixture's lint smoke uses the already existing bootstrap surface;
c-2's additional analyzer receipts are independently owned and are not a
registry-release start prerequisite. If a future fixture recipe actually
consumes a changed c-2 artifact, update the named dependency before execution.

Root: `req:REQ-RUN-0002`, `req:REQ-RUN-0503`, `req:REQ-RUN-0702`,
`adr:ADR-RUN-0203`.

- An authorized release record identifies `sc-runtime 0.2.0`; the release
  workflow and per-affected-package registry visibility checks succeed in
  dependency order. Canonical installer repeat dry-run is clean, version
  checks accept the proven mixed package set, and unchanged packages are
  excluded from publication without bypassing validation.
- The generated no-patch fixture resolves that crates.io package and completes
  the full standard command sequence without a generated-source edit.
- The release fixture’s locked metadata closure has one identity each for
  `rmcp`, `utoipa`, and `utoipa-axum`; any duplicate reaching the rendered
  daemon fails the test. Both `sc-command` and `sc-runtime` are exactly 0.2.0
  from crates.io, plus the affected cron version fixed by c-1 when required;
  no local checkout supplies an affected package.
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
