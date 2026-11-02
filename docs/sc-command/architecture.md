# `sc-command` Architecture

**ID Range:** ADR-CMD-0001 through ADR-CMD-0003  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-23  
**Owner:** sc-runtime maintainers  

---

Crate-level architecture and ADRs for `sc-command`. Repo-level architecture is in
[`../architecture.md`](../architecture.md); this crate's requirements are in
[`requirements.md`](requirements.md); its boundary manifest is
`boundaries/sc-command/`. ADRs follow
the shared SC requirement and ADR templates. Every ADR here is Active
and binding; an ADR changes only through a later ADR that names it.

## Role

A leaf crate of plain data types plus two feature-gated conversions. It owns
the wire shape of every response. It owns no routing, registry, transport or
handler.

## Public surface

| Item | Feature | Purpose |
|---|---|---|
| `Envelope<T>` | default | `{version, ok, data, error}`; version string `"1"` on constructors |
| `OpError`, `ErrorKind` | default | typed error; `Display` and `std::error::Error` on `OpError`; wire `suggested_action` is a string or `null` |
| `OpError::from_transport_strings` | default | plain strings plus the caller's suggestion; no `sc-transport` dependency |
| `Envelope::into_result` | default | `Result<T, OpError>` from an envelope |
| `From<Result<T, OpError>> for Envelope<T>` | default | `.into()` in handlers |
| `impl IntoResponse for Envelope<T>` | `server` | REST edge |
| `IntoMcp::into_mcp` | `server` | MCP edge; sealed; static dispatch only |

---

## ADR-CMD-0001: `OpError` is built on `sc-observability-types`

