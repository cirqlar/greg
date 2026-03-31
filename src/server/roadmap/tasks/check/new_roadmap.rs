use std::collections::HashMap;
use std::ops::Deref;

use libsql::Connection;
use log::info;
use time::OffsetDateTime;

use crate::roadmap::queries::{activity, cards, tabs};
use crate::roadmap::types::Roadmap;
use crate::shared::DatabaseError;

pub async fn save_new_roadmap(
    db: impl Deref<Target = Connection>,
    roadmap: Roadmap,
) -> Result<(), DatabaseError> {
    let start_time = OffsetDateTime::now_utc();
    info!("Started saving new roadmap at {start_time}");

    let roadmap_id = activity::add_activity(db.deref()).await?;
    let road_end = OffsetDateTime::now_utc();
    info!(
        "Finished saving tabs at {} took {}",
        road_end,
        road_end - start_time
    );

    let mut tab_ids: HashMap<String, u32> = HashMap::new();
    for tab in roadmap.tabs.iter() {
        let tab_id = tabs::save_tab_and_assignment(db.deref(), tab, roadmap_id).await?;

        tab_ids.insert(tab.id.clone(), tab_id);
    }
    let tab_end = OffsetDateTime::now_utc();
    info!(
        "Finished saving tabs at {} took {}",
        tab_end,
        tab_end - road_end
    );

    cards::add_and_assign_cards(db.deref(), &roadmap, roadmap_id, &tab_ids).await?;

    let end_time = OffsetDateTime::now_utc();
    info!(
        "Finished saving cards at {} took {}",
        end_time,
        end_time - start_time
    );

    info!("Saving new roadmap took {}", end_time - start_time);

    Ok(())
}
