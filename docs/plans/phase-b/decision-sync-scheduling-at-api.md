# Decision: sync scheduling stays at the bd-cache API

**Status:** Settled. Owner ruled 2026-10-26: no schedule slot cap in sc-cron or as a permanent beads-cache scheduler limit.  
**Date:** 2026-10-26  
**Bead:** `run-zif.2` (parent `run-zif`)  
**Owner intent:** 2026-10-26

This record is the decision for `run-zif.2`. It names no change to `crates/sc-cron` and no change to PR #68.

## Settled

1. **REQ-BDC-0720's two-row ceiling is temporary beads-cache behavior, not lasting scheduling policy.** Today the bd-cache CLI still enforces at most one built-in sync row and one Python sync row, at most one enabled (`What the API enforces today`, below). That shape is **not** an acceptable permanent limit on the schedule table; bead **`run-zif.2`** amends beads-cache requirements and CLI text toward the [owner ruling](#owner-ruling-no-slot-restriction-2026-10-26). **`REQ-BDC-0722`** (`tick_lease`) stays: one sync run at a time across the cache file is cross-process concurrency control, not a row cap. Those commands call sc-cron primitives (`replace_and_set_enabled`, `set_enabled_many`, and the per-name in-flight skip). Neither the temporary two-row ceiling nor the lease is an sc-cron rule.
2. **sc-cron stays a general scheduler.** Many schedule rows. Rows are added and replaced at runtime. A table may mix hook targets. Two different schedule names may run at the same time. The only skip inside the scheduler is a tick that comes due while that same schedule name is still in flight (beads-cache `REQ-CRON-0006`). sc-cron does not grow a two-row mutex, a global in-flight lock, or a beads-specific schedule limit.
3. **Monty is out of scope for beads-cache.** beads-cache sync stays a Rust built-in job or a consumer CPython script. beads-cache `REQ-CRON-0016` keeps that crate's target set at Rust or Python. An in-process Monty hook is an sc-runtime follow-on ([sprint-b-9-sc-cron-monty-hooks.md](sprint-b-9-sc-cron-monty-hooks.md)), after the b-3 port merges.

The reading that "replace Rust with Python" limits sc-cron itself to those two sync rows is rejected. That sentence is the bd-cache switch between its two designated rows.

## What the API enforces today

**Current beads-cache behavior**, subordinate to the [owner ruling](#owner-ruling-no-slot-restriction-2026-10-26). Quoted scope from beads-cache `docs/requirements.md` (approved 2026-10-26). This planning sprint does not edit that text.

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

## Owner ruling: no slot restriction (2026-10-26)

The scheduler must not cap how many schedule rows a consumer may hold or enable. sc-cron stays a general table: rows are added and replaced at runtime; mixed targets; per-name in-flight skip only (`REQ-CRON-0006`).

beads-cache **REQ-BDC-0720** (“at most one built-in row and one Python row; at most one enabled”) is **not** an acceptable permanent shape for scheduling. It may remain as a **temporary** bd-cache implementation detail until beads-cache is amended, but the product direction is:

- Sync fallback (Rust built-in vs consumer Python) is **commands and row semantics**, not a hard two-slot ceiling on the schedule table.
- Any mutual exclusion needed for sync (for example “only one sync pipeline active”) is enforced with **named rows, enable flags, and transactions** (`replace_and_set_enabled`, `set_enabled_many`) — not by forbidding a third cron row for other work.

**REQ-BDC-0722** (`tick_lease`) may stay: one sync run at a time across the cache file is concurrency control, not a slot limit.

Follow-up: amend **REQ-BDC-0720** (and related CLI text) in the **beads-cache** repo; link from bead `run-zif.2`. No sc-cron crate change required for this ruling.

## Out of scope

- `crates/sc-cron`, PR #68, and bead `run-9ny`.
- A Monty hook target. beads-cache does not take one.
- Implementing a beads-cache patch from this repo.

## Consequences

No sc-cron code moves because of this decision alone. The b-3 port copies a general scheduler. beads-cache must align its requirements and API with the no-slot-cap ruling in a separate beads-cache change set.
