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

Definitions: DTO means data transfer object. JSON means JavaScript Object Notation. CLI means command-line interface. HTTP means Hypertext Transfer Protocol. REST means representational state transfer, and the REST interface is the HTTP interface of the application. MCP means Model Context Protocol. SQL means Structured Query Language. ATM means agent-team-mail. The cron command contract is the three commands `cron.start`, `cron.end` and `cron.list`, with their request, result and error types, defined by the `sc-cron-commands` crate. A version-1 request is the JSON DTO of one command whose `version` field equals `1`. `Envelope` is the existing sc-command result type. `OpError` is the existing sc-command error type carried by a failed `Envelope`. A projection is one interface that exposes the cron command contract: the CLI, the REST interface or the MCP interface. A namespaced error code is an `OpError` code that starts with `CRON.`. A target is what a schedule runs. Rust, Python and Monty targets are the three target kinds that sc-cron already runs. A Monty target runs in the Monty interpreter and exists only in a build that includes Monty. The run binding is the Job or dispatcher that the application configured for Rust targets. A five-field expression is a cron expression with the fields minute, hour, day of month, month and day of week. A six-field expression is a five-field expression with a seconds field added. A supported alias is a name that the sc-cron schedule parser accepts in place of a five-field expression. `RunTimeout` is the sc-cron timeout type, whose maximum is `MAX_TIMEOUT`, 24 hours. The payload is the application JSON data carried by a `cron.start` request, which the command does not interpret. The environment is the hook-environment configuration carried by a `cron.start` request. A generation is the number that identifies one start of a named schedule. A row id is the storage identifier of a schedule row, which does not change. The limit is the number of rows a `cron.list` request asks for. The limit cap is the largest limit the contract accepts. A cursor is the row id of the last row of the prior page. A page is a `cron.list` result that holds at most the limit of rows, in ascending row-id order, only rows whose row id is greater than the cursor, and the cursor for the next page. A refused request is rejected before scheduler or store work begins and returns a failed `Envelope`. An invalid generation is a generation that is not a positive integer. An invalid limit is a limit that is zero or greater than the limit cap. An invalid cursor is a cursor that is not a positive row id. Overflow is a numeric value that its declared field type cannot represent. An unsupported target is a target unavailable in the selected build or in the application bindings. The contract sprint is the sprint that delivers the cron command contract. A core-derived size limit is an existing sc-cron scheduler input limit that the contract applies to a request field. The value-free error mapping is the table from each failure kind to one fixed namespaced error code; no entry holds raw SQL, an environment value, hook source or payload. A read view is a response of the cron command contract that shows schedule-row data. A separately authorized application projection is an application-owned response that the application explicitly permits to show hook source, environment or payload; it is not a read view, and this record specifies no authorization mechanism. A diagnostic is information emitted to explain a command failure or execution condition, including error fields, logs, traces and events. `OpError` message text and command log lines are examples, not the complete set. A serialized function is executable Rust function data carried in a request. A job catalog is a job registration system other than the run binding. Hard deletion is removing a schedule row from storage. The ATM team field is a request field that holds an ATM team name. The ATM phase field is a request field that holds an ATM phase name. The ATM mandatory-until field is a request field that holds the time until which an ATM schedule is mandatory.

The `sc-cron-commands` crate provides `cron.start`, `cron.end` and `cron.list` as one contract that the CLI, the REST interface and the MCP interface share.

**MUST statements:**

