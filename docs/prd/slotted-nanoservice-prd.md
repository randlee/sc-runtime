# Slotted Nanoservice Architecture

**Product Requirements Document**
Draft 0.2 · 2026-11-12 · Synaptic Canvas

**Status.** Draft for owner review. Not binding. This document is **not** an input
to Phase D planning, gating or review. Owner, 2026-11-12: "it is my hope that I we
can start phase-d w/out slotted-nanoservice architecture and layer it on in
phase-e/f."

**Relationship to the sc-runtime PRD.** `sc-runtime-prd.md` (v2.0) stays the top of
the sc-runtime document hierarchy. This document describes the architecture that
is layered onto it in later phases. Where the two disagree today, the sc-runtime
PRD governs.

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
| SNA-2 | **[D]** "This architecture strictly does NOT allow code to go outside it's nanoservice boundary." |
| SNA-3 | **[D]** "every function essentially a plug-in and the ability to add infinite functions that are tied together in a registry" |
| SNA-4 | **[D]** Business logic can "be replaced w/ python/monty at run-time". |
| SNA-5 | **[D]** "yes, you can chain functions." |
| SNA-6 | **[D]** "You need to underrstand exactly what inputs/outputs are when the function is created and registered." |
| SNA-7 | **[D]** "everything is statically type checked at compile time and even if you change rust at runtime you only need to compile a single crate w/ references to the traits in/out it requires." |
| SNA-8 | **[D]** Use cases are pushed as far as possible so nothing is ruled out: "I am trying to push use cases i.e. user stories as far as possible so things aren't ruled out." |

## 3. Building blocks

### 3.1 Daemon

**[P]** The daemon is only a supervisor. It builds services in dependency order,
hands each one the handles it needs at construction, owns one cancellation signal,
shuts services down in reverse start order, and applies a per-service policy when a
service task dies. It stays a singleton with a hard gate (existing sc-runtime rule).

### 3.2 Nanoservice

**[P]** One tokio task that exclusively owns one thing, reachable only through a
typed handle. The high-performance SQLite writer already has this shape.

| Role | Owns | Examples **[D]** | Business logic |
|---|---|---|---|
| Resource | State | Storage (SQLite, other databases, JSON config file); RAM state such as an immutable database cache with enrichment | None |
| Driver | An inbound edge | HTTP server; cron | None. It turns an event into a typed input. |

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
- The daemon supplies the second input and performs the write.
- The one-input transform of the sc-runtime command pattern is the same thing with
  no second input.

**[D]** An outcome that writes nothing is still a typed result: "returning nothing
would still return Result<_,E> unless it is impossible to emit an error."

**[P]** Recommended outcome shape: `Result<Outcome, E>` where the outcome is
"write this `Tsql`" or "write nothing". **[O]** Not yet accepted in place of
`Tsql` plus `bool`.

### 3.4 Registry

**[P]** The registry holds slots and implementations.

- **Slot:** a named, typed position. Its input and output types are fixed at
  registration (SNA-6).
- **Implementation:** fills a slot. Replacing logic at run time replaces the
  implementation; the slot's signature cannot change (SNA-4).

**[P]** A binding has four declared parts: source (a driver), reads (zero or more
resources), function (the slot), sink (a resource).

**[P]** Chain rules (SNA-5): an edge is legal only when the upstream output type is
the downstream input type; cycles are rejected at registration; an error or a
write-nothing outcome ends the chain; replacing one function cannot invalidate a
chain.

### 3.5 Atomic transactions

> "the transaction future WOULD be extensible both to consumers OR to a slotted
> plug-in. We may need to add monty support, but being able to define an atomic
> transaction w/ custom retry logic seems very aligned w/ slotted nanoservices."

**[D]** The transaction future is a first-class extension point for consumers and
for slotted plug-ins, including custom retry logic.

**[P]** A transaction body is a slot whose only permitted calls are transaction
operations. No separate transaction trait is needed in the Monty crate: it is an
ordinary invoke whose permitted host functions are the transaction operations.
Owner: "MontyTransaction, drop it if it is not needed."

**[P]** A stepped Monty script can only stop at a host call. If the only host
functions offered are transaction operations, a Monty transaction body cannot
suspend on anything else, which is the property a Rust transaction-body analyzer
has to prove.

## 4. Monty

> "I would think that sc-monty ONLY have pydantic/monty dependencies and that
> sc-crom would depend on monty. then we could define additional monty calling
> signatures for slotted-nanoservice and db-transaction"

