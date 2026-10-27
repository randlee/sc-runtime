/// Remote sqlx engine this layout pairs with SQLite.
pub const REMOTE_ENGINE: &str = "mysql";

/// MySQL store this adaptor reads and writes.
pub use store_mysql::Store as MysqlStore;
pub use store_mysql::open as open_mysql;
/// SQLite store this adaptor reads and writes.
pub use store_sqlite::Store as SqliteStore;
pub use store_sqlite::{get_widget, insert_widget, open as open_sqlite};
