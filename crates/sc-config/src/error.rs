use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use crate::provenance::Source;
use crate::standard::Slot;

/// Failure from application-name construction or configuration loading.
///
/// Each variant carries the path or variable that caused it. [`Self::code`]
/// is the stable code for that failure. [`Self::recovery`] is the next step
/// for the caller.
#[derive(Debug)]
#[non_exhaustive]
pub enum ConfigError {
    /// `default.json` is not in the config directory.
    MissingDefault {
        /// Full path that was looked up.
        path: PathBuf,
    },
    /// `default.json` or `local.json` exists but cannot be read.
    Read {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O failure.
        source: std::io::Error,
    },
    /// A configuration file is not valid JSON, including content that is not UTF-8.
    Parse {
        /// Path of the file that failed to parse.
        path: PathBuf,
        /// 1-based line reported by the JSON parser.
        line: usize,
        /// 1-based column reported by the JSON parser.
        column: usize,
        /// Parser message.
        message: String,
    },
    /// An environment override key path is empty or walks through a non-object.
    Override {
        /// Full environment variable name.
        variable: String,
    },
    /// The merged document does not deserialize into the caller's type.
    Deserialize {
        /// Message from the JSON deserializer.
        message: String,
        /// Files that were read, in read order.
        files: Vec<PathBuf>,
        /// Environment variables that were applied, in application order.
        variables: Vec<String>,
    },
    /// The application name failed the construction predicate.
    InvalidName {
        /// The rejected name.
        name: String,
    },
    /// A prefix-matching environment entry is not valid Unicode.
    NonUnicode {
        /// Variable name, which may itself be non-Unicode.
        variable: OsString,
    },
    /// A required file is absent.
    MissingFile {
        /// Full path that was looked up.
        path: PathBuf,
    },
    /// A file holds a key outside the paths its source accepts.
    UnknownKey {
        /// Path of the file holding the key.
        path: PathBuf,
        /// The offending key path.
        key: String,
    },
    /// A consumer-defined source failed.
    SourceFailed {
        /// The source that failed.
        origin: Source,
        /// Message from that source.
        message: String,
    },
    /// A standard slot was filled twice.
    DuplicateSlot {
        /// The slot that was filled twice.
        slot: Slot,
    },
}

impl ConfigError {
    fn layered_recovery(&self) -> &'static str {
        match self {
            Self::MissingFile { .. } => "Create the file, or make the source optional.",
            Self::UnknownKey { .. } => {
                "Remove the key from that file, or set it where it is accepted."
            }
            Self::SourceFailed { .. } => "Follow the message from that source.",
            Self::DuplicateSlot { .. } => "Fill each slot once; add extra sources with insert.",
            _ => "",
        }
    }

    /// Builds the error a consumer's `LayerSource` returns when it fails.
    pub fn source_failed(origin: Source, message: impl Into<String>) -> Self {
        Self::SourceFailed {
            origin,
            message: message.into(),
        }
    }

    /// Returns the stable code for this failure.
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingDefault { .. } => "CFG.MISSING_DEFAULT",
            Self::Read { .. } => "CFG.READ",
            Self::Parse { .. } => "CFG.PARSE",
            Self::Override { .. } => "CFG.OVERRIDE",
            Self::Deserialize { .. } => "CFG.DESERIALIZE",
            Self::InvalidName { .. } => "APP.INVALID_NAME",
            Self::NonUnicode { .. } => "CFG.NON_UNICODE",
            Self::MissingFile { .. } => "CFG.MISSING_FILE",
            Self::UnknownKey { .. } => "CFG.UNKNOWN_KEY",
            Self::SourceFailed { .. } => "CFG.SOURCE",
            Self::DuplicateSlot { .. } => "CFG.DUPLICATE_SLOT",
        }
    }

    /// Returns the recovery step for this failure.
    pub fn recovery(&self) -> &'static str {
        match self {
            Self::MissingDefault { .. } => {
                "Create default.json in the config directory and load again."
            }
            Self::Read { .. } => {
                "Make the file readable, or replace a directory at that path with a file."
            }
            Self::Parse { .. } => {
                "Fix the JSON at the reported line and column. The file must be UTF-8."
            }
            Self::Override { .. } => {
                "Remove the empty path segment, or change the variable so its path does not walk through a non-object value."
            }
            Self::Deserialize { .. } => {
                "Align the merged JSON with the requested type. The error lists the files and variables that were applied."
            }
            Self::InvalidName { .. } => {
                "Use 1 to 64 bytes of ASCII letters, digits, '-', and '_' ."
            }
            Self::NonUnicode { .. } => {
                "Give the variable a Unicode name and value. Names that do not start with the application prefix are ignored."
            }
            Self::MissingFile { .. }
            | Self::UnknownKey { .. }
            | Self::SourceFailed { .. }
            | Self::DuplicateSlot { .. } => self.layered_recovery(),
        }
    }
}

impl ConfigError {
    fn fmt_layered(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFile { path } => write!(
                formatter,
                "{}: required file not found at {}; {}",
                self.code(),
                path.display(),
                self.recovery()
            ),
            Self::UnknownKey { path, key } => write!(
                formatter,
                "{}: key {key} in {} is not accepted; {}",
                self.code(),
                path.display(),
                self.recovery()
            ),
            Self::SourceFailed { origin, message } => write!(
                formatter,
                "{}: source {origin} failed: {message}; {}",
                self.code(),
                self.recovery()
            ),
            Self::DuplicateSlot { slot } => write!(
                formatter,
                "{}: slot {slot} was filled twice; {}",
                self.code(),
                self.recovery()
            ),
            _ => Ok(()),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDefault { path } => write!(
                formatter,
                "{}: default configuration file not found at {}; {}",
                self.code(),
                path.display(),
                self.recovery()
            ),
            Self::Read { path, source } => write!(
                formatter,
                "{}: cannot read {}: {source}; {}",
                self.code(),
                path.display(),
                self.recovery()
            ),
            Self::Parse {
                path,
                line,
                column,
                message,
            } => write!(
                formatter,
                "{}: invalid JSON in {} at line {line} column {column}: {message}; {}",
                self.code(),
                path.display(),
                self.recovery()
            ),
            Self::Override { variable } => write!(
                formatter,
                "{}: cannot apply environment variable {variable}; {}",
                self.code(),
                self.recovery()
            ),
            Self::Deserialize {
                message,
                files,
                variables,
            } => write!(
                formatter,
                "{}: configuration does not match the requested type: {message}; files: {}; variables: {}; {}",
                self.code(),
                join_list(files.iter().map(|path| path.display())),
                join_list(variables.iter()),
                self.recovery()
            ),
            Self::InvalidName { name } => write!(
                formatter,
                "{}: application name `{name}` is invalid; {}",
                self.code(),
                self.recovery()
            ),
            Self::NonUnicode { variable } => write!(
                formatter,
                "{}: environment variable {} is not valid Unicode; {}",
                self.code(),
                variable.to_string_lossy(),
                self.recovery()
            ),
            Self::MissingFile { .. }
            | Self::UnknownKey { .. }
            | Self::SourceFailed { .. }
            | Self::DuplicateSlot { .. } => self.fmt_layered(formatter),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn join_list(items: impl IntoIterator<Item = impl fmt::Display>) -> String {
    let mut rendered = String::new();
    for (index, item) in items.into_iter().enumerate() {
        if index > 0 {
            rendered.push_str(", ");
        }
        rendered.push_str(&item.to_string());
    }
    rendered
}
