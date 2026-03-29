use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RCard {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(alias = "imageUrl")]
    pub image_url: Option<String>,
    pub slug: String,
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

pub struct CardAssignmentInfo {
    pub activity_id: u32,
    pub tab_id: u32,
    pub section_pos: u32,
    pub card_pos: u32,
}
