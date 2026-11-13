# sc-cron Requirements

**Status:** Active

Active as of 2026-10-28. The contract is [architecture.md](architecture.md), [ADR-RUN-0013](../architecture.md), and [ADR-RUN-0016](../architecture.md). Sprints `b-3` through `b-17` implement these items.

# Requirement: Workspace member

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRON-0001: Workspace member

**Status:** Active  

### Requirement Statement

Definitions: sc-monty is the shared Monty interpreter implementation. sc-storage is the shared `Storage<T>` crate. The common sc-command read/write helpers are the asynchronous `storage_read` and `storage_write` functions over `Storage<T>` named in ADR-RUN-0021. Sprint d-9 is the Phase D sc-cron boundary sprint (`run-d-9`) covering scheduler migration to application-supplied storage and thin sc-cron-commands API/bound adaptations. The Phase D public package set is the nine crates named in REQ-RUN-0002, released together at version 0.3.0. A Monty value is an interpreter value passed to a hook function through sc-monty's value-level invocation, as opposed to a JSON-encoded string. The port preserves the existing test assertions while moving the interpreter implementation into sc-monty. Weakening an assertion means removing a checked condition, widening an expected value, range or tolerance, or replacing an exact comparison with a looser one. The local facade is the former unpublished observability-types shim. A workspace library crate is a library crate that is a member of this Cargo workspace. Naming a crate here means a manifest dependency; comments and schema descriptions may reference documented wiring. Registry types are the canonical event types imported from the published sc-observability-types crate. The command contract edge is the normal dependency on sc-command with `default-features = false` and no features, permitted by ADR-RUN-0017. Test doubles are substitute components used only by tests. Capability-proven upstream cleanup is the removal of the command contract edge after the upstream canonical logging capability required by ADR-RUN-0017 has been verified. Server-feature leakage is activation of server implementation features through the common-helper dependency edges.

`crates/sc-cron` is a workspace member, ported from beads-cache at `develop`, and is one of the Phase D public package set. Phase D sc-cron consumes the single sc-monty implementation for interpreter behavior and sc-storage for shared storage behavior; it no longer directly depends on monty or monty-types. It retains cron policy and JSON-to/from-value adaptation; Monty hooks receive Monty values rather than JSON strings. sc-sha is permitted for SHA-256 cache keys. No independent Monty compile/session/host/limits/dump/load engine remains here. Existing unit and integration assertions are ported without deletion or weakening. sc-cron-commands consumes the common sc-command read/write helpers after sprint d-9 closes, without server-feature leakage.

The command contract edge supplies the retained observability port. Canonical event values remain direct registry `sc-observability-types` imports. Registry types use the open floor `>=1.5.0` with no upper bound, as every published registry dependency does under REQ-RUN-0002. The historical `0.1.1` release used an exact `=1.5.0` command pin; that pin does not govern Phase D.

`sc-runtime` does not name sc-cron. The generated daemon may name it only as the optional `cron` dependency selected by the embedding application; default generated builds do not link it. Here "name" means a Cargo or other manifest dependency. Documented wiring may reference the crate in comments and schema descriptions.

sc-cron's command contract edge enables neither the sc-command `server` feature nor the sc-command `test-util` feature; test doubles use dev-dependencies only. The local facade is gone and does not return. Capability-proven upstream cleanup removes the command contract edge, after which sc-storage and sc-monty are the only workspace library crates sc-cron depends on.

**MUST statements:**

- `crates/sc-cron` MUST be a member of this Cargo workspace.
- `crates/sc-cron` MUST be ported from beads-cache.
- sc-cron MUST be a member of the Phase D public package set.
- Phase D sc-cron MUST consume the single sc-monty implementation.
- Phase D sc-cron MUST consume sc-storage for shared storage behavior.
- sc-cron MUST retain cron policy.
- sc-cron MUST retain JSON-to/from-value adaptation.
- Hook data passed by sc-cron to Monty hooks MUST be Monty values.
- The port from beads-cache MUST preserve the existing unit-test assertions in the resulting test suites.
- The port from beads-cache MUST preserve the existing integration-test assertions in the resulting test suites.
- After sprint d-9 closes, sc-cron-commands MUST consume the common sc-command read/write helpers.
- sc-cron's only normal dependency on sc-command MUST be the command contract edge.
- The command contract edge MUST set `default-features = false`.
- The command contract edge MUST enable no features.
- Canonical event values MUST remain direct registry sc-observability-types imports.
- sc-cron's sc-observability-types requirement MUST be the registry floor `>=1.5.0`.
- Test doubles MUST use dev-dependencies only.
- Capability-proven upstream cleanup MUST remove the command contract edge.

- If the generated daemon declares an `sc-cron` manifest dependency, that dependency MUST be optional `cron` selected by the embedding application.

**MUST NOT statements:**

- sc-cron MUST NOT pass hook data to its Monty hooks as JSON strings.

- sc-cron MUST NOT depend on any workspace library crate other than sc-command, sc-storage and sc-monty.
- After capability-proven upstream cleanup, sc-cron MUST NOT depend on any workspace library crate other than sc-storage and sc-monty.
- sc-cron's sc-observability-types requirement MUST NOT declare an upper bound.
- sc-cron MUST NOT depend on the local facade.
- The resulting integration test suites MUST NOT omit any existing beads-cache integration-test assertion.
- The port MUST NOT weaken any existing integration-test assertion, using this record's definition of weakening.
- The resulting unit test suites MUST NOT omit any existing beads-cache unit-test assertion.
- The port MUST NOT weaken any existing unit-test assertion, using this record's definition of weakening.
- sc-cron MUST NOT directly depend on monty.
- sc-cron MUST NOT directly depend on monty-types.
- sc-cron MUST NOT retain an independent Monty compile/session/host/limits/dump/load engine.
- sc-cron-commands MUST NOT introduce server-feature leakage.
- The `sc-runtime` crate MUST NOT declare `sc-cron` in any dependency manifest.
- Default generated builds MUST NOT link sc-cron.
- The command contract edge MUST NOT enable the sc-command `server` feature.
- The command contract edge MUST NOT enable the sc-command `test-util` feature.

### Rationale

An independently packaged scheduler can be embedded without the generated template. Its sc-command dependency supplies the retained observability port, while sc-storage and sc-monty supply shared storage and interpreter behavior.

### Success Criteria

**Acceptance Criteria:**

- [ ] Isolated default/Monty consumers must prove no axum/rmcp/server leakage; workspace feature unification is not that proof.

## REQ-CRON-0002: Normal crontab strings

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

A schedule uses a normal crontab string of five fields: minute, hour, day of month, month, and day of week.

**MUST statements:**

- A schedule MUST use a normal crontab string of five fields: minute, hour, day of month, month, and day of week.

### Rationale

The beads-cache expression parser refused a five-field string; a normal crontab string is five fields, minute first (ADR-RUN-0013 Context). Owner, 2026-10-25: schedules are cron strings.
### Success Criteria

**Acceptance Criteria:**

- [ ] `*/5 * * * *` means every fifth minute.

# Requirement: Rust, Python, or Monty hook

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRON-0003: Rust, Python, or Monty hook

**Status:** Active  

### Requirement Statement

Definitions: Monty is the Pydantic Python interpreter. sc-monty is the standalone Rust crate owning that interpreter implementation. bd-cache, also called beads-cache, is the consumer application providing a local SQLite read/write cache in front of the Beads bd CLI; its application-specific cache fields are outside the scheduler hook protocol. ATM means agent-team-mail. MCP means Model Context Protocol. A domain transaction is the existing scheduler lifecycle storage transaction, as distinct from a user command calling storage_read or storage_write.

All Monty execution calls sc-monty. Retain the existing cron host adapters, capability behavior and unit/integration assertions as thin value-level adapters, not another interpreter host. A schedule supplies Monty or Python source plus JSON data, which the embedding application can enrich in the shared cron.start transform before persistence. cron.start and cron.end use storage_write with policy-checked CronStorage actions; cron.list uses storage_read. Internal scheduler lifecycle operations retain domain transactions because they are not user commands.

