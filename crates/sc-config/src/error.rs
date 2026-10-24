use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

/// Failure from application-name construction or configuration loading.
///
/// Each variant carries the path or variable that caused it. [`Self::code`]
/// is the stable code for that failure. [`Self::recovery`] is the next step
/// for the caller.
#[derive(Debug)]
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
}

impl ConfigError {
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
        }
    }

    /// Returns the recovery step for this failure.
    pub fn recovery(&self) -> &'static str {
        match self {
            Self::MissingDefault { .. } => {
                "Create default.json in the config directory and load again."
            }
            Self::Read { .. } => "Make the file readable, or replace a directory at that path with a file.",
            Self::Parse { .. } => "Fix the JSON at the reported line and column. The file must be UTF-8.",
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
                join_display(files),
                join_strings(variables),
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

fn join_display(paths: &[PathBuf]) -> String {
    let mut rendered = String::new();
    for (index, path) in paths.iter().enumerate() {
        if index > 0 {
            rendered.push_str(", ");
        }
        rendered.push_str(&path.display().to_string());
    }
    rendered
}

fn join_strings(values: &[String]) -> String {
    let mut rendered = String::new();
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            rendered.push_str(", ");
        }
        rendered.push_str(value);
    }
    rendered
}
