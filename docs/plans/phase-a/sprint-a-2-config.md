---
id: a-2
phase: a
status: complete
closure_type: boundary
target_boundary: sc-config
track: core
wave: 0
branch: sprint/a-2-config
worktree: sprint/a-2-config
recommended_agent: arch-ctm
recommended_model: terra
must_follow: []
parallel_safe: [a-1, a-3, a-4]
requirements:
  - REQ-CFG-0001
  - REQ-CFG-0002
  - REQ-CFG-0003
  - REQ-CFG-0004
  - NFR-CFG-0001
  - NFR-CFG-0002
  - NFR-CFG-0003
  - REQ-RUN-0002
  - REQ-RUN-0005
  - NFR-RUN-0003
  - NFR-RUN-0004
  - NFR-RUN-0005
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0002
  - ADR-CFG-0001
  - ADR-CFG-0002
  - ADR-CFG-0003
  - ADR-RUN-0003
  - ADR-RUN-0004
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0008
  - ADR-RUN-0010
owned_paths:
  - "crates/sc-config/**"
  - "boundaries/sc-config/**"
  - "docs/sc-config/requirements.md"
  - "docs/sc-config/architecture.md"
---

# Sprint a-2 — sc-config

## Outcome and closure

Create one independently usable synchronous configuration crate. Its public
surface is the smallest contract needed to load typed JSON configuration with
environment overrides; it does not prescribe generated application structure.

## Decision prerequisites

Resolve environment ordering, **invalid `AppName` rejection at construction**
(not pass-through `&str`), non-Unicode values, public constructor signatures,
and `ConfigError` variants. Publish an **error inventory** table (failure
mode, stable code, cause, recovery) in the crate ADR before the facade
freezes (RBP-F002, RBP-F005). Record only decisions necessary to make the
crate coherent. Do not freeze the `sc_config::AppName` constructor until
a-1 has made ADR-RUN-0010 Active in `docs/architecture.md`. This sprint
does not write that ADR. The wait is the gate bead
`triage:a-2-appname-freeze` in `.sprints/a/structure.ttl`. That bead
follows a-1. It does not add a-1 to this sprint's `must_follow` and it
does not move this sprint off wave 0. Do not write
`boundaries/sc-config/sc-config.toml` or mark the leaf lint gate done until
gate bead `triage:a-2-boundary-evidence` is open. That bead also follows
a-1. Its criterion is the recorded `sc-lint lint sc-boundary` command,
including the negative fixture outcomes, and the REQ-RUN-0005 slice this
manifest consumes. That manifest work is not parallel-safe.

## Public contract produced

