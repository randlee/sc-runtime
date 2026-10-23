---
id: a-4
phase: a
status: planned
closure_type: boundary
target_boundary: sc-command
track: core
wave: 1
branch: sprint/a-4-command
worktree: sprint/a-4-command
recommended_agent: arch-ctm
recommended_model: terra
must_follow: [a-1]
parallel_safe: [a-2, a-3]
requirements:
  - REQ-CMD-0001
  - REQ-CMD-0002
  - REQ-CMD-0003
  - REQ-CMD-0004
  - REQ-CMD-0005
  - REQ-CMD-0006
  - NFR-CMD-0001
  - NFR-CMD-0002
  - NFR-CMD-0003
  - REQ-RUN-0002
  - REQ-RUN-0005
  - NFR-RUN-0003
  - NFR-RUN-0004
adrs:
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CMD-0003
  - ADR-RUN-0001
  - ADR-RUN-0003
  - ADR-RUN-0006
owned_paths:
  - "crates/sc-command/**"
  - "boundaries/sc-command/**"
  - "docs/sc-command/requirements.md"
  - "docs/sc-command/architecture.md"
---

# Sprint a-4 — sc-command

## Outcome and closure

Create one independently usable crate for a lossless response envelope and
standard REST/MCP conversions. It is a wire helper, not a command registry,
handler framework, CLI generator, or test-generation system.

## Decision prerequisites

Resolve envelope version/representation, error-kind/status mapping, and
serialization-failure behavior. **Close REQ-CMD-0002:** `OpError` stores
`sc_observability_types::Remediation` (wire `suggested_action` derived at
serialize time; no bare `Option<String>` as the authoritative type). Publish
an **operation error inventory** before the facade freezes (RBP-F001). **Seal
`IntoMcp`** with a private supertrait; only `Result<T, OpError>` implements
it (RBP-F006). Record in the ADR that **`IntoMcp` is static dispatch only**
(`self` by value; no `dyn IntoMcp`) (RBP-F008). Leave application command
shape open.

## Public contract produced

This sprint owns the sc-command types and conversions in the canonical
[minimal public contract handoff](phase-a-plan.md#minimal-public-contract-handoff):
`Envelope`, `OpError`, `ErrorKind`, result conversion, and server-feature
Axum/rmcp conversions. The crate-local requirement and ADR sources are the
decision record; application command shape remains outside this boundary.

## Dependencies and parallel safety

- `must_follow a-1`: consumes demonstrated Axum/rmcp types and versions.
- `parallel_safe a-2` and `a-3`: owned paths and boundaries are disjoint.

## Deliverables

1. **REQ-CMD-0001 / REQ-CMD-0003 / REQ-CMD-0006** — A generic, lossless
   envelope that converts results without discarding received versions or
   unknown JSON payload content.
2. **REQ-CMD-0002 / ADR-CMD-0001** — `OpError` stores
   `sc_observability_types::ErrorCode` and
   `sc_observability_types::Remediation`. Wire `suggested_action` is derived
   from `Remediation` at serialization time. The crate depends on the
   `sc-observability-types` version a-1 recorded and does not depend on
   `sc-observability` or `sc-observability-otlp`. Before the facade freezes,
   ADR-CMD-0001 holds an operation error inventory: one row per Phase A
   envelope operation, with stable code, cause, recovery steps, and a docs
   link where one exists (RBP-F001).
3. **REQ-CMD-0004 / REQ-CMD-0005** — Server-feature conversions use ordinary
   Axum and rmcp types. `IntoMcp` is sealed with a private supertrait and
   implemented only for `Result<T, OpError>`; downstream callers can still
   call the method (RBP-F006). `into_mcp` takes `self` by value. ADR-CMD-0001
   records that dispatch is static and that `dyn IntoMcp` is not a supported
   API (RBP-F008). Serialization failure returns an error and does not panic.
4. **NFR-CMD-0001 / NFR-CMD-0002** — Minimal default/server feature graphs,
   focused tests, and a compiling standalone README example.
5. **REQ-RUN-0005** — Boundary manifest at `boundaries/sc-command/*.toml` for
   the actual facade and edges. The manifest must be valid for
   `sc-lint-boundary analyze`. That command runs once a-5's root `just lint`
   exists; this sprint does not install sc-lint or own the root `justfile`.

## Acceptance criteria

1. `boundary:sc-command` — canonical success/failure/null/unknown-field vectors
   round-trip under default features.
2. `boundary:sc-command` — server conversions produce the decided HTTP and MCP
   carriers with the same envelope semantics.
3. `ADR-CMD-0003` — inspection finds no registry, handler macro, attribute
   macro, transport client, runtime dependency, or generated-test framework.
4. `ADR-CMD-0001` — `IntoMcp` is sealed by a private supertrait; the only
   implementation is `Result<T, OpError>`; a downstream crate can call
   `into_mcp` and cannot implement the trait; `into_mcp` takes `self` by
   value; the ADR states that dispatch is static and that `dyn IntoMcp` is
   not a supported API (RBP-F006, RBP-F008).
5. `ADR-CMD-0001` — the operation error inventory lists every Phase A
   envelope operation with a stable code, cause, recovery steps, and a docs
   link where one exists. `OpError.remediation` is
   `sc_observability_types::Remediation`. `cargo tree -p sc-command -e normal --prefix none`
   has a `sc-observability-types ` line at the a-1 recorded version and has no
   `sc-observability ` or `sc-observability-otlp ` line (crate name, then a
   space) (RBP-F001).

## This sprint does not close

Application command declarations, CLI rendering policy, attribute generation,
and end-to-end surface parity close outside Phase A or in the minimal template
proof only. Root `just lint` and `sc-lint-boundary analyze` close in a-5.

## Paths to delete

None.

## Required validation

Run default/server tests and doctests, dependency-tree checks, and workspace
build once registered. Review the manifest against REQ-RUN-0005. Confirm
the ADR-CMD-0001 error inventory, the `Remediation` field, the sealed
`IntoMcp` impl, and the absence of `sc-observability` from the normal tree.
`sc-lint-boundary analyze` is the a-5 root-lint gate, not a command this
sprint adds.
