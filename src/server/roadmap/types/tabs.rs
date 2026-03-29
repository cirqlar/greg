use std::hash::Hash;

use serde::{Deserialize, Serialize};

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
