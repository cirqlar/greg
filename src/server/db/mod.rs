mod get;
mod migrations;
pub mod tables;
mod types;

pub use get::{GetDatabaseError, get_database, get_demo_database};
pub use migrations::{ApplyMigrationError, apply_migrations};

#[cfg(test)]
pub mod tests {
    use libsql::{Builder, Connection};
    use rstest::fixture;

    use super::*;

    async fn fill_db(db: &Connection, dump_path: &str) {
        use std::env::var;
        use std::path::PathBuf;

        let mut full_path = PathBuf::from(var("CARGO_MANIFEST_DIR").expect("has manifest dir"));
        full_path.push(dump_path);
        let sql = std::fs::read_to_string(full_path).expect("dump file exists");

        db.execute_batch(&sql).await.expect("Can fill database");
    }

    #[fixture]
    pub async fn empty_db() -> Connection {
        let db = Builder::new_local(":memory:")
            .build()
            .await
            .unwrap()
            .connect()
            .expect("Can connect to in memory database");

        apply_migrations(db.clone())
            .await
            .expect("Can migrate in memory database");

        db
    }

    #[fixture]
    pub async fn filled_db(#[future(awt)] empty_db: Connection) -> Connection {
        fill_db(&empty_db, "tests/dumps/v2_dump.sql").await;

        empty_db
    }
}
