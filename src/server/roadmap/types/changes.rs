// use log::info;
use serde::{Deserialize, Serialize};

// use crate::roadmap::utils::clean_description;

#[derive(Debug, Serialize, Deserialize)]
pub enum CardChange {
    Unchanged {
        tab_id: String,
        card_index: u32,
    },
    Added {
        tab_id: String,
        card_index: u32,
    },
    Removed {
        tab_id: String,
        card_index: u32,
    },
    Modified {
        tab_id: String,
        previous_card_index: u32,
        current_card_index: u32,
    },
}

impl CardChange {
    pub fn as_str(&self) -> &'static str {
        match self {
            CardChange::Unchanged { .. } => "card_unchanged",
            CardChange::Added { .. } => "card_added",
            CardChange::Removed { .. } => "card_removed",
            CardChange::Modified { .. } => "card_modified",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TabChange {
    Unchanged {
        tab_index: u32,
    },
    /// Tab is new in Current Roadmaps tab list
    Added {
        tab_index: u32,
    },
    /// Tab is removed in Current Roadmaps tab list
    Removed {
        tab_index: u32,
    },
}

impl TabChange {
    pub fn as_str(&self) -> &'static str {
        match self {
            TabChange::Unchanged { .. } => "tab_unchanged",
            TabChange::Added { .. } => "tab_added",
            TabChange::Removed { .. } => "tab_removed",
        }
    }
}

/// Regarding all the cards in a tab
#[derive(Debug, Serialize, Deserialize)]
pub enum TabCardsChange {
    /// No cards for watched tab are present on Current Roadmap
    NotInCurrent { tab_index: u32 },
    /// No cards for watched tab are present on Previous Roadmap
    NotInPrevious { tab_index: u32 },
}

impl TabCardsChange {
    pub fn as_str(&self) -> &'static str {
        match self {
            TabCardsChange::NotInCurrent { .. } => "tab_cards_removed",
            TabCardsChange::NotInPrevious { .. } => "tab_cards_added",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RChange {
    Tab(TabChange),
    Card(CardChange),
    TabCards(TabCardsChange),
}

impl RChange {
    pub fn as_str(&self) -> &'static str {
        match &self {
            RChange::Tab(tab_change) => tab_change.as_str(),
            RChange::Card(card_change) => card_change.as_str(),
            RChange::TabCards(tab_cards_change) => tab_cards_change.as_str(),
        }
    }
}

impl From<CardChange> for RChange {
    fn from(value: CardChange) -> Self {
        RChange::Card(value)
    }
}

impl From<TabChange> for RChange {
    fn from(value: TabChange) -> Self {
        RChange::Tab(value)
    }
}

impl From<TabCardsChange> for RChange {
    fn from(value: TabCardsChange) -> Self {
        RChange::TabCards(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RDBChange {
    id: u32,
    r#type: String,

    pub previous_card_id: Option<String>,
    pub previous_card_name: Option<String>,
    pub previous_card_description: Option<String>,
    pub previous_card_image_url: Option<String>,
    pub previous_card_slug: Option<String>,
    pub previous_card_db_id: Option<u32>,

    pub current_card_id: Option<String>,
    pub current_card_name: Option<String>,
    pub current_card_description: Option<String>,
    pub current_card_image_url: Option<String>,
    pub current_card_slug: Option<String>,
    pub current_card_db_id: Option<u32>,

    pub tab_id: Option<String>,
    pub tab_name: Option<String>,
    pub tab_slug: Option<String>,
    pub tab_db_id: Option<u32>,

    pub card_tab_name: Option<String>,
}

// Was never used?
// impl RDBChange {
//     pub fn clean_descriptions(&mut self) {
//         info!("Started cleaning current");
//         self.current_card_description = self.current_card_description.take().map(clean_description);
//         info!("Started cleaning previous");
//         self.previous_card_description =
//             self.previous_card_description.take().map(clean_description);
//     }
// }
