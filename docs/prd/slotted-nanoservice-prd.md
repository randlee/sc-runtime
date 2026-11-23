# Slotted Nanoservice Architecture

**Product Requirements Document**
Draft 0.3 · 2026-11-12 · Synaptic Canvas

**Status.** Draft for owner review. Not binding. This document is **not** an input
to Phase D planning, gating or review. Owner, 2026-11-12: "it is my hope that I we
can start phase-d w/out slotted-nanoservice architecture and layer it on in
phase-e/f."

**Relationship to the sc-runtime PRD.** `sc-runtime-prd.md` v2.1 on branch
`docs/prd-v2.1` (reviewed source `8d5462d`) stays the top of the sc-runtime
document hierarchy. Its section 2 and PRD-ENG-7 explicitly leave room for the
architecture described here in later phases. Where the two disagree today, the
sc-runtime PRD governs. References below are to that v2.1 revision, which is
under review on its own branch; this draft does not change that branch.

**Provenance rule.** Same rule as the sc-runtime PRD. A requirement is the owner's
statement, quoted. Everything else is marked:

- **[D] Decided** — the owner's statement, quoted or directly restated.
- **[P] Proposed** — a design consequence or recommendation. Not accepted yet.
- **[O] Open** — not decided.
- **[F] Future** — recorded so it is not ruled out. No work follows until the owner
  asks (sc-runtime PRD-ENG-6).

---

## 1. Summary

> "I am trying to specify an architecture pattern using tokio which would involve
> what in reality would be a series of microservices / nanoservices that all hang
> off the daemon. some services would be 'storage' (i.e. sqlite, database, json
> config file). some services would be RAM storage of state (i.e. immutable db
> cache w/ enrichment ). One service would be http server, one would be cron. Then
> some services would be outward facing i.e. cron job, hook, method which
> transformed data from Tin->Tout or (Tin1,Tin2)->Tout . the outward facing
> services could be extended and set at runtime through py03 or c-abi interfaces
> to call python, monty, rust or other languages."

A tokio daemon supervises small services. Each service owns exactly one thing. All
business logic lives outside the services, in functions that are registered in a
typed registry and can be replaced at run time.

```text
            +-------------------------- daemon (supervisor) --------------------------+
            |                                                                          |
  event --> | DRIVER            REGISTRY                               RESOURCE        |
            | (http, cron,  --> slot: (Tin1, Tin2) -> outcome     -->  Storage<Tsql>   |
            |  hook)                  ^        ^                       (write if told)  |
            |                         |        |                                       |
            |                   function     RESOURCE                                  |
            |                   (Rust, Monty, RAM cache snapshot (Tram)                |
            |                    Python, C-ABI)                                        |
            +--------------------------------------------------------------------------+
```

## 2. Principles

| Id | Principle |
|---|---|
| SNA-1 | **[D]** Tokio throughout; no synchronous code unless absolutely necessary (sc-runtime PRD-ENG-1, PRD-ENG-2). |
| SNA-2 | **[D, Rust scope per owner-requested narrowing, 2026-11-12]** Rust plug-ins must stay within their nanoservice boundary. This is the Rust dependency and call contract of section 5, not an in-process security sandbox. No equivalent enforcement guarantee is made for CPython or C-ABI (section 7). |
| SNA-3 | **[D]** "every function essentially a plug-in and the ability to add infinite functions that are tied together in a registry" |
| SNA-4 | **[D]** Business logic can "be replaced w/ python/monty at run-time". |
| SNA-5 | **[D]** "yes, you can chain functions." |
| SNA-6 | **[D]** "You need to underrstand exactly what inputs/outputs are when the function is created and registered." |
| SNA-7 | **[D, Rust scope per owner-requested narrowing, 2026-11-12]** Rust code is statically type checked at each compilation; a replacement Rust plug-in compiles as one crate against its input/output contracts. Separately compiled code still needs the load-time compatibility check in section 6. This compile-time guarantee does not apply to Monty, CPython or C-ABI calls. |
| SNA-8 | **[D]** Use cases are pushed as far as possible so nothing is ruled out: "I am trying to push use cases i.e. user stories as far as possible so things aren't ruled out." |

## 3. Building blocks

### 3.1 Daemon

