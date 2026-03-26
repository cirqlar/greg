use std::sync::Arc;

use libsql::{Connection, Transaction, de, params};
use log::info;
use thiserror::Error;
use time::OffsetDateTime;

use super::tables::MIGRATIONS_T;
use super::types::{DbMigration, Migration};
use crate::server::shared::DatabaseError;

// migration_import_start
mod m_000001774527605_add_sources;
mod m_000001774528994_add_activities;
mod m_000001774529002_add_logins;
mod m_000001774529132_add_roadmap_activities;
mod m_000001774529138_add_roadmap_watched_tabs;
mod m_000001774529754_add_roadmap_cards;
mod m_000001774529762_add_roadmap_tabs;
mod m_000001774529775_add_roadmap_card_assigns;
mod m_000001774529780_add_roadmap_tab_assigns;
mod m_000001774529792_add_roadmap_changes;
mod m_000001774529798_handle_version;
// migration_import_end

fn get_migrations() -> Vec<Migration> {
    vec![
        // migration_list_start
        Migration {
            name: m_000001774527605_add_sources::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| Box::pin(m_000001774527605_add_sources::run(db))),
        },
        Migration {
            name: m_000001774528994_add_activities::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| {
                Box::pin(m_000001774528994_add_activities::run(db))
            }),
        },
        Migration {
            name: m_000001774529002_add_logins::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| Box::pin(m_000001774529002_add_logins::run(db))),
        },
        Migration {
            name: m_000001774529132_add_roadmap_activities::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| {
                Box::pin(m_000001774529132_add_roadmap_activities::run(db))
            }),
        },
        Migration {
            name: m_000001774529138_add_roadmap_watched_tabs::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| {
                Box::pin(m_000001774529138_add_roadmap_watched_tabs::run(db))
            }),
        },
        Migration {
            name: m_000001774529754_add_roadmap_cards::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| {
                Box::pin(m_000001774529754_add_roadmap_cards::run(db))
            }),
        },
        Migration {
            name: m_000001774529762_add_roadmap_tabs::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| {
                Box::pin(m_000001774529762_add_roadmap_tabs::run(db))
            }),
        },
        Migration {
            name: m_000001774529775_add_roadmap_card_assigns::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| {
                Box::pin(m_000001774529775_add_roadmap_card_assigns::run(db))
            }),
        },
        Migration {
            name: m_000001774529780_add_roadmap_tab_assigns::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| {
                Box::pin(m_000001774529780_add_roadmap_tab_assigns::run(db))
            }),
        },
        Migration {
            name: m_000001774529792_add_roadmap_changes::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| {
                Box::pin(m_000001774529792_add_roadmap_changes::run(db))
            }),
        },
        Migration {
            name: m_000001774529798_handle_version::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| {
                Box::pin(m_000001774529798_handle_version::run(db))
            }),
        },
        // migration_list_end
    ]
}

async fn get_existing_migrations(db: Arc<Transaction>) -> Result<Vec<DbMigration>, DatabaseError> {
    let _ = db
        .execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS `{MIGRATIONS_T}`(
                    `id` INTEGER NOT NULL PRIMARY KEY,
                    `name` TEXT NOT NULL UNIQUE,
                    `timestamp` TEXT NOT NULL
                )",
            ),
            params!(),
        )
        .await?;

    let mut result = db
        .query(&format!("SELECT * FROM {MIGRATIONS_T}"), params!())
        .await?;

    let mut migrations = Vec::new();
    while let Some(r) = result.next().await? {
        migrations.push(de::from_row::<DbMigration>(&r)?);
    }

    Ok(migrations)
}

async fn save_migration(db: Arc<Transaction>, name: &str) -> Result<(), DatabaseError> {
    let _ = db
        .execute(
            &format!("INSERT INTO {MIGRATIONS_T} (name, timestamp) VALUES (?1, ?2)"),
            [
                name.to_owned(),
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ],
        )
        .await?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum ApplyMigrationError {
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error("{0}")]
    Other(String),
}

