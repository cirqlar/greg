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
