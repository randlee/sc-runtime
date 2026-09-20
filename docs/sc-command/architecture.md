# `sc-command` Architecture

**ID Range:** ADR-CMD-0001 through ADR-CMD-0003  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
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
| `Envelope<T>` | default | `{version, ok, data, error}` |
| `OpError`, `ErrorKind` | default | typed error with code and suggested action |
| `From<Result<T, OpError>> for Envelope<T>` | default | `.into()` in handlers |
| `impl IntoResponse for Envelope<T>` | `server` | REST edge |
| `IntoMcp::into_mcp` | `server` | MCP edge |

---

## ADR-CMD-0001: `OpError` is built on `sc-observability-types`

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19, sections "The crates" and "API surface"; applies [ADR-RUN-0004](../architecture.md)  

### Context

`sc-command` (directory `crates/sc-command`) defines the sc-ai-cli response
envelope `Envelope<T>`, a JSON object with keys `version`, `ok`, `data`,
`error`, and the typed error `OpError` with fields `kind`, `code`, `message`,
`details` and `suggested_action`.

The SC logging stack already publishes the types for a stable error code and
for remediation guidance in the crate `sc-observability-types` (1.2 series):
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

`OpError.code` is `sc_observability_types::ErrorCode`. `sc-command` defines
no error-code type and no remediation type of its own: if `OpError` carries,
or is built from, structured remediation, that structure is
`sc_observability_types::Remediation` or
`sc_observability_types::RecoverableSteps`.

`OpError.suggested_action` serialises as a plain JSON string, for example
`"run <app> daemon start"`, never as a JSON object. Its Rust type is not
decided (see the OPEN below) and is constrained to a type that serialises as
a string.

`sc-command` depends on `sc-observability-types` as an ordinary,
non-optional dependency. It does not depend on `sc-observability` or on
`sc-observability-otlp`.

This is the only observability dependency in any of the four library crates
of this repository (`sc-config`, `sc-transport`, `sc-command`, `sc-runtime`),
and it is a types-only crate (its 1.2.0 dependencies are `serde`,
`serde_json`, `thiserror` and `time`).

**OPEN:** The Rust type of `suggested_action` is not decided: a `String`, an
`Option<String>`, or a type of `sc-command` that holds a `Remediation` and
whose `Serialize` writes a string. `Remediation` alone cannot be the
serialised form, because it serialises as a JSON object tagged with `kind`.
Also undecided: the type of `details`; whether `details` and
`suggested_action` are present as `null` or omitted when empty; and whether
`OpError` converts to or from `sc_observability_types::Diagnostic`. All are
recorded on [REQ-CMD-0002](requirements.md), which owns them.
`sc-observability-types` 1.2.0 has no `ErrorKind`, `OpError` or
`suggested_action`; `ErrorKind` is defined in `sc-command`.

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

### Implementation

**Enforced by:** The boundary manifest under `boundaries/sc-command/`,
checked by `sc-lint-boundary` through `just lint`: its `forbidden_edges`
holds `"sc-command -> sc-observability"` and
`"sc-command -> sc-observability-otlp"`, and `sc-observability-types` is in
`allowed_dependencies` ([REQ-RUN-0005](../requirements.md) owns the
manifest). `cargo tree -p sc-command -e normal --prefix none`, with default
features and with `--features server`, prints no line beginning with
`sc-observability ` or `sc-observability-otlp ` (name then a space).
`crates/sc-command/Cargo.toml` lists `sc-observability-types` under
`[dependencies]` without `optional`. A test binds `OpError`'s `code` to a
value of type `sc_observability_types::ErrorCode`, and a test asserts that a
supplied `suggested_action` serialises as a JSON string
([REQ-CMD-0002](requirements.md)).

### Related Documents

- [REQ-CMD-0002](requirements.md)
- [NFR-CMD-0002](requirements.md)

---

## ADR-CMD-0002: REST and MCP edges are conversions, not wrappers

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
| extension trait `IntoMcp`, method `into_mcp()`, implemented for `Result<T, OpError>` where `T: Serialize` | `server` | `result.into_mcp()` as the last expression of an rmcp `#[tool]` function |

Handlers are ordinary axum handlers registered with `utoipa-axum`; tools are
ordinary rmcp `#[tool]` functions.

Because the conversions are implementations of, or feed, third-party
contracts, their signatures are dictated by axum and rmcp:
`IntoResponse::into_response` is infallible, and `IntoMcp::into_mcp` returns
`Result<CallToolResult, McpError>` with rmcp's `McpError`, which is what an
rmcp `#[tool]` function must return. These two signatures are the recorded
exception to the repository rule that a fallible public function returns a
typed error enum owned by its crate ([ADR-RUN-0006](../architecture.md),
[NFR-RUN-0009](../requirements.md)); that rule applies to errors a crate
itself originates.

**OPEN:** Whether `sc-command` needs a typed error enum of its own is not
decided; it is recorded on [NFR-CMD-0003](requirements.md).

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

With default features the crate's direct dependencies are `serde` and
`sc-observability-types`; `server` adds `axum` and `rmcp` and nothing else.

**OPEN:** Whether `serde_json` is also a direct default dependency is not
decided; it is recorded on [NFR-CMD-0001](requirements.md), which owns the
list of third-party crates `sc-command` may depend on.

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