sc-cron is a general-purpose scheduler for application-defined jobs. A schedule row names an in-process `Job`, a Python module, or `Monty { source }` under optional feature `monty`. Python and Monty receive the `sc-cron.hook.v1` object with `protocol`, `event`, `schedule`, `payload`, and `deadline`. It has no consumer-specific beads, bd-cache, or ATM field. The crate exposes no HTTP route, MCP tool, or binary. The consumer owns that API and configures schedule limits; sc-cron does not read sc-config.

Monty hosts are application-owned registrations in an immutable `MontyHostRegistry` installed with `Scheduler::with_monty_hosts`. The empty registry registers no application call. `MontyHostHandler: Send + Sync` accepts a `MontyHostCall` containing its validated `MontyHostName`, positional/keyword JSON arguments, schedule name, hook phase, and row deadline and returns `Result<serde_json::Value, CronError>`; matching closures are supported. `MontyHostName::new` validates the identifier and `as_str()` borrows it; registry keys use the same type while registration still accepts strings. Registration rejects invalid identifiers, duplicate names, and reserved `__hook`, `cron_stop`, and `cron_tick` bindings with `InvalidSpec`. Source can invoke registered functions but cannot install or replace handlers. An unregistered name fails with `HOST_UNDEFINED`. Bridge argument conversion and supplied CLI adapter shape rejection fail with `HOST_DENIED`; a custom handler's returned error, including `CronError::InvalidSpec`, records `HOST_CALL_FAILED` after configured-value redaction. Private typed dispatch preserves supplied-adapter errors without parsing custom error prose. The host checks the row deadline before and after a synchronous handler, but cannot preempt arbitrary application code.

Phase B supplies opt-in `sc_compose_run(subcommand, args)` and `atm_run(subcommand, args)` adapters through this same registry. `ScComposeHostConfig` and `AtmHostConfig` select non-empty executable paths (defaults `sc-compose` and `atm`) and working directories (default the daemon directory captured at configuration). Each adapter accepts one command/subcommand string and an optional list of string arguments, with no keyword arguments. It forwards the non-empty command/subcommand followed by those ordered arguments directly as argv, never through a shell. Every command/subcommand is supported; flags including `--root` are forwarded unchanged. Neither adapter forces render arguments or temp-root staging. Scripts choose arguments, not the configured executable or working directory. Both adapters share the existing blocking child driver in `monty_compose`, generalized for argv/cwd, with row deadline, process-group termination, and `MAX_HOOK_OUTPUT_BYTES` per stream. They do not call the async `process::supervise` from the blocking feed thread or introduce a runtime/`block_on` bridge. A completed child returns `{stdout, stderr, exit_code}`, including nonzero exits; script code decides how that result affects its hook. A deadline expiration records `TimedOut`; spawn, decoding, and capture-limit failures record stable host errors. The adapters receive the resolved environment snapshot from ADR-RUN-0016. Neither CLI is required when its adapter is unused. `render_template` is not a product requirement; an internal helper may remain an implementation convenience without becoming a mandatory registered host function.

MVP accepts application-supplied Monty source without signatures, a separate script-approval mechanism, or a consumer option enabling such a mechanism. Capability registration and environment policy are enforced independently. In-process Monty and custom Rust handlers retain stack-overflow, allocator-abort, panic, and non-cooperative deadline risks; they are not an isolation boundary for hostile code.

The stop/tick ordering, generation fencing, retained REPL, interpreter-checkpoint VM limits, and joined shutdown behavior remain as specified by ADR-RUN-0013. Host calls use the production `drive_feed` resume loop; the cfg(test) helper is not a separate production path.

**MUST statements:**

- All Monty interpreter execution MUST call `sc-monty`.
- The existing cron host adapters MUST remain thin value-level adapters.
- Interpreter extraction MUST preserve the existing cron capability behavior.
- Interpreter extraction MUST preserve existing unit/integration assertions.
- A schedule MUST support an in-process `Job` target.
- A schedule MUST support a Python module target.
- With optional feature `monty`, a schedule MUST support a `Monty { source }` target.
- Python and Monty MUST receive the `sc-cron.hook.v1` object with the fields listed above.
- Application-owned Monty handlers MUST be registered through the immutable `MontyHostRegistry` installed by `Scheduler::with_monty_hosts`.
- The empty host registry MUST register no application call.
- Host registration MUST reject invalid names with `InvalidSpec`, using the name rules above.
- Host registration MUST reject duplicate names with `InvalidSpec`, using the name rules above.
- Host registration MUST reject reserved names with `InvalidSpec`, using the name rules above.
- An unregistered host name MUST fail with `HOST_UNDEFINED`.
- Rejected bridge argument conversion MUST fail with `HOST_DENIED`.
- Rejected supplied CLI adapter shape MUST fail with `HOST_DENIED`.
- A custom handler error MUST record `HOST_CALL_FAILED`.
- Configured values in custom-handler error details MUST be redacted before those details are recorded.
- Host dispatch MUST check the row deadline before each synchronous handler.
- Host dispatch MUST check the row deadline after each synchronous handler.
- The opt-in `sc_compose_run` adapter MUST forward the command/subcommand and ordered string arguments directly as argv.
- The opt-in `atm_run` adapter MUST forward the command/subcommand and ordered string arguments directly as argv.
- CLI adapters MUST use the existing blocking child driver.
- The child driver MUST enforce the row deadline.
- The child driver MUST use process-group termination.
- The child driver MUST enforce MAX_HOOK_OUTPUT_BYTES per stream.
- A completed child MUST return the `{stdout, stderr, exit_code}` result, including nonzero exits.
- A child deadline expiry MUST record `TimedOut`.
- Both CLI adapters MUST receive the resolved environment snapshot from ADR-RUN-0016.
- User cron.start MUST use `storage_write`.
- The storage action used by user cron.start MUST enforce `CronStorage` policy.
- User cron.end MUST use `storage_write`.
- The storage action used by user cron.end MUST enforce `CronStorage` policy.
- User cron.list MUST use `storage_read`.
- cron.start MUST permit the application transform to enrich schedule JSON before persistence.
- Internal scheduler lifecycle operations MUST retain domain transactions.
- Monty host calls MUST use the production `drive_feed` resume loop.

**MUST NOT statements:**

- The opt-in CLI adapters MUST NOT execute those arguments through a shell.
- `sc-cron` MUST NOT implement a second Monty interpreter host.
- `sc-cron` MUST NOT introduce consumer-specific beads fields.
- `sc-cron` MUST NOT introduce consumer-specific bd-cache fields.
- `sc-cron` MUST NOT introduce consumer-specific ATM fields.
- `sc-cron` MUST NOT expose an HTTP route.
- `sc-cron` MUST NOT expose an MCP tool.
- `sc-cron` MUST NOT expose a binary.
- `sc-cron` MUST NOT read sc-config.
- Scripts MUST NOT select the configured executable.
- Scripts MUST NOT select the configured working directory.
- CLI adapters MUST NOT introduce an async-runtime or block_on bridge on the blocking feed thread.
- Unused CLI adapters MUST NOT require their executable to be installed.

### Rationale

Application-supplied hooks and value adapters let the reusable scheduler execute registered application behavior without implementing another interpreter host.

### Success Criteria

**Acceptance Criteria:**

- [ ] Host registry integration tests invoke custom, sc-compose, and ATM functions through one extension interface.
- [ ] Argument-forwarding tests cover render and non-render subcommands.
- [ ] MVP execution requires neither signatures nor approval configuration.

# Requirement: Application schedule storage and compiled-session cache

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRON-0004: Application schedule storage and compiled-session cache

**Status:** Active  
**Last Updated:** 2026-11-12

### Requirement Statement

