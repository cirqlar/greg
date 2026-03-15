use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Query {
    #[serde(default)]
    pub demo: bool,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default)]
    pub demo: bool,
    pub count: Option<u32>,
    pub skip: Option<u32>,
}
