use std::collections::HashMap;
use std::ops::Deref;

use libsql::Connection;
use log::error;
use thiserror::Error;

use crate::roadmap::queries::{cards, changes, tabs};
use crate::roadmap::types::{
    CardAssignmentInfo, CardChange, ChangeInfo, RChange, Roadmap, TabCardsChange, TabChange,
};
use crate::shared::DatabaseError;

pub trait SaveOrNotify {
    fn should_notify(&self) -> bool;

    fn should_save(&self) -> bool {
        self.should_notify()
    }
}

impl SaveOrNotify for TabChange {
    fn should_notify(&self) -> bool {
        match self {
            TabChange::Unchanged { .. } => false,
            TabChange::Added { .. } | TabChange::Removed { .. } => true,
        }
    }
}

impl SaveOrNotify for CardChange {
    fn should_notify(&self) -> bool {
        match self {
            CardChange::Unchanged { .. } => false,
            CardChange::Added { .. } | CardChange::Removed { .. } | CardChange::Modified { .. } => {
                true
            }
        }
    }
}

impl SaveOrNotify for TabCardsChange {
    fn should_notify(&self) -> bool {
        false
    }

    fn should_save(&self) -> bool {
        true
    }
}

impl SaveOrNotify for RChange {
    fn should_notify(&self) -> bool {
        match self {
            RChange::Tab(tab_change) => tab_change.should_notify(),
            RChange::Card(card_change) => card_change.should_notify(),
            RChange::TabCards(tab_cards_change) => tab_cards_change.should_notify(),
        }
    }

    fn should_save(&self) -> bool {
        match self {
            RChange::Tab(tab_change) => tab_change.should_save(),
            RChange::Card(card_change) => card_change.should_save(),
            RChange::TabCards(tab_cards_change) => tab_cards_change.should_save(),
        }
    }
}

