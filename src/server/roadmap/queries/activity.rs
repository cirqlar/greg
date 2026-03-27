use libsql::{Connection, de};

use crate::db::tables::{R_ACTIVITIES_T, R_CHANGES_T};
use crate::roadmap::types::RoadmapActivity;
use crate::shared::DatabaseError;

pub async fn get_roadmap_activity(
    db: Connection,
    limit: u32,
    skip: u32,
) -> Result<Vec<RoadmapActivity>, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "SELECT 
                    ra.id,
                    ra.timestamp,
                    rch.count as change_count
                FROM {R_ACTIVITIES_T} as ra
                LEFT JOIN (
                    SELECT inrch.activity_id, COUNT(inrch.id) as count FROM {R_CHANGES_T} AS inrch
                    WHERE
                        inrch.type = 'tab_removed'
                        OR inrch.type = 'tab_added'
                        OR inrch.type = 'card_removed'
                        OR inrch.type = 'card_added'
                        OR inrch.type = 'card_modified'
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
