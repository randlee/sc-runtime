---
id: c-1
phase: c
status: in_progress
closure_type: contract
target_boundary: phase-c-consumer-runtime-contract
track: contract
wave: 1
branch: sprint/c-1-consumer-runtime-contract
worktree: sprint/c-1-consumer-runtime-contract
requirements:
  - NFR-RT-0001
  - REQ-CRON-0001
  - NFR-CRON-0001
  - REQ-RUN-0004
  - REQ-RUN-0002
  - REQ-RUN-0301
  - REQ-RUN-0503
  - REQ-RUN-0307
  - REQ-RUN-0702
  - NFR-RUN-0006
  - NFR-RUN-0007
  - REQ-CMD-0005
  - NFR-CMD-0001
  - NFR-CMD-0002
  - REQ-CRON-0002
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
  - REQ-CRON-0019
  - REQ-CRON-0020
  - REQ-CRON-0021
  - NFR-CRON-0002
  - REQ-CRNC-0001
  - REQ-CRNC-0002
  - NFR-CRNC-0001
  - REQ-RUN-0332
  - REQ-RUN-0401
  - REQ-RUN-0402
  - REQ-RUN-0403
  - REQ-RUN-0001
  - REQ-RUN-0005
  - REQ-RT-0005
  - REQ-RT-0001
  - REQ-RT-0006
  - REQ-CMD-0001
  - REQ-CMD-0002
  - REQ-CMD-0003
  - REQ-CMD-0004
  - REQ-CMD-0006
  - NFR-CMD-0003
  - REQ-RT-0002
  - REQ-RT-0003
  - REQ-RT-0004
  - REQ-RT-0007
  - REQ-RT-0008
  - NFR-RT-0002
  - NFR-RT-0003
  - NFR-RT-0004
  - NFR-RUN-0009
  - NFR-RUN-0002
  - NFR-RUN-0005
  - NFR-RUN-0001
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0005
  - ADR-RUN-0017
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0303
  - ADR-RUN-0003
  - ADR-RUN-0013
  - ADR-RUN-0014
  - ADR-RUN-0016
  - ADR-RUN-0018
  - ADR-RUN-0019
  - ADR-RUN-0401
  - ADR-RUN-0402
  - ADR-CRNC-0001
  - ADR-RT-0001
  - ADR-RT-0004
  - ADR-RT-0005
  - ADR-RT-0002
  - ADR-RT-0003
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CMD-0003
owned_paths:
  - template/hooks/drop-template-exclude.rhai
  - crates/sc-command/src/observability/tests.rs
  - docs/requirements.md
  - docs/architecture.md
  - .claude/project/quality-policy.md
  - docs/sc-cron/requirements.md
  - docs/sc-cron/architecture.md
  - docs/sc-command/requirements.md
  - docs/sc-command/architecture.md
  - docs/sc-runtime/requirements.md
  - docs/sc-runtime/architecture.md
  - docs/sc-cron-commands/
  - docs/plans/phase-c/compatibility-contract.md
  - boundaries/sc-cron/
  - boundaries/sc-command/
  - boundaries/sc-runtime/
  - boundaries/sc-cron-commands/
  - boundaries/README.md
  - crates/sc-cron/src/
  - crates/sc-command/Cargo.toml
  - crates/sc-command/src/envelope.rs
  - crates/sc-command/src/error.rs
  - crates/sc-command/src/lib.rs
  - crates/sc-cron/Cargo.toml
  - crates/sc-cron-commands/
  - tests/contracts/phase-c/
  - Cargo.toml
  - Cargo.lock
  - template/cargo-generate.toml
  - template/config/default.json
  - wizard/answers.schema.json
  - wizard/pages/wizard-answers.js
  - wizard/fixtures/
  - tests/fixtures/refused/
  - template/Cargo.toml
  - template/workspace.template.toml
  - template/crates/daemon/Cargo.toml
  - template/crates/cli/Cargo.toml
  - template/crates/service/Cargo.toml
  - examples/spike/phase-c-contract/
must_follow: []
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The consumer-profile exception, its executable evidence, the frozen runtime package-identity contract, and the externally authorized release handoff must be decided together before disjoint implementation branches can act. The temporary probe is recorded evidence, not a partial product path; durable regression coverage belongs to the later integration sprints.
---

# Sprint c-1 — consumer/runtime compatibility contract

## Goal

