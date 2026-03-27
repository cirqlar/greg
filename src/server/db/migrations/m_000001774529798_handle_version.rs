use std::sync::Arc;

use libsql::{Transaction, params};

use crate::db::tables::{SOURCES_T, VERSION_T};
use crate::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "m_000001774529798_handle_version.rs";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {
    // Drop Version before alter as doing it after sometimes causes a database closed error
    let _ = db
        .execute(&format!("DROP TABLE IF EXISTS {VERSION_T}"), params!())
        .await?;

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
        // TODO: would batch execute work on a transaction?

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

    Ok(())
}
