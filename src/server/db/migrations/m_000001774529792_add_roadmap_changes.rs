use std::sync::Arc;

use libsql::{Transaction, params};

use crate::server::db::tables::R_CHANGES_T;
use crate::server::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529792_add_roadmap_changes.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{R_CHANGES_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `type` TEXT NOT NULL,
                    `activity_id` INTEGER NOT NULL,
                    `previous_card_id` INTEGER,
                    `current_card_id` INTEGER,
                    `tab_id` INTEGER,
                    `timestamp` TEXT NOT NULL
                )"
            ),
            params!(),
        )
        .await?;

    Ok(())
}
