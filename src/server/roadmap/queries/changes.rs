use std::ops::Deref;

use libsql::{Connection, de};
use time::OffsetDateTime;

use crate::db::tables::{R_CARD_ASSIGNS_T, R_CARDS_T, R_CHANGES_T, R_TABS_T};
use crate::roadmap::types::{ChangeInfo, RDBChange};
use crate::shared::DatabaseError;

pub async fn add_change(
    db: impl Deref<Target = Connection>,
    change_info: ChangeInfo,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!(
            "INSERT INTO {R_CHANGES_T} 
                    (type, activity_id, previous_card_id, current_card_id, tab_id, timestamp) 
                VALUES 
                    (?1,?2,?3,?4,?5,?6)
                "
        ),
        (
            change_info.change_type,
            change_info.activity_id,
            change_info.previous_card_id,
            change_info.current_card_id,
            change_info.tab_id,
            serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
        ),
    )
    .await
    .map_err(|e| e.into())
}

pub async fn get_roadmap_activity_changes(
    db: impl Deref<Target = Connection>,
    roadmap_activity_id: u32,
) -> Result<Vec<RDBChange>, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "SELECT
                    rch.id, rch.type,

                    rc1.id AS previous_card_db_id, rc1.roadmap_id AS previous_card_id,
                    rc1.name AS previous_card_name, rc1.description AS previous_card_description,
                    rc1.image_url  AS previous_card_image_url, rc1.slug AS previous_card_slug,

                    rc2.id AS current_card_db_id, rc2.roadmap_id AS current_card_id,
                    rc2.name AS current_card_name, rc2.description AS current_card_description,
                    rc2.image_url  AS current_card_image_url, rc2.slug AS current_card_slug,

                    rt.id AS tab_db_id, rt.roadmap_id AS tab_id,
                    rt.name AS tab_name, rt.slug AS tab_slug,

                    rct.name AS card_tab_name
                FROM {R_CHANGES_T} AS rch
                LEFT JOIN {R_CARDS_T} as rc1
                    ON rch.previous_card_id = rc1.id
                LEFT JOIN {R_CARDS_T} as rc2
                    ON rch.current_card_id = rc2.id
                LEFT JOIN {R_CARD_ASSIGNS_T} as rca
                    ON rch.previous_card_id = rca.card_id OR rch.current_card_id = rca.card_id
                LEFT JOIN {R_TABS_T} as rct
                    ON rca.tab_id = rct.id
                LEFT JOIN {R_TABS_T} as rt
                    ON rch.tab_id = rt.id
                WHERE rch.activity_id = ?1
                GROUP BY rch.id
                "
            ),
            [roadmap_activity_id],
        )
        .await?;

    let mut changes = Vec::new();
    while let Some(r) = result.next().await? {
        let c = de::from_row(&r)?;
        changes.push(c);
    }

    Ok(changes)
}

#[cfg(test)]
mod tests {
    use libsql::Value;
    use rstest::rstest;
    use time::ext::NumericalDuration;

    use super::super::{
        activity::add_activity,
        cards::add_and_assign_card,
        tabs::{add_and_assign_tab, assign_tab},
    };
    use super::super::{cards::tests::make_card, tabs::tests::make_tab};
    use super::*;
    use crate::roadmap::types::{CardChange, ChangeInfo, TabChange};
    use crate::{db::tests::empty_db, roadmap::types::CardAssignmentInfo};

