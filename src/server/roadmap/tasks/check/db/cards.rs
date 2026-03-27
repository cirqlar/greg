use libsql::Transaction;
use time::OffsetDateTime;

use crate::server::db::tables::{R_CARD_ASSIGNS_T, R_CARDS_T};
use crate::server::roadmap::types::RCard;
use crate::server::shared::DatabaseError;

pub async fn save_card_tx(db: &Transaction, card: &RCard) -> Result<u32, DatabaseError> {
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

pub struct AssignInfo {
    pub activity_id: u32,
    pub tab_id: u32,
    pub card_id: u32,
    pub section_pos: u32,
    pub card_pos: u32,
}

/// Save card assignment
pub async fn save_card_assignment_tx(
    db: &Transaction,
    assign_info: AssignInfo,
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
                assign_info.card_id,
                assign_info.section_pos,
                assign_info.card_pos,
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    Ok(())
}

pub struct PartAssignInfo {
    pub activity_id: u32,
    pub tab_id: u32,
    pub section_pos: u32,
    pub card_pos: u32,
}

impl PartAssignInfo {
    fn add_card_id(self, card_id: u32) -> AssignInfo {
        let PartAssignInfo {
            activity_id,
            tab_id,
            section_pos,
            card_pos,
        } = self;
        AssignInfo {
            activity_id,
            tab_id,
            card_id,
            section_pos,
            card_pos,
        }
    }
}

pub async fn save_card_and_assignment(
    db: &Transaction,
    card: &RCard,
    assign_info: PartAssignInfo,
) -> Result<u32, DatabaseError> {
    let card_id = save_card_tx(db, card).await?;
    save_card_assignment_tx(db, assign_info.add_card_id(card_id)).await?;
    Ok(card_id)
}
