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
classification, and the public error surface. Record those choices in the
owned sc-transport requirement/ADR sources before implementation proceeds
beyond the facade. Do not add auto-start APIs.

## Public contract produced

This sprint owns the sc-transport types and methods in the canonical
[minimal public contract handoff](phase-a-plan.md#minimal-public-contract-handoff):
`Endpoint`, `TransportConfig`, `resolve_endpoint`, `Client::new`, `get`,
`post`, `bind`, `Listener`, and `TransportError`. `Client::new` retains the
application name needed for `DAEMON.NOT_RUNNING`; endpoint resolution remains
separate. The crate-local requirement and ADR sources are the decision record.

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
5. **REQ-RUN-0005** — Boundary manifest for the actual facade and edges.

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

## This sprint does not close

Daemon lifecycle, CLI auto-start, application retries, service-manager
integration, and generated CLI tests are outside this boundary.

## Paths to delete

None.

## Required validation

Run default and server crate tests/doctests, feature-specific dependency-tree
checks, workspace build once registered, and boundary validation.