> "All of the business logic to support monty method calling should be implemented
> in sc-monty crate, potentially behind a MontyInvoke<Tin,Tout> trait."

| Id | Statement |
|---|---|
| SNA-MONTY-1 | **[D]** `sc-monty` depends on the pydantic Monty crates only. It is a wave-1 crate with no `sc-*` dependencies. |
| SNA-MONTY-2 | **[D]** All logic for calling Monty lives in `sc-monty`, potentially behind `MontyInvoke<Tin, Tout>`. |
| SNA-MONTY-3 | **[D]** `sc-cron` depends on `sc-monty`. serde is "a cron concern, not a monty concern". |
| SNA-MONTY-4 | **[D]** Further calling signatures are defined on top of it for slotted functions and database transactions. |
| SNA-MONTY-5 | **[D]** Monty source is changeable at run time: "no point in using monty for hook/cron if end user can't change it at run-time." |
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
| Host | Daemon, registry, drivers, resources | The contract. Never a plug-in crate. |

The plug-in rule is the nanoservice boundary for Rust code (SNA-2). A dependency
lint can enforce it, and it is what makes "compile a single crate" true (SNA-7).

## 6. Type checking across the boundary

**[P]** Each compilation is fully checked, but the host and a run-time-loaded
plug-in are two compilations and Rust has no stable ABI. One run-time check closes
the gap: at load, a plug-in presents a fingerprint of the contract it was compiled
against and the host refuses anything that differs.

| Backend | When types are checked | By what |
|---|---|---|
| Rust, linked into the host | Host compile | rustc |
| Rust, loaded at run time | Plug-in compile, plus fingerprint at load | rustc, then the host |
| Monty | Registration | Monty's type checker (SNA-MONTY-6) |
| CPython (PyO3) | Registration at best | Schema validation of values |
| C-ABI | Registration | Fingerprint and schema |

## 7. Backends and the boundary rule

| Backend | How strictly SNA-2 holds |
|---|---|
| Monty | By construction. No I/O unless the host exposes a function. |
| Native Rust | By function signature plus the dependency lint. |
| CPython via PyO3 | Not enforced. Python can open files and sockets. One interpreter lock: functions run one at a time. |
| C-ABI | Not enforced. Arbitrary native code in the daemon's process; a crash takes the daemon down. |

**[O]** SNA-2 is mechanical for Monty, achievable for Rust, and a convention for
CPython and C-ABI unless those run out of process or are accepted as trusted-only.

## 8. Run-time replacement

**[P]** Replacing an implementation requires versioned bindings (in-flight calls
finish on the old version), a check at registration, a revert path, and a record of
which implementation version produced each write.

## 9. Generators

**[D]** "the expection is generators will be created in a follow up phase. A
generator would work against a delivered sc-runtime by a consuming project". No
requirement may say the command path must be hand-written.

## 10. Future considerations and research

Nothing here is a requirement. No work follows until the owner asks.

| Item | Owner statement |
|---|---|
| **[F]** Typed struct in/out for Monty calls | "Longer term, I would expect crates could construct monty calls based on rust struct in/out." "typed in/out - save the idea as a 'research later' todo in PRD" |
| **[F]** Serialization route | "we can serialize/deserialize either in monty OR in rust at the rust/monty boundary in the fastest possible manner. i.e. if serde is faster, serialize/deserialize in serde and cron would get concrete types. If customer types are included we would likely need serialization/deserialization to be handled by monty as we would not know the types at compile time. this is an argument to support both in cron." Saved for future optimizations. |
| **[F]** JSON invoke | "if cron is json based, it would be defined by a MontyJsonInvoke : MontyInvoke<string,string>" |
| **[F]** Async and aggregate invokes | "MontyInvokeAsync and MontyChannelAsync (i.e. select-many or channel-aggregate) type functions will want to be on the 'future considerations list'" |
| **[F]** Monty transaction bodies | "We may need to add monty support" (section 3.5). |

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
6. **[O]** Untrusted CPython and C-ABI plug-ins: in process and trusted-only, or out
   of process.
7. **[O]** How the Rust Monty crates expose the type checker (SNA-MONTY-6).

## 12. Phasing

**[D]** Phase D proceeds without this architecture. It is layered on in Phase E/F.

**[P]** What Phase D must not foreclose is tracked in the Phase D plan itself, in
the sc-runtime PRD's terms, without reference to this document.

## 13. Naming

Working name: **Slotted Nanoservice Architecture (SNA)**. Nanoservices each own one
thing; typed slots are what functions plug into.
