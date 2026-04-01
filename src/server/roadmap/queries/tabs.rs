use std::ops::Deref;

use libsql::{Connection, de, params};
use time::OffsetDateTime;

use crate::db::tables::{R_ACTIVITIES_T, R_TAB_ASSIGNS_T, R_TABS_T, R_WATCHED_TABS_T};
use crate::roadmap::types::{RTab, RoadmapWatchedTab};
use crate::shared::DatabaseError;

async fn add_tab(db: impl Deref<Target = Connection>, tab: &RTab) -> Result<u32, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "INSERT INTO {R_TABS_T} 
                    (roadmap_id, name, slug, timestamp) 
                VALUES 
                    (?1,?2,?3,?4)
                RETURNING id
                "
            ),
            (
                tab.id.as_str(),
                tab.name.as_str(),
                tab.slug.as_str(),
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    let r = result.next().await?.unwrap();

    Ok(r.get(0)?)
}

pub async fn assign_tab(
    db: impl Deref<Target = Connection>,
    activity_id: u32,
    tab_db_id: u32,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!(
            "INSERT INTO {R_TAB_ASSIGNS_T} 
                    (activity_id, tab_id, timestamp) 
                VALUES 
                    (?1,?2,?3)
                "
        ),
        (
            activity_id,
            tab_db_id,
            serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
        ),
    )
    .await
    .map_err(|e| e.into())
}

pub async fn add_and_assign_tab(
    db: impl Deref<Target = Connection>,
    tab: &RTab,
    activity_id: u32,
) -> Result<u32, DatabaseError> {
    let tab_id = add_tab(db.deref(), tab).await?;

    assign_tab(db, activity_id, tab_id).await?;

    Ok(tab_id)
}

pub async fn get_watched_tabs(
    db: impl Deref<Target = Connection>,
) -> Result<Vec<RoadmapWatchedTab>, DatabaseError> {
    let mut result = db
        .query(&format!("SELECT * FROM {R_WATCHED_TABS_T}"), params!())
        .await?;

    let mut tabs = Vec::new();
    while let Some(r) = result.next().await? {
        let wts: RoadmapWatchedTab = de::from_row(&r)?;
        tabs.push(wts);
    }

    Ok(tabs)
}

pub async fn get_tabs(db: impl Deref<Target = Connection>) -> Result<Vec<RTab>, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "SELECT
                    rt.id AS db_id,
                    rt.roadmap_id AS id,
                    rt.name,
                    rt.slug,
                    (rt.id NOT IN (
                        -- most recent activity's tabs --
                        SELECT rta.tab_id FROM `{R_TAB_ASSIGNS_T}` AS rta
                        WHERE rta.activity_id = (
                            -- most recent activity --
                            SELECT ra.id FROM `{R_ACTIVITIES_T}` as ra
                            ORDER BY ra.id DESC
                            LIMIT 1
                        )
                    )) AS deleted,
                    rwt.id as watch_id
                FROM `{R_TABS_T}` AS rt
                LEFT JOIN `{R_WATCHED_TABS_T}` as rwt
                ON rwt.tab_roadmap_id = rt.roadmap_id;
                "
            ),
            params!(),
        )
        .await?;

    let mut tabs = Vec::new();
    while let Some(r) = result.next().await? {
        let t = de::from_row::<RTab>(&r)?;
        tabs.push(t);
    }

    Ok(tabs)
}

pub async fn get_roadmap_activity_tabs(
    db: impl Deref<Target = Connection>,
    roadmap_activity_id: u32,
) -> Result<Vec<RTab>, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "SELECT
                    ra.tab_id as db_id,
                    rt.roadmap_id AS id,
                    rt.name,
                    rt.slug
                FROM {R_TAB_ASSIGNS_T} AS ra
                INNER JOIN {R_TABS_T} AS rt
                    ON ra.tab_id = rt.id
                WHERE ra.activity_id = ?1
                "
            ),
            [roadmap_activity_id],
        )
        .await?;

    let mut tabs = Vec::new();
    while let Some(r) = result.next().await? {
        let t = de::from_row::<RTab>(&r)?;
        tabs.push(t);
    }

    Ok(tabs)
}

pub async fn add_watched_tab(
    db: impl Deref<Target = Connection>,
    tab_roadmap_id: String,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("INSERT INTO {R_WATCHED_TABS_T} (tab_roadmap_id, timestamp) VALUES (?1, ?2)"),
        [
            tab_roadmap_id,
            serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
        ],
    )
    .await
    .map_err(|e| e.into())
}

