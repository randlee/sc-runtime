---
id: a-3
phase: a
status: planned
closure_type: boundary
target_boundary: sc-transport
track: core
wave: 1
branch: sprint/a-3-transport
worktree: sprint/a-3-transport
recommended_agent: arch-ctm
recommended_model: terra
must_follow: [a-1]
parallel_safe: [a-2, a-4]
requirements:
  - REQ-TRN-0001
  - REQ-TRN-0002
  - REQ-TRN-0003
  - REQ-TRN-0004
  - REQ-TRN-0005
  - REQ-TRN-0006
  - NFR-TRN-0001
  - NFR-TRN-0002
  - NFR-TRN-0003
  - NFR-TRN-0004
  - REQ-RUN-0002
  - REQ-RUN-0005
  - NFR-RUN-0003
  - NFR-RUN-0004
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0004
  - ADR-RUN-0007
  - ADR-TRN-0001
  - ADR-TRN-0002
  - ADR-TRN-0003
  - ADR-TRN-0004
  - ADR-TRN-0005
  - ADR-TRN-0006
  - ADR-RUN-0003
  - ADR-RUN-0006
  - ADR-RUN-0010
owned_paths:
  - "crates/sc-transport/**"
  - "boundaries/sc-transport/**"
  - "docs/sc-transport/requirements.md"
  - "docs/sc-transport/architecture.md"
---

# Sprint a-3 — sc-transport

## Outcome and closure

Create one independently usable transport crate for endpoint resolution,
ordinary HTTP requests over UDS or TCP, and optional server listener binding.
It discovers and reports; it does not own application launch policy.

## Decision prerequisites

Resolve endpoint syntax and precedence, OS defaults, directory ownership,
listener representation, UDS permissions/stale-file behavior, timeout/status
classification, and the public error surface. Before `TransportConfig`
freezes, close the dual-bind OPEN in `docs/sc-transport/architecture.md` and
`docs/sc-transport/requirements.md`: v0.1 binds exactly one listener, and
`TransportConfig` reserves additive `additional_endpoints` (default empty,
reserved and ignored in v0.1) so a later dual-bind does not fork the facade.
The ADR amendment states that the field is ignored in v0.1. a-5 embeds
that type and does not decide a second shape. Publish an **error inventory**
(stable codes and recovery for every `TransportError` variant, including
non-`DaemonNotRunning` failures the CLI must report). `DaemonNotRunning`
carries `sc_observability_types::ErrorCode` (`DAEMON.NOT_RUNNING`) on the code
field and a plain `String` suggested action (`run <app> daemon start` with the
`AppName` passed to `Client::new`). No `Remediation` on transport (RBP-F002,
RBP-F005). The `ErrorCode`-only `sc-observability-types` dependency is the
a-1 recorded lead decision that amends ADR-RUN-0004 and REQ-RUN-0309. This
sprint consumes that amendment. It does not reopen it and it does not edit
those root documents. The string names the `daemon start` subcommand a-6
ships (exec the daemon binary). It is not a caller-supplied launch hint and
it is not auto-start. Record that in the crate ADR before the facade freezes.
Do not edit `docs/architecture.md`. Do not add auto-start APIs.
`sc_transport::AppName` is the client wire identity. It uses the predicate
a-1 records in ADR-RUN-0010 and the handoff vector list. This sprint does
not copy that list, does not invent a second predicate, and does not write
that ADR. It is not `sc_runtime::AppName`.

## Public contract produced

