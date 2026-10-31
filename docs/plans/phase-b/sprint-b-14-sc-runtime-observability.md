---
id: b-14
phase: b
status: complete
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
  - REQ-RT-0006
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
  - ADR-RT-0002
  - ADR-RT-0003
  - ADR-RT-0004
owned_paths:
  - crates/sc-runtime/src/builder.rs
  - crates/sc-runtime/src/router.rs
  - crates/sc-runtime/src/testing.rs
  - crates/sc-runtime/src/tests.rs
  - crates/sc-runtime/tests/observability_http.rs
must_follow: [b-13]
parallel_safe: [b-15, b-16]
recommended_agent: arch-ctm
recommended_model: terra
---

> **Release prerequisite supersession — 2026-10-31:** This completed Phase B
> body is historical. Only its initial-publication 1.5.1/direct-Arc prerequisite
> is superseded by the [initial release shim amendment](initial-release-observability-shim.md)
> and ADR-RUN-0017. Its approved implementation/evidence is not rewritten;
> future cleanup is tracked in [issue 97](https://github.com/randlee/sc-runtime/issues/97).


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

The `builder.rs` change is limited to the serial b-13 consumer seam: carry the
existing shared logger into request middleware, without changing builder APIs.

## Event contract

| Target | Action | Point | Required fields |
|---|---|---|---|
| `http.server` | `request.start` | after request-id selection, before handler | request_id, method, matched-route template (or fixed `<unmatched>`) |
| `http.server` | `request.finish` | non-5xx response | request_id, method, matched-route template, status, elapsed_ms; `success` for 1xx/2xx/3xx or `client_error` for 4xx; no diagnostic |
| `http.server` | `request.error` | arbitrary application 5xx | request_id, method, matched-route template, status, elapsed_ms, `server_error`; diagnostic absent because no framework code is known |
| `http.server` | `request.error` | handler deadline wins | request_id, method, matched-route template, status, elapsed_ms, `deadline_remaining_ms = 0`, `timeout`; diagnostic code `RUNTIME.REQUEST_TIMEOUT` |
| `http.server` | `request.error` | shutdown drain deadline wins | same typed fields and timeout outcome; diagnostic code `RUNTIME.DRAIN_TIMEOUT` |

`LogEvent.service` comes from `CanonicalLog::service_name`. Header values,
bodies, raw URI paths, path-parameter values, query strings, authorization
data, and secrets are never logged. `path` is Axum's matched-route template;
an unmatched route uses `<unmatched>` rather than its raw URI.
All other envelope/field types follow b-13's canonical mapping; an arbitrary
handler response never has its body inspected to manufacture a code.
For every emitted `request.start`, normal response and the two framework
timeout branches emit exactly one terminal `request.finish` or
`request.error`; rejection by the logger never creates a second terminal
attempt or changes request execution.

## Required work and deliverables

1. REQ-RT-0004: replace `info_span!("http_request", ...)` with the three events above.
2. REQ-RT-0004: preserve the exact `x-request-id` used by responses and canonical events.
3. REQ-RT-0005: preserve existing timeout/drain envelopes and stable codes
   `RUNTIME.REQUEST_TIMEOUT` and `RUNTIME.DRAIN_TIMEOUT`.
4. NFR-RT-0004 / NFR-RUN-0009: add a recording-emitter fixture without a full-observability dependency.
5. NFR-RT-0002 / NFR-RT-0003: remove direct tracing calls from runtime source. The shared manifest is
   owned by b-13 and may retain the dependency for staged cleanup.
6. REQ-RT-0005 / NFR-RUN-0005 / ADR-RT-0004: add a runtime regression that
   uses the existing daemon fixture's crate-private shutdown trigger
   with deadline coordination and an isolated temporary instance root; it
   must not spawn a generated-project daemon process.
7. ADR-RUN-0011: production changes add no sleep or `block_on` call and hold
   no `std::sync::Mutex` guard across `.await`; test deadline polling is
   exempt.
8. REQ-RT-0006 / ADR-RT-0004: `DaemonFixture::start_with_logger` uses the
   existing production builder and fixture startup path. It preserves a fresh
   temporary instance root per fixture, caller-owned endpoint selection, the
   connected client, per-fixture shutdown and join, and the existing typed
   `RuntimeError` startup/serving contract. Injecting the logger adds no shared
   fixture state or separate daemon mode.

ADR-RT-0002 and ADR-RT-0003 remain binding for these owned runtime paths;
their applicability is unchanged. The builder edit only forwards the existing
logger handle, preserving singleton locking, assembly order, and opaque
project stores/router/MCP pass-through.

## Acceptance criteria

1. `boundary:BOUNDARY-sc-runtime`: `observability_http` passes only when a
   successful request emits start then finish with the response ID.
2. `boundary:BOUNDARY-sc-runtime`: `observability_http` passes only when a
   handler timeout emits `RUNTIME.REQUEST_TIMEOUT` and a drain timeout emits
   `RUNTIME.DRAIN_TIMEOUT`.
3. ADR-RT-0001 / `boundary:BOUNDARY-sc-runtime`: `observability_http` passes only when
   concurrent requests retain distinct IDs and the fixture shutdown regression
   drains and joins with deadline polling rather than fixed sleeps.
4. ADR-RUN-0004 / ADR-RUN-0007 / NFR-RUN-0002: `observability_http` passes only when
   queue-full and disconnected rejection returns immediately and changes no
   status, body, header, counter, request deadline, or error. [roots: `req:NFR-RUN-0002`]
5. `boundary:BOUNDARY-sc-runtime`: the required boundary lint and
   `cargo check --workspace` pass with tests using the types test double, not
   another workspace crate or a full runtime sink.
   All production release crates exclude sc-compose and ATM crate dependencies
   and Rust imports, enforced by boundary lint; test/benchmark references are
   permitted (ANC-031). [roots: `req:NFR-RT-0004`]
6. `boundary:BOUNDARY-sc-runtime`: `observability_http` passes only when an
   arbitrary application 5xx has no diagnostic and the two framework timeout
   branches carry exactly their fixed codes.
7. `boundary:BOUNDARY-sc-runtime`: table-driven 1xx/2xx/3xx/4xx/5xx and
   unmatched-route cases prove total outcome mapping, matched-template path
   cardinality, and one terminal event per started request without recording
   raw path parameters. [roots: `req:REQ-RT-0004`]

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


## Implementation evidence

Consumed b-13 final contract `0a6a9dcfb6258d06a470154e24e0a2aec915430e`.
The inherited b-13 builder stored the injected logger but dropped it at `drive`.
The authorized serial consumer change forwards that existing handle through
`serve` into request middleware. Builder APIs, constructors, and typestate are
unchanged. The real-daemon regression checks `Arc::ptr_eq` in an HTTP handler
and records both lifecycle events through that injected receiver. Independent
requirements and architecture QA must assess this inherited omission and fix;
this implementation evidence does not assert a QA verdict.

`observability_http` exercises 101/200/302/404/500 responses, unmatched routes,
validated and generated IDs, concurrent IDs, secret exclusion, both timeout
codes, full and disconnected queues, response/header/body/counter preservation,
handler cancellation, and real-daemon handle identity. The crate-private
shutdown regression uses an isolated temporary instance root, deadline polling,
and joins both the request and daemon task after the drain timeout.

Development validation passed: focused HTTP tests (5), full runtime tests with
`test-util`, formatting, workspace all-target Clippy and runtime feature Clippy
with warnings denied, `cargo check --workspace`, direct-tracing scan,
`just adr-run-0011`, and boundary lint (zero findings). Only the types test
receivers are used; no full observability sink dependency was added. The
upstream publication guard remains enabled for the b-18 release condition.
