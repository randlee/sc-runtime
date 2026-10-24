---
id: a-1
phase: a
status: complete
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
  - ADR-RUN-0008
  - ADR-RUN-0009
  - ADR-RUN-0010
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0302
  - ADR-RUN-0402
  - ADR-TRN-0003
owned_paths:
  - "examples/spike/**"
  - "docs/architecture.md"
  - "docs/requirements.md"
  - "tests/fixtures/app-name-vectors.json"
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
6. **REQ-RUN-0309 / ADR-RUN-0004** — Record the spike's resolved
   `sc-observability` and `sc-observability-types` versions. Amend
   ADR-RUN-0004 and REQ-RUN-0309 so that pair replaces the Active 1.2.x
   sentences. Point 3 stays the Active rule: the only observability
   dependency inside the four libraries is `sc-observability-types` on
   `sc-command`. Do not extend that point to `sc-transport`.
   `DaemonNotRunning` carries `code: &'static str` (`DAEMON.NOT_RUNNING`)
   plus the endpoint and the cause. It does not carry
   `sc_observability_types::ErrorCode`. a-4's `sc-command` publishes the
   constructor from those strings into `OpError`; a-6's generated CLI calls
   it. Amend [ADR-TRN-0003](../../sc-transport/architecture.md) Active in
   this sprint. This sprint does not write
   `docs/plans/phase-a/evidence/lead-ruling-sc-observability-types-transport.md`.
   `Remediation` stays on `sc-command` only. The logging runtime stays
   forbidden on all four libraries. a-3 consumes this decision and does not
   add the types crate.
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
   hang this decision on it. ADR-RUN-0010 states two crate-local types:
   `sc_config::AppName` is the configuration identity; `sc_transport::AppName`
   is the client wire identity (`resolve_endpoint`, `instance_root`) and the
   name `Daemon::builder` borrows. `Client::new` takes an `Endpoint` only.
   `Daemon::builder` takes `&sc_transport::AppName`. There is no
   `sc_runtime::AppName`. `sc-runtime` does not re-export the transport type
   under another name and does not convert inside `run`. ADR-RUN-0010 records
   the rejected alternative of a third runtime newtype: `sc-runtime` already
   depends on `sc-transport`, so a duplicate predicate and a fallible
   conversion are not a boundary. There is no shared `AppName` type and no
   fifth crate. The handoff predicate (ASCII `A-Z`, `a-z`, `0-9`, `-`, `_`;
   length 1 through 64 bytes) is the only predicate for the two constructors;
   rejection code `APP.INVALID_NAME` on that crate's own error; display
   through `as_str` or `AsRef<str>`; no `Deref`. Generation tooling uses the
   project-name `pattern` on `wizard/answers.schema.json` and does not
   re-implement the predicate. This sprint writes
   `tests/fixtures/app-name-vectors.json`. That file is the only enumeration.
   Accept strings are `my-app`, `A`, `app_1`, and 64 `a` characters. Reject
   strings are the empty string, `my app`, `my.app`, `my/app`, 65 `a`
   characters, and `café`. ADR-RUN-0010 references that path and does not
   paste the vectors. The file is not packaged in any crate. Do not paste
   the ADR body into this sprint doc. a-2, a-3, and a-6 tests load the file.
   They do not invent a second predicate and they do not embed a second list.
   a-5 does not construct a runtime name.
9. **REQ-RUN-0005** — Amend the observability-types edges and success
   criterion 2. `sc-observability-types` is an allowed dependency of
   `sc-command` only. `sc-config` and `sc-transport` forbid it on every
   edge, direct and transitive. Their `forbidden_edges` list
   `sc-observability-types` as its own string, distinct from
   `sc-observability` and `sc-observability-otlp`. The logging-crate token
   must not match the types crate; deliverable 11 records that. The
   `sc-runtime` forbidden edge for `sc-observability-types` is direct-only
   when the analyzer can express that. A transitive copy that arrives
   through `sc-command` is allowed. Success criterion 5 exempts that
   transitive `sc-observability-types ` line for `sc-runtime` and still fails
   a direct `sc-runtime` dependency on the crate. If deliverable 11 shows
   the analyzer cannot express direct-only, this amendment moves that
   `sc-runtime` rule to the `cargo tree -p sc-runtime -e normal` check
   instead of a manifest edge. Success criterion 2 also checks
   `allowed_dependents`: empty for `sc-config`, `sc-runtime` only for
   `sc-transport` and `sc-command`, and empty for `sc-runtime`. The open
   `sc-runtime` → `sc-config` MAY is not copied into `allowed_dependents`.
   a-5 still deletes that MAY row. Do not edit the MAY in this sprint. a-5
   amends NFR-RT-0002 to the same direct-only rule. a-3 and a-4 amend
   NFR-TRN-0001 and NFR-CMD-0001 to these lists. a-3 does not add
   `sc-observability-types` to transport's allowed dependencies.
10. **NFR-RUN-0007** — Record the spike's rmcp pin. Every spike `Cargo.toml`
    that names `rmcp` pins one minor series: a tilde requirement on
    `MAJOR.MINOR`, or an equivalent range `>=MAJOR.MINOR.0, <MAJOR.(MINOR+1).0`.
    `"3"`, a caret `"3.4"`, and `"*"` are not pins. All of those manifests
    use the same `MAJOR.MINOR`. Write that series into `docs/architecture.md`
    and replace the NFR-RUN-0007 OPEN (which minor is pinned) with it.
    a-6 and a-7 verify later manifests against this record. They do not
    choose a second series.
