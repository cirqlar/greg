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
        let tab_db_id = card.tab_id.expect("came from db, should have tab id");
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::super::{
        activity::add_activity, cards::add_and_assign_card, tabs::add_and_assign_tab,
    };
    use super::super::{cards::tests::make_card, tabs::tests::make_tab};
    use super::*;
    use crate::db::tests::empty_db;
    use crate::roadmap::types::CardAssignmentInfo;

    #[rstest]
    #[tokio::test]
    async fn can_get_empty_recent_roadmap(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let rmap = get_most_recent_roadmap(&empty_db).await?;

        assert!(rmap.is_none());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_most_recent_roadmap(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let roadmap_id = add_activity(&empty_db).await?;
        let tab_names = ["fake_tab_1", "fake_tab_2"];
        let tabs = tab_names.iter().map(|n| make_tab(n)).collect::<Vec<_>>();
        let mut tab_ids = vec![];

        for tab in &tabs {
            tab_ids.push(add_and_assign_tab(&empty_db, tab, roadmap_id).await?);
        }

        let card_names = ["fake_card_1", "fake_card_2", "fake_card_3", "fake_card_4"];
        let cards = card_names
            .iter()
            .enumerate()
            .map(|(p, n)| make_card(n, p as u32))
            .collect::<Vec<_>>();

        let mut card_ids = vec![];

        for (pos, card) in cards.iter().enumerate() {
            card_ids.push(
                add_and_assign_card(
                    &empty_db,
                    card,
                    CardAssignmentInfo {
                        activity_id: roadmap_id,
                        tab_id: tab_ids[pos % tab_ids.len()],
                        section_pos: card.card_position.expect("has"),
                        card_pos: card.card_position.expect("has"),
                    },
                )
                .await?,
            );
        }

        let rmap = get_most_recent_roadmap(&empty_db)
            .await?
            .expect("Can get most recent roadmap");

        assert_eq!(rmap.tabs.len(), tabs.len());

        for tab in &rmap.tabs {
            assert!(tab_ids.contains(&tab.db_id.expect("tab from db has db_id")));
            assert!(tab_names.contains(&tab.id.as_str()));
        }

        // TODO: check tabs are in the right tab
        assert_eq!(
            rmap.cards.values().map(|v| v.len()).sum::<usize>(),
            cards.len()
        );
        assert!(
            rmap.cards
                .values()
                .all(|v| v.is_sorted_by_key(|c| c.id.clone()))
        );

        for card in rmap.cards.values().flatten() {
            assert!(card_ids.contains(&card.db_id.expect("card from db has db_id")));
            assert!(card_names.contains(&card.id.as_str()))
        }

        Ok(())
    }
}
