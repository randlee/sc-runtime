# Phase A — core runtime and minimal generator

Status: review iteration 2 fix applied; the external-Dolt development graph
and sprint declarations are recorded. Compatibility evidence and crate-local
implementation decisions remain before their dependent implementation work.

Phase id: `a`. Plan branch: `plan/phase-a`. Phase branch:
`integrate/phase-a`. Beads epic: `run-phase-a`.

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

## Authority and completed source reconciliation

The user-discussed scope and `docs/sc-runtime-design.md` define the intended
milestones. Commit `3e0aace` amended the source requirements and ADRs to
remove the conflicts found in review iteration 1 while preserving their
decision history. This table is the authoritative deferred-scope record for
identifiers absent from sprint frontmatter:

| Source id | Phase A disposition | Justification |
|---|---|---|
| `REQ-RUN-0206`, `ADR-RUN-0204` | Deferred pending placement decision | Default CLI auto-start and launchd-equivalent launch are application policy added after the five-milestone design. They are not needed to prove or publish the reusable core crates. Decide whether they become a later phase or a small optional generated layer after the core release. |
| `REQ-RUN-0312`, `ADR-RUN-0304` | Deferred and architecture reopened | The prescribed two-function CLI testing shape conflicts with the intended attribute-generated testing direction. Implement neither architecture in Phase A; evaluate the attribute layer against the completed core. |
| `REQ-RUN-0311` | Deferred with CLI test generation | Its generated-agent testing prescriptions depend on the application test architecture and should not freeze that architecture ahead of the attribute work. |

`ADR-RUN-0009` / `NFR-RUN-0010` remain active safety constraints: any real
daemon process test runs only on an isolated machine. Phase A adds no VM
definition, provisioning system, test daemon, or host fallback.

The remaining dispatch prerequisites are plan approval, the a-1 compatibility
evidence, and the crate-local decisions assigned below. The two review rounds
have converged and the development records now exist in the repository's
configured shared external Dolt service. No embedded Dolt database is
authorized.

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

## Minimal public contract handoff

The sprint cut intentionally avoids a separate shape-only contract sprint.
Instead, each leaf sprint commits its facade and behavior together; a-5 is not
dispatched until those three facade commits are merged. The following minimal
surface is normative for planning. Error variant details and third-party type
paths are finalized by the owning leaf sprint in its crate-local source ADR,
using a-1's recorded evidence, before implementation proceeds beyond the
facade.

```rust
// sc-config (default features)
pub fn load<T: serde::de::DeserializeOwned>(app: &str)
    -> Result<T, ConfigError>;

impl Loader {
    pub fn new(
        app: &str,
        config_dir: std::path::PathBuf,
        environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    ) -> Self;
    pub fn load<T: serde::de::DeserializeOwned>(&self)
        -> Result<T, ConfigError>;
}

// sc-transport (default features unless marked server)
pub enum Endpoint { Uds(std::path::PathBuf), Tcp(std::net::SocketAddr) }
pub struct TransportConfig {
    pub endpoint: Option<String>,
    pub instance_root: Option<std::path::PathBuf>,
    pub port: Option<u16>,
}
pub fn resolve_endpoint(
    app: &str,
    flag: Option<&std::ffi::OsStr>,
    environment: Option<&std::ffi::OsStr>,
    config: &TransportConfig,
) -> Result<Endpoint, TransportError>;
impl Client {
    pub fn new(app: &str, endpoint: Endpoint) -> Result<Self, TransportError>;
    pub async fn get<T: serde::de::DeserializeOwned>(&self, path: &str)
        -> Result<T, TransportError>;
    pub async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self, path: &str, body: &B,
    ) -> Result<T, TransportError>;
}
// #[cfg(feature = "server")]
pub async fn bind(endpoint: &Endpoint) -> Result<Listener, TransportError>;

// sc-command (default features)
pub enum Envelope<T> { Success { version: String, data: T }, Failure { version: String, error: OpError } }
pub struct OpError {
    pub kind: ErrorKind,
    pub code: sc_observability_types::ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub suggested_action: Option<String>,
}
pub enum ErrorKind { InvalidInput, NotFound, Conflict, Unavailable, Internal }
impl<T> From<Result<T, OpError>> for Envelope<T>;
// server: IntoResponse for Envelope<T> and IntoMcp for Result<T, OpError>
```