**[P]** Here, “the daemon” means the consuming project's `daemon` crate,
not the published `sc-runtime` crate. The project daemon supervises services:
it builds them in dependency order, hands each one its handles at construction,
owns one cancellation signal, shuts services down in reverse start order, and
applies a per-service policy when a service task dies. It retains the existing
singleton hard gate.

**[P]** Services start inside the existing REQ-RT-0001 startup steps; this
proposal does not replace or reorder those steps. Resource construction and
service readiness run in the project's asynchronous stores closure, after the
singleton lock is held. Route and MCP assembly use the returned project handles;
the existing final step binds the HTTP listener. The project owns cleanup through
the existing cleanup path. No new `sc-runtime` hook or public builder method is
implied. These are later-phase project wiring proposals, not Phase D work.

### 3.2 Nanoservice

**[P]** One tokio task that exclusively owns one thing, reachable only through a
typed handle. The high-performance SQLite writer already has this shape.

| Role | Owns | Examples **[D]** | Business logic |
|---|---|---|---|
| Resource | State | Storage (SQLite, other databases); configuration resource backed by a JSON file; RAM state such as an immutable database cache with enrichment | None |
| Driver | An inbound edge | HTTP server; cron | None. It turns an event into a typed input. |

**[P]** The configuration resource is a later service that wraps `sc-config`
through the async-trait extension path described in v2.1 section 9. The trait
names there are illustrative, not an API decision. The wrapper owns any future
task, watcher, reload or notification behavior; it changes nothing in `sc-config`
now. `sc-config` remains synchronous, with no tasks, watcher or reload. PRD-ENG-5
keeps the async option open and PRD-ENG-6 creates no current work for it.

### 3.3 Function

**[D]** A function transforms `Tin -> Tout` or `(Tin1, Tin2) -> Tout`. Cron jobs,
hooks and transform methods are all functions.

> "Tin1,Tin2 are traits. i.e. http would produce http-request<Thttp> which could go
> through a function/plugin which combined Tram (RAM cached table) w/ Thttp and
> conditionally produced data to write to database Tsql. therefore you would get
> http-request<Thttp> + Tram -> func<Thttp,Tram,Tsql,bool> -> if (true)
> sql.write<Tsql>()."

**[P]** Consequences:

- A function receives data and returns data. It holds no service handles.
- The consuming project's `service` crate supplies the second input and
  performs the write through the storage contract. The project `daemon` crate
  constructs and passes the resource handles; `sc-runtime` never inspects
  `Stores` or calls its methods (REQ-RT-0003). The shared command helpers remain
  in `sc-command`; this proposal does not move them into `sc-runtime`.
- The one-input transform of the sc-runtime command pattern is the same thing with
  no second input.

**[D]** Write-nothing outcomes follow PRD-CMD-9; that binding requirement is
not restated here.

**[P]** Recommended outcome shape: `Result<Outcome, E>` where the outcome is
"write this `Tsql`" or "write nothing". **[O]** Not yet accepted in place of
`Tsql` plus `bool`.

### 3.4 Registry

**[P]** The registry lives in a separate crate, provisionally `sc-registry`,
not in `sc-runtime`. Its public traits and registry API belong to that crate.
The consuming project wires it through its `service` crate; `sc-runtime` neither
re-exports it nor adds public items for it. NFR-RT-0001's closed facade therefore
needs no change. The crate name is proposed, not an owner-decided API.

**[P]** The registry holds slots and implementations.

- **Slot:** a named, typed position. Its input and output types are fixed at
  registration (SNA-6).
- **Implementation:** fills a slot. Replacing logic at run time replaces the
  implementation; the slot's signature cannot change (SNA-4).

**[P]** A slot is the replaceable transform called inside the operation's
existing async service function; it does not replace that service function.
REST and MCP handlers still call the same project `service` function, and CLI
requests still reach it through HTTP (REQ-RUN-0201, REQ-RUN-0302). Resource reads,
slot invocation and conditional writes are coordinated there, not in handlers.

**[P]** A binding has four declared parts: source (a driver), reads (zero or more
resources), function (the slot), sink (a resource).

**[P]** Chain rules (SNA-5): an edge is legal only when the upstream output type is
the downstream input type; cycles are rejected at registration; an error or a
write-nothing outcome ends the chain; replacing one function cannot invalidate a
chain.

### 3.5 Atomic transactions

