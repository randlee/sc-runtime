//! The standard layer order products share, enforced by [`StandardLayers`].
//!
//! The slots, lowest first: defaults, system, user, project, local,
//! environment, flags. A family names the folder and each product names its
//! files:
//!
//! - system: `<system_dir>/<product>.json`
//! - user: `<home>/.<family>/<product>.json`
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
use crate::provenance::{KeyPath, Source, SourceKind};
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

/// Names the slot a source is inserted directly above, as in `After::User`.
pub type After = Slot;

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
    system_dir: Option<PathBuf>,
    project_root: Option<PathBuf>,
}

impl StandardContext {
    /// Names the `family` folder and the `products` whose files are read.
    ///
    /// `environment` supplies the `EnvSource` variables and, unless
    /// [`with_home`](Self::with_home) is called, the home folder: `HOME` on
    /// Unix, `USERPROFILE` on Windows.
    pub fn new(
        family: AppName,
        products: Vec<AppName>,
        environment: Vec<(OsString, OsString)>,
    ) -> Self {
        Self {
            family,
            products,
            environment,
            home: None,
            system_dir: None,
            project_root: None,
        }
    }

    /// Sets the home folder instead of reading it from the environment.
    #[must_use]
    pub fn with_home(mut self, dir: impl Into<PathBuf>) -> Self {
        self.home = Some(dir.into());
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

    fn home_dir(&self) -> Option<PathBuf> {
        if let Some(home) = &self.home {
            return Some(home.clone());
        }
        let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        self.environment
            .iter()
            .find(|(name, _)| name.as_os_str() == variable)
            .map(|(_, value)| PathBuf::from(value))
    }

    fn family_dir(&self, root: Option<PathBuf>) -> Option<PathBuf> {
        root.map(|root| root.join(format!(".{}", self.family)))
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
        self.inserted.push((after, Box::new(source)));
        self
    }

    /// Limits `product`'s files, in every slot, to the key `paths`.
    ///
    /// See [`FileSource::accepts`].
    #[must_use]
    pub fn accepts<P: Into<KeyPath>>(
        mut self,
        product: &AppName,
        paths: impl IntoIterator<Item = P>,
    ) -> Self {
        self.accepts
            .insert(product.clone(), paths.into_iter().map(Into::into).collect());
        self
    }

    /// Maps the variable `variable` to the key path `path` in the `Env` slot.
    ///
    /// Aliases load in the `Env` slot, after its prefixed variables, even
    /// when [`env`](Self::env) is not called.
    #[must_use]
    pub fn alias(mut self, variable: impl Into<String>, path: impl Into<KeyPath>) -> Self {
        self.aliases.push((variable.into(), path.into()));
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
        let (kind, name, value) = match slot {
            Slot::Defaults => (SourceKind::Default, "defaults", self.defaults.clone()?),
            Slot::Flags => (SourceKind::Flag, "flags", self.flags.clone()?),
            _ => return None,
        };
        Some(ValueSource::new(Source::new(kind, name), value))
    }

    fn env_sources(&self) -> Vec<EnvSource> {
        let environment = &self.context.environment;
        let mut sources: Vec<EnvSource> = self
            .env_prefixes
            .iter()
            .map(|prefix| EnvSource::new(prefix.clone(), environment.clone()))
            .collect();
        if !self.aliases.is_empty() {
            // An empty prefix matches nothing, so this source holds only the aliases.
            let mut aliases = EnvSource::new("", environment.clone());
            for (variable, path) in &self.aliases {
                aliases = aliases.alias(variable.clone(), path.clone());
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
            Slot::User => (context.family_dir(context.home_dir()), true, true),
            Slot::Project => (
                context.family_dir(context.project_root.clone()),
                true,
                false,
            ),
            Slot::Local => (
                context.family_dir(context.project_root.clone()),
                false,
                true,
            ),
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
            Some(paths) => file.accepts(paths.iter().cloned()),
            None => file,
        }
    }
}