`Envelope<T>` uses a custom serde representation with exactly the four wire
keys `version`, `ok`, `data`, and `error`; contradictory/missing discriminant
content is rejected. The contract version is the string `"1"`. Absent details
or suggestion serialize as JSON `null`. HTTP mapping is InvalidInput→400,
NotFound→404, Conflict→409, Unavailable→503, Internal→500. a-1 records the
exact rmcp/Axum type paths before a-4 implements the server conversions.

The runtime contract is the five-step builder table and code sample in
`docs/sc-runtime/requirements.md` REQ-RT-0001: configuration is loaded by the
application; runtime acquires the lock before invoking the async stores
closure; it then builds ordinary routes/MCP and binds last. a-5 pins only the
generic bounds and concrete proved rmcp service type needed to compile that
existing shape. It may not add hooks, wrapper traits, or reorder the steps.

These signatures constrain reusable crate interoperability, not generated
application organization. If a-1 disproves a referenced third-party type,
dependent sprint dispatch stops; the owning leaf sprint records the proved
replacement in its source ADR and this handoff is corrected before code lands.

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
| 1 | core | a-2 config | boundary | sc-config | `crates/sc-config/**`, `boundaries/sc-config/**`, sc-config requirement/ADR sources |
| 1 | core | a-3 transport | boundary | sc-transport | `crates/sc-transport/**`, `boundaries/sc-transport/**`, sc-transport requirement/ADR sources |
| 1 | core | a-4 command | boundary | sc-command | `crates/sc-command/**`, `boundaries/sc-command/**`, sc-command requirement/ADR sources |
| 2 | core | a-5 runtime core | integration | runtime-composition | root workspace registries, `crates/sc-runtime/**`, `boundaries/sc-runtime/**`, sc-runtime requirement/ADR sources, rewritten spike, core CI/just recipes |
| 3 | template | a-6 template from JSON | integration | template-from-json | `template/**`, schema/fixtures, driver, root `justfile` generation entry, generation tests and template CI |
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
owner: a-5 creates the core workspace/CI registries, a-6 owns the sequential
root `justfile` edit that adds `just new` and automatic generation/wizard test
discovery,
and a-7 performs release-only cleanup. A discovered leaf-crate defect returns
to a-2, a-3, or a-4 rather than expanding a consumer sprint's ownership.

## Reconciliation and dispatch record

The first review fix (`3e0aace`) deferred `REQ-RUN-0206`, `ADR-RUN-0204`,
`REQ-RUN-0311`, `REQ-RUN-0312`, and `ADR-RUN-0304`; retained the process
safety constraints; and reconciled the active CLI behavior to report-only.
Iteration 2 removes the remaining active reference to the deferred CLI helper
shape and assigns every crate-local source decision to its implementing
sprint. The mechanical inventory is 120 source IDs: 115 represented in sprint
frontmatter and the five IDs above represented only in the deferred-scope
table.

The plan-to-development handoff is now recorded in both durable systems:

| Sprint | Bead | Declared branch | Predecessor beads |
|---|---|---|---|
| a-1 | `run-a-1` | `sprint/a-1-compatibility-spike` | — |
| a-2 | `run-a-2` | `sprint/a-2-config` | `run-a-1` |
| a-3 | `run-a-3` | `sprint/a-3-transport` | `run-a-1` |
| a-4 | `run-a-4` | `sprint/a-4-command` | `run-a-1` |
| a-5 | `run-a-5` | `sprint/a-5-runtime-core` | `run-a-2`, `run-a-3`, `run-a-4` |
| a-6 | `run-a-6` | `sprint/a-6-template-from-json` | `run-a-5` |
| a-7 | `run-a-7` | `sprint/a-7-core-release` | `run-a-6` |
| a-8 | `run-a-8` | `sprint/a-8-wizard` | `run-a-6` |

All eight records are open, unclaimed children of `run-phase-a`; dependency
edges enforce the wave graph above. `.sprints/a/structure.ttl` declares the
same sprint ids, branches, order, and criteria documents. Development does not
begin merely because the records exist: dispatch claims the ready bead only
after this plan is approved and its predecessor evidence is satisfied.

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
