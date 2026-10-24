---
id: a-5
phase: a
status: planned
closure_type: integration
target_boundary: runtime-composition
vertical_rationale: "sc-runtime is the one intended assembly boundary; its greenfield implementation and rewritten-spike consumer form one cohesive vertical closure after the three real leaf APIs exist."
track: core
wave: 2
branch: sprint/a-5-runtime-core
worktree: sprint/a-5-runtime-core
recommended_agent: solar
recommended_model: sol
must_follow: [a-2, a-3, a-4]
parallel_safe: []
requirements:
  - REQ-RT-0001
  - REQ-RT-0002
  - REQ-RT-0003
  - REQ-RT-0004
  - REQ-RT-0005
  - REQ-RT-0006
  - REQ-RT-0007
  - REQ-RT-0008
  - NFR-RT-0001
  - NFR-RT-0002
  - NFR-RT-0003
  - NFR-RT-0004
  - REQ-RUN-0001
  - REQ-RUN-0002
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0103
  - REQ-RUN-0204
  - REQ-RUN-0205
  - NFR-RUN-0002
  - NFR-RUN-0003
  - NFR-RUN-0004
  - NFR-RUN-0005
  - NFR-RUN-0008
  - NFR-RUN-0009
  - NFR-RUN-0010
adrs:
  - ADR-RUN-0005
  - ADR-TRN-0002
  - ADR-RT-0001
  - ADR-RT-0002
  - ADR-RT-0003
  - ADR-RT-0004
  - ADR-RUN-0001
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0004
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0008
  - ADR-RUN-0009
  - ADR-RUN-0010
  - ADR-RUN-0201
  - ADR-RUN-0202
owned_paths:
  - "Cargo.toml"
  - "Cargo.lock"
  - "Justfile"
  - "sc-lint.toml"
  - ".sc-lint/bootstrap.py"
  - "crates/sc-runtime/**"
  - "boundaries/sc-runtime/**"
  - "examples/spike/**"
  - ".github/workflows/crates.yml"
  - "docs/sc-runtime/requirements.md"
  - "docs/sc-runtime/architecture.md"
  - "docs/architecture.md"
  - "docs/requirements.md"
---

# Sprint a-5 — sc-runtime core

## Outcome and closure

Complete the reusable core milestone: implement the runtime assembly crate,
compose all four real libraries in the spike, and prove that the runtime owns
lifecycle rather than application policy.

The sprint spans one production boundary and its immediate composition root.
That is the vertical product boundary described by the original milestone;
splitting a shape-only runtime sprint from its only proof would add a serial PR
without increasing width.

## Decision prerequisites

Resolve builder inputs, router/service types, lock behavior, health/OpenAPI,
shutdown, fixture cleanup, and typed errors. The `sc-runtime` → `sc-config`
edge and the `cargo tree -p cli` exclusion are deliverable 9.
Choose **consuming typestate** for `DaemonBuilder` or explicit
`RuntimeError` for missing required closures (RBP-F007).
`RuntimeError::StoresFailed` is non-generic. Its code is `&'static str` and
its remediation, message, and optional cause are `String`. No
`sc-observability-types` type appears on this crate. The application maps
its store error into that variant inside the stores closure. Reject
`Box<dyn Error>` and generic `RuntimeError<E>` (RBP-F003). Publish runtime
**error inventory** (RBP-F002). Embed a-3's frozen `TransportConfig` (v0.1
single-bind, `additional_endpoints` reserved and unused). Do not resolve CLI
auto-start or application test architecture here. Record builder and
lifecycle choices in the owned crate ADR before the facade freezes.

Install sc-lint 0.8.0 as the source-maintainer model from the sc-lint docs.
Verify and extend the existing root `Justfile` (capital J). Keep its first
line unchanged:

```
# Managed by sc-lint; regenerate with `sc-lint init --just`.
```

