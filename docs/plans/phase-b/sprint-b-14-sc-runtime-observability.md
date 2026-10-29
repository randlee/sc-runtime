---
id: b-14
phase: b
status: planned
closure_type: boundary
target_boundary: sc-runtime
track: observability
wave: 8
branch: sprint/b-14-sc-runtime-observability
worktree: sprint/b-14-sc-runtime-observability
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-RT-0004
  - REQ-RT-0005
  - NFR-RT-0002
  - NFR-RT-0003
  - NFR-RT-0004
  - NFR-RUN-0005
  - NFR-RUN-0002
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0004
  - ADR-RUN-0007
  - ADR-RUN-0011
  - ADR-RT-0001
  - ADR-RT-0004
owned_paths:
  - crates/sc-runtime/src/router.rs
  - crates/sc-runtime/src/testing.rs
  - crates/sc-runtime/src/tests.rs
  - crates/sc-runtime/tests/observability_http.rs
must_follow: [b-13]
parallel_safe: [b-15, b-16]
recommended_agent: arch-ctm
recommended_model: terra
---

# Sprint b-14 — sc-runtime canonical HTTP events

## Goal

Implement the `sc-runtime` half of the fixed `SharedCanonicalLog` contract.
Replace the request tracing span with canonical lifecycle events while
preserving request IDs, response envelopes, metrics, timeouts, and shutdown.

## Relations

`must_follow` b-13 because this sprint consumes `SharedCanonicalLog` in
runtime state and the fixed `DaemonBuilder::logger` contract. Merge b-13 into
this branch before every development or fix round. It is `parallel_safe` with
b-15 and b-16 because their owned paths and production boundaries are
disjoint.

## Event contract

| Target | Action | Point | Required fields |
|---|---|---|---|
| `http.server` | `request.start` | after request-id selection, before handler | request_id, method, path without query |
| `http.server` | `request.finish` | non-5xx response | request_id, method, path, status, elapsed_ms; `success` for 2xx/3xx or `client_error` for 4xx; no diagnostic |
| `http.server` | `request.error` | arbitrary application 5xx | request_id, method, path, status, elapsed_ms, `server_error`; diagnostic absent because no framework code is known |
| `http.server` | `request.error` | handler deadline wins | request_id, method, path, status, elapsed_ms, `deadline_remaining_ms = 0`, `timeout`; diagnostic code `RUNTIME.REQUEST_TIMEOUT` |
| `http.server` | `request.error` | shutdown drain deadline wins | same typed fields and timeout outcome; diagnostic code `RUNTIME.DRAIN_TIMEOUT` |

`LogEvent.service` comes from `CanonicalLog::service_name`. Header values,
bodies, query strings, authorization data, and secrets are never logged.
All other envelope/field types follow b-13's canonical mapping; an arbitrary
handler response never has its body inspected to manufacture a code.

## Required work and deliverables

1. REQ-RT-0004: replace `info_span!("http_request", ...)` with the three events above.
2. REQ-RT-0004: preserve the exact `x-request-id` used by responses and canonical events.
3. REQ-RT-0005: preserve existing timeout/drain envelopes and stable codes
   `RUNTIME.REQUEST_TIMEOUT` and `RUNTIME.DRAIN_TIMEOUT`.
4. NFR-RT-0004 / NFR-RUN-0009: add a recording-emitter fixture without a full-observability dependency.
5. NFR-RT-0002 / NFR-RT-0003: remove direct tracing calls from runtime source. The shared manifest is
   owned by b-13 and may retain the dependency for staged cleanup.
6. REQ-RT-0005 / NFR-RUN-0005 / ADR-RT-0004: add a runtime regression that
   uses the existing real-daemon fixture to drive the signal/shutdown path
   with deadline coordination and an isolated temporary instance root; it
   must not spawn a generated-project daemon process.
7. ADR-RUN-0011: production changes add no sleep or `block_on` call and hold
   no `std::sync::Mutex` guard across `.await`; test deadline polling is
   exempt.

## Acceptance criteria

1. `boundary:BOUNDARY-sc-runtime`: `observability_http` passes only when a
   successful request emits start then finish with the response ID.
2. `boundary:BOUNDARY-sc-runtime`: `observability_http` passes only when a
   handler timeout emits `RUNTIME.REQUEST_TIMEOUT` and a drain timeout emits
   `RUNTIME.DRAIN_TIMEOUT`.
3. `boundary:BOUNDARY-sc-runtime`: `observability_http` passes only when
   concurrent requests retain distinct IDs and the real signal regression
   drains and joins with deadline polling rather than fixed sleeps.
4. ADR-RUN-0004 / ADR-RUN-0007: `observability_http` passes only when
   queue-full and disconnected rejection returns immediately and changes no
   status, body, header, counter, request deadline, or error.
5. `boundary:BOUNDARY-sc-runtime`: the required boundary lint and
   `cargo check --workspace` pass with tests using the types test double, not
   another workspace crate or a full runtime sink.
6. `boundary:BOUNDARY-sc-runtime`: `observability_http` passes only when an
   arbitrary application 5xx has no diagnostic and the two framework timeout
   branches carry exactly their fixed codes.

## Required validation

```sh
cargo test -p sc-runtime --test observability_http --features test-util
cargo test -p sc-runtime --features test-util
cargo clippy -p sc-runtime --all-targets --features test-util -- -D warnings
cargo check --workspace
! rg -n '#\[(tracing::)?instrument|tracing::|(trace|debug|info|warn|error|event|span|trace_span|debug_span|info_span|warn_span|error_span)!' crates/sc-runtime/src
just adr-run-0011
sc-lint lint sc-boundary --root . --config sc-lint.toml
```

## This sprint does not close

- File/console sinks or generated observability configuration (b-18).
- Cron events (b-15) or logger construction/shutdown.
