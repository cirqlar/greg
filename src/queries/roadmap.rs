use std::collections::HashMap;

use libsql::{Connection, de, params};

use crate::{
    db::{
        R_ACTIVITIES_T, R_CARD_ASSIGNS_T, R_CARDS_T, R_CHANGES_T, R_TAB_ASSIGNS_T, R_TABS_T,
        R_WATCHED_TABS_T,
    },
    types::{RCard, RDBChangeAlt, RTab, Roadmap, RoadmapActivity, RoadmapWatchedTab},
};

pub async fn get_most_recent_roadmap(db: Connection) -> anyhow::Result<Option<Roadmap>> {
    let mut result = db
        .query(
            &format!("SELECT * FROM {R_ACTIVITIES_T} ORDER BY id DESC LIMIT 1"),
            params!(),
        )
        .await?;
    let Some(r) = result.next().await? else {
        return Ok(None);
    };

    let activity: RoadmapActivity = de::from_row(&r)?;

    // Get Tabs
    let tabs = get_roadmap_tabs(db.clone(), activity.id).await?;

    // Get Cards
    let mut result = db
        .query(
            &format!(
                "SELECT 
                    ra.id as assign_db_id,
                    ra.tab_id,
                    ra.card_id as db_id,
                    ra.section_position,
                    ra.card_position,

                    rc.roadmap_id AS id,
                    rc.name,
                    rc.description,
                    rc.image_url,
                    rc.slug
                FROM {R_CARD_ASSIGNS_T} AS ra
                INNER JOIN {R_CARDS_T} AS rc 
                    ON ra.card_id = rc.id
                WHERE ra.activity_id = ?1
                "
            ),
            [activity.id],
        )
        .await?;

    let mut cards: HashMap<String, Vec<RCard>> = HashMap::new();

    while let Some(r) = result.next().await? {
        let c = de::from_row::<RCard>(&r)?;
        let t_id = c.tab_id.unwrap();
        let t_id = tabs
            .iter()
            .find(|t| *t.db_id.as_ref().unwrap() == t_id)
            .unwrap()
            .id
            .clone();
        let v = cards.entry(t_id).or_default();
        v.push(c);
    }

    cards
        .values_mut()
        .for_each(|c| c.sort_by_key(|c| c.id.clone()));

    Ok(Some(Roadmap::with_data(tabs, cards)))
}

pub async fn get_watched_tabs(db: Connection) -> anyhow::Result<Vec<RoadmapWatchedTab>> {
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

pub async fn get_roadmap_activities(
    db: Connection,
    limit: u32,
    skip: u32,
) -> anyhow::Result<Vec<RoadmapActivity>> {
    let mut result = db
        .query(
            &format!(
                "SELECT 
                    ra.id,
                    ra.timestamp
                FROM {R_ACTIVITIES_T} as ra
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

pub async fn get_roadmap_tabs(db: Connection, activity_id: u32) -> anyhow::Result<Vec<RTab>> {
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

pub async fn get_most_recent_roadmap_tabs(db: Connection) -> anyhow::Result<Vec<RTab>> {
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

    get_roadmap_tabs(db.clone(), activity.id).await
}

pub async fn get_roadmap_changes(
    db: Connection,
    activity_id: u32,
) -> anyhow::Result<Vec<RDBChangeAlt>> {
    let mut result = db
        .query(
            &format!(
                "SELECT
                    rch.id, rch.type,

                    rc1.id AS previous_card_db_id, rc1.roadmap_id AS previous_card_id,
                    rc1.name AS previous_card_name, rc1.description AS previous_card_description,
                    rc1.image_url  AS previous_card_image_url, rc1.slug AS previous_card_slug,

                    rc2.id AS current_card_db_id, rc2.roadmap_id AS current_card_id,
                    rc2.name AS current_card_name, rc2.description AS current_card_description,
                    rc2.image_url  AS current_card_image_url, rc2.slug AS current_card_slug,

                    rt.id AS tab_db_id, rt.roadmap_id AS tab_id,
                    rt.name AS tab_name, rt.slug AS tab_slug
                FROM {R_CHANGES_T} AS rch
                LEFT JOIN {R_CARDS_T} as rc1
                    ON rch.previous_card_id = rc1.id
                LEFT JOIN {R_CARDS_T} as rc2
                    ON rch.current_card_id = rc2.id
                LEFT JOIN {R_TABS_T} as rt
                    ON rch.tab_id = rt.id
                WHERE rch.activity_id = ?1
                "
            ),
            [activity_id],
        )
        .await?;

    let mut changes = Vec::new();
    while let Some(r) = result.next().await? {
        let c = de::from_row(&r)?;
        changes.push(c);
    }

    Ok(changes)
}