Do not replace that file with a hand-written recipe file, and do not delete or
contradict the header. Public recipes `setup`, `lint`, `test`, and `upgrade`
stay in the sc-lint-managed block. Each calls `.sc-lint/bootstrap.py` with
`sc-lint.toml`. Repo-specific recipes stay in the section that
`sc-lint init --just` does not regenerate; the existing `new` stub stays for
a-6. This checkout does not build sc-lint, so `setup` installs the verified
0.8.0 release rather than a `target/debug` binary or a source-checkout
`--dry-run`. `just lint` runs `sc-lint-boundary analyze` on every
`boundaries/<crate>/*.toml` manifest and the sc-lint line-counts check at
1000 source lines for non-test files under `crates/*/src` only
(NFR-RUN-0008). `template/**/*.rs` is not in this sprint; a-6 adds that half
after the generation tree exists. The line-counts configuration key is the
one the 0.8.0 package provides; record that key when the install lands.
`just new` is not implemented here.

## Public contracts consumed

This sprint consumes the `sc-transport` and `sc-command` facades from the
[minimal public contract handoff](phase-a-plan.md#minimal-public-contract-handoff)
after a-3 and a-4 have recorded their decisions in their owned source
documents. It consumes endpoint and listener types, and the `sc-command`
server conversions. It registers a-2's crate and manifest and does not
consume an `sc-config` type. It produces `sc_runtime::AppName` under the
handoff predicate a-1 records as ADR-RUN-0010. That type is the daemon and
application identity for builder paths. `Daemon::builder` does not take or
re-export `sc_transport::AppName`. See the
[handoff](phase-a-plan.md#minimal-public-contract-handoff) for the predicate
and the only vector list. This sprint does not copy that list.
This sprint does not write that ADR. `DaemonConfig` embeds the
`TransportConfig` type a-3 froze. The sc-runtime requirement and ADR sources
remain authoritative for the builder. `docs/architecture.md` edits are the
slices in deliverables 9 and 11.

## Dependencies and parallel safety

- `must_follow a-2`: registers the real `sc-config` crate and
  `boundaries/sc-config/sc-config.toml`. `sc-runtime` does not depend on
  `sc-config` and consumes no type from it. There is no fallback that adds
  that edge. Applications load configuration before calling runtime.
- `must_follow a-3`: consumes endpoint and listener types and the frozen
  `TransportConfig`, including the v0.1 single-bind decision.
- `must_follow a-4`: consumes the required `sc-command` server conversions. The
  `sc-runtime -> sc-command/server` edge is binding in REQ-RUN-0005 and
  ADR-RUN-0003; application handlers still use ordinary Axum/rmcp types.

a-3 and a-4 may run in parallel. a-2 is not `parallel_safe` with this sprint.
A dependency discovered to be unnecessary is removed rather than preserved
for symmetry.

## Deliverables

1. **REQ-RT-0001 / REQ-RT-0003 / REQ-RT-0008** — Application loads config
   before `Daemon::builder(sc_runtime::AppName, &DaemonConfig)`. `DaemonConfig`
   embeds a-3's `TransportConfig`. Builder uses consuming typestate so `run`
   exists only after `stores` and `routes`, with optional `mcp` on that ready
   state. The call chain remains `stores` → `routes` → optional `mcp` → `run`.
   If ADR-RT-0001 rejects typestate, a missing required closure returns an
   explicit `RuntimeError` (RBP-F007). The in-run ordering test does not
   replace that caller-facing check. Lock precedes stores; bind is last.
   Ordinary routers/services; no command or store registry.
2. **REQ-RT-0002** — A retained nonblocking singleton lock protects one daemon
   instance without PID/probe machinery.
3. **REQ-RT-0004 / REQ-RT-0005** — One router/listener composition and graceful
   shutdown use standard Axum/Tokio/rmcp facilities.
4. **REQ-RT-0006** — An in-process `DaemonFixture` uses the production assembly
   path and a fresh temporary instance root; it never launches a daemon process.
   The fixture and its temp-dir dependency sit behind the `test-util` feature
   closed in deliverable 12. A production daemon build does not enable that
   feature.
5. **REQ-RT-0007 / ADR-RT-0004** — Before the facade freezes, the crate ADR
   holds a runtime error inventory: every `RuntimeError` variant has a failure
   mode, stable code, cause payload, and recovery text (RBP-F002).
   `RuntimeError::StoresFailed` is non-generic. Code is `&'static str`.
   Message, remediation, and optional cause text are `String`. The
   application maps its store error into that variant inside the stores
   closure. `sc-runtime` does not name or match the foreign error type and
   does not depend on `sc-observability-types`.
   `Box<dyn Error>` and generic `RuntimeError<E>` are rejected. Delete any
   `source()`-without-downcast or named-foreign-payload requirement from the
   crate ADR and from REQ-RT-0007 (RBP-F003). `run` never calls `process::exit`.
6. **REQ-RUN-0103 / REQ-RUN-0204 / REQ-RUN-0205** — Rewrite the spike against
   the four crates, keep its server `main.rs` small, and prove one-listener
   REST/OpenAPI/MCP composition.
7. **REQ-RUN-0001 / REQ-RUN-0004 / REQ-RUN-0005** — Create the four-member
   workspace, root core recipes/CI, and runtime boundary manifest without
   registering template packages as workspace members. `examples/spike` is an
   excluded standalone package, not a fifth member. `boundaries/sc-runtime/sc-runtime.toml`
   joins the leaf manifests. The `sc-config` workspace member is path
   registration only and adds no `sc-runtime` dependency. `.github/workflows/crates.yml`
   calls `just setup`, `just lint`, and `just test` only. It names no spike
   job, so deleting `examples/spike/**` in a-7 cannot leave a stale spike step.
   Leaf sprints already analyzed their own manifests.
8. **REQ-RUN-0004 / NFR-RUN-0008** — Verify and extend the existing root
   `Justfile`. Do not contradict its sc-lint init header. Source-maintainer
   sc-lint 0.8.0 install: `sc-lint.toml` pins `minimum_version` to `0.8.0`,
   `.sc-lint/bootstrap.py` is the 0.8.0 helper, and `just lint` includes
   `sc-lint-boundary analyze` plus the 1000-line line-counts check for
   `crates/*/src` only. Crate CI calls `just setup` before `just lint` and
   `just test`. Do not copy boundary manifests into the template. Do not
   close the `template/**/*.rs` half of NFR-RUN-0008.
9. **ADR-RUN-0003 / NFR-RT-0003 / REQ-RUN-0005** — Record `sc-runtime` →
   `sc-config` as edge absent in every copy, and put `sc-config` on
   `sc-runtime` `forbidden_edges`:
   - `docs/architecture.md` crate-graph table and its OPEN
   - `docs/architecture.md` ADR-RUN-0003 table and its OPEN
   - `docs/requirements.md` REQ-RUN-0005 table, the "exactly this table"
     clause, and that OPEN
   - `docs/sc-runtime/requirements.md` NFR-RT-0003, the `sc-config` OPEN only.
     Deliverable 12 closes the fixture, temp-dir, and `rmcp` OPENs.
   - `boundaries/sc-runtime/sc-runtime.toml` `forbidden_edges`
   Keep a-1's rule that `sc-runtime` forbids `sc-observability-types`.
   `sc-runtime` does not depend on `sc-config` or on `sc-observability-types`.
   This sprint does not add `cargo tree -p cli`; the phase
   feature-enforcement table assigns that check to a-6.
   Workspace-unification was closed in a-1.
10. **REQ-RUN-0001 / REQ-RUN-0004 / REQ-RUN-0005** — Close these OPENs in
    `docs/requirements.md`:
    - REQ-RUN-0001: `examples/spike` is an excluded standalone package, not a
      workspace member and not a Cargo example. The member list stays the
      four library crates.
    - REQ-RUN-0004: `just lint` runs the sc-lint 0.8.0 full profile (boundary
      analyze, line-counts, fmt, clippy) through `.sc-lint/bootstrap.py`.
      `just lint gates` is the early boundary plus line-counts profile.
    - REQ-RUN-0005: the manifest file name is `boundaries/<crate>/<crate>.toml`.
    - REQ-RUN-0001 layout table: the root recipe path is `Justfile`
      (capital J), the name sc-lint init and this repository use. Delete the
      lowercase `justfile` cell. Do not leave a sentence in that requirement
      that requires a lowercase `justfile` path.
    Delete those three OPEN lines. The layout-table rename is not an OPEN;
    it is the filename amendment.
11. **REQ-RT-0001 / REQ-RUN-0205** — Copy a-3's dual-bind decision. Amend the
    runtime requirement and ADR OPENs, the REQ-RUN-0205 OPEN, and the
    dual-bind OPEN paragraphs in `docs/architecture.md`: v0.1 binds one
    listener; `additional_endpoints` is reserved and ignored in v0.1. Do not change
    the field list a-3 froze. The ignore is the one a-3's ADR amendment states.
12. **NFR-RT-0003** — Before the facade freezes, close the three NFR-RT-0003
    OPENs that deliverable 9 does not close. Delete those OPEN paragraphs in
    `docs/sc-runtime/requirements.md` and replace them with this decision:
    - `testing::DaemonFixture` sits behind the non-default cargo feature
      `test-util`. A production daemon build does not enable it and does not
      link the fixture.
    - The temporary-directory crate is an optional dependency of `test-util`
      only. It is not a normal dependency.
    - `rmcp` is a direct dependency. `.mcp(...)` accepts the concrete rmcp
      service type and mounts it as given. `sc-runtime` does not wrap that
      type and does not re-export it (ADR-RUN-0001 point 4).
    The `sc-config` OPEN stays the edge-absent close in deliverable 9.
    `Daemon::builder` still takes `sc_runtime::AppName`. It does not take or
    re-export `sc_transport::AppName`. Construction tests reference the
    handoff vector list and do not copy it.

## Acceptance criteria

1. `req:REQ-RUN-0204` — the rewritten spike loads configuration outside or at
   the explicit runtime seam, constructs the builder, and reaches the same
   service function through the proved surfaces with a server `main.rs` under
   the source requirement's limit.
2. `req:REQ-RUN-0205` — REST, OpenAPI, health, and optional stateless MCP share
   the intended process/listener without wrappers around ordinary framework
   types.
3. `req:REQ-RT-0006` — two in-process fixtures run concurrently on independent
   temporary roots, stop independently, and clean up their runtime resources.
4. `req:REQ-RT-0001` — an ordering test proves lock acquisition precedes the
   single stores invocation; lock contention never invokes stores, a stores
   error never binds, and success proceeds stores → routes/MCP → bind.
5. `req:REQ-RT-0005` — ephemeral-CI-only real-process tests prove SIGINT and
   SIGTERM use the production shutdown path, release the lock, and allow an
   immediate restart. They refuse to start on an unmarked host and add no test
   daemon, VM definition, or test-only production switch.
6. `req:REQ-RT-0008` — with two distinct temporary instance roots and one UDS
   endpoint override, the second startup fails before unlinking or replacing
   the live endpoint; the first daemon remains reachable and cleanup by the
   failed startup cannot remove its socket.
7. `boundary:sc-runtime` — crate tests, README example, default/server feature
   checks, workspace build, and all four boundary manifests are green.
8. `ADR-RUN-0001` / `ADR-RUN-0008` — inspection finds no application command
   registry, generated CLI testing abstraction, service-manager launcher, or
   SQL schema in the reusable libraries. This criterion does not check
   observability dependencies. Leaf sprints and criterion 7 cover those edges.
9. `req:REQ-RUN-0004` / `req:REQ-RUN-0005` / `req:NFR-RUN-0008` — `just --list`
    shows `setup`, `lint`, `test`, and `upgrade`. The root file is `Justfile`.
    Its first line is still the sc-lint init header quoted under Decision prerequisites.
    Repo-specific recipes stay in the section that init does not regenerate.
    `just lint` invokes `sc-lint-boundary analyze` and exits 0 on the four
    manifests. The line-counts check is configured for non-test files under
    `crates/*/src` at 1000 source lines, using the key the 0.8.0 package
    provides. This criterion checks that configuration. It does not add a
    synthetic oversize file. It does not close `template/**/*.rs`.
10. `req:REQ-RT-0007` — the crate ADR error-inventory table gives every
    `RuntimeError` variant a stable code, cause payload, and recovery text.
    A test matches `StoresFailed` and reads its code as `&'static str` and its
    message, remediation, and optional cause as `String`. The crate manifest
    does not depend on `sc-observability-types`. The stores closure is what maps an
    application store error into that variant. The test does not call
    `source()` and does not downcast. Public signatures contain no
    `Box<dyn Error>`, and `RuntimeError` is not generic (RBP-F002, RBP-F003).
11. `req:REQ-RT-0001` — caller-facing builder sequence is closed in
    ADR-RT-0001: either consuming typestate makes `run` available only after
    `stores` and `routes`, with optional `mcp` on that ready state, or the
    ADR rejects typestate and a missing required closure returns an explicit
    `RuntimeError`. `Daemon::builder` takes `sc_runtime::AppName`, the daemon
    and application identity for builder paths. It does not take or re-export
    `sc_transport::AppName`. Construction
    tests use the handoff accept and reject vectors and expect
    `APP.INVALID_NAME` on rejection. The vectors are the handoff list. This
    sprint does not copy that list. The type exposes `as_str` or `AsRef<str>`
    and does not implement `Deref`. The predicate is ADR-RUN-0010. The in-run
    ordering test remains, and it is not the only proof of the caller-facing
    sequence (RBP-F007).
12. `ADR-RUN-0003` / `req:REQ-RUN-0005` — every slice in deliverable 9 records
    the `sc-runtime` → `sc-config` edge as absent, and
    `boundaries/sc-runtime/sc-runtime.toml` `forbidden_edges` includes
    `sc-config`. `sc-runtime` does not depend on `sc-config`.
13. `req:REQ-RUN-0205` — the dual-bind OPEN paragraphs named in deliverable 11
    match a-3: v0.1 single-bind, `additional_endpoints` reserved and ignored
    in v0.1.
14. `req:NFR-RT-0003` — the three OPEN paragraphs named in deliverable 12 are
    gone. `testing::DaemonFixture` is behind non-default `test-util`. The
    temp-dir crate is optional on that feature only and is absent from normal
    dependencies. `rmcp` is a direct dependency, and the crate does not wrap
    or re-export the rmcp service type. A default build does not link
    `DaemonFixture`. The facade is not frozen while any of those three OPENs
    remains.
15. `req:REQ-RUN-0001` — this sprint closes the workspace half of the
    success criteria. Root `Cargo.toml` `[workspace] members` resolves to
    exactly `crates/sc-config`, `crates/sc-transport`, `crates/sc-command`,
    and `crates/sc-runtime`. `examples/spike` is an excluded standalone
    package, not a fifth member and not a Cargo example. `cargo metadata
    --no-deps --format-version 1` lists no package whose manifest path is
    under `template/`. `cargo build --workspace` exits 0 and compiles no
    file under `template/`. `test -d docs/adr` fails, and
    `grep -c '^## ADR-' docs/architecture.md` prints a number greater than 0.
    The amended layout table names `Justfile`. `template/`, `wizard/`,
    `scripts/`, and `tests/unit/` are later sprints. a-7 closes success
    criteria 3 and 4 for the paths that exist at the core tag. a-8 closes
    `wizard/wizard.json`, `wizard/pages/`, and `scripts/run_wizard.py`.
16. `req:NFR-RUN-0005` — maintainer half. `just test` passes with the
    default parallel runner. No recipe passes `--test-threads=1`. Two
    simultaneous `just test` runs from two checkouts of this repository
    both pass. `grep -rn '"/tmp\|/Users/\|/home/\|C:\\\\Users' crates/*/src`
    prints no line in non-test code. Every test in this sprint that starts
    a daemon takes its instance root from a temporary-directory API or from
    `DaemonFixture`. No test binds a fixed TCP port. `template/` is a-6's
    half of the same grep. A generated project running beside a developer's
    daemon is a-6.

## This sprint does not close

Generated application structure, the consumer sc-lint install inside the
template, JSON-driven generation, registry publication, CLI auto-start,
generated-application process tests, attribute-generated tests, and the
wizard remain open. `just new` is added by a-6. The `template/**/*.rs`
half of NFR-RUN-0008 stays with a-6. Edge absence and the `cargo tree -p cli`
exclusion are deliverable 9.

## Paths to delete

None. a-7 removes the rewritten spike after release evidence is preserved.

## Required validation

Run the acceptance criteria. Host checks use in-process fixtures. Signal and
process proofs run only on an ephemeral CI runner with an isolation guard.
CI output is the record.
