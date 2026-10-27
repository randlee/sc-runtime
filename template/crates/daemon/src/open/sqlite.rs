/// Application configuration for the SQLite layout.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// SQLite file. A relative path is joined to the instance root.
    pub database: PathBuf,
    /// Plain logging values. Example choice: console on, file off.
    #[serde(default)]
    pub logging: sc_observability::LogSettings,
    /// Endpoint fields passed through to the daemon builder.
    #[serde(flatten)]
    pub daemon: DaemonConfig,
}

/// SQLite file this layout opens.
#[derive(Debug)]
pub struct StoreTargets {
    /// Filesystem path. Relative configuration is already joined.
    pub path: PathBuf,
}

/// Resolves the SQLite path. A relative path joins the instance root.
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

/// Resolves the SQLite file for [`open_stores`].
///
/// # Errors
///
/// Returns the transport error when the instance root cannot be resolved.
pub fn store_targets(config: &AppConfig, app: &AppName) -> Result<StoreTargets, TransportError> {
    Ok(StoreTargets {
        path: database_path(config, app)?,
    })
}

/// Opens the SQLite store.
///
/// # Errors
///
/// Returns the daemon stores error when the file cannot be opened.
pub async fn open_stores(
    targets: StoreTargets,
) -> Result<service::Stores, sc_runtime::RuntimeError> {
    service::Stores::open(targets.path)
        .await
        .map_err(map_store_open)
}