Definitions: CronRow is the schedule-row type owned by sc-cron. Storage<T> is the shared storage trait for row type T. hook_environment is the canonical schedule environment containing dotenv sources and variable operations. ScheduleLimits is the embedding application's schedule-limit configuration. sc-monty is the shared Monty interpreter implementation. Monty dump version identifies the compiled-session serialization version. SHA-256 is the source digest supplied by sc-sha. Cache rows are the sc-cron-owned MontyBlobRow rows; a consumer table is any application table outside these sc-cron schemas. CronStorage is sc-cron's policy-checked schedule-storage facade over Storage<CronRow> (ADR-RUN-0021). The schedule-row limit is the row limit in ScheduleLimits. A schedule writer is any sc-cron component that writes a CronRow. A compiled session is the result of sc-monty compiling a schedule row's source; its blob is that result serialized in a MontyBlobRow. A cache entry is one MontyBlobRow. Generic row schemas are the schemas of rows stored through Storage<T>. The configuration schema here is sc-cron's schedule configuration schema.

sc-cron owns CronRow for schedules and MontyBlobRow for compiled hook sessions. Generic storage engines contain no cron-specific schema or SQL.

**MUST statements:**

- CronRow MUST be stored through Storage<CronRow>.
- The CronRow table MUST live in one of the application's databases.
- By default the CronRow table MUST live in the application's SQLite database.
- CronRow MUST store the schedule's canonical hook_environment unchanged.
- The default hook_environment MUST be the empty value {"dotenv":[],"variables":{}}.
- The configured schedule-row limit MUST count enabled and disabled rows.
- When the number of schedule rows has reached the schedule-row limit, CronStorage MUST reject a new schedule row with CronError::InvalidSpec.
- Every schedule writer MUST apply the embedding application's ScheduleLimits.
- A schedule write whose source sc-monty cannot compile MUST be rejected at the write, before that schedule's first tick.
- The `MontyBlobRow` written for a compiled session MUST contain the session as compiled.
- Compiled-session blobs MUST use sc-sha SHA-256 of the exact source plus Monty dump version as their key.
- The schedule-row source MUST remain authoritative for matching a cached blob.
- A cache entry that is missing, stale, corrupt, of an unreadable version or missing a hook MUST trigger recompilation from the schedule-row source.
- Recompilation MUST write a new `MontyBlobRow`.
- After that recompilation, the new `MontyBlobRow` MUST replace the cache entry for subsequent use.
- Cache rows MUST use sc-cron-owned generic row schemas.
- Cache rows MUST use sc-cron-owned migrations.

**MUST NOT statements:**

- A `MontyBlobRow` MUST NOT be modified after it is written.

- sc-cron MUST NOT create a separate cron database.
- sc-cron MUST NOT persist hook run state in compiled-session blobs.
- The configuration schema MUST NOT add a separate environment table.
- The configuration schema MUST NOT add a separate secret table.
- sc-cron MUST NOT migrate unrelated consumer tables.

### Rationale

configuration is one SQLite table. Reloading schedules loads every row into memory; an uncapped row count is unsafe when schedules come from operators or configuration, and sc-cron must not depend on `sc-config` (ADR-RUN-0014).

### Success Criteria

The ported cron conformance tests preserve schedule fields and row-limit behavior. Compiled-cache tests cover source/version match, invalid source at submit, pre-run snapshot timing and each cache-miss condition.

### Implementation Notes

The row limit is application configuration rather than stored table state; independent writers with different configured limits do not enforce one another's limits.

## REQ-CRON-0005: One error enum

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: A fallible sc-cron function is a `pub fn` exported by the `sc_cron` crate whose return type is a `Result`; trait methods that the application implements, such as `EnvironmentReferenceResolver::resolve`, are excluded. `Captured` is the child-output type in crates/sc-cron/src/process.rs. The CronError table is the variant-to-code table below. StoreErrorKind is the `#[non_exhaustive]` discriminator inside `CronError::Store`, defined in crates/sc-cron/src/error.rs. A cooperative operation is a schedule-store operation run under a `StoreOperationContext` (REQ-CRON-0020). The matching non-cooperative operation is the same schedule-store method called without a `StoreOperationContext`. Call arguments are the values passed to the store method. A cooperative operation failure is a `CronError` returned by a cooperative operation. Schedule-row contents are the stored field values of one schedule row. Provider prose is text produced by an application callback or a storage provider. A Monty feed is one run of a schedule row's Monty hook with feature `monty`. The condition table is the condition-to-code table below. A host adapter is an sc-cron component that answers a Monty host call, such as the CLI adapters `sc_compose_run` and `atm_run`. A host-deadline expiration is a Monty feed that exceeds its row deadline, as in the last two rows of the condition table. A code token is a code listed in the CronError table or in the condition table. To automatically retry is for sc-cron to run a failed operation again without a new call from the application. SQL text is the text of an SQL statement. In a `Failed` detail of the form `CODE: message; recovery`, message is the failure text and recovery is the row's operator recovery text. A `TransactionOutcome` is the persistence report returned by `StoreError::transaction_outcome()`.

`DeadlineExceeded` may carry confirmed committed-late evidence. `StoreError::transaction_outcome()` is optional. REQ-CRON-0020 and ADR-RUN-0018 govern the admission and persistence report policy. Application callbacks keep their own result types: `EnvironmentReferenceResolver::resolve` fails with the value-free `EnvironmentReferenceError::{Denied, Unavailable}`. `RunOutcome::failure_parts` in crates/sc-cron/src/run_failure.rs parses a `Failed` detail. Reading crates/sc-cron checks statements about types, signatures and enum variants. Unit tests in crates/sc-cron check the other statements by calling the named function or operation and reading the returned error, the recorded `RunOutcome` or the stored schedule row.

| CronError variant | Code |
|---|---|
| `BadExpression` | `BAD_EXPRESSION` |
| `InvalidSpec` | `INVALID_SPEC` |
| `AlreadyActive` | `ALREADY_ACTIVE` |
| `NotFound` | `NOT_FOUND` |
| `HookFailed` | `HOOK_FAILED` |
| `Store` | `STORE_FAILED` |
| `Fenced` | `FENCED` |

The condition table matches [architecture.md](architecture.md):

| Condition | Code | `RunOutcome` | Operator recovery |
|---|---|---|---|
| Bridge argument conversion or supplied CLI adapter shape rejected before execution | `HOST_DENIED` | `Failed` | Fix the call's JSON conversion or the supplied adapter's documented argument shape. |
| Custom application handler returns an error, including `InvalidSpec` | `HOST_CALL_FAILED` | `Failed` | Fix the application handler; configured environment values are redacted from the recorded detail. |
| CLI executable missing or not runnable | `HOST_SPAWN` | `Failed` | Fix the configured executable path or install the external CLI. |
| CLI output is invalid UTF-8 or exceeds the per-stream cap | `HOST_OUTPUT` | `Failed` | Return bounded UTF-8 stdout and stderr. |
| CLI exits nonzero | — | Script receives result | Inspect `exit_code` and bounded `stderr`; the adapter does not turn an application CLI result into an automatic hook failure. |
| Undefined name | `HOST_UNDEFINED` | `Failed` | Fix hook source; that name is not defined. |
| Monty `OsCall` other than `Getenv` or `GetEnviron` | `HOST_OS` | `Failed` — detail names the operation | Use the configured host registry and Getenv/GetEnviron; other OS operations remain denied. |
| Allowlisted daemon variable is not Unicode or exceeds the value cap | `ENV_PROCESS_INVALID` | `Failed` — value-free detail | Restart the daemon with a bounded UTF-8 value or remove the name from the allowlist. |
| `.env` path outside daemon policy | `ENV_FILE_DENIED` | `Failed` — value-free detail | Authorize the canonical file/root or remove the schedule source. |
| Required `.env` file missing | `ENV_FILE_MISSING` | `Failed` — value-free detail | Restore the absolute file or mark the source optional. |
| `.env` file invalid | `ENV_FILE_INVALID` | `Failed` — value-free detail | Fix the bounded data-file syntax. Shell expansion is not supported. |
| Reference missing or resolver failed | `ENV_REFERENCE` | `Failed` — value-free detail | Configure the daemon resolver and restore the referenced value. |
| Resolved entry-count or encoded-block limit exceeded | `ENV_SNAPSHOT_LIMIT` | `Failed` — value-free detail | Remove variables or reduce values before retrying. |
| Monty raises during the feed | `HOST_RAISED` | `Failed` | The detail is the exception message. |
| `cron_stop` return value is not a valid stop reply | `HOST_REPLY` | `Failed` — detail is `stop check: HOST_REPLY:` then the parse error | Fix hook source; return an object with string `"action"` (`continue`, `disable`, or `delete`) and a non-empty `"reason"` when disabling or deleting. |
| Monty feed task join failed (cancelled blocking-pool task) | `FEED_TASK_FAILED` | `Failed` | Retry the run; if it persists, inspect blocking-pool capacity and hook source. |
| Monty feed task panicked (worker join) | `FEED_JOIN_FAILED` | `Failed` | Fix the panic in the hook or host and retry the schedule. |
| Monty due task panicked or was cancelled | `DUE_JOIN_FAILED` | `Failed` | Retry the schedule; if it persists, inspect the Monty due task and hook source. |
| Monty due path used for a non-Monty hook | `MONTY_RESOLVE` | `Failed` | Run this row on its Rust or Python target, or set `HookTarget::Monty`. |
| `cron_tick` returned a stop action (disable/delete) | `MONTY_TICK` | `Failed` | Return normally from `cron_tick`; disable and delete belong in `cron_stop`. |
| After the CLI returns, host `Instant` exceeds the row budget |  | `TimedOut`, session dropped | Reduce CLI work or raise the row deadline; the VM clock was paused during the host call. |
| Monty feed-time-limit while the VM is executing (not during the host suspend) |  | `TimedOut` | Interpreter checkpoints govern VM execution; host handlers use the row deadline. |