    #[rstest]
    #[tokio::test]
    async fn can_add_tab_added_change(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let next_activity_id = add_activity(&empty_db).await?;

        let tab_name = "fake_tab";
        let tab = make_tab(tab_name);
        let tab_id = add_and_assign_tab(&empty_db, &tab, next_activity_id).await?;

        let rows_affected = add_change(
            &empty_db,
            ChangeInfo {
                activity_id: next_activity_id,
                change_type: TabChange::Added { tab_index: 0 }.as_str(),
                previous_card_id: None,
                current_card_id: None,
                tab_id: Some(tab_id),
            },
        )
        .await?;

        assert_eq!(rows_affected, 1);

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        type, activity_id, previous_card_id, current_card_id, tab_id, timestamp
                    FROM {R_CHANGES_T}
                    WHERE activity_id = ?1
                    "
                ),
                [next_activity_id],
            )
            .await?;

        let row = rows.next().await?.expect("Can get change from db");

        if let Value::Text(change_type) = row.get_value(0)? {
            assert_eq!(
                change_type.as_str(),
                TabChange::Added { tab_index: 0 }.as_str()
            );
        } else {
            panic!("type is not text");
        }

        if let Value::Integer(db_activity_id) = row.get_value(1)? {
            assert_eq!(db_activity_id, next_activity_id as i64);
        } else {
            panic!("activity_id is not integer");
        }

        if row.get_value(2)? != Value::Null {
            panic!("previous_card_id is not null");
        }

        if row.get_value(3)? != Value::Null {
            panic!("current_card_id is not null");
        }

        if let Value::Integer(db_tab_id) = row.get_value(4)? {
            assert_eq!(db_tab_id, tab_id as i64);
        } else {
            panic!("tab_id is not integer");
        }

        if let Value::Text(timestamp) = row.get_value(5)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialized to OffsetDateTime");

            let difference = timestamp - now;
            assert!(difference.abs() < 1.minutes());
        } else {
            panic!("timestamp is not text");
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_add_tab_removed_change(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let previous_activity_id = add_activity(&empty_db).await?;
        let next_activity_id = add_activity(&empty_db).await?;

        let tab_name = "fake_tab";
        let tab = make_tab(tab_name);
        let tab_id = add_and_assign_tab(&empty_db, &tab, previous_activity_id).await?;

        let rows_affected = add_change(
            &empty_db,
            ChangeInfo {
                activity_id: next_activity_id,
                change_type: TabChange::Removed { tab_index: 0 }.as_str(),
                previous_card_id: None,
                current_card_id: None,
                tab_id: Some(tab_id),
            },
        )
        .await?;

        assert_eq!(rows_affected, 1);

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        type, activity_id, previous_card_id, current_card_id, tab_id, timestamp
                    FROM {R_CHANGES_T}
                    WHERE activity_id = ?1
                    "
                ),
                [next_activity_id],
            )
            .await?;

        let row = rows.next().await?.expect("Can get change from db");

        if let Value::Text(change_type) = row.get_value(0)? {
            assert_eq!(
                change_type.as_str(),
                TabChange::Removed { tab_index: 0 }.as_str()
            );
        } else {
            panic!("type is not text");
        }

        if let Value::Integer(db_activity_id) = row.get_value(1)? {
            assert_eq!(db_activity_id, next_activity_id as i64);
        } else {
            panic!("activity_id is not integer");
        }

        if row.get_value(2)? != Value::Null {
            panic!("previous_card_id is not null");
        }

        if row.get_value(3)? != Value::Null {
            panic!("current_card_id is not null");
        }

        if let Value::Integer(db_tab_id) = row.get_value(4)? {
            assert_eq!(db_tab_id, tab_id as i64);
        } else {
            panic!("tab_id is not integer");
        }

        if let Value::Text(timestamp) = row.get_value(5)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialized to OffsetDateTime");

            let difference = timestamp - now;
            assert!(difference.abs() < 1.minutes());
        } else {
            panic!("timestamp is not text");
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_add_card_added_change(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let next_activity_id = add_activity(&empty_db).await?;

        let tab_name = "fake_tab";
        let tab = make_tab(tab_name);
        let tab_id = add_and_assign_tab(&empty_db, &tab, next_activity_id).await?;

        let card_name = "fake_card";
        let card = make_card(card_name, 1);
        let card_id = add_and_assign_card(
            &empty_db,
            &card,
            CardAssignmentInfo {
                activity_id: next_activity_id,
                tab_id,
                section_pos: 1,
                card_pos: 1,
            },
        )
        .await?;

        let rows_affected = add_change(
            &empty_db,
            ChangeInfo {
                activity_id: next_activity_id,
                change_type: CardChange::Added {
                    tab_id: "()".into(),
                    card_index: 0,
                }
                .as_str(),
                previous_card_id: None,
                current_card_id: Some(card_id),
                tab_id: None,
            },
        )
        .await?;

        assert_eq!(rows_affected, 1);

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        type, activity_id, previous_card_id, current_card_id, tab_id, timestamp
                    FROM {R_CHANGES_T}
                    WHERE activity_id = ?1
                    "
                ),
                [next_activity_id],
            )
            .await?;

        let row = rows.next().await?.expect("Can get change from db");

        if let Value::Text(change_type) = row.get_value(0)? {
            assert_eq!(
                change_type.as_str(),
                CardChange::Added {
                    tab_id: "()".into(),
                    card_index: 0
                }
                .as_str()
            );
        } else {
            panic!("type is not text");
        }

        if let Value::Integer(db_activity_id) = row.get_value(1)? {
            assert_eq!(db_activity_id, next_activity_id as i64);
        } else {
            panic!("activity_id is not integer");
        }

        if row.get_value(2)? != Value::Null {
            panic!("previous_card_id is not null");
        }

        if let Value::Integer(db_current_card_id) = row.get_value(3)? {
            assert_eq!(db_current_card_id, card_id as i64);
        } else {
            panic!("current_card_id is not integer");
        }

        if row.get_value(4)? != Value::Null {
            panic!("tab_id is not null");
        }

        if let Value::Text(timestamp) = row.get_value(5)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialized to OffsetDateTime");

            let difference = timestamp - now;
            assert!(difference.abs() < 1.minutes());
        } else {
            panic!("timestamp is not text");
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_add_card_removed_change(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let previous_activity_id = add_activity(&empty_db).await?;
        let next_activity_id = add_activity(&empty_db).await?;

        let tab_name = "fake_tab";
        let tab = make_tab(tab_name);
        let tab_id = add_and_assign_tab(&empty_db, &tab, previous_activity_id).await?;
        let _ = assign_tab(&empty_db, next_activity_id, tab_id).await?;

        let card_name = "fake_card";
        let card = make_card(card_name, 1);
        let card_id = add_and_assign_card(
            &empty_db,
            &card,
            CardAssignmentInfo {
                activity_id: previous_activity_id,
                tab_id,
                section_pos: 1,
                card_pos: 1,
            },
        )
        .await?;

        let rows_affected = add_change(
            &empty_db,
            ChangeInfo {
                activity_id: next_activity_id,
                change_type: CardChange::Removed {
                    tab_id: "()".into(),
                    card_index: 0,
                }
                .as_str(),
                previous_card_id: Some(card_id),
                current_card_id: None,
                tab_id: None,
            },
        )
        .await?;

        assert_eq!(rows_affected, 1);

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        type, activity_id, previous_card_id, current_card_id, tab_id, timestamp
                    FROM {R_CHANGES_T}
                    WHERE activity_id = ?1
                    "
                ),
                [next_activity_id],
            )
            .await?;

        let row = rows.next().await?.expect("Can get change from db");

        if let Value::Text(change_type) = row.get_value(0)? {
            assert_eq!(
                change_type.as_str(),
                CardChange::Removed {
                    tab_id: "()".into(),
                    card_index: 0
                }
                .as_str()
            );
        } else {
            panic!("type is not text");
        }

        if let Value::Integer(db_activity_id) = row.get_value(1)? {
            assert_eq!(db_activity_id, next_activity_id as i64);
        } else {
            panic!("activity_id is not integer");
        }

        if let Value::Integer(db_previous_card_id) = row.get_value(2)? {
            assert_eq!(db_previous_card_id, card_id as i64);
        } else {
            panic!("previous_card_id is not integer");
        }

        if row.get_value(3)? != Value::Null {
            panic!("current_card_id is not null");
        }

        if row.get_value(4)? != Value::Null {
            panic!("tab_id is not null");
        }

        if let Value::Text(timestamp) = row.get_value(5)? {
            let timestamp = serde_json::from_str::<OffsetDateTime>(&timestamp)
                .expect("timestamp from db can be deserialized to OffsetDateTime");

            let difference = timestamp - now;
            assert!(difference.abs() < 1.minutes());
        } else {
            panic!("timestamp is not text");
        }

        Ok(())
    }
}
