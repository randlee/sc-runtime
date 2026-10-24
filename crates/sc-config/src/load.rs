use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::app_name::AppName;
use crate::error::ConfigError;

/// Explicit configuration directory and environment.
///
/// `load` is the process convenience wrapper. `Loader` is the form tests and
/// parallel callers use, because it does not read the process environment or
/// the current directory except for the path the caller already built.
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
        let local_path = self.config_dir.join("local.json");
        let mut files = Vec::new();
        let default_value = read_required_json(&default_path)?;
        files.push(default_path);
        let mut document = default_value;
        if let Some(local_value) = read_optional_json(&local_path)? {
            document = merge(document, local_value);
            files.push(local_path);
        }
        let variables = apply_overrides(&mut document, &self.app, &self.environment)?;
        match serde_json::from_value(document) {
            Ok(typed) => Ok(typed),
            Err(error) => Err(ConfigError::Deserialize {
                message: error.to_string(),
                files,
                variables,
            }),
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

fn read_required_json(path: &Path) -> Result<Value, ConfigError> {
    match std::fs::read(path) {
        Ok(bytes) => parse_json_file(path, &bytes),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            Err(ConfigError::MissingDefault {
                path: path.to_path_buf(),
            })
        }
        Err(source) => Err(ConfigError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn read_optional_json(path: &Path) -> Result<Option<Value>, ConfigError> {
    match std::fs::read(path) {
        Ok(bytes) => parse_json_file(path, &bytes).map(Some),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ConfigError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn parse_json_file(path: &Path, bytes: &[u8]) -> Result<Value, ConfigError> {
    match serde_json::from_slice(bytes) {
        Ok(value) => Ok(value),
        Err(error) => Err(ConfigError::Parse {
            path: path.to_path_buf(),
            line: error.line(),
            column: error.column(),
            message: error.to_string(),
        }),
    }
}

fn merge(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Value::Object(mut base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                let merged = match base_map.remove(&key) {
                    Some(base_value) => merge(base_value, overlay_value),
                    None => overlay_value,
                };
                base_map.insert(key, merged);
            }
            Value::Object(base_map)
        }
        (_base, overlay) => overlay,
    }
}

fn apply_overrides(
    document: &mut Value,
    app: &AppName,
    environment: &[(OsString, OsString)],
) -> Result<Vec<String>, ConfigError> {
    let prefix = app.env_prefix();
    let mut matched = Vec::new();
    for (name, value) in environment {
        if name.as_encoded_bytes().starts_with(prefix.as_bytes()) {
            matched.push((name, value));
        }
    }
    matched.sort_by(|left, right| left.0.as_encoded_bytes().cmp(right.0.as_encoded_bytes()));

    let mut applied = Vec::new();
    for (name, value) in matched {
        let Some(name_str) = name.to_str() else {
            return Err(ConfigError::NonUnicode {
                variable: name.clone(),
            });
        };
        let Some(value_str) = value.to_str() else {
            return Err(ConfigError::NonUnicode {
                variable: name.clone(),
            });
        };
        let path = key_path(name_str, &prefix)?;
        let parsed = parse_override_value(value_str);
        match set_path(document, &path, parsed) {
            Ok(()) => {}
            Err(SetPathError::EmptyPath | SetPathError::NotAnObject) => {
                return Err(ConfigError::Override {
                    variable: name_str.to_owned(),
                });
            }
        }
        applied.push(name_str.to_owned());
    }
    Ok(applied)
}

fn key_path(name: &str, prefix: &str) -> Result<Vec<String>, ConfigError> {
    // Callers pass names that already start with this ASCII prefix, so the
    // suffix is the key path. A shorter name yields an empty segment below.
    let rest = name.get(prefix.len()..).unwrap_or("");
    let segments: Vec<String> = rest
        .split("__")
        .map(|segment| segment.to_ascii_lowercase())
        .collect();
    if segments.iter().any(|segment| segment.is_empty()) {
        return Err(ConfigError::Override {
            variable: name.to_owned(),
        });
    }
    Ok(segments)
}

fn parse_override_value(raw: &str) -> Value {
    match serde_json::from_str(raw) {
        Ok(value) => value,
        Err(_) => Value::String(raw.to_owned()),
    }
}

struct Frame {
    key: String,
    map: Map<String, Value>,
    existed: bool,
}

/// Failure from writing one override into the merged JSON document.
#[derive(Debug)]
enum SetPathError {
    /// The key path has no leaf.
    EmptyPath,
    /// The root or an intermediate value is not a JSON object.
    NotAnObject,
}

fn set_path(document: &mut Value, path: &[String], value: Value) -> Result<(), SetPathError> {
    let Some((leaf, parents)) = path.split_last() else {
        return Err(SetPathError::EmptyPath);
    };
    if !matches!(document, Value::Object(_)) {
        return Err(SetPathError::NotAnObject);
    }

    let mut stack = Vec::new();
    let mut current = std::mem::replace(document, Value::Null);
    for key in parents {
        let mut map = match current {
            Value::Object(map) => map,
            other => {
                reassemble(document, stack, other, false);
                return Err(SetPathError::NotAnObject);
            }
        };
        let (child, existed) = match map.remove(key) {
            Some(child) => (child, true),
            None => (Value::Object(Map::new()), false),
        };
        stack.push(Frame {
            key: key.clone(),
            map,
            existed,
        });
        current = child;
    }

    let current = match current {
        Value::Object(mut map) => {
            map.insert(leaf.clone(), value);
            Value::Object(map)
        }
        other => {
            reassemble(document, stack, other, false);
            return Err(SetPathError::NotAnObject);
        }
    };
    reassemble(document, stack, current, true);
    Ok(())
}

fn reassemble(document: &mut Value, stack: Vec<Frame>, mut current: Value, commit_new: bool) {
    for frame in stack.into_iter().rev() {
        let mut map = frame.map;
        if frame.existed || commit_new {
            map.insert(frame.key, current);
        }
        current = Value::Object(map);
    }
    *document = current;
}