async fn internal_apply_migrations(
    db: Connection,
    migrations: &[Migration],
) -> Result<(), ApplyMigrationError> {
    let tx = Arc::from(db.transaction().await.map_err(DatabaseError::from)?);

    let applied_migrations = get_existing_migrations(tx.clone()).await?;

    for migration in migrations {
        // Skip already run migrations
        if let Some(am) = applied_migrations
            .iter()
            .find(|am| am.name == migration.name)
        {
            info!(
                "Skipping migration {} because it was already applied on {}",
                am.name, am.timestamp
            );

            continue;
        }

        // Run migration
        (*migration.run)(tx.clone()).await?;

        // Save successful migrations
        save_migration(tx.clone(), migration.name).await?;
    }

    let Ok(tx) = Arc::try_unwrap(tx) else {
        return Err(ApplyMigrationError::Other(
            "Could not complete migration because of stray references".into(),
        ));
    };

    tx.commit().await.map_err(DatabaseError::from)?;

    Ok(())
}

pub async fn apply_migrations(db: Connection) -> Result<(), ApplyMigrationError> {
    let migrations = get_migrations();

    internal_apply_migrations(db, &migrations).await
}

#[cfg(test)]
mod tests {
    use libsql::{Builder, Database};
    use rstest::fixture;
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn all_migrations_are_unique(migrations: Vec<Migration>) {
        let mut found = Vec::with_capacity(migrations.len());

        for migration in migrations {
            assert!(!found.contains(&migration.name));

            found.push(migration.name);
        }
    }

    #[rstest]
    #[tokio::test]
    async fn all_apply_on_empty_db(
        #[future(awt)] empty_db: Database,
        migrations: Vec<Migration>,
    ) -> Result<(), ApplyMigrationError> {
        internal_apply_migrations(empty_db.connect().expect("Can connect"), &migrations).await
    }

    #[rstest]
    #[tokio::test]
    async fn resuming_from_any_position_works(
        migrations: Vec<Migration>,
    ) -> Result<(), ApplyMigrationError> {
        for split in 1..migrations.len() {
            let db = empty_db().await;

            internal_apply_migrations(db.connect().expect("Can connect"), &migrations[..split])
                .await?;

            internal_apply_migrations(db.connect().expect("Can connect"), &migrations).await?;
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_migrate_v1_database(
        #[future(awt)] v1_db: Connection,
        migrations: Vec<Migration>,
    ) -> Result<(), ApplyMigrationError> {
        internal_apply_migrations(v1_db, &migrations).await
    }

    #[rstest]
    #[tokio::test]
    async fn can_migrate_v2_database(
        #[future(awt)] v2_db: Connection,
        migrations: Vec<Migration>,
    ) -> Result<(), ApplyMigrationError> {
        internal_apply_migrations(v2_db, &migrations /*[..(migrations.len() - 1)]*/).await
    }

    // ----------- FIXTURES -----------

    #[fixture]
    fn migrations() -> Vec<Migration> {
        get_migrations()
    }

    #[fixture]
    async fn empty_db() -> Database {
        Builder::new_local(":memory:").build().await.unwrap()
    }

    #[fixture]
    async fn v1_db(#[future(awt)] empty_db: Database) -> Connection {
        use crate::server::db::tables::{
            ACTIVITIES_T, LOGINS_T, R_ACTIVITIES_T, R_CARD_ASSIGNS_T, R_CARDS_T, R_CHANGES_T,
            R_TAB_ASSIGNS_T, R_TABS_T, R_WATCHED_TABS_T, SOURCES_T, VERSION_T,
        };

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

        let v1_db = empty_db.connect().expect("Can connect");

        v1_db
            .execute_transactional_batch(&stmnts.join(";\n"))
            .await
            .expect("Can apply v1 migrations on empty database");

        v1_db
    }

    #[fixture]
    async fn v2_db(#[future(awt)] v1_db: Connection) -> Connection {
        use crate::server::db::tables::{SOURCES_T, VERSION_T};

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

        v1_db
            .execute_transactional_batch(&stmnts.join(";\n"))
            .await
            .expect("Can migrate v1 database to v2 database");

        v1_db
    }
}
