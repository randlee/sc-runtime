# Phase A — core runtime and minimal generator

Status: initial plan; scope reconciliation and owner decisions remain before
hardening or implementation dispatch.

Phase id: `a`. Plan branch: `plan/phase-a`. Phase branch:
`integrate/phase-a`.

The owner explicitly selected the existing single-letter Phase A name. This is
a grandfathered exception to the guideline's default next-unused letter-pair
rule; all new plan files, ids, branches, and worktrees consistently use lower
case `a`.

## Intent

Phase A follows the five milestones in `docs/sc-runtime-design.md`:

1. prove the external stack;
2. build the reusable core crates;
3. generate one minimal application from answers JSON;
4. publish and validate the core release; and
5. add the Wyvern wizard over the settled answers contract.

The phase deliberately does not standardize every generated application's
internal architecture. The template is an editable example and composition
root. Reusable behavior belongs in published crates; optional policy belongs
in later layers that can be evaluated against real generated applications.

In particular, Phase A does not prescribe hand-written request/render helper
functions or a fixed set of CLI unit tests. Attribute-generated CLI tests and
their supporting abstraction are deferred until the core APIs and one minimal
generated application exist. The phase must not introduce speculative traits,
macros, registries, or adapters merely to support that future work.

## Authority and unresolved source conflict

The user-discussed scope and `docs/sc-runtime-design.md` define the intended
milestones. Existing requirements and Active ADRs remain authoritative until
amended, so this plan is not dispatch-ready while they conflict with the
reduced scope.

Before hardening, reconcile these later additions explicitly. This table is
the authoritative deferred-scope record for identifiers absent from sprint
frontmatter:

| Source id | Phase A disposition | Justification |
|---|---|---|
| `REQ-RUN-0206`, `ADR-RUN-0204` | Deferred pending placement decision | Default CLI auto-start and launchd-equivalent launch are application policy added after the five-milestone design. They are not needed to prove or publish the reusable core crates. Decide whether they become a later phase or a small optional generated layer after the core release. |
| `REQ-RUN-0312`, `ADR-RUN-0304` | Deferred and architecture reopened | The prescribed two-function CLI testing shape conflicts with the intended attribute-generated testing direction. Implement neither architecture in Phase A; evaluate the attribute layer against the completed core. |
| `REQ-RUN-0311` | Deferred with CLI test generation | Its generated-agent testing prescriptions depend on the application test architecture and should not freeze that architecture ahead of the attribute work. |

`ADR-RUN-0009` / `NFR-RUN-0010` remain active safety constraints: any real
daemon process test runs only on an isolated machine. Phase A adds no VM
definition, provisioning system, test daemon, or host fallback.

This plan does not silently edit those source documents. Source amendments
are a prerequisite to dispatch and must preserve the owner decision history.

## Five milestones and eight sprints

The sprint count is eight because the core milestone contains three genuinely
independent leaf-crate boundaries. Keeping those as parallel boundary sprints
is required by the sprint-planning guidelines and buys real width. Everything
else stays at the original milestone granularity.

| Milestone | Sprint(s) | Result |
|---|---|---|
| 0. Compatibility spike | [a-1](sprint-a-1-compatibility-spike.md) | Proved versions and ordinary third-party composition; disposable evidence only. |
| 1. Core crates | [a-2](sprint-a-2-config.md), [a-3](sprint-a-3-transport.md), [a-4](sprint-a-4-command.md), [a-5](sprint-a-5-runtime-core.md) | Three independent leaf crates, then the runtime assembly and rewritten spike. |
| 2. Template from JSON | [a-6](sprint-a-6-template-from-json.md) | One minimal SQLite application generated through `--var-file`, green without edits. |
| 3. Core release | [a-7](sprint-a-7-core-release.md) | Four published crates and registry-backed template validation at `v0.1.0`. |
| 4. Wizard | [a-8](sprint-a-8-wizard.md) | Existing Wyvern UI adapted to produce the settled answers JSON; may land after the core release. |

