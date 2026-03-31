use std::ops::Deref;

use libsql::{Connection, de, params};
use time::OffsetDateTime;

use crate::db::tables::{ACTIVITIES_T, SOURCES_T};
use crate::rss::Activity;
use crate::shared::DatabaseError;

pub async fn add_activity(
    db: impl Deref<Target = Connection>,
    source_id: u32,
    url: &str,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!(
            "INSERT INTO {ACTIVITIES_T} 
                (source_id, post_url, timestamp) 
            VALUES 
                (?1, ?2, ?3)
            "
        ),
        (
            source_id,
            url,
            serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
        ),
    )
    .await
    .map_err(DatabaseError::from)
}

pub async fn get_activity(
    db: impl Deref<Target = Connection>,
    limit: u32,
    skip: u32,
) -> Result<Vec<Activity>, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "SELECT 
					a.id, 
					a.post_url, 
					a.timestamp, 
					s.url as source_url
				FROM {ACTIVITIES_T} AS a
				INNER JOIN {SOURCES_T} AS s
					ON a.source_id = s.id
				ORDER BY a.id DESC
				LIMIT ?1 OFFSET ?2
				"
            ),
            [limit, skip],
        )
        .await?;

    let mut activities = Vec::new();
    while let Some(row) = result.next().await? {
        let source: Activity = de::from_row(&row)?;
        activities.push(source);
    }

    Ok(activities)
}

pub async fn get_source_activity(
    db: impl Deref<Target = Connection>,
    limit: u32,
    skip: u32,
    source_id: u32,
) -> Result<Vec<Activity>, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "SELECT 
					a.id, 
					a.post_url, 
					a.timestamp, 
					s.url as source_url
				FROM {ACTIVITIES_T} AS a
				INNER JOIN {SOURCES_T} AS s
					ON a.source_id = s.id
                WHERE a.source_id = ?3
				ORDER BY a.id DESC
				LIMIT ?1 OFFSET ?2
				"
            ),
            [limit, skip, source_id],
        )
        .await?;

    let mut activities = Vec::new();
    while let Some(row) = result.next().await? {
        let source: Activity = de::from_row(&row)?;
        activities.push(source);
    }

    Ok(activities)
}

pub async fn delete_all_activity(
    db: impl Deref<Target = Connection>,
) -> Result<u64, DatabaseError> {
    db.execute(&format!("DELETE FROM {ACTIVITIES_T}"), params!())
        .await
        .map_err(|e| e.into())
}

