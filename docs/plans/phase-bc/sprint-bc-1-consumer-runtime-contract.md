---
id: bc-1
phase: bc
status: planned
closure_type: contract
target_boundary: phase-bc-consumer-runtime-contract
track: contract
wave: 1
branch: sprint/bc-1-consumer-runtime-contract
worktree: sprint/bc-1-consumer-runtime-contract
requirements:
  - REQ-RUN-0004
  - REQ-RUN-0307
  - REQ-RUN-0702
  - NFR-RUN-0006
  - NFR-RUN-0007
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0005
  - ADR-RUN-0202
  - ADR-RUN-0203
owned_paths:
  - docs/requirements.md
  - docs/architecture.md
  - .claude/project/quality-policy.md
  - examples/spike/phase-bc-contract/
must_follow: []
parallel_safe: []
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The consumer-profile exception, its executable evidence, the frozen runtime package-identity contract, and the externally authorized release handoff must be decided together before disjoint implementation branches can act. The temporary probe is recorded evidence, not a partial product path; durable regression coverage belongs to the later integration sprints.
---

# Sprint bc-1 — consumer/runtime compatibility contract

## Goal

Make the Phase BC contracts authoritative before implementation. This sprint
is a contract closure: it changes no runtime or template product code and owns
the only shared authoritative documents plus a temporary executable probe,
allowing `bc-2` and `bc-3` to proceed in parallel without overlapping paths.

## Deliverables

1. Amend REQ-RUN-0307 and NFR-RUN-0006 to pin the repository and generated
   consumer to **sc-lint 0.8.4**. Permit only the three declarative entries
   below in `template/sc-lint.toml`; they are sc-lint invocation wiring, not
   a copied policy. Continue to forbid `boundaries/`, clippy, rustfmt, deny,
   custom rules, and `just` modules.
2. Amend ADR-RUN-0005 to name that narrow exception, its preservation after
   `sc-lint init --just`, and the rule that the bootstrap receipt—not TOML
   inspection alone—proves all three analyzers ran. Amend the quality policy
   from 0.8.0 to 0.8.4 and reference the same authoritative exception.
3. Create and execute the temporary `examples/spike/phase-bc-contract/` probe.
   In temporary directories it installs/runs the released `sc-lint 0.8.4`
   binary against a minimal generated-consumer-shaped repository with no
   `boundaries/` path, asserts the exact three analyzer command receipts and
   zero exit status, and fails if a copied manifest, suppressed analyzer
   failure, or wrapper is required. It then runs `sc-lint init --just` twice
   and asserts that `db-prepare`, every declared analyzer entry, and ordinary,
   `default`, and `full` profile execution all remain intact. If 0.8.4 has a
   documented consumer-neutral mode, the probe uses and records its exact
   flags/configuration; otherwise this phase stops for a new sc-lint release
   rather than inventing a mode.
4. In the same test, recheck crates.io, create a temporary composition
   manifest for the newest stable candidates, and run locked `cargo metadata
   --format-version 1 --all-features`. Starting at the probe’s runtime/daemon
   resolve IDs, assert one reachable `(name, version, source)` tuple for each
   of `rmcp`, `utoipa`, and `utoipa-axum`. The probe must compile the ordinary
   `OpenApiRouter`/streamable-HTTP composition; otherwise it fails.
5. Amend ADR-RUN-0203 with the exact probe-selected versions and identity
   algorithm, replacing its obsolete 0.2/5.5.0/3.7 evidence matrix. `bc-3`
   consumes this frozen matrix and cannot select newer versions without a new
   ADR amendment and a repeat of this probe.
6. Define `bc-4` as the named authorized release handoff: only after an
   authorized `sc-runtime 0.2.0` publication and registry visibility may its
   no-patch fixture report the registry-consumer acceptance result.

## Acceptance criteria

Root: `req:REQ-RUN-0004`, `req:REQ-RUN-0307`, `req:REQ-RUN-0702`, `adr:ADR-RUN-0005`,
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

- The executable probe passes the no-manifest receipt and metadata reachability
  assertions plus the two-init preservation assertion before its selected
  versions are recorded. ADR-RUN-0203 names those exact versions and the
  metadata algorithm; it no longer leaves its old pins as active evidence.
- The probe output (release asset, exact commands, receipts, init-preservation
  result, metadata JSON digest, and selected versions) is attached to the
  `bc-1` PR and summarized in ADR-RUN-0005/0203. Before `bc-1` merges, its
  owner deletes `examples/spike/phase-bc-contract/`; `bc-2` and `bc-3` retain
  the durable generated-fixture and metadata regressions, respectively.
- The release handoff has a sole owner (`bc-4`) and its required authorization,
  registry visibility, no-patch fixture, and exact package-identity proof are
  unambiguous.

## Required validation

- `python3 examples/spike/phase-bc-contract/probe.py`, `just lint gates`, and
  `just test` pass before the binding documents are merged. The probe may use
  the registry only in this manually recorded pre-merge evidence run; it is
  deleted before merge and is never a network-dependent CI test.
- Documentation links and identifiers resolve; search confirms no remaining
  authoritative Phase BC baseline says `0.8.0` or permits generic `0.8.x`.
- The plan's `bc-1` amendments are reviewed before `bc-2`/`bc-3` begin.

## This sprint does not close

- Template configuration, bootstrap, runtime source, dependency manifests, or
  a publication. Those belong respectively to `bc-2`, `bc-3`, and `bc-4`.
