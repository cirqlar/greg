use std::sync::Arc;

use libsql::{Transaction, params};

use crate::server::db::tables::{SOURCES_T, VERSION_T};
use crate::server::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529798_handle_version.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    // Add enabled and failed_count if not there
    // It shouldn't be possible to have one and not
    // the other as they're added at the same time
    let mut rows = db
        .query(
            &format!("SELECT 1 from PRAGMA_TABLE_INFO('{SOURCES_T}') WHERE name='enabled'"),
            params!(),
        )
        .await?;

    if rows.next().await?.is_none() {
        let _ = db
            .execute(
                &format!(
                    "ALTER TABLE {SOURCES_T}
                    ADD enabled INTEGER NOT NULL DEFAULT 1"
                ),
                params!(),
            )
            .await?;

        let _ = db
            .execute(
                &format!(
                    "ALTER TABLE {SOURCES_T}
                    ADD failed_count INTEGER NOT NULL DEFAULT 0"
                ),
                params!(),
            )
            .await?;
    }

    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{VERSION_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `version_number` INTEGER NOT NULL
                )"
            ),
            params!(),
        )
        .await?;

    let mut res = db
        .query(&format!("SELECT * FROM {VERSION_T} WHERE id = ?1"), [1])
        .await?;

    let mut version: u32 = 1;
    if let Some(row) = res.next().await? {
        version = row.get(1)?;
    };

    if version < 2 {
        // add fields
        todo!()
    }

    let _ = db
        .execute(&format!("DROP TABLE IF EXISTS {VERSION_T}"), params!())
        .await?;

    Ok(())
}
