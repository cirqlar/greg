use std::collections::HashMap;

use libsql::Transaction;
use log::info;
use time::OffsetDateTime;

use super::db::{roadmap, tabs};
use crate::server::roadmap::types::Roadmap;
use crate::server::shared::DatabaseError;

pub async fn save_new_roadmap(db: &Transaction, roadmap: Roadmap) -> Result<(), DatabaseError> {
    let start_time = OffsetDateTime::now_utc();
    info!("Started saving new roadmap at {start_time}");

    let roadmap_id = roadmap::new_activity_tx(db).await?;
    let road_end = OffsetDateTime::now_utc();
    info!(
        "Finished saving tabs at {} took {}",
        road_end,
        road_end - start_time
    );

    let mut tab_ids: HashMap<String, u32> = HashMap::new();
    for tab in roadmap.tabs.iter() {
        let tab_id = tabs::save_tab_and_assignment(db, tab, roadmap_id).await?;

        tab_ids.insert(tab.id.clone(), tab_id);
    }
    let tab_end = OffsetDateTime::now_utc();
    info!(
        "Finished saving tabs at {} took {}",
        tab_end,
        tab_end - road_end
    );

    roadmap::save_all_cards_sync_tx(db, &roadmap, roadmap_id, &tab_ids).await?;

    let end_time = OffsetDateTime::now_utc();
    info!(
        "Finished saving cards at {} took {}",
        end_time,
        end_time - start_time
    );

    info!("Saving new roadmap took {}", end_time - start_time);

    Ok(())
}
