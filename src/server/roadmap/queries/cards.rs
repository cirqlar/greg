use std::collections::HashMap;
use std::ops::Deref;

use libsql::Connection;
use log::info;
use time::OffsetDateTime;

use crate::db::tables::{R_CARD_ASSIGNS_T, R_CARDS_T};
use crate::roadmap::types::{CardAssignmentInfo, RCard, Roadmap};
use crate::shared::DatabaseError;

async fn save_card(
    db: impl Deref<Target = Connection>,
    card: &RCard,
) -> Result<u32, DatabaseError> {
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

pub async fn save_card_assignment(
    db: impl Deref<Target = Connection>,
    card_id: u32,
    assign_info: CardAssignmentInfo,
) -> Result<(), DatabaseError> {
    let _result = db
        .execute(
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
        .await?;

    Ok(())
}

pub async fn save_card_and_assignment(
    db: impl Deref<Target = Connection>,
    card: &RCard,
    assign_info: CardAssignmentInfo,
) -> Result<u32, DatabaseError> {
    let card_id = save_card(db.deref(), card).await?;
    save_card_assignment(db, card_id, assign_info).await?;
    Ok(card_id)
}

pub async fn save_all_cards_for_tab(
    db: impl Deref<Target = Connection>,
    roadmap_id: u32,
    tab_id: u32,
    cards: &[RCard],
) -> Result<(), DatabaseError> {
    info!("Saving all cards for tab with id {}", tab_id);

    for card in cards {
        let _ = save_card_and_assignment(
            db.deref(),
            card,
            CardAssignmentInfo {
                activity_id: roadmap_id,
                tab_id,
                section_pos: card.section_position.unwrap(),
                card_pos: card.card_position.unwrap(),
            },
        )
        .await?;
    }

    info!("Finished Saving Cards for tab with id {}", tab_id);
    Ok(())
}

pub async fn save_all_cards(
    db: impl Deref<Target = Connection>,
    roadmap: &Roadmap,
    roadmap_id: u32,
    tab_ids: &HashMap<String, u32>,
) -> Result<(), DatabaseError> {
    info!("Saving all cards");

    for k in roadmap.cards.keys() {
        let tab_id = tab_ids.get(k).unwrap();
        let cards = roadmap.cards.get(k).unwrap();

        save_all_cards_for_tab(db.deref(), roadmap_id, *tab_id, cards).await?;
    }

    info!("Finished Saving Cards");
    Ok(())
}