**MUST statements:**

- Every fallible sc-cron function MUST declare `CronError` as the error type of its `Result`.
- The `CronError` enum in crates/sc-cron/src/error.rs MUST define exactly the variants in the CronError table.
- `CronError::code()` MUST map each variant to its code in the CronError table.
- `process::supervise` in crates/sc-cron/src/process.rs MUST return `Result<Captured, CronError>`.
- `process::supervise_with_logger` in crates/sc-cron/src/process.rs MUST return `Result<Captured, CronError>`.
- Every failure of a schedule-store method called with a `StoreOperationContext` MUST be a `CronError::Store` value.
- StoreErrorKind MUST carry the `#[non_exhaustive]` attribute.
- StoreErrorKind MUST include the variant `Cancelled`.
- StoreErrorKind MUST include the variant `DeadlineExceeded`.
- StoreErrorKind MUST include the variant `Indeterminate`.
- For equal call arguments and equal stored schedule rows before the call, a schedule-store method called with a `StoreOperationContext` MUST leave the same schedule-row contents as the same schedule-store method called without a `StoreOperationContext`.
- When a schedule's environment reference is missing, the Monty feed's recorded detail MUST start with `ENV_REFERENCE:`.
- When `EnvironmentReferenceResolver::resolve` returns an error, the Monty feed's recorded detail MUST start with `ENV_REFERENCE:`.
- For each row of the condition table, a Monty feed that meets the row's condition MUST record the row's `RunOutcome`.
- For each row of the condition table that lists a code, the recorded `RunOutcome::Failed` detail MUST have the form `CODE: message; recovery` with the row's code as `CODE`.
- For a host-deadline expiration, the Monty feed MUST record `RunOutcome::TimedOut`.

**MUST NOT statements:**

- The `RunOutcome::TimedOut` stored for a host-deadline expiration MUST NOT contain a code token.
- The `to_string()` output of a cooperative operation failure MUST NOT contain SQL text.
- The `to_string()` output of a cooperative operation failure MUST NOT contain provider prose.
- sc-cron MUST NOT automatically retry an operation that failed with `StoreErrorKind::DeadlineExceeded`.
- A `StoreError` of kind `Indeterminate` MUST NOT carry a `TransactionOutcome` whose persistence is `RolledBack`.
- A `StoreError` of kind `Indeterminate` MUST NOT carry a `TransactionOutcome` whose persistence is `NoMutationAttempted`.
- The `to_string()` output of a `StoreError` of kind `Indeterminate` MUST NOT state that its transaction was not persisted.
- A public error type in crates/sc-cron MUST NOT carry provider prose as its error value.
- The `CronError` enum in crates/sc-cron/src/error.rs MUST NOT contain a variant for host-adapter errors.

### Rationale

Every cross-boundary failure exposes stable codes, causes, and recovery/remediation; agents must be able to act on failures without reading source (framework charter). The variant set and codes match the beads-cache crate that sc-cron ports.
### Success Criteria

**Acceptance Criteria:**

- [ ] The public error type in crates/sc-cron is CronError.

## REQ-CRON-0019: Explicit Python and Monty hook environment

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-28  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: Canonical plaintext schedule data is the serialized HookEnvironment configuration, including deliberate Set values, rather than secret values inherited or resolved at execution. Provider prose is text from the application reference resolver; only Denied, Unavailable, missing or a value crosses that resolver protocol. Resolved values are values supplied from daemon variables, dotenv files or references. Output assembly means composing fixed codes/recovery, outcomes, events, traces and persisted reasons. Stable text and metadata mean the fixed codes, recovery messages and non-provider metadata; they and the canonical configuration write are excluded from value replacement. The resolved map is the immutable map of environment names to values resolved for a schedule run. Schedule operations are the name-keyed `Set`, `Reference` or `Clear` operations of the schedule environment. The row deadline is the deadline of the schedule row being run. The CLI adapter children are the child processes started by sc_compose_run and atm_run.

Python and Monty schedule rows share one `HookEnvironment`; existing
constructors default it to empty. A daemon-scoped `HookEnvironmentHost` names
which daemon-process variables may be inherited, authorizes absolute `.env`
files or canonical roots, and supplies an optional reference resolver. The
generated administrator config exposes empty-by-default `allow_process`,
`allow_dotenv_files`, and `allow_dotenv_roots` arrays. Every
host default is empty. The crate never infers or captures an invoking caller's
environment. The generated daemon passes a prebuilt host policy into scheduler
composition so an application-provided resolver is preserved.

Reference resolution receives the schedule name, variable name, opaque
reference, and row deadline. The provider returns only `Denied`, `Unavailable`,
missing, or a value; sc-cron emits no provider prose. Resolution is bounded by
the row deadline, and expiration is `RunOutcome::TimedOut` before hook source
runs.

The schedule environment contains ordered absolute `.env` sources followed
by name-keyed `Set`, `Reference`, or `Clear` operations. One immutable map is
resolved before `cron.stop` and reused for `cron.tick`: allowlisted daemon
values, then files in declaration order, then schedule operations. Explicit
set values are canonical plaintext schedule data. Daemon, file, and reference
values are never persisted. A non-empty environment on a Rust row is invalid;
trusted Rust jobs keep their existing `Job` interface.

Reference locators are non-secret plaintext registry metadata; only their
resolved values remain external. Every intermediate/final map has at most 128
entries, every stored/resolved value is at most 16 KiB, and the encoded final
block is at most 32,000 UTF-8 bytes. Per-source oversize failures and aggregate
`ENV_SNAPSHOT_LIMIT` failures are value-free.

Python starts with exactly the resolved map rather than the daemon's complete
environment. Monty source initialization and hook feeds answer `Getenv` and
`GetEnviron` from the same map with ordinary absent/default behavior; every
other OS call remains denied. Both sc_compose_run and atm_run children receive that map
and do not inherit the daemon environment. sc-cron never mutates the daemon
process environment. Full semantics, limits, file authorization, and the
release-v1 schema baseline are [ADR-RUN-0016](../architecture.md).

Every hook/provider-controlled failure detail or stop reason has each distinct
non-empty resolved value replaced with `[REDACTED_ENV]` before byte capping and
before fixed codes/recovery, outcomes, events, traces, or persisted reasons are
assembled. Stable text, metadata, and the canonical environment configuration
write are exempt; deliberate set values remain plaintext schedule data. Python
output, Monty exception/return text, and both CLI adapters' stdout/stderr are included. Trusted
direct callers may read raw public `HookOutput` fields, but `HookOutput` debug
formatting omits their contents. This exact-value replacement mitigates
accidental propagation; trusted hooks can transform or exfiltrate values and
remain outside an untrusted-code boundary.

**MUST statements:**

