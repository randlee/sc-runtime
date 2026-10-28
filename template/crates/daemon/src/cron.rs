//! Schedule row limit read from the application's sc-config.
//!
//! No layout opens `sc_cron::Scheduler` yet. The layout that does passes
//! [`CronConfig::row_limit`] as `sc_cron::ScheduleLimits::max_schedule_rows`
//! and the same injected logger handle as `Daemon::builder` (see
//! `daemon::observability` and `docs/sc-runtime/observability-injection.md`).
//! This module does not depend on `sc-cron`. JSON `null` and `0` mean
//! unlimited. A missing key defaults to 10.

use std::num::NonZeroUsize;

use serde::Deserialize;

/// `cron` object in the application configuration file.
#[derive(Debug, Deserialize)]
pub struct CronConfig {
    /// Maximum schedule rows, enabled and disabled.
    ///
    /// [`None`] is JSON `null` and means unlimited. `Some(0)` is also
    /// unlimited. `Some(n)` for `n > 0` caps the table at `n`. When the key
    /// is absent the value is `Some(10)`.
    #[serde(default = "default_max_schedule_rows")]
    pub max_schedule_rows: Option<u64>,
}

fn default_max_schedule_rows() -> Option<u64> {
    Some(10)
}

impl Default for CronConfig {
    fn default() -> Self {
        Self {
            max_schedule_rows: default_max_schedule_rows(),
        }
    }
}

impl CronConfig {
    /// Maps `max_schedule_rows` to the `ScheduleLimits` field of the same name.
    ///
    /// JSON `null` and `0` are [`None`]. Any other value that fits in
    /// [`usize`] is [`Some`]. A value that does not fit becomes
    /// [`usize::MAX`], which still caps the table. This is the only mapping
    /// from the sc-config integer. `sc-cron` receives the resulting
    /// `ScheduleLimits` and does not read sc-config.
    pub fn row_limit(&self) -> Option<NonZeroUsize> {
        let rows = self.max_schedule_rows?;
        let width = usize::try_from(rows).unwrap_or(usize::MAX);
        NonZeroUsize::new(width)
    }
}

#[cfg(test)]
mod tests {
    use super::CronConfig;

    #[test]
    fn zero_is_unlimited_and_the_default_is_ten() {
        let missing: CronConfig = serde_json::from_str("{}").expect("json");
        assert_eq!(missing.max_schedule_rows, Some(10));
        assert_eq!(
            missing.row_limit().map(std::num::NonZeroUsize::get),
            Some(10)
        );

        let zero: CronConfig = serde_json::from_str(r#"{"max_schedule_rows":0}"#).expect("json");
        assert_eq!(zero.max_schedule_rows, Some(0));
        assert!(zero.row_limit().is_none());

        let raised: CronConfig = serde_json::from_str(r#"{"max_schedule_rows":25}"#).expect("json");
        assert_eq!(
            raised.row_limit().map(std::num::NonZeroUsize::get),
            Some(25)
        );
    }

    #[test]
    fn json_null_is_unlimited() {
        let unlimited: CronConfig =
            serde_json::from_str(r#"{"max_schedule_rows": null}"#).expect("json");
        assert!(unlimited.max_schedule_rows.is_none());
        assert!(unlimited.row_limit().is_none());
    }
}
