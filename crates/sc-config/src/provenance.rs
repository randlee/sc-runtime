//! Where each resolved setting came from: sources, key paths and per-leaf provenance.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::layers::Layer;

/// Kind of a [`Source`].
///
/// A consumer's own kind uses [`SourceKind::Custom`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub enum SourceKind {
    /// A file.
    File,
    /// An environment variable.
    Env,
    /// A command-line flag value.
    Flag,
    /// A built-in default.
    Default,
    /// A consumer-defined kind.
    Custom(String),
}

impl fmt::Display for SourceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::File => "file",
            Self::Env => "env",
            Self::Flag => "flag",
            Self::Default => "default",
            Self::Custom(kind) => kind,
        })
    }
}

/// Origin of a layer or of one setting: a kind and a name.
///
/// For a file the name is its path; for an environment setting it is the
/// variable name.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct Source {
    /// What sort of origin this is.
    pub kind: SourceKind,
    /// Path, variable name or other identifier.
    pub name: String,
}

impl Source {
    /// Builds a source from a kind and a name.
    pub fn new(kind: SourceKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
        }
    }

    /// Builds a source with a consumer-defined kind, such as `keychain`.
    pub fn custom(kind: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(SourceKind::Custom(kind.into()), name)
    }
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.kind, self.name)
    }
}

/// Path of keys from the document root to a setting.
///
/// `Display` joins the segments with `.` and writes a segment that contains
/// `.` as a JSON string.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KeyPath(pub(crate) Vec<String>);

impl KeyPath {
    /// Builds a path from owned segments, outermost key first.
    pub fn from_segments(segments: impl IntoIterator<Item = String>) -> Self {
        Self(segments.into_iter().collect())
    }
}

/// Splits on `.`. Use [`KeyPath::from_segments`] for a key that contains `.`.
impl From<&str> for KeyPath {
    fn from(path: &str) -> Self {
        Self(path.split('.').map(str::to_owned).collect())
    }
}

impl fmt::Display for KeyPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, segment) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str(".")?;
            }
            if segment.contains('.') {
                write!(formatter, "{}", Value::String(segment.clone()))?;
            } else {
                formatter.write_str(segment)?;
            }
        }
        Ok(())
    }
}

impl Serialize for KeyPath {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// One leaf of the resolved value with the source that set it.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct ResolvedRow {
    /// Path of the leaf.
    pub path: KeyPath,
    /// Source of the highest layer that set it.
    pub source: Source,
    /// The leaf's JSON value.
    pub value: Value,
}

/// Source of every leaf of a resolved value.
///
/// A leaf is a scalar, an array, `null` or an empty object. A leaf replaced by
/// a higher layer is no longer listed.
#[derive(Clone, Debug, Default)]
pub struct Provenance {
    leaves: BTreeMap<KeyPath, (Source, Value)>,
}

impl Provenance {
    /// Returns the source of the leaf at `path`, if there is one.
    pub fn source_of(&self, path: &KeyPath) -> Option<&Source> {
        self.leaves.get(path).map(|(source, _)| source)
    }

    /// Returns every leaf, sorted by path.
    pub fn rows(&self) -> Vec<ResolvedRow> {
        self.leaves
            .iter()
            .map(|(path, (source, value))| ResolvedRow {
                path: path.clone(),
                source: source.clone(),
                value: value.clone(),
            })
            .collect()
    }

    /// Forgets every leaf at or below `path`.
    pub(crate) fn remove_under(&mut self, path: &KeyPath) {
        self.leaves.retain(|key, _| !key.0.starts_with(&path.0));
    }

    /// Records the leaves `layer` sets. `base` is the value below the layer.
    pub(crate) fn record(&mut self, layer: &Layer, base: &Value) {
        for (segments, value) in leaves(&layer.value) {
            // Merging an empty object into an object changes nothing.
            if matches!(value, Value::Object(_)) && is_object_at(base, &segments) {
                continue;
            }
            let path = KeyPath(segments);
            let source = layer.source_for(&path).clone();
            self.leaves
                .retain(|key, _| !(key.0.starts_with(&path.0) || path.0.starts_with(&key.0)));
            self.leaves.insert(path, (source, value.clone()));
        }
    }
}

fn is_object_at(root: &Value, path: &[String]) -> bool {
    let mut node = root;
    for key in path {
        match node.get(key.as_str()) {
            Some(child) => node = child,
            None => return false,
        }
    }
    node.is_object()
}

/// Lists the leaves of `root` with their paths. Walks without recursion.
pub(crate) fn leaves(root: &Value) -> Vec<(Vec<String>, &Value)> {
    let mut found = Vec::new();
    let mut pending = vec![(Vec::new(), root)];
    while let Some((path, value)) = pending.pop() {
        match value {
            Value::Object(map) if !map.is_empty() => {
                for (key, child) in map {
                    let mut child_path = path.clone();
                    child_path.push(key.clone());
                    pending.push((child_path, child));
                }
            }
            Value::Object(_) if path.is_empty() => {}
            _ => found.push((path, value)),
        }
    }
    found
}
