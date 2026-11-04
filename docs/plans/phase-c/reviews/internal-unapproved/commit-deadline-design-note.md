# Commit/deadline linearization and outcome design note

Read-only design assist for actual critical finding **COMPAT-CRIT-002**, reviewed baseline `0b32ae20f26c9a4db6fc67c98b7331724aeacf3c`. Read the completed critical-R1 FAIL report. This note does not amend that fixed baseline, close a finding, certify cooperative capability or claim a test ran. No source/plan/Git/Beads mutation, Cargo build or live call.

## What the supported APIs actually guarantee

Cached primary source root `/Users/randlee/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`:

- `rusqlite-0.39.0/src/transaction.rs:186-193`: Transaction::commit is a separate `execute_batch("COMMIT")`. A check before the preceding UPDATE does not guard this terminal statement.
- `rusqlite-0.39.0/src/hooks/mod.rs:344-358`: **commit_hook**, enabled by already selected `hooks`, takes `FnMut()->bool + Send+'static`; true requests rollback. This is a supported precommit guard for explicit and implicit transactions, not a physical-completion deadline.
- `libsqlite3-sys-0.37.0/sqlite3/sqlite3.h:7001-7037`: nonzero commit-hook return converts commit to rollback. The hook cannot run SQL/prepare/step or otherwise modify the invoking connection; it is not reentrant.
- `.../sqlite3.c:89810-89850`: the commit callback runs **before** BtreeCommitPhaseOne/PhaseTwo; a zero/allow result lets those phases proceed. No promise is made that stop/deadline can revoke an already allowed commit atomically.
- `.../sqlite3.h:2886-2917`: interrupt aborts at the earliest opportunity; a nearly finished operation may complete. Interrupt while no SQL is running is a no-op. Progress/busy/interrupt do not create a permanent fence on future statements.
- `.../sqlite3.c:26949-26952,65312-65329`: SQLite sync calls the VFS xSync directly; this API does not provide a deadline/cancellation argument or a proof that a blocked OS sync will return by G/K.
- `rusqlite-0.39.0/src/transaction.rs:219-245`: finalization skips rollback when the connection is already autocommit; Drop ignores the finish result. Dropping Transaction is not evidence that a previously successful commit was undone, or that rollback succeeded.
- `.../sqlite3.h:6833-6847`: some errors can automatically roll back a transaction; get_autocommit determines whether a transaction remains active. Autocommit alone does not identify whether a completed transaction committed or rolled back.

Thus the current literal phrases “no late ordinary commit” and “at K it cannot ... commit” lack a physical completion proof. A commit hook closes the **admission race**; it cannot promise to undo a physically completed transaction or preempt arbitrary commit/fsync native work. Holding the stop-state mutex across COMMIT would merely block second-stop/deadline propagation and is unacceptable.

## Precise implementable admission contract

Propose making the linearization point explicit, subject to the author's correction and independent review. This is an explicit clarification/tradeoff, **not** an assertion that the stricter physical guarantee already passed qualification.

1. The live controller state serializes earliest G/K, stop phase, epoch and operation class. First stop closes **new operation and launch admission** immediately; already admitted ordinary operations may drain until G. Restricted cleanup is a separately admitted class for stopped id/generation bookkeeping until K only.
2. Every transaction installs the owned-scope commit hook. At the hook's final admission decision, acquire only the short controller-state mutex, compare the same real Instant and live cutoff, recheck class/generation eligibility and cancel state, and record the operation's commit-admission decision/epoch. Release this mutex **before** returning to SQLite and before any commit phase/native I/O. No SQL/connection mutation in the hook.
3. If stop/deadline/shortening wins that serialization point, veto the commit. If commit admission wins before its cutoff, the commit is *logically admitted* even if its physical completion/visible result is later. Any second stop still latches cancellation, wakes lock/busy waiters and interrupts active SQL immediately; it may cause this admitted commit to fail, but may not revoke a commit SQLite has already completed.
4. Ordinary transactions cannot receive new commit admission at or after G; cleanup cannot receive new commit admission at or after K. Implicit autocommit writes need the same guard; otherwise an UPDATE outside Transaction escapes the fence. Initial checks, row loops and busy/progress/interrupt remain necessary before reaching the hook. No retry after cancellation/cutoff and no reset of the deadline after lock acquisition.
5. The admitted operation and its real blocking closure remain registered until SQLite returns, transaction outcome is handled, scope/TLS/hooks/settings restore and physical work joins. The stop controller never waits for that completion to deliver cancellation/expiry. It remains independently observable if completion overruns the budget.
6. A launch requires a separate final live launch-admission check after persistence. An earlier admitted/late-completing schedule write does not authorize launching a job after stop. Once the owner reports **physical release/join complete**, no task, callback, deferred retry, pending transaction or retained operation remains that can write or launch. This is the hard “no late writes/launches after owned release” fence; report release only when it is true.

Commit admission is not a host security certificate and not a new user-visible registry. It is the internal serialization of the existing operation and stop state. The known owner of the connection must install/restore its known commit hook alongside progress/busy hooks; supported APIs cannot recover arbitrary foreign previous callbacks. Nested/forwarding consumer scopes must participate explicitly or reject unsupported nesting.

## Outcomes must preserve truth after physical join

At least distinguish these typed internal outcomes; freeze their external error/report representation in c-1:

