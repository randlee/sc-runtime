# Decision: sync scheduling stays at the bd-cache API

**Status:** Settled for where the rule lives. The keep-versus-relax product question is open and still needs owner sign-off.  
**Date:** 2026-10-26  
**Bead:** `run-zif.2` (parent `run-zif`)  
**Owner intent:** 2026-10-26

This record is the decision for `run-zif.2`. It names no change to `crates/sc-cron` and no change to PR #68.

## Settled

1. **REQ-BDC-0720's two-slot mutex and REQ-BDC-0722's lease are bd-cache public API policy.** beads-cache sync is at most one built-in Rust row and one consumer Python row per repository, at most one of them enabled. `bdc cache schedule set`, `schedule enable`, `schedule disable`, `schedule clear`, `sync use-python`, and `sync use-builtin` are the commands that keep that rule. Every sync run takes the one `tick_lease` row (`REQ-BDC-0722`) before it starts. Those commands call sc-cron primitives (`replace_and_set_enabled`, `set_enabled_many`, and the per-name in-flight skip). The mutex and the lease are not sc-cron rules.
2. **sc-cron stays a general scheduler.** Many schedule rows. Rows are added and replaced at runtime. A table may mix hook targets. Two different schedule names may run at the same time. The only skip inside the scheduler is a tick that comes due while that same schedule name is still in flight (beads-cache `REQ-CRON-0006`). sc-cron does not grow a two-row mutex, a global in-flight lock, or a beads-specific schedule limit.
3. **Monty is out of scope for beads-cache.** beads-cache sync stays a Rust built-in job or a consumer CPython script. beads-cache `REQ-CRON-0016` keeps that crate's target set at Rust or Python. An in-process Monty hook is an sc-runtime follow-on ([sprint-b-9-sc-cron-monty-hooks.md](sprint-b-9-sc-cron-monty-hooks.md)), after the b-3 port merges.

The reading that "replace Rust with Python" limits sc-cron itself to those two sync rows is rejected. That sentence is the bd-cache switch between its two designated rows.

## What the API enforces today

Quoted scope, from beads-cache `docs/requirements.md` (approved 2026-10-26). This planning sprint does not edit that text.

`REQ-BDC-0720` — at most one built-in sync row and one Python sync row; at most one enabled. `schedule enable` while the Python row is enabled is `SCHEDULE_INVALID` (`details.reason: "python_sync_enabled"`). `schedule set` writes through `replace_and_set_enabled` so a row written enabled disables the Python row in the same transaction. While the Python row is enabled, `bdc cache tick` and `Cache.tick()` are refused with `PRECONDITION` (`details.reason: "python_sync_enabled"`).

`REQ-BDC-0722` — one `tick_lease` row in the cache file, taken under `BEGIN IMMEDIATE`, expiring at the run's deadline plus 10 seconds, released on every exit path. A scheduled or on-demand run that finds a live lease replies `skipped: "in_flight"`. `sync use-python` fails with `TICK_IN_FLIGHT`. `sync use-builtin` takes no lease.

`REQ-BDC-0724` — `sync use-python` takes the lease, then in one transaction writes the one Python row enabled and disables the built-in row. There is never a second Python sync row. `sync use-builtin` makes the built-in row the enabled one in one transaction.

## What sc-cron provides

The scheduler primitives those commands already use:

- `ScheduleStore::set_enabled_many` and `Scheduler::set_enabled_many` (beads-cache `REQ-CRON-0017`)
- `ScheduleStore::replace_and_set_enabled` and `Scheduler::replace_and_set_enabled` (`REQ-CRON-0017`)
- one run in flight per schedule name (`REQ-CRON-0006`)
- generation fencing so a replaced row's old run cannot write (`REQ-CRON-0007`)

`REQ-CRON-0006` is per schedule name, inside one scheduler. `REQ-BDC-0722` is one lease for every sync run in the cache file, across processes, whichever row is enabled. The crosswalk keeps them apart: [sc-cron-req-crosswalk.md](sc-cron-req-crosswalk.md).

## Open question

Owner sign-off still has to choose one of these. This record does not choose.

- **Keep** mutual enable as `REQ-BDC-0720` writes it: one built-in row, one Python row, at most one enabled, switched by `schedule enable` / `schedule disable`, `sync use-python`, and `sync use-builtin`, with the single `tick_lease`.
- **Relax** the product rule. Examples of a relaxation: more than two sync rows, both of the current rows enabled, or a different switch. A relaxation is still implemented in bd-cache, using the existing sc-cron primitives (`set_enabled_many`, `replace_and_set_enabled`).

Either answer names no sc-cron crate change. If the signed answer changes `REQ-BDC-0720` text, that edit is filed in the beads-cache repo and linked from `run-zif.2`. It is not an edit in this repo.

## Out of scope

- `crates/sc-cron`, PR #68, and bead `run-9ny`.
- A Monty hook target. beads-cache does not take one.
- Implementing a beads-cache patch from this repo.

## Consequences

No sc-cron code moves because of this decision. The b-3 port copies a general scheduler. beads-cache keeps enforcing its sync shape at the bd-cache commands above. `run-zif.2` stays open until the owner signs keep or relax.
