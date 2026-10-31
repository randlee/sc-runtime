//! Application-owned Monty host registrations and opt-in CLI adapters.
//!
//! Handlers run synchronously on the blocking feed thread. They are trusted
//! application code and cannot be forcibly preempted by a row deadline.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use monty_types::{MontyObject, ObjectRef};
use sc_observability_types::SharedCanonicalLog;
use serde_json::{Map, Number, Value};

use crate::monty_compose_common::ComposeHostError;
use crate::{CronError, HookEvent, ScheduleName};

/// A validated, non-reserved Monty host-function identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MontyHostName(String);

impl MontyHostName {
    /// Validates an application host-function name.
    ///
    /// # Errors
    /// Invalid identifiers and reserved hook names return InvalidSpec.
    pub fn new(name: impl Into<String>) -> Result<Self, CronError> {
        let name = name.into();
        if !crate::is_identifier(&name)
            || matches!(name.as_str(), "__hook" | "cron_stop" | "cron_tick")
        {
            return Err(CronError::InvalidSpec(format!(
                "invalid or reserved Monty host name: {name}"
            )));
        }
        Ok(Self(name))
    }

    /// Borrows the validated identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for MontyHostName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for MontyHostName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An owned JSON-shaped invocation of an application host function.
#[derive(Clone)]
pub struct MontyHostCall {
    /// Registered function name.
    pub name: MontyHostName,
    /// Positional JSON arguments in source order.
    pub args: Vec<Value>,
    /// Keyword JSON arguments.
    pub kwargs: Map<String, Value>,
    /// Schedule which invoked this handler.
    pub schedule: ScheduleName,
    /// Stop check or tick.
    pub phase: HookEvent,
    /// Monotonic deadline shared by the whole row execution.
    pub deadline: Instant,
}

impl fmt::Debug for MontyHostCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MontyHostCall")
            .field("name", &self.name)
            .field("argument_count", &self.args.len())
            .field("keyword_names", &self.kwargs.keys())
            .field("schedule", &self.schedule)
            .field("phase", &self.phase)
            .field("deadline", &self.deadline)
            .finish()
    }
}

/// A trusted synchronous application extension point, intentionally open.
///
/// Callbacks execute on the existing blocking feed thread. They must return
/// cooperatively; sc-cron checks the row deadline before and after the call.
/// Panics, allocator aborts, and stack exhaustion retain Rust Job semantics.
pub trait MontyHostHandler: Send + Sync {
    /// Handles one invocation and returns a JSON-shaped result.
    ///
    /// # Errors
    /// Application errors are recorded as `HOST_CALL_FAILED`, even InvalidSpec.
    fn call(&self, call: &MontyHostCall) -> Result<Value, CronError>;
}

impl<F> MontyHostHandler for F
where
    F: Fn(&MontyHostCall) -> Result<Value, CronError> + Send + Sync,
{
    fn call(&self, call: &MontyHostCall) -> Result<Value, CronError> {
        self(call)
    }
}

#[derive(Clone)]
enum HostEntry {
    Custom(Arc<dyn MontyHostHandler>),
    Cli(CliConfig),
}

/// Immutable application host registrations; a new registry is empty.
#[derive(Clone, Default)]
pub struct MontyHostRegistry {
    entries: BTreeMap<MontyHostName, HostEntry>,
}

impl fmt::Debug for MontyHostRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MontyHostRegistry")
            .field("names", &self.entries.keys())
            .finish()
    }
}

impl MontyHostRegistry {
    /// Creates a registry with no application functions or CLI adapters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an application handler, consuming this registry builder.
    ///
    /// # Errors
    /// Invalid identifiers, reserved hook names and duplicates are InvalidSpec.
    pub fn register(
        mut self,
        name: impl Into<String>,
        handler: impl MontyHostHandler + 'static,
    ) -> Result<Self, CronError> {
        self.insert(name.into(), HostEntry::Custom(Arc::new(handler)))?;
        Ok(self)
    }

    /// Registers the optional `sc_compose_run` adapter.
    ///
    /// # Errors
    /// Returns InvalidSpec when that name is already registered.
    pub fn with_sc_compose(mut self, config: ScComposeHostConfig) -> Result<Self, CronError> {
        self.insert("sc_compose_run".into(), HostEntry::Cli(config.0))?;
        Ok(self)
    }

    /// Registers the optional `atm_run` adapter.
    ///
    /// # Errors
    /// Returns InvalidSpec when that name is already registered.
    pub fn with_atm(mut self, config: AtmHostConfig) -> Result<Self, CronError> {
        self.insert("atm_run".into(), HostEntry::Cli(config.0))?;
        Ok(self)
    }

