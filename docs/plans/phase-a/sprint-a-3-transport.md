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
  - ADR-TRN-0001
  - ADR-TRN-0002
  - ADR-TRN-0003
  - ADR-TRN-0004
  - ADR-TRN-0005
  - ADR-TRN-0006
  - ADR-RUN-0003
  - ADR-RUN-0006
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
classification, and the public error surface. Publish an **error inventory**
(stable codes and recovery for every `TransportError` variant, including
non-`DaemonNotRunning` failures the CLI must report). `DaemonNotRunning`
carries `sc_observability_types::ErrorCode`, not plain strings (RBP-F002,
RBP-F005). Record choices in the owned ADR before the facade freezes. Do not
add auto-start APIs.

## Public contract produced

This sprint owns the sc-transport types and methods in the canonical
[minimal public contract handoff](phase-a-plan.md#minimal-public-contract-handoff):
`AppName`, `Endpoint`, `TransportConfig`, `resolve_endpoint`, `Client::new`,
`get`, `post`, `bind`, `Listener`, and `TransportError`. The crate-local
requirement and ADR sources are the decision record.

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
5. **REQ-RUN-0005** — Boundary manifest at `boundaries/sc-transport/*.toml`
   for the actual facade and edges. The manifest must be valid for
   `sc-lint-boundary analyze`. That command runs once a-5's root `just lint`
   exists; this sprint does not install sc-lint or own the root `justfile`.
6. **REQ-TRN-0006 / NFR-TRN-0004** — Before the facade freezes, the crate ADR
   contains an error-inventory table for every `TransportError` variant,
   including failures other than `DaemonNotRunning`: failure mode, stable
   code, cause payload, and recovery steps (RBP-F002). `DaemonNotRunning`
   carries `sc_observability_types::ErrorCode` from a direct
   `sc-observability-types` dependency at the version a-1 recorded. That
   dependency is types-only: the crate does not depend on `sc-command` or on
   `sc-observability` (RBP-F005). `AppName` follows the a-2 newtype rule:
   invalid names fail at construction, display is `as_str` or `AsRef<str>`,
   and the type does not implement `Deref`.

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
4. `docs/sc-transport/architecture.md` — the crate ADR error-inventory table
   lists every `TransportError` variant, including non-`DaemonNotRunning`
   failures, with a stable code, cause payload, and recovery steps
   (RBP-F002).
5. `boundary:sc-transport` — `DaemonNotRunning` exposes
   `sc_observability_types::ErrorCode`. `cargo tree -p sc-transport -e normal --prefix none`
   has a `sc-observability-types ` line at the a-1 recorded version and has no
   `sc-command ` line and no `sc-observability ` line (crate name, then a
   space, so the types crate does not match the logging crate). `AppName`
   rejects an invalid name at construction, public constructors take
   `AppName`, and the type exposes `as_str` or `AsRef<str>` and does not
   implement `Deref` (RBP-F005).

## This sprint does not close

Daemon lifecycle, CLI auto-start, application retries, service-manager
integration, and generated CLI tests are outside this boundary. Root
`just lint` and `sc-lint-boundary analyze` close in a-5.

## Paths to delete

None.

## Required validation

Run default and server crate tests/doctests, feature-specific dependency-tree
checks, and workspace build once registered. Review the manifest against
REQ-RUN-0005. Confirm the crate ADR error-inventory table, the
`DaemonNotRunning` code type, and the `AppName` construction tests.
`sc-lint-boundary analyze` is the a-5 root-lint gate, not a command this
sprint adds.
