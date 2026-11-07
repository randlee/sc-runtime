---
status: complete
branch: codex/restore-router-test-helper
worktree: /Users/randlee/github/sc-runtime-worktrees/codex/restore-router-test-helper
closure_type: boundary
target_boundary: sc-runtime
must_follow: []
parallel_safe: []
---

# Router test helper restoration

## Objective

Restore the former optional `sc_runtime::testing::instrument_router` API as a
small consumer testing utility. It applies the runtime's existing request
timeout, tracking, and canonical logging middleware to a caller-supplied Axum
router. The helper is available only with `test-util` and keeps the former
five-argument signature, including the `sc-command` 0.2.0 logger type.

This is an additive test utility. It does not configure or change the daemon's
private 30-second production request timeout, shutdown behavior, or startup
sequence. It adds no production dependency, runtime configuration, or version
change. The helper has no beads-cache-specific behavior.

## Governing requirements and decisions

- [REQ-RT-0006](../../sc-runtime/requirements.md): test utilities are exposed
  through the non-default `test-util` feature.
- [NFR-RT-0001](../../sc-runtime/requirements.md): amend the closed public
  surface to include this single feature-gated helper while retaining the
  existing builder, fixture, and accessor limits.
- [REQ-RUN-0005](../../requirements.md): update the sc-runtime facade manifest
  to include the exported helper.
- [ADR-RT-0001](../../sc-runtime/architecture.md): keep daemon assembly and
  lifecycle order unchanged; the helper is not a builder method, callback, or
  lifecycle hook.
- [ADR-RT-0003](../../sc-runtime/architecture.md): preserve ordinary router
  composition.
- [ADR-RT-0004](../../sc-runtime/architecture.md): the router unit test is not
  a daemon fixture or listenerless daemon mode.
- [ADR-RT-0005](../../sc-runtime/architecture.md): preserve existing cleanup
  and ownership behavior.
- [ADR-RUN-0001](../../architecture.md): the helper composes existing
  middleware around an ordinary Axum router; it introduces no wrapper type,
  registry, or framework dispatch layer.
- [REQ-RT-0004](../../sc-runtime/requirements.md): routes remain ordinary Axum
  routers and only existing middleware is composed.
- [NFR-RUN-0002](../../requirements.md), [NFR-RUN-0005](../../requirements.md),
  [NFR-RUN-0009](../../requirements.md), and
  [NFR-RUN-0003](../../requirements.md), [NFR-RUN-0004](../../requirements.md),
  [NFR-RT-0002](../../sc-runtime/requirements.md), and
  [NFR-RT-0003](../../sc-runtime/requirements.md): preserve nonblocking,
  isolated testing and dependency/public API constraints.
- Relevant middleware constraints are also governed by
  [ADR-RUN-0006](../../architecture.md), [ADR-RUN-0007](../../architecture.md),
  [ADR-RUN-0008](../../architecture.md), [ADR-RUN-0011](../../architecture.md),
  [ADR-RUN-0017](../../architecture.md), and [ADR-RUN-0202](../../architecture.md).

The closed surface, facade requirement, and ADR language must be amended in
place to authorize exactly this optional helper. The existing `DaemonFixture`
remains the only daemon-starting test fixture. Router middleware tests do not
start a daemon or introduce an in-memory daemon mode.

## Owned paths

- `crates/sc-runtime/src/testing.rs`
- `crates/sc-runtime/src/tests/observability_http.rs`
- `crates/sc-runtime/Cargo.toml` only if a dev-only virtual-time feature is
  required and not already unified
- `boundaries/sc-runtime/sc-runtime.toml`
- `docs/sc-runtime/requirements.md`
- `docs/sc-runtime/architecture.md`
- `docs/plans/maintenance/router-test-helper.md`
- `docs/project-plan.md`

## Implementation and acceptance

1. Export `testing::instrument_router` under the existing `test-util` module
   gate with the exact historical signature:
   `fn(axum::Router, SharedCanonicalLog, Duration, watch::Receiver<bool>, Duration) -> axum::Router`.
2. Compose the existing request-timeout middleware followed by request
   tracking/logger middleware with fresh request metrics. Do not copy or
   fork middleware behavior.
3. Keep the public module absent from default builds and do not add a
   production dependency, runtime option, or version change.
4. Amend NFR-RT-0001's closed surface and success criteria, ADR-RT-0001's
   no-other-public-method clause, and the sc-runtime boundary facade narrowly
   for this helper. Retain all existing builder method counts and fixture
   contracts.
5. Replace the private test-only duplicate with coverage through the public
   helper. Add an external-style public API regression that proves a caller
   selected short timeout returns the timeout response and cancels the handler
   future. Use deterministic coordination, owned tasks, and bounded cleanup;
   avoid sleeps, wall-clock assertions, fixed ports, and detached tasks.
6. Preserve existing default-feature tests and the production 30-second
   default. Do not weaken or skip causal timeout, tracking, or lifecycle tests.
7. Existing CI exercises the feature test target (`sc-lint.toml`:
   `sc-runtime-test-util`); Windows also runs the feature-gated library tests
   in `.github/workflows/crates.yml`. Keep the test in an already exercised
   target or update the existing target if necessary.
8. Validate the focused public feature test, existing runtime feature tests,
   rustdoc/API compilation, formatting, lint/boundary checks, and relevant
   default-feature tests. Record exact commands and outcomes in the handoff.

## Non-goals

No production timeout configurability, request lifecycle redesign, daemon
fixture changes, Beads-specific APIs or policies, dependency version changes,
release, or merge is part of this sprint.
