use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{RCard, RSection, RTab};

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
