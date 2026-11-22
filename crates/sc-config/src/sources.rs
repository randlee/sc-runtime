//! The sources `sc-config` ships: [`FileSource`], [`EnvSource`] and [`ValueSource`].

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::error::ConfigError;
use crate::layers::{Layer, LayerSource};
use crate::provenance::{KeyPath, Source, SourceKind, leaves};

/// One JSON file.
///
/// The file is required unless [`optional`](Self::optional) is called.
///
/// # Examples
///
/// ```no_run
/// use sc_config::{FileSource, Layers};
///
/// let layers = Layers::new().push(FileSource::new("config/app.json").optional());
/// let value: serde_json::Value = layers.load()?;
/// # let _ = value;
/// # Ok::<(), sc_config::ConfigError>(())
/// ```
#[derive(Clone, Debug)]
pub struct FileSource {
    path: PathBuf,
    optional: bool,
    accepts: Option<Vec<KeyPath>>,
}

impl FileSource {
    /// Reads the JSON file at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            optional: false,
            accepts: None,
        }
    }

    /// Makes an absent file contribute nothing instead of failing.
    #[must_use]
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Limits the file to the given key paths.
    ///
    /// A leaf at or below one of `paths` is accepted; any other leaf is
    /// [`ConfigError::UnknownKey`]. Without this call every key is allowed.
    #[must_use]
    pub fn accepts<P: Into<KeyPath>>(mut self, paths: impl IntoIterator<Item = P>) -> Self {
        self.accepts = Some(paths.into_iter().map(Into::into).collect());
        self
    }

    fn read(&self) -> Result<Option<Value>, ConfigError> {
        let bytes = match std::fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return if self.optional {
                    Ok(None)
                } else {
                    Err(ConfigError::MissingFile {
                        path: self.path.clone(),
                    })
                };
            }
            Err(source) => {
                return Err(ConfigError::Read {
                    path: self.path.clone(),
                    source,
                });
            }
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| ConfigError::Parse {
                path: self.path.clone(),
                line: error.line(),
                column: error.column(),
                message: error.to_string(),
            })
    }

    fn check_accepted(&self, value: &Value) -> Result<(), ConfigError> {
        let Some(accepted) = &self.accepts else {
            return Ok(());
        };
        for (segments, _) in leaves(value) {
            if !accepted.iter().any(|ok| segments.starts_with(&ok.0)) {
                return Err(ConfigError::UnknownKey {
                    path: self.path.clone(),
                    key: KeyPath(segments).to_string(),
                });
            }
        }
        Ok(())
    }
}

impl LayerSource for FileSource {
    fn source(&self) -> Source {
        file_source(&self.path)
    }

    fn load(&self, _base: &Value) -> Result<Option<Layer>, ConfigError> {
        let Some(value) = self.read()? else {
            return Ok(None);
        };
        self.check_accepted(&value)?;
        Ok(Some(Layer::new(self.source(), value)))
    }
}

fn file_source(path: &Path) -> Source {
    Source::new(SourceKind::File, path.to_string_lossy())
}

/// Environment variables from an explicit list. Never reads the process environment.
///
/// A variable whose name starts with the prefix sets the key path named by the
/// rest of its name: the rest is split on `__` and each segment is lower-cased,
/// so `MY_APP__DAEMON__PORT` sets `daemon.port`. The value is parsed as JSON,
/// or taken as a string when it is not JSON. Variables apply in ascending
/// order of their names' bytes, and a set value replaces what is below it. An
/// empty segment, or a path through a value that is not an object, is
/// [`ConfigError::Override`]; a matching name or value that is not Unicode is
/// [`ConfigError::NonUnicode`]. An empty prefix matches no variable, so the
/// source then applies only its aliases.
///
/// # Examples
///
/// ```
/// use std::ffi::OsString;
///
/// use sc_config::{EnvSource, Layers};
///
/// let environment = vec![(OsString::from("MY_APP__A__B"), OsString::from("2"))];
/// let value: serde_json::Value = Layers::new()
///     .push(EnvSource::new("MY_APP__", environment))
///     .load()?;
/// assert_eq!(value, serde_json::json!({"a": {"b": 2}}));
/// # Ok::<(), sc_config::ConfigError>(())
/// ```
#[derive(Clone, Debug)]
pub struct EnvSource {
    prefix: String,
    environment: Vec<(OsString, OsString)>,
    aliases: Vec<(String, KeyPath)>,
}

impl EnvSource {
    /// Matches `prefix` literally against the names in `environment`.
    pub fn new(prefix: impl Into<String>, environment: Vec<(OsString, OsString)>) -> Self {
        Self {
            prefix: prefix.into(),
            environment,
            aliases: Vec::new(),
        }
    }

