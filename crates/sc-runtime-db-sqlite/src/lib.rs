//! Rusqlite write actor: one connection, one thread, reply after commit.
//!
//! This crate does not implement a trait shared with sqlx. Template
//! `store-sqlite` does not depend on it.

#![doc = include_str!("../README.md")]

mod batch;
mod conn;
mod error;
mod value;
mod writer;

#[doc(inline)]
pub use conn::WriterConn;
#[doc(inline)]
pub use error::SqliteWriterError;
#[doc(inline)]
pub use value::SqlValue;
#[doc(inline)]
pub use writer::SqliteWriter;

/// Matches the measured atm-core queue. Callers block in the reply wait, not in a second queue.
pub const CHANNEL_CAPACITY: usize = 256;

/// Drain budget. The writer calls `yield_now` inside this window. It does not sleep.
pub const BATCH_TIME_BUDGET: std::time::Duration = std::time::Duration::from_millis(1);

/// How long `execute` and `transaction` wait for a reply after the call starts.
pub const WRITE_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);

/// How long dropping the last handle waits for the writer thread.
pub const SHUTDOWN_JOIN: std::time::Duration = std::time::Duration::from_secs(5);

#[cfg(test)]
mod tests;
