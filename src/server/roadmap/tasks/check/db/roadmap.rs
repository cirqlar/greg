use std::collections::HashMap;

use libsql::Transaction;
use log::info;
use time::OffsetDateTime;

use super::cards::{PartAssignInfo, save_card_and_assignment};
use crate::db::tables::R_ACTIVITIES_T;
use crate::roadmap::types::Roadmap;
use crate::shared::DatabaseError;

pub async fn save_all_cards_sync_tx(
    db: &Transaction,
    roadmap: &Roadmap,
    roadmap_id: u32,
    tab_ids: &HashMap<String, u32>,
) -> Result<(), DatabaseError> {
    info!("Saving all cards");

    for k in roadmap.cards.keys() {
        for card in roadmap.cards.get(k).unwrap() {
            let _ = save_card_and_assignment(
                db,
                card,
                PartAssignInfo {
                    activity_id: roadmap_id,
                    tab_id: *tab_ids.get(k).unwrap(),
                    section_pos: card.section_position.unwrap(),
                    card_pos: card.card_position.unwrap(),
                },
            )
            .await?;
        }
    }
    info!("Finished Saving Cards");
    Ok(())
}

pub async fn save_all_tab_cards_sync_tx(
    db: &Transaction,
    roadmap: &Roadmap,
    roadmap_id: u32,
    tab_ids: &HashMap<String, u32>,
    tab_index: usize,
) -> Result<(), DatabaseError> {
    let tab = &roadmap.tabs[tab_index];
    let tab_id = tab_ids.get(&tab.id).unwrap();

    info!("Saving all cards for tab {}", tab.name);

    let cards = roadmap.cards.get(&tab.id).unwrap();

    for card in cards {
        let _ = save_card_and_assignment(
            db,
            card,
            PartAssignInfo {
                activity_id: roadmap_id,
                tab_id: *tab_id,
                section_pos: card.section_position.unwrap(),
                card_pos: card.card_position.unwrap(),
            },
        )
        .await?;
    }

    info!("Finished Saving Cards for tab {}", tab.name);
    Ok(())
}

pub async fn new_activity_tx(db: &Transaction) -> Result<u32, DatabaseError> {
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
