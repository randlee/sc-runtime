/// Application configuration for SQLite plus MySQL.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// SQLite file. A relative path is joined to the instance root.
    pub database: PathBuf,
    /// MySQL URL. Not joined onto the instance root.
    pub remote_database: String,
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

/// The two stores this layout opens.
#[derive(Debug)]
pub struct StoreTargets {
    /// SQLite filesystem path.
    pub sqlite: PathBuf,
    /// MySQL URL.
    pub remote: String,
}

/// Resolves the SQLite path. A relative path joins the instance root.
///
/// The remote URL is not passed here. [`store_targets`] copies it unchanged.
///
/// # Errors
///
/// Returns the transport error when the instance root cannot be resolved.
pub fn database_path(config: &AppConfig, app: &AppName) -> Result<PathBuf, TransportError> {
    if config.database.is_absolute() {
        return Ok(config.database.clone());
    }
    Ok(instance_root(config, app)?.join(&config.database))
}

/// Resolves the SQLite path and copies the MySQL URL.
///
/// The URL is not inspected for a `mysql://` prefix and is not joined to the
/// instance root.
///
/// # Errors
///
/// Returns the transport error when the instance root cannot be resolved.
pub fn store_targets(config: &AppConfig, app: &AppName) -> Result<StoreTargets, TransportError> {
    Ok(StoreTargets {
        sqlite: database_path(config, app)?,
        remote: config.remote_database.clone(),
    })
}

/// Opens SQLite and MySQL.
///
/// # Errors
///
/// Returns the daemon stores error when either database cannot be opened.
pub async fn open_stores(
    targets: StoreTargets,
) -> Result<service::Stores, sc_runtime::RuntimeError> {
    service::Stores::open(targets.sqlite, targets.remote)
        .await
        .map_err(map_store_open)
}
