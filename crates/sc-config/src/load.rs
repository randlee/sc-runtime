use std::ffi::OsString;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

use crate::app_name::AppName;
use crate::error::ConfigError;
use crate::layers::Layers;
use crate::sources::{EnvSource, FileSource};

/// Explicit configuration directory and environment.
///
/// `load` is the process convenience wrapper. `Loader` is the form tests and
/// parallel callers use, because it does not read the process environment or
/// the current directory except for the path the caller already built.
///
/// `Loader` is a preset stack on [`Layers`]: see [`Loader::layers`].
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use sc_config::{AppName, Loader};
///
/// let app = AppName::new("my-app")?;
/// let loader = Loader::new(app, PathBuf::from("config"), Vec::new());
/// let value: serde_json::Value = loader.load()?;
/// # let _ = value;
/// # Ok::<(), sc_config::ConfigError>(())
/// ```
#[derive(Clone, Debug)]
pub struct Loader {
    app: AppName,
    config_dir: PathBuf,
    environment: Vec<(OsString, OsString)>,
}

impl Loader {
    /// Stores the application name, config directory, and environment pairs.
    ///
    /// The constructor does not read files and cannot fail. `app` is already
    /// validated. `environment` is the only override source `load` will see.
    pub fn new(app: AppName, config_dir: PathBuf, environment: Vec<(OsString, OsString)>) -> Self {
        Self {
            app,
            config_dir,
            environment,
        }
    }

    /// Returns the loader's stack: `default.json` (required), `local.json`
    /// (optional), then the environment variables of the app.
    ///
    /// Push more sources onto it to layer them above the environment.
    pub fn layers(&self) -> Layers {
        Layers::new()
            .push(FileSource::new(self.config_dir.join("default.json")))
            .push(FileSource::new(self.config_dir.join("local.json")).optional())
            .push(EnvSource::new(
                self.app.env_prefix(),
                self.environment.clone(),
            ))
    }

    /// Loads `T` from `default.json`, optional `local.json`, and `environment`.
    ///
    /// File merge and environment rules match [`load`]. Overrides are applied
    /// in ascending order of the variable name's encoded bytes, so a longer
    /// name that extends a shorter one wins.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when the default file is missing, a file cannot
    /// be read or parsed, an override cannot be applied, a prefix-matching
    /// variable is not Unicode, or the merged document does not match `T`.
    pub fn load<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        let default_path = self.config_dir.join("default.json");
        match self.layers().load() {
            Err(ConfigError::MissingFile { path }) if path == default_path => {
                Err(ConfigError::MissingDefault { path })
            }
            other => other,
        }
    }
}

/// Loads `T` from `./config` and the process environment.
///
/// The config directory is `config` under the current working directory.
/// Environment variables come from [`std::env::vars_os`]. The application
/// name selects only the variable prefix.
///
/// # Errors
///
/// Returns the same failures as [`Loader::load`]. Also returns
/// [`ConfigError::Read`] when the current directory cannot be read.
///
/// # Examples
///
/// ```no_run
/// use sc_config::{load, AppName};
///
/// let app = AppName::new("my-app")?;
/// let value: serde_json::Value = load(app)?;
/// # let _ = value;
/// # Ok::<(), sc_config::ConfigError>(())
/// ```
pub fn load<T: DeserializeOwned>(app: AppName) -> Result<T, ConfigError> {
    let current = std::env::current_dir().map_err(|source| ConfigError::Read {
        path: PathBuf::from("."),
        source,
    })?;
    let environment = std::env::vars_os().collect();
    Loader::new(app, current.join("config"), environment).load()
}
