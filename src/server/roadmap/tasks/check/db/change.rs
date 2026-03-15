use libsql::Transaction;
use time::OffsetDateTime;

use crate::db::R_CHANGES_T;
use crate::server::shared::DatabaseError;

pub struct ChangeInfo {
    pub activity_id: u32,
    pub change_type: &'static str,
    pub previous_card_id: Option<u32>,
    pub current_card_id: Option<u32>,
    pub tab_id: Option<u32>,
}

/// Save change
pub async fn save_change_tx(
    db: &Transaction,
    change_info: ChangeInfo,
) -> Result<(), DatabaseError> {
    let _result = db
        .execute(
            &format!(
                "INSERT INTO {R_CHANGES_T} 
                    (type, activity_id, previous_card_id, current_card_id, tab_id, timestamp) 
                VALUES 
                    (?1,?2,?3,?4,?5,?6)
                "
            ),
            (
                change_info.change_type,
                change_info.activity_id,
                change_info.previous_card_id,
                change_info.current_card_id,
                change_info.tab_id,
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    Ok(())
}
