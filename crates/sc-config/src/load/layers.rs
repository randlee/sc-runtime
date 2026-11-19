//! A layered JSON loader: several files, prefixed environment variables and flags, merged
//! with per-leaf provenance and validated by the caller's decode. The caller supplies the
//! files, the keys each layer accepts and the decode. It parses files, merges and reads
//! overrides like [`Loader`](crate::Loader), after a regular-file check and a capped read.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value};

use super::{key_path, merge, parse_json_file, parse_override_value, set_path};
use crate::ConfigError;

/// Where a resolved setting originated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// CLI flag, including its leading `--`.
    Flag(String),
    /// Environment variable name.
    Env(String),
    /// Configuration file path.
    File(PathBuf),
    /// Built-in value or an unset optional setting.
    Default,
}
impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flag(s) | Self::Env(s) => f.write_str(s),
            Self::File(p) => p.display().fmt(f),
            Self::Default => f.write_str("default"),
        }
    }
}

/// A configuration layer file, whether present or missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerFile {
    /// Its full path.
    pub path: PathBuf,
    /// Whether it exists.
    pub exists: bool,
}

/// How an environment value is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvValue {
    /// Kept as text.
    String,
    /// Parsed as JSON; text when it does not parse.
    Json,
}

/// One accepted dotted path; `*` matches one dynamic name.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    /// The dotted path, such as `local_runners.*.port`.
    pub path: &'static str,
    /// How an environment value for this path is read.
    pub env: EnvValue,
}

/// One layer file and the keys it may contain.
#[derive(Clone, Debug)]
pub struct FileLayer<'a> {
    /// Its full path.
    pub path: PathBuf,
    /// The keys it accepts; any other key is an error.
    pub accepts: &'a [Key],
}

/// Every input; loading never reads the process environment.
#[derive(Clone, Debug)]
pub struct Inputs<'a> {
    /// Layer files, highest precedence first.
    pub files: Vec<FileLayer<'a>>,
    /// Prefix of the variables that may name a setting, such as `DECIDR_`.
    pub env_prefix: &'a str,
    /// Exact variable names mapped to a dotted path, such as `("SC_PROJECT", "project")`.
    pub env_aliases: &'a [(&'a str, &'a str)],
    /// Keys a prefixed variable may name; any other prefixed variable is ignored.
    pub env_accepts: &'a [Key],
    /// Dotted paths to flag values.
    pub flags: BTreeMap<String, Value>,
    /// Optional flag spelling, including `--`, per dotted path; absent paths
    /// display as `--` plus the path with `_` replaced by `-`.
    pub flag_names: BTreeMap<String, String>,
    /// Every key the caller accepts: flags, environment value kinds and resolved roots.
    pub keys: &'a [Key],
    /// The only environment values the loader reads.
    pub environment: &'a [(OsString, OsString)],
}

/// A failed load; a layer's failure is wrapped in [`LayersError::At`] with its source.
#[derive(Debug)]
pub enum LayersError<E> {
    /// Reading or parsing a layer file failed ([`ConfigError::Read`] or [`ConfigError::Parse`]).
    Config(ConfigError),
    /// A layer file is larger than [`MAX_FILE_BYTES`].
    TooLarge {
        /// The file.
        file: PathBuf,
        /// Its size in bytes.
        bytes: u64,
    },
    /// A layer path is not a regular file; it is refused before any read.
    NotAFile {
        /// The path.
        file: PathBuf,
    },
    /// A key is not accepted, or a value cannot be placed at its key.
    InvalidKey {
        /// The dotted key path, or `$`.
        path: String,
        /// What is wrong.
        message: String,
    },
    /// A failure of the input from `source`.
    At {
        /// The flag, variable or file.
        source: Source,
        /// The failure.
        cause: Box<LayersError<E>>,
    },
    /// The caller's decode failed.
    Decode(E),
}

