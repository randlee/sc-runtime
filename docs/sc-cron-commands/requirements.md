# sc-cron-commands requirements

Status: approved Phase C target, planned and unimplemented. This optional
library reconciles the new human command requirement with the scheduler core's
existing no-product-API rule. It is not an ATM implementation or dependency.

# Requirement: One standard versioned cron command contract

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRNC-0001: One standard versioned cron command contract

**Status:** Approved  

### Requirement Statement

Definitions: The bounded timeout is RunTimeout, whole milliseconds from 1 ms through the core MAX_TIMEOUT of 24 hours. Overflow is a numeric input that cannot be represented by its declared DTO field type. Sensitive read fields are hook source, hook environment and payload, which the standard read view omits unless a separately authorized application projection explicitly exposes them.

Definitions: DTO means data transfer object. CLI means command-line interface. REST is the HTTP command projection. MCP means Model Context Protocol, implemented by the rmcp crate. ATM is the external agent-team-mail tool. CronRequest and CronResponse are the adapter-owned standard cron request and response DTO family. The contract sprint is the sprint defining the canonical command contract and its exact limits from sc-cron. A generation identifies the specific schedule incarnation to fence end operations. Continuation is the bounded list cursor over stable schedule ids.

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
- Start MUST support a RunTimeout of whole milliseconds from 1 ms through the core MAX_TIMEOUT of 24 hours.
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
- Unknown or duplicate fields MUST be rejected before writes.
- Invalid version MUST be rejected before writes.
- Invalid generation MUST be rejected before writes.
- Invalid limits MUST be rejected before writes.
- Invalid cursor MUST be rejected before writes.
- Overflow MUST be rejected before writes.
- Unsupported targets MUST be rejected before writes.
- The contract sprint MUST freeze core-derived size limits.
- The contract sprint MUST freeze value-free error mapping.
- Read views MUST exclude sensitive source, environment and payload unless a separately authorized application projection exposes them.

**MUST NOT statements:**

- The command contract MUST NOT add a serialized function or new job catalog.
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

# Requirement: Optional executor and actual opt-in UI wiring

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRNC-0002: Optional executor and actual opt-in UI wiring

**Status:** Approved  

### Requirement Statement

Definitions: Unsupported Monty means a Monty target requested without the optional Monty capability enabled in the selected build.

Definitions: DTO means data transfer object. CLI means command-line interface. REST is the HTTP command projection. MCP means Model Context Protocol, implemented by the rmcp crate. ATM is the external agent-team-mail tool. CronRequest and CronResponse are the adapter-owned standard cron request and response DTO family. CronExecutor is the object-safe adapter command-execution port. The default surface is the API available without the scheduler feature. Standard validation is the strict version-1 command validation owned by REQ-CRNC-0001. Scope propagation forwards the derived StoreOperationContext into native operations. The selected-default recipe is the README example that explicitly enables standard commands on the bundled store with worker and target setup.

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

- The standard executor MUST NOT introduce a separate timeout or ownership policy.
- A recommended example, DTO-only library or production success/unsupported stub MUST NOT satisfy selected generation.
- Selecting cron_commands MUST NOT enable MCP when MCP is disabled.

### Rationale

Owner (ANC-014): "it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface." (ANC-018): "it is sc-runtime job to provide an easy documented path that consumers will not accidently misuse the core libraries."

### Success Criteria

**Acceptance Criteria:**

- [ ] The generated-consumer and registry-only proofs consume that recipe using public APIs and documented selection, without manual framework-source edits.

### Implementation Notes

Phase C target, planned and unimplemented.

C-5 implements operations for real Scheduler; c-6 implements the executor with dev-only operation doubles. C-3 prepares injectable projections with test-only executor doubles; c-4 supplies real selected root binding after real c-5/c-6 artifacts.

# Requirement: Minimal independent dependency graphs

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## NFR-CRNC-0001: Minimal independent dependency graphs

**Status:** Approved  

### Requirement Statement

Definitions: Monty is the pydantic interpreter stack, including sc-monty, monty and monty-types. Axum is the Rust HTTP framework supplied by the axum crate.

Definitions: The approved cron/async edges are the optional adapter-to-sc-cron dependency and async support specified for StandardCronExecutor over native CronOperations and ApplicationTargetBindings in ADR-CRNC-0001. Selected daemon/service projections are the generated daemon and service crates wiring the selected cron commands; their own dependency edges enable sc-cron-commands/schema. Separately explicit Monty selection is an independent opt-in, not an effect of enabling scheduler.

Definitions: DTO means data transfer object. CLI means command-line interface. REST is the HTTP command projection. MCP means Model Context Protocol, implemented by the rmcp crate. ATM is the external agent-team-mail tool. CronRequest and CronResponse are the adapter-owned standard cron request and response DTO family. The schema feature provides schemars JsonSchema and utoipa ToSchema implementations for the adapter-owned DTOs. Owner-local means each schema implementation resides with the crate owning its Rust type. A foreign orphan implementation attempts to implement a foreign trait on a foreign type; a second wire DTO or generic schema substitutes another representation for the canonical typed contract.