**2026-10-31 amendment:** [ADR-RUN-0017](../architecture.md#adr-run-0017-initial-release-observability-shim)
adds only the command-owned canonical port and foreign EventError signature.
It explicitly supersedes the exact-pin rejection below for the temporary
registry types `=1.5.0` bridge; initial locks prove 1.5.0, while 1.5.1 is
capability-gated future cleanup. Event/error value identity remains upstream.

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19, sections "The crates" and "API surface"; applies [ADR-RUN-0004](../architecture.md)  

### Context

`sc-command` (directory `crates/sc-command`) defines the sc-ai-cli response
envelope `Envelope<T>`, a JSON object with keys `version`, `ok`, `data`,
`error`, and the typed error `OpError` with fields `kind`, `code`, `message`,
`details` and `suggested_action`.

The SC logging stack already publishes the types for a stable error code and
for remediation guidance in the crate `sc-observability-types` 1.5.0:
`ErrorCode`, a string newtype, and `Remediation` with `RecoverableSteps`.
Logs and diagnostics use them. If `sc-command` defined its own versions, the
same failure could carry one code in an API response and another in the log,
and there would be two definitions to keep in step.

Pulling against that: the library crates in this repository do not depend on
the logging runtime `sc-observability`, so that each stays usable in a
program with no SC logging
([ADR-RUN-0004](../architecture.md) makes the project's `main.rs` initialise
observability directly).

### Decision

`OpError.code` is `sc_observability_types::ErrorCode` from
`sc-observability-types` under temporary exact registry
requirement `=1.5.0` (ADR-RUN-0017).
`sc-command` defines no error-code type and no remediation type of its own.
`OpError.remediation` is `Option<sc_observability_types::Remediation>`.
`None` is the empty variant. A custom serializer writes the wire key
`suggested_action` as a JSON string or as JSON `null`. The tagged
`Remediation` object is never the wire form. `details` is
`Option<serde_json::Value>` and is JSON `null` when absent. Both keys are
always present. `sc-command` does not convert `OpError` to or from
`sc_observability_types::Diagnostic`.

The string is the first recovery step of `Remediation::Recoverable` when
that step is non-empty. Later steps stay off the wire. An empty step list,
an empty first step, and `Remediation::NotRecoverable` write JSON `null`,
never `""`. A not-recoverable justification is copied into wire `details`
under `justification`: the serializer inserts that key into a JSON object,
and wraps any other JSON value as
`{"value": <details>, "justification": <text>}`. Deserializing a non-empty
string builds a one-step `Recoverable`. Deserializing `null` or `""` builds
`None`. A single-step recoverable value with a non-empty step round-trips.
A multi-step value and a `NotRecoverable` value do not round-trip as the
same Rust variant. The JSON produced by the first serialization does
round-trip. REQ-CMD-0002 keeps `suggested_action` a string or `null` and
keeps the error object to those five keys, so this ADR does not add a
`recoverable` flag or a `remediation_steps` list. `NotRecoverable` is null
on `suggested_action` so an agent does not execute the justification.
This closes EXT-005.

`ErrorKind` is `#[non_exhaustive]`. The wire spellings are `invalid_input`,
`not_found`, `conflict`, `unavailable`, and `internal`. An unrecognised
spelling fails deserialization with `ENVELOPE.DISCRIMINANT`. HTTP status is
400, 404, 409, 503, and 500 in that order. That table lives only in
`ErrorKind::http_status_code`. The `server` conversion passes the `u16` to
`StatusCode::from_u16` and uses 500 when the number is not a status code.
The first three kinds are caller errors. The last two are internal errors.
`ErrorKind::all` returns `&'static [ErrorKind]`. The `ENVELOPE.KIND.*`
codes in the inventory below name the mapping in this ADR. They are
documentation of the mapping. They are not a public method and they are not
emitted. The service chooses `OpError.code`. A project that needs another
HTTP status builds the response itself. A later variant stays
source-compatible for downstream `match` because the enum is
`non_exhaustive`, and older decoders still reject the new spelling.
This closes EXT-002, EXT-007, and EXT-008.

`OpError` implements `Display` as `{code}: {message}` and
`std::error::Error`. `ErrorKind` implements `Display` as its wire spelling.
This closes EXT-006.

`Envelope<T>` is the enum `Success { version, data }` or
`Failure { version, error }` with a custom serde map of exactly `version`,
`ok`, `data`, and `error`. `OpError` is a custom serde map of exactly
`kind`, `code`, `message`, `details`, and `suggested_action`. Constructors
write version `"1"`. Deserialization keeps the version it read. That string
does not select a different key set. A contradictory or missing
discriminant, an unknown envelope or error key, or a value of the wrong
JSON type is a serde error whose message includes `ENVELOPE.DISCRIMINANT`.
Unknown fields inside `data` and inside `details` stay lossless
([REQ-CMD-0006](requirements.md)). Unknown keys of the envelope object and
of the error object stay rejected ([REQ-CMD-0001](requirements.md)). The
charter's additive-field upgrade path conflicts with that requirement.
This ADR keeps the requirement. A new envelope or error key is a new
contract, and every consumer upgrades together. `Envelope::into_result`
consumes the value and returns `Result<T, OpError>`, dropping `version`.
This closes EXT-001 and EXT-010.

`IntoMcp` is sealed by a private supertrait and is implemented only for
`Result<T, OpError>`. `into_mcp` takes `self` by value. The trait also
requires `Sized`. Dispatch is static. `dyn IntoMcp` is not a supported API.
Serialization failure does not panic: the HTTP conversion returns status 500
with the body of `Envelope::failure` for an `OpError` whose code is
`ENVELOPE.SERIALIZATION`. A static JSON body with the same four keys is
used only when that envelope itself fails to serialize. `into_mcp` returns
`Err` of rmcp's `ErrorData` (imported by tools as `McpError`). This closes
RBQA-F002. The single status table closes RBQA-F001. The shared
`require_exact_keys` helper closes RBQA-F003.

`OpError::from_transport_strings(code, endpoint, cause, suggested_action)`
builds an `OpError` from those plain strings. It does not depend on
`sc-transport`. The code `DAEMON.NOT_RUNNING` uses
`ErrorKind::Unavailable` and the message `the daemon is not running`.
Any other code uses `ErrorKind::Internal`, keeps that code as `ErrorCode`,
and sets `message` to `transport failure {code} at {endpoint}: {cause}`.
`endpoint` and `cause` are stored in `details` for every code.
`suggested_action` is the caller's already-rendered recovery text. `None`,
or text that is empty after trimming, leaves remediation unset. The crate
does not write a placeholder application name. The application name and the
daemon verb are caller policy. This closes EXT-003.

`sc-command` depends on `sc-observability-types` as an ordinary,
non-optional dependency, temporarily exact `=1.5.0` under ADR-RUN-0017.
This inspected validator/types boundary intentionally rejects consumers that
require newer types until coordinated cleanup; it is a conscious exception
to the earlier caret-only decision. It does not depend on
`sc-observability` or on `sc-observability-otlp`. `sc-command` remains the
only library using its error/remediation surface. `sc-runtime` and `sc-cron`
also depend on the types crate, but only for injected canonical logging;
`sc-config` and `sc-transport` do not. This closes EXT-004.
The logging runtime stays forbidden on all library crates.

#### Library wire-failure inventory

These rows are `sc-command`'s own wire failures. Widget create and get, and
every other application operation code, are not rows.

| Failure | Stable code | Cause | Recovery | Docs |
|---|---|---|---|---|
| Invalid or contradictory envelope discriminant | `ENVELOPE.DISCRIMINANT` | `ok` disagrees with `data` or `error`, a required key is missing, an unknown envelope key is present, or a key has the wrong JSON type | Send `version`, `ok`, `data`, and `error`. Success sets `error` to JSON `null`. Failure sets `data` to JSON `null`. | [REQ-CMD-0001](requirements.md) |
| Serialization failure | `ENVELOPE.SERIALIZATION` | `T`'s `Serialize` implementation returned an error | Return a payload that serializes to JSON. The HTTP edge responds 500 with this code and does not panic. The MCP edge returns rmcp `ErrorData` internal error carrying this code and does not panic. | [REQ-CMD-0004](requirements.md), [REQ-CMD-0005](requirements.md) |
| `ErrorKind::InvalidInput` | `ENVELOPE.KIND.INVALID_INPUT` | Caller error. The service still chooses `OpError.code`. | HTTP 400. Fix the request. | [REQ-CMD-0004](requirements.md) |
| `ErrorKind::NotFound` | `ENVELOPE.KIND.NOT_FOUND` | Caller error. The service still chooses `OpError.code`. | HTTP 404. Use an id that exists. | [REQ-CMD-0004](requirements.md) |
| `ErrorKind::Conflict` | `ENVELOPE.KIND.CONFLICT` | Caller error. The service still chooses `OpError.code`. | HTTP 409. Reload and retry the write. | [REQ-CMD-0004](requirements.md) |
| `ErrorKind::Unavailable` | `ENVELOPE.KIND.UNAVAILABLE` | Internal error. Includes `DAEMON.NOT_RUNNING` from `from_transport_strings`. | HTTP 503. `suggested_action` is the caller-supplied string when one was passed. | [REQ-CMD-0004](requirements.md) |
| `ErrorKind::Internal` | `ENVELOPE.KIND.INTERNAL` | Internal error. The service still chooses `OpError.code`. | HTTP 500. Read `message` and `details`. | [REQ-CMD-0004](requirements.md) |

v0.1 does not define a crate error enum. The only fallible public function
is `IntoMcp::into_mcp`, whose error type is rmcp's, the recorded exception.

### Consequences

An error code is the same value, of the same type, in an envelope and in a
log record. `sc-command` must follow `sc-observability-types` releases: a
breaking release there is a breaking release of `sc-command`, because
`ErrorCode` appears in its public API. A program that uses `sc-command` but
no SC logging still compiles `sc-observability-types`, which is small.

### Alternatives Considered

- Local copies of the code and remediation types inside `sc-command`.
  Rejected: two definitions of the same concepts would drift, and projects
  would write conversions between them at every logging call.
- Depending on the full `sc-observability` crate. Rejected: it would pull
  the logging runtime into every CLI that only parses envelopes, and it
  would break the rule that these crates do not wrap or depend on
  observability.
- Tolerating unknown envelope or error keys, or parking them in an `extra`
  map. Rejected: [REQ-CMD-0001](requirements.md) rejects an unknown key so a
  CLI reprint cannot drop a field the agent did not see. Payload fields
  inside `data` and `details` stay lossless under
  [REQ-CMD-0006](requirements.md).
- Writing a not-recoverable justification into `suggested_action`, or adding
  a remediation-steps key. Rejected: the justification is not an instruction,
  and [REQ-CMD-0002](requirements.md) keeps the wire value a string or
  `null` on the existing five keys.
- An exact `=` pin of `sc-observability-types`. Originally rejected because
  it narrows consumer resolution. ADR-RUN-0017 explicitly accepts that cost
  temporarily for inspected 1.5.0 validator/identity proof; cleanup removes it.
- A public `ErrorKind::mapping_code`. Rejected: the inventory codes above
  are documentation and are not on the wire.

### Implementation

**Enforced by:** The boundary manifest `boundaries/sc-command/sc-command.toml`,
checked by `sc-lint lint sc-boundary --root .` on the workspace.
`allowed_dependents` temporarily includes runtime+cron under ADR-RUN-0017. `allowed_dependencies` includes
`sc-observability-types` and does not treat it as a forbidden edge.
`forbidden_edges` does not name `sc-observability` or `sc-observability-otlp`,
because those crates are not workspace members, and it does not contain
`sc-observability-types`. sc-lint 0.8.0 rejects a forbidden edge whose target
is absent, so `cargo tree` enforces the logging crates. Sprint a-5 deleted
`crates/sc-command/lint-edge/` and added `crates/sc-runtime`, which depends on
`sc-command`. `boundaries/planning.toml` exists because `sc-lint` requires
planning metadata next to the workspace.
`cargo tree -p sc-command -e normal --prefix none`, with default features and
with `--features server`, may print a `sc-observability-types ` line at the
workspace-lock resolution `1.5.0` under ADR-RUN-0017
and no line beginning with `sc-observability ` or `sc-observability-otlp `
(name then a space). `crates/sc-command/Cargo.toml` lists
`sc-observability-types = "=1.5.0"` under `[dependencies]` for the bridge
without `optional`.
`axum` is optional at 0.8.9 with `default-features = false`.
A test binds `OpError`'s `code` to `sc_observability_types::ErrorCode`, and
a test asserts that a supplied suggestion serialises as a JSON string
([REQ-CMD-0002](requirements.md)).

### Related Documents

- [REQ-CMD-0002](requirements.md)
- [NFR-CMD-0002](requirements.md)

---

## ADR-CMD-0002: REST and MCP edges are conversions, not wrappers

**2026-10-31 bounded exception:** ADR-RUN-0017 allows only the neutral
CanonicalLog event-receiver trait in `observability`. It adds no REST/MCP
handler, extractor, middleware, router or framework dialect. All conversion
and handler prohibitions below continue to apply.

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19, section "API surface"; applies [ADR-RUN-0001](../architecture.md)  

### Context

Every REST handler and every MCP tool in a generated daemon must answer with
the response envelope `Envelope<T>` (JSON keys `version`, `ok`, `data`,
`error`). Service functions return `Result<T, OpError>`. Something has to
turn that result into an axum response and into an rmcp tool result.

A framework could do this by supplying its own handler type, extractor,
middleware layer or attribute macro that produces envelopes. Any of those
would sit between the project and axum or rmcp: the project would write
handlers in the framework's dialect, and axum and rmcp documentation would
no longer describe the project's code. The governing principle of this
repository is that the framework builds parts and the project wires them in
ordinary Rust ([ADR-RUN-0001](../architecture.md)), and the repository
admits no macro system and no wrapper around an axum or rmcp type
([NFR-RUN-0003](../requirements.md)).

### Decision

`sc-command` reaches both edges through three trait implementations on its
own and standard types, and nothing else:

| Conversion | Feature | Used as |
|---|---|---|
| `impl<T> From<Result<T, OpError>> for Envelope<T>` | default | `result.into()` as the last expression of an axum handler that returns `Envelope<T>` |
| `impl<T: Serialize> axum::response::IntoResponse for Envelope<T>` | `server` | applied by axum to the handler's return value |
| extension trait `IntoMcp`, method `into_mcp()`, implemented only for `Result<T, OpError>` where `T: Serialize`, sealed by a private supertrait, taking `self` by value. Dispatch is static. `dyn IntoMcp` is not a supported API. | `server` | `result.into_mcp()` as the last expression of an rmcp `#[tool]` function |

Handlers are ordinary axum handlers registered with `utoipa-axum`; tools are
ordinary rmcp `#[tool]` functions.

Because the conversions are implementations of, or feed, third-party
contracts, their signatures are dictated by axum and rmcp:
`IntoResponse::into_response` is infallible, and `IntoMcp::into_mcp` returns
`Result<CallToolResult, McpError>` with rmcp 3.7's `rmcp::ErrorData` (the
name tools import as `McpError`). These two signatures are the recorded
exception to the repository rule that a fallible public function returns a
typed error enum owned by its crate ([ADR-RUN-0006](../architecture.md),
[NFR-RUN-0009](../requirements.md)); that rule applies to errors a crate
itself originates. v0.1 does not add an `sc-command` error enum. Serialization
failure uses those two recorded signatures.

