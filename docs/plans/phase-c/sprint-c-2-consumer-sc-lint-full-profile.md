---
id: c-2
phase: c
status: planned
closure_type: integration
target_boundary: generated-consumer-lint
track: lint
wave: 2
branch: sprint/c-2-consumer-sc-lint-full-profile
worktree: sprint/c-2-consumer-sc-lint-full-profile
requirements:
  - REQ-RUN-0004
  - REQ-RUN-0301
  - REQ-RUN-0307
  - REQ-RUN-0503
  - REQ-RUN-0701
  - NFR-RUN-0006
adrs:
  - ADR-RUN-0005
  - ADR-RUN-0303
  - ADR-RUN-0401
  - ADR-RUN-0402
owned_paths:
  - template/sc-lint.toml
  - template/boundaries/
  - template/.sc-lint/bootstrap.py
  - template/Justfile
  - template/justfile
  - tests/unit/generation/test_consumer_sc_lint_profile.py
must_follow: [c-1]
parallel_safe: [c-3, c-5, c-6]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: The generated consumer has no independent lint boundary; its config, bootstrap, standard recipes, and generated-fixture proof are one consumer contract. Splitting them would leave an unprovable configuration-only sprint.
---

# Sprint c-2 — consumer sc-lint full profile

Dependency execution: consume `c-1`'s REQ-RUN-0307/NFR-RUN-0006 and
ADR-RUN-0005 contract amendments by merging each parent development push
before every development/fix round. Parent QA is not a start barrier;
the parent PR merges before this PR completes.

## Goal

A freshly generated consumer is a real **sc-lint 0.8.4** consumer, not a fmt/clippy
facade. After `just setup`, its ordinary lint surfaces have a pre-populated
boundary check, cross-platform (`sc-portability`) check, and runtime check.
The standard consumer command names remain `setup`, `lint`, `test`, and
`upgrade`; no manual post-generation installation or copied policy is needed.

## This sprint changes

1. Implement the binding REQ-RUN-0307/NFR-RUN-0006/ADR-RUN-0005 amendment
   produced by `c-1`; this sprint does not reinterpret it. The exception is
   limited to sc-lint's declared consumer profile and the fresh-install
   boundary policy derived from selected project configuration. It never
   permits copying the maintainer boundary policy or adding clippy, rustfmt,
   or deny policy.
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

3. Render `template/boundaries/` from the c-1 configuration-to-policy mapping
   for the generated application crate graph. Only selected crates and
   allowed edges appear; disabled stores and MCP components do not. Run the
   supported 0.8.4 analyzer against that policy. On completion of setup the
   policy belongs to the consumer; subsequent setup/init/upgrade preserves
   consumer edits. It must not substitute a no-op wrapper, suppress a failure,
   or copy this repository's maintainer manifests.
4. Preserve bootstrap idempotence. `just setup` installs the pinned release,
   verifies `sc-lint version --json`, and leaves the template's checked-in
   sources unchanged. A second `sc-lint init --just` still preserves
   `db-prepare` and the configured analyzer entries.
5. Add a focused generation regression executed through the existing test and
   template-fixture CI venues. Generate `sqlite-mcp`, run `just setup`, verify the single installed
   `sc-lint` launcher reports 0.8.4, and run ordinary/default/full lint
   profiles. Existing CI logs must demonstrate actual analyzer subcommand
   execution rather than merely discovering TOML entries; no extra workflow
   or separate receipt artifact is required.

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
- The rendered consumer contains the configuration-derived fresh boundary
  policy and no `clippy.toml`, `rustfmt.toml`, or `deny.toml`. Tests cover
  SQLite-only and combined store selections, with MCP on/off, asserting the
  generated policy follows actual selected crates/edges. A consumer policy
  edit survives repeated setup/init/upgrade. Removing an allowed-edge rule or
  adding a forbidden dependency is detected by the actual boundary analyzer.
- The focused regression and existing template fixture CI execute this proof
  on an actual generated consumer, using their ordinary logs for invocation
  evidence rather than adding a dedicated consumer-lint workflow.

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
- RMCP, Utoipa, or package-version changes; `c-3` owns them.
