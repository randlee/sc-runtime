# sc-runtime

**Product Requirements Document**
Version 2.1 · 2026-11-12 · Synaptic Canvas
Supersedes version 1.1 (June 2026) in full. Version 2.0 (2026-11-11) is included unchanged except where the revision history in section 11 says otherwise.

**Authority.** This document is the top of the sc-runtime document hierarchy. `docs/requirements.md`, `docs/architecture.md` and the per-crate documents implement it. Where one of them disagrees with this PRD, this PRD is right and the other document is fixed.

**Provenance rule.** Every requirement below is the owner's statement, quoted. Where the owner's exact words are not on file, the requirement is marked *restated* and carries no quotation marks. A mechanism chosen by an agent to satisfy a requirement is a design choice: it is recorded as a design choice in an ADR and is never written back here, or anywhere else, as an owner requirement.

---

## 1. Purpose

> "I need to write 10-20 apps that all have the same patterns. One or more databases (matrix I specified). Most will actually sync the same data in a one (sql/postgres)-many (sqlite) manner. They all have cli, http, mcp which would default to the same set of endpoints. Optional cron w/ storage in one of the databases."

sc-runtime exists so that each of those apps is written once as its own types and rules, and everything the apps share is written once here.

## 2. What sc-runtime is

> "Plus cron, sc-observability for logging, sc-config for json configuration would cover 100% of what sc-runtime is."

sc-runtime is exactly these parts:

| # | Part | Section |
|---|---|---|
| 1 | The command pattern over CLI, MCP and HTTP | 3 |
| 2 | `Storage<T>` over SQLite, Postgres and MySQL | 4 |
| 3 | Cron | 6 |
| 4 | Monty | 7 |
| 5 | `sc-observability` for logging | — |
| 6 | `sc-config` for JSON configuration | — |

Anything not on this list needs a stated reason to exist.

This list is what is built now. It does not close sc-runtime to architecture that later phases layer onto these parts (PRD-ENG-7). A later phase states its own reason when it adds something.

## 3. The command pattern

> ```
> cli write: cli-command<Tcli> <-- http --> daemon --> transform<Tcli,Tsql> -> storage<Tsql> -> write
> cli read:  cli-command<Tcli> <-- http --> daemon --> storage<Tsql> -> read --> transform<Tsql,Tcli> -> Tcli
>
> mcp write: mcp-command<Tcli> <-- http --> daemon --> transform<Tcli,Tsql> -> storage<Tsql> -> write
> mcp read:  mcp-command<Tcli> <-- http --> daemon --> storage<Tsql> -> read --> transform<Tsql,Tcli> -> Tcli
> ```
> "Since cli and mcp both use http, the same pattern would apply for http as well. Just like atm-core this would be offered over uds or tcp. We would want some access/permission differences between the interfaces."

| Id | Requirement |
|---|---|
| PRD-CMD-1 | Most commands follow the write path or read path drawn above. "I expect this pattern to be most situations." |
| PRD-CMD-2 | CLI, MCP and HTTP are three interfaces onto the same path through the daemon. They default to the same set of endpoints. |
| PRD-CMD-3 | A command is defined by `Tcli`, `Tsql` and the transform between them. |
| PRD-CMD-4 | The transform is where business logic goes. "You need to be able to get in the middle of the transform method w/ business logic. But a lot of basic things that is all you need." |
| PRD-CMD-5 | The transform function can be written in Python or Monty. "I really want to be able to write the function in python (or better yet monty)." |
| PRD-CMD-6 | The daemon is reached over HTTP on a Unix domain socket or TCP, as in atm-core. |
| PRD-CMD-7 | The interfaces differ in access and permission. The differences are not yet specified (section 9). |
| PRD-CMD-8 | Other paths follow the same pattern. "There will be additional paths like this, but they could all follow this pattern." |
| PRD-CMD-9 | A transform that writes nothing still returns a typed result. "Returning nothing would still return Result<_,E> unless it is impossible to emit an error." |
| PRD-CMD-10 | A transform written in Monty or Python can be changed without rebuilding the app. "No point in using monty for hook/cron if end user can't change it at run-time." |
| PRD-CMD-11 | No requirement or ADR says the command path must be hand-written. "The expectation is generators will be created in a follow up phase. A generator would work against a delivered sc-runtime by a consuming project." |
| PRD-CMD-12 | The one-input transform drawn above is the form built now. Nothing may rule out later forms of the same seam (PRD-ENG-7): a transform with a second input, and a transform that runs inside an atomic unit (PRD-STO-6). |