**[D]** Atomic units, transaction futures and consumer-defined retry follow
PRD-STO-6 through PRD-STO-8. Their binding requirements are not duplicated here.
**[D]** The owner's additional later-phase use is extension to slotted plug-ins.

**[D, scope clarification]** Custom transaction retry does not override cron's
no-replay rule. An uncertain mutation is never replayed (REQ-CRON-0020), and the
standard public command path does not automatically replay work
(REQ-CRON-0021). A transaction's retry policy must respect those rules and the
existing admission, deadline and outcome contract; cancellation or a timeout
cannot be treated as evidence of rollback.

**[P]** A transaction body is a slot whose only permitted calls are transaction
operations. No separate transaction trait is needed in the Monty crate: it is an
ordinary invoke whose permitted host functions are the transaction operations.
Owner: "MontyTransaction, drop it if it is not needed."

**[P]** A stepped Monty script can only stop at a host call. If the only host
functions offered are transaction operations, a Monty transaction body cannot
suspend on anything else, which is the property a Rust transaction-body analyzer
has to prove.

## 4. Monty

The binding Monty decisions are referenced below rather than quoted again.
The remaining part of the owner's section 4 statement is:

> "then we could define additional monty calling signatures for
> slotted-nanoservice and db-transaction"

| Id | Statement |
|---|---|
| SNA-MONTY-1 | **[D]** Dependency policy: PRD-MONTY-2. The existing wave-1 placement remains a delivery decision, not a second dependency contract. |
| SNA-MONTY-2 | **[D]** One implementation of Monty calling logic: PRD-MONTY-1. |
| SNA-MONTY-3 | **[D]** Cron dependency and serialization ownership: PRD-MONTY-2 and PRD-MONTY-3. |
| SNA-MONTY-4 | **[D]** Further calling signatures are defined on top of it for slotted functions and database transactions. |
| SNA-MONTY-5 | **[D]** Runtime source replacement: PRD-MONTY-4 (and PRD-CMD-10). |
| SNA-MONTY-6 | **[D]** Monty's own type checker "should be called for validation or inside MontyInvoke depending on use case." |

**[P]** Shape that follows from SNA-MONTY-1: with no runtime inside it, `sc-monty`
is a step driver. Start a script with inputs and limits; it yields a host call, a
finished value or a typed error; the caller resumes it with the host call's result.
A calling signature is the inputs, the permitted host functions and the output
type, and each consumer crate owns its own.

**Verified 2026-11-12 against `monty` 1.4.0, the version in the lockfile:**

- `monty` and `monty-types` already depend on `serde`, so serde traits in
  `sc-monty` add no crate. `serde_json` would.
- Monty does not support third-party packages, so pydantic models cannot run inside
  a script. Validation is host-side or through Monty's type checker.
- No type-checking crate is in the dependency graph today. How the Rust crates
  expose the checker is unverified (affects SNA-MONTY-6).
- `sc-cron` keeps one session per schedule generation: source is compiled on the
  first run of a generation and reused until the schedule is replaced.
- Monty can serialize a whole session to bytes behind a versioned header and
  rejects dumps it cannot read with a typed error.

## 5. Crate layout

**[P]**

| Crate | Contains | May depend on |
|---|---|---|
| Contract | Input and output types and traits. No logic, no service handles. | Nothing in the system |
| Plug-in | One or more function implementations | The contract only |
| Registry (`sc-registry`, proposed) | Slots, bindings and implementation selection | The contract. Never a plug-in implementation crate. |
| Project host (`daemon` and `service`) | Supervision, drivers, resources and operation coordination | The contract and registry. Never a plug-in implementation crate. |

The plug-in dependency rule is the Rust architectural boundary (SNA-2).
A dependency lint checks crate edges; signatures restrict the handles supplied to
a function. Neither prevents arbitrary native code from performing I/O through
standard-library or foreign calls. Rust plug-ins must honor that boundary; this
is not a sandbox claim. SNA-7 concerns compiling the plug-in against the contract.
The published `sc-runtime` facade is not the “Host” crate in this table.

## 6. Type checking across the boundary

**[P]** Each compilation is fully checked, but the host and a run-time-loaded
plug-in are two compilations and Rust has no stable ABI. One run-time check closes
the gap: at load, a plug-in presents a fingerprint of the contract it was compiled
against and the host refuses anything that differs.

