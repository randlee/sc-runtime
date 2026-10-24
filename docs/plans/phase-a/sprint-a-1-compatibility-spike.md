---
id: a-1
phase: a
status: planned
closure_type: contract
target_boundary: compatibility-evidence
vertical_rationale: "The production boundaries do not exist; one disposable composition proves the external interfaces they will use."
track: evidence
wave: 0
branch: sprint/a-1-compatibility-spike
worktree: sprint/a-1-compatibility-spike
recommended_agent: solar
recommended_model: sol
must_follow: []
parallel_safe: [a-2]
requirements:
  - REQ-RUN-0101
  - REQ-RUN-0102
  - NFR-RUN-0001
  - NFR-RUN-0007
  - NFR-RUN-0010
  - REQ-RUN-0306
  - REQ-RUN-0307
  - REQ-RUN-0309
  - REQ-RUN-0005
  - NFR-RUN-0006
adrs:
  - ADR-RUN-0001
  - ADR-RUN-0003
  - ADR-RUN-0004
  - ADR-RUN-0005
  - ADR-RUN-0009
  - ADR-RUN-0010
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0302
  - ADR-RUN-0402
owned_paths:
  - "examples/spike/**"
  - "docs/architecture.md"
  - "docs/requirements.md"
---

# Sprint a-1 — compatibility spike

## Outcome and closure

Prove only the external facts that the core libraries and generator need.
The spike is disposable evidence, not an early framework or template.

`closure_type: contract` means that evidence record. Later sprints consume
the recorded versions, signatures, and the amendments below. QA uses this
sprint's validation section. It does not use a contract-sprint checklist of
API signatures, test doubles, and boundary manifests.

## Decision prerequisites

Choose the smallest operation, route/tool name, and MCP proof client needed to
exercise the stack. Do not treat a proposed library signature as verified until
the spike compiles and runs it. Record exact versions rather than introducing
compatibility wrappers.

## Dependencies and parallel safety

There is no predecessor. `parallel_safe` with a-2: `sc-config` does not
consume spike signatures, and the owned paths do not intersect. a-3 and a-4
`must_follow` this sprint and consume the demonstrated reqwest, Axum, and
rmcp signatures plus the recorded `sc-observability-types` version.

## Deliverables

1. **REQ-RUN-0101** — One throwaway server combines Axum, utoipa-axum,
   stateless rmcp, SQLite, a UDS listener, and one service function; a Clap
   client reaches it using reqwest's supported UDS facility.
2. **REQ-RUN-0102** — Record the actual versions and results for the design's
   Verify items, including the concrete third-party Rust type paths and call
   signatures used by the spike and cargo-generate
   conditional/exclude/value-file syntax. The spike manifest depends on
   `sc-observability` and `sc-observability-types`. Record the exact resolved
   versions from the spike lockfile or `cargo tree` in `docs/architecture.md`.
   The pin is the latest crates.io release at spike time (1.5.0 as of this
   plan). Dependent sprints copy that recorded pair. Update the corresponding
   Proposed verification ADRs to Active, or amend them to the proved mechanism.
3. **NFR-RUN-0001 / ADR-RUN-0003** — Measure default vs `server` feature
   graphs for the spike workspace. Record whether Axum, rmcp, and sqlx stay
   out of the default **CLI** dependency graph, and document **policy options**
   in `docs/architecture.md`, then **close** the workspace-unification OPEN and
   NFR-RUN-0001 first OPEN: normative selection is `cargo build -p cli`,
   workspace-wide build is exempt, client/server crate split is rejected for
   v0.1. a-6 still owns the generated-project `cargo tree -p cli -e normal`
   proof; a-5 does not run that command.
4. **REQ-RUN-0306** — Prove plain cargo-generate can consume the proposed
   template mechanics; do not build the product template here.
5. **ADR-RUN-0003 / ADR-RUN-0004** — Amend the root ADRs so they point at the
   crate documents that own the signatures: `docs/sc-transport/architecture.md`
   for `DaemonNotRunning`, and `docs/sc-command/architecture.md` for the
   `OpError` wire form. Do not copy field-level types into
   `docs/architecture.md`. The phase handoff is planning input and is
   superseded by those crate ADRs on merge. Close the workspace-unification
   OPEN as in deliverable 3. The `sc-runtime` → `sc-config` MAY stays open
   until a-5 records it absent.
6. **REQ-RUN-0309 / ADR-RUN-0004** — Amend `docs/requirements.md` and
   `docs/architecture.md` so the daemon dependency and the library
   observability rule use the version pair this spike records. Replace the
   Active 1.2.x sentences with that pair. `sc-observability-types` is allowed
   on `sc-command` and, for `ErrorCode` only, on `sc-transport`. The logging
   runtime stays forbidden on all four libraries. `sc-command` remains the
   only `Remediation` user.
7. **ADR-RUN-0005 / REQ-RUN-0307 / NFR-RUN-0006** — Amend those ids to the
   sc-lint 0.8.0 consumer install: `sc-lint init --just`, committed
   `template/sc-lint.toml` (`minimum_version` 0.8.0) and
   `template/.sc-lint/bootstrap.py`, public recipes `setup`, `lint`, `test`,
   and `upgrade`, plus consumer-owned `db-prepare` that must survive a later
   init. `sc-lint.toml` is the consumer pin, not a hand-written
   clippy/rustfmt/deny or `boundaries/` policy. Remove the placeholder
   Justfile and the prototype-handoff order from the Active text.
