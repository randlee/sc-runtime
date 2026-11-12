# sc-cron-commands architecture

## ADR-CRNC-0001: Optional shared protocol and scheduler adapter

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-02  
**Owner:** randlee  
**Decision Date:** 2026-11-02  

### Related Documents

**External References:**

- [requirements.md](requirements.md) — the requirements for this decision

### Context

The problem is that CLI, HTTP and selected MCP need the same start/end/list
cron operation, result and error JSON interface, while default-off generated
configuration must leave core usable alone.

The actual ATM start/end/list API and ADR-069 are behavioral reference only; no ATM-specific identities, lifecycle or grammar are imported.

### Decision

The adapter owns its schema-trait implementations as well as its sole DTOs.
C-1 commits the optional independent schema feature (schemars JsonSchema and
Utoipa ToSchema) and compileable declarations/manifest/boundary baseline before
parallel consumers. Its existing probe freezes exact identities compatible
with selected RMCP Parameters<T> and runtime Utoipa, satisfying ADR-RUN-0203.
Defaults and CLI select neither schema nor scheduler; schema adds no runtime,
server or scheduling stack. Generated api-types imports these types rather
than violating orphan rules or duplicating a wire model. C-3 owns selected
projection feature activation and typed schema tests using the executor double;
c-6 preserves/tests the declaration surface while implementing behavior; c-4
compares actual tools/list/OpenAPI schemas and canonical fixtures. Canonical
Envelope/OpError wrapping remains unchanged; adapter schema forwards
command/schema, whose owner-local Envelope<T>/OpError/ErrorKind schemas match
custom wire serialization rather than deriving a different enum representation.
The full typed Envelope<CronResponse> family is compiled and fixture-checked
by c-1.

The default library contains sole CronRequest DTO/schema, canonical Envelope
results and object-safe CronExecutor interface (std boxed Future, no cron type).
Feature `scheduler` adds StandardCronExecutor over core-native CronOperations
and explicit ApplicationTargetBindings. It derives a child real-deadline context
from the same stop/admission state in the standard implementation. Consumers
may implement the existing CronOperations or CronExecutor ports for custom
behavior/lifecycle policies, owning their cancellation, joins, authorization
and accurate errors/outcomes. Such replacement does not inherit the standard
cooperative guarantee or remove its safeguards. Existing usage docs contain
a selected-default recipe plus replacement responsibilities; c-4 follows the
recipe using public APIs in generated and registry consumers.
Application bindings authorize Rust/Python/Monty
targets; the library neither starts a mandatory worker nor loads application
config. Generated opt-in composition belongs to daemon/service code; core
runtime remains assembly without cron dependencies. CLI sends the same DTO
used by HTTP/MCP. C-1 freezes exact ports/DTOs, limits, stable codes and canonical
tests/contracts/phase-c dev-only doubles plus existing test-utils context probe.
C-3 prepares injectable projections; c-6 implements against operation doubles
in parallel with c-5 real cron operations. C-4 binds the real selected factory
and root calls and closes composed behavior. No production stub, public testing
API or hidden concrete-Scheduler dependency replaces those ports.

Edges: application→adapter[scheduler]→cron→command (temporary logging port),
adapter→command[default-features=false]. A new
BOUNDARY-sc-cron-commands manifest and existing dependent-list updates are
c-1 deliverables.

### Rationale

Owner, 2026-11-03 (ANC-015): "it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface." Phase C objective 4: an optional standard start/end/list JSON adapter presents the same operation/result/error to CLI, HTTP and selected MCP. Default-off generated configuration leaves core usable alone; explicitly selected configuration wires the real shared handler into the existing UIs.

### Alternatives Considered

#### Alternative 1: Ship the command API inside sc-cron

**Description:**

Add the start/end/list HTTP, MCP and CLI API to the sc-cron scheduler crate
itself.

**Why Rejected:**

ADR-RUN-0013 states that sc-cron does not ship that API, and ADR-RUN-0019
amends that restriction only for a separate adapter: the scheduler core keeps
no binary, HTTP/MCP routes or configuration-file loading.

#### Alternative 2: Each consumer wraps its own cron API

**Description:**

Keep the ADR-RUN-0013 arrangement in which each consumer may wrap start, stop
and list in its own HTTP, MCP or CLI API.

**Why Rejected:**

ANC-015 requires supported cron user interfaces to share the standard JSON
command contract, and REQ-CRNC-0001 requires CLI, REST and MCP projections to
use the same request, typed result and namespaced error codes.

#### Alternative 3: Mandatory cron commands in every consumer

**Description:**

Expose the standard cron command set in every generated consumer.

**Why Rejected:**

ANC-014 requires that a consumer application be able to omit the standard
cron command set, and REQ-CRNC-0002 makes `cron_commands=false` the generation
default.

#### Alternative 4: Put the cron command DTOs and handler in sc-command or sc-runtime

**Description:**

Place the standard cron commands in the existing command or runtime crate,
with no separate adapter library.

**Why Rejected:**

NFR-CRNC-0001 allows no sc-command→cron/adapter cycle and no
sc-runtime→cron/adapter edge.

#### Alternative 5: Schema implementations or a copied wire DTO in generated api-types

**Description:**

Let generated api-types implement JsonSchema/ToSchema for the adapter's types
or define its own copy of the cron wire model.

**Why Rejected:**

NFR-CRNC-0001 forbids a foreign orphan impl, a second wire DTO or a generic
schema in place of the typed canonical contract.

#### Alternative 6: Import the ATM start/end/list API

**Description:**

Adopt ATM's cron command API, including its team/phase/mandatory-until fields
and six-field-only grammar.

**Why Rejected:**

REQ-CRNC-0001 excludes ATM team/phase/mandatory-until and six-field-only
grammar, and ADR-RUN-0019 allows no ATM package or second response envelope.

### Consequences

Command and runtime gain no reverse edge. Default DTO builds exclude scheduling/server dependencies. The cost is a new
crate with its own BOUNDARY-sc-cron-commands manifest and dependent-list
updates, amending ADR-RUN-0003's workspace graph and ADR-RUN-0013's
product-API restriction for this adapter. Consumers that replace
CronOperations or CronExecutor do not inherit the standard cooperative
guarantee and own their cancellation, joins, authorization and accurate
errors/outcomes.

### Implementation

Phase C design; implementation planned in c-1/c-6, real composition in c-4.
