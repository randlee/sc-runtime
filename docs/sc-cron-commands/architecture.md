# sc-cron-commands architecture

## ADR-CRNC-0001: Optional shared protocol and scheduler adapter

**Status:** Approved Phase C design; implementation planned in c-1/c-6, real
composition in c-4. Requirements are [requirements.md](requirements.md).

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
adapter→command[default-features=false]. Command and runtime gain no reverse
edge. Default DTO builds exclude scheduling/server dependencies. A new
BOUNDARY-sc-cron-commands manifest and existing dependent-list updates are
c-1 deliverables. The actual ATM start/end/list API and ADR-069 are behavioral
reference only; no ATM-specific identities, lifecycle or grammar are imported.
