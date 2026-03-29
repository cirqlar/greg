use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RSection {
    pub id: String,
    pub name: String,
    #[serde(rename = "portalTabId")]
    pub portal_tab_id: String,
    pub position: u32,
}
