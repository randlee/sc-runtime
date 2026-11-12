# sc-runtime

**Product Requirements Document**
Version 2.0 · 2026-11-11 · Synaptic Canvas
Supersedes version 1.1 (June 2026) in full.

**Authority.** This document is the top of the sc-runtime document hierarchy. `docs/requirements.md`, `docs/architecture.md` and the per-crate documents implement it. Where one of them disagrees with this PRD, this PRD is right and the other document is fixed.

**Provenance rule.** Every requirement below is the owner's statement, quoted. A mechanism chosen by an agent to satisfy a requirement is a design choice: it is recorded as a design choice in an ADR and is never written back here, or anywhere else, as an owner requirement.

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
| PRD-CMD-1 | Every command is a write path or a read path as drawn above. "I expect this pattern to be most situations." |
| PRD-CMD-2 | CLI, MCP and HTTP are three interfaces onto the same path through the daemon. They default to the same set of endpoints. |
| PRD-CMD-3 | A command is defined by `Tcli`, `Tsql` and the transform between them. |
| PRD-CMD-4 | The transform is where business logic goes. "You need to be able to get in the middle of the transform method w/ business logic. But a lot of basic things that is all you need." |
| PRD-CMD-5 | The transform function can be written in Python or Monty. "I really want to be able to write the function in python (or better yet monty)." |
| PRD-CMD-6 | The daemon is reached over HTTP on a Unix domain socket or TCP, as in atm-core. |
| PRD-CMD-7 | The interfaces differ in access and permission. The differences are not yet specified (section 9). |
| PRD-CMD-8 | Other paths follow the same pattern. "There will be additional paths like this, but they could all follow this pattern." |

## 4. Storage

> "I want sqlite/postgres/sql to be interchangeable behind traits (this is where sqlx came in), and I want the sqlite writer to be the high performance design used in atm-core."
> "Storage<T> would allow reading or writing T to a database."

| Id | Requirement |
|---|---|
| PRD-STO-1 | `Storage<T>` reads and writes `T`. That is its contract. |
| PRD-STO-2 | SQLite, Postgres and MySQL are interchangeable behind the storage trait. |
| PRD-STO-3 | The SQLite writer is the high-performance design used in atm-core: async channel input, transaction batching. "The channel is part of the sqlite implementation behind the storage trait." |
| PRD-STO-4 | The high-performance SQLite implementation is the one requirement brought in from atm-core. atm-core's API shape is not a requirement. |
| PRD-STO-5 | A table that needs more than read and write gets an extension over the base. "You could implement CronStorage : Storage<T> if you wanted to. I am just saying what I want is very simple." |

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

## 7. Monty

> "Monty is NOT out of scope. Monty is done."

Monty is part of sc-runtime and is already implemented.

## 8. Engineering rules

| Id | Rule |
|---|---|
| PRD-ENG-1 | "sc-runtime is a tokio app." "We should not have any synchronous code in sc-runtime unless absolutely necessary." |
| PRD-ENG-2 | Sync-over-async is not used. It "sounds like an unsound architecture." |
| PRD-ENG-3 | sc-runtime is built by copying code that already works. "We will worry about synchronization once [we] copy the code we already have to build sc-runtime." |
| PRD-ENG-4 | Keep it simple. "What I want is very simple." |
| PRD-ENG-5 | A requirement says what is built. It does not take an option away from a consumer, and leaving an option open creates no work. On `sc-config`: "I don't have a huge issue w/ it being synchronous today, I have issue w/ there being requirements forbidding async." "All I asked for is the option and not to have the option taken from a consumer." |
| PRD-ENG-6 | An open option is not a work item. Nothing is specified, built or tested for it until the owner asks. "Once you remove the requirement limiting something, then agents will get wrapped around the axle defining and testing every possible thing that could happen at run-time." |

## 9. Deferred and undecided

Nothing in this section is a requirement. No document may bind a mechanism for these until the owner decides, and no work is done on them until the owner asks (PRD-ENG-6).

| Item | State |
|---|---|
| Synchronization between databases | Deferred by the owner until the existing code has been copied in. |
| Access and permission differences between CLI, MCP and HTTP | Wanted (PRD-CMD-7); not specified. |
| Hand-written code between CLI and storage | Interim. Owner: "most likely where the 'hand write everything between cli and storage' came from, it is because we would need a sophisticated attribute based code generator to get to mvp." Hand-written commands are the stand-in until that exists; they follow the shape of section 3 and are not the target. |
| Code generation for the command pattern | Owner intent: "It would be easy to write a code generator that generates this code using sc-compose at the consumer end, or better yet, build a syn based generator." Not decided. |
| Repository shape | Owner: "I really think sc-runtime is too big and I need to start writing one crate per repo." Not decided. |
| Configuration reload and async configuration | Not built and not forbidden. Owner: "adding a file watcher and reload after a delay would be something we may want." Intended path, owner: "the path we need to enable is to allow sc-config to be replaced by an async version in the future. ConfigAsync trait that extends ConfigSync trait (names I just made up to illustrate)." `sc-config` stays as it is: synchronous, no watcher, no reload. A consumer keeps the option (PRD-ENG-5). No work follows (PRD-ENG-6). |
| The exact `Storage<T>` method set, how `T` maps to a table, who owns migrations | Open; see ADR-RUN-0020. |
