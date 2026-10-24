# Phase A — core runtime and minimal generator

Status: sc-lint 0.8.0 is in scope (a-5 maintainer, a-6 consumer). a-1 records
the observability pin (1.5.0 on crates.io as of this plan) and amends Active
1.2.x sentences before dependent sprints. Parents are the dependency-edges
table. Compatibility evidence and crate-local decisions remain before
dependent implementation work.

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
a-6 publishes the generation-driver error inventory: a stable code, cause,
and recovery for schema rejection, a reserved `db` value, a missing tool,
and a failed pipeline step.

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
dispatched until a-2, a-3, and a-4 have merged. a-5 registers a-2's crate and
boundary manifest. It consumes no `sc-config` type. The following block is
**planning input**. On merge, the owning crate ADR and crate requirement
supersede it. a-1 amends root ADRs so they point at those crate documents.
It does not copy field-level signatures into `docs/architecture.md`. Each leaf sprint
publishes an **error inventory** (failure mode, stable code, cause payload,
recovery steps) in its crate-local ADR **before** the public facade is frozen
(RBP-F002). Third-party type paths are finalized from a-1 evidence.

`AppName` is a crate-local validated newtype. There is no shared `AppName`
type and no fifth crate in Phase A. The three types are not redundant copies
of one identity:

- `sc_config::AppName` is the configuration identity. Loaders take it.
  `sc-config` does not depend on `sc-transport` or `sc-runtime` to obtain it.
- `sc_transport::AppName` is the client wire identity. `resolve_endpoint`
  and `instance_root` take it. `Client::new` takes an `Endpoint` only.
  Transport does not render suggested-action text. It is not a daemon
  builder argument.
- `sc_runtime::AppName` is the daemon and application identity for builder
  paths (instance root and lock). `Daemon::builder` takes this type.
  `sc-runtime` must not re-export `sc_transport::AppName` and must not put
  that transport type on the builder surface. Inside `run`, `sc-runtime`
  converts with `as_str()` into `sc_transport::AppName` and calls
  `instance_root`. The shared predicate makes a name that constructed the
  runtime type a valid transport name. That conversion is not a second
  predicate and is not a public re-export. If the transport constructor
  returns `Err`, `run` maps it to the `RuntimeError` variant in the a-5
  ADR-RT-0004 inventory. It does not use `unreachable!`.

No leaf depends on another to obtain its newtype, and `sc-runtime` does not
depend on `sc-config` for it.
Construction is the only validation site. Display uses `as_str` or
`AsRef<str>`. The type does not implement `Deref` (RBP-F005). a-6 converts
only at the application boundary: `as_str()` into each crate's constructor.

One validation predicate applies to all three constructors:

- allowed characters are ASCII `A-Z`, `a-z`, `0-9`, `-`, and `_`;
- length is 1 through 64 bytes;
- rejection uses the stable code string `APP.INVALID_NAME` on that crate's
  own error type.

