use std::sync::Arc;

use libsql::{Transaction, params};

use crate::db::tables::R_TABS_T;
use crate::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529762_add_roadmap_tabs.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{R_TABS_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `roadmap_id` TEXT NOT NULL UNIQUE,
                    `name` TEXT NOT NULL,
                    `slug` TEXT NOT NULL,
                    `timestamp` TEXT NOT NULL
                )"
            ),
            params!(),
        )
        .await?;

    Ok(())
}
