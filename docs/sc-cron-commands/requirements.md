# sc-cron-commands requirements

Status: approved Phase C target, planned and unimplemented. This optional
library reconciles the new human command requirement with the scheduler core's
existing no-product-API rule. It is not an ATM implementation or dependency.

## REQ-CRNC-0001: One standard versioned cron command contract

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-02  
**Owner:** randlee  

### Requirement Statement

Definitions: DTO means data transfer object; JSON means JavaScript Object Notation; CLI means command-line interface; MCP means Model Context Protocol; HTTP means Hypertext Transfer Protocol; SQL means Structured Query Language; ATM means agent-team-mail. A strict version-1 DTO is the request representation whose version and fields are validated before writes. Envelope and OpError are the existing sc-command result and error types. A Monty target runs the optional Monty interpreter; Rust targets use the application Job or dispatcher. Opaque payload/environment are the application JSON data and hook-environment configuration carried by the request. Bounded timeout uses the core RunTimeout type and its existing maximum, MAX_TIMEOUT of 24 hours. List continuation is ascending stable row-id pagination with a positive capped result limit and active/ended/all selection. A cursor is the prior page's exclusive row-id continuation value. Overflow means a numeric value not representable by its declared field type. An unsupported target is a target unavailable in the selected build or application bindings. Core-derived limits are the existing scheduler input limits frozen by the contract sprint. Value-free error mapping uses stable typed codes rather than raw SQL, environment, source or payload. Sensitive read fields are hook source, environment and payload. A serialized function would carry executable Rust function data in a request; a job catalog would add another job registration system. The five-field grammar is the ordinary cron syntax; supported aliases remain accepted. A read view is the command response exposing schedule-row data. A separately authorized application projection is an application-owned response representation explicitly permitted to expose the named sensitive fields; this record specifies no authorization mechanism.

Provide `cron.start`, `cron.end` and `cron.list` through strict version-1 JSON
DTOs and existing sc-command Envelope/OpError. CLI, REST and MCP projections
use the same request, typed result and namespaced error codes. Start selects
existing Rust, Python or optional Monty targets, ordinary five-field cron or
supported aliases, bounded timeout and opaque payload/environment. Rust uses
the application-configured Job/dispatcher; no serialized function or new job
catalog. End requires name and generation. List supports bounded deterministic
active/ended/all continuation with id/generation/state views. No implicit
end-all, hard deletion, ATM team/phase/mandatory-until or six-field-only grammar.

Reject unknown/duplicate fields, invalid version/generation/limits/cursor,
overflow and unsupported target before writes. Freeze exact core-derived size
limits and value-free error mapping in the contract sprint. Do not expose raw
SQL, environment values, source code or payload in diagnostics. Read views
exclude sensitive source/environment/payload unless a separate authorized
application projection explicitly exposes them.

**MUST statements:**

- `cron.start` MUST use a strict version-1 JSON DTO.
- `cron.end` MUST use a strict version-1 JSON DTO.
- `cron.list` MUST use a strict version-1 JSON DTO.
- Command results MUST use the existing sc-command Envelope/OpError.
- CLI, REST and MCP projections MUST use the same request.
- CLI, REST and MCP projections MUST use the same typed result.
- CLI, REST and MCP projections MUST use the same namespaced error codes.
- Start MUST support existing Rust targets.
- Start MUST support existing Python targets.
- Start MUST support optional Monty targets.
- Start MUST support ordinary five-field cron or supported aliases.
- Start MUST support a bounded timeout.
- Start MUST support opaque payload/environment.
- Rust commands MUST use the application-configured Job or dispatcher.
- End MUST require name.
- End MUST require generation.
- List MUST support bounded deterministic continuation for active schedules.
- List MUST support bounded deterministic continuation for ended schedules.
- List MUST support bounded deterministic continuation for all schedules.
- List views MUST expose id.
- List views MUST expose generation.
- List views MUST expose state.
- Unknown fields MUST be rejected before writes.
- Duplicate fields MUST be rejected before writes.
- Invalid version MUST be rejected before writes.
- Invalid generation MUST be rejected before writes.
- Invalid limits MUST be rejected before writes.
- Invalid cursor MUST be rejected before writes.
- Overflow MUST be rejected before writes.
- Unsupported targets MUST be rejected before writes.
- The contract sprint MUST freeze core-derived size limits.
- The contract sprint MUST freeze value-free error mapping.
- Read views MUST exclude sensitive source unless a separately authorized application projection explicitly exposes that source.
- Read views MUST exclude environment unless a separately authorized application projection explicitly exposes that environment.
- Read views MUST exclude payload unless a separately authorized application projection explicitly exposes that payload.

