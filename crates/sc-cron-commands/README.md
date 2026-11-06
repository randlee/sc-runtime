# sc-cron-commands

Opt-in `cron.start`, `cron.end`, and `cron.list` share the same version-1
`CronRequest` and `sc_command::Envelope<CronResponse>` over CLI, HTTP and MCP.
A handler kind selects Rust, Python or explicitly enabled Monty; the verbs and
JSON semantics stay the same. Nothing starts a job automatically.

Defaults contain only DTOs and the object-safe `CronExecutor` port. `schema`
adds the canonical owner-local Schemars/OpenAPI traits. `scheduler` adds
`StandardCronExecutor`, native `sc_cron::CronOperations`, and application target
bindings. `monty` explicitly enables the interpreter and requires Rust1.96.1;
the other builds retain workspace MSRV1.94.1. `schema` and `scheduler` are
independent, and CLI clients can keep defaults off.

```toml
[dependencies]
sc-cron-commands = { version = "0.2.0", default-features = false, features = ["scheduler"] }
sc-cron = { version = "0.2.0", default-features = false }
```

## Selected default path

When generating from sc-runtime, select `cron=true`, `cron_commands=true`,
`db=sqlite`, and `mcp=false` unless MCP is wanted. For `cargo generate`, these
are values in its values file; the wizard answers use the same option names.
`cron_commands=false` is the default and selecting commands requires cron.
Selecting commands does not enable MCP. C4 supplies and verifies the actual
selected application root binding after consuming these libraries; this
crate's adapter alone does not install a runnable daemon or migrate a consumer.

The application root opens `SqliteScheduleStore` with its configured
`ScheduleLimits`, builds `Scheduler` with its clock/jitter, and configures
finite command capacity before starting work. It explicitly starts
`spawn_worker_cooperative(Arc<dyn Job>)`; that application `Job` dispatches
Rust payloads. It retains the single worker handle for physical join, clones
`worker.stop_control()` for the root operation context, and builds
`StandardCronExecutor::new(Arc::clone(&scheduler), bindings, context)`.
Python and Monty are authorized by the application bindings before schedule
creation. Python modules and Monty source are configuration, not a serialized
Rust function catalog. The same executor is passed to each selected UI.

The following public constructor recipe compiles for any native operations
provider and application binding; concrete generated root wiring and real SQL
cross-UI execution are C4's composition proof.

```rust,no_run
use std::sync::Arc;
use sc_cron::{CronOperations, StoreOperationContext};
use sc_cron_commands::{ApplicationTargetBindings, CronExecutor, StandardCronExecutor};

fn commands<O, B>(operations: Arc<O>, bindings: Arc<B>, context: StoreOperationContext)
    -> Arc<dyn CronExecutor>
where
    O: CronOperations + 'static,
    B: ApplicationTargetBindings + 'static,
{
    Arc::new(StandardCronExecutor::new(operations, bindings, context))
}
```

Applications implement `ApplicationTargetBindings::resolve` to select an
existing `HookTarget` and map opaque payload. This is the authorization point
for Python paths, trusted Monty programs, and application Rust dispatch; it
must not expose provider/request secrets through its returned `OpError`.
Environment policy uses the existing core `HookEnvironment` setters,
references, clears and absolute dotenv paths. Resolution of files and secret
references remains the application's configured environment-host policy at
execution; the adapter never reads environment variables or files itself.

```json
{"version":"1","command":"cron.start","request":{"name":"nightly","cron":"0 * * * *","timeout_ms":25000,"target":{"kind":"rust"},"payload":{"job":"sync"},"environment":{"dotenv":[],"variables":{}}}}
{"version":"1","command":"cron.end","request":{"name":"nightly","generation":1,"reason":"operator stopped"}}
{"version":"1","command":"cron.list","request":{"state":"all","limit":100,"cursor":null}}
```

Python substitutes `{"kind":"python","module":"/configured/nightly.py"}`;
Monty substitutes `{"kind":"monty","source":"...trusted program..."}` and
requires the `monty` feature. Unsupported Monty fails before bindings or
native operations. The application explicitly runs the Rust/Python/Monty
worker targets; enabling command selection never starts work by itself.

Inputs use core schedule-name, expression, payload, timeout and environment
limits. The aggregate serialized request limit is2MiB. End requires a positive
generation, exact name and nonempty reason bounded to64KiB with no NUL; no
missing-name operation ends all rows. List permits1..=1000 rows and a positive
exclusive id cursor; views expose id/generation/name/expression/target-kind,
enabled/state/next-run time, never source/environment/payload.
Unknown/duplicate struct fields and unsupported versions fail parsing. Opaque
JSON payload semantics remain those of `serde_json::Value`.

## Lifecycle, errors and overrides

The adapter derives a child context retaining the earliest root/request
cutoff and shared cancellation. It adds no timeout wrapper, native task owner,
second capacity queue, or scheduling database. Standard scheduler capacity is
64; `with_command_capacity` accepts1..=65536 before worker/command startup.
All executors sharing that scheduler share its capacity. Immediate
`CRON.COMMAND_CAPACITY_EXHAUSTED` refusal does not enqueue a waiter or mutate;
wait for owned work to finish before submitting a new command.

Stop UI intake, request stop with checked `ShutdownBudget::new(G,K)`, and
await `WorkerHandle::join_owned()` before dropping resources. A second stop can
shorten cutoffs. Keep the owning runtime alive until notification and physical
join finish. `DeadlineExceeded`/`Indeterminate` never imply rollback; error
details retain commit admission, observed persistence and late completion when
available. A committed-late result is still committed. Reconcile through a
separately authorized bounded read before considering a mutation replay.
Standard error messages exclude SQL/provider/source/environment/payload prose;
application binding/executor error policy remains application-owned.

Consumers can implement `CronOperations` for different persistence and shutdown
contracts, or implement `CronExecutor` for an entirely different command policy.
For custom operations, create ordinary-feature `WorkerStopControl::new()`
inside the owning Tokio runtime and pass `control.operation_context()` here.
The application closes intake, owns and physically joins every task/native
call/cleanup, then calls `finish_application_shutdown()` to acknowledge
completion. The library cannot verify untracked custom work. Framework-worker
controls require their worker's `join_owned()` instead. Overriding either port
transfers authorization, finite overload policy, generation conflicts,
persistence truth, deadline/cancellation and physical ownership to the consumer;
it does not inherit the bundled cooperative guarantee automatically.

See the [core cooperative example](../sc-cron/README.md#cooperative-consumers-and-physical-ownership)
and [custom-store extension example](../../tests/integration/cron-consumer/tests/application_schema.rs).
The bundled SQLite schema stays core-owned; consumers may keep their own
application schema and own its reconciliation guarantees. The intended fast
path is documented composition with explicit override points, not prevention
of every possible misuse.

## Local qualification

Run adapter tests with no features, `schema`, `scheduler`, `schema,scheduler`,
and explicit `monty` (Rust1.96.1). The isolated consumer fixture has no dev
dependencies, so its normal graph cannot be masked by workspace test unification:

```sh
cargo tree --manifest-path crates/sc-cron-commands/tests/fixtures/minimal/Cargo.toml --edges normal
cargo tree --manifest-path crates/sc-cron-commands/tests/fixtures/minimal/Cargo.toml --edges normal --features schema
```

Both graphs exclude cron, rusqlite/SQLite, Tokio, Monty, Axum, RMCP, clap,
sc-runtime, and sc-command/server. Explicit scheduler selects the core, and
Monty remains independently opt-in.
