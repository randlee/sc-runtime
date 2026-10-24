# sc-runtime framework charter

**Status:** Active planning authority  
**Audience:** Maintainers, plan authors, QA reviewers, and every generated consumer project  
**Companion:** [`sc-runtime-design.md`](sc-runtime-design.md), [`architecture.md`](architecture.md)

This document states what **sc-runtime** is in product terms. It is binding for
plan review, sprint review, and architectural QA whenever work touches public
crate APIs, the template, or generation contracts.

## What we are building

**sc-runtime is a multi-consumer scaffolding framework**, not a single
application and not a traditional “batteries included” web framework.

- **Many projects** will depend on the four published crates and the
  cargo-generate template. API and boundary choices must assume **unknown
  downstream wiring**, not one reference app.
- **The framework instantiates; each project wires.** Published crates own
  daemon lifecycle, transport, envelopes, and assembly. Generated projects own
  operations, routes, MCP tools, CLI commands, stores, and schemas.
- **Thin template, versioned crates.** Anything that should improve across all
  consumers belongs in a published crate. Anything a team will edit belongs in
  the generated tree.

Phase A delivers the **minimal spine** of that framework: prove the stack,
ship four small libraries, generate one editable example application, publish,
and optionally add the Wyvern wizard over the answers JSON contract.

## Agent-first, non-traditional framework

Consumers are often **agents and automation**, not only human developers.

| Pillar | Framework obligation | Project freedom |
| --- | --- | --- |
| **JSON CLI** | Clap commands talk HTTP to the daemon; responses use the shared envelope and typed errors. | Command names, payloads, and business rules. |
| **Actionable errors (mandatory)** | Every cross-boundary failure exposes stable codes, causes, and **recovery/remediation** (`OpError`, leaf error inventories, transport `ErrorCode`). Agents must be able to act on failures without reading source. | Domain-specific error variants inside project code. |
| **MCP “a plug-in away”** | Same service functions serve REST and MCP; `sc-command` server feature converts envelopes; MCP mounts on the shared Axum router. | Which tools exist and what they call. |
| **Data: local first, cloud path** | Template ships **SQLite** via sqlx in a dedicated store crate; Postgres (or other backends) is an **additional store crate**, not a rewrite of the framework. | Schemas, migrations, and which store each route uses. |
| **Minimal surface** | No command registry, no codegen framework, no portable query layer, no opinions on frontend or deployment. Standard crates used the documented way. | All application-layer structure. |

If a plan or change adds registries, macros, indirection layers, or “helper
frameworks” inside reusable crates **without** reducing code in **many**
consumers, treat it as suspect until justified against this charter.

## Extensibility rules (maintainers)

1. **Prefer fewer public types** with clear composition (`Daemon::builder` typestate,
   ordinary Axum `Router`, plain async service functions) over extensibility
   hooks that only one consumer needs.
2. **Do not freeze application architecture.** Example layout in the template
   is illustrative; boundary manifests and ADRs govern **libraries**, not how
   every generated repo organizes `service/` or `store/`.
3. **Preserve upgrade paths.** Breaking changes to published crates require ADR
   amendment and semver discipline; generated code is not upgraded in place.
4. **Keep CLI lean.** Default dependency graphs stay free of server stacks
   (ADR-RUN-0003). Agents and scripts depend on a small CLI binary.
5. **Errors are part of the API.** Removing or weakening remediation, inventing
   stringly-typed failures, or hiding sources behind `Box<dyn Error>` on public
   surfaces violates the agent-first contract.

## What arch-qa and plan review must verify

When reviewing Phase A or later work, confirm the artifact under review:

- Names **published crates + template** as the deliverable, not “an app.”
- Treats generated projects as **consumers** with editable application layers.
- Does not smuggle product-specific policy (auto-start, test harness shape,
  deployment) into reusable libraries without an ADR.
- Keeps Phase A scope **minimal** while leaving room for Postgres, OTLP, and
  richer MCP without redesigning the spine.

Normative detail lives in requirements and ADRs; this charter is the
one-screen reminder that **sc-runtime exists to be reused**.