impl<E: fmt::Display> fmt::Display for LayersError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => error.fmt(f),
            Self::TooLarge { file, bytes } => write!(
                f,
                "{} is {bytes} bytes, more than the {MAX_FILE_BYTES} a layer file may have",
                file.display()
            ),
            Self::NotAFile { file } => write!(f, "{} is not a regular file", file.display()),
            Self::InvalidKey { path, message } => write!(f, "invalid value at {path}: {message}"),
            Self::At { source, cause } => write!(f, "configuration input from {source}: {cause}"),
            Self::Decode(error) => error.fmt(f),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for LayersError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config(error) => Some(error),
            Self::At { cause, .. } => Some(cause.as_ref()),
            Self::Decode(error) => Some(error),
            _ => None,
        }
    }
}

/// The largest layer file read.
pub const MAX_FILE_BYTES: u64 = 1024 * 1024;

/// The decoded configuration with the winning source of each setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layered<T> {
    /// Resolved values.
    pub config: T,
    /// Dotted setting path to winning layer.
    pub sources: BTreeMap<String, Source>,
    /// The layer files in lookup order.
    pub files: Vec<LayerFile>,
}

/// One resolved setting with its source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resolved {
    /// Dotted setting path.
    pub path: String,
    /// Human-readable resolved value.
    pub value: String,
    /// Flag, variable, file path, or `default`.
    pub source: String,
}

fn invalid<E>(path: &str, message: impl Into<String>) -> LayersError<E> {
    LayersError::InvalidKey {
        path: path.into(),
        message: message.into(),
    }
}
fn at<E>(source: &Source, cause: LayersError<E>) -> LayersError<E> {
    LayersError::At {
        source: source.clone(),
        cause: Box::new(cause),
    }
}

fn path_matches(pattern: &str, path: &[&str], allow_prefix: bool) -> bool {
    let parts: Vec<_> = pattern.split('.').collect();
    (allow_prefix && path.len() <= parts.len() || path.len() == parts.len())
        && path
            .iter()
            .zip(parts)
            .all(|(actual, expected)| expected == "*" || actual == &expected)
}

fn accepted(keys: &[Key], path: &[&str]) -> bool {
    keys.iter().any(|key| path_matches(key.path, path, true))
}

fn check_object<E>(
    value: &Value,
    path: &mut Vec<String>,
    keys: &[Key],
) -> Result<(), LayersError<E>> {
    let object = value.as_object().ok_or_else(|| {
        invalid(
            path.last().map_or("$", String::as_str),
            "configuration must be a JSON object",
        )
    })?;
    for (key, child) in object {
        path.push(key.clone());
        let parts: Vec<_> = path.iter().map(String::as_str).collect();
        if !accepted(keys, &parts) {
            return Err(invalid(
                &path.join("."),
                "setting is not accepted in this scope",
            ));
        }
        if child.is_object() {
            check_object(child, path, keys)?;
        }
        path.pop();
    }
    Ok(())
}

fn check_keys<E>(value: &Value, keys: &[Key]) -> Result<(), LayersError<E>> {
    check_object(value, &mut Vec::new(), keys)
}

fn env_value_kind(keys: &[Key], path: &[&str]) -> EnvValue {
    keys.iter()
        .find(|key| path_matches(key.path, path, false))
        .map_or(EnvValue::Json, |key| key.env)
}

fn file_value<E>(path: &Path) -> Result<Option<Value>, LayersError<E>> {
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(LayersError::Config(ConfigError::Read {
                path: path.into(),
                source,
            }));
        }
    };
    if !metadata.is_file() {
        return Err(LayersError::NotAFile { file: path.into() });
    }
    if metadata.len() > MAX_FILE_BYTES {
        return Err(LayersError::TooLarge {
            file: path.into(),
            bytes: metadata.len(),
        });
    }
    let read = |source| {
        LayersError::Config(ConfigError::Read {
            path: path.into(),
            source,
        })
    };
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(read)?
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(read)?;
    // The read itself is capped, so a file that grew after the check is still refused.
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(LayersError::TooLarge {
            file: path.into(),
            bytes: bytes.len() as u64,
        });
    }
    parse_json_file(path, &bytes)
        .map(Some)
        .map_err(LayersError::Config)
}