1. `cron.start` MUST accept its input only as a version-1 request.
2. `cron.end` MUST accept its input only as a version-1 request.
3. `cron.list` MUST accept its input only as a version-1 request.
4. Every command of the cron command contract MUST return its result in an `Envelope`.
5. Every error that a command of the cron command contract returns MUST be an `OpError`.
6. Each command MUST have one request type that every projection uses.
7. Each command MUST have one result type that every projection returns.
8. Every projection MUST show each failed command's namespaced error code exactly as the command returned it.
9. `cron.start` MUST accept a Rust target.
10. `cron.start` MUST accept a Python target.
11. In a build that includes Monty, `cron.start` MUST accept a Monty target.
12. `cron.start` MUST accept a five-field expression.
13. `cron.start` MUST accept a supported alias.
14. `cron.start` MUST store the request timeout as a `RunTimeout`.
15. `cron.start` MUST accept a payload.
16. `cron.start` MUST accept an environment.
17. A Rust target MUST run through the run binding.
18. The `cron.end` request MUST have a required name field.
19. The `cron.end` request MUST have a required generation field.
20. A `cron.list` request that selects active schedules MUST return a page.
21. A `cron.list` request that selects ended schedules MUST return a page.
22. A `cron.list` request that selects all schedules MUST return a page.
23. Every schedule entry of a `cron.list` result MUST contain the row id.
24. Every schedule entry of a `cron.list` result MUST contain the generation.
25. Every schedule entry of a `cron.list` result MUST contain the state.
26. A request with an unknown field MUST be a refused request.
27. A request with a duplicate field MUST be a refused request.
28. A request whose `version` is not `1` MUST be a refused request.
29. A request with an invalid generation MUST be a refused request.
30. A request with an invalid limit MUST be a refused request.
31. A request with an invalid cursor MUST be a refused request.
32. A request with an overflow MUST be a refused request.
33. A request with an unsupported target MUST be a refused request.
34. The contract sprint MUST state each core-derived size limit as an exact number in the cron command contract.
35. The contract sprint MUST state the value-free error mapping in the cron command contract.
36. A read view MUST omit hook source.
37. A read view MUST omit the environment.
38. A read view MUST omit the payload.

**MUST NOT statements:**

39. The version-1 request types MUST NOT contain a field that carries a serialized function.
40. The version-1 cron command contract MUST NOT define a job catalog.
41. A diagnostic MUST NOT contain the text of a SQL statement.
42. A diagnostic MUST NOT contain the value of an environment variable.
43. A diagnostic MUST NOT contain hook source code.
44. A diagnostic MUST NOT contain payload content.
45. One `cron.end` request MUST NOT end more than one schedule.
46. Each command of the version-1 cron command contract MUST NOT perform hard deletion.
47. The version-1 request types MUST NOT contain an ATM team field.
48. The version-1 request types MUST NOT contain an ATM phase field.
49. The version-1 request types MUST NOT contain an ATM mandatory-until field.
50. `cron.start` MUST NOT require a six-field expression.

### Rationale

The cron commands are an optional library. It gives people one command contract without adding a product API to the scheduler core. One request type, one result type and one set of error codes keep the three interfaces from drifting apart. Refusing bad input before any write leaves the schedule store unchanged by a failed request. Keeping values out of diagnostics and read views keeps secrets out of logs and responses. Version 1 has no job catalog and no serialized function because Rust targets already run through the application's run binding. Statements 39, 40 and 45 to 50 describe version 1 of the contract; they do not rule out a later contract version.

### Success Criteria

**Acceptance Criteria:**