`sc-command` MUST NOT define: a handler type or trait; an axum extractor
(`FromRequest` or `FromRequestParts` implementation); a tower `Layer` or
`Service`; a procedural or declarative macro; a newtype or struct that wraps
an axum or rmcp type; a router, registry or list of operations.

### Consequences

A generated handler body is one line,
`service::create_widget(&s, input).await.into()`, and a tool body is
`service::create_widget(&self.stores, input).await.into_mcp()`. A project
that wants a different response shape on one route does not call the
conversion and returns any axum response it likes; nothing in `sc-command`
or `sc-runtime` notices. Because nothing is applied automatically, a handler
that forgets `.into()` and returns a bare type is not caught by this crate;
keeping the three surfaces in step is left to lint tooling.

### Alternatives Considered

- An attribute macro that generates the handler and the tool from a service
  function. Rejected: it is a macro system to maintain and to learn, and it
  hides the axum and rmcp code the project is meant to own.
- An envelope extractor or a middleware layer that wraps every response.
  Rejected: it wraps axum's types, applies to routes that may not want the
  envelope, and has no equivalent on the rmcp side, so the two edges would
  work differently.

### Implementation

**Enforced by:** `arch-qa` architecture review of `crates/sc-command/src`
against the MUST NOT list in the Decision: no `proc-macro = true` in
`crates/sc-command/Cargo.toml`; no `macro_rules!`; no `impl` of
`FromRequest`, `FromRequestParts`, `Layer`, `Service` or `Handler`; no
struct with a field of an axum or rmcp type. The same review applies the
repository rule against wrappers and macro systems
([NFR-RUN-0003](../requirements.md)). The template's example handler in
`daemon/src/routes.rs` and tool in `daemon/src/mcp.rs` end in `.into()` and
`.into_mcp()` respectively.

