use std::ops::Deref;

use libsql::{Connection, de, params};
use time::{OffsetDateTime, ext::NumericalDuration};

use crate::db::tables::SOURCES_T;
use crate::rss::Source;
use crate::shared::DatabaseError;

pub async fn get_sources(
    db: impl Deref<Target = Connection>,
) -> Result<Vec<Source>, DatabaseError> {
    let mut result = db
        .query(&format!("SELECT * FROM {SOURCES_T}"), params!())
        .await?;

    let mut sources = Vec::new();
    while let Some(row) = result.next().await? {
        let source: Source = de::from_row(&row)?;
        sources.push(source);
    }

    Ok(sources)
}

pub async fn add_source(
    db: impl Deref<Target = Connection>,
    source_url: String,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("INSERT INTO {SOURCES_T} (url, last_checked) VALUES (?1, ?2)"),
        [
            source_url,
            serde_json::to_string(&(OffsetDateTime::now_utc() - 1.hours())).unwrap(),
        ],
    )
    .await
    .map_err(|e| e.into())
}

pub async fn enable_source(
    db: impl Deref<Target = Connection>,
    source_id: u32,
    enabled: bool,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("UPDATE {SOURCES_T} SET failed_count = ?1, enabled = ?2 WHERE id = ?3"),
        (0, if enabled { 1 } else { 0 }, source_id),
    )
    .await
    .map_err(|e| e.into())
}

// TODO: Find better name or a way to combine with other update functions
pub async fn maybe_disable_source(
    db: impl Deref<Target = Connection>,
    source_id: u32,
    enabled: u8,
    failed_count: u32,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("UPDATE {SOURCES_T} SET failed_count = ?1, enabled = ?2 WHERE id = ?3"),
        (failed_count, enabled, source_id),
    )
    .await
    .map_err(DatabaseError::from)
}

pub async fn update_source_last_checked(
    db: impl Deref<Target = Connection>,
    source_id: u32,
    most_recent: OffsetDateTime,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("UPDATE {SOURCES_T} SET last_checked = ?1, failed_count = ?2 WHERE id = ?3"),
        (serde_json::to_string(&most_recent).unwrap(), 0, source_id),
    )
    .await
    .map_err(DatabaseError::from)
}

pub async fn delete_source(
    db: impl Deref<Target = Connection>,
    source_id: u32,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("DELETE FROM {SOURCES_T} WHERE id = ?1"),
        [source_id],
    )
    .await
    .map_err(|e| e.into())
}
