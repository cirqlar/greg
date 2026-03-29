use std::collections::HashMap;
use std::ops::Deref;

use libsql::{Connection, de, params};
use time::OffsetDateTime;

use super::tabs::get_roadmap_tabs;
use crate::db::tables::{R_ACTIVITIES_T, R_CARD_ASSIGNS_T, R_CARDS_T};
use crate::roadmap::types::{RCard, Roadmap, RoadmapActivity};
use crate::shared::DatabaseError;

pub async fn get_most_recent_roadmap(
    db: impl Deref<Target = Connection>,
) -> Result<Option<Roadmap>, DatabaseError> {
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
    let tabs = get_roadmap_tabs(db.deref(), activity.id).await?;

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

pub async fn new_activity(db: impl Deref<Target = Connection>) -> Result<u32, DatabaseError> {
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