#[derive(Debug, Error)]
pub enum SaveChangesError {
    #[error("Failed to save changes")]
    DatabaseError(#[from] DatabaseError),
    #[error("{0}")]
    Other(String),
}

pub async fn handle_tab_changes(
    db: impl Deref<Target = Connection>,
    previous_roadmap: &Roadmap,
    new_roadmap: &Roadmap,
    new_roadmap_id: u32,
    changes: &[RChange],
    tab_ids: &mut HashMap<String, u32>,
) -> Result<(), SaveChangesError> {
    for change in changes {
        match change {
            RChange::Tab(tab_change) => match tab_change {
                TabChange::Unchanged { tab_index } => {
                    let tab_id = previous_roadmap.tabs[*tab_index as usize].db_id.unwrap();

                    tabs::assign_tab(db.deref(), new_roadmap_id, tab_id).await?;
                }
                TabChange::Added { tab_index } => {
                    let tab = &new_roadmap.tabs[*tab_index as usize];

                    let tab_id = tabs::add_and_assign_tab(db.deref(), tab, new_roadmap_id).await?;

                    tab_ids.insert(tab.id.clone(), tab_id);

                    changes::add_change(
                        db.deref(),
                        ChangeInfo {
                            activity_id: new_roadmap_id,
                            change_type: tab_change.as_str(),
                            previous_card_id: None,
                            current_card_id: None,
                            tab_id: Some(tab_id),
                        },
                    )
                    .await?;
                }
                TabChange::Removed { tab_index } => {
                    let tab_id = previous_roadmap.tabs[*tab_index as usize].db_id.unwrap();
                    changes::add_change(
                        db.deref(),
                        ChangeInfo {
                            activity_id: new_roadmap_id,
                            change_type: tab_change.as_str(),
                            previous_card_id: None,
                            current_card_id: None,
                            tab_id: Some(tab_id),
                        },
                    )
                    .await?;
                }
            },
            x => {
                error!("Non tab change passed to handle_tab_changes. change: {x:?}");
                return Err(SaveChangesError::Other(
                    "Non tab change passed to handle_tab_changes".to_string(),
                ));
            }
        }
    }

    Ok(())
}

pub async fn handle_card_changes(
    db: impl Deref<Target = Connection>,
    previous_roadmap: &Roadmap,
    new_roadmap: &Roadmap,
    new_roadmap_id: u32,
    changes: &[RChange],
    tab_ids: &HashMap<String, u32>,
) -> Result<(), SaveChangesError> {
    for change in changes {
        let mut change_info = ChangeInfo {
            activity_id: new_roadmap_id,
            change_type: change.as_str(),
            previous_card_id: None,
            current_card_id: None,
            tab_id: None,
        };

        match change {
            RChange::Card(card_change) => match card_change {
                CardChange::Unchanged { tab_id, card_index } => {
                    let card = &previous_roadmap.cards.get(tab_id).unwrap()[*card_index as usize];

                    cards::assign_card(
                        db.deref(),
                        card.db_id.unwrap(),
                        CardAssignmentInfo {
                            activity_id: new_roadmap_id,
                            tab_id: *tab_ids.get(tab_id).unwrap(),
                            section_pos: card.section_position.unwrap(),
                            card_pos: card.card_position.unwrap(),
                        },
                    )
                    .await?;
                }
                CardChange::Added { tab_id, card_index } => {
                    let card = &new_roadmap.cards.get(tab_id).unwrap()[*card_index as usize];

                    let card_id = cards::add_and_assign_card(
                        db.deref(),
                        card,
                        CardAssignmentInfo {
                            activity_id: new_roadmap_id,
                            tab_id: *tab_ids.get(tab_id).unwrap(),
                            section_pos: card.section_position.unwrap(),
                            card_pos: card.card_position.unwrap(),
                        },
                    )
                    .await?;

                    change_info.current_card_id = Some(card_id);
                }
                CardChange::Removed { tab_id, card_index } => {
                    let card = &previous_roadmap.cards.get(tab_id).unwrap()[*card_index as usize];
                    change_info.previous_card_id = Some(card.db_id.unwrap());
                }
                CardChange::Modified {
                    tab_id,
                    previous_card_index,
                    current_card_index,
                } => {
                    let card =
                        &previous_roadmap.cards.get(tab_id).unwrap()[*previous_card_index as usize];
                    change_info.previous_card_id = Some(card.db_id.unwrap());

                    let card =
                        &new_roadmap.cards.get(tab_id).unwrap()[*current_card_index as usize];
                    let card_id = cards::add_and_assign_card(
                        db.deref(),
                        card,
                        CardAssignmentInfo {
                            activity_id: new_roadmap_id,
                            tab_id: *tab_ids.get(tab_id).unwrap(),
                            section_pos: card.section_position.unwrap(),
                            card_pos: card.card_position.unwrap(),
                        },
                    )
                    .await?;

                    change_info.current_card_id = Some(card_id);
                }
            },
            RChange::TabCards(tab_cards_change) => match tab_cards_change {
                TabCardsChange::NotInCurrent { .. } => continue,
                TabCardsChange::NotInPrevious { tab_index } => {
                    let tab_roadmap_id = &new_roadmap.tabs.get(*tab_index as usize).unwrap().id;
                    let tab_id = tab_ids.get(tab_roadmap_id).unwrap();
                    let cards = new_roadmap.cards.get(tab_roadmap_id).unwrap();

                    cards::add_and_assign_tab_cards(db.deref(), new_roadmap_id, *tab_id, cards)
                        .await?;

                    continue;
                }
            },
            x => {
                error!("Non card/tabcards change passed to handle_card_changes. change: {x:?}");
                return Err(SaveChangesError::Other(
                    "Non card/tabcards change passed to handle_card_changes".to_string(),
                ));
            }
        }

        // Save change
        if change_info.previous_card_id.is_some() || change_info.current_card_id.is_some() {
            changes::add_change(db.deref(), change_info).await?;
        }
    }

    Ok(())
}
