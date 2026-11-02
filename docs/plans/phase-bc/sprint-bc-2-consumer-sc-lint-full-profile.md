---
id: bc-2
phase: bc
status: planned
closure_type: integration
target_boundary: generated-consumer-lint
track: lint
wave: 2
branch: sprint/bc-2-consumer-sc-lint-full-profile
worktree: sprint/bc-2-consumer-sc-lint-full-profile
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
must_follow: [bc-1]
parallel_safe: [bc-3]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The generated consumer has no independent lint boundary; its config, bootstrap, standard recipes, and generated-fixture proof are one consumer contract. Splitting them would leave an unprovable configuration-only sprint.
---

# Sprint bc-2 — consumer sc-lint full profile

## Goal

A freshly generated consumer is a real **sc-lint 0.8.4** consumer, not a fmt/clippy
facade. After `just setup`, its ordinary lint surfaces have a pre-populated
boundary check, cross-platform (`sc-portability`) check, and runtime check.
The standard consumer command names remain `setup`, `lint`, `test`, and
`upgrade`; no manual post-generation installation or copied policy is needed.

## This sprint changes

1. Implement the binding REQ-RUN-0307/NFR-RUN-0006/ADR-RUN-0005 amendment
   produced by `bc-1`; this sprint does not reinterpret it. The exception is
   limited to sc-lint's declared consumer profile and never permits a template
   `boundaries/`, clippy, rustfmt, or deny policy.
2. Extend `template/sc-lint.toml` from its current fmt/clippy-only profiles
   using the exact 0.8.4 contract below. Both `default` and `full` contain all
   three analyzer entries; `full` is a superset of `default`. The bootstrap's
   ordinary no-selector `just lint` path must select a profile containing the
   same three entries.

   ```toml
   [tool.sc-lint]
   minimum_version = "0.8.4"

   [[tool.sc-lint.lint]]
   name = "sc-boundary"
   command = ["sc-lint", "lint", "sc-boundary", "--root", ".", "--config", "sc-lint.toml"]

   [[tool.sc-lint.lint]]
   name = "sc-portability"
   command = ["sc-lint", "lint", "sc-portability", "--root", ".", "--config", "sc-lint.toml"]

   [[tool.sc-lint.lint]]
   name = "sc-runtime"
   command = ["sc-lint", "lint", "sc-runtime", "--root", ".", "--config", "sc-lint.toml"]

   [tool.sc-lint.lint_profiles]
   default = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]
   full = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]
   ```

3. `bc-1` records whether the supported 0.8.4 boundary analyzer succeeds with
   no consumer manifest or needs a documented consumer-neutral mode. This
   sprint uses that exact mode. It must not substitute a no-op wrapper,
   suppress a failure, or copy this repository's manifest.
4. Preserve bootstrap idempotence. `just setup` installs the pinned release,
   verifies `sc-lint version --json`, and leaves the template's checked-in
   sources unchanged. A second `sc-lint init --just` still preserves
   `db-prepare` and the configured analyzer entries.
5. Add a focused generation test and a dedicated consumer-lint CI receipt. They
   generate `sqlite-mcp`, run `just setup`, verify the single installed
   `sc-lint` launcher reports 0.8.4, and run ordinary/default/full lint
   profiles. The receipt must demonstrate the three analyzer subcommand
   invocations rather than merely discovering TOML entries.

## Acceptance criteria

Root: `req:REQ-RUN-0307`, `req:REQ-RUN-0503`.

- A generated `sqlite-mcp` project completes `just setup` with no source
  change, and `sc-lint version --json` reports **0.8.4**.
- Its installed release contains the working `sc-lint` launcher. The analyzer
  contract is the three recorded subcommand invocations, not sibling binaries.
- `just lint`, `just lint default`, and `just lint full` all emit the
  bootstrap receipt lines `lint.sc-boundary: running`,
  `lint.sc-portability: running`, and `lint.sc-runtime: running`, and exit
  successfully on the generated fixture. A test that removes one configured
  analyzer or profile member fails, proving the receipt is not only parsing
  TOML.
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
  missing profile member and missing configured analyzer command.
- `just new <dest> --var-file wizard/fixtures/sqlite-mcp.json`, followed by
  `just setup`, `just lint`, `just lint full`, `just test`, `just build-cli`,
  and `just build-daemon` inside `<dest>` with no edit between commands.
- The full existing template fixture matrix and root `just lint && just test`.

## This sprint does not close

- Repository-maintainer-only checks such as line counts, xwin, or the
  repository's boundary manifests. Those remain root policy.
- New analyzer behavior or rule implementation in sc-lint itself.
- RMCP, Utoipa, or package-version changes; `bc-3` owns them.
