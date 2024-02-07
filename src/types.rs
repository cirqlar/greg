use libsql_client::{Client, Row};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::Mutex;

pub const LOGGED_IN_COOKIE: &str = "logged_in";

// DB Types

pub trait FromRow {
    fn from_row(row: Row) -> Self;
}

#[derive(Serialize, Deserialize)]
pub struct Source {
    pub id: i32,
    pub url: String,
    pub last_checked: OffsetDateTime,
}

impl FromRow for Source {
    fn from_row(row: Row) -> Source {
        let id: i32 = row.try_column("id").unwrap();
        let url: &str = row.try_column("url").unwrap();
        let last_checked_string: &str = row.try_column("last_checked").unwrap();
        let last_checked: OffsetDateTime = serde_json::from_str(last_checked_string).unwrap();

        Source {
            id,
            url: url.into(),
            last_checked,
        }
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
    fn from_row(row: Row) -> Activity {
        let id: i32 = row.try_column("id").unwrap();
        let source_id: i32 = row.try_column("source_id").unwrap();
        let post_url: &str = row.try_column("post_url").unwrap();
        let timestamp_string: &str = row.try_column("timestamp").unwrap();
        let timestamp: OffsetDateTime = serde_json::from_str(timestamp_string).unwrap();

        Activity {
            id,
            source_id,
            post_url: post_url.into(),
            timestamp,
        }
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
    pub error: String,
}

pub struct AppState {
    pub db_handle: Mutex<Client>,
}