**MUST NOT statements:**

- The command contract MUST NOT add a serialized function.
- The command contract MUST NOT add a new job catalog.
- Diagnostics MUST NOT expose raw SQL.
- Diagnostics MUST NOT expose environment values.
- Diagnostics MUST NOT expose source code.
- Diagnostics MUST NOT expose payload.
- The contract MUST NOT introduce implicit end-all.
- The contract MUST NOT introduce hard deletion.
- The contract MUST NOT introduce ATM team/phase/mandatory-until.
- The contract MUST NOT introduce six-field-only grammar.

### Rationale

This optional library reconciles the new human command requirement with the scheduler core's existing no-product-API rule.

### Success Criteria

**Acceptance Criteria:**

- [ ] Serialization/schema tests demonstrate one canonical versioned start/end/list and Envelope result/error across projection-neutral fixtures.
- [ ] All invalid forms fail before scheduler/store work; no absent name means end-all.

### Implementation Notes

Phase C target, planned and unimplemented.

## REQ-CRNC-0002: Optional executor and actual opt-in UI wiring

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: Unsupported Monty means a Monty target requested without the optional Monty capability enabled in the selected build. CLI means command-line interface. MCP means Model Context Protocol. Standard validation is the version, field, target, limit and cursor validation in REQ-CRNC-0001. Scope propagation passes the live StoreOperationContext into the native operation. Selected generation means cron_commands=true with cron selected. The core is sc-cron's scheduler API, usable without sc-cron-commands. A CLI end projection is the generated end command forwarding its typed request to the same CronExecutor as the other selected command surfaces. The adapter is the `sc-cron-commands` crate. `CronExecutor` is the object-safe executor trait exported by the adapter with default features. `StandardCronExecutor` is the executor provided under its non-default scheduler feature; the standard executor and standard implementation mean it. `CronOperations` is sc-cron's native operations port ([REQ-CRON-0021](../sc-cron/requirements.md)).

Default-surface object-safe CronExecutor accepts sole CronRequest and real
deadline. Scheduler-feature StandardCronExecutor consumes native CronOperations
and explicit ApplicationTargetBindings ports defined in the Phase C contract;
it derives a child operation context, never a separate timeout/ownership policy
inside the standard executor. Applications may implement CronOperations or
CronExecutor through these public seams for different behavior or shutdown
policy; their task/physical-join ownership, persistence truth, authorization
and error mapping responsibilities must be documented rather than inheriting
the bundled cooperative guarantee. Standard validation and scope propagation
remain mandatory for the standard implementation.
Unsupported Monty fails before persistence.
Selected generator configuration installs actual CLI start/end/list, HTTP and
optional MCP projections calling this same executor; a recommended example or
DTO-only library does not satisfy selected wiring. `cron_commands=false` is
the generation default; `true` requires cron selected and does not enable MCP
when MCP is disabled. No production success/unsupported stub satisfies this requirement. Existing
applications can use the core with no commands. The existing adapter README
provides one working selected-default recipe with bundled store, explicit
worker/target registration and startup, commands, and stop/join/outcome handling;
it links the core custom-store example and describes supported replacement.
No consumer-product migration or automatic job start is implied.

**MUST statements:**

