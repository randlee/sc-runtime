# sc-runtime-db-sqlite Requirements

**Amended 2026-11-11.** See the note at the top of [architecture.md](architecture.md): PRD-STO-3 puts this writer behind `Storage<T>` with an async input.

**Status:** Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0012](../architecture.md). Sprint `b-1` implements these items.

## REQ-DBSQ-0001: Workspace member

**Status:** Active  
**Version:** 1.1.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

### Requirement Statement

Definitions: The dependency list is the crate's normal and build dependencies in `crates/sc-runtime-db-sqlite/Cargo.toml`; dev-dependencies are not part of it. A third-party dependency is a dependency that is not a workspace member. sc-storage is the workspace crate that defines the shared `Storage<T>` trait ([REQ-RUN-0002](../requirements.md)). Tokio is the `tokio` crate, which supplies the bounded async channel and awaited reply of the async input ([REQ-DBSQ-0002](#req-dbsq-0002-public-items), PRD-STO-3).

`crates/sc-runtime-db-sqlite` is a workspace member. This item owns its dependency list. Implementing `Storage<T>` with an async input needs the `Storage<T>` trait and an async channel, so the list is exactly `rusqlite` with the `bundled` feature, `tokio`, and the workspace crate `sc-storage`. Until the async input lands (code-alignment STO-2), the code depends on `rusqlite` alone; that is the code as it is, not the target. [REQ-RUN-0005](../requirements.md) point 4 points here.

**MUST statements:**

- `crates/sc-runtime-db-sqlite` MUST be a workspace member.
- The dependency list MUST contain `rusqlite` with the `bundled` feature.
- The dependency list MUST contain `tokio`.
- The dependency list MUST contain the workspace crate `sc-storage`.

**MUST NOT statements:**

- The dependency list MUST NOT contain a third-party dependency other than `rusqlite` and `tokio`.
- This crate MUST NOT depend on a workspace crate other than `sc-storage`.
- The dependency list MUST NOT contain `sqlx`, directly or transitively.

### Rationale

Owner, 2026-10-25 (ADR-RUN-0012): copy the atm-core writer machine into a library crate; do not put it behind a SQL trait shared with sqlx. The rusqlite `bundled` feature gives zero system SQLite dependency (PRD NF-04). Owner, 2026-11-11 (PRD-STO-3): the writer is the SQLite implementation behind `Storage<T>` with atm-core's async channel input, which brings in exactly `sc-storage` and `tokio`; nothing else is added.
### Success Criteria

**Acceptance Criteria:**

- [ ] The crate's boundary manifest lists `sc-storage` as its only allowed workspace dependency and forbids every other workspace crate.
- [ ] `cargo tree -p sc-runtime-db-sqlite -e normal,build --depth 1 --prefix none` prints, after the crate's own line, exactly one line each for `rusqlite`, `tokio` and `sc-storage`.
- [ ] `cargo tree -p sc-runtime-db-sqlite -e normal --prefix none` prints no line `sqlx`.

## REQ-DBSQ-0002: Public items

**Version:** 1.1.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-13
**Owner:** randlee  
**Status:** Active  

### Requirement Statement

Definitions: Storage<T> is the sc-storage trait for reading and writing application rows of type T.
The public SQLite storage compile test passes the exported SQLite storage type from sc-runtime-db-sqlite to a function generic over sc-storage’s Storage<T> from a consuming Rust crate.
An exported type is a public Rust type usable from a consuming crate.
The SQLite storage implementation is the sc-runtime-db-sqlite implementation of Storage<T>.
Channel submission is sending storage work to the SQLite storage implementation through an asynchronous channel.
An asynchronous channel accepts submitted storage work without blocking the calling thread while the caller awaits a reply.

