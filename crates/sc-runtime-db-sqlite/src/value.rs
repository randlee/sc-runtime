use rusqlite::types::{ToSql, ToSqlOutput, Value, ValueRef};

/// A bound SQL parameter.
///
/// The writer accepts owned values because the job runs later on another thread.
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

impl ToSql for SqlValue {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            Self::Null => Ok(ToSqlOutput::Owned(Value::Null)),
            Self::Integer(value) => Ok(ToSqlOutput::Owned(Value::Integer(*value))),
            Self::Real(value) => Ok(ToSqlOutput::Owned(Value::Real(*value))),
            Self::Text(value) => Ok(ToSqlOutput::Borrowed(ValueRef::Text(value.as_bytes()))),
            Self::Blob(value) => Ok(ToSqlOutput::Borrowed(ValueRef::Blob(value))),
        }
    }
}