    /// Returns configured names in stable identifier order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(MontyHostName::as_str)
    }

    fn insert(&mut self, name: String, entry: HostEntry) -> Result<(), CronError> {
        let name = MontyHostName::new(name)?;
        if self.entries.contains_key(&name) {
            return Err(CronError::InvalidSpec(format!(
                "duplicate Monty host name: {name}"
            )));
        }
        self.entries.insert(name, entry);
        Ok(())
    }

    pub(crate) fn invoke(
        &self,
        call: &MontyHostCall,
        logger: &SharedCanonicalLog,
        environment: &crate::environment::EnvironmentSnapshot,
        correlation: &crate::worker::RunCorrelation,
    ) -> Result<Value, HostDispatchError> {
        match self.entries.get(&call.name) {
            None => Err(HostDispatchError::Undefined),
            Some(HostEntry::Custom(handler)) => {
                handler.call(call).map_err(HostDispatchError::Handler)
            }
            Some(HostEntry::Cli(config)) => {
                let args = cli_arguments(call)?;
                let result = crate::monty_compose::run_cli(
                    config.program.as_path(),
                    config.cwd.as_deref(),
                    &args,
                    call.deadline,
                    logger,
                    environment,
                    correlation,
                )
                .map_err(HostDispatchError::Child)?;
                Ok(
                    serde_json::json!({"stdout": result.stdout, "stderr": result.stderr, "exit_code": result.exit_code}),
                )
            }
        }
    }
}

pub(crate) enum HostDispatchError {
    Undefined,
    Denied(&'static str),
    Handler(CronError),
    Child(ComposeHostError),
}

#[derive(Clone, PartialEq, Eq)]
struct CliConfig {
    program: PathBuf,
    cwd: Option<PathBuf>,
}

impl fmt::Debug for CliConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CliConfig")
            .field("program_configured", &!self.program.as_os_str().is_empty())
            .field("working_directory_configured", &self.cwd.is_some())
            .finish()
    }
}

impl CliConfig {
    fn new(program: PathBuf) -> Result<Self, CronError> {
        if program.as_os_str().is_empty() {
            return Err(CronError::InvalidSpec("host program path is empty".into()));
        }
        let cwd = std::env::current_dir().map_err(|error| {
            CronError::InvalidSpec(format!("cannot capture host working directory: {error}"))
        })?;
        let program = if program.is_relative() && program.components().count() > 1 {
            cwd.join(program)
        } else {
            program
        };
        let program = crate::process::resolve_program(&program).unwrap_or(program);
        Ok(Self {
            program,
            cwd: Some(cwd),
        })
    }
    fn default_program(program: &str) -> Self {
        Self {
            program: crate::process::resolve_program(Path::new(program))
                .unwrap_or_else(|_| PathBuf::from(program)),
            cwd: std::env::current_dir().ok(),
        }
    }
    fn working_directory(mut self, cwd: &Path) -> Result<Self, CronError> {
        let base = self.cwd.as_ref().ok_or_else(|| {
            CronError::InvalidSpec("host working directory was unavailable at configuration".into())
        })?;
        self.cwd = Some(if cwd.is_absolute() {
            cwd.to_path_buf()
        } else {
            base.join(cwd)
        });
        Ok(self)
    }
}

/// Embedder-selected executable and working directory for `sc_compose_run`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScComposeHostConfig(CliConfig);
impl Default for ScComposeHostConfig {
    fn default() -> Self {
        Self(CliConfig::default_program("sc-compose"))
    }
}
impl ScComposeHostConfig {
    /// Captures the daemon directory and selects a non-empty executable path.
    ///
    /// # Errors
    /// Empty program or unavailable current directory returns InvalidSpec.
    pub fn new(program: impl Into<PathBuf>) -> Result<Self, CronError> {
        CliConfig::new(program.into()).map(Self)
    }
    /// Selects a working directory relative to the captured daemon directory.
    ///
    /// # Errors
    /// An unavailable captured daemon directory returns InvalidSpec.
    pub fn with_working_directory(self, cwd: impl AsRef<Path>) -> Result<Self, CronError> {
        self.0.working_directory(cwd.as_ref()).map(Self)
    }
}

/// Embedder-selected executable and working directory for `atm_run`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtmHostConfig(CliConfig);
impl Default for AtmHostConfig {
    fn default() -> Self {
        Self(CliConfig::default_program("atm"))
    }
}
impl AtmHostConfig {
    /// Captures the daemon directory and selects a non-empty executable path.
    ///
    /// # Errors
    /// Empty program or unavailable current directory returns InvalidSpec.
    pub fn new(program: impl Into<PathBuf>) -> Result<Self, CronError> {
        CliConfig::new(program.into()).map(Self)
    }
    /// Selects a working directory relative to the captured daemon directory.
    ///
    /// # Errors
    /// An unavailable captured daemon directory returns InvalidSpec.
    pub fn with_working_directory(self, cwd: impl AsRef<Path>) -> Result<Self, CronError> {
        self.0.working_directory(cwd.as_ref()).map(Self)
    }
}