### Related Documents

- [REQ-CMD-0003](requirements.md)
- [REQ-CMD-0004](requirements.md)
- [REQ-CMD-0005](requirements.md)
- [NFR-CMD-0003](requirements.md)
- [NFR-RUN-0003](../requirements.md)
- [NFR-RUN-0009](../requirements.md)

---

## ADR-CMD-0003: The `server` feature gates axum and rmcp

**2026-10-31 amendment:** ADR-RUN-0017 adds `test-util = []`, off by default
and independent of `server`; it exposes std-only recorder/rejector doubles.
The public canonical port/Noop is unconditional. Exact bridge types pin is
`=1.5.0`; verify isolated default/test-util/cron consumers do not activate server.

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19, section "Repository layout"; applies [ADR-RUN-0003](../architecture.md)  

### Context

Two programs use `sc-command`. The daemon needs the envelope types plus the
conversions into an axum response and an rmcp tool result, which require
axum and rmcp. The CLI needs only `Envelope<T>` and `OpError`, to parse the
daemon's answer and print it for `--json`; a CLI binary must link none of
axum, rmcp or sqlx, because that would multiply its build time and binary
size. One published crate has to serve both. The repository-level decision
is that server-side code in `sc-transport` and `sc-command` sits behind a
cargo feature named `server`, off by default
([ADR-RUN-0003](../architecture.md)).

