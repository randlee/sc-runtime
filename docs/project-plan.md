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

| Sprint | Plan | Status | Branch |
| --- | --- | --- | --- |
| b-1 rusqlite writer | [sprint-b-1](plans/phase-b/sprint-b-1-sqlite-writer.md) | planned | `sprint/b-1-sqlite-writer` |
| b-2 host lint profile | [sprint-b-2](plans/phase-b/sprint-b-2-lint-profile.md) | complete | `sprint/b-2-lint-profile` |
| b-3 sc-cron | [sprint-b-3](plans/phase-b/sprint-b-3-sc-cron.md) | planned | `sprint/b-3-sc-cron` |
| b-4 store pairing | [sprint-b-4](plans/phase-b/sprint-b-4-store-pairing.md) | complete | `sprint/b-4-store-pairing` |
| b-5 store-postgres | [sprint-b-5](plans/phase-b/sprint-b-5-store-postgres.md) | complete | `sprint/b-5-store-postgres` |
| b-6 store-mysql | [sprint-b-6](plans/phase-b/sprint-b-6-store-mysql.md) | complete | `sprint/b-6-store-mysql` |
| b-7 store adaptor | [sprint-b-7](plans/phase-b/sprint-b-7-store-adaptor.md) | planned | `sprint/b-7-store-adaptor` |
| b-8 generation layouts | [sprint-b-8](plans/phase-b/sprint-b-8-generation-layouts.md) | planned | `sprint/b-8-generation-layouts` |
