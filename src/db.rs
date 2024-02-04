use std::env;

use libsql_client::Client;

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
        .execute(
            "CREATE TABLE IF NOT EXISTS `sources`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `url` TEXT NOT NULL,
                `last_checked` TEXT NOT NULL
            );",
        )
        .await?;
    let _result = client
        .execute(
            "CREATE TABLE IF NOT EXISTS `activities`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `source_id` INTEGER NOT NULL,
                `post_url` TEXT NOT NULL,
                `timestamp` TEXT NOT NULL
            );",
        )
        .await?;

    Ok(())
}
