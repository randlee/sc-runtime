# Project plan

Phase A follows the original five milestones: compatibility spike, reusable
core crates, minimal JSON-driven SQLite template, core release, and wizard.
Sprints a-1 through a-8 are merged on `integrate/phase-a`.

See [Phase A plan](plans/phase-a/phase-a-plan.md) for the eight-sprint boundary
cut, five-milestone map, ownership, decision prerequisites, and flexibility
guardrails.

The [deferred-scope table](plans/phase-a/phase-a-plan.md#authority-and-completed-source-reconciliation)
still applies. CLI auto-start (`REQ-RUN-0206`, `ADR-RUN-0204`),
attribute-generated CLI tests (`REQ-RUN-0312`, `ADR-RUN-0304`, `REQ-RUN-0311`),
and framework-generated responses for unknown routes, Axum body rejection, and
rmcp protocol errors (`REQ-RUN-0203` OPEN) stay deferred. Postgres/both,
prototype-repository handoff, and other post-core layers stay outside Phase A.
sc-lint 0.8.0 is in Phase A: a-5 installs it for this repository, and a-6
installs the consumer contract into generated projects.

`REQ-RUN-0702` stays open. Git tag `v0.1.0`, crates.io publication of the four
crates, and the registry-backed fixture matrix at that tag still need release
authorization and registry credentials. Merging the release kit onto
`integrate/phase-a` is the integration record for that later publish.

| Sprint | Plan | Status | Branch |
| --- | --- | --- | --- |
| a-1 compatibility spike | [sprint-a-1](plans/phase-a/sprint-a-1-compatibility-spike.md) | complete | `sprint/a-1-compatibility-spike` |
| a-2 sc-config | [sprint-a-2](plans/phase-a/sprint-a-2-config.md) | complete | `sprint/a-2-config` |
| a-3 sc-transport | [sprint-a-3](plans/phase-a/sprint-a-3-transport.md) | complete | `sprint/a-3-transport` |
| a-4 sc-command | [sprint-a-4](plans/phase-a/sprint-a-4-command.md) | complete | `sprint/a-4-command` |
| a-5 sc-runtime core | [sprint-a-5](plans/phase-a/sprint-a-5-runtime-core.md) | complete | `sprint/a-5-runtime-core` |
| a-6 template from JSON | [sprint-a-6](plans/phase-a/sprint-a-6-template-from-json.md) | complete | `sprint/a-6-template-from-json` |
| a-7 core release | [sprint-a-7](plans/phase-a/sprint-a-7-core-release.md) | complete | `sprint/a-7-core-release` |
| a-8 Wyvern wizard | [sprint-a-8](plans/phase-a/sprint-a-8-wizard.md) | complete | `sprint/a-8-wizard` |

Phase b is the storage phase: writer, lint, cron, store pairing, Postgres, MySQL, the adaptor, and the six generation layouts. See [Phase b plan](plans/phase-b/phase-b-plan.md).

Continuation execution lead: `codexis`. The b-13 through b-18 integration
branch is `integrate/phase-b-observability`; its dependency and execution
ledger is Beads epic `run-03z`. Sprint scope and ownership remain defined by
the approved sprint documents.

The b-13 through b-18 continuation establishes the repository `0.1.1`
contract. b-13 owns the non-blocking version/public API boundary and whole
cron component propagation but performs no release;
b-14, b-15, and b-16 then run in parallel. b-16 follows b-13's generated
graph amendment and owns both generated dependency inventories plus the
trusted in-process Monty example; b-17 adds the shared Python/Monty hook
environment and release-v1 sc-cron schema baseline; b-18 owns the sc-publish
consumer update, a real install and complete synchronization from the exact
pinned kit revision, the human-authorized five-crate publish order, and the
registry-only generated proof plus isolated-CI SIGINT/SIGTERM lifecycle
proof. Publication evidence and the registry-only proof are produced from one
unchanged source commit and that exact commit is tagged `v0.1.1`. b-18 is
blocked until an exact later kit
revision excludes `docs/sprints/**`; the current pin is not a valid input.

| Sprint | Plan | Status | Branch |
| --- | --- | --- | --- |
| b-1 rusqlite writer | [sprint-b-1](plans/phase-b/sprint-b-1-sqlite-writer.md) | complete | `sprint/b-1-sqlite-writer` |
| b-2 host lint profile | [sprint-b-2](plans/phase-b/sprint-b-2-lint-profile.md) | complete | `sprint/b-2-lint-profile` |
| b-3 sc-cron | [sprint-b-3](plans/phase-b/sprint-b-3-sc-cron.md) | complete | `sprint/b-3-sc-cron` |
| b-4 store pairing | [sprint-b-4](plans/phase-b/sprint-b-4-store-pairing.md) | complete | `sprint/b-4-store-pairing` |
| b-5 store-postgres | [sprint-b-5](plans/phase-b/sprint-b-5-store-postgres.md) | complete | `sprint/b-5-store-postgres` |
| b-6 store-mysql | [sprint-b-6](plans/phase-b/sprint-b-6-store-mysql.md) | complete | `sprint/b-6-store-mysql` |
| b-7 store adaptor | [sprint-b-7](plans/phase-b/sprint-b-7-store-adaptor.md) | complete | `sprint/b-7-store-adaptor` |
| b-8 generation layouts | [sprint-b-8](plans/phase-b/sprint-b-8-generation-layouts.md) | complete | `sprint/b-8-generation-layouts` |
| b-9 sc-cron Monty hooks | [sprint-b-9](plans/phase-b/sprint-b-9-sc-cron-monty-hooks.md) | complete | `sprint/b-9-sc-cron-monty-hooks` |
| b-10 sc-cron Monty host bridge | [sprint-b-10](plans/phase-b/sprint-b-10-sc-cron-monty-host-bridge.md) | complete | `sprint/b-10-sc-cron-monty-host-bridge` |
| b-11 benchmark Monty reports | [sprint-b-11](plans/phase-b/sprint-b-11-benchmark-monty-reports.md) | complete | `sprint/b-11-benchmark-monty-reports` |
| b-12 sc-cron schedule row limit | [sprint-b-12](plans/phase-b/sprint-b-12-sc-cron-schedule-row-limit.md) | complete | `sprint/b-12-sc-cron-schedule-row-limit` |
| b-13 observability contract | [sprint-b-13](plans/phase-b/sprint-b-13-observability-contract.md) | implemented; QA pending | `sprint/b-13-observability-contract` |
| b-14 runtime observability | [sprint-b-14](plans/phase-b/sprint-b-14-sc-runtime-observability.md) | implemented; QA pending | `sprint/b-14-sc-runtime-observability` |
| b-15 cron observability | [sprint-b-15](plans/phase-b/sprint-b-15-sc-cron-observability.md) | implemented; QA pending | `sprint/b-15-sc-cron-observability` |
| b-16 Monty template examples | [sprint-b-16](plans/phase-b/sprint-b-16-monty-template-examples.md) | implemented; QA pending | `sprint/b-16-monty-template-examples` |
| b-17 sc-cron hook environment | [sprint-b-17](plans/phase-b/sprint-b-17-sc-cron-hook-environment.md) | planned | `sprint/b-17-sc-cron-hook-environment` |
| b-18 template observability integration | [sprint-b-18](plans/phase-b/sprint-b-18-template-observability-integration.md) | planned | `sprint/b-18-template-observability-integration` |
