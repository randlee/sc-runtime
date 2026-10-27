// Layout flags and the consumer rollup. `lib.rs` includes the shared adaptor,
// then this file, then the layout engine.

/// Consumer-owned rollup for the copy and host-aggregate paths.
///
/// The consumer replaces this type to drop a one-sided column or fold hosts.
/// The default keeps every host row. Postgres and MySQL layouts include this
/// same type.
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
/// Returns [`schema::StoreError`] when `membership.toml` is not a valid
/// description. The code is [`schema::StoreCode::Query`]. Call
/// [`schema::StoreError::remediation`] for what to do next.
pub fn layout_flags() -> Result<TableFlags, schema::StoreError> {
    let membership =
        membership_toml::membership_from_toml(include_str!("../../schema/membership.toml"))?;
    Ok(TableFlags::from_membership(&membership, REMOTE_ENGINE))
}