- Python and Monty schedule rows MUST share one `HookEnvironment`.
- Existing constructors MUST default the hook environment to empty.
- Every default of `HookEnvironmentHost` MUST be empty.
- Generated administrator configuration MUST expose an empty-by-default `allow_process` array.
- Generated administrator configuration MUST expose an empty-by-default `allow_dotenv_files` array.
- Generated administrator configuration MUST expose an empty-by-default `allow_dotenv_roots` array.
- Generated scheduler composition MUST preserve an application-provided resolver in its prebuilt host policy.
- Reference resolution MUST receive the schedule name.
- Reference resolution MUST receive the variable name.
- Reference resolution MUST receive the opaque reference.
- Reference resolution MUST receive the row deadline.
- Reference resolution MUST be bounded by the row deadline.
- If the row deadline expires during reference resolution, the run's outcome MUST be `RunOutcome::TimedOut`.
- One immutable map MUST be resolved before cron.stop.
- cron.tick MUST reuse the immutable map resolved for cron.stop.
- Resolution MUST first apply allowlisted daemon values.
- After allowlisted daemon values, resolution MUST apply files in declaration order.
- After the files, resolution MUST apply schedule operations.
- Explicit Set values MUST be canonical plaintext schedule data.
- A non-empty hook environment on a Rust row MUST be rejected.
- Each intermediate and final map MUST contain at most 128 entries.
- Each stored or resolved value MUST be at most 16 KiB.
- The encoded final environment block MUST be at most 32,000 UTF-8 bytes.
- Per-source oversize failures MUST be value-free.
- Aggregate-limit failures MUST be value-free.
- Python MUST start with exactly the resolved map.
- Monty Getenv MUST return the resolved-map value for a present key.
- Monty Getenv MUST return the call's supplied default when the key is missing or cleared.
- Monty GetEnviron MUST enumerate that same resolved map, excluding missing or cleared names.
- Every Monty OS call other than `Getenv` and `GetEnviron` MUST be denied.
- Both CLI adapter children MUST receive the resolved map.
- Hook/provider-controlled failure details and stop reasons MUST replace distinct non-empty resolved values with `[REDACTED_ENV]` before byte caps and the output assembly described above.
- Stable text, metadata and canonical environment configuration writes MUST remain exempt from resolved-value replacement.
- The `Debug` formatting of `HookOutput` MUST omit the contents of its raw output fields.

**MUST NOT statements:**

- If the row deadline expires during reference resolution, hook source MUST NOT run.

- CLI adapter children MUST NOT inherit the daemon environment.
- `sc-cron` MUST NOT infer or capture the environment of an invoking caller.
- `sc-cron` MUST NOT emit provider prose.
- Resolved daemon values MUST NOT be persisted.
- Resolved file values MUST NOT be persisted.
- Resolved reference values MUST NOT be persisted.
- `sc-cron` MUST NOT mutate the daemon process environment.

### Rationale

Owner (ANC-034): "python/monty having consistent settings would be least confusing" (ANC-037): "Never capture or persist inherited host values accidentally" ADR-RUN-0016: environment variables are not uniformly secrets. Durable values such as `ATM_IDENTITY` and `ATM_TEAM` belong with a schedule. API keys may instead be daemon-wide inherited values, explicit schedule values, references resolved by an application secret provider, or values loaded from an authorized absolute `.env` file.
### Success Criteria

**Acceptance Criteria:**

- [ ] The same environment cases produce equivalent Python and Monty results.
- [ ] An allowed daemon value is delivered and an unlisted value is absent.
- [ ] Two schedules set distinct identities and clearing removes a prior value.
- [ ] Ambient canaries are absent from persisted rows without explicit set input.
- [ ] A consumer resolver supplies a referenced value without persisting the result.
- [ ] An authorized absolute dotenv file supplies hook values.
- [ ] A top-level getenv reads the configured snapshot before cron_stop or cron_tick.

## NFR-CRON-0001: Allowed third-party dependencies

**Version:** 1.0.0  
**Created:** 2026-10-27  
**Last Updated:** 2026-11-12  
**Owner:** randlee  
**Status:** Active  

### Requirement Statement

Definitions: ATM means agent-team-mail and its tool crates. sc-compose crates are sc-compose, sc-composer and sc-composer-beads; sc-sha is published from the sc-compose repository but is not an sc-compose crate (ANC-061). The command contract edge is the dependency on sc-command with `default-features = false` and no features, permitted by ADR-RUN-0017. sc-storage is the shared `Storage<T>` crate and sc-monty is the shared Monty interpreter crate; both are workspace crates that REQ-RUN-0002 requires sc-cron to consume. The interpreter crates are sc-monty, monty, monty-types, monty-pool, monty-fs and monty-runtime. The default tree is the output of `cargo tree -p sc-cron -e normal`; the monty-feature tree is the same command with `--features monty`. sc-lint-boundary accepts workspace names only, so monty/monty-types are enforced by cargo-tree gates rather than entries in that list. Test and benchmark references are permitted; CLI process invocation creates no crate dependency. The `allowed_dependencies` list is in `boundaries/sc-cron/sc-cron.toml`. Production dependency checks cover normal and build dependencies across production features; production Rust imports are imports in non-test and non-benchmark code in `crates/sc-cron`. Permitted interpreter crates in the monty-feature tree are sc-monty, monty and monty-types.

`rusqlite` leaves this list when the schedule store moves to `Storage<CronRow>`.

This item owns the third-party crates `sc-cron` may depend on. The `allowed_dependencies` list is the normal third-party dependencies: `async-trait`, `chrono`, `cron`, `getrandom`, `process-wrap`, `rusqlite` (bundled), `sc-observability-types`, `sc-sha`, `serde`, `serde_json`, `tempfile`, `tokio`, and `tracing`. `sc-sha` supplies the SHA-256 compiled-session cache key. `sc-lint-boundary` accepts only workspace package names, so `monty` and `monty-types` are not entries in that list. Feature `monty` stays optional and adds the interpreter only through sc-monty. It is enforced by the `sc-cron-tree-default` and `sc-cron-tree-monty` cargo-tree gates in `sc-lint.toml`: the default tree has no interpreter crate, and the only interpreter crates in the monty-feature tree are `sc-monty`, `monty` and `monty-types`, with `monty` and `monty-types` reached through `sc-monty`. `monty-pool`, `monty-fs`, and `monty-runtime` are not allowed. The workspace crates sc-cron may depend on are the command contract edge, sc-storage and sc-monty; no other workspace crate is allowed.

The tool-crate exclusion in REQ-RUN-0005 applies to sc-cron: sc-compose and ATM crates are forbidden in production dependencies/imports; test and benchmark references are permitted. CLI process invocation does not create a crate dependency.

**MUST statements:**

- Until the schedule store moves to Storage<CronRow>, the normal allowed_dependencies list in `boundaries/sc-cron/sc-cron.toml` MUST be async-trait, chrono, cron, getrandom, process-wrap, rusqlite (bundled), sc-observability-types, sc-sha, serde, serde_json, tempfile, tokio and tracing.
- When the schedule store moves to Storage<CronRow>, rusqlite MUST leave the normal allowed_dependencies list in `boundaries/sc-cron/sc-cron.toml`.
- Feature monty MUST remain optional.
- The default tree MUST contain no interpreter crate.
- Every interpreter crate in the monty-feature tree MUST be a permitted interpreter crate.
- In the monty-feature tree, every path to monty MUST pass through sc-monty.
- In the monty-feature tree, every path to monty-types MUST pass through sc-monty.
- sc-cron-tree-default in sc-lint.toml MUST enforce the default tree.
- sc-cron-tree-monty in sc-lint.toml MUST enforce the monty-feature tree.

**MUST NOT statements:**

- sc-cron MUST NOT depend on any workspace crate outside the permitted workspace edges: sc-command through the command contract edge, sc-storage and sc-monty.
- sc-cron MUST NOT depend on monty-pool.
- sc-cron MUST NOT depend on monty-fs.
- sc-cron MUST NOT depend on monty-runtime.
- Production dependencies MUST NOT include sc-compose crates.
- Production dependencies MUST NOT include ATM crates.
- Production Rust imports MUST NOT include sc-compose crates.
- Production Rust imports in `crates/sc-cron` MUST NOT include ATM crates.

