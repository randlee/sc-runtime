//! The standard layer order products share, enforced by [`StandardLayers`].
//!
//! The slots, lowest first: defaults, system, user, project, local,
//! environment, flags. A family names the folder and each product names its
//! files:
//!
//! - system: `<system_dir>/<product>.json`
//! - user: `<home>/.<family>/<product>.json` (see [`StandardContext::with_home`])
//! - project: `<project_root>/.<family>/<product>.json`
//! - local: `<project_root>/.<family>/<product>.local.json`
//!
//! In the system and user folders `<product>.local.json` is also read, after
//! the `.json` files of the same folder, so a local file wins.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::app_name::AppName;
use crate::error::ConfigError;
use crate::layers::{LayerSource, Layers, Resolved};
use crate::provenance::{KeyPath, Source};
use crate::sources::{EnvSource, FileSource, ValueSource};

/// One position in the standard order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Slot {
    /// Built-in defaults.
    Defaults,
    /// Machine-wide files.
    System,
    /// Files in the user's home folder.
    User,
    /// Files in the project folder.
    Project,
    /// Local files in the project folder.
    Local,
    /// Environment variables.
    Env,
    /// Parsed command-line flags.
    Flags,
}

/// The slot a source is inserted directly above, as in `After::User`.
///
/// Same variants as [`Slot`] except `Flags`, which has nothing above it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum After {
    /// Above [`Slot::Defaults`].
    Defaults,
    /// Above [`Slot::System`].
    System,
    /// Above [`Slot::User`].
    User,
    /// Above [`Slot::Project`].
    Project,
    /// Above [`Slot::Local`].
    Local,
    /// Above [`Slot::Env`].
    Env,
}

impl After {
    fn slot(self) -> Slot {
        match self {
            Self::Defaults => Slot::Defaults,
            Self::System => Slot::System,
            Self::User => Slot::User,
            Self::Project => Slot::Project,
            Self::Local => Slot::Local,
            Self::Env => Slot::Env,
        }
    }
}

const SLOTS: [Slot; 7] = [
    Slot::Defaults,
    Slot::System,
    Slot::User,
    Slot::Project,
    Slot::Local,
    Slot::Env,
    Slot::Flags,
];

impl fmt::Display for Slot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Defaults => "defaults",
            Self::System => "system",
            Self::User => "user",
            Self::Project => "project",
            Self::Local => "local",
            Self::Env => "env",
            Self::Flags => "flags",
        })
    }
}

/// Names and folders for a [`StandardLayers`] stack.
#[derive(Clone, Debug)]
pub struct StandardContext {
    family: AppName,
    products: Vec<AppName>,
    environment: Vec<(OsString, OsString)>,
    home: Option<PathBuf>,
    home_var: OsString,
    system_dir: Option<PathBuf>,
    project_root: Option<PathBuf>,
}

impl StandardContext {
    /// Names the `family` folder and the `products` whose files are read.
    ///
    /// `environment` supplies the `EnvSource` variables and the variables
    /// that locate the user folder; see [`with_home`](Self::with_home).
    pub fn new(
        family: AppName,
        products: Vec<AppName>,
        environment: Vec<(OsString, OsString)>,
    ) -> Self {
        let prefix = family.env_prefix();
        let home_var = OsString::from(format!("{}_HOME", prefix.trim_end_matches("__")));
        Self {
            family,
            products,
            environment,
            home: None,
            home_var,
            system_dir: None,
            project_root: None,
        }
    }

    /// Sets the user folder itself, ahead of any variable.
    ///
    /// Without it the user folder is, first match wins: the value of the home
    /// variable (`SC_AGENT_HOME` for family `sc-agent`; see
    /// [`with_home_var`](Self::with_home_var)) from the context's environment,
    /// used as the folder itself; else `<home>/.<family>/`, where `<home>` is
    /// `USERPROFILE` on Windows and `HOME` elsewhere.
    #[must_use]
    pub fn with_home(mut self, dir: impl Into<PathBuf>) -> Self {
        self.home = Some(dir.into());
        self
    }

    /// Names the variable that relocates the user folder.
    ///
    /// The default is the family name upper-cased with `-` mapped to `_`, plus
    /// `_HOME`.
    #[must_use]
    pub fn with_home_var(mut self, name: impl Into<OsString>) -> Self {
        self.home_var = name.into();
        self
    }