11. **REQ-RUN-0005** — Prove single-crate boundary lint, including the
    negative fixtures this plan relies on. On a disposable single-crate tree
    that is its own `[workspace]` root (fixture `exit-ok`; a tree with no
    `[workspace]` exits 3), run `sc-lint lint sc-boundary` and record the
    exact command plus each exit code in `docs/architecture.md` before a-2,
    a-3, a-4, or a-5 writes a manifest.
    - One manifest and crate that satisfy the command exit 0.
    - Prefix match: a manifest forbids `sc-observability` while the crate
      depends on `sc-observability-types`. The expected exit is 0. A
      non-zero exit means the logging token matches the types crate. Record
      that outcome and stop. Do not treat the two names as distinct
      forbidden edges.
    - Direct forbidden edge: the crate depends directly on a name its
      manifest forbids. The expected exit is non-zero, and the report names
      that edge.
    - Transitive versus direct: one manifest forbids a crate that arrives
      only transitively, and one uses the direct-only shape `sc-runtime`
      needs for `sc-observability-types` through `sc-command`. Record both
      exit codes. If the analyzer cannot express direct-only, deliverable 9
      moves the `sc-runtime` rule to `cargo tree`.
    a-2, a-3, and a-4 run the recorded command on their own crate
    `Cargo.toml` and manifest. They do not wait for the a-5 workspace, and
    they do not paste analyzer output into a pull request. Leaf QA re-runs
    the command at the reviewed commit.

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
   as the Phase A pin. Point 3 stays command-only: `sc-observability-types`
   is allowed on `sc-command` and is not allowed on `sc-transport`.
   `DaemonNotRunning` is a `&'static str` code plus endpoint and cause.
   `Remediation` stays on `sc-command` only. They forbid the logging runtime
   on all four libraries. No Active sentence still requires 1.2.x. The sprint
   diff does not add a lead-ruling artifact. a-3 cites this decision.
   a-4 publishes the `sc-command` constructor; a-6 calls it in the generated
   CLI.
8. `boundary:compatibility-evidence` — ADR-RUN-0005, REQ-RUN-0307, and
   NFR-RUN-0006 are the sc-lint 0.8.0 consumer install named in deliverable 7,
   including the retired placeholder Justfile and prototype-handoff order.
   a-6 remains the feature-level close of the generated consumer install.
9. `ADR-RUN-0010` — `docs/architecture.md` contains Active ADR-RUN-0010 and
   its index row, matching deliverable 8: two identities, and
   `Daemon::builder` takes `&sc_transport::AppName`. There is no
   `sc_runtime::AppName`. The ADR references
   `tests/fixtures/app-name-vectors.json` and records the rejected
   third-newtype alternative. The sprint diff adds that ADR and that file.
   It does not rewrite ADR-RUN-0006 into the AppName decision. a-2 does not
   freeze `sc_config::AppName` until this ADR is Active. The gate bead
   `triage:a-2-appname-freeze` follows this sprint.
10. `req:REQ-RUN-0005` — the amended table allows `sc-observability-types` on
    `sc-command` only. `sc-config` and `sc-transport` forbid it on every
    edge, as its own forbidden-edge string. The `sc-runtime` forbidden edge
    is direct-only, and success criterion 5 allows the transitive
    `sc-observability-types ` line that arrives through `sc-command`, unless
    deliverable 11 moved that rule to `cargo tree`. A direct `sc-runtime`
    dependency on that crate still fails. Success criterion 2 names
    `allowed_dependents`: empty for `sc-config` and `sc-runtime`, and
    `sc-runtime` only for `sc-transport` and `sc-command`. The
    `sc-runtime` → `sc-config` MAY is unchanged in this sprint and is not
    copied into `allowed_dependents`.
11. `req:NFR-RUN-0007` — the evidence record names one rmcp `MAJOR.MINOR`.
    Every spike manifest that names `rmcp` uses a tilde requirement or an
    equivalent bounded range on that series. The NFR-RUN-0007 OPEN that
    asked which minor is pinned is gone. a-6 and a-7 grep later manifests
    against this series.
12. `req:REQ-RUN-0005` — `docs/architecture.md` records the single-crate
    `sc-lint lint sc-boundary` command from deliverable 11, the exit-0 run
    on a single-crate `[workspace]` root (`exit-ok`), and the negative fixture runs:
    prefix match (`sc-observability` forbidden while depending on
    `sc-observability-types`), a direct forbidden edge that exits non-zero,
    and transitive versus direct. The record is in `docs/architecture.md`
    before a-2, a-3, a-4, or a-5 writes a manifest. Gate bead
    `triage:a-2-boundary-evidence` follows this sprint.

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
REQ-RUN-0005, NFR-RUN-0006, and NFR-RUN-0007 amendments landed in the owned files, including the AppName vector file, the two-newtype ADR, the direct-only `sc-runtime` edge, success criterion 2 `allowed_dependents`, and the single-crate lint record with its negative fixture outcomes. The diff does not add a lead-ruling artifact. If a Verify item fails, stop dependent
planning rather than adding an adapter automatically.
