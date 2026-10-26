---
id: b-2
phase: b
status: complete
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
  - template/sc-lint.toml
must_follow: []
parallel_safe: [b-1, b-4]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One lint profile. Splitting the host analyzers from the Windows cross-check would make both sprints edit sc-lint.toml.
---

# Sprint b-2 — host lint profile

## Goal

`just lint` runs every installed host analyzer and the Windows cross-check, and it fails on the calls [ADR-RUN-0011](../../architecture.md) names, until [sc-lint#640](https://github.com/randlee/sc-lint/issues/640) replaces the grep. Warnings that fire are fixed. A local allow is a same-line comment that states why that corner case must break the practice. The usual place is a test. There is no profile-wide or file-wide exclusion. The profile is the one `sc-cron`, `store-postgres`, `store-mysql`, and the adaptor will run when those sprints add them.

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

At this sprint's close the two cross commands are package-scoped to `sc-config`, `sc-transport`, `sc-command`, and `sc-runtime`. The recipe keeps the workspace form for later sprints. A failure outside the owned paths is not this sprint's failure.

## Deliverables

1. REQ-RUN-0004, NFR-RUN-0008, and ADR-RUN-0005: regenerate `sc-lint.toml`, re-apply the steps above, and fix every warning the new steps report under the owned paths.
2. NFR-RUN-0002 and ADR-RUN-0011: add the grep step to the `Justfile` recipe that `just lint` already runs.
3. REQ-RUN-0004 and ADR-RUN-0011: declare `cargo-xwin` and run the two cross commands. Fix the warnings they report under the same allow rule.

## Acceptance criteria

Root: `boundary:host-lint-profile` and [ADR-RUN-0011](../../architecture.md).

- The new profile exits 0 on the paths this sprint owns: `crates/sc-config`, `crates/sc-transport`, `crates/sc-command`, `crates/sc-runtime`, `sc-lint.toml`, `Justfile`, `.sc-lint/install.json`, and `template/sc-lint.toml`.
- A violation of ADR-RUN-0011 point 1 under any other path is not this sprint's failure.
- A non-test `tokio::time::sleep` under an owned path fails the grep. `yield_now` does not.
- Warning fixes land only under the owned paths. A warning in a file `b-4` or `b-8` owns is fixed in that sprint.

## Required validation

- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' crates/sc-config crates/sc-transport crates/sc-command crates/sc-runtime` exits 1.
- A non-test `tokio::time::sleep` added under `crates/sc-runtime` fails that command. `yield_now` does not.
- `cargo xwin check -p sc-config -p sc-transport -p sc-command -p sc-runtime --target x86_64-pc-windows-msvc` exits 0 on macOS and on Linux.

## Non-goals

No new sc-lint crate. No phase-complete tag. No edit to `crates/sc-runtime-db-sqlite/`.
