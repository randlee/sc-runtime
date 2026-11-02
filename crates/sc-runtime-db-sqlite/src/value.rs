/// A bound SQL parameter.
///
/// The writer accepts owned values because the job runs later on another thread.
/// Conversion to rusqlite values happens inside the writer thread. This type
/// does not implement rusqlite's `ToSql`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SqlValue {
    /// SQL `NULL`.
    Null,
    /// A 64-bit integer.
    Integer(i64),
    /// A 64-bit float.
    Real(f64),
    /// UTF-8 text.
    Text(String),
    /// An untyped blob.
    Blob(Vec<u8>),
}