### Rationale

Owner (ANC-031): "sc-compose and atm crates should be blocked from any production release crate.  sc-compose and atm can be referenced for tests i.e. benchmark tests" `monty` runs in-process when the host controls the code; `monty-pool` runs the interpreter in worker subprocesses for untrusted code. The Monty hook target is the trusted case, and a subprocess pool would spend the process spawn the target exists to avoid. A consumer who does not enable `monty` does not link the interpreter.
### Success Criteria

**Acceptance Criteria:**

- [ ] The `sc-cron-tree-default` and `sc-cron-tree-monty` cargo-tree gates in `sc-lint.toml` enforce that feature `monty` stays optional: the default tree has no interpreter crate, and the only interpreter crates in the `--features monty` tree are `sc-monty`, `monty` and `monty-types`, with `monty` and `monty-types` reached through `sc-monty`.
### Related Documents

**Requirements:**

- [REQ-RUN-0005](../requirements.md) — point 4 points here

## NFR-CRON-0002: Environment isolation and redaction

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-28  
**Last Updated:** 2026-10-28  
**Owner:** randlee  

### Requirement Statement

Definitions: Allowlisted daemon values are process variables whose names are permitted by HookEnvironmentHost.allow_process. A non-Unicode value cannot be decoded as Unicode text. Diagnostic surfaces are configuration Debug, scheduled errors, recovery text, RunOutcome, tracing, canonical events and persisted stop reasons; trusted direct HookOutput fields remain the named exception. Stable metadata is fixed non-provider metadata and error/recovery text, rather than values copied from environment sources. A store connection is a database connection held for a storage operation.

Environment names use `[A-Za-z_][A-Za-z0-9_]*`. Construction, persistence,
row decode, and execution enforce 256 bytes per name, 16 `.env` sources, 128 entries, 16 KiB per stored/resolved
value, 4 KiB per reference locator/path, a 32,000-byte encoded final block, a
256 KiB canonical document, and a 1 MiB dotenv file. Names,
paths, values, and references are UTF-8 and NUL-free; ASCII case-fold name
collisions are rejected on every platform. An allowlisted non-Unicode daemon
value fails as `ENV_PROCESS_INVALID`. `.env` files are bounded UTF-8 data
files: no interpolation, command substitution, duplicate names, or NUL.
Canonical path authorization follows symlinks before checking an exact
file/root policy; authorization precedes missing classification. Lookup and
enumeration preserve spelling on Unix; Windows runtime keys are uppercased for
both Python and Monty and lookup is case-insensitive, while stored config keeps
the original spelling. Python, sc-compose, and ATM executables are resolved before
child `PATH` is cleared: basenames use daemon launch `PATH`/Windows `PATHEXT`,
absolute paths are retained, and relative paths with components use the
daemon current directory captured when hook/host configuration is built.

Set values, resolved values, file contents, and reference strings are never
copied from those sources into configuration `Debug`, scheduled errors,
recovery text, `RunOutcome`, tracing, canonical events, or persisted stop
reasons. Stable metadata that happens to equal a short value is not rewritten.
`ChildEnv`, `ChildSpec`, and `PythonHook` use value-free custom `Debug` just
like the environment model. Direct `HookOutput` field access is the sole
trusted raw-output exception. Resolution performs
file I/O off the async hot path, holds no store connection or synchronous lock
across await, and calls no `std::env::set_var` or `std::env::remove_var`.

The implemented registry builder uses `register(name, handler)`,
`with_sc_compose(ScComposeHostConfig)`, and `with_atm(AtmHostConfig)`, each
returning `Result<MontyHostRegistry, CronError>`. The generated development
example and custom closure setup are in [monty-generated-example.md](monty-generated-example.md).
The b-16 child environment is explicitly empty; the resolved-snapshot text
above is the b-17 successor contract.

Shared hook environment implementation (B17):

`sc-cron::HookEnvironment` is the validated, canonical schedule policy;
`HookEnvironmentHost` is separately injected daemon authorization. Names use
portable ASCII identifiers and reject ASCII case collisions. Administrator
process-name, exact dotenv-file, and dotenv-root allowlists default empty.
Absolute normalized file paths are authorized after canonicalization, including
nearest-existing-ancestor resolution for missing suffixes. Bounded regular-file
reads run on the blocking pool. The dotenv parser never evaluates shell or
reads ambient values. Application-owned `EnvironmentReferenceResolver` receives
schedule, variable, opaque locator, and row deadline; missing, denied, unavailable
and invalid provider results use fixed value-free `ENV_REFERENCE` diagnostics.

One occurrence resolves daemon → ordered files → Set/Reference/Clear into an
immutable snapshot before stop, then reuses it for tick, Monty initialization,
and both opt-in CLI adapters. Python starts with `ChildEnv::Only`; direct trusted
callers can explicitly choose Inherit. Monty permits only environment OS reads,
including top-level reads, and denies registered application host calls during
initialization. Retained module globals initialize once per generation;
function-time getenv uses the current occurrence. Executables resolve before
child PATH clearing, independently of configured child working directories.
Windows snapshot names are uppercase and getenv lookup ignores case. Rust jobs
retain their application process environment and reject nonempty schedule policy.

Explicit Set values and opaque reference metadata are intentional plaintext
configuration. Resolved process/file/reference values never persist in registry
writes. Runtime details and reasons redact distinct nonempty snapshot values,
longest first, before caps and fixed diagnostic/recovery assembly. Debug exposes
names/kinds/counts only; trusted raw HookOutput fields remain accessible.
Exact replacement mitigates accidental disclosure and does not isolate hostile
source. The complete first-release SQLite schema is version 1 with canonical
`hook_environment` default `{"dotenv":[],"variables":{}}`; semantic schema checks
reject incompatible unreleased development tables without migrating/overwriting.
The library remains general purpose, with open application host registrations,
no signing/approval gates, and no production dependency on sc-compose or ATM.

**MUST statements:**

- Environment names MUST match `[A-Za-z_][A-Za-z0-9_]*`.
- Construction MUST enforce the size limits listed above.
- Persistence MUST enforce the size limits listed above.
- Row decode MUST enforce the size limits listed above.
- Execution MUST enforce the size limits listed above.
- Environment names MUST be UTF-8.
- Environment names MUST be NUL-free.
- Environment paths MUST be UTF-8.
- Environment paths MUST be NUL-free.
- Environment values MUST be UTF-8.
- Environment values MUST be NUL-free.
- Environment references MUST be UTF-8.
- Environment references MUST be NUL-free.
- ASCII case-fold name collisions MUST be rejected on every platform.
- An allowlisted non-Unicode daemon value MUST fail as `ENV_PROCESS_INVALID`.
- Dotenv files MUST be bounded UTF-8 data.
- Canonical file authorization MUST follow symlinks before checking exact-file or root policy.
- Authorization MUST precede missing-file classification.
- Unix lookup and enumeration MUST preserve variable spelling.
- Windows runtime keys MUST be uppercased for Python.
- Windows runtime keys MUST be uppercased for Monty.
- Stored configuration MUST preserve variable spelling.
- Windows lookup MUST be case-insensitive.
- Child executable paths MUST be resolved before the child PATH is cleared, using the resolution rules above.
- ChildEnv MUST use value-free Debug formatting.
- ChildSpec MUST use value-free Debug formatting.
- PythonHook MUST use value-free Debug formatting.
- Resolution MUST perform file I/O off the async hot path.
- Missing, denied, unavailable and invalid reference-provider results MUST use value-free `ENV_REFERENCE` diagnostics.
- Stop MUST use the same resolved environment snapshot.
- Tick MUST use the same resolved environment snapshot.
- Monty initialization MUST use the same resolved environment snapshot.
- The sc_compose_run adapter MUST use the same resolved environment snapshot.
- The atm_run adapter MUST use the same resolved environment snapshot.
- The first-release schema version 1 MUST use canonical hook_environment default {"dotenv":[],"variables":{}}.
- Incompatible unreleased development tables MUST be rejected without migration or overwrite.

