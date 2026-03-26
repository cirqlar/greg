use std::sync::Arc;

use libsql::{Transaction, params};

use crate::server::db::tables::R_TAB_ASSIGNS_T;
use crate::server::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529780_add_roadmap_tab_assigns.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{R_TAB_ASSIGNS_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `activity_id` INTEGER NOT NULL,
                    `tab_id` INTEGER NOT NULL,
                    `timestamp` TEXT NOT NULL
                )"
            ),
            params!(),
        )
        .await?;

    Ok(())
}