Small, independently reviewable changes inside a sprint may be delivered as
a `gh-stack` PR stack. PR count is not sprint count. A new sprint is justified
only by independent owned paths and useful parallel execution, not by a desire
for smaller diffs alone.

## Boundary map

No production crate or boundary manifest exists in the initial tree. Each
greenfield leaf-crate sprint creates and owns its public facade, implementation,
manifest, README, tests, and boundary manifest. This avoids a speculative
shape-only contract sprint: the public API lands with the behavior that proves
it. The compatibility spike supplies the verified third-party signatures.

| Boundary | Minimal contract | Dependencies / consumers |
|---|---|---|
| `sc-config` | typed JSON-file plus environment loading | `serde`, `serde_json`; consumed by applications and optionally runtime |
| `sc-transport` | endpoint resolution, listener binding, generic HTTP client, typed not-running error | Tokio/reqwest; Axum only under `server`; consumed by runtime and generated CLI |
| `sc-command` | lossless envelope and standard REST/MCP conversions | serde and observability types; Axum/rmcp only under `server`; consumed by runtime and generated application |
| `sc-runtime` | daemon assembly, singleton lock, graceful shutdown, in-process `DaemonFixture` | consumes the settled leaf crates and ordinary Axum/rmcp types; owns no application registry |
| `template-from-json` | answers JSON to one editable generated application | consumes published crate APIs; no new reusable framework boundary |
| `wizard` | interactive/prefill UI to the same answers JSON | consumes schema and generation driver; remains outside generated output |

Boundary manifests and repository linting protect this repository's four
libraries. They are not copied into generated applications and must not force
application structure.

## Minimal deliverable scope

- Four independently usable and publishable library crates with focused tests,
  compiling README examples, independent versions, and no unnecessary sibling
  dependencies.
- A compatibility spike, rewritten once against the four libraries and removed
  before the release tag.
- One SQLite-only generated example with editable `api-types`, `store-sqlite`,
  `service`, `daemon`, and thin `cli` crates; one create/get operation reaches
  the service through REST, optional stateless MCP, and the CLI transport.
- A small answers schema and fixtures for project name, SQLite, MCP on/off, and
  project licence; unsupported database choices fail before generation.
- A small Python driver for validated `--var-file` generation through
  cargo-generate and sc-compose, followed by the generated project's own lint
  and test commands.
- Fixture CI, four crates.io publications, a registry-backed generated-project
  check, and tag `v0.1.0`.
- A final wizard milestone copied from the specified working p3 example and
  adapted to the settled schema without changing the core crates.

## Explicit non-goals and flexibility guardrails

Phase A does not deliver:

- attribute macros or code generation for CLI tests;
- a mandatory request/render function architecture or fixed generated test
  suite layout;
- CLI auto-start/service-manager integration until its post-core placement is
  explicitly resolved;
- Postgres, `both`, MySQL, SQL Server, store actors, a command registry, a
  portable query layer, generated Rust clients, or custom HTTP/MCP wrappers;
- configuration reload/watch/async layers, tracing bridges, OTel export,
  TCP authentication, or stdio bridges;
- sc-lint installation into generated projects, prototype-repository handoff,
  VM provisioning, or a custom test daemon;
- enforcement that an application keep the example's crate graph, route names,
  operation names, pool counts, or internal helper functions after generation.

The generated example may demonstrate a good default, but only wire contracts
needed for interoperability are normative. Application organization stays
editable.

## Wave table

