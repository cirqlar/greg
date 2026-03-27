use std::sync::Arc;

use libsql::{Transaction, params};

use crate::db::tables::R_WATCHED_TABS_T;
use crate::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529138_add_roadmap_watched_tabs.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{R_WATCHED_TABS_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `tab_roadmap_id` TEXT NOT NULL UNIQUE,
                    `timestamp` TEXT NOT NULL
                )"
            ),
            params!(),
        )
        .await?;

    Ok(())
}
