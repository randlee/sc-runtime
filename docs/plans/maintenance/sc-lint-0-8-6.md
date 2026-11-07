---
id: sc-lint-0-8-6
status: complete
branch: codex/upgrade-sc-lint-0.8.6
worktree: /Users/randlee/github/sc-runtime-worktrees/codex/upgrade-sc-lint-0.8.6
closure_type: integration
target_boundary: generated-project-install
recommended_agent: rust-developer
recommended_model: gpt-6.1-sol
---

# sc-lint 0.8.6 repeat-init maintenance sprint

## Requirements and decisions

- **Requirements:** REQ-RUN-0004 (root lint integration and release pin); REQ-RUN-0307 (consumer install, version pin, recipes and repeated init); REQ-RUN-0308 (generated SQLite store regression); NFR-RUN-0005 (isolated, bounded tests); NFR-RUN-0006 (template remains free of copied lint policy).
- **ADRs:** ADR-RUN-0005 (sc-lint owns lint and `just` infrastructure); ADR-RUN-0301 (one store crate per database backend); ADR-RUN-0011 (bounded causal async synchronization without sleeps or blocking).
- **Scope:** maintenance correction to the current operational contract and regression tests. Historical Phase C acceptance evidence and immutable 0.2-era records are preserved.

## Deliverables

- Update root and generated-template minimum pins to released sc-lint 0.8.6, and update the active Boundary Enforcement product minimum if consistent.
- Keep the sc-publish-vendored setup action's 0.8.4 default unchanged while no repository workflow calls it; enforce that it remains unused. The kit fallback is not an active root or generated-consumer pin.
- Prove the released 0.8.6 binary preserves actual SDK custom profiles, the consumer-owned `extra-db-prepare` recipe, and an edited external boundary policy through repeated native `init --just` and `setup`/`upgrade` operations. Keep this verification isolated from global installations and consumer policy.
- Keep the generated SQLite writer regression bounded and deterministic: use two independent one-connection pools, 50 two-party barrier-started insert rounds, and joined futures for 100 writes.
- Restore successful repeated-init requirements, ADR statements, and regression assertions where they were weakened for the 0.8.4 expected-exit-3 workaround. Remove the #117 waiver only if the released binary passes the preservation proof.

## Acceptance criteria

1. Root and template pins, current REQ-RUN-0307/ADR-RUN-0005 text, and active quality policy consistently name 0.8.6; historical Phase C evidence is unchanged.
2. Current REQ-RUN-0004 describes the 0.8.6 root integration. The vendored setup action fallback remains 0.8.4 only while no workflow invokes it, and a regression guards that fact.
3. The regression exercises a generated SDK-shaped project with its real custom profiles, an `extra-db-prepare` recipe, and an edited external boundary policy. Using the released 0.8.6 binary, repeated native init/setup/upgrade succeeds and preserves customized bytes and recipe behavior.
4. The test fails on loss of any profile, recipe, or edited policy and asserts successful exit status; no expected exit-3 waiver remains if criterion 3 passes.
5. Existing consumer-profile and runtime-upgrade regressions pass. No generated template lint policy is added.

## Owned paths

- `docs/plans/maintenance/sc-lint-0-8-6.md`
- `docs/project-plan.md`
- `.claude/project/quality-policy.md`
- `.github/actions/setup-sc-lint/action.yml` (usage guard only; the pinned kit fallback stays intact)
- `sc-lint.toml`, `template/sc-lint.toml`
- `docs/requirements.md`, `docs/architecture.md`
- `tests/unit/generation/test_consumer_sc_lint_profile.py`, `tests/unit/test_sc_lint_setup_action_pin.py`
- `template/crates/store-sqlite/src/lib.rs` (bounded regression only)

## This sprint does not close

No sc-lint implementation/release changes, release publication, immutable tag changes, historical Phase C evidence rewrites, or global consumer installation changes.

## Validation

- Run the focused generated consumer regression against the released 0.8.6 binary in an isolated temp environment.
- Run the generated SQLite store test, `test_concurrent_inserts_do_not_lock`, through its existing fixture validation.
- Run `python -m unittest tests.unit.generation.test_consumer_sc_lint_profile tests.unit.generation.test_runtime_ecosystem_upgrade`.
- Run relevant repository lint/doc checks if the focused tests and available disk permit; do not run a full Rust workspace build for this bounded consumer integration and template regression change.

## QA authorization

User approval (2026-11-06): “Approve QA reports and rechecks on PR129.” This
approval covers the sprint findings ARCH-001, EXT-001, QA-001, QA-002,
RBQA-F001, REQ-QA-001, REQ-QA-002, and REQ-QA-003 against the listed owned
paths and review commit SHAs. QA may publish scoped findings and rechecks to
PR129. It excludes source materials, raw logs, credentials, and Beads data.