**MUST NOT statements:**

- Dotenv files MUST NOT contain interpolation.
- Dotenv files MUST NOT contain command substitution.
- Dotenv files MUST NOT contain duplicate names.
- Dotenv files MUST NOT contain NUL.
- Set values MUST NOT be copied into the diagnostic surfaces listed above, subject to the named trusted HookOutput exception.
- Resolved values MUST NOT be copied into the diagnostic surfaces listed above, subject to the named trusted HookOutput exception.
- File contents MUST NOT be copied into the diagnostic surfaces listed above, subject to the named trusted HookOutput exception.
- Reference strings MUST NOT be copied into the diagnostic surfaces listed above, subject to the named trusted HookOutput exception.
- Stable metadata MUST NOT be rewritten by resolved-value redaction, including when it coincides with a resolved value.
- Resolution MUST NOT hold a store connection across await.
- Resolution MUST NOT hold a synchronous lock across await.
- Resolution MUST NOT call std::env::set_var.
- Resolution MUST NOT call std::env::remove_var.

### Rationale

ADR-RUN-0016: the Python hook inherited the complete daemon environment while Monty denied all OS calls. That is an inconsistent capability boundary and has caused difficult identity/team bugs when a daemon starts under one agent and later executes another agent's schedule. The library must support environment choices without silently capturing host values. Owner (ANC-037): "Never capture or persist inherited host values accidentally"
### Success Criteria

**Acceptance Criteria:**

- [ ] Tests make Python, Monty, both CLI adapter, and resolver paths echo canaries and prove redaction, positive delivery, and absence of a non-allowlisted daemon secret.

## Phase C approved target contract (planned, not implemented)

The following additive requirements describe the approved next release; they
do not claim current implementation or alter Phase B provenance. The existing
core remains free of product HTTP/MCP/CLI APIs. A separate optional standard
command adapter is permitted by ADR-RUN-0019.

## REQ-CRON-0020: Public receiver and cooperative owned shutdown

**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-03  
**Owner:** randlee  
**Status:** Approved  

### Requirement Statement

Definitions: The canonical receiver is the CanonicalLog logging contract supplied by sc_command::observability and re-exported by sc_cron::observability, rather than a parallel trait. Source compatibility means existing constructor calls and legacy store implementations compile without source edits. Physical ownership retains admitted work and its resources until actual completion and join even if its async waiter disappears. Physical release occurs after that owned work finishes and its resources are released. The commit fence is the shared admission check for explicit and implicit writes against live stop/class/generation/deadline state. The four named outcome classes are confirmed committed-late, confirmed rollback, no-mutation and indeterminate. TLS means thread-local storage used for the SQLite operation context, not transport security. Uncertain connection state is a connection whose actual transaction outcome is unknown. Restoring the known hooks/TLS baseline means returning SQLite operation hooks and thread-local context to their known pre-operation state on a path that can do so safely. The live operation scope is the shared StoreOperationContext carrying current stop, deadline and cancellation state. A safe restoration path is one on which connection state permits restoring the known operation hooks and thread-local context, with uncertain state retained or quarantined rather than released as known-good.

This target contract is written against `ScheduleStore` and `SqliteScheduleStore`, which PRD-CRON-2 replaces with `Storage<CronRow>`. It has not been re-planned.

The crate exposes the same nominal canonical logging receiver and supporting

values through `sc_cron::observability`; no parallel trait is introduced.

Existing constructors and legacy store implementations remain source compatible.

The open ScheduleStore and generic Scheduler seam permits application-owned

columns/tables through custom stores; bundled-schema preservation does not

forbid consumer schemas or add product fields to Schedule/command DTOs.

Cooperative custom providers must truthfully declare support and forward the

live operation scope, fencing/query semantics and outcome/ownership contract

using their own mechanisms. SQLite-specific callbacks apply to the bundled

SqliteScheduleStore, not to every provider. Legacy/custom integrations may use

the existing caller-owned path with explicitly documented guarantees.

An opt-in cooperative mode rejects unsupported stores before starting work.

It installs one shared operation context before initial load, queue submission,

lock acquisition, reload, decode and all persistence/retry/end paths. Context

checks and provider interruption observe directly mutable

deadlines while blocked. No guard waits indefinitely before consulting context.

Independent stop control closes admission, records only earlier deadlines and

wakes/interrupts operations while an owning join runs. Grace and cleanup are

absolute `std::time::Instant` deadlines in a real stop domain distinct from

the schedule `Clock`. Equal deadlines are valid: T=0 cancels immediately and

grants no cleanup-write budget. Physical joins may take time; no literal

zero-duration or arbitrary native/custom-host preemption guarantee is made.

All admitted worker and context-aware command tasks remain physically owned

until completion even if their async waiter is cancelled. The human-approved

commit admission/outcome contract forbids new ordinary commit admission at/after

G and cleanup commit admission at/after K; it does not promise universal physical

commit/fsync completion by those cutoffs. For bundled SQLite, final commit_hook admission briefly

serializes with live stop state/class/generation/real time, releases the

controller mutex before COMMIT/native I/O, and records stop-first/admission-first.

Second stop remains independently callable while the commit path is blocked.

Explicit and implicit writes share the fence. Confirmed committed-late,

confirmed rollback, no-mutation and indeterminate outcomes are typed distinctly;

expiry/Drop cannot imply rollback or not-persisted. No replay of uncertain

mutations or expired-context readback. Bounded separately authorized readback

may reconcile observed state without claiming exactly-once identity. Uncertain

connection state remains owned/quarantined; restore known hooks/TLS baseline

on safe paths. No effects occur after actual physical release. Cleanup expiry

and actual persisted result are reported honestly after join; no bool false

stands for unknown, no instantaneous T=0 join or universal 2T guarantee.

Legacy direct calls remain

caller-owned. Preserve bundled schema, generation fencing, stop-before-tick, existing

Rust/Python/Monty hooks and REQ-CRON-0019 environment policy.

**MUST statements:**

- `sc_cron::observability` MUST expose the same nominal canonical logging receiver and supporting values.
- Existing constructors and legacy store implementations MUST remain source compatible.
- Custom cooperative providers MUST truthfully declare support.
- Custom cooperative providers MUST forward the live operation scope.
- Custom cooperative providers MUST honor the fencing contract.
- Custom cooperative providers MUST honor the query contract.
- Custom cooperative providers MUST honor the outcome contract.
- Custom cooperative providers MUST honor the physical-ownership contract.
- The opt-in cooperative mode MUST reject unsupported stores before starting work.
- Cooperative work MUST install one shared operation context before initial load, queue submission, lock acquisition, reload, decode and every persistence/retry/end path.
- Blocked context checks MUST observe directly mutable deadlines.
- Provider interruption MUST observe directly mutable deadlines.
- Independent stop control MUST close admission.
- Independent stop control MUST record only earlier deadlines.
- Independent stop control MUST wake or interrupt operations while the owning join runs.
- Grace G MUST be an absolute std::time::Instant deadline in the real stop domain.
- Cleanup K MUST be an absolute std::time::Instant deadline in the real stop domain.
- The real stop domain MUST be distinct from the schedule Clock.
- Equal G and K MUST be accepted.
- T=0 MUST cancel immediately without a cleanup-write budget.
- Admitted worker and context-aware command tasks MUST remain physically owned until completion even if their async waiter is cancelled.
- The bundled SQLite final commit_hook admission decision MUST serialize with live stop state, operation class, generation and real time.
- The bundled SQLite commit path MUST release the controller mutex before COMMIT or native I/O.
- Second stop MUST remain independently callable while the commit path is blocked.
- Explicit and implicit writes MUST share the commit fence.
- Each named outcome class MUST have a typed representation distinct from the representations of the other three.
- Uncertain connection state MUST remain owned or quarantined.
- On safe paths, the known hooks and TLS baseline MUST be restored.
- Cleanup expiry and actual persisted results MUST be reported honestly after join.

**MUST NOT statements:**

