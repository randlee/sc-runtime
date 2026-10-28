//! Named size and time limits on schedule input.
//!
//! The limits bound what one schedule can cost: every reload reads every
//! active row, payload included, into memory. They are enforced once, by
//! the validated types ([`ScheduleName`](crate::ScheduleName),
//! [`CronExpr`](crate::CronExpr), [`RunTimeout`](crate::RunTimeout)) and by
//! the payload check that both [`Scheduler::start`](crate::Scheduler::start)
//! and [`SqliteScheduleStore`](crate::SqliteScheduleStore) apply, and again
//! by table CHECKs.
//!
//! Row count is not capped unless the embedder passes [`ScheduleLimits`]
//! (ADR-RUN-0014). With each row bounded here, memory is linear in a count
//! the caller controls. A caller that accepts schedules from untrusted input
//! should pass a row limit or enforce one before insert. This crate does not
//! read sc-config.

use std::num::NonZeroUsize;
use std::time::Duration;

/// Longest schedule name, in bytes.
pub const MAX_NAME_LEN: usize = 128;

/// Longest cron expression, in bytes.
pub const MAX_EXPR_LEN: usize = 256;

/// Largest payload, in bytes of its compact JSON encoding.
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

/// Largest trusted Monty hook source, in UTF-8 bytes.
///
/// This intentionally matches [`MAX_PAYLOAD_BYTES`]: both values are loaded
/// with every active schedule row, and neither should dominate reload memory.
pub const MAX_HOOK_SOURCE_BYTES: usize = MAX_PAYLOAD_BYTES;

/// Longest run timeout.
pub const MAX_TIMEOUT: Duration = Duration::from_hours(24);

/// Embedder ceiling on schedule rows.
///
/// [`ScheduleLimits::max_schedule_rows`] of [`None`] does not enforce a count.
/// That is the library default ([`Default`]). A generated application maps
/// its sc-config integer with [`ScheduleLimits::from_config_rows`]. `0` in
/// that integer means unlimited. This crate never reads sc-config.
///
/// The count is every row that has not ended, enabled and disabled together.
/// Replacing a name that is already in that count does not change it. Ending
/// a row frees a slot. A lower limit does not delete rows that are already
/// stored; only a new name is rejected until the count is under the limit.
///
/// # Examples
///
/// ```
/// use std::num::NonZeroUsize;
///
/// use sc_cron::ScheduleLimits;
///
/// let limits = ScheduleLimits {
///     max_schedule_rows: NonZeroUsize::new(10),
/// };
/// assert_eq!(limits.max_schedule_rows.map(NonZeroUsize::get), Some(10));
/// assert!(ScheduleLimits::default().max_schedule_rows.is_none());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[must_use]
pub struct ScheduleLimits {
    /// Maximum schedule rows that have not ended.
    ///
    /// [`None`] means unlimited.
    pub max_schedule_rows: Option<NonZeroUsize>,
}

impl ScheduleLimits {
    /// Maps an sc-config integer to a row limit.
    ///
    /// `0` is unlimited. Any other value that fits in [`usize`] is the cap.
    /// A value that does not fit becomes [`usize::MAX`], which still caps
    /// the table.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::num::NonZeroUsize;
    ///
    /// use sc_cron::ScheduleLimits;
    ///
    /// assert_eq!(ScheduleLimits::from_config_rows(0).max_schedule_rows, None);
    /// assert_eq!(
    ///     ScheduleLimits::from_config_rows(10).max_schedule_rows,
    ///     NonZeroUsize::new(10)
    /// );
    /// ```
    pub fn from_config_rows(rows: u64) -> Self {
        let width = usize::try_from(rows).unwrap_or(usize::MAX);
        Self {
            max_schedule_rows: NonZeroUsize::new(width),
        }
    }
}

/// Encodes `payload` as compact JSON, refusing it above [`MAX_PAYLOAD_BYTES`].
pub(crate) fn encode_payload(payload: &serde_json::Value) -> Result<String, crate::CronError> {
    let encoded = payload.to_string();
    if encoded.len() > MAX_PAYLOAD_BYTES {
        return Err(crate::CronError::InvalidSpec(format!(
            "payload is {} bytes; the limit is {MAX_PAYLOAD_BYTES}",
            encoded.len()
        )));
    }
    Ok(encoded)
}