Make the Phase C contracts authoritative and compileable before implementation.
This contract closure owns shared documents, public signatures/legacy-safe
defaults, initial adapter skeleton/manifest membership, generator declarations,
contract double and temporary probe. It does not implement production cron or
adapter behavior. Its baseline permits c-2/c-3/c-5/c-6 parallel development.

## Deliverables

1. Amend REQ-RUN-0307 and NFR-RUN-0006 plus the current operational lint
   clauses of REQ-RUN-0004 and REQ-RUN-0503 to pin the repository and generated
   consumer to **sc-lint 0.8.4**; preserve historical version observations.
   Permit only the three declarative entries
   below in `template/sc-lint.toml`; they are sc-lint invocation wiring.
   Define a fresh-install default boundary policy from the configured
   application crate graph (ADR-RUN-0303): include only selected crates and
   allowed edges; omit disabled store/MCP components. After setup this policy
   belongs to the consumer and subsequent setup/upgrade preserves it.
   Forbid copying maintainer manifests, clippy, rustfmt, deny, custom rules,
   and `just` modules. This is the user-approved fresh-install policy
   exception to the existing no-lint-knowledge requirements.
2. Amend ADR-RUN-0005 to name that narrow exception, its preservation after
   `sc-lint init --just`, and the rule that the bootstrap receipt—not TOML
   inspection alone—proves all three analyzers ran. Amend the quality policy
   from 0.8.0 to 0.8.4 and reference the same authoritative exception.
3. Create and execute the temporary `examples/spike/phase-c-contract/` probe.
   In temporary directories it installs/runs the released `sc-lint 0.8.4`
   binary against a minimal generated-consumer-shaped repository with the
   configuration-derived fresh boundary policy, asserts the exact three analyzer command receipts and
   zero exit status, and fails if a copied maintainer manifest, suppressed analyzer
   failure, or wrapper is required. It then runs `sc-lint init --just` twice
   and asserts that `db-prepare`, every declared analyzer entry, and ordinary,
   `default`, and `full` profile execution all remain intact, including a
   consumer edit to the generated policy. Record the supported 0.8.4 policy
   schema, selected-crate/edge mapping and initialization behavior in
   ADR-RUN-0005. If supported configuration cannot express it, report the
   upstream capability gap rather than suppressing the analyzer.
4. Query crates.io at probe execution and date its latest released `rmcp` AND
   `utoipa` results. Record both exact versions in ADR-RUN-0203's runtime
   dependency matrix. Create the composition manifest with matching
   `utoipa-axum` and run locked `cargo metadata --format-version 1 --all-features`.
   From runtime/daemon resolve IDs, require one reachable `(name, version,
   source)` tuple each for `rmcp`, `utoipa`, and `utoipa-axum`. Both locked
   rmcp/utoipa versions must equal their dated query's exact latest releases;
   a mismatch fails. Compile ordinary `OpenApiRouter`/streamable-HTTP
   composition; failure blocks closure, with no older-release fallback.
5. Freeze those exact registry-selected versions and the identity algorithm
   in ADR-RUN-0203, superseding the prior version matrix. `c-3`
   consumes this frozen matrix and cannot select newer versions without a new
   ADR amendment and a repeat of this probe. Freeze `sc-command 0.2.0`
   alongside `sc-runtime 0.2.0`: the command crate exposes the changed RMCP
   types. Include the command-dependent cron manifest and shared logging
   trait in the composition probe; freeze cron 0.2.0 for the new public
   compatibility contract and adapter 0.2.0 as part of the closure. Leave
   unrelated config/transport packages at 0.1.1. Record exact per-package
   versions and command-before-cron/runtime publication order. Amend
   ADR-RUN-0017's temporary command dependency version and the cron
   requirements/architecture when that affected edge changes, preserving
   defaults-off/no-server semantics and deferred upstream logging cleanup.
   Amend REQ-CMD-0005's prior RMCP type identity and NFR-CMD-0001's
   independently binding RMCP minor-series allowlist to the frozen matrix
   in `docs/sc-command/requirements.md`, retaining its conversion, error,
   serialization and server-feature semantics, optional server gating,
   defaults-off behavior and all remaining dependency restrictions. c-3
   consumes both amended requirements with that frozen minor series.
6. Define `c-4` as the named authorized release handoff: only after an
   authorized `sc-runtime 0.2.0` publication and registry visibility may its
   no-patch fixture report the registry-consumer acceptance result.
