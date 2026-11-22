//! The layered engine: ordered sources folded into one JSON value.
//!
//! [`Layers`] calls each [`LayerSource`] from the lowest layer to the highest,
//! merges what it returns, and deserialises the final value once. The engine
//! does no I/O; only sources read files or the environment.

use std::fmt;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::error::ConfigError;
use crate::provenance::{Provenance, ResolvedRow, Source, SourceKind};

/// One layer's contribution: a JSON value and where it came from.
#[derive(Clone, Debug)]
pub struct Layer {
    pub(crate) source: Source,
    pub(crate) value: Value,
    replace: Vec<Vec<String>>,
    attributes: Vec<(Vec<String>, Source)>,
}

impl Layer {
    /// Builds a layer whose leaves are attributed to `source`.
    pub fn new(source: Source, value: Value) -> Self {
        Self {
            source,
            value,
            replace: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// Marks `path` so its value replaces the value below instead of merging into it.
    pub(crate) fn replace_at(mut self, path: Vec<String>) -> Self {
        self.replace.push(path);
        self
    }

    /// Attributes the leaves at and below `path` to `source`; the deepest path wins.
    pub(crate) fn attribute(mut self, path: Vec<String>, source: Source) -> Self {
        self.attributes.push((path, source));
        self
    }

    pub(crate) fn source_for(&self, leaf: &[String]) -> &Source {
        self.attributes
            .iter()
            .filter(|(path, _)| leaf.starts_with(path))
            .max_by_key(|(path, _)| path.len())
            .map_or(&self.source, |(_, source)| source)
    }
}

/// A place configuration comes from.
///
/// Implement this to add a keychain, a remote store or another file format.
pub trait LayerSource {
    /// Returns this source's layer, or `Ok(None)` when the source is absent.
    ///
    /// `base` is the value accumulated from the layers below, read-only.
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigError`]; use [`ConfigError::SourceFailed`] for a
    /// failure of your own.
    fn load(&self, base: &Value) -> Result<Option<Layer>, ConfigError>;
}

/// The merged value of a stack with the source of each leaf.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Resolved {
    /// The merged value.
    pub value: Value,
    provenance: Provenance,
    files: Vec<PathBuf>,
    variables: Vec<String>,
}

impl Resolved {
    /// Returns every leaf of the merged value with the source that set it.
    ///
    /// A leaf is a scalar, an array, `null` or an empty object, found in the
    /// highest layer that set it; an environment leaf names its variable.
    /// Rows are sorted by path.
    pub fn rows(&self) -> Vec<ResolvedRow> {
        self.provenance.rows()
    }

    /// Deserialises the merged value into `T`, once.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Deserialize`] listing the files that were read
    /// and the variables that were applied.
    pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, ConfigError> {
        serde_json::from_value(self.value).map_err(|error| ConfigError::Deserialize {
            message: error.to_string(),
            files: self.files,
            variables: self.variables,
        })
    }
}

/// An ordered stack of sources. The last one pushed is the highest layer.
///
/// # Examples
///
/// ```
/// use sc_config::{Layers, Source, ValueSource};
/// use serde_json::json;
///
/// let source = |value| ValueSource::new(Source::default("x"), value);
/// let resolved = Layers::new()
///     .push(source(json!({"a": 1, "b": 1})))
///     .push(source(json!({"a": 2})))
///     .resolve()?;
/// assert_eq!(resolved.value, json!({"a": 2, "b": 1}));
/// # Ok::<(), sc_config::ConfigError>(())
/// ```
#[derive(Default)]
pub struct Layers {
    sources: Vec<Box<dyn LayerSource>>,
}

impl fmt::Debug for Layers {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Layers")
            .field("sources", &self.sources.len())
            .finish()
    }
}

impl Layers {
    /// Creates an empty stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `source` as the new highest layer.
    #[must_use]
    pub fn push(self, source: impl LayerSource + 'static) -> Self {
        self.push_boxed(Box::new(source))
    }

    pub(crate) fn push_boxed(mut self, source: Box<dyn LayerSource>) -> Self {
        self.sources.push(source);
        self
    }

    /// Calls each source once, lowest first, and merges the layers.
    ///
    /// # Errors
    ///
    /// Returns the first error a source returns, unchanged. Sources above it
    /// are not called.
    pub fn resolve(&self) -> Result<Resolved, ConfigError> {
        let mut value = Value::Object(Map::new());
        let mut provenance = Provenance::default();
        let mut files = Vec::new();
        let mut variables = Vec::new();
        for source in &self.sources {
            let Some(layer) = source.load(&value)? else {
                continue;
            };
            if layer.source.kind == SourceKind::File {
                files.push(PathBuf::from(&layer.source.name));
            }
            for (_, origin) in &layer.attributes {
                if origin.kind == SourceKind::Env {
                    variables.push(origin.name.clone());
                }
            }
            for path in &layer.replace {
                remove_path(&mut value, path);
                provenance.remove_under(path);
            }
            provenance.record(&layer, &value);
            value = merge(value, layer.value);
        }
        Ok(Resolved {
            value,
            provenance,
            files,
            variables,
        })
    }

    /// Resolves the stack and deserialises the result into `T`, once.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Layers::resolve`] and [`Resolved::deserialize`].
    pub fn load<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        self.resolve()?.deserialize()
    }
}

/// Merges two objects key by key; in every other case `overlay` wins.
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

/// Deletes the value at `path`, if there is one.
fn remove_path(root: &mut Value, path: &[String]) {
    let Some((last, parents)) = path.split_last() else {
        return;
    };
    let mut node = root;
    for key in parents {
        match node.get_mut(key.as_str()) {
            Some(child) => node = child,
            None => return,
        }
    }
    if let Value::Object(map) = node {
        map.remove(last);
    }
}