## 4. Storage

> "I want sqlite/postgres/sql to be interchangeable behind traits (this is where sqlx came in), and I want the sqlite writer to be the high performance design used in atm-core."
> "Storage<T> would allow reading or writing T to a database."

| Id | Requirement |
|---|---|
| PRD-STO-1 | `Storage<T>` reads and writes `T`. That is its contract. |
| PRD-STO-2 | SQLite, Postgres and MySQL are interchangeable behind the storage trait. Postgres and MySQL are where sqlx is used. |
| PRD-STO-3 | The SQLite writer is the high-performance design used in atm-core: async channel input, transaction batching. "The channel is part of the sqlite implementation behind the storage trait." |
| PRD-STO-4 | The high-performance SQLite implementation is the one requirement brought in from atm-core. atm-core's API shape is not a requirement. |
| PRD-STO-5 | A table that needs more than read and write gets an extension over the base. "You could implement CronStorage : Storage<T> if you wanted to. I am just saying what I want is very simple." |
| PRD-STO-6 | Storage can run several row operations as a single atomic unit. The owner assumed this "would be part of the design either way. We should be able to define a bundled transaction as a default function for SQLite or sql." |
| PRD-STO-7 | The preferred form of the atomic unit is a transaction future. "You could define a transaction-future which had requirements and an analyzer. You gain observability and complete flexibility." |
| PRD-STO-8 | The transaction future is an extension point, and a transaction can carry its own retry logic. "The transaction future WOULD be extensible … to consumers." "Being able to define an atomic transaction w/ custom retry logic." |

## 5. Database layouts

An app has one or more databases from this matrix: `none`, `sqlite`, `postgres`, `mysql`, `sqlite-postgres`, `sqlite-mysql`.

Most apps will sync the same data between one Postgres or MySQL database and many SQLite databases. Synchronization is deferred (section 9).

## 6. Cron

> "Cron is just a table. It is defined by a struct for the rows. Why do you even need a storage trait? It could just be Storage<CronRow>."

| Id | Requirement |
|---|---|
| PRD-CRON-1 | Cron is optional. |
| PRD-CRON-2 | Cron's schedule table is a row struct stored through `Storage<CronRow>`. |
| PRD-CRON-3 | The table lives in one of the app's databases: "in the apps database … in the sqlite database by default." There is no separate cron database. |
| PRD-CRON-4 | "Cron would require a database, so 'none' isn't really an option." |
| PRD-CRON-5 | Cron follows the command pattern of section 3. "Cron would work that way." |
| PRD-CRON-6 | *Restated.* An end user schedules cron with a Monty or Python script plus JSON data that is passed to the script. The consuming app can modify or append to that JSON before it is written, to add its own data. |

## 7. Monty

> "Monty is NOT out of scope. Monty is done."

Monty is part of sc-runtime and is already implemented. Giving it a crate of its own is a move of that working code (PRD-ENG-3), not a rewrite.

