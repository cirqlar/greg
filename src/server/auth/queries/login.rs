use std::ops::Deref;

use libsql::Connection;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::tables::LOGINS_T;
use crate::shared::DatabaseError;

pub async fn save_login_id(
    db: impl Deref<Target = Connection>,
    id: &Uuid,
) -> Result<u64, DatabaseError> {
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
    db: impl Deref<Target = Connection>,
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

#[cfg(test)]
mod tests {
    use libsql::{Connection, Value};
    use rstest::rstest;

    use super::*;
    use crate::db::tests::empty_db;

    #[rstest]
    #[tokio::test]
    async fn can_save_login_key(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let id = Uuid::new_v4();

        save_login_id(&empty_db, &id).await?;

        let mut rows = empty_db
            .query(
                &format!("SELECT key FROM {LOGINS_T} WHERE key = ?1"),
                [id.to_string()],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Did not find key in database");
        };

        let Value::Text(id_from_db) = row.get_value(0)? else {
            panic!("Key isn't text");
        };

        assert_eq!(id_from_db, id.to_string());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_not_save_duplicate_login_key(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let id = Uuid::new_v4();

        save_login_id(&empty_db, &id).await?;
        let second_try = save_login_id(&empty_db, &id).await;

        assert!(second_try.is_err());

        let mut rows = empty_db
            .query(
                &format!("SELECT key FROM {LOGINS_T} WHERE key = ?1"),
                [id.to_string()],
            )
            .await?;

        let Some(_) = rows.next().await? else {
            panic!("Did not find key in database");
        };

        assert!(rows.next().await?.is_none());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_retrieve_timestamp_for_key(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let id = Uuid::new_v4();

        save_login_id(&empty_db, &id).await?;

        let timestamp = get_key_timestamp(&empty_db, &id.to_string()).await?;

        assert!(timestamp.is_some());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn missing_key_does_not_error(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let id = Uuid::new_v4();

        let timestamp = get_key_timestamp(&empty_db, &id.to_string()).await?;

        assert!(timestamp.is_none());

        Ok(())
    }
}
