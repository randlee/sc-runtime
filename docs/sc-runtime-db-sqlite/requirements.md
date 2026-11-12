# sc-runtime-db-sqlite Requirements

**Status:** Active

Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0012](../architecture.md). Sprint `b-1` implements these items.

## REQ-DBSQ-0001: Workspace member

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

`crates/sc-runtime-db-sqlite` is a workspace member. This item owns its third-party dependencies: `rusqlite` with the `bundled` feature, and no other dependency. It depends on no other workspace crate.

### Rationale

### Success Criteria

### Related Documents

**Requirements:**

- [REQ-RUN-0005](../requirements.md) — point 4 points here

## REQ-DBSQ-0002: Public items

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

The public items are `SqliteWriter`, `WriterConn`, `SqlValue`, `SqliteWriterError`, `CHANNEL_CAPACITY`, `BATCH_TIME_BUDGET`, `WRITE_DEADLINE`, and `SHUTDOWN_JOIN`, with the signatures in the architecture file.

### Rationale

### Success Criteria

## REQ-DBSQ-0003: Reply after commit

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

The writer thread is the only caller of rusqlite. `execute` replies with the number of rows affected only after the outer commit.

### Rationale

### Success Criteria

## REQ-DBSQ-0004: One commit order

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

A test of two concurrent `execute` calls observes one commit order: the second reply is not sent before the first statement has committed or failed.

### Rationale

### Success Criteria

**Acceptance Criteria:**

- [ ] A test of two concurrent `execute` calls observes one commit order: the second reply is not sent before the first statement has committed or failed.

## REQ-DBSQ-0005: No runtime or template edge

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Template `store-sqlite` does not name this crate. `sc-runtime` does not name it.

### Rationale

### Success Criteria
