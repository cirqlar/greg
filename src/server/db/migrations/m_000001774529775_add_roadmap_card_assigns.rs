use std::sync::Arc;

use libsql::{Transaction, params};

use crate::server::db::tables::R_CARD_ASSIGNS_T;
use crate::server::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529775_add_roadmap_card_assigns.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{R_CARD_ASSIGNS_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `activity_id` INTEGER NOT NULL,
                    `tab_id` INTEGER NOT NULL,
                    `card_id` INTEGER NOT NULL,
                    `section_position` INTEGER NOT NULL,
                    `card_position` INTEGER NOT NULL,
                    `timestamp` TEXT NOT NULL
                )"
            ),
            params!(),
        )
        .await?;

    Ok(())
}
