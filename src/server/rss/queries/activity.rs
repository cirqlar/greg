use libsql::{Connection, de, params};

use crate::server::db::tables::{ACTIVITIES_T, SOURCES_T};
use crate::server::rss::Activity;
use crate::server::shared::DatabaseError;

pub async fn get_activity(
    db: Connection,
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
    db: Connection,
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

pub async fn delete_all_activity(db: Connection) -> Result<(), DatabaseError> {
    let _ = db
        .execute(&format!("DELETE FROM {ACTIVITIES_T}"), params!())
        .await?;

    Ok(())
}

pub async fn delete_activity(db: Connection, num: u32) -> Result<u64, DatabaseError> {
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