DTO-only defaults depend on serde/serde_json and defaults-off sc-command;
exclude cron, SQLite, Tokio, Monty, clap, Axum, RMCP and command/server. The
non-default scheduler feature adds only approved cron/async edges; Monty remains
separately explicit. No sc-command→cron/adapter cycle or sc-runtime→cron/adapter
edge is allowed. UI CLI clients share DTOs without linking scheduler/server.

Optional `schema = ["dep:schemars", "dep:utoipa", "sc-command/schema"]` supplies JsonSchema and
ToSchema on the same adapter-owned DTO family. Dependencies remain optional;
c-1 freezes exact compatible identities with ADR-RUN-0203's real Parameters<T>
and Utoipa probe, then commits compileable declarations/impls and feature/boundary
baseline before c-3/c-6 parallel work. Schema is independent of scheduler and
adds no cron/SQLite/Tokio/Monty/Axum/RMCP/command-server edge. Defaults activate
neither feature. Selected daemon/service projections enable schema on their
own edges; isolated CLI defaults do not. No foreign orphan impl, second wire DTO
or generic schema substitutes for the typed canonical contract. 
The command schema feature supplies canonical owner-local Envelope<T>, OpError
and ErrorKind traits matching their existing custom serialization exactly.
Typed Envelope<CronResponse> and all nested schema types compile under the
same identities; no copied envelope/error model or generic response schema.

**MUST statements:**

- DTO-only defaults MUST depend only on serde, serde_json and defaults-off sc-command.
- The non-default scheduler feature MUST add only approved cron and async dependency edges.
- Monty MUST be enabled only by a separate explicit opt-in feature selection.
- The optional schema feature MUST supply JsonSchema for the adapter-owned DTO family.
- The optional schema feature MUST supply ToSchema for that same adapter-owned DTO family.
- Schema dependencies MUST remain optional.
- The schema feature MUST be independent of scheduler.
- Selected daemon/service projections MUST enable schema on their own edges.
- Canonical Envelope<T> schema traits MUST be owner-local.
- Canonical OpError schema traits MUST be owner-local.
- Canonical ErrorKind schema traits MUST be owner-local.
- Canonical Envelope<T> schema traits MUST match existing custom serialization.
- Canonical OpError schema traits MUST match existing custom serialization.
- Canonical ErrorKind schema traits MUST match existing custom serialization.
- Envelope<CronResponse> and nested schema types MUST compile with the same selected dependency identities.

**MUST NOT statements:**

- An sc-command-to-cron/adapter cycle MUST NOT be introduced.
- An sc-runtime-to-cron/adapter edge MUST NOT be introduced.
- DTO-only defaults MUST NOT link cron.
- DTO-only defaults MUST NOT link SQLite.
- DTO-only defaults MUST NOT link Tokio.
- DTO-only defaults MUST NOT link Monty.
- DTO-only defaults MUST NOT link clap.
- DTO-only defaults MUST NOT link Axum.
- DTO-only defaults MUST NOT link RMCP.
- DTO-only defaults MUST NOT link command/server.
- CLI clients sharing DTOs MUST NOT link scheduler or server implementation.
- The schema feature MUST NOT add cron dependencies.
- The schema feature MUST NOT add SQLite dependencies.
- The schema feature MUST NOT add Tokio dependencies.
- The schema feature MUST NOT add dependencies on the Monty interpreter stack (including `sc-monty`, `monty` and `monty-types`).
- The schema feature MUST NOT add dependencies on the Axum HTTP framework (`axum`).
- The schema feature MUST NOT add RMCP dependencies.
- The schema feature MUST NOT add command/server dependencies.
- Defaults MUST NOT activate schema.
- Defaults MUST NOT activate scheduler.
- Isolated CLI defaults MUST NOT enable schema.
- The typed schema contract MUST NOT use foreign orphan implementations.
- The typed schema contract MUST NOT use a second wire DTO.
- The typed schema contract MUST NOT use generic schema substitutes.
- The schema contract MUST NOT copy the envelope/error model.
- The schema contract MUST NOT use a generic response schema.

### Rationale

UI CLI clients share DTOs without linking scheduler/server. Keep CLI lean: default dependency graphs stay free of server stacks (ADR-RUN-0003); agents and scripts depend on a small CLI binary.

### Success Criteria

**Acceptance Criteria:**

- [ ] C-3 checks real typed projections.
- [ ] c-6 preserves/tests all feature combinations.
- [ ] c-4 compares actual tools/list and OpenAPI schemas in its existing composition fixture.

### Implementation Notes

Phase C target, planned and unimplemented.
