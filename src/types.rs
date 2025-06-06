use std::{collections::HashMap, fmt::Display, hash::Hash};

use actix_web::web;
use libsql::Database;
use log::info;
use serde::{Deserialize, Serialize, de, ser};
use serde_with::with_prefix;
use time::{OffsetDateTime, format_description};

use crate::utils::clean_description;

pub const LOGGED_IN_COOKIE: &str = "logged_in";

// DB Types
#[derive(Serialize, Deserialize)]
pub struct Source {
    pub id: u32,
    pub url: String,
    #[serde(
        deserialize_with = "deserialize_timestamp",
        serialize_with = "serialize_timestamp"
    )]
    pub last_checked: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct Activity {
    pub id: u32,
    pub source_url: String,
    pub post_url: String,
    #[serde(
        deserialize_with = "deserialize_timestamp",
        serialize_with = "serialize_timestamp"
    )]
    pub timestamp: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct RoadmapActivity {
    pub id: u32,
    #[serde(
        deserialize_with = "deserialize_timestamp",
        serialize_with = "serialize_timestamp"
    )]
    pub timestamp: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoadmapWatchedTab {
    pub id: u32,
    #[serde(alias = "tab_roadmap_id")]
    pub tab_id: String,
    #[serde(
        deserialize_with = "deserialize_timestamp",
        serialize_with = "serialize_timestamp"
    )]
    pub timestamp: OffsetDateTime,
}

// JSON Types
#[derive(Deserialize)]
pub struct AddSource {
    pub url: String,
}

#[derive(Deserialize)]
pub struct LoginInfo {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq)]
pub struct RTab {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub db_id: Option<u32>,
}

impl PartialEq for RTab {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id // && self.name == other.name && self.slug == other.slug && self.db_id == other.db_id
    }
}

