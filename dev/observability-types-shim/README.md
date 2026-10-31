# Temporary development facade

This unpublished package permits local development against the planned canonical
logging injection API while the matching upstream 1.5.1 packages are unavailable.
It re-exports the exact registry `sc-observability-types =1.5.0` package, preserving
its value types, v1 compatibility feature, and canonical error API. It supplies
only `CanonicalLog`, `SharedCanonicalLog`, `NoopCanonicalLog`, and `test-util`
recording/rejecting receivers. It does not supply or prove a production Logger
adapter, queue admission, saturation, disconnection, sinks, or lifecycle behavior.
Rust orphan rules prevent downstream consumers from implementing this facade's
trait for the external production Logger: neither belongs to the consumer crate.

The sibling `dev/observability-logger-adapter` package wraps the real registry
`sc-observability =1.5.0` logger. Its `LoggerAdapter` newtype implements this
facade's trait by forwarding to that logger's `service_name` and `try_log`.
This bridge uses the actual logger, queue and sinks. It is a local development
bridge, and does not prove the still-unpublished upstream 1.5.1 adapter.

The root `[workspace.dependencies]` entry uses an explicit development path.
New library consumers use `sc-observability-types.workspace = true`; their
dev-dependencies alone enable `test-util`. Existing `sc-command` continues to use
the registry 1.5.0 package and receives identical re-exported types.

Do not replace this path with `[patch.crates-io]`: Cargo unifies compatible
versions from the patched registry source. A forced 1.5.1 consumer then conflicts
with the facade's exact registry 1.5.0 dependency; unconstrained 1.5.0 consumers
instead leave the patch unused. Distinct path and registry sources avoid that
conflict without copying upstream code or creating wrapper value types.

An application retains ownership of the real logger and injects a separate Arc
around the adapter. Given an application-constructed `Arc<sc_observability::v2::Logger>`:

```rust,ignore
use sc_observability_dev_adapter::LoggerAdapter;
use sc_observability_types::SharedCanonicalLog;
use std::sync::Arc;

let injected: SharedCanonicalLog = Arc::new(LoggerAdapter::new(Arc::clone(&logger)));
// Pass injected to the library's .logger(...) or *_with_logger(...) API.
// The application keeps logger and owns its flush and shutdown lifecycle.
```

Libraries never create, flush, or shut down the underlying logger. The local
adapter changes only the trait boundary; it does not add another queue, sink,
or lifecycle policy.

The owner confirmed on 2026-10-30 that Phase B completes before upstream
1.5.1. The approved real 1.5.0 adapter supports the phase's actual shared-logger,
JSONL, shutdown-precedence, and isolated-CI signal-lifecycle evidence. These
proofs do not claim published 1.5.1 support. Release epic `run-h0h`, including
external prerequisite `run-03z.7`, owns the following removal/publication work
after the phase; phase-end readiness must report those release limits.

Before publication, remove both development directories and workspace exclusions,
remove the types dependency's `path` and the development adapter dependency,
update the lockfile to the actual matching registry 1.5.1 packages, and verify the
published logger adapter and its non-blocking contract tests. Replace construction
of the local `LoggerAdapter` with direct coercion of an Arc clone of the real
logger to `SharedCanonicalLog` when the upstream trait implementation is available.
Both development packages have `publish = false`; the repository publication
preflight rejects active development paths and requires the published contracts.
Optimized development builds and benchmarks remain permitted: a release build
profile is not publication authorization.

Development generation uses the explicit mode and the correct checkout recipe:

```sh
just --justfile Justfile new /tmp/sc-runtime-development-example --var-file wizard/fixtures/sqlite-mcp.json --development-observability
```

Use a fresh absent destination. Generated-project recipes select that project's
own `Justfile`. Never combine `--development-observability` with `--release`.
The release guard is expected to reject this bridge; optimized development
builds do not waive that publication fence.

Focused validation:

```sh
cargo test --manifest-path dev/observability-types-shim/Cargo.toml --all-features
cargo test --manifest-path dev/observability-types-shim/Cargo.toml --no-default-features
cargo test --release --manifest-path dev/observability-types-shim/Cargo.toml --all-features
cargo clippy --manifest-path dev/observability-types-shim/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path dev/observability-logger-adapter/Cargo.toml --locked
```
