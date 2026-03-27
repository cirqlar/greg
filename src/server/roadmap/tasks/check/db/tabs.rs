use libsql::Transaction;
use time::OffsetDateTime;

use crate::server::db::tables::{R_TAB_ASSIGNS_T, R_TABS_T};
use crate::server::roadmap::types::RTab;
use crate::server::shared::DatabaseError;

pub async fn save_tab_tx(db: &Transaction, tab: &RTab) -> Result<u32, DatabaseError> {
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

pub async fn save_tab_assignment_tx(
    db: &Transaction,
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
    db: &Transaction,
    tab: &RTab,
    activity_id: u32,
) -> Result<u32, DatabaseError> {
    let tab_id = save_tab_tx(db, tab).await?;

    save_tab_assignment_tx(db, activity_id, tab_id).await?;

    Ok(tab_id)
}