- The default-surface object-safe CronExecutor MUST accept the sole CronRequest type.
- The default-surface object-safe CronExecutor MUST accept a real deadline.
- Scheduler-feature StandardCronExecutor MUST consume native CronOperations.
- Scheduler-feature StandardCronExecutor MUST consume explicit ApplicationTargetBindings.
- The standard executor MUST derive a child operation context.
- Custom CronOperations or CronExecutor implementations MUST document task/physical-join ownership responsibilities.
- Custom CronOperations or CronExecutor implementations MUST document persistence truth responsibilities.
- Custom CronOperations or CronExecutor implementations MUST document authorization responsibilities.
- Custom CronOperations or CronExecutor implementations MUST document error mapping responsibilities.
- The standard implementation MUST retain standard validation.
- The standard implementation MUST retain scope propagation.
- Unsupported Monty MUST fail before persistence.
- Selected generation MUST install an actual CLI start projection calling that same executor.
- Selected generation MUST install an actual CLI end projection calling that same executor.
- Selected generation MUST install an actual CLI list projection calling that same executor.
- Selected generation MUST install actual HTTP projections calling that same executor.
- Selected generation MUST install actual optional MCP projections calling that same executor.
- cron_commands MUST default to false.
- Selecting cron_commands MUST require cron.
- Applications MUST be able to use the core without commands.
- The adapter README MUST provide a working selected-default recipe.
- That recipe MUST demonstrate the bundled store.
- That recipe MUST demonstrate explicit worker/target registration.
- That recipe MUST demonstrate startup.
- That recipe MUST demonstrate commands.
- That recipe MUST demonstrate stop handling.
- That recipe MUST demonstrate join handling.
- That recipe MUST demonstrate outcome handling.
- The adapter README MUST link the core custom-store example.
- The adapter README MUST describe supported replacement.

**MUST NOT statements:**

- The standard executor MUST NOT introduce a separate timeout policy.
- The standard executor MUST NOT introduce a separate ownership policy.
- A recommended example MUST NOT satisfy selected generation.
- A DTO-only library MUST NOT satisfy selected generation.
- A production success stub MUST NOT satisfy selected generation.
- A production unsupported stub MUST NOT satisfy selected generation.
- Selecting cron_commands MUST NOT enable MCP when MCP is disabled.

### Rationale

Owner (ANC-014): "it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface." (ANC-018): "it is sc-runtime job to provide an easy documented path that consumers will not accidently misuse the core libraries."

### Success Criteria

**Acceptance Criteria:**

- [ ] The generated-consumer and registry-only proofs consume that recipe using public APIs and documented selection, without manual framework-source edits.

### Implementation Notes

Phase C target, planned and unimplemented.

C-5 implements operations for real Scheduler; c-6 implements the executor with dev-only operation doubles. C-3 prepares injectable projections with test-only executor doubles; c-4 supplies real selected root binding after real c-5/c-6 artifacts.

## NFR-CRNC-0001: Minimal independent dependency graphs

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-02  
**Owner:** randlee  

### Requirement Statement

