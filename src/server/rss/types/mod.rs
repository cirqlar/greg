use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::server::shared::timestamp::{deserialize_timestamp, serialize_timestamp};

#[derive(Serialize, Deserialize)]
pub(super) struct Source {
    pub id: u32,
    pub url: String,
    #[serde(
        deserialize_with = "deserialize_timestamp",
        serialize_with = "serialize_timestamp"
    )]
    pub last_checked: OffsetDateTime,
    pub enabled: bool,
    pub failed_count: u32,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Activity {
    pub id: u32,
    pub source_url: String,
    pub post_url: String,
    #[serde(
        deserialize_with = "deserialize_timestamp",
        serialize_with = "serialize_timestamp"
    )]
    pub timestamp: OffsetDateTime,
}
