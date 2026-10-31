//! Validated, persisted hook policy and daemon-owned resolution authority.
//!
//! Explicit set values are plaintext configuration. Process, file and resolved
//! reference values are read only for an occurrence and never written back.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use crate::{CronError, ScheduleName};

mod dotenv;
mod resolve;
mod wire;
pub(crate) use resolve::EnvironmentSnapshot;

/// Maximum variables in a policy or resolved snapshot.
pub const MAX_ENV_ENTRIES: usize = 128;
/// Maximum ordered dotenv sources in a schedule.
pub const MAX_ENV_DOTENV_SOURCES: usize = 16;
/// Maximum UTF-8 bytes in an environment name.
pub const MAX_ENV_NAME_BYTES: usize = 256;
/// Maximum UTF-8 bytes in one environment value.
pub const MAX_ENV_VALUE_BYTES: usize = 16 * 1024;
/// Maximum UTF-8 bytes in an opaque reference locator.
pub const MAX_ENV_REFERENCE_BYTES: usize = 4 * 1024;
/// Maximum UTF-8 bytes in a dotenv path.
pub const MAX_ENV_PATH_BYTES: usize = 4 * 1024;
/// Portable encoded environment block ceiling, including terminators.
pub const MAX_ENV_SNAPSHOT_BYTES: usize = 32_000;
/// Maximum bytes in canonical persisted policy JSON.
pub const MAX_ENV_DOCUMENT_BYTES: usize = 256 * 1024;
/// Maximum bytes read from one dotenv file.
pub const MAX_ENV_FILE_BYTES: usize = 1024 * 1024;

fn invalid() -> CronError {
    CronError::InvalidSpec("invalid or oversized hook environment configuration".into())
}

fn text_valid(text: &str, limit: usize) -> bool {
    text.len() <= limit && !text.contains('\0')
}

/// Validated portable ASCII environment identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnvName(String);
impl EnvName {
    /// Validates a name without including rejected input in errors.
    ///
    /// # Errors
    /// Invalid identifiers or names above [`MAX_ENV_NAME_BYTES`] are InvalidSpec.
    pub fn new(name: impl AsRef<str>) -> Result<Self, CronError> {
        let name = name.as_ref();
        if !crate::is_identifier(name) || name.len() > MAX_ENV_NAME_BYTES {
            return Err(invalid());
        }
        Ok(Self(name.to_owned()))
    }
    /// Returns the configured spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for EnvName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn normalized(path: PathBuf) -> Result<PathBuf, CronError> {
    if !path.is_absolute()
        || path
            .to_str()
            .is_none_or(|s| !text_valid(s, MAX_ENV_PATH_BYTES))
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(invalid());
    }
    Ok(path
        .components()
        .filter(|part| !matches!(part, Component::CurDir))
        .collect())
}

/// An absolute, normalized dotenv path with explicit absence handling.
#[derive(Clone, PartialEq, Eq)]
pub struct DotenvSource {
    path: PathBuf,
    required: bool,
}
impl DotenvSource {
    /// Validates an absolute UTF-8 path without parent components.
    ///
    /// # Errors
    /// Invalid paths are InvalidSpec. Existence is checked only at execution.
    pub fn new(path: impl Into<PathBuf>, required: bool) -> Result<Self, CronError> {
        Ok(Self {
            path: normalized(path.into())?,
            required,
        })
    }
    /// Returns the configured path, without resolving symlinks.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Whether absence fails the occurrence.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}
impl fmt::Debug for DotenvSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DotenvSource")
            .field("required", &self.required)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, PartialEq, Eq)]
enum Operation {
    Set(String),
    Reference(String),
    Clear,
}

/// A borrowed operation; Debug intentionally excludes values and locators.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvOperationRef<'a> {
    /// Deliberately persisted plaintext value.
    Set(&'a str),
    /// Application-owned reference metadata, resolved only at execution.
    Reference(&'a str),
    /// Remove the name even when an earlier layer supplied it.
    Clear,
}
impl fmt::Debug for EnvOperationRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Set(_) => "Set([REDACTED])",
            Self::Reference(_) => "Reference([REDACTED])",
            Self::Clear => "Clear",
        })
    }
}
impl Operation {
    fn as_ref(&self) -> EnvOperationRef<'_> {
        match self {
            Self::Set(s) => EnvOperationRef::Set(s),
            Self::Reference(s) => EnvOperationRef::Reference(s),
            Self::Clear => EnvOperationRef::Clear,
        }
    }
}

