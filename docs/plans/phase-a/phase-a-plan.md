# Phase A — core runtime and minimal generator

Status: plan-hardening pass 2 in progress (scope PASS at STEP2-R2).
RBP sprint-doc pass on this branch assigns findings RBP-F001–F008 (plan
review on develop `5240774`) to sprint deliverables and acceptance criteria;
formal QA-1 has not run. The external-Dolt development graph and sprint
declarations are recorded. sc-lint 0.8.0 is in scope (a-5 maintainer, a-6
consumer). a-1 amends the observability pin and the sc-lint consumer model
in `docs/architecture.md` and `docs/requirements.md` to the versions the
spike records (1.5.0 on crates.io as of this plan). Active 1.2.x sentences
are edited in that sprint, not left in place. Compatibility evidence and
crate-local implementation decisions remain before dependent implementation
work.

Phase id: `a`. Plan branch: `plan/phase-a`. Phase branch:
`integrate/phase-a`. Beads epic: `run-phase-a`.

The owner explicitly selected the existing single-letter Phase A name. This is
a grandfathered exception to the guideline's default next-unused letter-pair
rule; all new plan files, ids, branches, and worktrees consistently use lower
case `a`.

## Intent

**Framework posture:** Phase A ships reusable **published crates** and a
**cargo-generate template** consumed by many unrelated projects—not a single
product application. Normative framing is in
[`docs/framework-charter.md`](../../framework-charter.md): agent-first JSON
CLI, **mandatory actionable errors** (stable codes plus remediation on every
cross-boundary failure), MCP on the shared daemon router, SQLite in the example
store with Postgres as an add-on store crate, and **minimal scaffolding**
without registries or application-framework opinions inside the libraries.

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
decision history. This table is the authoritative deferred-scope record. The
first three rows are identifiers absent from sprint frontmatter. The
`REQ-RUN-0203` row defers only the named OPEN; that requirement's
operation-envelope obligations stay on a-6.

