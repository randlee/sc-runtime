//! The sources `sc-config` ships: [`FileSource`], [`EnvSource`] and [`ValueSource`].

use std::ffi::OsString;
use std::path::PathBuf;

use serde_json::{Map, Value};

use crate::error::ConfigError;
use crate::layers::{Layer, LayerSource};
use crate::provenance::{Source, leaves};

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
    accepts: Option<Vec<Vec<String>>>,
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

    /// Limits the file to the given key paths, each a slice of segments.
    ///
    /// A leaf at or below one of `paths` is accepted; any other leaf is
    /// [`ConfigError::UnknownKey`]. Without this call every key is allowed.
    #[must_use]
    pub fn accepts<'a, P: AsRef<[&'a str]>>(mut self, paths: impl IntoIterator<Item = P>) -> Self {
        self.accepts = Some(paths.into_iter().map(strs).collect());
        self
    }

    pub(crate) fn with_accepted(mut self, accepted: Option<Vec<Vec<String>>>) -> Self {
        self.accepts = accepted;
        self
    }

    fn read(&self) -> Result<Option<Value>, ConfigError> {
        let bytes = match std::fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return if self.optional {
                    Ok(None)
                } else {
                    Err(ConfigError::MissingDefault {
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
            if !accepted.iter().any(|ok| segments.starts_with(ok)) {
                return Err(ConfigError::UnknownKey {
                    path: self.path.clone(),
                    key: segments.join("."),
                });
            }
        }
        Ok(())
    }
}

impl LayerSource for FileSource {
    fn load(&self, _base: &Value) -> Result<Option<Layer>, ConfigError> {
        let Some(value) = self.read()? else {
            return Ok(None);
        };
        self.check_accepted(&value)?;
        Ok(Some(
            Layer::new(Source::file(&self.path), value).read_from(self.path.clone()),
        ))
    }
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
/// [`ConfigError::NonUnicode`]. An empty prefix matches every variable.
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
    /// `None` matches no variable, so only aliases apply.
    prefix: Option<String>,
    environment: Vec<(OsString, OsString)>,
    aliases: Vec<(String, Vec<String>)>,
    /// A variable never applied, even when it matches.
    excluded: Option<OsString>,
}

impl EnvSource {
    /// Matches `prefix` literally against the names in `environment`.
    pub fn new(prefix: impl Into<String>, environment: Vec<(OsString, OsString)>) -> Self {
        Self {
            prefix: Some(prefix.into()),
            environment,
            aliases: Vec::new(),
            excluded: None,
        }
    }

    /// A source with no prefix match, holding only the aliases added to it.
    pub(crate) fn aliases_only(environment: Vec<(OsString, OsString)>) -> Self {
        Self {
            prefix: None,
            environment,
            aliases: Vec::new(),
            excluded: None,
        }
    }

    /// Maps the variable named exactly `variable` to the key path `path`, a slice of segments.
    ///
    /// Aliases apply after prefixed variables, in ascending order of the
    /// variable name's bytes. The value is parsed the same way.
    #[must_use]
    pub fn alias<'a>(self, variable: impl Into<String>, path: impl AsRef<[&'a str]>) -> Self {
        self.alias_path(variable.into(), strs(path))
    }

    pub(crate) fn alias_path(mut self, variable: String, path: Vec<String>) -> Self {
        self.aliases.push((variable, path));
        self
    }

    /// Never applies the variable `name`, even when it matches the prefix.
    pub(crate) fn exclude(mut self, name: OsString) -> Self {
        self.excluded = Some(name);
        self
    }

    /// Lists the variables to apply, in order, with the alias path if any.
    fn matches(&self) -> Vec<(&OsString, &OsString, Option<&Vec<String>>)> {
        let prefix = self.prefix.as_deref();
        let mut prefixed: Vec<_> = self
            .environment
            .iter()
            .filter(|(name, _)| self.excluded.as_ref() != Some(name))
            .filter(|(name, _)| {
                prefix.is_some_and(|prefix| name.as_encoded_bytes().starts_with(prefix.as_bytes()))
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
                Some(path) => path.clone(),
                None => key_segments(name, self.prefix.as_deref().unwrap_or_default()),
            };
            let parsed = parse_value(value);
            let failed = || ConfigError::Override {
                variable: name.to_owned(),
            };
            set_path(&mut scratch, &segments, parsed.clone()).map_err(|()| failed())?;
            set_path(&mut overlay, &segments, parsed).map_err(|()| failed())?;
            applied.push((segments, Source::env(name)));
        }
        let mut layer = Layer::new(
            Source::env(self.prefix.clone().unwrap_or_default()),
            overlay,
        );
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
/// use sc_config::{Layers, Source, ValueSource};
///
/// let flags = ValueSource::new(
///     Source::flag("--port"),
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
    fn load(&self, _base: &Value) -> Result<Option<Layer>, ConfigError> {
        Ok(Some(Layer::new(self.source.clone(), self.value.clone())))
    }
}

/// Copies a slice of key segments.
pub(crate) fn strs<'a>(segments: impl AsRef<[&'a str]>) -> Vec<String> {
    segments.as_ref().iter().map(|s| (*s).to_owned()).collect()
}
