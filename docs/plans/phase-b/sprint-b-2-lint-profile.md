---
id: b-2
phase: b
status: planned
closure_type: boundary
target_boundary: host-lint-profile
track: lint
wave: 1
branch: sprint/b-2-lint-profile
worktree: sprint/b-2-lint-profile
requirements:
  - REQ-RUN-0004
  - NFR-RUN-0002
  - NFR-RUN-0008
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0005
  - ADR-RUN-0011
owned_paths:
  - sc-lint.toml
  - .sc-lint/install.json
  - Justfile
  - crates/sc-config/
  - crates/sc-transport/
  - crates/sc-command/
  - crates/sc-runtime/
  - template/
must_follow: []
parallel_safe: [b-1]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One lint profile. Splitting the host analyzers from the Windows cross-check would make both sprints edit sc-lint.toml.
---

# Sprint b-2 — host lint profile

## Goal

`just lint` runs every installed host analyzer and the Windows cross-check, and it fails on the calls [ADR-RUN-0011](../../architecture.md) names, until [sc-lint#640](https://github.com/randlee/sc-lint/issues/640) replaces the grep. Warnings that fire are fixed. A local allow is a same-line comment that states why that corner case must break the practice. The usual place is a test. There is no profile-wide or file-wide exclusion.

## Profile

`sc-lint.toml` is managed. Regenerate with `sc-lint init --just`, then re-apply the maintainer steps below. Do not raise `minimum_version`. Do not vendor an sc-lint-runtime rule.

Add these steps to the full profile, with `exclude_rules` empty and `fail_rules` equal to each analyzer's rule set:

- `sc-portability` (PORT-001 through PORT-010)
- `sc-runtime` (SCB-RUNTIME-001, SCB-RUNTIME-002)
- function-length
- identity-literals
- `sc-lint compatibility check`

`NFR-RUN-0002` still allows `std::fs` on the bind and shutdown paths in `sc-runtime` and `sc-transport`. This sprint does not ban those calls. The grep is only the names in ADR-RUN-0011 point 1.

## Grep

A `just lint` step searches `crates/` (including `sc-runtime-db-sqlite` once that crate is on the branch) and `template/` for `std::thread::sleep`, `std::thread::sleep_until`, `tokio::time::sleep`, `tokio::time::sleep_until`, `Runtime::block_on`, `Handle::block_on`, and `futures::executor::block_on`. A match outside `#[cfg(test)]` and outside a same-line allow comment fails. `yield_now` is not a match. A call written as `sleep(...)` after a `use` import is not this grep. [ADR-RUN-0011](../../architecture.md) point 5 stays with arch-qa until sc-lint#640. Delete this step when the pinned sc-lint release that closes sc-lint#640 is in `minimum_version`. That deletion is not this sprint.

## Windows cross-check

`.sc-lint/install.json` declares `cargo-xwin` and the rustup target `x86_64-pc-windows-msvc`. The full profile runs, once on macOS and once on Linux:

- `cargo xwin check --workspace --target x86_64-pc-windows-msvc`
- `cargo xwin clippy --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings -D clippy::await_holding_lock`

The clippy step `just lint` already runs passes `-D clippy::await_holding_lock` as well. That is [ADR-RUN-0011](../../architecture.md) point 2. arch-qa still reviews a guard the lint cannot name.

## Tasks

1. ADR-RUN-0005: regenerate `sc-lint.toml`, re-apply the steps above, and fix every warning the new steps report under the owned paths.
2. ADR-RUN-0011: add the grep step to the `Justfile` recipe that `just lint` already runs.
3. ADR-RUN-0011: declare `cargo-xwin` and run the two cross commands. Fix the warnings they report under the same allow rule.

## Required validation

- `just lint` exits 0 on macOS and on Linux.
- A non-test `tokio::time::sleep` added under `crates/` fails the grep. `yield_now` does not.

## Non-goals

No new sc-lint crate. No phase-complete tag. No edit to `crates/sc-runtime-db-sqlite/`.
