use std::sync::Arc;

use libsql::{Transaction, params};

use crate::db::tables::R_ACTIVITIES_T;
use crate::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529132_add_roadmap_activities.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{R_ACTIVITIES_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `timestamp` TEXT NOT NULL
                )"
            ),
            params!(),
        )
        .await?;

    Ok(())
}