pub async fn delete_watched_tab(
    db: impl Deref<Target = Connection>,
    watched_tab_id: u32,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!("DELETE FROM {R_WATCHED_TABS_T} WHERE id = ?1"),
        [watched_tab_id],
    )
    .await
    .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use libsql::Value;
    use rstest::rstest;
    use time::ext::NumericalDuration;

    use super::super::activity::add_activity;
    use super::*;
    use crate::db::tests::empty_db;

    #[rstest]
    #[tokio::test]
    async fn can_add_tab(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();

        let tab_name = "fake tab";
        let tab = make_tab(tab_name);

        let id = add_tab(&empty_db, &tab).await?;

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        id,
                        roadmap_id,
                        name,
                        slug,
                        timestamp
                    FROM `{R_TABS_T}`
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get tab");
        };

        if let Value::Integer(db_id) = row.get_value(0)? {
            assert_eq!(db_id, id as i64);
        } else {
            panic!("id isn't an integer");
        }

        if let Value::Text(db_roadmap_id) = row.get_value(1)? {
            assert_eq!(db_roadmap_id.as_str(), tab_name);
        } else {
            panic!("roadmap_id isn't text");
        }

        if let Value::Text(name) = row.get_value(2)? {
            assert_eq!(name.as_str(), tab_name);
        } else {
            panic!("name isn't text");
        }

        if let Value::Text(slug) = row.get_value(3)? {
            assert_eq!(slug.as_str(), tab_name);
        } else {
            panic!("slug isn't text");
        }

        if let Value::Text(timestamp) = row.get_value(4)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialized to OffsetDateTime");

            let difference = timestamp - now;
            assert!(difference.abs() < 1.minutes());
        } else {
            panic!("timestamp isn't text");
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_assign_tab(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();

        let parent_activity_id = add_activity(&empty_db).await?;
        let tab_name = "fake_tab";
        let tab_id = add_tab(&empty_db, &make_tab(tab_name)).await?;

        let rows_affected = assign_tab(&empty_db, parent_activity_id, tab_id).await?;

        assert_eq!(rows_affected, 1);

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        tab_id,
                        activity_id,
                        timestamp
                    FROM {R_TAB_ASSIGNS_T}
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get tab assignment");
        };

        if let Value::Integer(tab_db_id) = row.get_value(0)? {
            assert_eq!(tab_db_id, tab_id as i64);
        } else {
            panic!("tab_id isn't an integer");
        }

        if let Value::Integer(parent_activity_db_id) = row.get_value(1)? {
            assert_eq!(parent_activity_db_id, parent_activity_id as i64);
        } else {
            panic!("activity_id isn't an integer");
        }

        if let Value::Text(timestamp) = row.get_value(2)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialized to OffsetDateTime");

            let difference = timestamp - now;
            assert!(difference.abs() < 1.minutes());
        } else {
            panic!("timestamp isn't text");
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_assign_non_existent_tab(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let parent_activity_id = add_activity(&empty_db).await?;

        let result = assign_tab(&empty_db, parent_activity_id, 0).await;

        assert!(result.is_err());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_assign_to_non_existent_activity(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let tab_name = "fake_tab";
        let tab_id = add_tab(&empty_db, &make_tab(tab_name)).await?;

        let result = assign_tab(&empty_db, 0, tab_id).await;

        assert!(result.is_err());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_add_and_assign_tab(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let parent_activity_id = add_activity(&empty_db).await?;

        let tab_name = "fake tab";
        let tab = make_tab(tab_name);

        let id = add_and_assign_tab(&empty_db, &tab, parent_activity_id).await?;

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        id,
                        roadmap_id,
                        name,
                        slug,
                        timestamp
                    FROM `{R_TABS_T}`
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get tab");
        };

        if let Value::Integer(db_id) = row.get_value(0)? {
            assert_eq!(db_id, id as i64);
        } else {
            panic!("id isn't an integer");
        }

        if let Value::Text(db_roadmap_id) = row.get_value(1)? {
            assert_eq!(db_roadmap_id.as_str(), tab_name);
        } else {
            panic!("roadmap_id isn't text");
        }

        if let Value::Text(name) = row.get_value(2)? {
            assert_eq!(name.as_str(), tab_name);
        } else {
            panic!("name isn't text");
        }

        if let Value::Text(slug) = row.get_value(3)? {
            assert_eq!(slug.as_str(), tab_name);
        } else {
            panic!("slug isn't text");
        }

        if let Value::Text(timestamp) = row.get_value(4)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialized to OffsetDateTime");

            let difference = timestamp - now;
            assert!(difference.abs() < 1.minutes());
        } else {
            panic!("timestamp isn't text");
        }

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        tab_id,
                        activity_id,
                        timestamp
                    FROM {R_TAB_ASSIGNS_T}
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get tab assignment");
        };

        if let Value::Integer(tab_db_id) = row.get_value(0)? {
            assert_eq!(tab_db_id, id as i64);
        } else {
            panic!("tab_id isn't an integer");
        }

        if let Value::Integer(parent_activity_db_id) = row.get_value(1)? {
            assert_eq!(parent_activity_db_id, parent_activity_id as i64);
        } else {
            panic!("activity_id isn't an integer");
        }

        if let Value::Text(timestamp) = row.get_value(2)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialized to OffsetDateTime");

            let difference = timestamp - now;
            assert!(difference.abs() < 1.minutes());
        } else {
            panic!("timestamp isn't text");
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_add_and_assign_to_non_existent_activity(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let tab_name = "fake tab";
        let tab = make_tab(tab_name);

        let result = add_and_assign_tab(&empty_db, &tab, 0).await;

        assert!(result.is_err());

        let mut rows = empty_db
            .query(&format!("SELECT * FROM `{R_TABS_T}`"), params!())
            .await?;

        assert!(rows.next().await?.is_none());

        Ok(())
    }

    // ------- Util -------
    fn make_tab(tab_name: &str) -> RTab {
        RTab {
            id: tab_name.to_string(),
            name: tab_name.to_string(),
            slug: tab_name.to_string(),
            db_id: None,
            deleted: None,
            watch_id: None,
        }
    }
}