/// Ordered dotenv sources followed by one final operation per variable name.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct HookEnvironment {
    sources: Vec<DotenvSource>,
    variables: BTreeMap<EnvName, Operation>,
}
impl fmt::Debug for HookEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HookEnvironment")
            .field("dotenv_sources", &self.sources.len())
            .field(
                "variables",
                &self
                    .variables
                    .iter()
                    .map(|(name, op)| (name, op.as_ref()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
impl HookEnvironment {
    /// Creates an empty policy, capturing no ambient values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Appends an absolute dotenv source.
    ///
    /// # Errors
    /// Invalid/duplicate paths or policy limits are InvalidSpec.
    pub fn dotenv(mut self, path: impl Into<PathBuf>, required: bool) -> Result<Self, CronError> {
        let source = DotenvSource::new(path, required)?;
        if self.sources.len() >= MAX_ENV_DOTENV_SOURCES
            || self.sources.iter().any(|s| s.path == source.path)
        {
            return Err(invalid());
        }
        self.sources.push(source);
        self.validate()?;
        Ok(self)
    }
    /// Replaces the name's operation with an explicit plaintext value.
    ///
    /// # Errors
    /// Invalid names, NUL values, collisions or policy limits are InvalidSpec.
    pub fn set(self, name: impl AsRef<str>, value: impl Into<String>) -> Result<Self, CronError> {
        let value = value.into();
        if !text_valid(&value, MAX_ENV_VALUE_BYTES) {
            return Err(invalid());
        }
        self.insert(name.as_ref(), Operation::Set(value))
    }
    /// Replaces the name's operation with an opaque, nonempty reference.
    ///
    /// # Errors
    /// Invalid names/locators, collisions or policy limits are InvalidSpec.
    pub fn reference(
        self,
        name: impl AsRef<str>,
        reference: impl Into<String>,
    ) -> Result<Self, CronError> {
        let reference = reference.into();
        if reference.is_empty() || !text_valid(&reference, MAX_ENV_REFERENCE_BYTES) {
            return Err(invalid());
        }
        self.insert(name.as_ref(), Operation::Reference(reference))
    }
    /// Replaces the name's operation with removal.
    ///
    /// # Errors
    /// Invalid names, collisions or policy limits are InvalidSpec.
    pub fn clear(self, name: impl AsRef<str>) -> Result<Self, CronError> {
        self.insert(name.as_ref(), Operation::Clear)
    }
    fn insert(mut self, name: &str, operation: Operation) -> Result<Self, CronError> {
        let name = EnvName::new(name)?;
        if self
            .variables
            .keys()
            .any(|n| n != &name && n.0.eq_ignore_ascii_case(&name.0))
        {
            return Err(invalid());
        }
        self.variables.insert(name, operation);
        self.validate()?;
        Ok(self)
    }
    /// True only when both variables and sources are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty() && self.variables.is_empty()
    }
    /// Number of final variable operations, excluding sources.
    #[must_use]
    pub fn len(&self) -> usize {
        self.variables.len()
    }
    /// Sources in declaration order.
    pub fn dotenv_sources(&self) -> impl ExactSizeIterator<Item = &DotenvSource> {
        self.sources.iter()
    }
    /// Names sorted by UTF-8 bytes, preserving configured spelling.
    pub fn variable_names(&self) -> impl ExactSizeIterator<Item = &EnvName> {
        self.variables.keys()
    }
    /// Borrows the operation for `name`.
    #[must_use]
    pub fn operation(&self, name: &EnvName) -> Option<EnvOperationRef<'_>> {
        self.variables.get(name).map(Operation::as_ref)
    }
    pub(crate) fn validate(&self) -> Result<(), CronError> {
        if self.variables.len() > MAX_ENV_ENTRIES || self.sources.len() > MAX_ENV_DOTENV_SOURCES {
            return Err(invalid());
        }
        if self.encode()?.len() > MAX_ENV_DOCUMENT_BYTES {
            return Err(invalid());
        }
        Ok(())
    }
}

/// A resolver request; Debug includes the variable name and deadline only.
pub struct EnvironmentReferenceRequest<'a> {
    schedule: &'a ScheduleName,
    variable: &'a EnvName,
    reference: &'a str,
    deadline: Instant,
}
impl EnvironmentReferenceRequest<'_> {
    /// Schedule requesting authorization and resolution.
    #[must_use]
    pub fn schedule(&self) -> &ScheduleName {
        self.schedule
    }
    /// Variable to which the resolved value will be assigned.
    #[must_use]
    pub fn variable(&self) -> &EnvName {
        self.variable
    }
    /// Opaque application metadata. Never included in Debug or diagnostics.
    #[must_use]
    pub fn reference(&self) -> &str {
        self.reference
    }
    /// Shared row deadline.
    #[must_use]
    pub const fn deadline(&self) -> Instant {
        self.deadline
    }
}
impl fmt::Debug for EnvironmentReferenceRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnvironmentReferenceRequest")
            .field("variable", self.variable)
            .field("deadline", &self.deadline)
            .finish_non_exhaustive()
    }
}