8. **ADR-RUN-0010** — Before a-2, a-3, or a-5 freezes a facade, record the
   AppName newtype family in `docs/architecture.md` as new ADR-RUN-0010 and
   add its index row. Leave ADR-RUN-0006 as the typed-error rule; do not
   hang this decision on it. ADR-RUN-0010 states three crate-local types,
   `sc_config::AppName`, `sc_transport::AppName`, and `sc_runtime::AppName`;
   no shared `AppName` type and no fifth crate; the handoff predicate
   (ASCII `A-Z`, `a-z`, `0-9`, `-`, `_`; length 1 through 64 bytes);
   rejection code `APP.INVALID_NAME` on that crate's own error; display
   through `as_str` or `AsRef<str>`; no `Deref`. Do not paste the ADR body
   into this sprint doc. Leaf sprints consume the Active ADR. They do not
   invent a second predicate.
9. **REQ-RUN-0005** — Amend only the observability-types edges in that
   requirement's table. `sc-observability-types` is an allowed dependency of
   `sc-transport` and `sc-command` only. Their `forbidden_edges` keep
   `sc-observability` and `sc-observability-otlp` and do not list
   `sc-observability-types` (the logging-crate token must not match the types
   crate). `sc-config` and `sc-runtime` forbid `sc-observability-types`.
   Do not edit the `sc-runtime` → `sc-config` MAY in this sprint. a-3 and a-4
   amend NFR-TRN-0001 and NFR-CMD-0001 to the same dependency lists.

## Acceptance criteria

1. `boundary:compatibility-evidence` — curl over UDS, the Clap client, and an
   MCP tools call all reach the same service function in one process.
2. `boundary:compatibility-evidence` — every recorded version and signature is
   taken from the running spike or tool output, not memory or proposed prose.
3. `boundary:compatibility-evidence` — `docs/architecture.md` records the exact
   `sc-observability` and `sc-observability-types` versions resolved by the
   spike, and names that pair as the Phase A pin for a-4, a-6, and a-7.
4. `ADR-RUN-0003` / `req:NFR-RUN-0001` — measurement records default and
   server selections with evidence, and the workspace-unification OPEN
   paragraphs are **removed** and replaced with all three parts: `cargo build
   -p cli` is normative, workspace-wide build is exempt, and client/server
   crate split is rejected for v0.1. a-6 owns the generated-project
   `cargo tree -p cli -e normal` check; a-5 does not run it.
5. `ADR-RUN-0203` / `ADR-RUN-0402` — every evidence row is recorded with the
   exact version, type path, signature or template syntax, both ADRs are made
   Active (or amended to the proved result), and no dependent sprint starts
   while either remains Proposed.
6. `boundary:compatibility-evidence` — ADR-RUN-0003 and ADR-RUN-0004 point at
   the transport and command crate ADRs for `DaemonNotRunning` and the
   `OpError` wire form. The root ADR text does not restate those field types.
   Workspace-unification OPEN text is gone (see criterion 4).
7. `boundary:compatibility-evidence` — REQ-RUN-0309 and ADR-RUN-0004 name the
   spike's recorded `sc-observability` and `sc-observability-types` versions
   as the Phase A pin. They allow `sc-observability-types` on `sc-command`
   and, for `ErrorCode` only, on `sc-transport`. They forbid the logging
   runtime on all four libraries. No Active sentence still requires 1.2.x.
   a-6 remains the feature-level close of the generated daemon.
8. `boundary:compatibility-evidence` — ADR-RUN-0005, REQ-RUN-0307, and
   NFR-RUN-0006 are the sc-lint 0.8.0 consumer install named in deliverable 7,
   including the retired placeholder Justfile and prototype-handoff order.
   a-6 remains the feature-level close of the generated consumer install.
9. `ADR-RUN-0010` — `docs/architecture.md` contains Active ADR-RUN-0010 and
   its index row, matching deliverable 8. The sprint diff adds that ADR. It
   does not rewrite ADR-RUN-0006 into the AppName decision. a-2 does not
   freeze `sc_config::AppName` until this ADR is Active.
10. `req:REQ-RUN-0005` — the amended table allows `sc-observability-types` on
    `sc-transport` and `sc-command` only, and forbids it on `sc-config` and
    `sc-runtime`. The forbidden token `sc-observability` is not the types
    crate. The `sc-runtime` → `sc-config` MAY is unchanged in this sprint.

## This sprint does not close

No public SC crate API, daemon lifecycle, generated application, CLI policy,
release artifact, or wizard is closed here. This sprint does not produce contract tests, test doubles, or
boundary manifests.

## Paths to delete

None in this sprint. a-7 deletes `examples/spike/**` before the release tag.

## Required validation

Build and run each proof with the exact recorded versions on an ephemeral CI
runner; refuse any child-process proof on an unmarked developer host. Save
commands, platform, commit, concrete type/signature evidence, and results in
`docs/architecture.md`, including the resolved `sc-observability` and
`sc-observability-types` versions, and confirm the ADR-RUN-0003,
ADR-RUN-0004, ADR-RUN-0005, ADR-RUN-0010, REQ-RUN-0309, REQ-RUN-0307,
REQ-RUN-0005, and NFR-RUN-0006 amendments landed in the owned files. If a Verify item fails, stop dependent
planning rather than adding an adapter automatically.