pub async fn delete_activity(
    db: impl Deref<Target = Connection>,
    num: u32,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!(
            "DELETE FROM {ACTIVITIES_T} 
            WHERE id IN (
                SELECT id 
                FROM {ACTIVITIES_T} 
                ORDER BY id ASC 
                LIMIT ?1
            )
            "
        ),
        [num],
    )
    .await
    .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use libsql::Value;
    use rstest::rstest;
    use time::ext::NumericalDuration;

    use super::super::sources::{add_source, get_sources};
    use super::*;
    use crate::db::tests::empty_db;

    #[rstest]
    #[tokio::test]
    async fn can_add_activity(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let source_url = "http://fake_source.com";
        let activity_url = "http://fake_source.com/activity_1";
        let now = OffsetDateTime::now_utc();

        add_source(&empty_db, source_url.to_string()).await?;
        let source = get_sources(&empty_db).await?.pop().unwrap();

        let rows_affected = add_activity(&empty_db, source.id, activity_url).await?;

        assert_eq!(rows_affected, 1);

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
					post_url, 
					timestamp, 
					source_id
				FROM {ACTIVITIES_T}
				"
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Couldn't retrieve activity");
        };

        if let Value::Text(post_url) = row.get_value(0)? {
            assert_eq!(post_url, activity_url.to_string());
        } else {
            panic!("Post url is not a string");
        }

        if let Value::Text(timestamp) = row.get_value(1)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialised to OffsetDateTime");

            let difference = timestamp - now;

            assert!(difference.abs() <= 1.minutes());
        } else {
            panic!("Post url is not a string");
        }

        if let Value::Integer(source_id) = row.get_value(2)? {
            assert_eq!(source_id, source.id as i64);
        } else {
            panic!("Post url is not a string");
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally, will be fixed in a future migration"]
    async fn can_not_add_activity_for_non_existent_source(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let activity_url = "http://fake_source.com/activity_1";

        let result = add_activity(&empty_db, 0, activity_url).await;

        assert!(result.is_err());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_activity(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let source_url = "http://fake_source.com";
        let activity_urls = [
            "http://fake_source.com/activity_1",
            "http://fake_source.com/activity_2",
            "http://fake_source.com/activity_3",
            "http://fake_source.com/activity_4",
        ];

        add_source(&empty_db, source_url.to_string()).await?;
        let source = get_sources(&empty_db).await?.pop().unwrap();

        for url in activity_urls.iter() {
            add_activity(&empty_db, source.id, url).await?;
        }

        let activity = get_activity(&empty_db, u32::MAX, 0).await?;

        assert_eq!(activity.len(), activity_urls.len());

        for act in activity {
            assert!(activity_urls.contains(&act.post_url.as_str()));
            assert_eq!(act.source_url.as_str(), source_url);
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_empty_activity(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let activity = get_activity(&empty_db, u32::MAX, 0).await?;

        assert!(activity.is_empty());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_activity_with_limit(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let source_url = "http://fake_source.com";
        let activity_urls = [
            "http://fake_source.com/activity_1",
            "http://fake_source.com/activity_2",
            "http://fake_source.com/activity_3",
            "http://fake_source.com/activity_4",
        ];

        add_source(&empty_db, source_url.to_string()).await?;
        let source = get_sources(&empty_db).await?.pop().unwrap();

        for url in activity_urls.iter() {
            add_activity(&empty_db, source.id, url).await?;
        }

        let activity = get_activity(&empty_db, 2, 0).await?;

        assert_eq!(activity.len(), 2);

        for act in activity {
            assert!(activity_urls[2..4].contains(&act.post_url.as_str()));
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_activity_with_skip(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let source_url = "http://fake_source.com";
        let activity_urls = [
            "http://fake_source.com/activity_1",
            "http://fake_source.com/activity_2",
            "http://fake_source.com/activity_3",
            "http://fake_source.com/activity_4",
        ];

        add_source(&empty_db, source_url.to_string()).await?;
        let source = get_sources(&empty_db).await?.pop().unwrap();

        for url in activity_urls.iter() {
            add_activity(&empty_db, source.id, url).await?;
        }

        let activity = get_activity(&empty_db, 2, 2).await?;

        assert_eq!(activity.len(), 2);

        for act in activity {
            assert!(activity_urls[0..2].contains(&act.post_url.as_str()));
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_source_activity(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let source_1_url = "http://fake_source_1.com";
        let source_1_activity_urls = [
            "http://fake_source_1.com/activity_1",
            "http://fake_source_1.com/activity_2",
            "http://fake_source_1.com/activity_3",
            "http://fake_source_1.com/activity_4",
        ];

        let source_2_url = "http://fake_source_2.com";
        let source_2_activity_urls = [
            "http://fake_source_2.com/activity_1",
            "http://fake_source_2.com/activity_2",
            "http://fake_source_2.com/activity_3",
            "http://fake_source_2.com/activity_4",
        ];

        add_source(&empty_db, source_1_url.to_string()).await?;
        add_source(&empty_db, source_2_url.to_string()).await?;

        let mut sources = get_sources(&empty_db).await?;

        let (source_1, source_2) = if sources[1].url.as_str() == source_1_url {
            (sources.pop().unwrap(), sources.pop().unwrap())
        } else {
            let first = sources.pop().unwrap();
            (sources.pop().unwrap(), first)
        };

        for url in source_1_activity_urls.iter() {
            add_activity(&empty_db, source_1.id, url).await?;
        }

        for url in source_2_activity_urls.iter() {
            add_activity(&empty_db, source_2.id, url).await?;
        }

        let activity = get_source_activity(&empty_db, u32::MAX, 0, source_1.id).await?;

        assert_eq!(activity.len(), source_1_activity_urls.len());

        for act in activity {
            assert!(source_1_activity_urls.contains(&act.post_url.as_str()));
            assert_eq!(act.source_url.as_str(), source_1_url);
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_empty_source_activity(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let activity = get_source_activity(&empty_db, u32::MAX, 0, 0).await?;

        assert!(activity.is_empty());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_source_activity_with_limit(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let source_1_url = "http://fake_source_1.com";
        let source_1_activity_urls = [
            "http://fake_source_1.com/activity_1",
            "http://fake_source_1.com/activity_2",
            "http://fake_source_1.com/activity_3",
            "http://fake_source_1.com/activity_4",
        ];

        let source_2_url = "http://fake_source_2.com";
        let source_2_activity_urls = [
            "http://fake_source_2.com/activity_1",
            "http://fake_source_2.com/activity_2",
            "http://fake_source_2.com/activity_3",
            "http://fake_source_2.com/activity_4",
        ];

        add_source(&empty_db, source_1_url.to_string()).await?;
        add_source(&empty_db, source_2_url.to_string()).await?;

        let mut sources = get_sources(&empty_db).await?;

        let (source_1, source_2) = if sources[1].url.as_str() == source_1_url {
            (sources.pop().unwrap(), sources.pop().unwrap())
        } else {
            let first = sources.pop().unwrap();
            (sources.pop().unwrap(), first)
        };

        for url in source_1_activity_urls.iter() {
            add_activity(&empty_db, source_1.id, url).await?;
        }

        for url in source_2_activity_urls.iter() {
            add_activity(&empty_db, source_2.id, url).await?;
        }

        let activity = get_source_activity(&empty_db, 2, 0, source_1.id).await?;

        assert_eq!(activity.len(), 2);

        for act in activity {
            assert!(source_1_activity_urls[2..4].contains(&act.post_url.as_str()));
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_source_activity_with_skip(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let source_1_url = "http://fake_source_1.com";
        let source_1_activity_urls = [
            "http://fake_source_1.com/activity_1",
            "http://fake_source_1.com/activity_2",
            "http://fake_source_1.com/activity_3",
            "http://fake_source_1.com/activity_4",
        ];

        let source_2_url = "http://fake_source_2.com";
        let source_2_activity_urls = [
            "http://fake_source_2.com/activity_1",
            "http://fake_source_2.com/activity_2",
            "http://fake_source_2.com/activity_3",
            "http://fake_source_2.com/activity_4",
        ];

        add_source(&empty_db, source_1_url.to_string()).await?;
        add_source(&empty_db, source_2_url.to_string()).await?;

        let mut sources = get_sources(&empty_db).await?;

        let (source_1, source_2) = if sources[1].url.as_str() == source_1_url {
            (sources.pop().unwrap(), sources.pop().unwrap())
        } else {
            let first = sources.pop().unwrap();
            (sources.pop().unwrap(), first)
        };

        for url in source_1_activity_urls.iter() {
            add_activity(&empty_db, source_1.id, url).await?;
        }

        for url in source_2_activity_urls.iter() {
            add_activity(&empty_db, source_2.id, url).await?;
        }

        let activity = get_source_activity(&empty_db, 2, 2, source_1.id).await?;

        assert_eq!(activity.len(), 2);

        for act in activity {
            assert!(source_1_activity_urls[0..2].contains(&act.post_url.as_str()));
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_delete_all_activity(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let source_url = "http://fake_source.com";
        let activity_urls = [
            "http://fake_source.com/activity_1",
            "http://fake_source.com/activity_2",
            "http://fake_source.com/activity_3",
            "http://fake_source.com/activity_4",
        ];

        add_source(&empty_db, source_url.to_string()).await?;
        let source = get_sources(&empty_db).await?.pop().unwrap();

        for url in activity_urls.iter() {
            add_activity(&empty_db, source.id, url).await?;
        }

        let rows_affected = delete_all_activity(&empty_db).await?;

        assert_eq!(rows_affected, activity_urls.len() as u64);

        let activity = get_activity(&empty_db, u32::MAX, 0).await?;

        assert!(activity.is_empty());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_delete_empty_all_activity(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let rows_affected = delete_all_activity(&empty_db).await?;

        assert_eq!(rows_affected, 0);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_delete_activity(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let source_url = "http://fake_source.com";
        let activity_urls = [
            "http://fake_source.com/activity_1",
            "http://fake_source.com/activity_2",
            "http://fake_source.com/activity_3",
            "http://fake_source.com/activity_4",
        ];

        add_source(&empty_db, source_url.to_string()).await?;
        let source = get_sources(&empty_db).await?.pop().unwrap();

        for url in activity_urls.iter() {
            add_activity(&empty_db, source.id, url).await?;
        }

        let rows_affected = delete_activity(&empty_db, 2).await?;

        assert_eq!(rows_affected, 2);

        let activity = get_activity(&empty_db, u32::MAX, 0).await?;

        assert_eq!(activity.len(), 2);

        for act in activity {
            assert!(activity_urls[2..4].contains(&act.post_url.as_str()));
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_delete_empty_activity(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let rows_affected = delete_activity(&empty_db, 2).await?;

        assert_eq!(rows_affected, 0);

        Ok(())
    }
}
