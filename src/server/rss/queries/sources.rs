use std::ops::Deref;

use libsql::{Connection, de, params};
use time::{OffsetDateTime, ext::NumericalDuration};

use crate::db::tables::SOURCES_T;
use crate::rss::Source;
use crate::shared::DatabaseError;

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

pub async fn enable_source(
    db: impl Deref<Target = Connection>,
    source_id: u32,
    enabled: bool,
    failed_count: u32,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("UPDATE {SOURCES_T} SET failed_count = ?1, enabled = ?2 WHERE id = ?3"),
        (failed_count, if enabled { 1 } else { 0 }, source_id),
    )
    .await
    .map_err(|e| e.into())
}

/// Will zero out failed count
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

#[cfg(test)]
mod tests {
    use libsql::Value;
    use rstest::rstest;
    use time::ext::NumericalDuration;

    use super::*;
    use crate::db::tests::empty_db;

    #[rstest]
    #[tokio::test]
    async fn can_add_source(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let url = String::from("http://fake_source.com");
        let now = OffsetDateTime::now_utc();

        let rows_affected = add_source(&empty_db, url.clone()).await?;

        assert_eq!(rows_affected, 1);

        let mut rows = empty_db
            .query(
                &format!("SELECT url, last_checked, failed_count, enabled FROM {SOURCES_T} WHERE url = ?1"),
                [url.as_str()],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Did not find source in db");
        };

        if let Value::Text(url_from_db) = row.get_value(0)? {
            assert_eq!(url_from_db, url);
        } else {
            panic!("Url isn't text");
        };

        if let Value::Text(last_checked) = row.get_value(1)? {
            let last_checked = serde_json::from_str::<OffsetDateTime>(&last_checked);
            assert!(
                last_checked.is_ok(),
                "last_checked from db can be deserialized to offsetdatetime"
            );

            let last_checked = last_checked.unwrap();

            let difference = last_checked - now;

            // An hour ago +/- a minute
            assert!(difference.abs() >= 59.minutes());
            assert!(difference.abs() <= 61.minutes());
        } else {
            panic!("last_checked isn't text");
        }

        if let Value::Integer(failed_count) = row.get_value(2)? {
            assert_eq!(failed_count, 0);
        } else {
            panic!("failed count isn't an integer");
        }

        if let Value::Integer(enabled) = row.get_value(3)? {
            assert_eq!(enabled, 1);
        } else {
            panic!("enabled isn't an integer");
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_not_add_source_twice(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let url = String::from("http://fake_source.com");

        add_source(&empty_db, url.clone()).await?;
        let second_try = add_source(&empty_db, url.clone()).await;

        assert!(second_try.is_err());

        let mut rows = empty_db
            .query(
                &format!("SELECT * FROM {SOURCES_T} WHERE url = ?1"),
                [url.as_str()],
            )
            .await?;

        let Some(_) = rows.next().await? else {
            panic!("Did not find first source in db");
        };

        assert!(rows.next().await?.is_none());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_sources(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let urls = ["http://fake_source_1.com", "http://fake_source_2.com"];

        for url in urls.iter() {
            add_source(&empty_db, url.to_string()).await?;
        }

        let sources = get_sources(&empty_db).await?;

        assert_eq!(sources.len(), urls.len());

        for source in sources {
            assert!(urls.contains(&source.url.as_str()));
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_empty_sources(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let sources = get_sources(&empty_db).await?;

        assert!(sources.is_empty());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_disable_source(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let urls = ["http://fake_source_1.com", "http://fake_source_2.com"];

        for url in urls.iter() {
            add_source(&empty_db, url.to_string()).await?;
        }

        let sources = get_sources(&empty_db).await?;

        let source = sources
            .iter()
            .find(|s| s.url == urls[0])
            .expect("Source can be gotten");

        let rows_affected = enable_source(&empty_db, source.id, false, 5).await?;

        assert_eq!(rows_affected, 1);

        let sources = get_sources(&empty_db).await?;

        let source = sources
            .iter()
            .find(|s| s.url == urls[0])
            .expect("Source can be gotten");

        assert!(!source.enabled);
        assert_eq!(source.failed_count, 5);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_reenable_source(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let urls = ["http://fake_source_1.com", "http://fake_source_2.com"];

        for url in urls.iter() {
            add_source(&empty_db, url.to_string()).await?;
        }

        let sources = get_sources(&empty_db).await?;

        let source = sources
            .iter()
            .find(|s| s.url == urls[0])
            .expect("Source can be gotten");

        enable_source(&empty_db, source.id, false, 5).await?;
        let rows_affected = enable_source(&empty_db, source.id, true, 0).await?;

        assert_eq!(rows_affected, 1);

        let sources = get_sources(&empty_db).await?;

        let source = sources
            .iter()
            .find(|s| s.url == urls[0])
            .expect("Source can be gotten");

        assert!(source.enabled);
        assert_eq!(source.failed_count, 0);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_update_source(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let urls = ["http://fake_source_1.com", "http://fake_source_2.com"];

        for url in urls.iter() {
            add_source(&empty_db, url.to_string()).await?;
        }

        let sources = get_sources(&empty_db).await?;

        let source = sources
            .iter()
            .find(|s| s.url == urls[0])
            .expect("Source can be gotten");

        enable_source(&empty_db, source.id, true, 6).await?;

        let now = OffsetDateTime::now_utc();

        let rows_affected = update_source_last_checked(&empty_db, source.id, now).await?;

        assert_eq!(rows_affected, 1);

        let sources = get_sources(&empty_db).await?;

        let source = sources
            .iter()
            .find(|s| s.url == urls[0])
            .expect("Source can be gotten");

        assert_eq!(source.last_checked, now);
        assert_eq!(source.failed_count, 0);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn updating_source_resets_failed_count(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let urls = ["http://fake_source_1.com", "http://fake_source_2.com"];

        for url in urls.iter() {
            add_source(&empty_db, url.to_string()).await?;
        }

        let sources = get_sources(&empty_db).await?;

        let source = sources
            .iter()
            .find(|s| s.url == urls[0])
            .expect("Source can be gotten");

        let now = OffsetDateTime::now_utc();

        let rows_affected = update_source_last_checked(&empty_db, source.id, now).await?;

        assert_eq!(rows_affected, 1);

        let sources = get_sources(&empty_db).await?;

        let source = sources
            .iter()
            .find(|s| s.url == urls[0])
            .expect("Source can be gotten");

        assert_eq!(source.last_checked, now);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_delete_sources(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let urls = ["http://fake_source_1.com", "http://fake_source_2.com"];

        for url in urls.iter() {
            add_source(&empty_db, url.to_string()).await?;
        }

        let deleted_source = get_sources(&empty_db).await?.pop().unwrap();

        let rows_affected = delete_source(&empty_db, deleted_source.id).await?;

        assert_eq!(rows_affected, 1);

        let sources = get_sources(&empty_db).await?;

        assert_eq!(sources.len(), urls.len() - 1);

        assert!(
            sources
                .iter()
                .find(|s| s.url == deleted_source.url)
                .is_none()
        );

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_delete_non_existent_source(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let urls = ["http://fake_source_1.com", "http://fake_source_2.com"];

        for url in urls.iter() {
            add_source(&empty_db, url.to_string()).await?;
        }

        let rows_affected = delete_source(&empty_db, 3).await?;

        assert_eq!(rows_affected, 0);

        let sources = get_sources(&empty_db).await?;

        assert_eq!(sources.len(), urls.len());

        Ok(())
    }
}
