use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::server::shared::timestamp::{deserialize_timestamp, serialize_timestamp};

#[derive(Serialize, Deserialize)]
pub struct RoadmapActivity {
    pub id: u32,
    pub change_count: Option<u32>,
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
