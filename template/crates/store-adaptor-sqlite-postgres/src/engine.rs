// SQLite and Postgres. `lib.rs` includes the shared adaptor, then this file.

use std::collections::BTreeMap;

use schema::EngineMembership;
use serde::Deserialize;

/// Remote sqlx engine this layout pairs with SQLite.
pub const REMOTE_ENGINE: &str = "postgres";

/// Postgres store this adaptor reads and writes.
pub use store_postgres::Store as PostgresStore;
pub use store_postgres::open as open_postgres;
/// SQLite store this adaptor reads and writes.
pub use store_sqlite::Store as SqliteStore;
pub use store_sqlite::{get_widget, insert_widget, open as open_sqlite};

/// Consumer-owned rollup for the copy and host-aggregate paths.
///
/// The consumer replaces this type to drop a one-sided column or fold hosts.
/// The default keeps every host row.
pub struct CopyRollup;

impl Rollup for CopyRollup {
    fn rollup(&self, host_rows: &[RowRecord], _one_sided_columns: &[String]) -> Vec<RowRecord> {
        host_rows.to_vec()
    }
}

/// Flags from the schema crate's `membership.toml` for [`REMOTE_ENGINE`].
///
/// # Errors
///
/// Returns an error string when `membership.toml` is not a valid description.
pub fn layout_flags() -> Result<TableFlags, String> {
    let membership = membership_from_toml(include_str!("../../schema/membership.toml"))?;
    Ok(TableFlags::from_membership(&membership, REMOTE_ENGINE))
}

fn membership_from_toml(text: &str) -> Result<schema::Membership, String> {
    let file: MembershipFile = toml::from_str(text).map_err(|err| err.to_string())?;
    let only_in = file
        .only_in
        .into_iter()
        .map(|(engine, side)| (engine, EngineMembership::new(side.tables, side.columns)))
        .collect();
    schema::Membership::try_new(file.shared, only_in).map_err(|err| err.to_string())
}

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

#[cfg(test)]
mod layout_tests {
    use crate::Rollup;
    use schema::RowRecord;

    #[test]
    fn membership_names_sqlite_and_postgres() {
        let flags = crate::layout_flags().expect("membership");
        assert_eq!(flags.sqlite_only, vec!["local_notes".to_owned()]);
        assert_eq!(flags.remote_only, vec!["remote_audit".to_owned()]);
        assert_eq!(flags.column_on_one_side, vec!["widgets.host_id".to_owned()]);
        assert_eq!(crate::REMOTE_ENGINE, "postgres");
        let rows = [RowRecord {
            table: "widgets".to_owned(),
            payload: br#"{"id":"w1"}"#.to_vec(),
        }];
        assert_eq!(
            crate::CopyRollup.rollup(&rows, &flags.column_on_one_side),
            rows
        );
    }
}
