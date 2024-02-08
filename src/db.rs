use std::env;

use libsql_client::Client;
use serde_json::Value;

pub trait ToSerdeJsonValue {
    fn convert(self) -> Value;
}

impl ToSerdeJsonValue for libsql_client::Value {
    fn convert(self) -> Value {
        match self {
            libsql_client::Value::Null => Value::Null,
            libsql_client::Value::Integer { value } => value.into(),
            libsql_client::Value::Float { value } => value.into(),
            libsql_client::Value::Text { value } => value.into(),
            libsql_client::Value::Blob { value } => value.into_iter().collect(),
        }
    }
}

pub async fn establish_connection() -> Client {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let auth_key = env::var("DATABASE_AUTH_KEY").expect("DATABASE_AUTH_KEY must be set");

    libsql_client::Client::from_config(libsql_client::Config {
        url: url::Url::parse(&database_url).unwrap(),
        auth_token: Some(auth_key),
    })
    .await
    .unwrap()
}

pub async fn migrate_db(client: &Client) -> anyhow::Result<()> {
    let _result = client
        .batch([
            "CREATE TABLE IF NOT EXISTS `sources`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `url` TEXT NOT NULL,
                `last_checked` TEXT NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS `activities`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `source_id` INTEGER NOT NULL,
                `post_url` TEXT NOT NULL,
                `timestamp` TEXT NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS `logins`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `timestamp` TEXT NOT NULL,
                `key` TEXT NOT NULL UNIQUE
            );",
            "CREATE INDEX IF NOT EXISTS idx_key ON logins (key)",
        ])
        .await?;

    Ok(())
}
