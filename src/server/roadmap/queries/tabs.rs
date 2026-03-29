use std::ops::Deref;

use libsql::{Connection, de, params};
use time::OffsetDateTime;

use crate::db::tables::{R_ACTIVITIES_T, R_TAB_ASSIGNS_T, R_TABS_T, R_WATCHED_TABS_T};
use crate::roadmap::types::{RTab, RoadmapActivity, RoadmapWatchedTab};
use crate::shared::DatabaseError;

pub async fn get_watched_tabs(
    db: impl Deref<Target = Connection>,
) -> Result<Vec<RoadmapWatchedTab>, DatabaseError> {
    let mut result = db
        .query(&format!("SELECT * FROM {R_WATCHED_TABS_T}"), params!())
        .await?;

    let mut tabs = Vec::new();
    while let Some(r) = result.next().await? {
        let wts: RoadmapWatchedTab = de::from_row(&r)?;
        tabs.push(wts);
    }

    Ok(tabs)
}

pub async fn get_roadmap_tabs(
    db: impl Deref<Target = Connection>,
    activity_id: u32,
) -> Result<Vec<RTab>, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "SELECT
                    ra.tab_id as db_id,
                    rt.roadmap_id AS id,
                    rt.name,
                    rt.slug
                FROM {R_TAB_ASSIGNS_T} AS ra
                INNER JOIN {R_TABS_T} AS rt
                    ON ra.tab_id = rt.id
                WHERE ra.activity_id = ?1
                "
            ),
            [activity_id],
        )
        .await?;

    let mut tabs = Vec::new();
    while let Some(r) = result.next().await? {
        let t = de::from_row::<RTab>(&r)?;
        tabs.push(t);
    }

    Ok(tabs)
}

pub async fn get_most_recent_roadmap_tabs(
    db: impl Deref<Target = Connection>,
) -> Result<Vec<RTab>, DatabaseError> {
    let mut result = db
        .query(
            &format!("SELECT * FROM {R_ACTIVITIES_T} ORDER BY id DESC LIMIT 1"),
            params!(),
        )
        .await?;
    let Some(r) = result.next().await? else {
        return Ok(Vec::default());
    };

    let activity: RoadmapActivity = de::from_row(&r)?;

    get_roadmap_tabs(db, activity.id).await
}

pub async fn add_watched_tab(
    db: impl Deref<Target = Connection>,
    tab_roadmap_id: String,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("INSERT INTO {R_WATCHED_TABS_T} (tab_roadmap_id, timestamp) VALUES (?1, ?2)"),
        [
            tab_roadmap_id,
            serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
        ],
    )
    .await
    .map_err(|e| e.into())
}

pub async fn delete_watched_tab(
    db: impl Deref<Target = Connection>,
    watched_tab_id: u32,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("DELETE FROM {R_WATCHED_TABS_T} WHERE id = ?1"),
        [watched_tab_id],
    )
    .await
    .map_err(|e| e.into())
}

async fn save_tab(db: impl Deref<Target = Connection>, tab: &RTab) -> Result<u32, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "INSERT INTO {R_TABS_T} 
                    (roadmap_id, name, slug, timestamp) 
                VALUES 
                    (?1,?2,?3,?4)
                RETURNING id
                "
            ),
            (
                tab.id.as_str(),
                tab.name.as_str(),
                tab.slug.as_str(),
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    let r = result.next().await?.unwrap();

    Ok(r.get(0)?)
}

pub async fn save_tab_assignment(
    db: impl Deref<Target = Connection>,
    activity_id: u32,
    tab_db_id: u32,
) -> Result<(), DatabaseError> {
    let _result = db
        .execute(
            &format!(
                "INSERT INTO {R_TAB_ASSIGNS_T} 
                    (activity_id, tab_id, timestamp) 
                VALUES 
                    (?1,?2,?3)
                "
            ),
            (
                activity_id,
                tab_db_id,
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    Ok(())
}

pub async fn save_tab_and_assignment(
    db: impl Deref<Target = Connection>,
    tab: &RTab,
    activity_id: u32,
) -> Result<u32, DatabaseError> {
    let tab_id = save_tab(db.deref(), tab).await?;

    save_tab_assignment(db, activity_id, tab_id).await?;

    Ok(tab_id)
}
