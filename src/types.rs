use actix_web::web;
use libsql_client::{ResultSet, Row, Statement};
use serde::{Deserialize, Serialize};
use time::{format_description, OffsetDateTime};
use tokio::sync::mpsc::{Receiver, Sender};

pub const LOGGED_IN_COOKIE: &str = "logged_in";

// DB Types

pub trait FromRow: std::marker::Sized {
    fn from_row(row: Row) -> anyhow::Result<Self>;
}

#[derive(Serialize, Deserialize)]
pub struct Source {
    pub id: i32,
    pub url: String,
    pub last_checked: String,
}

impl FromRow for Source {
    fn from_row(row: Row) -> anyhow::Result<Source> {
        let id: i32 = row.try_column("id")?;
        let url: &str = row.try_column("url")?;
        let last_checked_string: &str = row.try_column("last_checked")?;
        let last_checked: OffsetDateTime = serde_json::from_str(last_checked_string)?;
        let last_checked = last_checked
            .format(&format_description::well_known::Rfc2822)
            .unwrap();

        Ok(Source {
            id,
            url: url.into(),
            last_checked,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct ISource {
    pub id: i32,
    pub url: String,
    pub last_checked: OffsetDateTime,
}

impl FromRow for ISource {
    fn from_row(row: Row) -> anyhow::Result<ISource> {
        let id: i32 = row.try_column("id")?;
        let url: &str = row.try_column("url")?;
        let last_checked_string: &str = row.try_column("last_checked")?;
        let last_checked: OffsetDateTime = serde_json::from_str(last_checked_string)?;

        Ok(ISource {
            id,
            url: url.into(),
            last_checked,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Activity {
    pub id: i32,
    pub source_url: String,
    pub post_url: String,
    pub timestamp: String,
}

impl FromRow for Activity {
    fn from_row(row: Row) -> anyhow::Result<Activity> {
        let id: i32 = row.try_column("id")?;
        let source_url: &str = row.try_column("url")?;
        let post_url: &str = row.try_column("post_url")?;
        let timestamp_string: &str = row.try_column("timestamp")?;
        let timestamp: OffsetDateTime = serde_json::from_str(timestamp_string)?;
        let timestamp = timestamp
            .format(&format_description::well_known::Rfc2822)
            .unwrap();

        Ok(Activity {
            id,
            source_url: source_url.into(),
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

pub type DbReturnSender = Sender<anyhow::Result<ResultSet>>;
pub type DbReturnReciever = Receiver<anyhow::Result<ResultSet>>;
pub type DbMesssage = (Statement, DbReturnSender);
pub type DbSender = Sender<DbMesssage>;
pub type DbReceiver = Receiver<DbMesssage>;

pub struct AppState {
    pub db_channel: DbSender,
}

pub type AppData = web::Data<AppState>;
