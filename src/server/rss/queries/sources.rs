use libsql::{Connection, de, params};
use time::{OffsetDateTime, ext::NumericalDuration};

use crate::server::db::tables::SOURCES_T;
use crate::server::rss::Source;
use crate::server::shared::DatabaseError;

pub async fn get_sources(db: Connection) -> Result<Vec<Source>, DatabaseError> {
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

pub async fn add_source(db: Connection, source_url: String) -> Result<u64, DatabaseError> {
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
    db: Connection,
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

pub async fn delete_source(db: Connection, source_id: u32) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("DELETE FROM {SOURCES_T} WHERE id = ?1"),
        [source_id],
    )
    .await
    .map_err(|e| e.into())
}
