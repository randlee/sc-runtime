//! Engine-keyed table membership for paired stores.
//!
//! [`EngineStore`](crate::EngineStore) does not read this module. A project
//! whose engines share one schema uses [`Membership::empty`] and can delete
//! `membership.toml` plus this module. The adaptor in sprint b-7 is the first
//! runtime consumer (ADR-RUN-0305 point 5). Until then the `toml` crate
//! parses the file only in tests, so a generated project does not take a
//! runtime parser dependency.

use std::collections::BTreeMap;

/// Failure in a paired-store membership description.
#[derive(Debug, PartialEq, Eq)]
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

/// Tables and columns that exist on one named engine and not on every engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineMembership {
    tables: Vec<String>,
    columns: Vec<String>,
}

impl EngineMembership {
    /// Builds one engine's one-sided names.
    ///
    /// Empty `tables` and empty `columns` are valid. [`Membership::try_new`]
    /// checks the names.
    #[must_use]
    pub fn new(tables: Vec<String>, columns: Vec<String>) -> Self {
        Self { tables, columns }
    }

    /// Tables stored on this engine and not listed as shared.
    #[must_use]
    pub fn tables(&self) -> &[String] {
        &self.tables
    }

    /// Columns stored on this engine only, written as `table.column`.
    #[must_use]
    pub fn columns(&self) -> &[String] {
        &self.columns
    }
}

/// Tables shared by every engine, plus tables and columns that exist on only
/// some engines.
///
/// Keys of [`Membership::only_in`] are engine names (`sqlite`, `postgres`,
/// `mysql`), not a fixed sqlite/remote pair. An empty description is valid
/// when every engine stores the same tables and columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    shared: Vec<String>,
    only_in: BTreeMap<String, EngineMembership>,
}

impl Membership {
    /// A description with no shared tables and no one-sided names.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            shared: Vec::new(),
            only_in: BTreeMap::new(),
        }
    }

    /// Builds a description from already-split names.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] when a name is empty or dotted, a table is
    /// both shared and one-sided, or a column is not `table.column` on a
    /// table that engine stores.
    pub fn try_new(
        shared: Vec<String>,
        only_in: BTreeMap<String, EngineMembership>,
    ) -> Result<Self, MembershipError> {
        let membership = Self { shared, only_in };
        membership.check()?;
        Ok(membership)
    }

    /// Tables that exist on every engine.
    #[must_use]
    pub fn shared(&self) -> &[String] {
        &self.shared
    }

    /// One-sided tables and columns, keyed by engine name.
    #[must_use]
    pub fn only_in(&self) -> &BTreeMap<String, EngineMembership> {
        &self.only_in
    }

    /// Engine names that declare a one-sided table or column.
    pub fn engines(&self) -> impl Iterator<Item = &str> {
        self.only_in.keys().map(String::as_str)
    }

    fn check(&self) -> Result<(), MembershipError> {
        check_names("shared", &self.shared)?;
        for (engine, side) in &self.only_in {
            if engine.is_empty() || engine.contains('.') {
                return Err(MembershipError::new(format!(
                    "engine name {engine:?} is empty or dotted"
                )));
            }
            check_names(&format!("only_in.{engine}.tables"), side.tables())?;
            for table in side.tables() {
                if self.shared.iter().any(|name| name == table) {
                    return Err(MembershipError::new(format!(
                        "table {table} is shared and listed under {engine}"
                    )));
                }
            }
            for column in side.columns() {
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
                let on_engine = self.shared.iter().any(|name| name == table)
                    || side.tables().iter().any(|name| name == table);
                if !on_engine {
                    return Err(MembershipError::new(format!(
                        "column {column} names a table {engine} does not store"
                    )));
                }
            }
        }
        Ok(())
    }
}

fn check_names(label: &str, names: &[String]) -> Result<(), MembershipError> {
    let mut seen = Vec::new();
    for name in names {
        if name.is_empty() || name.contains('.') {
            return Err(MembershipError::new(format!(
                "{label} contains an empty or dotted name"
            )));
        }
        if seen.contains(name) {
            return Err(MembershipError::new(format!(
                "{label} lists {name} more than once"
            )));
        }
        seen.push(name.clone());
    }
    Ok(())
}
