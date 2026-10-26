//! One decoder for `membership.toml`.
//!
//! Schema tests own this file. The store-adaptor tests include it with
//! `#[path]` so they do not declare a second TOML shape. The `toml` crate
//! stays a dev-dependency. The generated runtime graph does not parse this
//! file.

use std::collections::BTreeMap;

use schema::{EngineMembership, Membership};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MembershipFile {
    #[serde(default)]
    shared: Vec<String>,
    #[serde(default)]
    only_in: BTreeMap<String, EngineFile>,
}

#[derive(Debug, Deserialize, Default)]
struct EngineFile {
    #[serde(default)]
    tables: Vec<String>,
    #[serde(default)]
    columns: Vec<String>,
}

pub(crate) fn membership_from_toml(text: &str) -> Result<Membership, schema::MembershipError> {
    let file: MembershipFile = toml::from_str(text).expect("membership toml");
    let only_in = file
        .only_in
        .into_iter()
        .map(|(engine, side)| (engine, EngineMembership::new(side.tables, side.columns)))
        .collect();
    Membership::try_new(file.shared, only_in)
}
