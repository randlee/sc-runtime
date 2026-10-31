use super::*;

#[test]
fn reload_retry_doubles_per_failure_then_caps_at_one_minute() {
    let now = DateTime::<Utc>::UNIX_EPOCH;
    let mut retry = ReloadRetry::default();
    let mut delays = Vec::new();
    for _ in 0..9 {
        retry.failed(now);
        delays.push(retry.due.map(|due| (due - now).num_seconds()));
    }
    let expected = [1, 2, 4, 8, 16, 32, 60, 60, 60].map(Some);
    assert_eq!(delays, expected);
    assert!(!retry.is_due(now));
    assert!(retry.is_due(now + chrono::Duration::seconds(60)));
}

#[test]
fn reload_retry_delay_never_overflows() {
    for failures in (62..=64).chain([u32::MAX - 1, u32::MAX]) {
        assert_eq!(
            ReloadRetry::delay(failures),
            chrono::Duration::seconds(60),
            "{failures}"
        );
    }
    let now = DateTime::<Utc>::UNIX_EPOCH;
    let mut retry = ReloadRetry {
        failures: u32::MAX,
        due: None,
    };
    retry.failed(now);
    assert_eq!(retry.failures, u32::MAX);
    assert_eq!(retry.due, Some(now + chrono::Duration::seconds(60)));
}

#[test]
fn reload_retry_success_clears_the_backoff() {
    let now = DateTime::<Utc>::UNIX_EPOCH;
    let mut retry = ReloadRetry::default();
    retry.failed(now);
    retry.failed(now);
    retry.succeeded();
    assert!(retry.due.is_none());
    assert!(!retry.is_due(now + chrono::Duration::days(1)));
    retry.failed(now);
    assert_eq!(retry.due, Some(now + chrono::Duration::seconds(1)));
}
