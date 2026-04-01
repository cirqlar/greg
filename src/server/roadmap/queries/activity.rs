use std::ops::Deref;

use libsql::{Connection, de};
use time::OffsetDateTime;

use crate::db::tables::{R_ACTIVITIES_T, R_CHANGES_T};
use crate::roadmap::types::RoadmapActivity;
use crate::shared::DatabaseError;

pub async fn add_activity(db: impl Deref<Target = Connection>) -> Result<u32, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "INSERT INTO {R_ACTIVITIES_T} 
                    (timestamp) 
                VALUES
                    (?1)
                RETURNING id
                "
            ),
            [serde_json::to_string(&OffsetDateTime::now_utc()).unwrap()],
        )
        .await?;

    let r = result.next().await?.unwrap();

    Ok(r.get(0)?)
}

pub async fn get_roadmap_activity(
    db: impl Deref<Target = Connection>,
    limit: u32,
    skip: u32,
) -> Result<Vec<RoadmapActivity>, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "SELECT 
                    ra.id,
                    ra.timestamp,
                    IFNULL(rch.count, 0) as change_count
                FROM {R_ACTIVITIES_T} as ra
                LEFT JOIN (
                    SELECT inrch.activity_id, COUNT(inrch.id) as count FROM {R_CHANGES_T} AS inrch
                    GROUP BY inrch.activity_id
                ) rch
                    ON ra.id = rch.activity_id
                ORDER BY ra.id DESC
                LIMIT ?1 OFFSET ?2
                "
            ),
            [limit, skip],
        )
        .await?;

    let mut activities = Vec::new();
    while let Some(row) = result.next().await? {
        let activity: RoadmapActivity = de::from_row(&row)?;
        activities.push(activity);
    }

    Ok(activities)
}

#[cfg(test)]
mod tests {
    use libsql::{Value, params};
    use rstest::rstest;
    use time::ext::NumericalDuration;

    use super::*;
    use crate::db::tests::empty_db;

    #[rstest]
    #[tokio::test]
    async fn can_add_activity(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();

        let id = add_activity(&empty_db).await?;

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT 
                        id,
                        timestamp
                    FROM {R_ACTIVITIES_T}
                "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not retrieve activity");
        };

        if let Value::Integer(db_id) = row.get_value(0)? {
            assert_eq!(db_id, id as i64);
        } else {
            panic!("id isn't an integer");
        }

        if let Value::Text(timestamp) = row.get_value(1)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialized to OffsetDateTime");

            let difference = timestamp - now;
            assert!(difference.abs() < 1.minutes());
        } else {
            panic!("timestamp isn't text");
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_activity(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();

        let id = add_activity(&empty_db).await?;

        let activity = get_roadmap_activity(&empty_db, u32::MAX, 0).await?;

        assert_eq!(activity.len(), 1);
        assert_eq!(activity[0].id, id);
        let difference = activity[0].timestamp - now;
        assert!(difference.abs() < 1.minutes());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_empty_activity(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let activity = get_roadmap_activity(&empty_db, u32::MAX, 0).await?;

        assert_eq!(activity.len(), 0);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_activity_with_limit(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        const ACTIVITY_COUNT: usize = 4;

        let mut activity_ids = Vec::with_capacity(ACTIVITY_COUNT);
        for _ in 0..ACTIVITY_COUNT {
            activity_ids.push(add_activity(&empty_db).await?);
        }

        let activity = get_roadmap_activity(&empty_db, 2, 0).await?;

        assert_eq!(activity.len(), 2);

        for act in activity {
            assert!(activity_ids[2..].contains(&act.id));
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_activity_with_skip(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        const ACTIVITY_COUNT: usize = 4;

        let mut activity_ids = Vec::with_capacity(ACTIVITY_COUNT);
        for _ in 0..ACTIVITY_COUNT {
            activity_ids.push(add_activity(&empty_db).await?);
        }

        let activity = get_roadmap_activity(&empty_db, 2, 2).await?;

        assert_eq!(activity.len(), 2);

        for act in activity {
            assert!(activity_ids[..2].contains(&act.id));
        }

        Ok(())
    }
}
