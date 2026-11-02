---
id: bc-1
phase: bc
status: planned
closure_type: integration
target_boundary: generated-consumer-lint
track: lint
wave: 1
branch: sprint/bc-1-consumer-sc-lint-full-profile
worktree: sprint/bc-1-consumer-sc-lint-full-profile
requirements:
  - REQ-RUN-0004
  - REQ-RUN-0307
  - REQ-RUN-0503
  - REQ-RUN-0701
  - NFR-RUN-0006
adrs:
  - ADR-RUN-0005
  - ADR-RUN-0401
  - ADR-RUN-0402
owned_paths:
  - template/sc-lint.toml
  - .github/workflows/template-consumer-lint.yml
  - tests/unit/generation/test_consumer_sc_lint_profile.py
  - docs/requirements.md
must_follow: []
parallel_safe: [bc-2]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The generated consumer has no independent lint boundary; its config, bootstrap, standard recipes, and generated-fixture proof are one consumer contract. Splitting them would leave an unprovable configuration-only sprint.
---

# Sprint bc-1 — consumer sc-lint full profile

## Goal

A freshly generated consumer is a real sc-lint consumer, not a fmt/clippy
facade. After `just setup`, its ordinary lint surfaces have a pre-populated
boundary check, cross-platform (`sc-portability`) check, and runtime check.
The standard consumer command names remain `setup`, `lint`, `test`, and
`upgrade`; no manual post-generation installation or copied policy is needed.

## This sprint changes

1. Amend REQ-RUN-0307's consumer-install success criteria to distinguish
   installing the executable from proving the configured analyzers run. The
   amended requirement names `sc-boundary`, `sc-portability`, and
   `sc-runtime`, and retains the prohibition on a template `boundaries/`
   policy from NFR-RUN-0006.
2. Extend `template/sc-lint.toml` from its current fmt/clippy-only profiles.
   It declares named lint entries for boundary, portability, and runtime using
   sc-lint 0.8-compatible invocation syntax, then includes them in both the
   default and full consumer lint profiles. `full` may include additional
   safe consumer checks, but it must be a superset of default.
3. Keep the configuration consumer-neutral. A boundary command must be valid
   for a newly generated repository without shipping a hand-written boundary
   manifest. If sc-lint requires explicit no-manifest behavior, use its
   documented 0.8-compatible configuration—not a no-op wrapper, a suppressed
   failure, or a copied root manifest.
4. Preserve bootstrap idempotence. `just setup` installs the pinned release,
   verifies `sc-lint version --json`, and leaves the template's checked-in
   sources unchanged. A second `sc-lint init --just` still preserves
   `db-prepare` and the configured analyzer entries.
5. Add a focused generation test and a dedicated consumer-lint CI receipt. They generate
   `sqlite-mcp`, run `just setup`, inspect the installed executable directory
   for the three sibling analyzers, and run default and full lint profiles.
   The receipt must demonstrate an invocation of each analyzer rather than
   merely discovering a TOML entry.

## Acceptance criteria

Root: `req:REQ-RUN-0307`, `req:REQ-RUN-0503`.

- A generated `sqlite-mcp` project completes `just setup` with no source
  change, and `sc-lint version --json` reports the consumer's pinned 0.8.x
  version.
- Its installed release contains working `sc-lint`, `sc-lint-boundary`,
  `sc-lint-portability`, and `sc-lint-runtime` executables for the test
  platform.
- `just lint` (the default profile) and `just lint full` both execute the
  boundary, portability, and runtime checks and exit successfully on the
  generated fixture. A test that removes one configured analyzer or profile
  member fails, proving the test is not only parsing TOML.
- `just --list` still exposes `setup`, `lint`, `test`, `upgrade`, and
  `db-prepare`; after `sc-lint init --just`, `db-prepare` and the complete
  configured profile remain present.
- The rendered consumer still contains no `boundaries/`, `clippy.toml`,
  `rustfmt.toml`, or `deny.toml` policy file. Its only lint policy input is
  `sc-lint.toml` plus sc-lint-managed bootstrap content.
- The dedicated consumer-lint CI workflow runs this proof on an actual generated consumer,
  not only against the unrendered `template/` tree.

## Required validation

- Focused Python generation/profile test, including negative fixtures for a
  missing profile member and missing installed sibling binary.
- `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json`, followed by
  `just setup`, `just lint`, `just lint full`, `just test`, `just build-cli`,
  and `just build-daemon` inside `<dest>` with no edit between commands.
- The full existing template fixture matrix and root `just lint && just test`.

## This sprint does not close

- Repository-maintainer-only checks such as line counts, xwin, or the
  repository's boundary manifests. Those remain root policy.
- New analyzer behavior or rule implementation in sc-lint itself.
- RMCP, Utoipa, or package-version changes; `bc-2` owns them.
