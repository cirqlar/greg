use libsql_client::Client;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::Mutex;

pub const LOGGED_IN: &str = "logged_in";
pub const LOGGED_IN_VALUE: &str = "currently";

// DB Types

#[derive(Serialize, Deserialize)]
pub struct Source {
    pub id: i32,
    pub url: String,
    pub last_checked: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct Activity {
    pub id: i32,
    pub source_id: i32,
    pub post_url: String,
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
