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
