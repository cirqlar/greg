use std::sync::Arc;

use libsql::{Transaction, params};

use crate::db::tables::LOGINS_T;
use crate::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529002_add_logins.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{LOGINS_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `timestamp` TEXT NOT NULL,
                    `key` TEXT NOT NULL UNIQUE
                )"
            ),
            params!(),
        )
        .await?;

    let _ = db
        .execute(
            &format!("CREATE INDEX IF NOT EXISTS idx_key ON {LOGINS_T} (key)"),
            params!(),
        )
        .await?;

    Ok(())
}