- [ ] Statements 1 to 3, 26 to 28 and 32: unit tests in `crates/sc-cron-commands/tests/contract.rs` parse one valid version-1 request per command, and one request each with an unknown field, a duplicate field, a `version` other than `1` and a number too large for its field; each bad request fails to parse.
- [ ] Statements 4 and 5: a unit test runs each command once with valid input and once with bad input and asserts the return type is `Envelope` and the failure carries an `OpError`.
- [ ] Statements 6 to 8: a unit test feeds one fixture file per command to the CLI, REST and MCP projections and asserts equal parsed requests, equal serialized results and equal error codes starting with `CRON.`.
- [ ] Statements 9 to 13: unit tests start one schedule with each target kind (Monty under the Monty build feature), one with a five-field expression and one with a supported alias; each returns a successful `Envelope`.
- [ ] Statement 14: a unit test starts a schedule and asserts the stored timeout is a `RunTimeout`; a timeout above `MAX_TIMEOUT` returns a failed `Envelope`.
- [ ] Statements 15 and 16: a unit test starts a schedule with a payload and an environment and asserts both are stored unchanged.
- [ ] Statement 17: a unit test with a recording run binding asserts a Rust target ran through it.
- [ ] Statements 18 and 19: unit tests parse a `cron.end` request without a name and one without a generation; each fails to parse.
- [ ] Statements 20 to 25: unit tests list five active and five ended schedules with limit 2 for each of active, ended and all, follow the cursor to the end, and assert ascending row ids, at most 2 rows per page, no repeated row, and a row id, generation and state in every entry.
- [ ] Statements 29 to 31 and 33: unit tests send a generation of 0, a limit of 0, a limit above the limit cap, a cursor of 0 and a target missing from the bindings; each returns a failed `Envelope`.
- [ ] Statements 26 to 33: each invalid-request test asserts that rejection occurs before any scheduler or store work begins. A recording scheduler/store boundary receives no call for the refused request.
- [ ] Statements 34 and 35: inspection of `crates/sc-cron-commands/src/contract.rs` and `crates/sc-cron-commands/src/executor/errors.rs` finds each size limit as a numeric constant and one fixed code per failure kind.
- [ ] Statements 36 to 38 and 41 to 44: a unit test starts a schedule whose hook source, environment value and payload each hold a unique marker string, forces a store failure, and asserts no marker and no SQL text appears in any `cron.list` result or any command diagnostic output, including error fields, logs, traces and events.
- [ ] Statements 39, 40 and 47 to 49: inspection of `crates/sc-cron-commands/src/contract.rs` finds no request field for function data, no job registration type and no team, phase or mandatory-until field.
- [ ] Statement 45: a unit test with two active schedules sends one `cron.end` request and asserts exactly one schedule ended; a request without a name ends none.
- [ ] Statement 46: a unit test ends a schedule and asserts `cron.list` for ended schedules still returns its row.
- [ ] Statement 50: the five-field test of statement 12 passes with no seconds field.

### Implementation Notes

Phase C target, planned and unimplemented.

## REQ-CRNC-0002: Optional executor and actual opt-in UI wiring

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: Unsupported Monty means a Monty target requested without the optional Monty capability enabled in the selected build. CLI means command-line interface. MCP means Model Context Protocol. Standard validation is the version, field, target, limit and cursor validation in REQ-CRNC-0001. Scope propagation passes the live StoreOperationContext into the native operation. Selected generation means cron_commands=true with cron selected. The core is sc-cron's scheduler API, usable without sc-cron-commands. A CLI end projection is the generated end command forwarding its typed request to the same CronExecutor as the other selected command surfaces. The adapter is the `sc-cron-commands` crate. `CronExecutor` is the object-safe executor trait exported by the adapter with default features. `StandardCronExecutor` is the executor provided under its non-default scheduler feature; the standard executor and standard implementation mean it. `CronOperations` is sc-cron's native operations port ([REQ-CRON-0021](../sc-cron/requirements.md)). A custom cron implementation is an application's own implementation of `CronOperations` or of `CronExecutor`. A custom-implementation application is an application that supplies a custom cron implementation. To document a responsibility of a custom cron implementation is to state, in the application's documentation of that implementation, how that implementation carries the responsibility instead of inheriting the bundled cooperative guarantee. `CronRequest` is the adapter's request type. A real deadline is the caller's absolute deadline for one request, passed as a `std::time::Instant`. `ApplicationTargetBindings` is the adapter's trait through which an application names its targets. The operation context is the `StoreOperationContext` that the standard executor was built with. A child operation context is derived from that context for a request, preserves its shared stop and cancellation state, and has the earlier of the parent deadline and the request deadline. The cron store is the store that `CronOperations` writes to; persistence is a write to the cron store. The bundled store is the SQLite cron store that sc-cron provides. A selected project is a project made by selected generation. A projection is a generated, working command surface (a CLI command, an HTTP route or an MCP tool) that forwards its typed request to a `CronExecutor`. A selected project contains a projection when the generated project serves that projection without further edits. The shared executor is the one `CronExecutor` execution path, over one scheduler, that all projections of a selected project use. A request reaches the shared executor when the result its sender receives is the result that the shared executor returned for that request. The cron commands are `cron.start`, `cron.end` and `cron.list` (REQ-CRNC-0001). A recommended example is example code that a user must copy to obtain a command surface. A DTO-only library is a library that provides the request and response types and no command surface. A production success stub is non-test code, in a projection or in an executor, that returns success for a cron command without performing it. A production unsupported stub is non-test code, in a projection or in an executor, that returns an unsupported error for a cron command in place of performing it. DTO means data transfer object. The public API of the core is the set of `pub` items of the `sc-cron` crate that make up the core. An application can call an item when the application compiles with a call to that item. Monty is the Pydantic Python interpreter. A Monty target is a schedule target that runs Monty source. A refused request is rejected before scheduler or store work begins and returns a failed `Envelope` (REQ-CRNC-0001). The selected-default recipe is the section of the adapter README that shows how to wire a selected project with the bundled store. The generated-consumer proof is the test in which a generated project consumes the selected-default recipe. The registry-only proof is the test in which a project that takes its `sc-*` crates only from the crates.io registry consumes that recipe. Each of the two proofs uses only public APIs and documented selection, without manual framework-source edits. The core custom-store example is sc-cron's documented working example of an application-owned store and schema. Supported replacement means supplying an application implementation of `CronOperations` or `CronExecutor` through its public seam.

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

