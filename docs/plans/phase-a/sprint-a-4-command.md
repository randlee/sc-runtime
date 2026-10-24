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
  - NFR-RUN-0005
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0004
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CMD-0003
  - ADR-RUN-0001
  - ADR-RUN-0003
  - ADR-RUN-0006
  - ADR-RUN-0008
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
serialization-failure behavior. Record REQ-CMD-0002 in the crate ADR:
`OpError.remediation` is `Option<sc_observability_types::Remediation>` or a
documented empty variant. A custom serializer writes wire `suggested_action`
as a JSON string when present and JSON `null` when absent, never the tagged
`Remediation` object. `details` is `Option` and JSON `null` when absent.
Delete that OPEN. Do not treat the field type as already closed until the
ADR matches this handoff. Publish a **library wire-failure inventory** before
the facade freezes (RBP-F001). **Seal `IntoMcp`** with a private supertrait;
only `Result<T, OpError>` implements it (RBP-F006). Record in the ADR that
**`IntoMcp` is static dispatch only** (`self` by value; no `dyn IntoMcp`)
(RBP-F008). Leave application command shape open. Do not edit
`docs/architecture.md`.

## Public contract produced

This sprint owns the sc-command types and conversions in the canonical
[minimal public contract handoff](phase-a-plan.md#minimal-public-contract-handoff):
`Envelope`, `OpError`, `ErrorKind`, result conversion, and server-feature
Axum/rmcp conversions. The crate-local requirement and ADR sources are the
decision record. This sprint amends `docs/sc-command/architecture.md` and
`docs/sc-command/requirements.md` only. It does not edit
`docs/architecture.md`. Application command shape remains outside this
boundary.

## Dependencies and parallel safety

- `must_follow a-1`: consumes demonstrated Axum/rmcp types and versions.
- `parallel_safe a-2` and `a-3`: owned paths and boundaries are disjoint.

## Deliverables

1. **REQ-CMD-0001 / REQ-CMD-0003 / REQ-CMD-0006** — A generic, lossless
   envelope that converts results without discarding received versions or
   unknown JSON payload content.
2. **REQ-CMD-0002 / ADR-CMD-0001** — `OpError` stores
   `sc_observability_types::ErrorCode`. `remediation` is
   `Option<sc_observability_types::Remediation>` or a documented empty
   variant. A custom serializer writes wire `suggested_action` as a JSON
   string or JSON `null`. `details` is `Option` and JSON `null` when absent.
   The tagged `Remediation` object is not the wire form. The crate depends on
   the `sc-observability-types` version a-1 recorded and does not depend on
   `sc-observability` or `sc-observability-otlp`. Amend the ADR-CMD-0001
   sentence that called this the only library observability dependency so it
   matches a-1: `sc-transport` may depend on `sc-observability-types` for
   `ErrorCode` only. Before the facade freezes, ADR-CMD-0001 holds a library
   wire-failure inventory. The rows are an invalid or contradictory envelope
   discriminant, serialization failure, and the five `ErrorKind` mappings.
   Each row has a stable code, cause, recovery steps, and a docs link where
   one exists (RBP-F001). Widget create/get and other application operation
   codes are not rows.
3. **REQ-CMD-0004 / REQ-CMD-0005** — Server-feature conversions use ordinary
   Axum and rmcp types. `IntoMcp` is sealed with a private supertrait and
   implemented only for `Result<T, OpError>`; downstream callers can still
   call the method (RBP-F006). `into_mcp` takes `self` by value. ADR-CMD-0001
   records that dispatch is static and that `dyn IntoMcp` is not a supported
   API (RBP-F008). Serialization failure returns an error and does not panic.
4. **NFR-CMD-0001 / NFR-CMD-0002** — Minimal default/server feature graphs,
   focused tests, and a compiling standalone README example.
5. **REQ-RUN-0005 / NFR-CMD-0001** — Boundary manifest at
   `boundaries/sc-command/sc-command.toml`. Amend NFR-CMD-0001 so the default
   direct dependencies are `serde`, `serde_json`, and `sc-observability-types`
   at the a-1 pin, and delete the `serde_json` OPEN. `allowed_dependencies`
   includes `sc-observability-types`.    `forbidden_edges` keeps
   `sc-observability` and `sc-observability-otlp` and does not list
   `sc-observability-types`. Once `crates/sc-command/Cargo.toml` exists,
   run the single-crate `sc-lint lint sc-boundary` command a-1 recorded.
   Exit 0 is the gate. Do not paste analyzer output into the pull request.
   Do not wait for a-5. Do not own the permanent root workspace `Cargo.toml`.

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
5. `ADR-CMD-0001` — the wire-failure inventory lists an invalid or
   contradictory envelope discriminant, serialization failure, and the five
   `ErrorKind` mappings, each with a stable code, cause, recovery steps, and
   a docs link where one exists. It has no widget create/get row and no other
   application operation code. `OpError.remediation` is
   `Option<sc_observability_types::Remediation>` or the documented empty
   variant. A custom serializer writes `suggested_action` as a JSON string or
   JSON `null`, and `details` is JSON `null` when absent. The tagged
   `Remediation` object is not on the wire. `cargo tree -p sc-command -e normal --prefix none`
   has a `sc-observability-types ` line at the a-1 recorded version and has no
   `sc-observability ` or `sc-observability-otlp ` line (crate name, then a
   space) (RBP-F001).
6. `req:NFR-CMD-0001` — the amended default dependency list is `serde`,
   `serde_json`, and `sc-observability-types`. Analyze accepts the types
   crate while `forbidden_edges` still contains `sc-observability`.
   The single-crate `sc-lint lint sc-boundary` command a-1 recorded exits 0
   on `boundaries/sc-command/sc-command.toml`. Analyzer output is not pasted
   into the pull request. The `serde_json` OPEN is gone.
7. `boundary:sc-command` — NFR-RUN-0009 for this crate: serialization
   failure returns an error and does not panic; `unwrap`, `expect`,
   `panic!`, `unreachable!`, `todo!`, and `unimplemented!` are not reachable
   from a public function; no public signature names `anyhow::Error` or
   `Box<dyn Error>` except the recorded `IntoMcp` `McpError` exception.
   Non-test code under `crates/sc-command/src` contains no literal `/tmp`,
   `/Users/`, `/home/`, or `C:\Users` path. No test binds a fixed TCP port.
   Tests that write files use a temporary directory. a-5 closes the
   repo-wide NFR-RUN-0005 parallel `just test` runs and the four-crate
   NFR-RUN-0009 grep.

## This sprint does not close

Application command declarations, CLI rendering policy, attribute generation,
and end-to-end surface parity close outside Phase A or in the minimal template
proof only. Widget and other application operation codes stay in the generated
service. a-6 documents the example operation inventory on the generated
service and does not extend ADR-CMD-0001. This sprint runs the boundary
analyzer on its own manifest. Root `just lint` across all four manifests
stays with a-5. Repo-wide parallel `just test` (NFR-RUN-0005) and the
four-crate panic grep (NFR-RUN-0009) stay with a-5. This sprint does not
edit `docs/architecture.md` or `docs/requirements.md`.

## Paths to delete

None.

## Required validation

Run default/server tests and doctests, dependency-tree checks, and workspace
build once registered. Review the manifest against REQ-RUN-0005. Confirm
the ADR-CMD-0001 wire-failure inventory, the `suggested_action` string-or-null
serializer, the sealed `IntoMcp` impl, and the absence of `sc-observability`
from the normal tree.
Run the single-crate `sc-lint lint sc-boundary` command a-1 recorded on
this crate's manifest.
