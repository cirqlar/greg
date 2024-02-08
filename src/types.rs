use libsql_client::{Client, Row};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::Mutex;

pub const LOGGED_IN_COOKIE: &str = "logged_in";

// DB Types

pub trait FromRow: std::marker::Sized {
    fn from_row(row: Row) -> anyhow::Result<Self>;
}

#[derive(Serialize, Deserialize)]
pub struct Source {
    pub id: i32,
    pub url: String,
    pub last_checked: OffsetDateTime,
}

impl FromRow for Source {
    fn from_row(row: Row) -> anyhow::Result<Source> {
        let id: i32 = row.try_column("id")?;
        let url: &str = row.try_column("url")?;
        let last_checked_string: &str = row.try_column("last_checked")?;
        let last_checked: OffsetDateTime = serde_json::from_str(last_checked_string)?;

        Ok(Source {
            id,
            url: url.into(),
            last_checked,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Activity {
    pub id: i32,
    pub source_id: i32,
    pub post_url: String,
    pub timestamp: OffsetDateTime,
}

impl FromRow for Activity {
    fn from_row(row: Row) -> anyhow::Result<Activity> {
        let id: i32 = row.try_column("id")?;
        let source_id: i32 = row.try_column("source_id")?;
        let post_url: &str = row.try_column("post_url")?;
        let timestamp_string: &str = row.try_column("timestamp")?;
        let timestamp: OffsetDateTime = serde_json::from_str(timestamp_string)?;

        Ok(Activity {
            id,
            source_id,
            post_url: post_url.into(),
            timestamp,
        })
    }
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
    pub db_handle: Mutex<Client>,
}