- The `CronExecutor` trait MUST accept a request only as a `CronRequest` value.
- The `CronExecutor` trait MUST accept a real deadline with each request.
- The constructor of `StandardCronExecutor` MUST take a `CronOperations` implementation.
- The constructor of `StandardCronExecutor` MUST take an `ApplicationTargetBindings` implementation.
- For each request that passes standard validation, the standard executor MUST derive one child operation context.
- A custom-implementation application MUST document its custom cron implementation's task ownership responsibilities.
- A custom-implementation application MUST document its custom cron implementation's physical-join ownership responsibilities.
- A custom-implementation application MUST document its custom cron implementation's persistence truth responsibilities.
- A custom-implementation application MUST document its custom cron implementation's authorization responsibilities.
- A custom-implementation application MUST document its custom cron implementation's error mapping responsibilities.
- The standard executor MUST apply standard validation to every request.
- The standard executor MUST apply scope propagation to every request that passes standard validation.
- A request with an unsupported Monty target MUST be a refused request.
- In a selected project, the output of the CLI command for a `cron.start` request MUST be the result the shared executor returned for that request.
- In a selected project, the output of the CLI command for a `cron.end` request MUST be the result the shared executor returned for that request.
- In a selected project, the output of the CLI command for a `cron.list` request MUST be the result the shared executor returned for that request.
- In a selected project, the HTTP response to each cron command request MUST be the result the shared executor returned for that request.
- When MCP is enabled in a selected project, the MCP tool result for each cron command request MUST be the result the shared executor returned for that request.
- In `wizard/answers.schema.json`, the `default` of the option `cron_commands` MUST be `false`.
- When `cron_commands` is `true`, the generated project MUST also have the `cron` option equal to `true`.
- The public API of the core MUST be callable from an application that does not depend on `sc-cron-commands`.
- The adapter README MUST contain the selected-default recipe.
- The selected-default recipe MUST pass the generated-consumer proof.
- The selected-default recipe MUST pass the registry-only proof.
- The selected-default recipe MUST show the bundled store.
- The selected-default recipe MUST show explicit worker/target registration.
- The selected-default recipe MUST show startup.
- The selected-default recipe MUST show the commands.
- The selected-default recipe MUST show stop handling.
- The selected-default recipe MUST show join handling.
- The selected-default recipe MUST show outcome handling.
- The adapter README MUST link the core custom-store example.
- The adapter README MUST describe supported replacement.

**MUST NOT statements:**

- The standard executor MUST NOT introduce a separate timeout policy.
- The standard executor MUST NOT introduce a separate ownership policy.
- Selected generation MUST NOT produce a recommended example as a command surface.
- The files that selected generation produces MUST NOT be limited to a DTO-only library.
- Selected generation MUST NOT produce a production success stub.
- Selected generation MUST NOT produce a production unsupported stub.
- When the `mcp` option is `false`, the files that selected generation produces MUST NOT contain an MCP endpoint.

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

