# sc-cron-commands architecture

## ADR-CRNC-0001: Optional shared protocol and scheduler adapter

**Status:** Approved Phase C design; implementation planned in c-1/c-6, real
composition in c-4. Requirements are [requirements.md](requirements.md).

The default library contains versioned DTO/schema and canonical Envelope
results. Feature `scheduler` adds a handler consuming the core's tracked
context-aware API. Application bindings authorize existing Rust/Python/Monty
targets; the library neither starts a mandatory worker nor loads application
config. Generated opt-in composition belongs to daemon/service code; core
runtime remains assembly without cron dependencies. CLI sends the same DTO
used by HTTP/MCP. c-1 freezes exact signatures, limits, stable codes and a
contract double; c-6 implements against it in parallel with real cron work.

Edges: application→adapter[scheduler]→cron→command (temporary logging port),
adapter→command[default-features=false]. Command and runtime gain no reverse
edge. Default DTO builds exclude scheduling/server dependencies. A new
BOUNDARY-sc-cron-commands manifest and existing dependent-list updates are
c-1 deliverables. The actual ATM start/end/list API and ADR-069 are behavioral
reference only; no ATM-specific identities, lifecycle or grammar are imported.
