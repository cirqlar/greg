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
                    `timestamp` TEXT NOT NULL,
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

pub async fn apply_migrations(db: Connection) -> Result<(), ApplyMigrationError> {
    let tx = Arc::from(db.transaction().await.map_err(DatabaseError::from)?);

    let applied_migrations = get_existing_migrations(tx.clone()).await?;

    let migrations = get_migrations();

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

#[cfg(test)]
mod tests {
    use rstest::fixture;
    use rstest::rstest;

    use super::*;

    #[fixture]
    fn migrations() -> Vec<Migration> {
        get_migrations()
    }

    #[rstest]
    fn all_migrations_are_unique(migrations: Vec<Migration>) {
        let mut found = Vec::with_capacity(migrations.len());

        for migration in migrations {
            assert!(!found.contains(&migration.name));

            found.push(migration.name);
        }
    }
}
