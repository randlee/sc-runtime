/// Application configuration for the no-database layout.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// Plain logging values. Defaults: console and file on; logs under `<instance>/logs/`.
    #[serde(default)]
    pub logging: sc_observability::LogSettings,
    /// Endpoint fields passed through to the daemon builder.
    #[serde(flatten)]
    pub daemon: DaemonConfig,
    /// Schedule row ceiling from sc-config. See [`cron::CronConfig`].
    #[serde(default)]
    pub cron: cron::CronConfig,
}

/// No database target. Opening the in-memory store takes no path and no URL.
#[derive(Debug)]
pub struct StoreTargets;

/// This layout has no database configuration to resolve.
///
/// # Errors
///
/// This function does not fail.
pub fn store_targets(_config: &AppConfig, _app: &AppName) -> Result<StoreTargets, TransportError> {
    Ok(StoreTargets)
}

/// Opens the in-memory store. That open cannot fail.
///
/// # Errors
///
/// This function does not return an error. The `Result` matches the other
/// layouts' daemon entry point.
pub async fn open_stores(
    _targets: StoreTargets,
) -> Result<service::Stores, sc_runtime::RuntimeError> {
    Ok(service::Stores::open().await)
}
