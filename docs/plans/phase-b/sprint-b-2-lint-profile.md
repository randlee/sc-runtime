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
  - REQ-RUN-0005
  - REQ-RUN-0307
  - NFR-RUN-0002
  - NFR-RUN-0004
  - NFR-RUN-0006
  - NFR-RUN-0008
  - NFR-RUN-0009
  - REQ-CFG-0001
  - REQ-CFG-0002
  - REQ-CFG-0003
  - REQ-CFG-0004
  - NFR-CFG-0001
  - NFR-CFG-0002
  - NFR-CFG-0003
  - REQ-TRN-0001
  - REQ-TRN-0002
  - REQ-TRN-0003
  - REQ-TRN-0004
  - REQ-TRN-0005
  - REQ-TRN-0006
  - NFR-TRN-0001
  - NFR-TRN-0002
  - NFR-TRN-0003
  - NFR-TRN-0004
  - REQ-CMD-0001
  - REQ-CMD-0002
  - REQ-CMD-0003
  - REQ-CMD-0004
  - REQ-CMD-0005
  - REQ-CMD-0006
  - NFR-CMD-0001
  - NFR-CMD-0002
  - NFR-CMD-0003
  - REQ-RT-0001
  - REQ-RT-0002
  - REQ-RT-0003
  - REQ-RT-0004
  - REQ-RT-0005
  - REQ-RT-0006
  - REQ-RT-0007
  - REQ-RT-0008
  - NFR-RT-0001
  - NFR-RT-0002
  - NFR-RT-0003
  - NFR-RT-0004
adrs:
  - ADR-RUN-0005
  - ADR-RUN-0007
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

[NFR-RUN-0002](../../requirements.md) and [ADR-RUN-0007](../../architecture.md) forbid blocking a Tokio worker on a request path. `NFR-RUN-0002` still allows `std::fs` on the bind and shutdown paths in `sc-runtime` and `sc-transport`. This sprint does not ban those calls. The grep is the names in ADR-RUN-0011 point 1, plus the method form `.block_on(`. Criterion 5 of NFR-RUN-0002 names the fixture `runtime.block_on` in `crates/sc-runtime/src/testing.rs`: it drives `run` on the fixture thread and is not a connection handler. That call keeps a same-line `allow:`.

## Grep

The full profile step `adr-run-0011` runs `just adr-run-0011`, which runs `scripts/adr_run_0011_grep.py` with `{{python}}` (`python3` on Unix, `python` on Windows). The recipe lives in the repo-specific section of the `Justfile`, so `sc-lint init --just` does not drop it. Re-apply the `adr-run-0011` profile step with the other maintainer steps after a regenerate.

The scanner searches `crates/` (including `sc-runtime-db-sqlite` once that crate is on the branch) and `template/` for `std::thread::sleep`, `std::thread::sleep_until`, `tokio::time::sleep`, `tokio::time::sleep_until`, `Runtime::block_on`, `Handle::block_on`, `futures::executor::block_on`, and `.block_on(`. A match outside `#[cfg(test)]` and outside a same-line `allow:` comment fails. `yield_now` is not a match. A call written as `sleep(...)` after a `use` import is not this grep. [ADR-RUN-0011](../../architecture.md) point 5 stays with arch-qa until sc-lint#640. Delete this step when the pinned sc-lint release that closes sc-lint#640 is in `minimum_version`. That deletion is not this sprint.

Handler deadlines use `tokio::time::timeout_at` on the handler future. A separate timer future whose only job is to dodge the banned names is not an allow.

## Windows cross-check

`.sc-lint/install.json` declares `cargo-xwin` and the rustup target `x86_64-pc-windows-msvc`. `just setup` installs them. `just lint` on macOS and on Ubuntu runs the full profile, which includes:

- `cargo xwin check --workspace --exclude sc-runtime-db-sqlite --target x86_64-pc-windows-msvc`
- `cargo xwin clippy --workspace --exclude sc-runtime-db-sqlite --all-targets --target x86_64-pc-windows-msvc -- -D warnings -D clippy::await_holding_lock`

`sc-runtime-db-sqlite` is excluded until that crate cross-compiles. Later sprints drop the exclude and keep `--workspace`. A failure in an excluded crate is not this sprint's failure.

The clippy step `just lint` already runs passes `-D clippy::await_holding_lock` as well. That is [ADR-RUN-0011](../../architecture.md) point 2. arch-qa still reviews a guard the lint cannot name.

`template/sc-lint.toml` denies `clippy::await_holding_lock` on the consumer clippy step. That flag sits in the sc-lint command [REQ-RUN-0307](../../requirements.md) already owns. [NFR-RUN-0006](../../requirements.md) still forbids a `clippy.toml`, a `[lints]` table, and any other lint file under `template/`.

On a non-Unix target, `bind` of a UDS endpoint returns `TransportError::UnsupportedEndpoint` (`TRANSPORT.UNSUPPORTED_ENDPOINT`). Recovery tells the caller to use a `tcp:` endpoint. That is the early return in `bind_uds`; Windows does not build the Unix lock helpers.

## Deliverables

1. REQ-RUN-0004, NFR-RUN-0008, and ADR-RUN-0005: regenerate `sc-lint.toml`, re-apply the steps above, and fix every warning the new steps report under the owned paths.
2. NFR-RUN-0002, ADR-RUN-0007, and ADR-RUN-0011: register `adr-run-0011` on the full profile. The scanner is `scripts/adr_run_0011_grep.py`.
3. REQ-RUN-0004 and ADR-RUN-0011: declare `cargo-xwin` and run the two cross commands. Fix the warnings they report under the same allow rule.

## Acceptance criteria

Root: `boundary:host-lint-profile` and [ADR-RUN-0011](../../architecture.md).

- The new profile exits 0 on the paths this sprint owns: `crates/sc-config`, `crates/sc-transport`, `crates/sc-command`, `crates/sc-runtime`, `sc-lint.toml`, `Justfile`, `.sc-lint/install.json`, and `template/sc-lint.toml`.
- A violation of ADR-RUN-0011 point 1 under any other path is not this sprint's failure.
- A non-test `tokio::time::sleep` or `.block_on(` under an owned path fails `adr-run-0011`. `yield_now` does not. A same-line `allow:` is the only exemption outside `#[cfg(test)]`.
- Warning fixes land only under the owned paths. A warning in a file `b-4` or `b-8` owns is fixed in that sprint.

## Required validation

- `just adr-run-0011` exits 0 on this tree. A probed `.block_on(` under `crates/*/src` without a same-line allow fails that command. `yield_now` does not.
- `cargo xwin check --workspace --exclude sc-runtime-db-sqlite --target x86_64-pc-windows-msvc` exits 0 on macOS and on Linux. Ubuntu evidence is `just lint` in crate CI, which runs the full profile after `just setup`. On commit `1118f8c1fb2cb92cca7a64c00b0d6e4c561edbe9` that step exited 0 on `ubuntu-latest` in [Actions run 40282316408](https://github.com/randlee/sc-runtime/actions/runs/40282316408), job `just lint and just test`, step Lint, after `just setup`. That profile includes `xwin-check` and `xwin-clippy`. macOS evidence is `just lint full` on this worktree.

## Non-goals

No new sc-lint crate. No phase-complete tag. No edit to `crates/sc-runtime-db-sqlite/`.