The current public items are `SqliteWriter`, `WriterConn`, `SqlValue`, `SqliteWriterError`, `CHANNEL_CAPACITY`, `BATCH_TIME_BUDGET`, `WRITE_DEADLINE` and `SHUTDOWN_JOIN`, with signatures in the [architecture file](architecture.md). The blocking signatures describe the current code; the required replacement is SQLite storage with asynchronous input.

**MUST statements:**

- Running `cargo test -p sc-runtime-db-sqlite` with the public SQLite storage compile test MUST exit with code 0.
- The SQLite storage implementation MUST allow callers to await channel submission without blocking the calling thread.

### Rationale

Sprint b-1 copies the atm-core writer with these types substituted for its `WriteOp`, `WriteOpResult`, `AtmError`, and `SharedDbTarget`.
### Success Criteria

**Acceptance Criteria:**

- [ ] `sc-lint lint sc-boundary` exits 0 with the crate's boundary manifest, whose `[public] facade` lists exactly the crate's exported items: the inventory above for the code as it is, and the SQLite `Storage<T>` implementation and its async input once the target lands (code-alignment STO-8).
- [ ] Once the target lands, `cargo test -p sc-runtime-db-sqlite` exits 0 with a test that uses the SQLite implementation through a function generic over sc-storage's `Storage<T>` and awaits its replies without a blocking call on the caller.

## REQ-DBSQ-0003: Reply after commit

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: execute is the SQLite writer operation; the outer commit commits the batch transaction containing that operation.

The writer thread is the only caller of rusqlite. `execute` replies with the number of rows affected only after the outer commit.

**MUST statements:**

- The writer thread MUST be the only caller of rusqlite.
- `execute` MUST reply with the number of rows affected only after the outer commit.

### Rationale

ADR-RUN-0012 copies the measured atm-core path: one rusqlite connection on one thread, a bounded channel, `BEGIN IMMEDIATE`, and a reply only after commit.
### Success Criteria

**Acceptance Criteria:**

- [ ] `cargo test -p sc-runtime-db-sqlite` exits 0, covering one `execute` round-trip and two concurrent `execute` calls with reply-after-commit.

## REQ-DBSQ-0004: One commit order

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: execute is the writer call submitting one SQLite write operation. The first and second concurrent calls are ordered by their admission into the single writer queue.

A test of two concurrent `execute` calls observes one commit order: the second reply is not sent before the first statement has committed or failed.

**MUST NOT statements:**

- For two concurrent `execute` calls, the writer MUST NOT send the second call's reply before the first call's statement has committed or failed.

### Rationale

ADR-RUN-0012 point 3: contiguous executes share one savepoint inside one `BEGIN IMMEDIATE` transaction, and each caller is answered only after that transaction commits.
### Success Criteria

**Acceptance Criteria:**

- [ ] A test of two concurrent `execute` calls observes one commit order: the second reply is not sent before the first statement has committed or failed. The test drives queue admission order with controlled inputs rather than timing or sleeps.

## REQ-DBSQ-0005: No runtime or template edge

**Status:** Superseded  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Definitions: This crate is sc-runtime-db-sqlite; naming it means declaring a dependency on it. The listed historical statements remain nonbinding under the Superseded status and the explicit scope text.

The template's SQLite store and the runtime use this writer. The text below binds nothing.

Template `store-sqlite` does not name this crate. `sc-runtime` does not name it.

**MUST NOT statements:**

- Template `store-sqlite` does not name this crate.
- `sc-runtime` does not name it.

### Rationale

ADR-RUN-0301 binds `template/crates/store-sqlite` to sqlx 0.9 with no write actor; a `StorageBackend` that runs the same SQL on rusqlite and on Postgres is the portable query layer that ADR rejects. `sc-runtime` does not open the writer (ADR-RUN-0012).
### Success Criteria

**Acceptance Criteria:**

- [ ] `rg sc-runtime-db-sqlite template` prints nothing.
- [ ] `sc-lint lint sc-boundary` exits 0 with `sc-runtime-db-sqlite` in the forbidden edges of `sc-runtime`.
