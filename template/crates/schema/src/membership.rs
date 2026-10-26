//! Table membership for two stores that share a schema, or a similar one.

use serde::Deserialize;

/// Failure reading the paired-store membership description.
#[derive(Debug)]
pub struct MembershipError {
    message: String,
}

impl std::fmt::Display for MembershipError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MembershipError {}

impl MembershipError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct MembershipFile {
    shared: Vec<String>,
    sqlite_only: Vec<String>,
    remote_only: Vec<String>,
    column_on_one_side: Vec<String>,
}

/// Tables and columns that are shared, or that exist on only one engine.
///
/// Loaded from `membership.toml`. That file is not an answers-schema property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    shared: Vec<String>,
    sqlite_only: Vec<String>,
    remote_only: Vec<String>,
    column_on_one_side: Vec<String>,
}

impl Membership {
    /// Parses the membership description from `text`.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] when `text` is not the membership document,
    /// or when it does not name a shared table, a SQLite-only table, a
    /// remote-only table, and a column that exists on only one side.
    pub fn parse(text: &str) -> Result<Self, MembershipError> {
        let file: MembershipFile = toml::from_str(text)
            .map_err(|err| MembershipError::new(format!("membership document: {err}")))?;
        let membership = Self {
            shared: file.shared,
            sqlite_only: file.sqlite_only,
            remote_only: file.remote_only,
            column_on_one_side: file.column_on_one_side,
        };
        membership.check()?;
        Ok(membership)
    }

    /// Loads the description shipped with this crate.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] when the shipped document is invalid.
    pub fn bundled() -> Result<Self, MembershipError> {
        Self::parse(include_str!("../membership.toml"))
    }

    /// Tables that exist on both engines.
    #[must_use]
    pub fn shared(&self) -> &[String] {
        &self.shared
    }

    /// Tables that exist only in SQLite.
    #[must_use]
    pub fn sqlite_only(&self) -> &[String] {
        &self.sqlite_only
    }

    /// Tables that exist only on the remote engine.
    #[must_use]
    pub fn remote_only(&self) -> &[String] {
        &self.remote_only
    }

    /// Columns that exist on only one side, written as `table.column`.
    #[must_use]
    pub fn column_on_one_side(&self) -> &[String] {
        &self.column_on_one_side
    }

    fn check(&self) -> Result<(), MembershipError> {
        require_names("shared", &self.shared)?;
        require_names("sqlite_only", &self.sqlite_only)?;
        require_names("remote_only", &self.remote_only)?;
        if self.column_on_one_side.is_empty() {
            return Err(MembershipError::new("column_on_one_side names no column"));
        }
        let mut seen = Vec::new();
        for name in self
            .shared
            .iter()
            .chain(self.sqlite_only.iter())
            .chain(self.remote_only.iter())
        {
            if seen.contains(name) {
                return Err(MembershipError::new(format!(
                    "table {name} is listed more than once"
                )));
            }
            seen.push(name.clone());
        }
        for column in &self.column_on_one_side {
            let Some((table, field)) = column.split_once('.') else {
                return Err(MembershipError::new(format!(
                    "column {column} is not table.column"
                )));
            };
            if table.is_empty() || field.is_empty() || field.contains('.') {
                return Err(MembershipError::new(format!(
                    "column {column} is not table.column"
                )));
            }
            if !seen.iter().any(|name| name == table) {
                return Err(MembershipError::new(format!(
                    "column {column} names an unknown table"
                )));
            }
        }
        Ok(())
    }
}

fn require_names(label: &str, names: &[String]) -> Result<(), MembershipError> {
    if names.is_empty() {
        return Err(MembershipError::new(format!("{label} names no table")));
    }
    if names
        .iter()
        .any(|name| name.is_empty() || name.contains('.'))
    {
        return Err(MembershipError::new(format!(
            "{label} contains an empty or dotted name"
        )));
    }
    Ok(())
}