| Backend | When types are checked | By what |
|---|---|---|
| Rust, linked into the host | Host compile | rustc |
| Rust, loaded at run time | Plug-in compile, plus fingerprint at load | rustc, then the host |
| Monty | Registration, proposed; not Rust compile time | Monty's type checker (SNA-MONTY-6); Rust exposure remains open |
| CPython (PyO3) | Registration at best | Schema validation of values |
| C-ABI | Registration | Fingerprint and schema |

## 7. Backends and the boundary rule

| Backend | How strictly SNA-2 holds |
|---|---|
| Monty | By construction. No I/O unless the host exposes a function. |
| Native Rust | Architectural contract: signatures and dependency lint constrain supplied handles and crate edges, not arbitrary native I/O. |
| CPython via PyO3 | Not enforced. Python can open files and sockets. In the single-interpreter-lock model considered here, calls requiring that lock serialize. |
| C-ABI | Not enforced. Arbitrary native code in the daemon's process; a crash takes the daemon down. |

**[D, scope clarification]** SNA-2 binds Rust's architectural boundary and
SNA-7 binds Rust compilation. Neither promises enforced isolation or compile-time
type checking for CPython or C-ABI; Monty has its own registration-time checking
path. This resolves the universal claims without selecting a sandbox mechanism.

**[D]** CPython calls never run on a Tokio request worker. **[P]** The host
awaits results from dedicated blocking execution capacity outside the request
workers; interpreter-lock acquisition and Python execution both happen there.
This preserves PRD-ENG-1 and PRD-ENG-2 without sync-over-async. The precise
executor is a later-phase design choice, not current implementation work.

**[O]** Whether CPython and C-ABI plug-ins are accepted as trusted in-process
code or isolated out of process remains a deployment decision. This open choice
is no longer needed to make SNA-2 or SNA-7 true.

## 8. Run-time replacement

**[P]** Replacing an implementation requires versioned bindings (in-flight calls
finish on the old version), a check at registration, a revert path, and a record of
which implementation version produced each write.

## 9. Generators

**[D]** Generators and the prohibition on requiring hand-written command paths
are governed by PRD-CMD-11. No separate requirement is restated here.

## 10. Future considerations and research

Nothing here is a requirement. No work follows until the owner asks.

| Item | Owner statement |
|---|---|
| **[F]** Typed struct in/out for Monty calls | See v2.1 section 9, “Typed Rust struct in and out for Monty calls”; remains research, not implementation work. |
| **[F]** Serialization route | "we can serialize/deserialize either in monty OR in rust at the rust/monty boundary in the fastest possible manner. i.e. if serde is faster, serialize/deserialize in serde and cron would get concrete types. If customer types are included we would likely need serialization/deserialization to be handled by monty as we would not know the types at compile time. this is an argument to support both in cron." Saved for future optimizations. |
| **[F]** JSON invoke | "if cron is json based, it would be defined by a MontyJsonInvoke : MontyInvoke<string,string>" |
| **[F]** Async and aggregate invokes | "MontyInvokeAsync and MontyChannelAsync (i.e. select-many or channel-aggregate) type functions will want to be on the 'future considerations list'" |
| **[F]** Monty transaction bodies | Later-phase proposal in section 3.5; PRD-CMD-12 and PRD-MONTY-6 leave the calling form open. |

## 11. Open decisions

1. **[O]** Outcome shape (section 3.3).
2. **[O]** What `Tram` is: the whole cached table, or the rows selected by the
   request. A whole-table snapshot is free for native Rust and expensive to pass
   into Python or a C-ABI plug-in on every call.
3. **[O]** The snapshot race: a function decides against a snapshot and the write
   happens afterwards. Either a database constraint is the guard, or the decision
   runs as a transaction body (section 3.5).
4. **[O]** Read-your-writes: whether the cache must reflect a write before the
   response returns, and who invalidates it.
5. **[O]** What crosses into a run-time-loaded Rust plug-in: native Rust types or
   serialized values.
6. **[O]** CPython and C-ABI deployment: trusted-only in process, or isolated
   out of process. SNA-2 and SNA-7 make no guarantee for these backends.
7. **[O]** How the Rust Monty crates expose the type checker (SNA-MONTY-6).

## 12. Phasing

**[D]** Phase D proceeds without this architecture. It is layered on in Phase E/F.