| Id | Requirement |
|---|---|
| PRD-MONTY-1 | There is one Monty implementation. "All of the business logic to support monty method calling should be implemented in sc-monty crate, potentially behind a MontyInvoke<Tin,Tout> trait." |
| PRD-MONTY-2 | "I would think that sc-monty ONLY have pydantic/monty dependencies and that sc-cron would depend on monty." |
| PRD-MONTY-3 | serde is "a cron concern, not a monty concern". |
| PRD-MONTY-4 | Monty source can be changed at run time (PRD-CMD-10). |
| PRD-MONTY-5 | *Restated.* sc-cron can save a compiled Monty binary, keyed by the SHA-256 of the Monty source through the sc-sha crate, in a table that belongs to sc-cron. The owner described this as designed on the fly and open to refinement. |
| PRD-MONTY-6 | Further Monty calling signatures can be defined on top of the one implementation (PRD-ENG-7). None beyond what cron and the command transform need is built now (PRD-ENG-6). |

## 8. Engineering rules

| Id | Rule |
|---|---|
| PRD-ENG-1 | "sc-runtime is a tokio app." "We should not have any synchronous code in sc-runtime unless absolutely necessary." |
| PRD-ENG-2 | Sync-over-async is not used. It "sounds like an unsound architecture." |
| PRD-ENG-3 | sc-runtime is built by copying code that already works. "We will worry about synchronization once [we] copy the code we already have to build sc-runtime." |
| PRD-ENG-4 | Keep it simple. "What I want is very simple." |
| PRD-ENG-5 | A requirement says what is built. It does not take an option away from a consumer, and leaving an option open creates no work. On `sc-config`: "I don't have a huge issue w/ it being synchronous today, I have issue w/ there being requirements forbidding async." "All I asked for is the option and not to have the option taken from a consumer." |
| PRD-ENG-6 | An open option is not a work item. Nothing is specified, built or tested for it until the owner asks. "Once you remove the requirement limiting something, then agents will get wrapped around the axle defining and testing every possible thing that could happen at run-time." |
| PRD-ENG-7 | A requirement does not rule out later extension. "I am trying to push use cases i.e. user stories as far as possible so things aren't ruled out." A record that describes how something is wired in this release binds this release. It is not worded as a permanent prohibition. |
| PRD-ENG-8 | *Restated.* Fallible operations return `Result<T, E>` and public paths do not panic. The owner asked for an anchor requiring this and for an sc-lint lint that enforces it (sc-lint issue 931). |
| PRD-ENG-9 | sc-runtime does not pin its consumers' dependencies. "It is not sc-runtime scope to limit consumers from choosing differently and the consumer assume the risk." Published crates and the template state the minimum version they were tested with. |

## 9. Deferred and undecided

Nothing in this section is a requirement. No document may bind a mechanism for these until the owner decides, and no work is done on them until the owner asks (PRD-ENG-6).

| Item | State |
|---|---|
| Synchronization between databases | Deferred by the owner until the existing code has been copied in. |
| Access and permission differences between CLI, MCP and HTTP | Wanted (PRD-CMD-7); not specified. |
| Hand-written code between CLI and storage | Interim. Owner: "most likely where the 'hand write everything between cli and storage' came from, it is because we would need a sophisticated attribute based code generator to get to mvp." Hand-written commands are the stand-in until that exists; they follow the shape of section 3 and are not the target (PRD-CMD-11). |
| Code generation for the command pattern | Follow-up phase, on the consumer side, against a delivered sc-runtime (PRD-CMD-11). Owner: "It would be easy to write a code generator that generates this code using sc-compose at the consumer end, or better yet, build a syn based generator." Which generator is not decided. |
| Repository shape | Owner: "I really think sc-runtime is too big and I need to start writing one crate per repo." Not decided. |
| Configuration reload and async configuration | Not built and not forbidden. Owner: "adding a file watcher and reload after a delay would be something we may want." Intended path, owner: "the path we need to enable is to allow sc-config to be replaced by an async version in the future. ConfigAsync trait that extends ConfigSync trait (names I just made up to illustrate)." `sc-config` stays as it is: synchronous, no watcher, no reload. A consumer keeps the option (PRD-ENG-5). No work follows (PRD-ENG-6). |
| The exact `Storage<T>` method set, how `T` maps to a table, who owns migrations | Open; see ADR-RUN-0020. |
| The transaction future's requirements and its analyzer | Wanted (PRD-STO-7); the rules a transaction body must satisfy and how they are checked are not specified. |
| Second transform input, and a transform inside an atomic unit | Not built now and not ruled out (PRD-CMD-12). |
| Typed Rust struct in and out for Monty calls | Owner: "typed in/out - save the idea as a 'research later' todo in PRD." |
| The `sc-runtime-cli`, `-core`, `-daemon`, `-mcp-stdio` and `-web` documents | The owner holds that these crates "still have requirements". Their documents predate this PRD, cite version 1.1 requirement ids that no longer exist, and have not been realigned. Until they are, this PRD governs wherever they disagree with it. |
| Architecture layered on in later phases | Not part of the current phase and not ruled out (PRD-ENG-7). Its requirements are stated when the owner opens that phase. |