fn put<E>(root: &mut Value, parts: &[&str], value: Value) -> Result<(), LayersError<E>> {
    if parts.iter().any(|part| part.is_empty()) {
        return Err(invalid("$", "empty key in setting path"));
    }
    let path: Vec<String> = parts.iter().map(|part| (*part).to_owned()).collect();
    set_path(root, &path, value)
        .map_err(|_| invalid(parts[0], "nested setting conflicts with a scalar"))
}

// Records the source of every leaf `value` sets when merged over `into`, as `merge` places it.
fn record(
    into: Option<&Value>,
    value: &Value,
    path: &str,
    source: &Source,
    sources: &mut BTreeMap<String, Source>,
) {
    if let (Some(dst), Some(src)) = (into.and_then(Value::as_object), value.as_object()) {
        if src.is_empty() && dst.is_empty() && !path.is_empty() {
            sources.insert(path.into(), source.clone());
        }
        for (key, val) in src {
            let next = if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            };
            record(dst.get(key), val, &next, source, sources);
        }
    } else {
        let prefix = format!("{path}.");
        sources.retain(|key, _| key != path && !key.starts_with(&prefix));
        match value.as_object() {
            Some(obj) if !obj.is_empty() => {
                for (key, val) in obj {
                    record(None, val, &format!("{path}.{key}"), source, sources);
                }
            }
            _ => {
                sources.insert(path.into(), source.clone());
            }
        }
    }
}

fn layer_over(
    into: &mut Value,
    value: &Value,
    source: &Source,
    sources: &mut BTreeMap<String, Source>,
) {
    record(Some(into), value, "", source, sources);
    *into = merge(std::mem::take(into), value.clone());
}

// Names the error path's own source, else its nearest recorded parent's, else the one
// source of all its recorded descendants; otherwise no layer.
fn attributed<E>(
    error: E,
    path: Option<&str>,
    sources: &BTreeMap<String, Source>,
) -> LayersError<E> {
    let source = path.and_then(|path| {
        sources
            .iter()
            .filter(|(key, _)| {
                path.strip_prefix(key.as_str())
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
            })
            .max_by_key(|(key, _)| key.len())
            .map(|(_, source)| source)
            .or_else(|| {
                let below = format!("{path}.");
                let mut descendants = sources.iter().filter(|(key, _)| key.starts_with(&below));
                let (_, first) = descendants.next()?;
                descendants
                    .all(|(_, source)| source == first)
                    .then_some(first)
            })
            .cloned()
    });
    match source {
        Some(source) => at(&source, LayersError::Decode(error)),
        None => LayersError::Decode(error),
    }
}

/// Loads every layer, validating every supplied input even if a higher layer wins.
///
/// Precedence, highest first: flags, environment, then `files` in order. `decode` sees the
/// merged value and then each layer re-applied over it, so a masked invalid value still
/// fails under its own source. Leaves of the decoded value that no layer set are recorded
/// as [`Source::Default`]. `decode_path` gives a decode error's key path, which names the
/// layer that supplied it.
///
/// # Errors
/// File, JSON and key errors wrapped in [`LayersError::At`] with their source, and decode
/// errors, attributed to a layer where one supplied the failing path. No file is written.
pub fn load<T: Serialize, E>(
    inputs: &Inputs<'_>,
    decode: impl Fn(Value) -> Result<T, E>,
    decode_path: impl Fn(&E) -> Option<&str>,
) -> Result<Layered<T>, LayersError<E>> {
    let mut files = Vec::new();
    let mut layers = Vec::new();
    for file in &inputs.files {
        let source = Source::File(file.path.clone());
        let input = file_value(&file.path)?;
        files.push(LayerFile {
            path: file.path.clone(),
            exists: input.is_some(),
        });
        if let Some(value) = input {
            check_keys(&value, file.accepts).map_err(|e| at(&source, e))?;
            layers.push((value, source));
        }
    }
    layers.reverse();
    for (name, raw) in inputs.environment {
        if let Some(layer) = env_layer(inputs, name, raw)? {
            layers.push(layer);
        }
    }
    for (key, value) in &inputs.flags {
        let source = Source::Flag(
            inputs
                .flag_names
                .get(key)
                .cloned()
                .unwrap_or_else(|| format!("--{}", key.replace('_', "-"))),
        );
        let mut object = Value::Object(Map::new());
        put(
            &mut object,
            &key.split('.').collect::<Vec<_>>(),
            value.clone(),
        )
        .map_err(|e| at(&source, e))?;
        check_keys(&object, inputs.keys).map_err(|e| at(&source, e))?;
        layers.push((object, source));
    }

    let mut value = Value::Object(Map::new());
    let mut sources = BTreeMap::new();
    for (layer, source) in &layers {
        layer_over(&mut value, layer, source, &mut sources);
    }
    let config = decode(value.clone()).map_err(|error| {
        let path = decode_path(&error).map(str::to_owned);
        attributed(error, path.as_deref(), &sources)
    })?;
    for (layer, source) in layers.iter().rev().skip(1) {
        let mut checked = value.clone();
        let mut checked_sources = sources.clone();
        layer_over(&mut checked, layer, source, &mut checked_sources);
        decode(checked).map_err(|error| {
            let path = decode_path(&error).map(str::to_owned);
            attributed(error, path.as_deref(), &checked_sources)
        })?;
    }
    let mut layered = Layered {
        config,
        sources,
        files,
    };
    for row in resolved(&layered, inputs.keys) {
        layered.sources.entry(row.path).or_insert(Source::Default);
    }
    Ok(layered)
}