**[D]** The published basis for keeping later extensions possible is now
PRD-CMD-12, PRD-ENG-7 and v2.1 section 10 line 7. The Phase D plan is not a
separate source of that authority. Line 7 leaves today's startup, store opacity
and per-operation service wiring unchanged for this release. It creates no
later-phase specification, implementation or testing work (PRD-ENG-6).

**[P]** Sections 3.1 through 3.4 describe how the later architecture can compose
with those existing boundaries. They are proposals for a later phase, not new
Phase D acceptance criteria.

## 13. Naming

Working name: **Slotted Nanoservice Architecture (SNA)**. Nanoservices each own one
thing; typed slots are what functions plug into.

## 14. Private watch list for apyr

Owner-only review aid; do not copy into Phase D plans, gates or team briefs.
Consumer: apyr. It protects later extension while gating only this draft's
consistency review. The observed defect class is a current-release wiring rule
being read as a permanent prohibition, plus the host/facade ambiguity corrected
in sections 3.1–3.4. Remove this list when the later-phase requirements are
published and replace it with their ordinary record links. It assigns no work.

Record baseline: `1bdcebb`; PRD authority: v2.1 at `8d5462d`. These are watch
points, not claims that every listed record currently conflicts. Re-read the
exact target revision before reporting a conflict. Published PRD authority alone
must justify any finding sent to the Phase D team.

| Draft id | Records to watch | Boundary or blocking reading to check |
|---|---|---|
| SNA-1 | REQ-RT-0001; NFR-RUN-0002; NFR-CFG-0003; ADR-CFG-0003 | Service startup stays inside existing steps; later config wrapper does not make sc-config async now; Python execution stays off request workers. |
| SNA-2 | REQ-RUN-0201; REQ-RUN-0302; REQ-RT-0003 | The project service coordinates resource access; handlers and sc-runtime do not. Rust dependency checks are not process isolation. |
| SNA-3 | NFR-RUN-0003; NFR-RT-0001; ADR-RUN-0001 | A release-scoped registry exclusion must not become a permanent ban; proposed registry has its own crate, outside the closed sc-runtime facade. |
| SNA-4 | ADR-RUN-0021; REQ-RUN-0201; REQ-RUN-0302 | A replaceable implementation remains behind the existing per-operation service function; PRD-CMD-10 governs runtime source changes. |
| SNA-5 | REQ-RUN-0201; REQ-RUN-0302; ADR-RUN-0021 | One entry service function per operation must not be read as forbidding later internal function chains. |
| SNA-6 | NFR-RT-0001; REQ-RT-0003; ADR-RUN-0021 | Registration contracts live outside sc-runtime; its opaque Stores parameter does not become a resource interface. |
| SNA-7 | NFR-RT-0001; ADR-RUN-0001; ADR-RUN-0021 | Rust contract types and registration APIs belong outside the closed facade; do not turn Rust compile-time checking into a claim about other backends. |
| SNA-8 | REQ-RT-0001; REQ-RT-0003; REQ-RUN-0201; REQ-RUN-0302; NFR-RT-0001 | Preserve current startup, store opacity and service entry points; PRD-ENG-7 keeps extension possible without current work. |
| SNA-MONTY-1 | REQ-CRON-0001; NFR-CRON-0001 | Dependency descriptions must preserve the separate sc-monty implementation under PRD-MONTY-2. |
| SNA-MONTY-2 | ADR-RUN-0021; REQ-CRON-0001 | Transform and cron calling paths must not duplicate Monty implementation (PRD-MONTY-1). |
| SNA-MONTY-3 | REQ-CRON-0001; NFR-CRON-0001 | Blanket workspace-dependency bans must not prevent cron using sc-monty; serialization remains a cron concern. |
| SNA-MONTY-4 | ADR-RUN-0020; ADR-RUN-0021; REQ-CRON-0020; REQ-CRON-0021 | Later calling forms remain possible; transaction retry never authorizes replay of uncertain cron mutations. |
| SNA-MONTY-5 | ADR-RUN-0021; REQ-CRON-0003 | Do not turn a schedule generation or transform binding into a permanent prohibition on replacing source. |
| SNA-MONTY-6 | ADR-RUN-0021; REQ-CRON-0001 | Do not claim the type checker is integrated before its Rust interface is verified; no Phase D work follows from this draft. |
