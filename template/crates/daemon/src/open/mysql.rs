/// Application configuration for the MySQL layout.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// MySQL URL (`mysql://`). Not a filesystem path.
    pub database: String,
    /// Plain logging values. Example choice: console on, file off.
    #[serde(default)]
    pub logging: sc_observability::LogSettings,
    /// Endpoint fields passed through to the daemon builder.
    #[serde(flatten)]
    pub daemon: DaemonConfig,
    /// Schedule row ceiling from sc-config. See [`cron::CronConfig`].
    #[serde(default)]
    pub cron: cron::CronConfig,
}

/// MySQL URL this layout opens.
#[derive(Debug)]
pub struct StoreTargets {
    /// Database URL. It is not joined onto the instance root.
    pub url: String,
}

/// Returns the configured MySQL URL unchanged.
#[must_use]
pub fn database_url(config: &AppConfig) -> &str {
    config.database.as_str()
}

/// Passes the MySQL URL through. This does not join it to a directory.
///
/// # Errors
///
/// This layout's target is a URL, so resolving it does not fail.
pub fn store_targets(config: &AppConfig, _app: &AppName) -> Result<StoreTargets, TransportError> {
    Ok(StoreTargets {
        url: database_url(config).to_owned(),
    })
}

/// Opens the MySQL store.
///
/// # Errors
///
/// Returns the daemon stores error when the URL cannot be opened.
pub async fn open_stores(
    targets: StoreTargets,
) -> Result<service::Stores, sc_runtime::RuntimeError> {
    service::Stores::open(targets.url)
        .await
        .map_err(map_store_open)
}