This sprint owns the sc-transport types and methods in the canonical
[minimal public contract handoff](phase-a-plan.md#minimal-public-contract-handoff):
`sc_transport::AppName`, `Endpoint`, `TransportConfig`, `instance_root`,
`resolve_endpoint`, `Client::new`, `get`, `post`, `bind`, `Listener`, and
`TransportError`.
`Client::get` and `Client::post` are the handoff signatures, and both take
that crate's `AppName` only through `Client::new`. The crate-local
requirement and ADR sources are the decision record. This sprint amends
`docs/sc-transport/architecture.md` and `docs/sc-transport/requirements.md`
only, including `&str` application-name parameters those sources still show.
It does not edit `docs/architecture.md`.

## Dependencies and parallel safety

- `must_follow a-1`: consumes the demonstrated reqwest/Axum/UDS signatures.
- `parallel_safe a-2` and `a-4`: owned paths and boundaries are disjoint.

## Deliverables

1. **REQ-TRN-0001 / REQ-TRN-0002** — Deterministic endpoint and instance-root
   resolution from explicit inputs, with lossless endpoint text.
2. **REQ-TRN-0003 / REQ-TRN-0004** — Feature-gated UDS/TCP listeners accepted
   by Axum, with safe UDS placement, permissions, and stale-socket handling.
3. **REQ-TRN-0005 / REQ-TRN-0006** — Generic asynchronous GET/POST using
   reqwest, preserving response bodies and typed connection failures without
   spawning, retrying for readiness, or knowing application envelopes.
4. **NFR-TRN-0001 / NFR-TRN-0002** — Default/server builds use standard
   Tokio, reqwest, and Axum facilities with no custom connector or raw HTTP.
5. **REQ-RUN-0005 / NFR-TRN-0001** — Boundary manifest at
   `boundaries/sc-transport/sc-transport.toml`. Amend NFR-TRN-0001 so the
   default direct dependencies are `tokio`, `reqwest`, `serde`, and
   `sc-observability-types` at the a-1 pin. `server` still adds only `axum`.
   `allowed_dependencies` includes `sc-observability-types`.
   `forbidden_edges` keeps `sc-observability` and does not list
   `sc-observability-types`. Once `crates/sc-transport/Cargo.toml` exists,
   run `just lint gates` or `sc-lint lint sc-boundary --root . --config sc-lint.toml`
   on this manifest. Do not wait for a-5. Do not own the permanent root
   workspace `Cargo.toml`.
6. **REQ-TRN-0006 / NFR-TRN-0004** — Before the facade freezes, the crate ADR
   contains an error-inventory table for every `TransportError` variant,
   including failures other than `DaemonNotRunning`: failure mode, stable
   code, cause payload, and recovery steps (RBP-F002). `DaemonNotRunning`
   carries `sc_observability_types::ErrorCode` from a direct
   `sc-observability-types` dependency at the version a-1 recorded. That
   dependency is types-only: the crate does not depend on `sc-command` or on
   `sc-observability` (RBP-F005). That dependency is the a-1 recorded lead
   decision amending ADR-RUN-0004 and REQ-RUN-0309. `sc_transport::AppName`
   uses the handoff predicate and the handoff vector list. This sprint does
   not copy that list. It is not `sc_config::AppName` and it is not
   `sc_runtime::AppName`. Invalid names fail
   at construction with `APP.INVALID_NAME`. Display is `as_str` or
   `AsRef<str>`, and the type does not implement `Deref`.

## Acceptance criteria

1. `boundary:sc-transport` — default and server feature tests pass, including
   real loopback UDS/TCP behavior where supported and pure OS-path tests.
   Two application names using the same unreachable endpoint produce the same
   `DAEMON.NOT_RUNNING` code and their respective suggested actions.
   UDS bind tests also prove endpoint-scoped ownership prevents a second
   owner from unlinking a live listener.
2. `boundary:sc-transport` — the default dependency graph contains no Axum,
   rmcp, sqlx, runtime crate, process launcher, or application command type.
3. `ADR-TRN-0005` — client failures remain transport facts; generated CLI
   policy is neither implemented nor implied.
4. `ADR-TRN-0001` — the crate ADR error-inventory table
   lists every `TransportError` variant, including non-`DaemonNotRunning`
   failures, with a stable code, cause payload, and recovery steps
   (RBP-F002).
5. `req:REQ-TRN-0006` — amend point 2, success criteria 4 and 5, and the
   rationale that both fields are plain strings so they match the handoff:
   code is `sc_observability_types::ErrorCode` (`DAEMON.NOT_RUNNING`);
   suggested_action is `String` (`run <app> daemon start` with the `AppName`
   passed to `Client::new`). No `Remediation` on transport.
6. `boundary:sc-transport` — `DaemonNotRunning` exposes
   `sc_observability_types::ErrorCode`. `cargo tree -p sc-transport -e normal --prefix none`
   has a `sc-observability-types ` line at the a-1 recorded version and has no
   `sc-command ` line and no `sc-observability ` line (crate name, then a
   space, so the types crate does not match the logging crate).
   `sc_transport::AppName` accepts the handoff accept vectors and rejects the
   handoff reject vectors at construction. The vectors are the handoff list,
   not a copy in this sprint. Rejection uses stable code
   `APP.INVALID_NAME`. Public constructors take `sc_transport::AppName`. The
   type exposes `as_str` or `AsRef<str>` and does not implement `Deref`
   (RBP-F005).
7. `req:NFR-TRN-0001` — the amended dependency list names
   `sc-observability-types` on the default build. Analyze accepts that
   dependency while `forbidden_edges` still contains `sc-observability`.
   `just lint gates` or `sc-lint lint sc-boundary` exits 0 on
   `boundaries/sc-transport/sc-transport.toml`.
8. `ADR-TRN-0002` — the crate ADR amendment states that v0.1 binds exactly
   one listener and that `additional_endpoints` is reserved and ignored in
   v0.1. The field is present, defaults empty, and the implementation does
   not read it. The ADR says so, so the ignore is not an undocumented
   failure. A later release may use the field without forking the facade.
   The transport dual-bind OPEN is deleted. The suggested action is the a-6
   `daemon start` subcommand, not a caller launch hint.

## This sprint does not close

Daemon lifecycle, CLI auto-start, application retries, service-manager
integration, and generated CLI tests are outside this boundary. This sprint
runs the boundary analyzer on its own manifest. Root `just lint` across all
four manifests stays with a-5. This sprint does not edit
`docs/architecture.md` or `docs/requirements.md`. a-5 copies the dual-bind
decision into those root OPENs.

## Paths to delete

None.

## Required validation

Run default and server crate tests/doctests, feature-specific dependency-tree
checks, and workspace build once registered. Review the manifest against
REQ-RUN-0005. Confirm the crate ADR error-inventory table, the
`DaemonNotRunning` code type, and the `AppName` construction tests.
Run `just lint gates` or `sc-lint lint sc-boundary` on this crate's manifest.