    /// Sets the system folder, which holds the product files directly.
    ///
    /// `sc-config` has no built-in system location. The recommendation is
    /// `/etc/<family>/` on Unix and `%ProgramData%\<family>\` on Windows. The
    /// slot reads nothing when no folder is set.
    #[must_use]
    pub fn with_system_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.system_dir = Some(dir.into());
        self
    }

    /// Sets the project root that holds the `.<family>` folder.
    #[must_use]
    pub fn with_project_root(mut self, dir: impl Into<PathBuf>) -> Self {
        self.project_root = Some(dir.into());
        self
    }

    fn variable(&self, name: &str) -> Option<PathBuf> {
        self.environment
            .iter()
            .find(|(key, _)| key.as_os_str() == name)
            .map(|(_, value)| PathBuf::from(value))
    }

    fn user_dir(&self) -> Option<PathBuf> {
        if let Some(dir) = &self.home {
            return Some(dir.clone());
        }
        let relocated = self
            .environment
            .iter()
            .find(|(key, _)| *key == self.home_var)
            .map(|(_, value)| PathBuf::from(value));
        relocated.or_else(|| {
            let home = self.variable(if cfg!(windows) { "USERPROFILE" } else { "HOME" })?;
            Some(home.join(format!(".{}", self.family)))
        })
    }

    fn project_dir(&self) -> Option<PathBuf> {
        let root = self.project_root.clone()?;
        Some(root.join(format!(".{}", self.family)))
    }
}

/// The layers a product uses, in the one standard order.
///
/// Fill the slots you want; call order does not matter, slots load in slot
/// order. Filling a slot twice is [`ConfigError::DuplicateSlot`], returned by
/// [`resolve`](Self::resolve) and [`load`](Self::load). A slot whose folder
/// is not set reads nothing.
///
/// # Examples
///
/// ```
/// use sc_config::{AppName, StandardContext, StandardLayers};
///
/// let family = AppName::new("my-suite")?;
/// let context = StandardContext::new(family.clone(), vec![family], Vec::new());
/// let value: serde_json::Value = StandardLayers::new(context)
///     .defaults(serde_json::json!({"port": 7400}))
///     .user()
///     .env(["MY_SUITE__"])
///     .load()?;
/// assert_eq!(value["port"], 7400);
/// # Ok::<(), sc_config::ConfigError>(())
/// ```
pub struct StandardLayers {
    context: StandardContext,
    filled: BTreeSet<Slot>,
    duplicate: Option<Slot>,
    defaults: Option<Value>,
    flags: Option<Value>,
    env_prefixes: Vec<String>,
    aliases: Vec<(String, KeyPath)>,
    accepts: BTreeMap<AppName, Vec<KeyPath>>,
    inserted: Vec<(Slot, Box<dyn LayerSource>)>,
}

impl fmt::Debug for StandardLayers {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StandardLayers")
            .field("filled", &self.filled)
            .finish_non_exhaustive()
    }
}

impl StandardLayers {
    /// Starts an empty stack for `context`.
    pub fn new(context: StandardContext) -> Self {
        Self {
            context,
            filled: BTreeSet::new(),
            duplicate: None,
            defaults: None,
            flags: None,
            env_prefixes: Vec::new(),
            aliases: Vec::new(),
            accepts: BTreeMap::new(),
            inserted: Vec::new(),
        }
    }

    fn fill(mut self, slot: Slot) -> Self {
        if !self.filled.insert(slot) && self.duplicate.is_none() {
            self.duplicate = Some(slot);
        }
        self
    }

    /// Fills the `Defaults` slot with `value`.
    #[must_use]
    pub fn defaults(mut self, value: Value) -> Self {
        self.defaults = Some(value);
        self.fill(Slot::Defaults)
    }

    /// Fills the `System` slot.
    #[must_use]
    pub fn system(self) -> Self {
        self.fill(Slot::System)
    }

    /// Fills the `User` slot.
    #[must_use]
    pub fn user(self) -> Self {
        self.fill(Slot::User)
    }

    /// Fills the `Project` slot.
    #[must_use]
    pub fn project(self) -> Self {
        self.fill(Slot::Project)
    }

    /// Fills the `Local` slot.
    #[must_use]
    pub fn local(self) -> Self {
        self.fill(Slot::Local)
    }

    /// Fills the `Env` slot with one literal env prefix per product.
    ///
    /// Prefixes apply in list order, so a later prefix wins.
    #[must_use]
    pub fn env<P: Into<String>>(mut self, prefixes: impl IntoIterator<Item = P>) -> Self {
        self.env_prefixes = prefixes.into_iter().map(Into::into).collect();
        self.fill(Slot::Env)
    }

    /// Fills the `Flags` slot with `value`.
    #[must_use]
    pub fn flags(mut self, value: Value) -> Self {
        self.flags = Some(value);
        self.fill(Slot::Flags)
    }