| Wave | Milestone | Sprint | Closure | Target boundary | Owned paths |
|---|---|---|---|---|---|
| 0 | spike | a-1 compatibility spike | contract | compatibility-evidence | `examples/spike/**`, `docs/architecture.md` |
| 1 | core | a-2 config | boundary | sc-config | `crates/sc-config/**`, `boundaries/sc-config/**` |
| 1 | core | a-3 transport | boundary | sc-transport | `crates/sc-transport/**`, `boundaries/sc-transport/**` |
| 1 | core | a-4 command | boundary | sc-command | `crates/sc-command/**`, `boundaries/sc-command/**` |
| 2 | core | a-5 runtime core | integration | runtime-composition | root workspace registries, `crates/sc-runtime/**`, `boundaries/sc-runtime/**`, rewritten spike, core CI/just recipes |
| 3 | template | a-6 template from JSON | integration | template-from-json | `template/**`, schema/fixtures, driver, generation tests and template CI |
| 4 | release | a-7 core release | integration | release-distribution | release workflow/docs, root release metadata, spike removal |
| 4 | wizard | a-8 wizard | integration | wizard | `wizard/wizard.json`, `wizard/pages/**`, wizard runner/tests, narrow driver entry-mode change |

Critical path: **5 sprints**: a-1 → one leaf boundary → a-5 → a-6 → a-7
(or a-8). Width: **3** at wave 1. Sprint count: **8**.

The path exceeds three because the external stack must be proved before the
greenfield public APIs are committed, the runtime must consume the real leaf
crates, and the template must consume the real runtime before release. The
wizard and release are parallel-safe once the template/schema contract is
settled; this preserves the original ability for the wizard to slip without
blocking the core release.

## Dependency edges

| Child | Parent(s) | Relation and named artifact |
|---|---|---|
| a-2, a-3, a-4 | a-1 | `must_follow`: consume verified dependency versions and demonstrated third-party signatures. |
| a-5 | a-2, a-3, a-4 | `must_follow`: consumes the three real public crate facades; speculative facade stubs are intentionally not hoisted. |
| a-6 | a-5 | `must_follow`: consumes the production runtime facade and rewritten-spike composition pattern. |
| a-7 | a-6 | `must_follow`: publishes only after one real generated project is green against the release candidates. |
| a-8 | a-6 | `must_follow`: consumes the settled answers schema and `--var-file` pipeline. |
| a-7 ↔ a-8 | — | `parallel_safe`: disjoint release and wizard paths; wizard does not modify published crates or the template contract. |

Parent development is merged into a child before each development/fix round;
parent PR merges before the child PR. Shared root files have one sequential
owner: a-5 creates the core workspace/CI registries, a-6 adds generator entries,
and a-7 performs release-only cleanup. A discovered leaf-crate defect returns
to a-2, a-3, or a-4 rather than expanding a consumer sprint's ownership.

## Scope reconciliation before hardening

Before `/plan-hardening`, perform a source-only reconciliation pass that:

1. marks the prescribed CLI test architecture and its generated-agent guidance
   as deferred to the attribute-generation follow-up;
2. records the disposition of auto-start without embedding it in the core
   library milestone;
3. removes acceptance criteria that mandate application-internal organization
   rather than an interoperability contract;
4. retains safety constraints for any process tests that remain; and
5. confirms every Phase A REQ/NFR and ADR appears in exactly one sprint's
   authoritative lists or in a named deferred-scope record.

After the sprint cut is approved and before dispatch, create the eight
`.sprints/` `triage:branch` declarations from the exact branch values in the
sprint frontmatter, using the repository tool's verified current format. Do
not invent that format in the plan.

No Beads are created until that reconciliation and plan hardening converge.

## QA consumption

Boundary QA for a-2 through a-4 reviews one crate, its manifest, tests, README,
and boundary manifest. a-5 reviews real composition and the rewritten spike.
a-6 reviews only the minimal generated application and `--var-file` workflow;
it must reject framework-like abstractions that are unnecessary for that proof.
a-7 reviews packaging/registry evidence. a-8 reviews wizard/schema equivalence
without reopening core code.

QA must distinguish normative interoperability contracts from illustrative
template choices. An example layout is not a boundary rule for generated
applications.