This sprint owns the sc-config types and methods in the canonical
[minimal public contract handoff](phase-a-plan.md#minimal-public-contract-handoff):
`sc_config::AppName`, `load`, `Loader::new`, `Loader::load`, and
`ConfigError`. The newtype uses the handoff validation predicate.
Construction tests load `tests/fixtures/app-name-vectors.json`. This sprint
does not copy that file. It is not
`sc_transport::AppName`. There is no `sc_runtime::AppName`. The
crate-local requirement and ADR sources record any remaining signature
decisions, including the `&str` application-name parameters those sources
still show. This sprint amends `docs/sc-config/architecture.md` and
`docs/sc-config/requirements.md` only. It does not edit `docs/architecture.md`
or `docs/requirements.md`, and it does not create a second copy of the
signature contract.

## Dependencies and parallel safety

- `must_follow` is empty. `sc-config` consumes no spike signature. Its
  dependencies are `serde` and `serde_json`.
- `parallel_safe` with `a-1`, `a-3`, and `a-4` for work that does not
  consume a-1 evidence. Owned paths and public boundaries are disjoint.
  Wave 0 places this sprint beside a-1. The constructor freeze and the
  boundary manifest are gated on a-1 and are not parallel-safe.

## Deliverables

1. **REQ-CFG-0001 / REQ-CFG-0002** — JSON defaults/local overrides and
   app-prefixed environment overrides with deterministic merge behavior.
2. **REQ-CFG-0003** — An explicit-input loader suitable for parallel tests,
   plus a convenience entry point for normal process use.
3. **REQ-CFG-0004 / NFR-CFG-0001** — Typed, source-preserving errors and no
   reachable panic path for malformed external input.
4. **NFR-CFG-0002 / NFR-CFG-0003** — Minimal dependencies, no async/reload
   layer, focused tests, and a compiling standalone README example.
5. **REQ-RUN-0005** — The sc-config boundary manifest at
   `boundaries/sc-config/sc-config.toml`. `allowed_dependents` is empty.
   Do not copy the open `sc-runtime` → `sc-config` MAY into that key. a-5
   does not own this manifest. In this single-crate lint tree,
   `forbidden_edges` lists only workspace-member targets (none besides
   `sc-config` yet), per REQ-RUN-0005 criterion 2 and ADR-CFG-0003.
   Crates.io names (`sc-observability-types`, logging crates, `tokio`) are
   enforced with `cargo tree -p sc-config -e normal --prefix none`, not
   manifest strings that make sc-lint 0.8.0 exit 3. This deliverable and
   acceptance criterion 6 are
   done only after `triage:a-2-boundary-evidence`. Once
   `crates/sc-config/Cargo.toml` exists, run the single-crate
   `sc-lint lint sc-boundary` command a-1 recorded in
   `docs/architecture.md` against this manifest. Do not
   paste analyzer output into the pull request. The binding evidence is the
   QA re-run in Required validation, not this developer run and not a
   pull-request statement. Do not wait for a-5. Do not
   install sc-lint or own the permanent root workspace `Cargo.toml` (a-5
   still creates the four-member workspace). The command does not need that
   workspace.
6. **REQ-CFG-0004 / ADR-CFG-0001** — Before the facade freezes, the crate ADR
   contains an error-inventory table with one row per `ConfigError` failure:
   failure mode, stable code, cause payload, and recovery steps. Typed path
   and line fields stay on the variants that carry a source location
   (RBP-F002). `sc_config::AppName` is a validated newtype using the handoff
   predicate. Tests load `tests/fixtures/app-name-vectors.json`. This sprint
   does not copy that file. Invalid names fail in the constructor with
   `APP.INVALID_NAME`. Public loaders take `sc_config::AppName`. Display is
   `as_str` or `AsRef<str>`. The type does not implement `Deref` (RBP-F005).
   Amend the crate requirement's `&str` application-name parameters to this
   newtype.

## Acceptance criteria

1. `boundary:sc-config` — crate tests cover file, merge, environment, malformed
   input, and explicit-input parallel behavior; the README example compiles.
2. `boundary:sc-config` — the crate builds independently with only its approved
   normal dependencies and no sibling SC crate.
3. `ADR-CFG-0003` — inspection finds no reload, watcher, async runtime, global
   mutable registry, or generated-application policy.
4. `ADR-CFG-0001` — the crate ADR error-inventory table lists every
   `ConfigError` failure with a stable code, cause payload, and recovery
   steps, and the variants that name a file or variable still carry that
   typed path or line data (RBP-F002).
5. `boundary:sc-config` — tests load
   `tests/fixtures/app-name-vectors.json`. `sc_config::AppName` accepts every
   accept string in that file and rejects every reject string at
   construction, with stable code `APP.INVALID_NAME`. `load` and
   `Loader::new` take `sc_config::AppName`. The type exposes `as_str` or
   `AsRef<str>` and does not implement `Deref` (RBP-F005). The test source
   does not embed a second vector list.
6. `boundary:sc-config` — the single-crate `sc-lint lint sc-boundary`
   command a-1 recorded exits 0 on `boundaries/sc-config/sc-config.toml`
   after `crates/sc-config/Cargo.toml` exists. `allowed_dependents` is
   empty. `forbidden_edges` is empty in the lint tree; forbidden crates.io
   names are checked with `cargo tree` per ADR-CFG-0003. The pull request
   does not retain analyzer output. This criterion is marked done only
   after `triage:a-2-boundary-evidence`. Leaf QA re-runs the command at
   the reviewed commit and records only the exit code. That re-run is the
   evidence.
7. `boundary:sc-config` — NFR-CFG-0001 is this crate's strictness on
   NFR-RUN-0009. Public loaders return `Result<_, ConfigError>`. Tests pass
   a missing file and malformed JSON and assert the `ConfigError` variant.
   `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, and
   `unimplemented!` are not reachable from a public function. No public
   signature names `anyhow::Error` or `Box<dyn Error>`. The repo-wide grep
   and the parallel `just test` runs stay with a-5.

## This sprint does not close

Daemon assembly, template configuration layout, application defaults, and
generated-project behavior close in a-5 or a-6. This sprint runs the
boundary analyzer on its own manifest. Root `just lint` across all four
manifests stays with a-5. This sprint does not edit
`docs/architecture.md` or `docs/requirements.md`.

## Paths to delete

None.

## Required validation

Run crate tests, doctests, formatting/linting, and workspace build once
registered. Check the default dependency tree. Review the manifest against
REQ-RUN-0005. Confirm the ADR-CFG-0001 error-inventory table and the
`AppName` construction tests, which load
`tests/fixtures/app-name-vectors.json`. Leaf QA re-runs the single-crate
`sc-lint lint sc-boundary` command a-1 recorded on this crate's manifest
at the reviewed commit and records only the exit code in the QA report.
That re-run is the binding evidence. A statement in the pull request is not.
