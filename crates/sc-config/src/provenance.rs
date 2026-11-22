//! Where each resolved setting came from: sources, key paths and per-leaf provenance.

use std::collections::BTreeMap;
use std::fmt;

use serde::Serialize;
use serde_json::Value;

use crate::layers::Layer;

/// Kind of a [`Source`].
///
/// A consumer's own kind uses [`SourceKind::Custom`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
pub struct Source {
    /// What sort of origin this is.
    pub kind: SourceKind,
    /// Path, variable name or other identifier.
    pub name: String,
}

impl Source {
    fn new(kind: SourceKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
        }
    }

    /// A file, named by its path.
    pub fn file(path: impl AsRef<std::path::Path>) -> Self {
        Self::new(SourceKind::File, path.as_ref().to_string_lossy())
    }

    /// An environment variable.
    pub fn env(variable: impl Into<String>) -> Self {
        Self::new(SourceKind::Env, variable)
    }

    /// A command-line flag.
    pub fn flag(name: impl Into<String>) -> Self {
        Self::new(SourceKind::Flag, name)
    }

    /// A built-in default.
    pub fn default(name: impl Into<String>) -> Self {
        Self::new(SourceKind::Default, name)
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

/// One leaf of the resolved value with the source that set it.
#[derive(Clone, Debug, Serialize)]
pub struct ResolvedRow {
    /// Key segments from the document root to the leaf, outermost first.
    pub path: Vec<String>,
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
pub(crate) struct Provenance {
    leaves: BTreeMap<Vec<String>, (Source, Value)>,
}

impl Provenance {
    /// Returns every leaf, sorted by path.
    pub(crate) fn rows(&self) -> Vec<ResolvedRow> {
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
    pub(crate) fn remove_under(&mut self, path: &[String]) {
        self.leaves.retain(|key, _| !key.starts_with(path));
    }

    /// Records the leaves `layer` sets. `base` is the value below the layer.
    pub(crate) fn record(&mut self, layer: &Layer, base: &Value) {
        for (segments, value) in leaves(&layer.value) {
            // Merging an empty object into an object changes nothing.
            if matches!(value, Value::Object(_)) && is_object_at(base, &segments) {
                continue;
            }
            let source = layer.source_for(&segments).clone();
            self.leaves
                .retain(|key, _| !(key.starts_with(&segments) || segments.starts_with(key)));
            self.leaves.insert(segments, (source, value.clone()));
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