| Outcome | What may be reported |
| --- | --- |
| Not admitted / vetoed | No commit was admitted. Report fixed cancellation/deadline failure. Mark rollback confirmed only if SQLite's result and explicit cleanup establish it. |
| Commit succeeded before cutoff | Confirmed committed, with ordinary success only if the operation's other contract conditions also hold. |
| Commit admitted before cutoff, succeeded after cutoff | **Confirmed committed late**, plus grace/cleanup deadline failure as applicable. Never claim rollback or “not persisted” simply because the timer fired. This fails any strict physical-completion-by-cutoff criterion. |
| Commit failed, rollback explicitly confirmed | Rolled back; fixed typed cause/recovery and deadline state. Do not rely only on Drop. |
| Commit/rollback outcome uncertain | Fixed typed indeterminate-store outcome; do not replay the mutation automatically or claim committed/not committed. Retain/quarantine unsafe connection state under ownership until cleanup completes. |

If the API keeps `cleanup_persisted: bool`, define true only for an actually confirmed commit; pair it with explicit `cleanup_exceeded` and persistence outcome so a late confirmed commit cannot be disguised as successful bounded cleanup. If false is intended to mean “not persisted,” replace it with an outcome enum: unknown is not false. A missed transport deadline likewise does not prove a mutation did not happen.

Readback is permitted only as a bounded, authorized reconciliation operation; it cannot silently reopen an expired worker context. If no budget remains at K, report indeterminate instead of adding time or issuing an unbounded read. A later separate authorized read can report observed id/generation/state. Without a durable idempotency identity, identical current rows or get_autocommit do not prove which request committed; generation fencing prevents stale overwrite but is not a general exactly-once/replay token. Do **not** add a new schema/idempotency table merely to hide this issue. Preserve the existing table/version/fencing contract and default to a typed unresolved outcome when proof is unavailable.

For create/restart/end/replace-and-toggle, never automatically retry an uncertain transaction: a replay might create a second generation or affect a newer row. Consumer lease cleanup may legitimately fall back to its documented expiry when no budget remains; it must not claim an uncertain release succeeded.

## Causal proof matrix for c-1 qualification / c-5 implementation

Use actual rusqlite transactions plus private test gates around the **real commit hook decision**, not a simulated boolean commit. Keep the connection/operation owner until joins finish. Inspect persisted state from a separate connection after physical completion.

1. Pause after the wrapper's final pre-COMMIT check but **before** hook admission. Expire G/K or issue second stop while paused; release the gate. Hook must read updated state and veto; verify rows/generation unchanged and accurate cancellation/rollback outcome. This catches a stale precheck-only implementation.
2. Race stop and hook admission using a barrier and the same serialized state. Test both orderings. Stop-first must veto. Admission-first must record that classification; do not assert cancellation forcibly undoes an admitted SQLite commit.
3. Pause inside the hook **after the allow decision** but before returning to SQLite, expire/shorten, then resume. This demonstrates the final check-to-physical-commit window. Record SQLite's actual return and persisted state; require accurate committed-late versus rollback classification, never an unconditional “timed out therefore rolled back.” The gate must not hold the controller mutex, and expiry/second-stop acknowledgement must occur while the commit path is paused.
4. Repeat ordinary and restricted-cleanup transactions, including implicit write transactions, generation-fenced updates and atomic multi-row changes. The stop/cleanup class is not inferred from arbitrary SQL text; it is controlled by the owned operation scope.
5. Cause real SQLITE_BUSY/SQLITE_INTERRUPT/error paths where safely reproducible. Verify explicit transaction state/rollback handling and no automatic mutation replay. An injected “result delivery lost/indeterminate” adapter test must not pretend an actual I/O uncertainty was physically reproduced; it tests the classification path only.
6. Hold connection acquisition, busy SQL and decode stages while first/second stop occurs; unchanged existing cancellation tests remain required. After actual owned release, release every planted external gate and assert no later writes, job/bd calls, callbacks or retained logger/instance owner. Expiry notification alone is not physical release.
7. Confirm real-clock T>0 completion-by-K under the supported cooperative conditions and reject unsupported providers. For T=0, G=K=s means immediate cancellation/no new commit admission or cleanup budget, not a fabricated positive C0 or instantaneous physical join. A commit admitted before the zero-time stop may still finish; classify it honestly and retain ownership.

Custom VFS/fsync fault injection is not assumed available through safe rusqlite APIs in this crate. The cached source is already sufficient to establish the lack of a generic interruptible-xSync deadline guarantee; do not add unsafe production FFI, filesystem settings that weaken durability, or broad timeout tolerances to manufacture one.

## Capability gap that must stay explicit

There is no source-supported general proof of **physical no-commit-after-G/K** or **physical join by 2T for arbitrary storage/native fsync** using busy/progress/interrupt/commit_hook alone. Actual cooperative SQLite tests can qualify their supported scenarios and admission fences; they cannot turn that missing guarantee into a universal statement.

The proposed admission-linearization rule preserves immediate stop/second-stop propagation and the hard no-effects-after-owned-release requirement. It changes the interpretation of a literal physical cutoff and must be stated openly in the contract rather than silently substituted. If the approved requirement really demands that stronger physical guarantee for all admitted native operations, **do not freeze cooperative support as qualified**: report the capability gap and leave that criterion blocked pending an explicit implementable policy/architecture decision. Neither detaching blocking work nor reporting expiry while secretly continuing writes closes it. The author and parent must resolve this against actual approved intent; this note requests no blanket waiver or new T=0 allowance.
