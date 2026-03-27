use std::sync::Arc;

use libsql::{Transaction, params};

use crate::db::tables::ACTIVITIES_T;
use crate::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774528994_add_activities.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{ACTIVITIES_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `source_id` INTEGER NOT NULL,
                    `post_url` TEXT NOT NULL,
                    `timestamp` TEXT NOT NULL
                )"
            ),
            params!(),
        )
        .await?;

    Ok(())
}