## 10. What this means for the documents below this one

This section is not a set of owner requirements. It applies the requirements above to requirement and ADR records as they stood on 2026-11-12, so that a reviewer can fix a record without asking. Each line names the PRD requirement that governs.

| # | What a record must not say | Governed by | Records known to say it |
|---|---|---|---|
| 1 | That sc-runtime may never contain a command or operation registry, a macro, a generator or a public trait. Those lists were design recommendations. They bind as "not built in this release". | PRD-ENG-5, PRD-ENG-7, PRD-CMD-11 | NFR-RUN-0003, NFR-RT-0001, ADR-RUN-0001 |
| 2 | That `sc-config` must never be async, watch a file, reload or notify. | PRD-ENG-5 | NFR-CFG-0003, ADR-CFG-0003 |
| 3 | That a dependency is pinned to one exact version or capped. | PRD-ENG-9 | REQ-RUN-0309, REQ-CMD-0002, NFR-CMD-0001, NFR-CMD-0002, REQ-CRON-0001 |
| 4 | That sc-cron may depend on no other crate of this workspace, or links Monty directly. | PRD-MONTY-1, PRD-MONTY-2, PRD-CRON-2 | NFR-CRON-0001, REQ-CRON-0001 |
| 5 | That the SQLite storage crate may depend on nothing but its SQLite client library. It implements `Storage<T>` with an async channel input. | PRD-STO-2, PRD-STO-3 | REQ-DBSQ-0001 |
| 6 | That a crate must avoid tokio or stay usable without an async runtime. | PRD-ENG-1 | `sc-runtime-core` NF-CORE-02 |
| 7 | That today's wiring is permanent: a fixed list of daemon startup steps, stores the daemon never touches, configuration the daemon never loads, or one service function per operation. These describe this release. | PRD-ENG-7 | REQ-RT-0001, REQ-RT-0003, NFR-RT-0002, NFR-RT-0003, REQ-RUN-0201, REQ-RUN-0302 |

Lines 1 to 6 are corrections to make. Line 7 is not: those records are right for this release and stay as they are. They are not strengthened, and no later-phase work is planned or tested because of this line (PRD-ENG-6).

A record that counts "the four crates" or calls cron "the fifth public library" is out of date wherever a storage crate and a Monty crate are also published.

## 11. Revision history

| Version | Date | Change |
|---|---|---|
| 2.0 | 2026-11-11 | Rewritten from the owner's statements. Supersedes 1.1 in full. |
| 2.1 | 2026-11-12 | Added PRD-CMD-9 to 12, PRD-STO-6 to 8, PRD-CRON-6, PRD-MONTY-1 to 6, PRD-ENG-7 to 9. Section 2 no longer reads as closed to later phases. Section 9 gains the transaction future, the deferred transform forms, the five older crate documents and later-phase architecture. Section 10 is new. PRD-STO-2 names where sqlx is used. The provenance rule admits requirements marked *restated*. No 2.0 requirement was removed or weakened. |