impl Hash for RTab {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        // self.name.hash(state);
        // self.slug.hash(state);
        // self.db_id.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RSection {
    pub id: String,
    pub name: String,
    #[serde(rename = "portalTabId")]
    pub portal_tab_id: String,
    pub position: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RCardAssignmentInfo {
    pub section_position: u32,
    pub card_position: u32,
    #[serde(rename = "assign_db_id")]
    pub id: u32,
    // #[serde(rename = "tab_id")]
    pub tab_id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RCard {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(alias = "imageUrl")]
    pub image_url: Option<String>,
    pub slug: String,
    // #[serde(flatten, default)]
    // pub assignment: Option<RCardAssignmentInfo>,
    // #[serde(default)]
    pub db_id: Option<u32>,

    #[serde(skip_serializing)]
    pub section_position: Option<u32>,
    #[serde(skip_serializing)]
    pub card_position: Option<u32>,
    #[serde(skip_serializing)]
    pub assign_db_id: Option<u32>,
    #[serde(skip_serializing)]
    pub tab_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RAssigns {
    #[serde(rename = "portalCardId")]
    pub portal_card_id: String,
    #[serde(rename = "portalSectionId")]
    pub portal_section_id: String,
    #[serde(rename = "portalTabId")]
    pub portal_tab_id: String,
    pub position: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebRoadmap {
    #[serde(rename = "portalTabs")]
    pub portal_tabs: Vec<RTab>,
    #[serde(rename = "portalSections")]
    pub portal_sections: Vec<RSection>,
    #[serde(rename = "portalCards")]
    pub portal_cards: Vec<RCard>,
    #[serde(rename = "portalCardAssignments")]
    pub portal_card_assignments: Vec<RAssigns>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Roadmap {
    pub tabs: Vec<RTab>,
    pub cards: HashMap<String, Vec<RCard>>,
}

impl Roadmap {
    pub fn with_data(tabs: Vec<RTab>, cards: HashMap<String, Vec<RCard>>) -> Self {
        Roadmap { tabs, cards }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RChange {
    CardUnchanged {
        tab_id: String,
        card_index: u32,
    },
    CardAdded {
        tab_id: String,
        card_index: u32,
    },
    CardRemoved {
        tab_id: String,
        card_index: u32,
    },
    CardModified {
        tab_id: String,
        previous_card_index: u32,
        current_card_index: u32,
    },
    TabUnchanged {
        tab_index: u32,
    },
    /// Tab is new in Current Roadmaps tab list
    TabAdded {
        tab_index: u32,
    },
    /// Tab is removed in Current Roadmaps tab list
    TabRemoved {
        tab_index: u32,
    },
    /// No cards for watched tab are present on Current Roadmap
    TabCardsNotInCurrent {
        tab_index: u32,
    },
    /// No cards for watched tab are present on Previous Roadmap
    TabCardsNotInPrevious {
        tab_index: u32,
    },
}

with_prefix!(prefix_previous_card "previous_card_");
with_prefix!(prefix_current_card "current_card_");
with_prefix!(prefix_tab "tab_");

#[derive(Debug, Serialize, Deserialize)]
pub struct RDBChange {
    id: u32,
    r#type: String,
    #[serde(flatten, with = "prefix_previous_card")]
    previous_card: Option<RCard>,
    #[serde(flatten, with = "prefix_current_card")]
    current_card: Option<RCard>,
    #[serde(flatten, with = "prefix_tab")]
    tab: Option<RTab>,
}

/// libsql crate's deserializer does not support flattened fields turns out
#[derive(Debug, Serialize, Deserialize)]
pub struct RDBChangeAlt {
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
}

impl RDBChangeAlt {
    pub fn into(self) -> RDBChange {
        let previous_card = if let Some(id) = self.previous_card_id {
            Some(RCard {
                id,
                name: self.previous_card_name.unwrap(),
                description: self.previous_card_description.unwrap(),
                image_url: self.previous_card_image_url,
                slug: self.previous_card_slug.unwrap(),
                db_id: self.previous_card_db_id,
                section_position: None,
                card_position: None,
                assign_db_id: None,
                tab_id: None,
            })
        } else {
            None
        };

        let current_card = if let Some(id) = self.current_card_id {
            Some(RCard {
                id,
                name: self.current_card_name.unwrap(),
                description: self.current_card_description.unwrap(),
                image_url: self.current_card_image_url,
                slug: self.current_card_slug.unwrap(),
                db_id: self.current_card_db_id,
                section_position: None,
                card_position: None,
                assign_db_id: None,
                tab_id: None,
            })
        } else {
            None
        };

        let tab = if let Some(id) = self.tab_id {
            Some(RTab {
                id,
                name: self.tab_name.unwrap(),
                slug: self.tab_slug.unwrap(),
                db_id: self.tab_db_id,
            })
        } else {
            None
        };

        RDBChange {
            id: self.id,
            r#type: self.r#type,
            previous_card,
            current_card,
            tab,
        }
    }

    pub fn clean_descriptions(&mut self) {
        info!("Started cleaning current");
        self.current_card_description = self.current_card_description.take().map(clean_description);
        info!("Started cleaning previous");
        self.previous_card_description =
            self.previous_card_description.take().map(clean_description);
    }
}

// Server Types
#[derive(Serialize)]
pub struct Success {
    pub message: String,
}

#[derive(Serialize)]
pub struct Failure {
    pub message: String,
}

pub struct AppState {
    pub db: Database,
}

pub type AppData = web::Data<AppState>;

// Other types
#[derive(thiserror::Error, Debug)]
pub struct StringError(pub String);

impl Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// Util
fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(de::Error::custom)
}

fn serialize_timestamp<S>(timestamp: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    let s = timestamp
        .format(&format_description::well_known::Rfc2822)
        .map_err(ser::Error::custom)?;
    // let s = serde_json::to_string(timestamp).map_err(ser::Error::custom)?;
    serializer.serialize_str(&s)
}
