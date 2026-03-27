use log::info;

use crate::roadmap::types::{CardChange, RChange, Roadmap, TabCardsChange, TabChange};

pub fn compare_roadmaps(previous: &Roadmap, current: &Roadmap) -> Vec<RChange> {
    info!("Started roadmap comparison");

    let mut all_changes: Vec<RChange> = Vec::new();

    // Check tabs for changes
    for (index, tab) in previous.tabs.iter().enumerate() {
        if !current.tabs.iter().any(|t| t.id == tab.id) {
            all_changes.push(
                TabChange::Removed {
                    tab_index: index.try_into().unwrap(),
                }
                .into(),
            );
        } else {
            all_changes.push(
                TabChange::Unchanged {
                    tab_index: index.try_into().unwrap(),
                }
                .into(),
            );
        }
    }
    for (index, tab) in current.tabs.iter().enumerate() {
        if !previous.tabs.iter().any(|t| t.id == tab.id) {
            all_changes.push(
                TabChange::Added {
                    tab_index: index.try_into().unwrap(),
                }
                .into(),
            );
        }
    }
    info!("Finished tab comparison");

    // Check previous cards for removals and modifications
    for (k, cards) in previous.cards.iter() {
        if !current.cards.contains_key(k) {
            // The tab was removed one way or another and will be caught by the tab check
            let tab_index = previous
                .tabs
                .iter()
                .enumerate()
                .find_map(|(index, t)| (t.id == *k).then_some(index))
                .unwrap();

            all_changes.push(
                TabCardsChange::NotInCurrent {
                    tab_index: tab_index.try_into().unwrap(),
                }
                .into(),
            );
            continue;
        }

        let other_cards = current.cards.get(k).unwrap();
        cards.iter().enumerate().for_each(|(index, card)| {
            if let Ok(c_index) =
                other_cards.binary_search_by(|other_card| other_card.id.cmp(&card.id))
            {
                let _current = &other_cards[c_index];

                let title = card.name != _current.name;
                let description = card.description != _current.description;
                let image = card.image_url != _current.image_url;

                if title || description || image {
                    all_changes.push(
                        CardChange::Modified {
                            tab_id: k.clone(),
                            previous_card_index: index.try_into().unwrap(),
                            current_card_index: c_index.try_into().unwrap(),
                        }
                        .into(),
                    );
                } else {
                    all_changes.push(
                        CardChange::Unchanged {
                            tab_id: k.clone(),
                            card_index: index.try_into().unwrap(),
                        }
                        .into(),
                    );
                }
            } else {
                all_changes.push(
                    CardChange::Removed {
                        tab_id: k.clone(),
                        card_index: index.try_into().unwrap(),
                    }
                    .into(),
                );
            }
        });
    }

    info!("Finished Previous comparison");

    // Check current cards for additions (including whole tab)
    for (k, cards) in current.cards.iter() {
        if !previous.cards.contains_key(k) {
            let tab_index = current
                .tabs
                .iter()
                .enumerate()
                .find_map(|(index, t)| (t.id == *k).then_some(index))
                .unwrap();

            all_changes.push(
                TabCardsChange::NotInPrevious {
                    tab_index: tab_index.try_into().unwrap(),
                }
                .into(),
            );
            continue;
        }

        let other_cards = previous.cards.get(k).unwrap();
        cards.iter().enumerate().for_each(|(index, card)| {
            if other_cards
                .binary_search_by(|other_card| other_card.id.cmp(&card.id))
                .is_err()
            {
                all_changes.push(
                    CardChange::Added {
                        tab_id: k.clone(),
                        card_index: (index).try_into().unwrap(),
                    }
                    .into(),
                );
            }
        });
    }

    info!("Finished current comparisons");

    info!("Finished comparisons");

    all_changes
}
