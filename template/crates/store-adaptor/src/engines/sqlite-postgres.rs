/// Remote sqlx engine this layout pairs with SQLite.
pub const REMOTE_ENGINE: &str = "postgres";

/// Postgres store this adaptor reads and writes.
pub use store_postgres::Store as PostgresStore;
pub use store_postgres::open as open_postgres;
/// SQLite store this adaptor reads and writes.
pub use store_sqlite::Store as SqliteStore;
pub use store_sqlite::{get_widget, insert_widget, open as open_sqlite, probe as probe_sqlite};
pub use store_postgres::probe as probe_postgres;
