use std::sync::Arc;

use libsql::{Connection, Transaction, de, params};
use log::info;
use thiserror::Error;
use time::OffsetDateTime;

use super::tables::MIGRATIONS_T;
use super::types::{DbMigration, Migration};
use crate::shared::DatabaseError;

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
    use std::collections::HashMap;

    use libsql::{Builder, Database};
    use rstest::fixture;
    use rstest::rstest;

    use crate::roadmap::types::{cards::RCard, tabs::RTab};

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
    async fn can_migrate_empty_v1_database(
        #[future(awt)] empty_v1_db: Connection,
        migrations: Vec<Migration>,
    ) -> Result<(), ApplyMigrationError> {
        internal_apply_migrations(empty_v1_db, &migrations).await
    }

    #[rstest]
    #[tokio::test]
    async fn can_migrate_nonempty_v1_database(
        #[future(awt)] v1_db: Connection,
        migrations: Vec<Migration>,
    ) -> Result<(), ApplyMigrationError> {
        internal_apply_migrations(v1_db, &migrations).await
    }

    #[rstest]
    #[tokio::test]
    async fn can_migrate_empty_v2_database(
        #[future(awt)] empty_v2_db: Connection,
        migrations: Vec<Migration>,
    ) -> Result<(), ApplyMigrationError> {
        internal_apply_migrations(empty_v2_db, &migrations).await
    }

    #[rstest]
    #[tokio::test]
    async fn can_migrate_nonempty_v2_database(
        #[future(awt)] v2_db: Connection,
        migrations: Vec<Migration>,
    ) -> Result<(), ApplyMigrationError> {
        internal_apply_migrations(v2_db, &migrations).await
    }
    // Note: Currently useless until migrations to change roadmap tables are added
    #[rstest]
    #[tokio::test]
    async fn can_migrate_database_with_duplicate_cards(
        #[future(awt)] duplicate_db: Connection,
        migrations: Vec<Migration>,
    ) -> Result<(), ApplyMigrationError> {
        internal_apply_migrations(duplicate_db, &migrations).await
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
    async fn empty_v1_db(#[future(awt)] empty_db: Database) -> Connection {
        use crate::db::tables::{
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

    fn make_card(card_id: u32, tab_id: u32) -> RCard {
        let id = format!("fake_card_{}", card_id);
        RCard {
            id: id.clone(),
            name: id.clone(),
            description: id.clone(),
            image_url: Some(format!("http://roadmap.com/{}.png", id)),
            slug: id.clone(),
            db_id: Some(card_id),
            section_position: Some(card_id),
            card_position: Some(card_id),
            assign_db_id: None,
            tab_id: Some(tab_id),
        }
    }

    fn make_tab(tab_id: u32) -> RTab {
        let id = format!("fake_tab_{}", tab_id);
        RTab {
            id: id.clone(),
            name: id.clone(),
            slug: id.clone(),
            db_id: Some(tab_id),
        }
    }

    async fn fill_db_rss(db: &Connection) {
        use crate::rss::queries::{activity, sources};

        // Sources
        sources::add_source(db, "http://fake_source_1.com".into())
            .await
            .expect("Can add source");
        sources::add_source(db, "http://fake_source_2.com".into())
            .await
            .expect("Can add source");

        // Activity
        activity::add_activity(db, 1, "http://fake_source_1.com/fake_activity_1")
            .await
            .expect("Can add activity");
        activity::add_activity(db, 1, "http://fake_source_1.com/fake_activity_2")
            .await
            .expect("Can add activity");
        activity::add_activity(db, 2, "http://fake_source_1.com/fake_activity_3")
            .await
            .expect("Can add activity");
        activity::add_activity(db, 3, "http://fake_source_1.com/fake_activity_4")
            .await
            .expect("Can add activity");
    }

    async fn fill_db_roadmap_in(db: &Connection, mut tabs: Vec<RTab>, cards: Vec<RCard>) {
        use crate::roadmap::queries::roadmap;
        use crate::roadmap::tasks::check::{changes, compare, new_roadmap};
        use crate::roadmap::types::{changes::RChange, roadmap::Roadmap};

        // Roadmap
        let cards: HashMap<String, Vec<RCard>> = tabs
            .iter()
            .map(|tab| {
                (
                    tab.id.clone(),
                    cards
                        .iter()
                        .filter(|c| c.tab_id.as_ref().unwrap() == tab.db_id.as_ref().unwrap())
                        .cloned()
                        .collect(),
                )
            })
            .collect();
        let rmap = Roadmap::with_data(tabs.clone(), cards.clone());
        new_roadmap::save_new_roadmap(db, rmap.clone())
            .await
            .expect("Can save roadmap");

        // Changes
        tabs.push(make_tab(2));
        let n_card = make_card(4, 1);

        let mut n_rmap = rmap.clone();
        n_rmap
            .cards
            .entry(tabs[1].id.clone())
            .and_modify(|c| c.push(n_card));

        let changes = compare::compare_roadmaps(&rmap, &n_rmap);

        let mut tab_ids: HashMap<String, u32> = rmap
            .tabs
            .iter()
            .map(|t| (t.id.clone(), t.db_id.unwrap()))
            .collect();

        let first_non_tab_index = changes
            .iter()
            .enumerate()
            .find_map(|(index, ch)| (!matches!(ch, RChange::Tab(_))).then_some(index))
            .unwrap_or(changes.len());

        let (tab_changes, card_changes) = changes.split_at(first_non_tab_index);

        roadmap::new_activity(db).await.expect("Can add roadmap");

        changes::handle_tab_changes(db, &rmap, &n_rmap, 1, tab_changes, &mut tab_ids)
            .await
            .expect("Can save tab change");

        changes::handle_card_changes(db, &rmap, &n_rmap, 1, card_changes, &tab_ids)
            .await
            .expect("Can save card change");
    }

    async fn fill_db_roadmap(db: &Connection) {
        // Tabs
        let tabs = vec![make_tab(0), make_tab(1)];

        // Cards
        let cards = vec![
            make_card(0, 0),
            make_card(1, 0),
            make_card(2, 1),
            make_card(3, 1),
        ];

        fill_db_roadmap_in(db, tabs, cards).await;
    }

    async fn fill_db_roadmap_with_duplicate(db: &Connection) {
        // Tabs
        let tabs = vec![make_tab(0), make_tab(1)];

        // Cards
        let cards = vec![
            make_card(0, 0),
            make_card(1, 0),
            make_card(2, 1),
            make_card(2, 1),
        ];

        fill_db_roadmap_in(db, tabs, cards).await;
    }

    #[fixture]
    async fn v1_db(#[future(awt)] empty_v1_db: Connection) -> Connection {
        fill_db_rss(&empty_v1_db).await;
        fill_db_roadmap(&empty_v1_db).await;

        empty_v1_db
    }

    #[fixture]
    async fn empty_v2_db(#[future(awt)] empty_v1_db: Connection) -> Connection {
        use crate::db::tables::{SOURCES_T, VERSION_T};

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

        empty_v1_db
            .execute_transactional_batch(&stmnts.join(";\n"))
            .await
            .expect("Can migrate v1 database to v2 database");

        empty_v1_db
    }

    #[fixture]
    async fn v2_db(#[future(awt)] empty_v2_db: Connection) -> Connection {
        fill_db_rss(&empty_v2_db).await;
        fill_db_roadmap(&empty_v2_db).await;

        empty_v2_db
    }

    #[fixture]
    async fn duplicate_db(#[future(awt)] empty_v2_db: Connection) -> Connection {
        fill_db_rss(&empty_v2_db).await;
        fill_db_roadmap_with_duplicate(&empty_v2_db).await;

        empty_v2_db
    }
}
