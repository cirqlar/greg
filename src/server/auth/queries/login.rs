use libsql::Connection;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::tables::LOGINS_T;
use crate::shared::DatabaseError;

pub async fn save_login_id(db: Connection, id: &Uuid) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("INSERT INTO {LOGINS_T} (timestamp, key) VALUES (?1, ?2)"),
        [
            serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            id.to_string(),
        ],
    )
    .await
    .map_err(|e| e.into())
}

pub async fn get_key_timestamp(
    db: Connection,
    key: &str,
) -> Result<Option<OffsetDateTime>, DatabaseError> {
    let mut rows = db
        .query(
            &format!("SELECT timestamp FROM {LOGINS_T} WHERE key = ?1 LIMIT 1"),
            [key],
        )
        .await?;

    if let Some(row) = rows.next().await? {
        let timestamp = serde_json::from_str(row.get_str(0)?)?;
        Ok(Some(timestamp))
    } else {
        Ok(None)
    }
}
