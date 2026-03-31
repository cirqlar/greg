use std::collections::HashMap;
use std::ops::Deref;

use libsql::{Connection, de};
use log::info;
use time::OffsetDateTime;

use crate::db::tables::{R_CARD_ASSIGNS_T, R_CARDS_T};
use crate::roadmap::types::{CardAssignmentInfo, RCard, Roadmap};
use crate::shared::DatabaseError;

async fn add_card(db: impl Deref<Target = Connection>, card: &RCard) -> Result<u32, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "INSERT INTO {R_CARDS_T} 
                    (roadmap_id, name, description, image_url, slug, timestamp)
                VALUES 
                    (?1,?2,?3,?4,?5,?6)
                RETURNING id
                "
            ),
            (
                card.id.as_str(),
                card.name.as_str(),
                card.description.as_str(),
                card.image_url.as_deref(),
                card.slug.as_str(),
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    let r = result.next().await?.unwrap();

    Ok(r.get(0)?)
}

pub async fn assign_card(
    db: impl Deref<Target = Connection>,
    card_id: u32,
    assign_info: CardAssignmentInfo,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!(
            "INSERT INTO {R_CARD_ASSIGNS_T} 
                (activity_id, tab_id, card_id, section_position, card_position, timestamp) 
            VALUES 
                (?1,?2,?3,?4,?5,?6)
            "
        ),
        (
            assign_info.activity_id,
            assign_info.tab_id,
            card_id,
            assign_info.section_pos,
            assign_info.card_pos,
            serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
        ),
    )
    .await
    .map_err(|e| e.into())
}

pub async fn add_and_assign_card(
    db: impl Deref<Target = Connection>,
    card: &RCard,
    assign_info: CardAssignmentInfo,
) -> Result<u32, DatabaseError> {
    let card_id = add_card(db.deref(), card).await?;
    assign_card(db, card_id, assign_info).await?;
    Ok(card_id)
}

pub async fn add_and_assign_tab_cards(
    db: impl Deref<Target = Connection>,
    roadmap_activity_id: u32,
    tab_db_id: u32,
    cards: &[RCard],
) -> Result<(), DatabaseError> {
    info!("Saving all cards for tab with id {}", tab_db_id);

    for card in cards {
        let _ = add_and_assign_card(
            db.deref(),
            card,
            CardAssignmentInfo {
                activity_id: roadmap_activity_id,
                tab_id: tab_db_id,
                section_pos: card.section_position.unwrap(),
                card_pos: card.card_position.unwrap(),
            },
        )
        .await?;
    }

    info!("Finished Saving Cards for tab with id {}", tab_db_id);
    Ok(())
}

pub async fn add_and_assign_cards(
    db: impl Deref<Target = Connection>,
    roadmap: &Roadmap,
    roadmap_activity_id: u32,
    tab_roadmap_to_db_ids: &HashMap<String, u32>,
) -> Result<(), DatabaseError> {
    info!("Saving all cards");

    for k in roadmap.cards.keys() {
        let tab_db_id = tab_roadmap_to_db_ids.get(k).unwrap();
        let cards = roadmap.cards.get(k).unwrap();

        add_and_assign_tab_cards(db.deref(), roadmap_activity_id, *tab_db_id, cards).await?;
    }

    info!("Finished Saving Cards");
    Ok(())
}

pub async fn get_roadmap_activity_cards(
    db: impl Deref<Target = Connection>,
    roadmap_activity_id: u32,
) -> Result<Vec<RCard>, DatabaseError> {
    let mut rows = db
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
            [roadmap_activity_id],
        )
        .await?;

    let mut cards = Vec::new();
    while let Some(row) = rows.next().await? {
        let card = de::from_row::<RCard>(&row)?;
        cards.push(card);
    }

    Ok(cards)
}
