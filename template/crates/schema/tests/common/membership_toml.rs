//! One decoder for `membership.toml`.
//!
//! Schema tests compile this module. The two-store adaptor loads the same file
//! with `#[path]` so a second TOML shape is not declared. The file lives next
//! to the schema tests because one-store layouts do not generate an adaptor
//! crate.

use schema::{EngineMembership, Membership, StoreCode, StoreError};

/// Reads a membership description from TOML text.
///
/// # Errors
///
/// Returns [`StoreCode::Membership`] when `text` is not the membership document
/// or the document is not a valid [`Membership`]. The remediation names
/// `membership.toml` and the engine, table, and column rule.
pub(crate) fn membership_from_toml(text: &str) -> Result<Membership, StoreError> {
    let file: MembershipFile = toml::from_str(text).map_err(|err| {
        StoreError::new(
            StoreCode::Membership,
            format!("membership.toml is not valid TOML: {err}"),
        )
    })?;
    let only_in = file
        .only_in
        .into_iter()
        .map(|(engine, side)| (engine, EngineMembership::new(side.tables, side.columns)))
        .collect();
    Membership::try_new(file.shared, only_in)
        .map_err(|err| StoreError::new(StoreCode::Membership, format!("membership.toml: {err}")))
}

#[derive(Debug, serde::Deserialize)]
struct MembershipFile {
    #[serde(default)]
    shared: Vec<String>,
    #[serde(default)]
    only_in: std::collections::BTreeMap<String, EngineFile>,
}

#[derive(Debug, serde::Deserialize, Default)]
struct EngineFile {
    #[serde(default)]
    tables: Vec<String>,
    #[serde(default)]
    columns: Vec<String>,
}