### Decision

In `crates/sc-command/Cargo.toml`, `axum` and `rmcp` are `optional = true`
dependencies enabled only by the feature `server`. `server` is not a default
feature.

`impl IntoResponse for Envelope<T>` and the `IntoMcp` trait with its
implementation are compiled only under `#[cfg(feature = "server")]`.
Everything else (`Envelope<T>`, `OpError`, `ErrorKind`,
`From<Result<T, OpError>>`) compiles with default features.

With default features the crate's direct dependencies are `serde`,
`serde_json`, and `sc-observability-types` at temporary exact requirement
`=1.5.0` under ADR-RUN-0017. `server`
adds `axum` 0.8.9 and `rmcp` `~3.7` and nothing else.
`axum` is declared with `default-features = false`. The conversion uses
`IntoResponse`, `Body`, and `http`. Axum's default features stay off so a
daemon does not enable `form`, `query`, or `tower-log` through feature
unification with this crate. `rmcp` is already `default-features = false`.
`serde_json` is a direct dependency because `details` holds arbitrary JSON.
This closes EXT-009.

Within this repository `sc-runtime` depends on `sc-command` with `server`
enabled. In a generated project the `cli` crate and the `service` crate use
`sc-command` with default features (`service` returns `OpError`; `cli`
parses `Envelope<T>`), and the `daemon` crate needs the `server` conversions
(its handlers return `Envelope<T>` and its tools call `into_mcp()`). The
generated workspace's crate graph is owned by
[REQ-RUN-0301](../requirements.md) and
[ADR-RUN-0303](../architecture.md); this ADR does not decide it.

