use std::sync::Arc;

use libsql::{Transaction, params};

use crate::server::db::tables::R_CARDS_T;
use crate::server::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529754_add_roadmap_cards.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{R_CARDS_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `roadmap_id` TEXT NOT NULL,
                    `name` TEXT NOT NULL,
                    `description` TEXT NOT NULL,
                    `image_url` TEXT,
                    `slug` TEXT NOT NULL,
                    `timestamp` TEXT NOT NULL
                )"
            ),
            params!(),
        )
        .await?;

    Ok(())
}