/// One environment variable as a layer, or `None` when it names no accepted setting.
fn env_layer<E>(
    inputs: &Inputs<'_>,
    name: &OsStr,
    raw: &OsStr,
) -> Result<Option<(Value, Source)>, LayersError<E>> {
    let Some(name) = name.to_str() else {
        return Ok(None);
    };
    let alias = inputs.env_aliases.iter().find(|(alias, _)| *alias == name);
    let key = if let Some((_, key)) = alias {
        (*key).to_owned()
    } else if name.starts_with(inputs.env_prefix) {
        // A name with an empty segment names no setting.
        let Ok(segments) = key_path(name, inputs.env_prefix) else {
            return Ok(None);
        };
        segments.join(".")
    } else {
        return Ok(None);
    };
    let parts: Vec<_> = key.split('.').collect();
    // A prefixed variable names one of `env_accepts` or is ignored.
    if alias.is_none() && !accepted(inputs.env_accepts, &parts) {
        return Ok(None);
    }
    let source = Source::Env(name.into());
    let text = raw
        .to_str()
        .ok_or_else(|| at(&source, invalid(&key, "value is not UTF-8")))?;
    let value = match env_value_kind(inputs.keys, &parts) {
        EnvValue::String => Value::String(text.into()),
        EnvValue::Json => parse_override_value(text),
    };
    let mut object = Value::Object(Map::new());
    put(&mut object, &parts, value).map_err(|e| at(&source, e))?;
    check_keys(&object, inputs.keys).map_err(|e| at(&source, e))?;
    Ok(Some((object, source)))
}

/// Resolved settings with their sources: one per leaf of the serialized config under the
/// top-level paths of `keys`, in key order. This performs no I/O.
#[must_use]
pub fn resolved<T: Serialize>(layered: &Layered<T>, keys: &[Key]) -> Vec<Resolved> {
    let config = serde_json::to_value(&layered.config).unwrap_or_default();
    let mut rows = Vec::new();
    for root in keys.iter().map(|key| key.path).filter(|p| !p.contains('.')) {
        if let Some(value) = config.get(root) {
            walk(value, root.to_owned(), &layered.sources, &mut rows);
        }
    }
    rows
}

fn walk(value: &Value, path: String, sources: &BTreeMap<String, Source>, rows: &mut Vec<Resolved>) {
    if let Some(fields) = value.as_object().filter(|fields| !fields.is_empty()) {
        for (key, child) in fields {
            walk(child, format!("{path}.{key}"), sources, rows);
        }
        return;
    }
    let value = match value {
        Value::String(text) => text.clone(),
        Value::Null => "unset".into(),
        other => other.to_string(),
    };
    let source = sources.get(&path).unwrap_or(&Source::Default).to_string();
    rows.push(Resolved {
        path,
        value,
        source,
    });
}
