use libsql::{Connection, de, params};

use crate::{
    db::{ACTIVITIES_T, SOURCES_T},
    types::{Activity, Source},
};

pub async fn get_sources(db: Connection) -> anyhow::Result<Vec<Source>> {
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

pub async fn get_activity(db: Connection, limit: u32, skip: u32) -> anyhow::Result<Vec<Activity>> {
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
