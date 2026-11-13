# sc-runtime-db-sqlite Requirements

**Amended 2026-11-11.** See the note at the top of [architecture.md](architecture.md): PRD-STO-3 puts this writer behind `Storage<T>` with an async input.

**Status:** Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0012](../architecture.md). Sprint `b-1` implements these items.

## REQ-DBSQ-0001: Workspace member

**Status:** Active  
**Version:** 1.1.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: The dependency list is the crate's normal and build dependencies in `crates/sc-runtime-db-sqlite/Cargo.toml`; dev-dependencies are not part of it. The dependency graph is every crate that the dependency list pulls in, directly or through other crates. sc-storage is the workspace crate that defines the shared `Storage<T>` trait ([REQ-RUN-0002](../requirements.md)). Tokio is the `tokio` crate, which supplies the bounded async channel and awaited reply of the async input ([REQ-DBSQ-0002](#req-dbsq-0002-public-items), PRD-STO-3).

`crates/sc-runtime-db-sqlite` is a workspace member. This item owns its dependency list. Implementing `Storage<T>` with an async input needs the `Storage<T>` trait and an async channel, so the list is exactly `rusqlite` with the `bundled` feature, `tokio`, and the workspace crate `sc-storage`. Until the async input lands (code-alignment STO-2), the code depends on `rusqlite` alone; the code as it is is not the target. [REQ-RUN-0005](../requirements.md) point 4 points here.

**MUST statements:**

- The `[workspace] members` list in the root `Cargo.toml` MUST contain `crates/sc-runtime-db-sqlite`.
- The `[dependencies]` table of `crates/sc-runtime-db-sqlite/Cargo.toml` MUST contain a `rusqlite` key.
- The `features` array of the `rusqlite` entry in `crates/sc-runtime-db-sqlite/Cargo.toml` MUST contain `"bundled"`.
- The `[dependencies]` table of `crates/sc-runtime-db-sqlite/Cargo.toml` MUST contain a `tokio` key.
- The `[dependencies]` table of `crates/sc-runtime-db-sqlite/Cargo.toml` MUST contain an `sc-storage` key.

**MUST NOT statements:**

- The dependency list MUST NOT contain a crate other than `rusqlite`, `tokio` and `sc-storage`.
- The dependency graph MUST NOT contain a workspace crate other than `sc-storage`.
- The dependency graph MUST NOT contain `sqlx`.

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
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: `execute` is the SQLite writer operation.
`rusqlite` is the Rust SQLite library used by `sc-runtime-db-sqlite`.
The write-mode connection is the single `rusqlite` connection of `sc-runtime-db-sqlite` that opens the database for writing.
The writer thread owns the write-mode connection and processes operations from the bounded channel.
A read is a `Storage<T>` read operation of the SQLite storage implementation.
A reader worker is a thread of the same SQLite storage implementation that serves reads ([ADR-RUN-0020](../architecture.md) points 16 to 19).
A reader connection is a read-only `rusqlite` connection opened by one reader worker and separate from the write-mode connection.
The outer commit commits the batch transaction containing an `execute` operation.

**MUST statements:**

- The writer thread MUST execute every `rusqlite` call made on the write-mode connection.
- Each read made outside an atomic unit MUST execute on a reader worker's reader connection.
- A successful `execute` reply MUST contain the number of rows affected by that operation.
- The batch transaction containing an `execute` operation MUST have completed its commit when its successful reply is sent.

### Rationale

ADR-RUN-0012 copies the measured atm-core path: one write-mode rusqlite connection on one thread, a bounded channel, `BEGIN IMMEDIATE`, and a reply only after commit. ADR-RUN-0020 points 16 to 19 keep reads on reader workers; reply-after-commit concerns writes only (PRD-STO-3).
### Success Criteria

**Acceptance Criteria:**

- [ ] `cargo test -p sc-runtime-db-sqlite` exits 0, covering one `execute` round-trip and two concurrent `execute` calls with reply-after-commit. The test controls admission and commit rather than relying on sleeps or scheduling order.
- [ ] `cargo test -p sc-runtime-db-sqlite` exits 0 with a test that holds the writer thread inside an uncommitted batch transaction and shows a read still returns the last committed rows, so the read did not run on the write-mode connection. The test releases the writer by an explicit signal, not by sleeps.

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
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: sc-runtime-db-sqlite is the workspace crate at `crates/sc-runtime-db-sqlite`. A dependency entry is a key in any `[dependencies]`, `[dev-dependencies]` or `[build-dependencies]` table of a `Cargo.toml`. The listed historical statements remain nonbinding under the Superseded status and the explicit scope text.

The template's SQLite store uses this writer ([REQ-RUN-0326](../requirements.md)). `sc-runtime` still has no dependency on this crate ([REQ-RUN-0005](../requirements.md)). The text below binds nothing.

**MUST NOT statements:**

- `template/crates/store-sqlite/Cargo.toml` contains no dependency entry for `sc-runtime-db-sqlite`.
- `crates/sc-runtime/Cargo.toml` contains no dependency entry for `sc-runtime-db-sqlite`.

### Rationale

ADR-RUN-0301 binds `template/crates/store-sqlite` to sqlx 0.9 with no write actor; a `StorageBackend` that runs the same SQL on rusqlite and on Postgres is the portable query layer that ADR rejects. `sc-runtime` does not open the writer (ADR-RUN-0012).
### Success Criteria

**Acceptance Criteria:**

- [ ] `rg sc-runtime-db-sqlite template` prints nothing.
- [ ] `sc-lint lint sc-boundary` exits 0 with `sc-runtime-db-sqlite` in the forbidden edges of `sc-runtime`.
