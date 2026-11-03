# Portable fixture endpoint ownership — external intake design addendum

Read-only source assessment, 2026-11-03. Input `/tmp/phase-c-execution/c-4/qa-1/flaky-test-qa-result.md`, C4-FTQ-001 at beads-cache `e7bafc2e5bdda1bf480823c7dd3ac73d0e16b702`. Its reserve/drop interleaving is a source-proven risk, not an observed hosted CI failure. Its separate CI-C4-001 subprocess cleanup finding is not this upstream fixture task. No tests/builds, implementation, plan/Git/Beads writes, finding waiver or consumer QA closure.

## Existing upstream seam already solves listener ownership

At sc-runtime integration `725420b5980e4a0e4a85db3d77a227c9b18a8c88`:

- `crates/sc-transport/src/server.rs:48-49` explicitly supports TCP port zero. `Listener::local_endpoint:104-126` reads the actual bound TcpListener address, including its assigned nonzero port. `bind_tcp:339-347` creates and retains that listener. Endpoint parsing (`endpoint.rs:83-88`) accepts socket address port zero.
- `crates/sc-runtime/src/builder.rs:356-370` binds once through sc-transport, obtains local_endpoint, and sends the actual bound Endpoint through its private ready channel **without dropping or rebinding the listener**. The listener moves into axum::serve later in the same function.
- `crates/sc-runtime/src/testing.rs:429-460` receives that actual bound Endpoint, constructs the client and completes a real health-serving handshake before returning the fixture. Existing `DaemonFixture::endpoint/endpoint_str` expose that actual Endpoint publicly (`testing.rs:169-181`).

Therefore `DaemonFixture::start` can already use `config.endpoint_flag=Some("tcp:127.0.0.1:0".into())` directly and read its chosen port after startup. The proposed `start_with_cleanup` should reuse that exact path and retain it through cleanup/owned thread joins. **No transport source/API change or transport version bump is required.** No reservation listener, listener handoff, random-port loop or bind retry should be added.

REQ-RT-0006 currently says Windows callers choose a distinct endpoint; clarify that a supplied loopback port-zero endpoint requests a distinct OS-assigned port and the returned fixture endpoint is its actual usable address. Do not make fixtures invent a TCP endpoint implicitly, change endpoint precedence, or force Unix fixtures away from their normal private UDS path.

## Real-signal consumer needs a bound receipt, not a private-trigger replacement

The consumer's `runtime_lifecycle.rs:191-198` reserves `127.0.0.1:0`, drops it, fixes the released address in endpoint_flag, then starts production run_with_cleanup. Its start handshake at lines 61-77 reports stores ready **before bind**, not an owned endpoint. The test later sends a real process signal and asserts the actual application cleanup and instance lock; replacing it with DaemonFixture's signal-disabled private trigger alone would not preserve that acceptance.

Select the smallest additive runtime observation seam, frozen by c-1 and implemented in c-3 alongside cleanup-fixture work:

```rust
pub async fn run_with_cleanup_and_bound<FBound, FCleanup, Cleanup>(
    self,
    on_bound: FBound,
    cleanup: FCleanup,
) -> Result<(), RuntimeError>
where
    FBound: FnOnce(sc_transport::Endpoint) + Send + 'static,
    FCleanup: FnOnce() -> Cleanup,
    Cleanup: Future<Output = ()>;
```

The ready-state builder owns the callback. Invoke it once only after successful bind/local_endpoint while **the same listener remains owned**, before serving; no callback on a failed bind. It reports **bound**, not application readiness. A prompt callback can send Endpoint over a caller-owned channel; the caller then uses the actual address for its serving/application handshake. The method changes neither signal registration nor stop triggers. Existing run and run_with_cleanup delegate to the same internal path with an empty bound callback, preserving source compatibility and cleanup error precedence. Keep the private fixture shutdown trigger private.

The callback does not receive or transfer Listener ownership and cannot cause a rebind. It adds no public transport type wrapper. Document that application callbacks must return promptly and callers await the owning run future; no new panic-preemption guarantee is implied. A dropped bound-receipt receiver should be handled by the caller's callback without panicking, and must not detach runtime or skip cleanup.

Consumer application of this future public API: configure tcp:127.0.0.1:0, receive the real Endpoint from on_bound, then perform the existing serving/read-cancellation/real-signal/instance-lock/cleanup scenario against that endpoint. The original state handshake remains useful for application state but cannot stand in for the bind receipt. This avoids the reservation race without weakening the real-signal test or duplicating runtime internals.

## Ownership and validation changes

c-1 owns the added signature and the bounded REQ-RT-0005/0006, ADR-RT-0005/public-surface documentation amendment. c-3 owns runtime builder/RunControl plumbing and fixture implementation/tests, already within the same crate owner; no new sprint, no shared-wave path overlap and no change to the R1 executor/operation-double recut. c-4 consumes/publishes the existing affected sc-runtime 0.2.0 package and repeats exact registry fixture proof. sc-transport remains unchanged at its existing approved version. If implementation unexpectedly needs a transport edit, stop assuming that closure: explicitly qualify and include only the affected transport version in c-1/c-4 rather than bumping unrelated libraries.

Required focused tests, in the existing safe fixture venue:

1. Start port-zero cleanup fixtures concurrently; returned endpoints have nonzero ports and are distinct, health probes reach the corresponding fixture, and endpoint()/endpoint_str()/client agree on the actual bound endpoint.
2. After the bound receipt and while runtime/fixture owns the endpoint, a competing bind to **that exact endpoint** fails; no listener release is permitted between receipt and readiness. Use causal gates to control any pre-serving observation point and a planted reserve/drop handoff negative that permits a competitor to take the address; no timing sleeps or retry-to-green.
3. Retain cleanup callback once, real owned run/thread join, application stores/logger/instance lock retention through cleanup, and endpoint disposal ordering. Failure before bind emits no misleading bound receipt; bind/serve errors still execute cleanup according to the existing contract.
4. Keep independent isolated real Unix signals and native Windows Ctrl-C/Ctrl-Break tests using the bound observation seam. The portable in-process fixture still uses its private stop and does not install process handlers.

This is a small cohesive extension of the already approved portable fixture/cleanup compatibility scope. The external consumer finding stays open for its owning reviewer to verify against the actual authorized published baseline; this design addendum alone does not close it.