7. Extend the temporary probe to qualify the canonical sc-publish selective
   release contract for that affected closure (REQ-RUN-0702). Test actual
   version verification, literal-base/publish=false handling, ordered publish
   plan, and per-package registry lookups using the pinned kit's existing
   dry-run/preflight facilities. Current code checks every declared crate
   against the workspace/tag version, so disabling publication alone is not
   proof. Record supported inputs, affected-package versions, unchanged
   package treatment, and `v0.2.0` tag semantics in the release ADR amendment.
   If the current pin cannot support it, own an upstream sc-publish change
   request and qualify a released exact pin before freezing the contract.
   A demonstrated upstream gap leaves c-1 incomplete; no hand-written
   workflow bypass or blanket version bump is permitted.

Additional contract deliverable under the framework charter, REQ-CRON-0020/0021
and REQ-CRNC-0002: freeze the default/override guide in compatibility-contract.md.
Consumers may replace the bundled store/schema through ScheduleStore, override
scheduler operations and lifecycle/error policy through CronOperations, or
replace command execution through CronExecutor. Consumers own cancellation,
physical joins, target authorization and accurate persistence/error reporting
for their overrides. Distinguish advertised custom cooperative guarantees and
scope forwarding from the caller-owned legacy path; preserve default safety.
C-5 owns the custom-store README/example proof, c-6 the adapter/default recipe,
and c-4 real recipe composition. Use existing paths; no new framework or API
without a demonstrated missing hook.

A consumer may omit the entire standard cron command set
(`cron_commands=false`, default). Selection requires cron; CLI, HTTP and enabled
MCP then share the same versioned CronRequest JSON contract: `cron.start`,
`cron.end`, `cron.list`. The same start/end/list schedule-management verbs apply
to Rust, Python and optional Monty handler-kind selections; options select the
execution target. C-1 owns prompt/schema/default/answers and optional manifests
once; c-3 supplies generic projections, c-4 actual root binding. Independently
disabled MCP stays disabled.

Amend NFR-RUN-0001 to permit selected defaults-only sc-cron-commands DTOs directly
or through api-types, keeping that edge free of scheduler/server/SQLite/Tokio/
Monty and preserving existing forbidden edges and isolated CLI proofs.
Freeze the bounded default command intake
section of compatibility-contract.md: default64, validated finite pre-start
configuration, shared Scheduler ownership, immediate typed refusal, no hidden
wait queue, physical slot retention after waiter cancellation and independent
stop/cleanup. C-5 implements the core gate, c-6 maps its error, c-4 proves real
mixed-UI saturation. This is a default capability, not a mandatory mechanism for
arbitrary consumer overrides. Existing NFR-CMD/RT and ADR-CMD/RT constraints
in frontmatter remain binding throughout these narrow amendments.

## Acceptance criteria

- REQ-CRNC-0001/NFR-CRNC-0001, ADR-CRNC-0001 and ADR-RUN-0203: supply the
  compileable sole adapter DTO family and canonical command Envelope<T>/OpError/
  ErrorKind with owner-local JsonSchema/ToSchema
  implementations under independent optional schema feature, optional
  schemars/Utoipa identities frozen by the existing compatibility probe and
  exact manifest/boundary declarations. Adapter/schema forwards command/schema;
  command schema manually matches custom Envelope serialization and all five
  OpError wire fields, including nullability/success/failure alternatives, with
  no wire changes or copied model. C-1 owns only the narrow declaration modules/
  manifests listed above before c-3 migrates their engine/source in its later
  handoff. Real Parameters<CronRequest> and Envelope<CronResponse>,
  schema_for! and Utoipa schema generation compile against those identities.
  Schema fixtures preserve version/tag/required/target structure; runtime-only
  checks remain explicit. Defaults stay lean, with no schema/server/scheduler
  activation. This declaration prerequisite lands before c-3/c-6; neither a
  generated orphan impl, second DTO, generic schema nor c-6 stub is accepted.

- The contract identifies default guarantees and transferred responsibilities
  for each supported replacement seam. The c-5 custom-field consumer proof and
  c-6 public executor composition test have concrete expected outcomes, and
  c-4 consumes their existing README recipe. Custom schema extension is allowed
  without changing bundled schema or adding product fields to shared DTOs.

Additional approved compatibility deliverables (REQ-CRON-0020/0021,
REQ-CRNC-0001/0002, NFR-CRNC-0001, REQ-RUN-0332, REQ-RT-0005/0006):