    /// Maps the variable named exactly `variable` to the key path `path`.
    ///
    /// Aliases apply after prefixed variables, in ascending order of the
    /// variable name's bytes. The value is parsed the same way.
    #[must_use]
    pub fn alias(mut self, variable: impl Into<String>, path: impl Into<KeyPath>) -> Self {
        self.aliases.push((variable.into(), path.into()));
        self
    }

    /// Lists the variables to apply, in order, with the alias path if any.
    fn matches(&self) -> Vec<(&OsString, &OsString, Option<&KeyPath>)> {
        let mut prefixed: Vec<_> = self
            .environment
            .iter()
            .filter(|(name, _)| {
                !self.prefix.is_empty()
                    && name.as_encoded_bytes().starts_with(self.prefix.as_bytes())
            })
            .map(|(name, value)| (name, value, None))
            .collect();
        prefixed.sort_by(|left, right| left.0.as_encoded_bytes().cmp(right.0.as_encoded_bytes()));
        let mut aliases: Vec<_> = self.aliases.iter().collect();
        aliases.sort_by(|left, right| left.0.cmp(&right.0));
        for (variable, path) in aliases {
            for (name, value) in &self.environment {
                if name.as_os_str() == variable.as_str() {
                    prefixed.push((name, value, Some(path)));
                }
            }
        }
        prefixed
    }
}

impl LayerSource for EnvSource {
    fn source(&self) -> Source {
        Source::new(SourceKind::Env, self.prefix.clone())
    }

    fn load(&self, base: &Value) -> Result<Option<Layer>, ConfigError> {
        let matches = self.matches();
        if matches.is_empty() {
            return Ok(None);
        }
        // `scratch` checks each path against what is below; `overlay` is the layer.
        let mut scratch = base.clone();
        let mut overlay = Value::Object(Map::new());
        let mut applied = Vec::new();
        for (name, value, alias) in matches {
            let unicode = name.to_str().zip(value.to_str());
            let Some((name, value)) = unicode else {
                return Err(ConfigError::NonUnicode {
                    variable: name.clone(),
                });
            };
            let segments = match alias {
                Some(path) => path.0.clone(),
                None => key_segments(name, &self.prefix),
            };
            let parsed = parse_value(value);
            let failed = || ConfigError::Override {
                variable: name.to_owned(),
            };
            set_path(&mut scratch, &segments, parsed.clone()).map_err(|()| failed())?;
            set_path(&mut overlay, &segments, parsed).map_err(|()| failed())?;
            applied.push((KeyPath(segments), Source::new(SourceKind::Env, name)));
        }
        let mut layer = Layer::new(self.source(), overlay);
        for (path, origin) in applied {
            layer = layer.replace_at(path.clone()).attribute(path, origin);
        }
        Ok(Some(layer))
    }
}

/// Splits the part of `name` after `prefix` at `__` and lower-cases each segment.
fn key_segments(name: &str, prefix: &str) -> Vec<String> {
    name.get(prefix.len()..)
        .unwrap_or("")
        .split("__")
        .map(|segment| segment.to_ascii_lowercase())
        .collect()
}

fn parse_value(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_owned()))
}

/// Sets `value` at `path`, creating missing objects on the way.
///
/// Fails on an empty path or empty segment, or when the walk meets a value
/// that is not an object.
fn set_path(root: &mut Value, path: &[String], value: Value) -> Result<(), ()> {
    let Some((leaf, parents)) = path.split_last() else {
        return Err(());
    };
    if path.iter().any(String::is_empty) {
        return Err(());
    }
    let mut node = root;
    for key in parents {
        let Value::Object(map) = node else {
            return Err(());
        };
        node = map
            .entry(key.clone())
            .or_insert_with(|| Value::Object(Map::new()));
    }
    let Value::Object(map) = node else {
        return Err(());
    };
    map.insert(leaf.clone(), value);
    Ok(())
}

/// A caller-built JSON value, for defaults and parsed command-line flags.
///
/// `sc-config` does not parse command lines.
///
/// # Examples
///
/// ```
/// use sc_config::{Layers, Source, SourceKind, ValueSource};
///
/// let flags = ValueSource::new(
///     Source::new(SourceKind::Flag, "--port"),
///     serde_json::json!({"port": 9000}),
/// );
/// let value: serde_json::Value = Layers::new().push(flags).load()?;
/// assert_eq!(value, serde_json::json!({"port": 9000}));
/// # Ok::<(), sc_config::ConfigError>(())
/// ```
#[derive(Clone, Debug)]
pub struct ValueSource {
    source: Source,
    value: Value,
}

impl ValueSource {
    /// Contributes `value`, attributed to `source`.
    pub fn new(source: Source, value: Value) -> Self {
        Self { source, value }
    }
}

impl LayerSource for ValueSource {
    fn source(&self) -> Source {
        self.source.clone()
    }

    fn load(&self, _base: &Value) -> Result<Option<Layer>, ConfigError> {
        Ok(Some(Layer::new(self.source.clone(), self.value.clone())))
    }
}
