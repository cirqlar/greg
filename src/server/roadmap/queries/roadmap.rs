use std::collections::HashMap;
use std::ops::Deref;

use libsql::Connection;

use super::activity::get_roadmap_activity;
use super::cards::get_roadmap_activity_cards;
use super::tabs::get_roadmap_activity_tabs;
use crate::roadmap::types::{RCard, Roadmap};
use crate::shared::DatabaseError;

pub async fn get_most_recent_roadmap(
    db: impl Deref<Target = Connection>,
) -> Result<Option<Roadmap>, DatabaseError> {
    let Some(activity) = get_roadmap_activity(db.deref(), 1, 0).await?.pop() else {
        return Ok(None);
    };

    // Get Tabs
    let tabs = get_roadmap_activity_tabs(db.deref(), activity.id).await?;

    // Get Cards
    let cards = get_roadmap_activity_cards(db.deref(), activity.id).await?;

    let mut cards_map: HashMap<String, Vec<RCard>> = HashMap::new();

    for card in cards {
        let tab_db_id = card.db_id.expect("came from db, should have tab id");
        let tab_roadmap_id = tabs
            .iter()
            .find(|tab| *tab.db_id.as_ref().unwrap() == tab_db_id)
            .unwrap()
            .id
            .clone();
        let tab_cards = cards_map.entry(tab_roadmap_id).or_default();
        tab_cards.push(card);
    }

    cards_map
        .values_mut()
        .for_each(|cards| cards.sort_by_key(|card| card.id.clone()));

    Ok(Some(Roadmap::with_data(tabs, cards_map)))
}
