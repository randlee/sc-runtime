---
id: c-1
phase: c
status: planned
closure_type: contract
target_boundary: phase-c-consumer-runtime-contract
track: contract
wave: 1
branch: sprint/c-1-consumer-runtime-contract
worktree: sprint/c-1-consumer-runtime-contract
requirements:
  - REQ-CRON-0001
  - NFR-CRON-0001
  - REQ-RUN-0004
  - REQ-RUN-0307
  - REQ-RUN-0702
  - NFR-RUN-0006
  - NFR-RUN-0007
  - REQ-CMD-0005
  - NFR-CMD-0001
  - NFR-CMD-0002
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0005
  - ADR-RUN-0017
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0303
owned_paths:
  - docs/requirements.md
  - docs/architecture.md
  - .claude/project/quality-policy.md
  - docs/sc-cron/requirements.md
  - docs/sc-cron/architecture.md
  - examples/spike/phase-c-contract/
must_follow: []
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The consumer-profile exception, its executable evidence, the frozen runtime package-identity contract, and the externally authorized release handoff must be decided together before disjoint implementation branches can act. The temporary probe is recorded evidence, not a partial product path; durable regression coverage belongs to the later integration sprints.
---

# Sprint c-1 — consumer/runtime compatibility contract

## Goal

Make the Phase C contracts authoritative before implementation. This sprint
is a contract closure: it changes no runtime or template product code and owns
the only shared authoritative documents plus a temporary executable probe,
allowing `c-2` and `c-3` to proceed in parallel without overlapping paths.

## Deliverables

1. Amend REQ-RUN-0307 and NFR-RUN-0006 to pin the repository and generated
   consumer to **sc-lint 0.8.4**. Permit only the three declarative entries
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
4. In the same test, recheck crates.io, create a temporary composition
   manifest for the newest stable candidates, and run locked `cargo metadata
   --format-version 1 --all-features`. Starting at the probe’s runtime/daemon
   resolve IDs, assert one reachable `(name, version, source)` tuple for each
   of `rmcp`, `utoipa`, and `utoipa-axum`. The probe must compile the ordinary
   `OpenApiRouter`/streamable-HTTP composition; otherwise it fails.
5. Amend ADR-RUN-0203 with the exact probe-selected versions and identity
   algorithm, replacing its obsolete 0.2/5.5.0/3.7 evidence matrix. `c-3`
   consumes this frozen matrix and cannot select newer versions without a new
   ADR amendment and a repeat of this probe. Freeze `sc-command 0.2.0`
   alongside `sc-runtime 0.2.0`: the command crate exposes the changed RMCP
   types. Include the command-dependent cron manifest and shared logging
   trait in the composition probe; freeze a cron 0.2.0 release if its command
   requirement must change to preserve that public trait identity. Leave
   unrelated config/transport packages at 0.1.1. Record exact per-package
   versions and command-before-cron/runtime publication order. Amend
   ADR-RUN-0017's temporary command dependency version and the cron
   requirements/architecture when that affected edge changes, preserving
   defaults-off/no-server semantics and deferred upstream logging cleanup.
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

## Acceptance criteria

Root: `boundary:phase-c-consumer-runtime-contract`, `adr:ADR-RUN-0005`,
`adr:ADR-RUN-0202`, `adr:ADR-RUN-0203`.

- The three documents have one consistent 0.8.4 baseline and explicitly
  delimit the consumer-profile exception.
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
  result, metadata JSON digest, and selected versions) is attached to the
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
- Push the contract amendments for `c-2`/`c-3` to merge-forward before each
  development/fix round. Parent QA does not block their start; `c-1`'s PR
  merges before either child PR completes.

## This sprint does not close

- Template configuration, bootstrap, runtime source, dependency manifests, or
  a publication. Those belong respectively to `c-2`, `c-3`, and `c-4`.
