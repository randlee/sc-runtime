# Initial release observability shim

**Status:** Approved release amendment, 2026-10-31; implementation under verification.
**Owner:** `run-h0h` release work; planning task `run-h0h.1`.
**Authority:** [ADR-RUN-0017](../../architecture.md#adr-run-0017-initial-release-observability-shim),
[REQ-RUN-0702](../../requirements.md#req-run-0702-release-v010-crates-published-repository-tagged).

## Scope and owner direction

Owner request, 2026-10-31, in this planning conversation:

> proceed w/ plan to include a shim.  ideally the shim is not an additional crate we would need to publish to crates.io.  We could add a low level dependency to sc-compose 1.5.0 and add the logging interface there today w/ explicit plan to remove this dependency and add a boundary once 1.5.1 ships.

The immediate correction was:

> oops, sc-observability, not sc-compose

The corrected direction authorizes planning the 1.5.0 bridge and explicit
upstream cleanup. This amendment records that request after
Phase B closed in PR 96 at `c3295c2320c7d7bc801665b99d9aee008fef2a1c`.
It does not reopen b-13–b-18, add a phase/sprint or authorize publication.
The source implementation is under verification; no release proof is claimed.

Use existing `sc-command` for the small injection contract. Its existing registry
`sc-observability-types = "1.5.0"` dependency supplies the values and errors.
Runtime already depends on command; cron gains the sole temporary SDK edge.
The concrete logger and a private inline adapter live only in generated daemon
source. Publish the same five SDKs, with no sixth shim crate and no new SDK
edge to full `sc-observability`, OTLP, sc-compose or ATM.

## Current and planned graphs

At the baseline, runtime/cron import a local package impersonating types 1.5.1,
while explicit development generation inserts an unpublished 0.0.0 adapter
around the registry 1.5.0 logger. The normal release template expects a future
upstream trait/direct Arc conversion; the release guard still requires it.
These are existing implementation facts, not the planned release policy.

```text
CURRENT development:
runtime/cron -> local types facade 1.5.1 -> registry types 1.5.0
application -> local dev adapter -> registry logger 1.5.0

PLANNED initial 0.1.1 release:
runtime --existing server edge--> command::observability -> registry types 1.5.0
cron --temporary defaults-off edge--> command::observability
runtime/cron -------------------------------> registry types 1.5.0
application -> private inline LoggerAdapter -> registry v2::Logger 1.5.0
application -> one SharedCanonicalLog cloned into runtime and cron

FUTURE capability-proven upstream cleanup:
runtime/cron -> upstream types::CanonicalLog/SharedCanonicalLog
application Arc<v2::Logger> -> upstream SharedCanonicalLog directly
cron -X-> command (original forbidden edge restored)
```

`sc-config` and `sc-transport` gain no observability types. Command remains a
leaf: it depends on no sibling SDK. Runtime never depends on cron; cron never
depends on runtime/config/transport/db. ANC-031's all-feature normal/build,
aliased/transitive sc-compose/ATM package and production-import exclusions
remain absolute, including the existing external-CLI host adapter distinction.

## Exact API, ownership and feature strategy

Create unconditional `pub mod observability` in `crates/sc-command/src/lib.rs`.
Export only the named contract items from `sc_command::observability`:

```rust
pub trait CanonicalLog: Send + Sync {
    fn service_name(&self) -> &sc_observability_types::ServiceName;
    fn emit(&self, event: sc_observability_types::LogEvent)
        -> Result<(), sc_observability_types::v2::EventError>;
}
pub type SharedCanonicalLog = std::sync::Arc<dyn CanonicalLog>;
// NoopCanonicalLog::new(ServiceName), named_or_default(&'static str)
// impl CanonicalLog for NoopCanonicalLog: emit returns Ok(())
```

Port the existing small Noop and doubles without adding a runtime, queue or
sink. `test-util = []` exposes `RecordingCanonicalLog::{new, events, clear}`
and `RejectingCanonicalLog::{new, attempts}` from that module, using std-only
synchronization and poison recovery in the recorder. Keep command `default=[]`
and `server` unchanged. Runtime's existing `test-util` and cron's existing
`test-utils` remain distinct features. Enable command/test-util only through
SDK dev-dependencies; the normal cron dependency is exactly:

```toml
sc-command = { version = "0.1.1", path = "../sc-command", default-features = false }
```

No feature forwarding, server activation, alias, build/target alternative or
second contract definition is permitted. The published sibling path+version
dependency becomes a registry requirement in Cargo's normalized manifest.
Runtime uses its existing server-enabled command dependency. Runtime, cron,
generated application and downstream custom receivers import handles/Noop/
doubles from `sc_command::observability`; canonical `ServiceName`, `LogEvent`,
`EventError`, and all values remain direct published types imports. Do not
wildcard re-export values or define lookalike error types.

Temporarily pin command's types dependency to `=1.5.0`; root/runtime/cron may
retain caret `1.5.0` and unify through that exact edge. Pin the generated logger
and types explicitly to `=1.5.0`, with root/generated locks proving the exact
registry pair. Logger 1.5.0 itself requires types with a caret, so its pin alone
cannot bound validator/type identity. This explicitly amends ADR-CMD-0001's
rejection of exact pins and REQ-CMD-0002/NFR-CMD-0001/0002 for this bridge only.
The cost is that a consumer requiring newer types cannot resolve this bridge
selection. Remove the exact pin during capability-proven upstream cleanup;
`cargo update` alone cannot migrate nominal trait ownership.

## Private application adapter and panic policy amendments

In `template/crates/daemon/src/observability.rs`, a private `LoggerAdapter`
contains `Arc<sc_observability::v2::Logger>` and implements the command trait.
`service_name` forwards the validated identity. `emit` calls only the real
`try_log(event)` admission method; ordinary returned `EventError`s pass through
unchanged. Build one Logger, retain its owner Arc, construct one adapter Arc,
and clone that same shared handle into runtime and cron. Never clone Logger,
add another queue/sink, use `Logger::log`/`enqueue_blocking`, or move lifecycle
ownership into a library. Drain/join consumers and drop their handles, then
drop `Observability.shared`, before the owner invokes shutdown. Keep existing
startup/run/shutdown failure precedence and fixed agent-facing diagnostics.

Published 1.5.0 `try_log` can panic when internal runtime locks are poisoned.
The existing absolute no-panic policy cannot simply be claimed unchanged.
ADR-RUN-0017 explicitly amends ADR-RUN-0006/0007, NFR-RUN-0002/0009 and
NFR-CMD-0003 for these two bounded dispositions:

- Preserve Noop's infallible legacy construction with an invariant exception
  only for `ServiceName::new("noop")` on that compile-known fixed literal.
  Validate `noop`, `sc-runtime` and `sc-cron` against the pinned and supported
  types versions in tests. No caller/environment-derived value may reach an
  `expect`; an invalid supplied name falls back to the validated fixed value.
  This is a named policy exception, never a comment-only waiver.
- Use a small adapter-local `catch_unwind(AssertUnwindSafe(...))` around **only**
  `logger.try_log(event)`. Map unwind to a freshly constructed, value-free
  `EventError::Routing` with fixed code
  `DAEMON.OBSERVABILITY_ADMISSION_PANIC` and fixed cause `Logging admission panicked.` and recovery
  `Restart the daemon to restore logging; the application operation is unaffected.`
  Use infallible 1.5.0 `ErrorCode::new_static`, `ErrorContext::new` and
  `Remediation::recoverable` with an empty command list.
  Never include the panic payload, event fields, provider text, or recursive
  logging. Keep native Results unchanged. Do not catch arbitrary library
  callbacks, startup/shutdown, or add a panic framework. Document why the
  logger ownership/invariants make the narrowly scoped assertion safe.

Unwind containment cannot catch `panic=abort`, allocator abort or process
termination; the panic hook runs before a catch and may print, so caught
payload confidentiality is not guaranteed by this adapter. Do not install a
global hook or claim universal panic isolation. Healthy-state admission has
no queue-capacity wait or sink I/O but takes internal locks; it is not
wait-free and may block on lock contention. Custom receivers remain trusted
prompt-return/no-panic callbacks. Best-effort rejection never changes HTTP,
store, cron or runtime outcomes. Test real poison/unwind mapping separately
from native saturation/disconnection and abort limitations.

## Implementation owned_paths

These are the bounded future implementation surfaces, not changes made by
this documentation PR. Contract import movement preserves existing algorithms.

**Contract and import migration:**

```text
Cargo.toml
Cargo.lock
crates/sc-command/Cargo.toml
crates/sc-command/src/lib.rs
crates/sc-command/src/observability.rs (new)
crates/sc-command/src/observability/test_util.rs (new)
crates/sc-command/src/observability/tests.rs (new)
crates/sc-runtime/Cargo.toml
crates/sc-runtime/src/{builder,router,testing,tests}.rs
crates/sc-runtime/tests/{observability_contract,observability_http}.rs
crates/sc-cron/Cargo.toml
crates/sc-cron/src/{lib,sqlite,hook,process,monty_host,jitter,worker,monty_run,coord,monty_compose,scheduler}.rs
crates/sc-cron/src/scheduler/run_due.rs
crates/sc-cron/src/worker/event_contract_tests.rs
crates/sc-cron/tests/{observability,observability_contract,observability_components}.rs
crates/sc-cron/tests/hook_environment/{resolution,failure_boundaries}.rs
crates/sc-cron/tests/worker/lifecycle.rs
```

**Generated composition, tests and enforcement:**

```text
template/Cargo.toml
template/workspace.template.toml
template/Cargo.lock (if generated/checked in by the existing pipeline)
template/crates/daemon/Cargo.toml
template/crates/daemon/src/{observability,cron,composition}.rs
template/crates/daemon/tests/cron_environment.rs
template/crates/daemon/src/observability_adapter_tests.rs (new)
scripts/new_project.py
scripts/check_observability_dependencies.py
scripts/check_observability_release.py
scripts/tests/test_observability_dependencies.py
tests/unit/test_observability_release.py
tests/integration/generation/test_observability.py
boundaries/sc-command/sc-command.toml
boundaries/sc-cron/sc-cron.toml
boundaries/sc-runtime/sc-runtime.toml (contract source notes only)
sc-lint.toml
justfile
.github/workflows/{template,crates}.yml
release/sc-publish-consumer-input.json
release/publish-preflight.toml (kit-generated)
template/release/publish-preflight.toml (only if owned by kit inventory)
```

**Deletion/retirement:** delete `dev/observability-types-shim/**` and
`dev/observability-logger-adapter/**` only after their contract and real-adapter
tests are ported. Remove both workspace entries/exclusions, every local facade/
adapter dependency and lock identity, `apply_development_observability`, its
`--development-observability` flag, flag-specific rewrites/tests/workflow
plumbing and stale README references. Retain ordinary checkout patches for
the five SDKs solely in explicit prepublication generated-app tests. A root
maintainer patch never propagates to an independent consumer root.

**Normative/doc sync:** `docs/{architecture,requirements}.md`,
`docs/sc-{command,runtime,cron}/{architecture,requirements}.md`,
`docs/sc-runtime/observability-injection.md`, `docs/development/sc-publish.md`
and this plan; add release-only supersession notices to the historical phase
index and b-13–b-18 plans. Do not edit approved bodies or locked ANC snapshots.
The release owner maintains Beads/anchor authority separately.

## Guards and acceptance

Amend exactly the cron boundary to permit command and remove its command
forbidden edge; command allowed_dependents becomes runtime+cron. Retain every
reverse edge and other prohibition. Change only the old command rejection in
cron default/Monty tree checks. Add an isolated cron-root proof that neither
command/server nor axum/rmcp leaks: workspace all-features cannot prove it,
because runtime legitimately unifies server. Preserve the isolated CLI proof.

Expand the observability guard to all five SDK normal/build closures, all
supported production feature/target combinations, and resolved package
identities (aliases/transitive edges included). Reject full logger, OTLP,
deleted local packages/overrides and any extra temporary edge. Exact manifest
checking rejects default/server/test-util activation on cron's normal edge.
Keep existing production tool boundary checks intact and mandatory; wire the
expanded observability/edge guard into lint. Replace old release predicates
with explicit initial-shim and cleanup-capability checks, not a disabled or
arbitrary-version guard. Add negative fixtures for each bounded exception.

After implementation, run the existing gates with these focused commands;
this plan has executed none of them:

```sh
cargo fmt --all -- --check
cargo +1.94.1 test -p sc-command --no-default-features
cargo +1.94.1 test -p sc-command --features server,test-util
cargo +1.94.1 test -p sc-runtime --features test-util
cargo +1.94.1 test -p sc-cron
cargo +1.96.1 test -p sc-cron --features monty
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/check_observability_dependencies.py
python3 -m unittest discover -s scripts/tests -p test_observability_dependencies.py
python3 -m unittest discover -s tests/unit -p test_observability_release.py
python3 tests/unit/test_production_tool_boundary.py --check --tree
python3 scripts/check_observability_release.py
python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_contract_sources
just test-generation
just lint
just test
cargo +1.96.1 publish --dry-run --locked -p sc-config -p sc-transport -p sc-command -p sc-runtime -p sc-cron
```

Inspect each actual `.crate` archive and normalized manifest: registry version
requirements, no local facade/adapter/path/git/patch dependency and no escaped
dev content. Record Cargo/source identities, checksums and verified builds.
The single dry run selects all five interdependent SDKs before any upload;
Cargo supports this in the pinned toolchains. A list-only or `--no-verify`
result is not build proof. No independent single-package dry-run prerequisite
should force publication of siblings to verify the source release.

Acceptance includes one trait/types identity, shared Arc identity, legacy
Noops, downstream custom receiver compile tests, ordered recorder/clear,
rejecting attempts and poison recovery. Port real 1.5.0 adapter JSONL/admission,
queue-full, disconnected, poison/unwind and lifecycle tests to generated daemon
tests. Use deterministic barriers for saturation, not timing-only sleeps.
Retain HTTP/cron JSONL, no-edits generation, shutdown precedence and isolated
SIGINT/SIGTERM lifecycle proof. Record exact isolated cron default/Monty and
CLI normal/build metadata/feature closure; no command/server leakage.

Retain controlled-host benchmark requirements, qualified pinned-kit inventory/
install/drift, release preflight, credentials and human release authorization.
Change installed kit outputs only by editing consumer input and reinstalling
the qualified pin; do not hand-edit vendored publisher workflows/agents.

Only after all five authorized publications and index availability, execute:

```sh
SC_RUNTIME_REGISTRY_PROOF=1 python3 -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_mode_uses_registry
```

That test must generate with `--release` outside the checkout and verify all
five SDKs plus full logger/types resolve from crates.io, with no path/git/patch
or hidden Cargo config substitution; build/test cron and Monty variants and
confirm the isolated CLI budget. It is **postpublication** proof and cannot be
a required prepublication check. A skipped test is never passing proof.
Five actual publications, proof and `v0.1.1` tag bind to one unchanged source
commit. The dry run authorizes no upload or release dispatch.

## Mandatory future cleanup outside the initial release epic

[Issue 97](https://github.com/randlee/sc-runtime/issues/97) tracks cleanup as a
separate future task outside `run-h0h`; the initial release
does not depend on unpublished 1.5.1. At every release readiness run, refresh
registry availability independently of the old lock/cache and record source,
version, checksum and outcome. Registry indeterminacy fails release closed.
When the matching 1.5.1 pair is actually published, compile an isolated
registry-only capability probe for required trait/Noop/error signatures and
direct `Arc<sc_observability::v2::Logger>` to upstream
`sc_observability_types::SharedCanonicalLog`. A version string alone is never
proof. A published but incompatible pair blocks release for remediation;
when capability succeeds, shim mode must fail until cleanup is complete.
A later matching publication may satisfy an explicitly invoked probe too.

Cleanup removes the private application adapter and local contract/test-double
implementation; consumes the actual upstream types trait and direct Arc
coercion; removes cron's command dependency/imports; restores command's sole
runtime dependent and cron's original forbidden edge/tree gates; removes
temporary policy state and refreshes registry locks/proofs. It retains every
full-runtime/OTLP and sc-compose/ATM exclusion and application lifecycle rule.

The local trait and future upstream trait are nominally distinct Rust types.
A source-path `pub use` from command can be retained only if compatibility
fixtures and an explicit version strategy require it; it cannot retain the
cron edge or make old/new published SDK mixtures share identity. Test a representative downstream custom impl/current consumer and the new
upstream direct probe; require compatible dependency minima and one resolved
contract owner for the supported selection. The migration version is not
predetermined: choose a coordinated breaking pre-1.0 minor line if needed; do not promise that a patch-only re-export is nonbreaking or
add a permanent adapter/blanket impl to avoid the boundary restoration.