The generated `service` crate takes `sc-command` (default features) as a
direct dependency, because it may not depend on `sc-runtime`.

**OPEN:** Whether the generated `daemon` crate takes `sc-command` as a direct
dependency with `features = ["server"]`, or reaches it through a re-export
from `sc-runtime`, is not decided. A re-export would have to be added to the
public surface of `sc-runtime`. Once decided in
favour of direct dependencies: the generated `daemon` is the only generated
crate that enables `server`.

### Consequences

The crate is two build configurations, and CI builds and tests both:
`cargo test -p sc-command` and `cargo test -p sc-command --features server`.
A mistake in gating shows up as axum or rmcp in the CLI's `cargo tree`.
Cargo unifies features across a workspace build, so the property "the CLI
links no axum or rmcp" is defined for the CLI built as its own selection
(`cargo build -p cli`), and evidence must come from `cargo tree -p` on the
CLI crate or on `sc-command` alone, not from a whole-workspace build.

**OPEN:** Under `cargo build --workspace` cargo unifies features, so the CLI
is built against a `server`-enabled `sc-command`. Whether workspace-wide
builds are exempt from the property, or the crates must instead be split
into client and server crates, is not decided; it is recorded on
[NFR-RUN-0001](../requirements.md) and
[ADR-RUN-0003](../architecture.md) and is to be measured by the spike.

### Alternatives Considered

- A separate `sc-command-server` crate holding the two conversions.
  Rejected: Rust's orphan rule forbids implementing axum's `IntoResponse`
  for `Envelope<T>` outside the crate that defines `Envelope<T>` without a
  wrapper type, and it would double the crates to publish and version.
  (The feature-unification OPEN above may reopen a client/server split; the
  orphan-rule obstacle would then have to be solved first.)
- No gating, with axum and rmcp as ordinary dependencies. Rejected: every
  CLI would link the server stack.

### Implementation

**Enforced by:** `cargo tree -p sc-command -e normal --prefix none` with
default features printing no line beginning with `axum ` or `rmcp ` (name
then a space), and with `--features server` printing a line beginning with
each; `just test`, which runs `cargo test -p sc-command` and
`cargo test -p sc-command --features server`
([REQ-RUN-0004](../requirements.md)). `sc-lint-boundary` is not part of this
enforcement: its manifest format has no cargo-feature key, so it checks
inter-crate edges and the public facade list only and cannot express "axum
only with `server`".

### Related Documents

- [NFR-CMD-0001](requirements.md)
- [NFR-RUN-0001](../requirements.md)
- [REQ-RUN-0301](../requirements.md)
- [ADR-RUN-0303](../architecture.md)
