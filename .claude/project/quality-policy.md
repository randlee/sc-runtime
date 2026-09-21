# Repository Quality Policy

This file contains repository-specific QA policy. Reusable agents and skills
must read this file rather than embedding repository names, commands,
interfaces, approval authorities, or temporary architectural exceptions in
their own prompts.

## Repository Baseline

- Default comparison branch: `develop`
- Requirements index: `docs/requirements.md`
- Architecture index: `docs/architecture.md`
- Project plan: `docs/project-plan.md` when present
- Team protocol: `docs/team-protocol.md`

## Reviewer Policy

- Initial implementation review: `req-qa`, `arch-qa`,
  `ruthless-boundary-qa`, `rust-qa-agent`,
  `rust-best-practices-agent`, and `rust-service-hardening-agent`
- Fix verification: `req-qa`, `arch-qa`, and `rust-qa-agent`; dispatch an
  additional reviewer only when rechecking a carry-forward finding owned by
  that reviewer
- Plan review: `req-qa`, `arch-qa`, `ruthless-boundary-qa`,
  `rust-best-practices-agent`, and `rust-service-hardening-agent`
- Phase-end review: the initial implementation set plus `flaky-test-qa`
- Run `flaky-test-qa` earlier when tests changed or instability is suspected
- Run `schema-reviewer` only when the governed-interface section below names
  an interface relevant to the assigned scope

## Validation Policy

No repository-wide phase-end artifact command is configured yet. An
implementation phase must add its authoritative command here before phase-end
QA can pass. Reviewer assignments may provide narrower commands for earlier
rounds.

## Governed Interfaces

No governed interface policy is currently defined. Until this section names
an interface, its compatibility rule, evidence paths, and approval authority,
`schema-reviewer` returns `SKIPPED`.

## Repository Exceptions

None.
