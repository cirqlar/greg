use std::env;

use libsql::{Builder, Connection, Database, OpenFlags};

use crate::types::StringError;

pub async fn get_database() -> Database {
    let use_local = env::var("USE_LOCAL").unwrap_or("false".into());
    if use_local == "false" {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let auth_key = env::var("DATABASE_AUTH_KEY").expect("DATABASE_AUTH_KEY must be set");
        Builder::new_remote(database_url, auth_key)
            .build()
            .await
            .unwrap()
    } else {
        Builder::new_local(env::var("LOCAL_DB_URL").expect("LOCAL_DB_URL must be set"))
            .flags(OpenFlags::default())
            .build()
            .await
            .unwrap()
    }
}

pub const SOURCES_T: &str = "sources";
pub const ACTIVITIES_T: &str = "activities";
pub const LOGINS_T: &str = "logins";
pub const R_ACTIVITIES_T: &str = "roadmap_activities";
pub const R_WATCHED_TABS_T: &str = "roadmap_watched_tabs";
pub const R_CARDS_T: &str = "roadmap_cards";
pub const R_TABS_T: &str = "roadmap_tabs";
pub const R_CARD_ASSIGNS_T: &str = "roadmap_card_assignments";
pub const R_TAB_ASSIGNS_T: &str = "roadmap_tab_assignments";
pub const R_CHANGES_T: &str = "roadmap_changes";

pub const VERSION_T: &str = "db_version";

async fn v1(conn: Connection) -> anyhow::Result<()> {
    #[rustfmt::skip]
    let stmnts = [
        format!(
            "CREATE TABLE IF NOT EXISTS `{SOURCES_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `url` TEXT NOT NULL UNIQUE,
                `last_checked` TEXT NOT NULL
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS `{ACTIVITIES_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `source_id` INTEGER NOT NULL,
                `post_url` TEXT NOT NULL,
                `timestamp` TEXT NOT NULL
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS `{LOGINS_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `timestamp` TEXT NOT NULL,
                `key` TEXT NOT NULL UNIQUE
            )"
        ),
        format!("CREATE INDEX IF NOT EXISTS idx_key ON {LOGINS_T} (key)"),

        // Added 26/05/025
        format!(
            "CREATE TABLE IF NOT EXISTS `{R_ACTIVITIES_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `timestamp` TEXT NOT NULL
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS `{R_WATCHED_TABS_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `tab_roadmap_id` TEXT NOT NULL UNIQUE,
                `timestamp` TEXT NOT NULL
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS `{R_CARDS_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `roadmap_id` TEXT NOT NULL,
                `name` TEXT NOT NULL,
                `description` TEXT NOT NULL,
                `image_url` TEXT,
                `slug` TEXT NOT NULL,
                `timestamp` TEXT NOT NULL
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS `{R_TABS_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `roadmap_id` TEXT NOT NULL UNIQUE,
                `name` TEXT NOT NULL,
                `slug` TEXT NOT NULL,
                `timestamp` TEXT NOT NULL
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS `{R_CARD_ASSIGNS_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `activity_id` INTEGER NOT NULL,
                `tab_id` INTEGER NOT NULL,
                `card_id` INTEGER NOT NULL,
                `section_position` INTEGER NOT NULL,
                `card_position` INTEGER NOT NULL,
                `timestamp` TEXT NOT NULL
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS `{R_TAB_ASSIGNS_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `activity_id` INTEGER NOT NULL,
                `tab_id` INTEGER NOT NULL,
                `timestamp` TEXT NOT NULL
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS `{R_CHANGES_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `type` TEXT NOT NULL,
                `activity_id` INTEGER NOT NULL,
                `previous_card_id` INTEGER,
                `current_card_id` INTEGER,
                `tab_id` INTEGER,
                `timestamp` TEXT NOT NULL
            )"
        ),

        // Added 18/07/025
        format!(
            "CREATE TABLE IF NOT EXISTS `{VERSION_T}`(
                `id` INTEGER NOT NULL PRIMARY KEY,
                `version_number` INTEGER NOT NULL
            )"
        ),
    ];

    let mut _res = conn
        .execute_transactional_batch(&stmnts.join(";\n"))
        .await?;

    // I don't think this is a thing I need to worry about
    // let mut counter = 0;
    // while let Some(r) = res.next_stmt_row() {
    //     if let Some(mut r) = r {
    //         let _ = r.next().await?;
    //     } else {
    //         log::error!("[Migrate db] Statement {counter} didn't execute");
    //     }
    //     counter += 1;
    // }

    Ok(())
}

async fn get_version_number(conn: Connection) -> anyhow::Result<u32> {
    let mut res = conn
        .query(&format!("SELECT * FROM {VERSION_T} WHERE id = ?1"), [1])
        .await?;

    debug_assert_eq!(
        "version_number",
        res.column_name(1)
            .ok_or_else(|| StringError("Missing second column in version table".into()))?
    );

    let Some(row) = res.next().await? else {
        return Ok(1);
    };

    Ok(row.get(1)?)
}

async fn v2(conn: Connection) -> anyhow::Result<()> {
    #[rustfmt::skip]
    let stmnts = [
        format!("INSERT INTO {VERSION_T} (version_number) VALUES (2)"),
        format!("
            ALTER TABLE {SOURCES_T}
            ADD enabled INTEGER NOT NULL DEFAULT 1
        "),
        format!("
            ALTER TABLE {SOURCES_T}
            ADD failed_count INTEGER NOT NULL DEFAULT 0
        "),
    ];

    let mut _res = conn
        .execute_transactional_batch(&stmnts.join(";\n"))
        .await?;

    Ok(())
}

pub async fn migrate_db(conn: Connection) -> anyhow::Result<()> {
    v1(conn.clone()).await?;

    let version_number = get_version_number(conn.clone()).await?;

    if version_number < 2 {
        v2(conn.clone()).await?;
    }

    Ok(())
}