- The crate MUST NOT introduce a parallel canonical logging trait.
- Guards MUST NOT wait indefinitely before consulting context.
- Ordinary commit admission MUST NOT occur at or after G.
- Cleanup commit admission MUST NOT occur at or after K.
- Expiry or Drop MUST NOT imply rollback or not-persisted.
- Uncertain mutations MUST NOT be replayed.
- Expired contexts MUST NOT perform readback.
- Effects MUST NOT occur after actual physical release.
- Unknown outcomes MUST NOT be represented as false.

### Rationale

Make the existing cron library consumable through its public logging facade and cooperative operation/lifecycle contract, preserving the already ported Rust, Python and optional Monty execution targets. The framework charter leaves application stores, schemas and wiring to the consumer. Owner (ANC-017): "it is not sc-runtime job to guarantee a customer can not misuse the core libraries."
### Success Criteria

**Acceptance Criteria:**

- [ ] The cron README and isolated external consumer prove one custom field and the chosen cooperative-or-legacy lifecycle through public scheduling/query/end APIs.
### Implementation Notes

Phase C approved target contract (planned, not implemented).

## REQ-CRON-0021: Tracked public operations and bounded schedule query

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-02  
**Owner:** randlee  

### Requirement Statement

Definitions: DTO means data transfer object; adapter DTOs are the command request/response representations owned by sc-cron-commands rather than the native CronOperations port. A permit is one admission-capacity slot held by an admitted public operation. Physical completion is completion of the actual worker or command, rather than completion of its async waiter. The fixed unsupported error is the default response of a legacy store without the new capability. G is the grace deadline and K is the cleanup deadline. WorkerHandle owns framework worker joins. A transport timeout is expiry of the caller-facing request wait; it does not complete the underlying storage mutation. The earliest applicable deadline is the minimum of the parent deadline and requested child deadline. A live scope is the shared StoreOperationContext carrying current stop, deadline and cancellation state. StoreOperationContext is the context input to each CronOperations start, end and query operation. Ascending stable id continuation is pagination in ascending schedule-row id order, with the prior page's continuation id excluded from the next page. A positive capped query limit is an integer from 1 through 1000 specifying the maximum returned rows; a limit outside that range is rejected.

The standard Scheduler path has finite shared intake for public context-aware
start/end/query roots: default64, validated pre-start capacity 1..=65536,
immediate fixed typed unavailable refusal before registered/queued work when
full. One permit remains owned through actual physical completion even when
its async waiter is dropped. Stop/deadline/cancellation admission is atomic;
no hidden capacity-wait queue or automatic replay. Worker dispatch and restricted
internal cleanup retain separate existing ownership and cannot serve as a
public-command bypass. This completes REQ-CRON-0020's admission promise under
NFR-RUN-0002 and ADR-RUN-0018; it does not alter embedder row limits or impose
this mechanism on custom CronOperations/CronExecutor policy. The Phase C
contract defines exact configuration, reason mapping and causal proof owners.

The scheduler provides async context-aware start, generation-fenced stop and
query operations using the same tracked admission and cancellation protocol.
They are required by standard command execution; transport timeout cannot detach
a blocking mutation. A bounded query supports active/ended/all selection,
optional name, ascending stable id continuation and a positive capped limit.
It reads the existing table; `list_active` alone cannot satisfy this contract.
A legacy store's new capability defaults to a fixed unsupported error. End by
name resolves a specific id/generation and cannot end a replacement generation.
The public object-safe native CronOperations port returns boxed Result futures
for start/end/query with StoreOperationContext; it carries no adapter DTOs.
Scheduler's real implementation belongs to c-5. with_deadline derives a child
sharing stop/admission state and the earliest deadline; no independent registry.
No new job kind, job registry, second database or arbitrary hard-delete command
is implied. Ordinary registration continues through REQ-CRON-0003 APIs.

C1 application-owned control construction:

The ordinary-feature `WorkerStopControl::new()` factory creates a live
operation scope inside its owning Tokio runtime, without a Scheduler,
framework worker or test-utils. Applications implementing CronOperations own
intake, physical work and joins. After requesting stop and physically joining
that work, `finish_application_shutdown()` acknowledges completion and
revokes existing and newly obtained scopes even when G was in the future.
The runtime must remain alive for independent status notifications. A checked
ShutdownBudget has private fields and read-only grace()/cleanup() accessors;
K cannot precede G. Existing framework workers retain WorkerHandle-owned joins.

**MUST statements:**

- The standard Scheduler public start/end/query intake MUST have default capacity 64.
- A pre-start capacity override MUST be validated in 1..=65536.
- When intake is full, the scheduler MUST refuse the operation immediately with the fixed typed unavailable error.
- An admitted permit MUST remain owned until actual physical completion even after its async waiter is dropped.
- Stop/deadline/cancellation admission MUST be atomic.
- The scheduler MUST provide async context-aware start under the tracked admission and cancellation protocol.
- The scheduler MUST provide async generation-fenced stop under the tracked admission and cancellation protocol.
- The scheduler MUST provide async context-aware query under the tracked admission and cancellation protocol.
- Bounded query MUST support active selection.
- Bounded query MUST support ended selection.
- Bounded query MUST support selection of all schedules, active and ended.
- Bounded query MUST support an optional name filter.
- Bounded query MUST use ascending stable id continuation.
- Bounded query MUST enforce a positive capped limit.
- Query MUST read the existing table.
- A legacy store capability default MUST return the fixed unsupported error.
- End by name MUST resolve a specific id.
- End by name MUST resolve that schedule incarnation's generation.
- The CronOperations port MUST be object-safe.
- The CronOperations start operation MUST take a StoreOperationContext.
- The CronOperations start operation MUST return a boxed future whose output is a Result.
- The CronOperations end operation MUST take a StoreOperationContext.
- The CronOperations end operation MUST return a boxed future whose output is a Result.
- The CronOperations query operation MUST take a StoreOperationContext.
- The CronOperations query operation MUST return a boxed future whose output is a Result.
- with_deadline MUST derive a child sharing stop state.
- The with_deadline child MUST share admission state.
- The with_deadline child MUST use the earliest applicable deadline.
- WorkerStopControl::new MUST create a live scope within its owning Tokio runtime.
- Application CronOperations implementations MUST own intake.
- Application CronOperations implementations MUST own physical work.
- Application CronOperations implementations MUST own joins.
- After stop and physical join, finish_application_shutdown MUST revoke existing scopes, even when the grace deadline G is in the future.
- After stop and physical join, finish_application_shutdown MUST revoke newly obtained scopes, even when the grace deadline G is in the future.
- The Tokio runtime owning WorkerStopControl MUST remain alive.
- ShutdownBudget MUST have private fields.
- ShutdownBudget MUST provide a read-only grace() accessor.
- ShutdownBudget MUST provide a read-only cleanup() accessor.
- ShutdownBudget MUST reject K before G.
- Framework workers MUST retain WorkerHandle-owned joins.

**MUST NOT statements:**

- When intake is full, the scheduler MUST NOT register work for the refused operation.
- When intake is full, the scheduler MUST NOT queue work for the refused operation.

- WorkerStopControl::new MUST NOT require a Scheduler.
- WorkerStopControl::new MUST NOT require a framework worker.
- WorkerStopControl::new MUST NOT require test-utils.
- The standard path MUST NOT create a hidden capacity-wait queue.
- The standard path MUST NOT automatically replay work.
- Public commands MUST NOT bypass public command admission through private worker dispatch.
- Public commands MUST NOT bypass public command admission through restricted internal cleanup.
- Transport timeout MUST NOT detach a blocking mutation.
- End by name MUST NOT end a replacement generation.
- CronOperations MUST NOT carry adapter DTOs.
- with_deadline MUST NOT create an independent registry.

### Rationale

This completes the existing pre-submission admission/physical-ownership promise of REQ-CRON-0020/0021 and ADR-RUN-0018 under NFR-RUN-0002's nonblocking request path and the existing service-hardening production checklist section 7.
### Success Criteria

**Acceptance Criteria:**

- [ ] Real query returns ended as well as active rows with deterministic bounded continuation; malformed limits/cursors fail; replacement races preserve generation fencing.
- [ ] Cancellation of command waiters leaves no untracked work.
### Implementation Notes

Phase C approved target contract (planned, not implemented).