- Reconcile NFR-RT-0001's closed public-method table and exact rustdoc criterion
  with exactly stores/routes/mcp/logger and Ready-only run/run_with_cleanup/
  run_with_cleanup_and_bound. Amend REQ-RT-0001 and ADR-RT-0001/0005 consistently.
  This includes existing cleanup and the bound-receipt alternative only;
  preserve fixed startup/typestate, no registry/macro/wrapper or extra engine.
- Freeze the explicitly human-approved commit admission/outcome semantics in
  compatibility-contract.md, REQ-CRON-0020 and ADR-RUN-0018. Final ordinary G/
  cleanup K admission in bundled SQLite uses owned commit_hook and a short live stop-state/class/
  generation/real-time decision; release controller lock before COMMIT/native
  I/O. Qualify the hook through supported rusqlite0.39 hooks API without claiming
  universal physical fsync/2T preemption. Typed TransactionOutcome replaces
  cleanup_persisted bool: known committed-late, rollback and indeterminate are
  distinct. Freeze no-replay/bounded-authorized-readback and owned-release rules.
  Specify causal precheck/hook/stop-first/admission-first tests with actual
  persisted generation state after physical join; both explicit/implicit writes.

- Freeze [compatibility-contract.md](compatibility-contract.md) as exact public
  types/signatures, query limits/error inventory, absolute G/K semantics,
  real stop clock, restricted cleanup and physical ownership. Merge additive
  compileable declarations/legacy-safe unsupported defaults and the adapter
  CronExecutor/CronOperations/ApplicationTargetBindings signatures and sole
  CronRequest struct/operation DTO. Canonical dev-only doubles live at
  `crates/sc-cron-commands/tests/support/operation_double.rs` and
  `tests/contracts/phase-c/executor_double.rs`;
  declare both in corresponding core/adapter boundary manifests. C-6 imports
  the operation module via package-local #[path], including packaged tests;
  c-3's driver
  copies executor source only into isolated generated tests/support, never src.
  Provide the real context probe under existing non-default cron test-utils,
  enabled only by the adapter dev edge. No required unimplemented trait
  method, production stub or duplicate consumer protocol may leak downstream.
- Establish the new adapter manifest/skeleton, shared workspace membership and
  initial lock baseline, public facade and allowed-dependent manifests.
  Default DTO-only graph excludes cron/SQLite/Tokio/Monty/server; scheduler is
  optional. Existing runtime→cron/adapter and command→cron/adapter stay forbidden.
  Same nominal logging trait, bundled schema/generation/environment behavior preserved.
- Enable rusqlite 0.39's `hooks` feature alongside existing `bundled` in
  `crates/sc-cron/Cargo.toml`, and establish its Cargo.lock baseline before
  wave 2. `progress_handler` is hooks-gated; this is a known c-5 prerequisite,
  not a conditional discovery. Qualify interrupt, busy/progress callbacks and
  cancellation facility availability on that baseline. Preserve rusqlite
  0.39.x. C-1 owns this feature/lock change first; c-3 subsequently owns
  ecosystem version/path edits in the same manifest and must retain those
  features. Do not remove the manifest from c-3 wholesale. C-5 writes only
  source/tests and compiles against the committed c-1 feature baseline.
- Freeze sc-cron and new sc-cron-commands at 0.2.0 in the affected package
  matrix, order command→cron→adapter and command→runtime, qualify selective kit
  treatment including the new crate, and preserve unrelated 0.1.1 packages.
  Later ecosystem version/lock migration remains exclusively c-3 because its
  dependency/API update must compile atomically; initial skeleton membership
  in c-1 is an earlier baseline handoff, not concurrent manifest ownership.

Root: `boundary:phase-c-consumer-runtime-contract`, `adr:ADR-RUN-0005`,
`adr:ADR-RUN-0202`, `adr:ADR-RUN-0203`.

- The three documents have one consistent 0.8.4 baseline and explicitly
  delimit the consumer-profile exception.
- Current operational clauses of REQ-RUN-0004/0307/0503 and NFR-RUN-0006
  agree on 0.8.4; historical evidence remains intact. REQ-CMD-0005 names
  the frozen RMCP identity while preserving its conversion/error semantics.