    /// Adds `source` directly above `after`. Slots cannot be reordered.
    ///
    /// Sources inserted after the same slot stack in insertion order.
    #[must_use]
    pub fn insert(mut self, after: After, source: impl LayerSource + 'static) -> Self {
        self.inserted.push((after.slot(), Box::new(source)));
        self
    }

    /// Limits `product`'s files, in every slot, to the key `paths`.
    ///
    /// See [`FileSource::accepts`].
    #[must_use]
    pub fn accepts<'a, P: AsRef<[&'a str]>>(
        mut self,
        product: &AppName,
        paths: impl IntoIterator<Item = P>,
    ) -> Self {
        self.accepts.insert(
            product.clone(),
            paths.into_iter().map(KeyPath::from_strs).collect(),
        );
        self
    }

    /// Maps the variable `variable` to the key path `path` in the `Env` slot.
    ///
    /// Aliases load in the `Env` slot, after its prefixed variables, even
    /// when [`env`](Self::env) is not called.
    #[must_use]
    pub fn alias<'a>(mut self, variable: impl Into<String>, path: impl AsRef<[&'a str]>) -> Self {
        self.aliases
            .push((variable.into(), KeyPath::from_strs(path)));
        self
    }

    /// Resolves the stack.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::DuplicateSlot`] if a slot was filled twice, or
    /// the first error of [`Layers::resolve`].
    pub fn resolve(self) -> Result<Resolved, ConfigError> {
        self.into_layers()?.resolve()
    }

    /// Resolves the stack and deserialises the result into `T`, once.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`StandardLayers::resolve`] and
    /// [`Resolved::deserialize`].
    pub fn load<T: DeserializeOwned>(self) -> Result<T, ConfigError> {
        self.resolve()?.deserialize()
    }

    fn into_layers(mut self) -> Result<Layers, ConfigError> {
        if let Some(slot) = self.duplicate {
            return Err(ConfigError::DuplicateSlot { slot });
        }
        let mut layers = Layers::new();
        for slot in SLOTS {
            for file in self.files(slot) {
                layers = layers.push(file);
            }
            if let Some(value) = self.value_for(slot) {
                layers = layers.push(value);
            }
            if slot == Slot::Env {
                for env in self.env_sources() {
                    layers = layers.push(env);
                }
            }
            let (here, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.inserted)
                .into_iter()
                .partition(|(after, _)| *after == slot);
            self.inserted = rest;
            for (_, source) in here {
                layers = layers.push_boxed(source);
            }
        }
        Ok(layers)
    }

    fn value_for(&self, slot: Slot) -> Option<ValueSource> {
        let (source, value) = match slot {
            Slot::Defaults => (Source::default("defaults"), self.defaults.clone()?),
            Slot::Flags => (Source::flag("flags"), self.flags.clone()?),
            _ => return None,
        };
        Some(ValueSource::new(source, value))
    }

    fn env_sources(&self) -> Vec<EnvSource> {
        let environment = &self.context.environment;
        let mut sources: Vec<EnvSource> = self
            .env_prefixes
            .iter()
            .map(|prefix| EnvSource::new(prefix.clone(), environment.clone()))
            .collect();
        if !self.aliases.is_empty() {
            let mut aliases = EnvSource::aliases_only(environment.clone());
            for (variable, path) in &self.aliases {
                aliases = aliases.alias_path(variable.clone(), path.clone());
            }
            sources.push(aliases);
        }
        sources
    }

    /// The file sources of a file slot: `.json` files, then `.local.json` files.
    fn files(&self, slot: Slot) -> Vec<FileSource> {
        if !self.filled.contains(&slot) {
            return Vec::new();
        }
        let context = &self.context;
        let (folder, plain, local) = match slot {
            Slot::System => (context.system_dir.clone(), true, true),
            Slot::User => (context.user_dir(), true, true),
            Slot::Project => (context.project_dir(), true, false),
            Slot::Local => (context.project_dir(), false, true),
            _ => return Vec::new(),
        };
        let Some(folder) = folder else {
            return Vec::new();
        };
        let suffixes = [(plain, ".json"), (local, ".local.json")];
        let mut files = Vec::new();
        for (wanted, suffix) in suffixes {
            if !wanted {
                continue;
            }
            for product in &context.products {
                files.push(self.product_file(&folder, product, suffix));
            }
        }
        files
    }

    fn product_file(&self, folder: &Path, product: &AppName, suffix: &str) -> FileSource {
        let file = FileSource::new(folder.join(format!("{product}{suffix}"))).optional();
        match self.accepts.get(product) {
            Some(paths) => file.with_accepted(Some(paths.clone())),
            None => file,
        }
    }
}
