use std::sync::Arc;

use libsql::{Transaction, params};

use crate::db::tables::SOURCES_T;
use crate::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774527605_add_sources.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{SOURCES_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `url` TEXT NOT NULL UNIQUE,
                    `last_checked` TEXT NOT NULL,
                    `enabled` INTEGER NOT NULL DEFAULT 1,
                    `failed_count` INTEGER NOT NULL DEFAULT 0
                )"
            ),
            params!(),
        )
        .await?;

    Ok(())
}
