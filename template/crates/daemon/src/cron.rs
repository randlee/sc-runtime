//! Schedule row limit read from the application's sc-config.
//!
//! No layout opens `sc_cron::Scheduler` yet. The layout that does passes
//! [`CronConfig::row_limit`] as `sc_cron::ScheduleLimits::max_schedule_rows`.
//! This module does not depend on `sc-cron`. `0` means unlimited. The default
//! is 10.

use std::num::NonZeroUsize;

use serde::Deserialize;

/// `cron` object in the application configuration file.
#[derive(Debug, Deserialize)]
pub struct CronConfig {
    /// Maximum schedule rows, enabled and disabled.
    ///
    /// `0` means unlimited. When the key is absent the value is 10.
    #[serde(default = "default_max_schedule_rows")]
    pub max_schedule_rows: u64,
}

fn default_max_schedule_rows() -> u64 {
    10
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
    /// `0` is [`None`]. Any other value that fits in [`usize`] is [`Some`].
    /// A value that does not fit becomes [`usize::MAX`], which still caps
    /// the table. This matches `sc_cron::ScheduleLimits::from_config_rows`.
    pub fn row_limit(&self) -> Option<NonZeroUsize> {
        let width = usize::try_from(self.max_schedule_rows).unwrap_or(usize::MAX);
        NonZeroUsize::new(width)
    }
}

#[cfg(test)]
mod tests {
    use super::CronConfig;

    #[test]
    fn zero_is_unlimited_and_the_default_is_ten() {
        let missing: CronConfig = serde_json::from_str("{}").expect("json");
        assert_eq!(missing.max_schedule_rows, 10);
        assert_eq!(
            missing.row_limit().map(std::num::NonZeroUsize::get),
            Some(10)
        );

        let zero: CronConfig = serde_json::from_str(r#"{"max_schedule_rows":0}"#).expect("json");
        assert!(zero.row_limit().is_none());

        let raised: CronConfig = serde_json::from_str(r#"{"max_schedule_rows":25}"#).expect("json");
        assert_eq!(
            raised.row_limit().map(std::num::NonZeroUsize::get),
            Some(25)
        );
    }
}