Definitions: DTO means data transfer object. CLI means command-line interface. The adapter is the `sc-cron-commands` crate at `crates/sc-cron-commands`. The adapter manifest is `crates/sc-cron-commands/Cargo.toml`. The cron DTO family is the set of public structs and enums in `crates/sc-cron-commands/src/contract.rs`. The DTO dependency entries are `serde`, `serde_json` and `sc-command` with `default-features = false`. A required dependency entry is a `[dependencies]` entry of the adapter manifest without `optional = true`. The schema traits are `schemars::JsonSchema`, `utoipa::ToSchema` and `utoipa::PartialSchema`. An implementation of a trait is an `impl` block or a `derive` of that trait. The schema dependencies are the `schemars` and `utoipa` entries of the adapter manifest. The envelope/error model is the `sc-command` types `Envelope<T>`, `OpError` and `ErrorKind`. `CronResponse` is the cron response enum in the cron DTO family. The `server` cargo feature of `sc-command` adds `axum` and `rmcp`. The default tree command is `cargo tree -p sc-cron-commands -e normal --prefix none`. The schema tree command is `cargo tree -p sc-cron-commands -e normal --prefix none --features schema`. The sc-command tree command is `cargo tree -p sc-command -e normal --prefix none`. The sc-runtime tree command is `cargo tree -p sc-runtime -e normal --prefix none`. The schema feature tree is `cargo tree -p sc-cron-commands -e features --prefix none --features schema`. The scheduler feature tree is `cargo tree -p sc-cron-commands -e features --prefix none --features scheduler`. The default feature query is `cargo tree -p sc-cron-commands -e features --prefix none -i sc-command`. The schema feature query is `cargo tree -p sc-cron-commands -e features --prefix none -i sc-command --features schema`. A generated project with cron commands is a project that `cargo-generate` renders from `template/` with the `cron_commands` placeholder set to true ([REQ-CRNC-0002](#req-crnc-0002-optional-executor-and-actual-opt-in-ui-wiring)). The CLI tree command is `cargo tree -p <project>-cli -e normal --prefix none`, run in a generated project with cron commands ([NFR-RUN-0001](../requirements.md)). A feature list is the array assigned to a feature name in the `[features]` table of the adapter manifest. A command output lists a package when one of its lines starts with the package name followed by ` v`. The schemars schema of a type is the schema returned by `schemars::schema_for!` for that type. The utoipa schema of a type is the schema returned by `utoipa::PartialSchema::schema` for that type. The serialized members of a type are the field names and variant names that its `Serialize` implementation emits. The schema members of a schema are the property names and enum variant names that the schema declares. A copy of a type is a separate struct or enum with the same serde fields or variants. A schema rejects a JSON value when that value fails validation against the schema.

**MUST statements:**

- Every required dependency entry MUST be a DTO dependency entry.
- Each DTO dependency entry MUST be a required dependency entry.
- The schemars schema of `CronRequest` MUST reject at least one JSON value.
- The utoipa schema of `CronRequest` MUST reject at least one JSON value.
- The schemars schema of `Envelope<CronResponse>` MUST reject at least one JSON value.
- The utoipa schema of `Envelope<CronResponse>` MUST reject at least one JSON value.
- In the adapter manifest, the `scheduler` feature list MUST be exactly `["dep:sc-cron"]`.
- In the adapter manifest, the `monty` feature list MUST contain `sc-cron/monty`.
- If a feature list contains `sc-cron/monty`, it MUST be the `monty` feature list.
- With the `schema` feature enabled, `schemars::schema_for!` MUST return a schema for every type in the cron DTO family.
- With the `schema` feature enabled, `utoipa::PartialSchema::schema` MUST return a schema for each type in the cron DTO family.
- Each schema dependency MUST be declared with `optional = true` in the adapter manifest.
- In a generated project with cron commands, the `features` array of the `sc-cron-commands` dependency entry in `crates/daemon/Cargo.toml` MUST contain `"schema"`.
- In a generated project with cron commands, the `features` array of the `sc-cron-commands` dependency entry in `crates/service/Cargo.toml` MUST contain `"schema"`.
- Every implementation of a schema trait for `Envelope<T>` MUST be in a file under `crates/sc-command/src/`.
- Every implementation of a schema trait for `OpError` MUST be in a file under `crates/sc-command/src/`.
- Every implementation of a schema trait for `ErrorKind` MUST be in a file under `crates/sc-command/src/`.
- Every JSON value emitted by the `Serialize` implementation of `Envelope<T>` MUST validate against the schemars schema of `Envelope<T>`.
- Every JSON value emitted by the `Serialize` implementation of `Envelope<T>` MUST validate against the utoipa schema of `Envelope<T>`.
- The schema members of the schemars schema of `Envelope<T>` MUST equal the serialized members of `Envelope<T>`.
- The schema members of the utoipa schema of `Envelope<T>` MUST equal the serialized members of `Envelope<T>`.
- Every JSON value emitted by the `Serialize` implementation of `OpError` MUST validate against the schemars schema of `OpError`.
- Every JSON value emitted by the `Serialize` implementation of `OpError` MUST validate against the utoipa schema of `OpError`.
- The schema members of the schemars schema of `OpError` MUST equal the serialized members of `OpError`.
- The schema members of the utoipa schema of `OpError` MUST equal the serialized members of `OpError`.
- Every JSON value emitted by the `Serialize` implementation of `ErrorKind` MUST validate against the schemars schema of `ErrorKind`.
- Every JSON value emitted by the `Serialize` implementation of `ErrorKind` MUST validate against the utoipa schema of `ErrorKind`.
- The schema members of the schemars schema of `ErrorKind` MUST equal the serialized members of `ErrorKind`.
- The schema members of the utoipa schema of `ErrorKind` MUST equal the serialized members of `ErrorKind`.
- With the `schema` feature enabled, calling `schemars::schema_for!(Envelope<CronResponse>)` MUST return a JSON Schema document.
- With the `schema` feature enabled, calling `<Envelope<CronResponse> as utoipa::PartialSchema>::schema()` MUST return a JSON Schema document.
- The number of distinct `schemars` versions in the output of the schema tree command MUST be 1.
- The number of distinct `utoipa` versions in the output of the schema tree command MUST be 1.

**MUST NOT statements:**

- The output of the schema feature tree MUST NOT contain the text `sc-cron-commands feature "scheduler"`.
- The output of the scheduler feature tree MUST NOT contain the text `sc-cron-commands feature "schema"`.
- The `default` feature list in the adapter manifest MUST NOT contain `schema`.
- The `default` feature list in the adapter manifest MUST NOT contain `scheduler`.
- A crate other than `sc-cron-commands` MUST NOT implement a schema trait for a type in the cron DTO family.
- A crate other than `sc-cron-commands` MUST NOT define a copy of a type in the cron DTO family.
- A crate other than `sc-command` MUST NOT define a copy of a type in the envelope/error model.
- The output of the default tree command MUST NOT list the `sc-cron` package.
- The output of the default tree command MUST NOT contain the text `sqlite`.
- The output of the default tree command MUST NOT list the `tokio` package.
- The output of the default tree command MUST NOT contain the text `monty`.
- The output of the default tree command MUST NOT list the `clap` package.
- The output of the default tree command MUST NOT list the `axum` package.
- The output of the default tree command MUST NOT list the `rmcp` package.
- The output of the default feature query MUST NOT contain the text `sc-command feature "server"`.
- The output of the CLI tree command MUST NOT list the `sc-cron` package.
- In a generated project with cron commands, the output of `cargo tree -p <project>-cli -e features --prefix none -i sc-command` MUST NOT contain the text `sc-command feature "server"`.
- The output of the schema tree command MUST NOT list the `sc-cron` package.
- The output of the schema tree command MUST NOT contain the text `sqlite`.
- The output of the schema tree command MUST NOT list the `tokio` package.
- The output of the schema tree command MUST NOT contain the text `monty`.
- The output of the schema tree command MUST NOT list the `axum` package.
- The output of the schema tree command MUST NOT list the `rmcp` package.
- The output of the schema feature query MUST NOT contain the text `sc-command feature "server"`.
- In a generated project with cron commands, the output of `cargo tree -p <project>-cli -e features --prefix none -i sc-cron-commands` MUST NOT contain the text `sc-cron-commands feature "schema"`.
- The output of the sc-command tree command MUST NOT list the `sc-cron` package.
- The output of the sc-command tree command MUST NOT list the `sc-cron-commands` package.
- The output of the sc-runtime tree command MUST NOT list the `sc-cron` package.
- The output of the sc-runtime tree command MUST NOT list the `sc-cron-commands` package.

### Rationale

UI CLI clients share DTOs without linking scheduler/server. Keep CLI lean: default dependency graphs stay free of server stacks (ADR-RUN-0003); agents and scripts depend on a small CLI binary.

### Success Criteria

**Acceptance Criteria:**

- [ ] C-3 checks real typed projections.
- [ ] c-6 preserves/tests all feature combinations.
- [ ] c-4 compares actual tools/list and OpenAPI schemas in its existing composition fixture.

### Implementation Notes

Phase C target, planned and unimplemented.

c-1 freezes the exact compatible schemars and utoipa identities with ADR-RUN-0203's real Parameters<T> and Utoipa probe, then commits compileable declarations, implementations and the feature/boundary baseline before c-3/c-6 parallel work.