fn cli_arguments(call: &MontyHostCall) -> Result<Vec<String>, HostDispatchError> {
    let denied = || {
        HostDispatchError::Denied(
            "CLI host expects a non-empty command and optional list of strings, without keywords",
        )
    };
    if !call.kwargs.is_empty() || !(1..=2).contains(&call.args.len()) {
        return Err(denied());
    }
    let command = call.args[0]
        .as_str()
        .filter(|command| !command.is_empty())
        .ok_or_else(denied)?;
    let mut argv = vec![command.to_owned()];
    if let Some(tail) = call.args.get(1) {
        let tail = tail.as_array().ok_or_else(denied)?;
        for arg in tail {
            argv.push(arg.as_str().ok_or_else(denied)?.to_owned());
        }
    }
    Ok(argv)
}

/// A Monty value outside the supported JSON shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MontyJsonError;
impl fmt::Display for MontyJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("monty value is not JSON")
    }
}

/// Converts a JSON value into the Monty object passed as `__hook` or a context.
#[must_use]
pub(crate) fn json_to_monty(value: &Value) -> MontyObject {
    match value {
        Value::Null => MontyObject::none(),
        Value::Bool(item) => MontyObject::bool(*item),
        Value::Number(number) => number.as_i64().map_or_else(
            || MontyObject::float(number.as_f64().unwrap_or(0.0)),
            MontyObject::int,
        ),
        Value::String(text) => MontyObject::string(text.clone()),
        Value::Array(items) => MontyObject::list(items.iter().map(json_to_monty)),
        Value::Object(map) => MontyObject::dict(
            map.iter()
                .map(|(key, item)| (MontyObject::string(key.clone()), json_to_monty(item))),
        ),
    }
}

/// Converts a Monty value that mirrors [`json_to_monty`].
///
/// Null, bool, int, float, string, list, and dict (string keys) are accepted.
/// Any other Monty type, a non-string dict key, an int that does not fit in
/// `i64`, or a non-finite float is [`MontyJsonError`].
///
/// # Errors
///
/// [`MontyJsonError`] when `value` is not one of those JSON shapes.
pub(crate) fn monty_value_to_json(value: ObjectRef<'_>) -> Result<Value, MontyJsonError> {
    match value.type_name() {
        "NoneType" => Ok(Value::Null),
        "bool" => Ok(Value::Bool(value.as_bool().ok_or(MontyJsonError)?)),
        "int" => Ok(Value::Number(value.as_int().ok_or(MontyJsonError)?.into())),
        "float" => {
            let number = value.as_float().ok_or(MontyJsonError)?;
            Ok(Value::Number(
                Number::from_f64(number).ok_or(MontyJsonError)?,
            ))
        }
        "str" => Ok(Value::String(
            value.as_str().ok_or(MontyJsonError)?.to_owned(),
        )),
        "list" => {
            let items = value.items().ok_or(MontyJsonError)?;
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(monty_value_to_json(item)?);
            }
            Ok(Value::Array(out))
        }
        "dict" => {
            let pairs = value.pairs().ok_or(MontyJsonError)?;
            let mut map = Map::new();
            for (key, item) in pairs {
                let key = key.as_str().ok_or(MontyJsonError)?.to_owned();
                map.insert(key, monty_value_to_json(item)?);
            }
            Ok(Value::Object(map))
        }
        _ => Err(MontyJsonError),
    }
}

#[cfg(feature = "monty-test-handshake")]
pub(crate) fn host_budget_exhausted(deadline: Option<Instant>) -> bool {
    deadline.is_some_and(|deadline| Instant::now() >= deadline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_names_validate_once_and_keep_string_lookup_and_json_arguments() {
        for name in ["", "bad-name", "2host", "__hook", "cron_stop", "cron_tick"] {
            assert!(MontyHostName::new(name).is_err(), "{name}");
        }
        let name = MontyHostName::new("my_host_2").unwrap();
        assert_eq!(name.as_str(), "my_host_2");
        let registry = MontyHostRegistry::new()
            .register(name.as_str(), |call: &MontyHostCall| {
                Ok(call.args[0].clone())
            })
            .unwrap();
        assert!(registry.entries.contains_key(name.as_str()));
        let call = MontyHostCall {
            name,
            args: vec![serde_json::json!({"nested":[1,true,null]})],
            kwargs: Map::new(),
            schedule: ScheduleName::new("host").unwrap(),
            phase: HookEvent::Tick,
            deadline: Instant::now(),
        };
        let value = registry
            .invoke(
                &call,
                &crate::noop_logger(),
                &crate::environment::EnvironmentSnapshot::default(),
                &None,
            )
            .unwrap_or_else(|_| panic!("registered host"));
        assert_eq!(value, call.args[0]);
    }
}