- The amended REQ-RUN-0702 identifies `sc-runtime 0.2.0`, its tag/version
  semantics, the no-patch registry fixture, and its relationship to the prior
  0.1.x release contract. The probe preserves ADR-RUN-0002's versioned-crate
  template boundary and ADR-RUN-0202's one-router REST/OpenAPI/MCP shape.
- The following contract is reproduced verbatim or semantically equivalently
  in the authoritative requirements/ADR text:

  ```toml
  [[tool.sc-lint.lint]]
  name = "sc-boundary"
  command = ["sc-lint", "lint", "sc-boundary", "--root", ".", "--config", "sc-lint.toml"]
  # Equivalent entries invoke sc-portability and sc-runtime.
  [tool.sc-lint.lint_profiles]
  default = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]
  full = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]
  ```

- The executable probe passes the fresh-policy receipt and metadata reachability
  assertions plus the two-init preservation assertion before its selected
  versions are recorded. ADR-RUN-0203 names those exact versions and the
  metadata algorithm; it no longer leaves its old pins as active evidence.
- The probe output (release asset, exact commands, receipts, init-preservation
  result, and selected versions) is attached to the
  `c-1` PR and summarized in ADR-RUN-0005/0203. Before `c-1` merges, its
  owner deletes `examples/spike/phase-c-contract/`; `c-2` and `c-3` retain
  the durable generated-fixture and metadata regressions, respectively.
- The release handoff has a sole owner (`c-4`) and its required authorization,
  registry visibility, no-patch fixture, and exact package-identity proof are
  unambiguous.
- The affected command/runtime/cron dependency closure and selective kit
  contract are demonstrated by the temporary probe, including a negative
  stale-command-version case. ADR-RUN-0203 names every affected release,
  publication order, tag/version lookup semantics, and qualified exact kit
  pin; unsupported selective publication blocks this contract's closure.
- ADR-RUN-0005 states the default policy mapping for each selected crate and
  edge, identifies fresh generation as its consumer and the observed missing
  consumer boundary policy as its purpose, and defines the handoff: after
  setup the generated policy is consumer-owned and later setup preserves it.

## Required validation

- `python3 examples/spike/phase-c-contract/probe.py`, `just lint gates`, and
  `just test` pass before the binding documents are merged. The probe may use
  the registry only in this manually recorded pre-merge evidence run; it is
  deleted before merge and is never a network-dependent CI test.
- Documentation links and identifiers resolve; search confirms no remaining
  authoritative Phase C baseline says `0.8.0` or permits generic `0.8.x`.
- Push the contract amendments for `c-2`/`c-3`/`c-5`/`c-6` to merge-forward before each
  development/fix round. Parent QA does not block their start; `c-1`'s PR
  merges before any child PR completes. Contract tests, workspace build and
  configured boundary checks also pass for the additive skeleton and defaults.

## This sprint does not close

- Production cron/adapter implementation (c-5/c-6), lint bootstrap (c-2),
  runtime/template composition and atomic ecosystem migration (c-3), or actual
  cross-UI composition and publication (c-4). Contract declarations are not
  claims of those behaviors.


C1 initial declaration ownership clarification (parent-approved 2026-11-04):
`template/hooks/drop-template-exclude.rhai` rejects cron_commands without cron
before output mutation; C3 owns subsequent generator implementation. The
existing sc-command observability unit-test unused import is removed only to
run feature-contract clippy, preserving all test meaning and lint enforcement.


Temporary user-approved waiver (2026-11-04, [sc-runtime#117](https://github.com/randlee/sc-runtime/issues/117), upstream
[sc-lint#849](https://github.com/randlee/sc-lint/issues/849)): waive only successful repeated `sc-lint init --just`
after SDK/consumer customization. Released0.8.4 returns exit3
`CLI.SC_LINT_INTEGRATION_CONFLICT` and preserves the customized configuration,
Justfile, `db-prepare` and boundary policy byte-for-byte. Record that actual
result explicitly; never report init PASS, hide its error, force overwrite,
delete consumer edits or add a workaround wrapper. This known result does not
block current C1/C2 acceptance. Issue117 owns restoration of the successful
repeat-init criterion after an approved upstream solution. Shared exact0.8.4,
all three real analyzers in ordinary/default/full profiles, causal failure
proofs, configuration-derived fresh boundary policy and consumer ownership/
preservation, and usable public setup/lint/test/upgrade remain binding. No
upstream source fix, release or new pin is authorized by this waiver.