| Source id | Phase A disposition | Justification |
|---|---|---|
| `REQ-RUN-0206`, `ADR-RUN-0204` | Deferred pending placement decision | Default CLI auto-start and launchd-equivalent launch are application policy added after the five-milestone design. They are not needed to prove or publish the reusable core crates. Decide whether they become a later phase or a small optional generated layer after the core release. |
| `REQ-RUN-0312`, `ADR-RUN-0304` | Deferred and architecture reopened | The prescribed two-function CLI testing shape conflicts with the intended attribute-generated testing direction. Implement neither architecture in Phase A; evaluate the attribute layer against the completed core. |
| `REQ-RUN-0311` | Deferred with CLI test generation | Its generated-agent testing prescriptions depend on the application test architecture and should not freeze that architecture ahead of the attribute work. |
| `REQ-RUN-0203` (framework-generated responses OPEN) | Deferred for v0.1 | Unknown route, Axum body rejection, and an rmcp protocol `Err(McpError)` stay on Axum and rmcp defaults. They are not part of the v0.1 agent contract. a-6 closes operation success and operation-failure envelope equality only. |

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
independent leaf-crate boundaries. Those three stay parallel-safe with one
another. a-2 starts in wave 0 beside a-1, because its facade consumes no spike
signature. a-3 and a-4 stay in wave 1, because they consume the spike.
Everything else stays at the original milestone granularity. sc-lint 0.8.0 is
not a ninth sprint; see [Boundary and lint tooling](#boundary-and-lint-tooling).

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
| `sc-config` | typed JSON-file plus environment loading | `serde`, `serde_json`; consumed by generated applications only (`sc-runtime` does not depend on `sc-config`) |
| `sc-transport` | endpoint resolution, listener binding, generic HTTP client, typed not-running error | Tokio/reqwest; Axum only under `server`; consumed by runtime and generated CLI |
| `sc-command` | lossless envelope and standard REST/MCP conversions | serde and observability types; Axum/rmcp only under `server`; consumed by runtime and generated application |
| `sc-runtime` | daemon assembly, singleton lock, graceful shutdown, in-process `DaemonFixture` | consumes the settled `sc-transport` and `sc-command` facades and ordinary Axum/rmcp types; no dependency on `sc-config`; owns no application registry |
| `template-from-json` | answers JSON to one editable generated application | consumes published crate APIs; no new reusable framework boundary |
| `wizard` | interactive/prefill UI to the same answers JSON | consumes schema and generation driver; remains outside generated output |

Boundary manifests and repository linting protect this repository's four
libraries. They are not copied into generated applications and must not force
application structure.

## Minimal public contract handoff

The sprint cut intentionally avoids a separate shape-only contract sprint.
Instead, each leaf sprint commits its facade and behavior together. a-5 is not
dispatched until the a-3 and a-4 facade commits are merged. a-2 is not on that
join. The following minimal surface is normative for planning. Each leaf sprint
publishes an **error inventory** (failure mode, stable code, cause payload,
recovery steps) in its crate-local ADR **before** the public facade is frozen
(RBP-F002). Third-party type paths are finalized from a-1 evidence.

`AppName` is a crate-local validated newtype. There is no shared `AppName`
type and no fifth crate in Phase A. The three types are `sc_config::AppName`,
`sc_transport::AppName`, and `sc_runtime::AppName`. No leaf depends on another
to obtain its newtype, and `sc-runtime` does not depend on `sc-config` for it.
Construction is the only validation site. Display uses `as_str` or
`AsRef<str>`. The type does not implement `Deref` (RBP-F005). a-6 converts
only at the application boundary: `as_str()` into each crate's constructor.

One validation predicate applies to all three constructors:

- allowed characters are ASCII `A-Z`, `a-z`, `0-9`, `-`, and `_`;
- length is 1 through 64 bytes;
- rejection uses the stable code string `APP.INVALID_NAME` on that crate's
  own error type.

Accept vectors: `my-app`, `A`, `app_1`, and 64 `a` characters. Reject
vectors: the empty string, `my app`, `my.app`, `my/app`, 65 `a` characters,
and `café`. a-2, a-3, and a-5 tests use these vectors. Each of those sprints
amends the `&str` application-name parameters in the crate requirement file
it already owns.

```rust
// Shape of each crate-local newtype. Not one shared type.
// sc_config::AppName, sc_transport::AppName, sc_runtime::AppName
pub struct AppName(/* validated; TryFrom<&str> or constructor returns the crate error */);
impl AppName {
    pub fn as_str(&self) -> &str;
}
// AsRef<str> for display. Deref is not implemented.

// sc-config (default features) — app is sc_config::AppName
pub fn load<T: serde::de::DeserializeOwned>(app: AppName)
    -> Result<T, ConfigError>;

impl Loader {
    pub fn new(
        app: AppName,
        config_dir: std::path::PathBuf,
        environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    ) -> Self;
    pub fn load<T: serde::de::DeserializeOwned>(&self)
        -> Result<T, ConfigError>;
}

// sc-transport (default features unless marked server)
// app is sc_transport::AppName
pub enum Endpoint { Uds(std::path::PathBuf), Tcp(std::net::SocketAddr) }
// a-3 freezes TransportConfig's field list in docs/sc-transport/architecture.md
// before the facade freezes. a-5 DaemonConfig embeds that sc-transport type.
// It does not decide a second endpoint-config shape.
pub struct TransportConfig { /* fields frozen in the crate ADR */ }
pub fn resolve_endpoint(
    app: AppName,
    flag: Option<&std::ffi::OsStr>,
    environment: Option<&std::ffi::OsStr>,
    config: &TransportConfig,
) -> Result<Endpoint, TransportError>;
impl Client {
    pub fn new(app: AppName, endpoint: Endpoint) -> Result<Self, TransportError>;
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, TransportError>;
    pub async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, TransportError>;
}
// #[cfg(feature = "server")]
pub async fn bind(endpoint: &Endpoint) -> Result<Listener, TransportError>;
// TransportError::DaemonNotRunning: code = sc_observability_types::ErrorCode
// (DAEMON.NOT_RUNNING); suggested_action = String (`run <app> daemon start`
// with the AppName passed to Client::new). No Remediation type on transport.

// sc-command (default features)
pub enum Envelope<T> { Success { version: String, data: T }, Failure { version: String, error: OpError } }
pub struct OpError {
    pub kind: ErrorKind,
    pub code: sc_observability_types::ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
    // None, or a documented empty variant that serializes the same way.
    pub remediation: Option<sc_observability_types::Remediation>,
}
pub enum ErrorKind { InvalidInput, NotFound, Conflict, Unavailable, Internal }
impl<T> From<Result<T, OpError>> for Envelope<T>;
// server: IntoResponse for Envelope<T>; sealed IntoMcp for Result<T, OpError> only (static dispatch)
```

`OpError` has one shape that satisfies the wire rule. The Rust field may be
absent (`Option<sc_observability_types::Remediation>`, or a documented empty
variant). A custom serializer writes the wire key `suggested_action` as a
JSON string when the value is present and as JSON `null` when it is absent.
The tagged `Remediation` object is never the wire form. `details` is
`Option<serde_json::Value>` and is JSON `null` when absent. a-4 records this
decision in ADR-CMD-0001 and REQ-CMD-0002, which it owns, and deletes that
OPEN. This handoff is the shape a-4 implements. It is not a claim that the
crate ADR already says this (RBP-F001).

The a-4 crate ADR holds the library wire-failure inventory before the facade
freezes. The rows are only `sc-command`'s own wire failures: an invalid or
contradictory envelope discriminant, serialization failure, and the five
`ErrorKind` mappings. Each row has a stable code, cause, recovery steps, and
a docs link where one exists. Widget create/get and any other application
operation codes live in the generated service. They are not rows in
ADR-CMD-0001. a-6 does not extend that inventory when the example operation
is added.

`Envelope<T>` uses a custom serde representation with exactly the four wire
keys `version`, `ok`, `data`, and `error`; contradictory/missing discriminant
content is rejected. The contract version is the string `"1"`. Absent details
or suggestion serialize as JSON `null`. HTTP mapping is InvalidInput→400,
NotFound→404, Conflict→409, Unavailable→503, Internal→500. a-1 records the
exact rmcp/Axum type paths before a-4 implements the server conversions.

The runtime contract is the builder sequence in REQ-RT-0001.
`Daemon::builder` takes `sc_runtime::AppName`. a-5 chooses **consuming
typestate** (`stores` → `routes` → optional `mcp` → `run`) so omitting a
required step is unrepresentable; if typestate is rejected in the ADR, a-5
adds an explicit `RuntimeError` for a missing required closure (RBP-F007).
`RuntimeError::StoresFailed` is non-generic. Its payload is owned by
`sc-runtime`: a stable code, a message, remediation, and optional owned cause
text. The application maps its store error into that variant inside the
stores closure. `sc-runtime` does not name or match the foreign error type.
`Box<dyn Error>` and generic `RuntimeError<E>` are rejected (RBP-F003).
Configuration is loaded by the application. `sc-runtime` does not depend on
`sc-config` and consumes no type from it. Runtime acquires the lock before
stores, then routes/MCP, then bind. a-5 may not add wrapper traits or reorder
steps inside `run`.

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
  cargo-generate and sc-compose, followed by the generated project's consumer
  `just setup` and `just lint`, then its own test command.
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
- prototype-repository handoff to the sc-lint team, VM provisioning, or a
  custom test daemon;
- enforcement that an application keep the example's crate graph, route names,
  operation names, pool counts, or internal helper functions after generation.

The generated example may demonstrate a good default, but only wire contracts
needed for interoperability are normative. Application organization stays
editable. Generated projects do receive the sc-lint 0.8.0 consumer install
described under Boundary and lint tooling. They do not receive this
repository's boundary manifests.

## Boundary and lint tooling

sc-lint 0.8.0 is the lint and `just` system for this phase. Two roles stay
distinct, matching the sc-lint source-maintainer and consumer docs.

This repository is the source maintainer. a-5 owns that install. The root
`justfile` is maintained here and is not produced by `sc-lint init --just`.
Its public recipes include `setup`, `lint`, `test`, and `upgrade`, and they
call `.sc-lint/bootstrap.py` with `sc-lint.toml`. `just lint` includes
`sc-lint-boundary analyze` over `boundaries/<crate>/*.toml` for the four
library crates (REQ-RUN-0005) and the sc-lint line-counts check at 1000
source lines, blank and comment-only lines excluded (REQ-RUN-0004,
NFR-RUN-0008).
This checkout does not build sc-lint, so `setup` installs the verified 0.8.0
release. Maintainer profiles stay in this `justfile`; they are not inferred
from a directory name.

Generated applications are consumers. a-6 installs that contract into the
template with `sc-lint init --just` at sc-lint 0.8.0. The committed consumer
files are `sc-lint.toml`, `.sc-lint/bootstrap.py`, and a `Justfile` whose
public recipes are `setup`, `lint`, `test`, and `upgrade`. `db-prepare` stays
a consumer-owned recipe beside those four (REQ-RUN-0307). The template
carries that product-owned install, not a hand-written lint configuration,
a `boundaries/` tree, or a `just` import (NFR-RUN-0006). Fixture CI runs
`just setup && just lint`
in each generated project. a-7's release gate checks that the template pin
remains sc-lint 0.8.0.

No sprint a-0 is added. The install is not its own boundary, and it creates
no parallel work. a-1 is disposable evidence and must not become the permanent
lint install. The root `justfile` already has one sequential owner. Putting
the maintainer install in a-5 and the consumer install in a-6 keeps the
critical path at five sprints and the sprint count at eight.

## CLI/server feature enforcement

`sc-lint-boundary` enforces inter-crate edges and facade lists only; it has
no cargo-feature key (ADR-RUN-0003). Server-only items (`bind`, `Listener`,
sealed `IntoMcp`, Axum/rmcp conversions) stay behind the `server` feature and
**off the default facade** checked by boundary lint.

| Sprint | Responsibility |
|---|---|
| a-1 | Measure default vs `server` graphs; **close** ADR-RUN-0003 workspace-unification OPEN and NFR-RUN-0001 first OPEN: `cargo build -p cli` is normative, workspace-wide build is exempt, **no** client/server crate split in v0.1. |
| a-5 | Workspace composition only. Records `sc-runtime` → `sc-config` MAY as **edge absent** in `docs/architecture.md`. Does not run `cargo tree -p cli` (a-6 owns that). |
| a-6 | Sole owner of the NFR-RUN-0001 feature-level close. Generated fixture CI runs `cargo tree -p cli -e normal`. `cargo build -p cli` is the normative selection. A workspace-wide build is the documented exemption. Named server-only crates are absent from that tree. |
| a-7 | Release evidence under `req:REQ-RUN-0702` repeats the a-6 check on the registry-backed fixture. It is not a second `req:NFR-RUN-0001` root. |

a-6 does not wait for a-5 to close ADR-RUN-0003 (RBP-F004). a-7 keeps the
command output as release evidence under REQ-RUN-0702.

## Observability placement

ADR-RUN-0004 stays the placement rule. The generated daemon initialises
logging after configuration. The four library crates do not take the logging
runtime.

Generated `crates/daemon` `main` loads configuration with `sc-config`,
initialises `sc-observability` with plain values from that config, then calls
`Daemon::builder`. The template depends on the full `sc-observability` crate
for that call. The template does not depend on `sc-observability-otlp`.
OpenTelemetry export stays an optional project addition and is deferred.

`sc-config`, `sc-transport`, `sc-command`, and `sc-runtime` do not depend on
`sc-observability` or `sc-observability-otlp`, and they do not initialise or
wrap logging. `sc-command` depends on `sc-observability-types` for `OpError`.
`sc-transport` depends on `sc-observability-types` for `ErrorCode` on
`DaemonNotRunning`. That edge is types-only: it is not an edge to
`sc-command` and it is not a dependency on the logging runtime. `sc-config`
stays on `serde` and `serde_json`. `sc-runtime` does not add a direct logging
dependency; shared codes it needs come through `sc-command`.

Version policy: a-1 records the exact `sc-observability` and
`sc-observability-types` versions the spike resolves. The pin is the latest
crates.io release at spike time (1.5.0 as of this plan). a-1 amends
ADR-RUN-0004 and REQ-RUN-0309 so that recorded pair replaces the Active
1.2.x sentences before a-3, a-4, and a-6 are dispatched. a-4, a-6, and a-7
use that recorded pair.

| Sprint | Responsibility |
|---|---|
| a-1 | Depend on both crates in the spike, record the exact resolved versions in `docs/architecture.md`, and amend ADR-RUN-0004 and REQ-RUN-0309 to that pair. |
| a-4 | `sc-command` depends on the recorded `sc-observability-types` version and not on `sc-observability`. |
| a-6 | Generated daemon depends on the recorded `sc-observability` version; init order is config, then observability, then `Daemon::builder`; no OTLP crate in the template. |
| a-7 | Release gate checks the template requirement and the registry-backed fixture lockfile against the a-1 record. |

## Wave table

| Wave | Milestone | Sprint | Closure | Target boundary | Owned paths |
|---|---|---|---|---|---|
| 0 | spike | a-1 compatibility spike | contract (evidence record) | compatibility-evidence | `examples/spike/**`, `docs/architecture.md`, `docs/requirements.md` |
| 0 | core | a-2 config | boundary | sc-config | `crates/sc-config/**`, `boundaries/sc-config/**`, sc-config requirement/ADR sources |
| 1 | core | a-3 transport | boundary | sc-transport | `crates/sc-transport/**`, `boundaries/sc-transport/**`, sc-transport requirement/ADR sources |
| 1 | core | a-4 command | boundary | sc-command | `crates/sc-command/**`, `boundaries/sc-command/**`, sc-command requirement/ADR sources |
| 2 | core | a-5 runtime core | integration | runtime-composition | root workspace registries, source-maintainer `sc-lint.toml` and `.sc-lint/bootstrap.py`, `crates/sc-runtime/**`, `boundaries/sc-runtime/**`, sc-runtime requirement/ADR sources, the `docs/architecture.md` edit that records the `sc-runtime` → `sc-config` edge as absent, rewritten spike, core CI/just recipes |
| 3 | template | a-6 template from JSON | integration | template-from-json | `template/**` including the sc-lint 0.8.0 consumer install, schema/fixtures, driver, root `justfile` generation entry, generation tests and template CI |
| 4 | release | a-7 core release | integration | release-distribution | authored `release/sc-publish-*`, `docs/development/sc-publish.md`, `docs/architecture.md` release-section edit, install.py workflow output, root release metadata, spike removal |
| 4 | wizard | a-8 wizard | integration | wizard | `wizard/wizard.json`, `wizard/pages/**`, wizard runner/tests, narrow driver entry-mode change |

`contract` on a-1 means the disposable compatibility-evidence record later
sprints consume. QA uses a-1's validation section. It does not use a
contract-sprint checklist of API signatures, test doubles, and boundary
manifests.

Critical path: **5 sprints**: a-1 → a-3 or a-4 → a-5 → a-6 → a-7 (or a-8).
Scheduled width: **2** (wave 0 is a-1 and a-2; wave 1 is a-3 and a-4). a-2 is
`parallel_safe` with a-3 and a-4, so the three leaf crates still do not block
one another. Sprint count: **8**.

The path exceeds three because a-3 and a-4 must consume the proved external
stack before their public APIs are committed, the runtime must consume those
real leaf crates, and the template must consume the real runtime before
release. a-2 is not on that path: `sc-config` consumes no spike signature, and
`sc-runtime` consumes no `sc-config` type. The wizard and release are
parallel-safe once the template, answers schema, and `--var-file` driver are
settled; this preserves the original ability for the wizard to slip without
blocking the core release.

## Dependency edges

| Child | Parent(s) | Relation and named artifact |
|---|---|---|
| a-2 | — | `must_follow` is empty. `parallel_safe` with a-1, a-3, and a-4. The sc-config facade is `serde` / `serde_json` plus `sc_config::AppName`. It consumes no spike signature. |
| a-3, a-4 | a-1 | `must_follow`: consume demonstrated reqwest/Axum/rmcp signatures and the recorded `sc-observability-types` version. |
| a-5 | a-3, a-4 | `must_follow`: consumes endpoint and listener types from `sc-transport`, and the `sc-command` server conversions. `parallel_safe` with a-2. `sc-runtime` does not depend on `sc-config` and consumes no type from it. Root `just lint` is the first `sc-lint-boundary analyze` of the manifests on the branch. |
| a-6 | a-5 | `must_follow`: consumes the production runtime facade and rewritten-spike composition pattern. Sole owner of the NFR-RUN-0001 `cargo tree -p cli -e normal` close. |
| a-7 | a-6 | `must_follow`: consumes the template tree, the answers schema, and the `--var-file` driver. Publishes only after that generated project is green against the release candidates. |
| a-8 | a-6 | `must_follow`: consumes the settled answers schema and `--var-file` pipeline. |
| a-1 ↔ a-2 | — | `parallel_safe`: spike evidence and the sc-config crate do not share owned paths. |
| a-7 ↔ a-8 | — | `parallel_safe`: disjoint release and wizard paths; wizard does not modify published crates or the template contract. a-8 does not wait for or diff against a-7. |

Parent development is merged into a child before each development/fix round;
parent PR merges before the child PR. Shared root files have one sequential
owner. **`docs/architecture.md` sequential owners:** a-1 (compatibility and
ADR amendments), a-5 (`sc-runtime` → `sc-config` edge absent only), a-7
(release section: sc-publish kit sole workflow owner; GitHub Release includes
`sc-runtime-{version}-source.tar.gz`). a-1 owns `docs/requirements.md` for its
amendment tasks. a-2, a-3, and a-4 amend crate-local ADRs only. a-5 creates
the core workspace/CI registries and the source-maintainer sc-lint install.
a-6 owns the sequential root `justfile` edit that adds `just new` and the
template consumer install. a-7 performs release cleanup and kit install. The sc-lint 0.8.0 pin, the
a-6 CLI isolation command, and the a-1 observability versions are release
evidence under REQ-RUN-0702, not second acceptance roots. A discovered
leaf-crate defect returns to a-2, a-3, or a-4 rather than expanding a consumer
sprint's ownership. a-6 must not replace the maintainer `justfile` with the
generated consumer file.

## Reconciliation and dispatch record

The first review fix (`3e0aace`) deferred `REQ-RUN-0206`, `ADR-RUN-0204`,
`REQ-RUN-0311`, `REQ-RUN-0312`, and `ADR-RUN-0304`; retained the process
safety constraints; and reconciled the active CLI behavior to report-only.
Iteration 2 removes the remaining active reference to the deferred CLI helper
shape and assigns every crate-local source decision to its implementing
sprint. The mechanical inventory is 120 source IDs: 115 represented in sprint
frontmatter and the five IDs in the first three deferred-scope rows
represented only there. The REQ-RUN-0203 framework-response OPEN is deferred
in the same table and is not a sixth fully deferred id.

The plan-to-development handoff is now recorded in both durable systems:

| Sprint | Bead | Declared branch | Predecessor beads |
|---|---|---|---|
| a-1 | `run-a-1` | `sprint/a-1-compatibility-spike` | — |
| a-2 | `run-a-2` | `sprint/a-2-config` | — |
| a-3 | `run-a-3` | `sprint/a-3-transport` | `run-a-1` |
| a-4 | `run-a-4` | `sprint/a-4-command` | `run-a-1` |
| a-5 | `run-a-5` | `sprint/a-5-runtime-core` | `run-a-3`, `run-a-4` |
| a-6 | `run-a-6` | `sprint/a-6-template-from-json` | `run-a-5` |
| a-7 | `run-a-7` | `sprint/a-7-core-release` | `run-a-6` |
| a-8 | `run-a-8` | `sprint/a-8-wizard` | `run-a-6` |

All eight records are open, unclaimed children of `run-phase-a`. The
predecessor column is the plan graph after this hardening pass. Bead
dependency edges are aligned to that column before dispatch.
`.sprints/a/structure.ttl` declares the sprint ids, branches, and criteria
documents; its order edges are aligned to this table before dispatch.
Development does not begin merely because the records exist: dispatch claims
the ready bead only after this plan is approved and its predecessor evidence
is satisfied.

## QA consumption

Plan and sprint **QA-1** include `extensibility-qa` (adversarial multi-consumer
review against `docs/framework-charter.md`). It does **not** rerun on QA-2+
fix rounds. `arch-qa` must read the framework charter and
`docs/sc-runtime-design.md` for every phase plan review.

Boundary QA for a-2 through a-4 reviews one crate, its manifest, tests, README,
and boundary manifest. Those three manifests are written to pass
`sc-lint-boundary analyze`, which first runs in a-5 once root lint exists.
a-5 reviews real composition, the rewritten spike, and the source-maintainer
sc-lint 0.8.0 install. a-6 reviews only the minimal generated application,
the consumer install, and the `--var-file` workflow; it must reject
framework-like abstractions that are unnecessary for that proof, and it must
reject copying `boundaries/` into the template. a-7 reviews packaging,
registry evidence, and the REQ-RUN-0702 release re-check of the sc-lint 0.8.0
pin, the a-6 CLI isolation command, and the a-1 observability version pins in
the template and fixture lockfile. Those re-checks are not second acceptance
roots.
a-8 reviews wizard/schema equivalence without reopening core code.

QA must distinguish normative interoperability contracts from illustrative
template choices. An example layout is not a boundary rule for generated
applications.