Definitions: DTO means data transfer object. CLI means command-line interface. The adapter is the `sc-cron-commands` crate at `crates/sc-cron-commands`. The adapter manifest is `crates/sc-cron-commands/Cargo.toml`. The cron DTO family is the set of public structs and enums in `crates/sc-cron-commands/src/contract.rs`. The DTO dependency entries are `serde`, `serde_json` and `sc-command` with `default-features = false`. A required dependency entry is a `[dependencies]` entry of the adapter manifest without `optional = true`. The schema traits are `schemars::JsonSchema`, `utoipa::ToSchema` and `utoipa::PartialSchema`. An implementation of a trait is an `impl` block or a `derive` of that trait. The schema dependencies are the `schemars` and `utoipa` entries of the adapter manifest. The envelope/error model is the `sc-command` types `Envelope<T>`, `OpError` and `ErrorKind`. `CronResponse` is the cron response enum in the cron DTO family. The `server` cargo feature of `sc-command` adds `axum` and `rmcp`. The default tree command is `cargo tree -p sc-cron-commands -e normal --prefix none`. The schema tree command is `cargo tree -p sc-cron-commands -e normal --prefix none --features schema`. The sc-command tree command is `cargo tree -p sc-command -e normal --prefix none`. The sc-runtime tree command is `cargo tree -p sc-runtime -e normal --prefix none`. The schema feature tree is `cargo tree -p sc-cron-commands -e features --prefix none --features schema`. The scheduler feature tree is `cargo tree -p sc-cron-commands -e features --prefix none --features scheduler`. The default feature query is `cargo tree -p sc-cron-commands -e features --prefix none -i sc-command`. The schema feature query is `cargo tree -p sc-cron-commands -e features --prefix none -i sc-command --features schema`. A generated project with cron commands is a project that `cargo-generate` renders from `template/` with the `cron_commands` placeholder set to true ([REQ-CRNC-0002](#req-crnc-0002-optional-executor-and-actual-opt-in-ui-wiring)). The CLI tree command is `cargo tree -p <project>-cli -e normal --prefix none`, run in a generated project with cron commands ([NFR-RUN-0001](../requirements.md)). A feature list is the array assigned to a feature name in the `[features]` table of the adapter manifest. A command output lists a package when one of its lines starts with the package name followed by ` v`. The schemars schema of a type is the schema returned by `schemars::schema_for!` for that type. The utoipa schema of a type is the schema returned by `utoipa::PartialSchema::schema` for that type. The serialized members of a type are the field names and variant names that its `Serialize` implementation emits. The schema members of a schema are the property names and enum variant names that the schema declares. A standard cron command projection is one interface that exposes the cron command contract of [REQ-CRNC-0001](#req-crnc-0001-one-standard-versioned-cron-command-contract). A copy of a type is a separate struct or enum with the same serde fields or variants. A schema rejects a JSON value when that value fails validation against the schema.

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
- A standard cron command projection MUST NOT use a copy of a type in the cron DTO family.
- A standard cron command projection MUST NOT use a copy of a type in the envelope/error model.
- The output of the default tree command MUST NOT list the `sc-cron` package.
- The output of the default tree command MUST NOT contain the text `sqlite`.
- The output of the default tree command MUST NOT contain the text `monty`.
- The output of the default tree command MUST NOT list the `clap` package.
- The output of the default tree command MUST NOT list the `axum` package.
- The output of the default tree command MUST NOT list the `rmcp` package.
- The output of the default feature query MUST NOT contain the text `sc-command feature "server"`.
- The output of the CLI tree command MUST NOT list the `sc-cron` package.
- In a generated project with cron commands, the output of `cargo tree -p <project>-cli -e features --prefix none -i sc-command` MUST NOT contain the text `sc-command feature "server"`.
- The output of the schema tree command MUST NOT list the `sc-cron` package.
- The output of the schema tree command MUST NOT contain the text `sqlite`.
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