/// Value-free application resolver failures. Provider prose has no channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvironmentReferenceError {
    /// Application policy denied this request.
    Denied,
    /// Provider could not resolve this request now.
    Unavailable,
}
impl fmt::Display for EnvironmentReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Denied => "environment reference denied",
            Self::Unavailable => "environment reference unavailable",
        })
    }
}
impl std::error::Error for EnvironmentReferenceError {}

/// Object-safe, application-owned authorization and reference resolution.
#[async_trait::async_trait]
pub trait EnvironmentReferenceResolver: Send + Sync {
    /// Resolves one reference before its row deadline.
    ///
    /// # Errors
    /// Denied/unavailable results are recorded as fixed, value-free ENV_REFERENCE.
    async fn resolve(
        &self,
        request: EnvironmentReferenceRequest<'_>,
    ) -> Result<Option<String>, EnvironmentReferenceError>;
}

/// Immutable daemon policy. Defaults inherit nothing and authorize no files.
#[derive(Clone, Default)]
pub struct HookEnvironmentHost {
    process: BTreeSet<EnvName>,
    files: BTreeSet<PathBuf>,
    roots: BTreeSet<PathBuf>,
    resolver: Option<Arc<dyn EnvironmentReferenceResolver>>,
}
impl fmt::Debug for HookEnvironmentHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HookEnvironmentHost")
            .field("process_names", &self.process)
            .field("dotenv_files", &self.files.len())
            .field("dotenv_roots", &self.roots.len())
            .field("has_reference_resolver", &self.resolver.is_some())
            .finish()
    }
}
impl HookEnvironmentHost {
    /// Creates an empty policy.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Accumulates allowed daemon variable names. Values are read at execution.
    ///
    /// # Errors
    /// Invalid names or ASCII case collisions are InvalidSpec.
    pub fn allow_process(
        mut self,
        names: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Self, CronError> {
        for name in names {
            let name = EnvName::new(name)?;
            if self
                .process
                .iter()
                .any(|n| n != &name && n.0.eq_ignore_ascii_case(&name.0))
            {
                return Err(invalid());
            }
            self.process.insert(name);
        }
        if self.process.len() > MAX_ENV_ENTRIES {
            return Err(invalid());
        }
        Ok(self)
    }
    /// Accumulates an exact absolute file authorization.
    ///
    /// # Errors
    /// Invalid paths are InvalidSpec; existence is checked at execution.
    pub fn allow_dotenv_file(mut self, path: impl Into<PathBuf>) -> Result<Self, CronError> {
        self.files.insert(normalized(path.into())?);
        Ok(self)
    }
    /// Accumulates an absolute directory-root authorization.
    ///
    /// # Errors
    /// Invalid paths are InvalidSpec; existence is checked at execution.
    pub fn allow_dotenv_root(mut self, path: impl Into<PathBuf>) -> Result<Self, CronError> {
        self.roots.insert(normalized(path.into())?);
        Ok(self)
    }
    /// Attaches the application's resolver without changing the allowlists.
    #[must_use]
    pub fn with_reference_resolver(
        mut self,
        resolver: Arc<dyn EnvironmentReferenceResolver>,
    ) -> Self {
        self.resolver = Some(resolver);
        self
    }
    /// Allowed names in configured spelling.
    pub fn process_names(&self) -> impl ExactSizeIterator<Item = &EnvName> {
        self.process.iter()
    }
    /// Configured exact-file authorizations.
    pub fn dotenv_files(&self) -> impl ExactSizeIterator<Item = &Path> {
        self.files.iter().map(PathBuf::as_path)
    }
    /// Configured directory-root authorizations.
    pub fn dotenv_roots(&self) -> impl ExactSizeIterator<Item = &Path> {
        self.roots.iter().map(PathBuf::as_path)
    }
    /// Whether an application resolver is attached.
    #[must_use]
    pub fn has_reference_resolver(&self) -> bool {
        self.resolver.is_some()
    }
}