The only enumeration of accept and reject vectors is
`tests/fixtures/app-name-vectors.json`, which a-1 writes and owns. Accept
strings in that file are `my-app`, `A`, `app_1`, and 64 `a` characters.
Reject strings are the empty string, `my app`, `my.app`, `my/app`, 65 `a`
characters, and `café`. ADR-RUN-0010 references that file and does not paste
the vectors. a-2, a-3, a-5, and a-6 tests load the file. They do not embed a
second list and they do not copy the vectors into sprint docs. The wizard
and the generation driver load the same file. Crate packaging excludes it.
Each leaf sprint amends the `&str` application-name parameters in the crate
requirement file it already owns. a-1 records this family, including the
three identities, the no-re-export rule above, the fixture path, and the
rejected alternative that `Daemon::builder` take `sc_transport::AppName`
directly, as new ADR-RUN-0010 in `docs/architecture.md` before a-2, a-3, or
a-5 freezes a facade. ADR-RUN-0006 stays the typed-error rule. a-2's
constructor freeze waits on that Active ADR. The wait is the gate bead
`triage:a-2-appname-freeze` in `.sprints/a/structure.ttl`. That bead
`triage:follows` a-1. It is not a sprint `must_follow` edge and it does not
move a-2 off wave 0.
The generation driver refuses the reject vectors with `APP.INVALID_NAME`
before it creates a destination.

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
// resolve_endpoint and instance_root take sc_transport::AppName.
// Client::new takes an Endpoint only.
pub enum Endpoint { Uds(std::path::PathBuf), Tcp(std::net::SocketAddr) }
// a-3 freezes TransportConfig's field list in docs/sc-transport/architecture.md
// before the facade freezes. a-5 DaemonConfig embeds that sc-transport type.
// It does not decide a second endpoint-config shape.
pub struct TransportConfig { /* fields frozen in the crate ADR */ }
pub fn instance_root(app: AppName) -> std::path::PathBuf;
pub fn resolve_endpoint(
    app: AppName,
    flag: Option<&std::ffi::OsStr>,
    environment: Option<&std::ffi::OsStr>,
    config: &TransportConfig,
) -> Result<Endpoint, TransportError>;
impl Client {
    pub fn new(endpoint: Endpoint) -> Result<Self, TransportError>;
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
// (DAEMON.NOT_RUNNING); fields are the endpoint and the cause only.
// No suggested_action string and no Remediation type on transport.
// a-6's generated CLI mapping sets suggested_action for every TransportError.

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

a-6 ships a generated `daemon start` subcommand that execs the daemon binary.
The generated CLI mapping owns `suggested_action` for every `TransportError`,
including `run <app> daemon start` on `DaemonNotRunning`. Transport carries
the endpoint and the cause only. That subcommand is not auto-start.
a-6 closes the REQ-RUN-0202 OPEN.
a-3 freezes `TransportConfig` as v0.1 single-bind and reserves additive
`additional_endpoints` (default empty, reserved and ignored in v0.1) before
the facade freezes. The a-3 ADR amendment states that ignore. a-5 embeds
that type and closes the runtime and REQ-RUN-0205 OPENs to that decision.

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
a docs link where one exists. Widget create/get codes live in the generated
service. They are not rows in ADR-CMD-0001. a-6 deliverable 17 is the only
copy of that operation error inventory, including its golden vectors. This
handoff does not repeat the table. v0.1 has no other widget application
code. A duplicate name is allowed. Store failures stay
`RuntimeError::StoresFailed` inside the stores closure. They are not widget
codes.

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
`sc-runtime` as plain types: code is `&'static str`, remediation is `String`,
message is `String`, and optional cause text is `String`. There is no
`sc-observability-types` item on the `sc-runtime` facade or in its
dependencies. The application maps its store error into that variant inside the
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

This repository is the source maintainer. a-5 owns that install. a-5 verifies
and extends the existing root `Justfile` (capital J). a-5 also amends the
REQ-RUN-0001 layout table so that cell is `Justfile`, not `justfile`. Its first line stays
the sc-lint init header already on that file. a-5 does not
replace that file or contradict the header. Public recipes `setup`, `lint`,
`test`, and `upgrade` stay in the sc-lint-managed block and call
`.sc-lint/bootstrap.py` with `sc-lint.toml`. Repo-specific recipes stay in
the section init does not regenerate. `just lint` includes
`sc-lint-boundary analyze` over `boundaries/<crate>/*.toml` for the four
library crates (REQ-RUN-0005) and the sc-lint line-counts check at 1000
source lines, blank and comment-only lines excluded (REQ-RUN-0004,
NFR-RUN-0008). a-5 closes that check for `crates/*/src` only. a-6 adds
`template/**/*.rs` after the generation tree exists.
This checkout does not build sc-lint, so `setup` installs the verified 0.8.0
release. Maintainer profiles stay in this `Justfile`; they are not inferred
from a directory name.

Generated applications are consumers. a-6 installs that contract into the
template with `sc-lint init --just` at sc-lint 0.8.0. The committed consumer
files are `sc-lint.toml`, `.sc-lint/bootstrap.py`, and a `Justfile` whose
public recipes are `setup`, `lint`, `test`, and `upgrade`. `db-prepare` and
`build-cli` stay consumer-owned recipes beside those four (REQ-RUN-0307,
NFR-RUN-0001). The template carries that product-owned install, not a
hand-written lint configuration, a `boundaries/` tree, or a `just` import
(NFR-RUN-0006). Fixture CI runs the amended REQ-RUN-0701 sequence. a-7 runs
that same sequence in release mode.

No sprint a-0 is added. The install is not its own boundary, and it creates
no parallel work. a-1 is disposable evidence and must not become the permanent
lint install. The root `Justfile` already has one sequential owner. Putting
the maintainer install in a-5 and the consumer install in a-6 keeps the
critical path at five sprints and the sprint count at eight.

## CLI/server feature enforcement

`sc-lint-boundary` enforces inter-crate edges and facade lists only; it has
no cargo-feature key (ADR-RUN-0003). Server-only items (`bind`, `Listener`,
sealed `IntoMcp`, Axum/rmcp conversions) stay behind the `server` feature and
**off the default facade** checked by boundary lint. This table is the one
statement of the `sc-runtime` → `sc-config` edge-absent decision and of the
NFR-RUN-0001 `cargo tree -p cli` close. Sprint text points here.

| Sprint | Responsibility |
|---|---|
| a-1 | Measure default vs `server` graphs; **close** ADR-RUN-0003 workspace-unification OPEN and NFR-RUN-0001 first OPEN: `cargo build -p cli` is normative, workspace-wide build is exempt, **no** client/server crate split in v0.1. |
| a-5 | Workspace composition only. Records `sc-runtime` → `sc-config` as **edge absent** in every architecture edge table, the REQ-RUN-0005 slice, and `sc-runtime` `forbidden_edges`. Does not run `cargo tree -p cli` (a-6 owns that). |
| a-6 | Sole owner of the NFR-RUN-0001 feature-level close. Fixture CI runs `just` recipes, not a manual check: `just build-cli` includes `cargo tree -p cli -e normal`, and `just test` includes the observability grep and `cargo tree -p daemon -e normal` with no `test-util`, `tempfile`, or `tempdir` line. A workspace-wide build is the documented exemption and is not the shipped agent binary. |
| a-7 | REQ-RUN-0702 is the registry-backed fixture running a-6's fixture CI in release mode. It is not a second pin, tree, or observability gate. |

a-6 does not wait for a-5 to close ADR-RUN-0003. a-7 does not add a second
isolation or pin check.

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
`DaemonNotRunning`. That `ErrorCode`-only dependency on `sc-transport` and
`sc-command` is the a-1 recorded lead decision that amends ADR-RUN-0004 and
REQ-RUN-0309. The amendments cite
`docs/plans/phase-a/evidence/lead-ruling-sc-observability-types-transport.md`,
which a-1 writes. The edge is types-only: it is not an edge to `sc-command`
from `sc-transport`, and it is not a dependency on the logging runtime.
`sc-config` stays on `serde` and `serde_json` and forbids
`sc-observability-types`. `sc-runtime` forbids a direct dependency on
`sc-observability-types`. A transitive copy through `sc-command` is allowed
by REQ-RUN-0005 success criterion 5. `StoresFailed` uses the plain types in
the handoff.

Version policy: a-1 records the exact `sc-observability` and
`sc-observability-types` versions the spike resolves. The pin is the latest
crates.io release at spike time (1.5.0 as of this plan). a-1 amends
ADR-RUN-0004 and REQ-RUN-0309 so that recorded pair replaces the Active
1.2.x sentences before a-3, a-4, and a-6 are dispatched. a-4, a-6, and a-7
use that recorded pair.

| Sprint | Responsibility |
|---|---|
| a-1 | Depend on both crates in the spike, record the exact resolved versions in `docs/architecture.md`, and amend ADR-RUN-0004 and REQ-RUN-0309 to that pair. The same amendment is the recorded lead decision that allows `sc-observability-types` on `sc-command` and, for `ErrorCode` only, on `sc-transport`. The amendments cite `docs/plans/phase-a/evidence/lead-ruling-sc-observability-types-transport.md`. The `sc-runtime` forbidden edge for that crate is direct-only. |
| a-4 | `sc-command` depends on the recorded `sc-observability-types` version and not on `sc-observability`. |
| a-6 | Generated daemon depends on the recorded `sc-observability` version; init order is config, then observability, then `Daemon::builder`; no OTLP crate in the template. Fixture CI's `just test` runs the observability grep. |
| a-7 | The registry-backed fixture is a-6's fixture CI in release mode. That run is the release evidence. |

## Wave table

| Wave | Milestone | Sprint | Closure | Target boundary | Owned paths |
|---|---|---|---|---|---|
| 0 | spike | a-1 compatibility spike | contract (evidence record) | compatibility-evidence | `examples/spike/**`, `docs/architecture.md`, `docs/requirements.md`, `docs/plans/phase-a/evidence/lead-ruling-sc-observability-types-transport.md`, `tests/fixtures/app-name-vectors.json` |
| 0 | core | a-2 config | boundary | sc-config | `crates/sc-config/**`, `boundaries/sc-config/**`, sc-config requirement/ADR sources |
| 1 | core | a-3 transport | boundary | sc-transport | `crates/sc-transport/**`, `boundaries/sc-transport/**`, sc-transport requirement/ADR sources |
| 1 | core | a-4 command | boundary | sc-command | `crates/sc-command/**`, `boundaries/sc-command/**`, sc-command requirement/ADR sources |
| 2 | core | a-5 runtime core | integration | runtime-composition | root workspace registries, source-maintainer `sc-lint.toml` and `.sc-lint/bootstrap.py`, `crates/sc-runtime/**`, `boundaries/sc-runtime/**`, sc-runtime requirement/ADR sources, the edge-absent slices listed in the a-5 deliverables (architecture edge tables, REQ-RUN-0001/0004/0005 and REQ-RUN-0205 sections of `docs/requirements.md`), rewritten spike, core CI/just recipes |
| 3 | template | a-6 template from JSON | integration | template-from-json | `template/**` including the sc-lint 0.8.0 consumer install, `wizard/answers.schema.json`, `wizard/fixtures/**`, driver, root `Justfile` generation entry, the `sc-lint.toml` line-counts glob for `template/**/*.rs` only, REQ-RUN-0202, REQ-RUN-0503, and REQ-RUN-0701 sections of `docs/requirements.md`, ADR-RUN-0401 and ADR-RUN-0201 point 6 in `docs/architecture.md`, generation tests and template CI |
| 4 | release | a-7 core release | integration | release-distribution | authored `release/sc-publish-*`, `docs/development/sc-publish.md`, `docs/architecture.md` ADR-RUN-0002 publish-contract amendment, `install.py` outputs (`.github/workflows/release*.yml`, `scripts/release_*.py`), `.github/workflows/crates.yml` spike-reference cleanup, `.github/workflows/template.yml` release-mode selector only, publish-metadata fields on `crates/*/Cargo.toml`, `crates/*/README.md`, and per-crate `LICENSE` copies, root release metadata, spike removal |
| 4 | wizard | a-8 wizard | integration | wizard | `wizard/wizard.json`, `wizard/pages/**`, `scripts/run_wizard.py`, sequential `scripts/new_project.py` and `docs/generation.md` (entry-mode only), wizard tests |

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
release. a-5 also `must_follow` a-2, because it registers a-2's crate and
manifest. a-2 finishes in wave 0, before a-3 and a-4, so it is not the long
pole. `sc-runtime` still consumes no `sc-config` type. The wizard and release are
parallel-safe once the template, answers schema, and `--var-file` driver are
settled; this preserves the original ability for the wizard to slip without
blocking the core release.

## Dependency edges

| Child | Parent(s) | Relation and named artifact |
|---|---|---|
| a-2 | — | `must_follow` is empty. `parallel_safe` with a-1, a-3, and a-4. The sc-config facade is `serde` / `serde_json` plus `sc_config::AppName`. It consumes no spike signature. The `sc_config::AppName` constructor freeze waits on gate bead `triage:a-2-appname-freeze`, which `triage:follows` a-1 until ADR-RUN-0010 is Active. That bead is not a sprint `must_follow` edge. |
| a-3, a-4 | a-1 | `must_follow`: consume demonstrated reqwest/Axum/rmcp signatures and the recorded `sc-observability-types` version. |
| a-5 | a-2, a-3, a-4 | `must_follow`: registers the `sc-config` crate and `boundaries/sc-config/sc-config.toml` from a-2; consumes endpoint and listener types from `sc-transport`; consumes the `sc-command` server conversions. Not `parallel_safe` with a-2. `sc-runtime` does not depend on `sc-config` and consumes no type from it. Leaf sprints already run the single-crate `sc-lint lint sc-boundary` command a-1 recorded. Exit 0 is that gate. |
| a-6 | a-5 | `must_follow`: consumes the production runtime facade and rewritten-spike composition pattern. Sole owner of the NFR-RUN-0001 close in the feature-enforcement table. |
| a-7 | a-6 | `must_follow`: consumes the template tree, the answers schema, and the `--var-file` driver. Publishes only after that generated project is green against the release candidates. |
| a-8 | a-6 | `must_follow`: consumes the settled answers schema and `--var-file` pipeline. |
| a-1 ↔ a-2 | — | `parallel_safe`: spike evidence and the sc-config crate do not share owned paths. |
| a-7 ↔ a-8 | — | `parallel_safe`: disjoint release and wizard paths; wizard does not modify published crates or the template contract. a-8 does not wait for or diff against a-7. |

Parent development is merged into a child before each development/fix round;
parent PR merges before the child PR. Shared root files have one sequential
owner. **`docs/architecture.md` sequential owners:** a-1 (compatibility and
ADR amendments, including new ADR-RUN-0010, as pointers to crate ADRs, and
the NFR-RUN-0007 rmcp minor recorded in the evidence section), a-5
(edge-absent tables and the dual-bind OPEN paragraphs, copying a-3's
decision), a-6 (ADR-RUN-0401 points 2, 6, and 8, the generation-driver error
inventory and stderr contract, and ADR-RUN-0201 point 6
only), a-7 (ADR-RUN-0002 publish contract: sc-publish kit sole
workflow owner; GitHub Release includes
`sc-runtime-{version}-source.tar.gz`). a-7 does not add a release section to
ADR-RUN-0001. a-6 does not edit a-1, a-5, or a-7 slices of that file.
**`docs/requirements.md` sequential slices:** a-1 (its amendment
tasks, including the REQ-RUN-0005 `sc-observability-types` rows: allowed on
`sc-transport` and `sc-command` only, the `sc-runtime` edge direct-only with
criterion 5 allowing the transitive crate through `sc-command`, and the
NFR-RUN-0007 pin OPEN), a-5 (REQ-RUN-0001 spike membership,
the layout-table rename of `justfile` to `Justfile`,
REQ-RUN-0004 lint tools, REQ-RUN-0005 manifest filename and the
`sc-runtime` → `sc-config` row, REQ-RUN-0205 dual-bind), a-6 (REQ-RUN-0202,
REQ-RUN-0503, REQ-RUN-0701 only). a-2, a-3, and a-4 amend crate-local
ADRs and requirements. a-5 creates the core workspace/CI registries and verifies and
extends the existing root `Justfile` without contradicting its sc-lint init
header. a-6 owns the sequential root `Justfile` edit that adds `just new`
and the template consumer install. a-6's only `sc-lint.toml` edit is the
line-counts path glob that adds `template/**/*.rs` after a-5. **Release
slices after a-6:** a-7 may edit publish-metadata fields on
`crates/*/Cargo.toml`, `crates/*/README.md`, and per-crate `LICENSE` copies,
and may edit `.github/workflows/template.yml` only to select the driver's
release mode. Leaf behavior defects return to the leaf sprint. a-6 still
owns the generation sequence in that workflow. **`wizard/` and driver files:** a-6 owns
`wizard/answers.schema.json` and `wizard/fixtures/**`. a-8 owns
`wizard/wizard.json` and `wizard/pages/**`. `scripts/new_project.py` and
`docs/generation.md` pass from a-6 to a-8; a-8's diff is entry-mode flags
only, and the a-6 generation suite stays byte-identical. a-7 performs release
cleanup and kit install. A discovered leaf-crate defect returns to a-2, a-3, or a-4
rather than expanding a consumer sprint's ownership. a-6 must not replace
the maintainer `Justfile` with the generated consumer file.

## Dispatch ids

Sprint beads are `run-a-1` through `run-a-8` on the `sprint/a-*` branches
named in each sprint frontmatter. Parents are the dependency-edges table.

## QA consumption

Plan and sprint **QA-1** include `extensibility-qa` (adversarial multi-consumer
review against `docs/framework-charter.md`). It does **not** rerun on QA-2+
fix rounds. `arch-qa` must read the framework charter and
`docs/sc-runtime-design.md` for every phase plan review.

Boundary QA for a-2 through a-4 reviews one crate, its manifest, tests, README,
and boundary manifest. Each of those sprints runs the single-crate
`sc-lint lint sc-boundary` command a-1 recorded. Exit 0 is the gate. Analyzer
output is not pasted into the pull request. a-5 reviews composition and the source-maintainer install, not the
first analyze. a-6 reviews the minimal generated application and the
`--var-file` workflow. a-7 reviews packaging and the registry-backed run of
a-6's fixture CI. a-8 reviews wizard/schema equivalence. The a-8 diff to
`scripts/new_project.py` is entry-mode flags only.

QA must distinguish normative interoperability contracts from illustrative
template choices. An example layout is not a boundary rule for generated
applications.
