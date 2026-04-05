use std::collections::HashMap;
use std::ops::Deref;

use libsql::{Connection, de};
use log::info;
use time::OffsetDateTime;

use crate::db::tables::{R_CARD_ASSIGNS_T, R_CARDS_T};
use crate::roadmap::types::{CardAssignmentInfo, RCard, Roadmap};
use crate::shared::DatabaseError;

async fn add_card(db: impl Deref<Target = Connection>, card: &RCard) -> Result<u32, DatabaseError> {
    let mut result = db
        .query(
            &format!(
                "INSERT INTO {R_CARDS_T} 
                    (roadmap_id, name, description, image_url, slug, timestamp)
                VALUES 
                    (?1,?2,?3,?4,?5,?6)
                RETURNING id
                "
            ),
            (
                card.id.as_str(),
                card.name.as_str(),
                card.description.as_str(),
                card.image_url.as_deref(),
                card.slug.as_str(),
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    let r = result.next().await?.ok_or(DatabaseError::RowError)?;

    Ok(r.get(0)?)
}

pub async fn assign_card(
    db: impl Deref<Target = Connection>,
    card_id: u32,
    assign_info: CardAssignmentInfo,
) -> Result<u64, DatabaseError> {
    db.execute(
        &format!(
            "INSERT INTO {R_CARD_ASSIGNS_T} 
                (activity_id, tab_id, card_id, section_position, card_position, timestamp) 
            VALUES 
                (?1,?2,?3,?4,?5,?6)
            "
        ),
        (
            assign_info.activity_id,
            assign_info.tab_id,
            card_id,
            assign_info.section_pos,
            assign_info.card_pos,
            serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
        ),
    )
    .await
    .map_err(|e| e.into())
}

pub async fn add_and_assign_card(
    db: impl Deref<Target = Connection>,
    card: &RCard,
    assign_info: CardAssignmentInfo,
) -> Result<u32, DatabaseError> {
    let card_id = add_card(db.deref(), card).await?;
    assign_card(db, card_id, assign_info).await?;
    Ok(card_id)
}

pub async fn add_and_assign_tab_cards(
    db: impl Deref<Target = Connection>,
    roadmap_activity_id: u32,
    tab_db_id: u32,
    cards: &[RCard],
) -> Result<(), DatabaseError> {
    info!("Saving all cards for tab with id {}", tab_db_id);

    for card in cards {
        let _ = add_and_assign_card(
            db.deref(),
            card,
            CardAssignmentInfo {
                activity_id: roadmap_activity_id,
                tab_id: tab_db_id,
                section_pos: card.section_position.unwrap(),
                card_pos: card.card_position.unwrap(),
            },
        )
        .await?;
    }

    info!("Finished Saving Cards for tab with id {}", tab_db_id);
    Ok(())
}

pub async fn add_and_assign_cards(
    db: impl Deref<Target = Connection>,
    roadmap: &Roadmap,
    roadmap_activity_id: u32,
    tab_roadmap_to_db_ids: &HashMap<String, u32>,
) -> Result<(), DatabaseError> {
    info!("Saving all cards");

    for k in roadmap.cards.keys() {
        let tab_db_id = tab_roadmap_to_db_ids.get(k).unwrap();
        let cards = roadmap.cards.get(k).unwrap();

        add_and_assign_tab_cards(db.deref(), roadmap_activity_id, *tab_db_id, cards).await?;
    }

    info!("Finished Saving Cards");
    Ok(())
}

pub async fn get_roadmap_activity_cards(
    db: impl Deref<Target = Connection>,
    roadmap_activity_id: u32,
) -> Result<Vec<RCard>, DatabaseError> {
    let mut rows = db
        .query(
            &format!(
                "SELECT 
                    ra.id as assign_db_id,
                    ra.tab_id,
                    ra.card_id as db_id,
                    ra.section_position,
                    ra.card_position,

                    rc.roadmap_id AS id,
                    rc.name,
                    rc.description,
                    rc.image_url,
                    rc.slug
                FROM {R_CARD_ASSIGNS_T} AS ra
                INNER JOIN {R_CARDS_T} AS rc 
                    ON ra.card_id = rc.id
                WHERE ra.activity_id = ?1
                "
            ),
            [roadmap_activity_id],
        )
        .await?;

    let mut cards = Vec::new();
    while let Some(row) = rows.next().await? {
        let card = de::from_row::<RCard>(&row)?;
        cards.push(card);
    }

    Ok(cards)
}

#[cfg(test)]
pub mod tests {
    use libsql::{Value, params};
    use rstest::rstest;
    use time::ext::NumericalDuration;

    use super::super::activity::add_activity;
    use super::super::tabs::{add_and_assign_tab, tests::make_tab};
    use super::*;
    use crate::db::tests::empty_db;

    #[rstest]
    #[tokio::test]
    async fn can_add_card(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();

        let card_name = "fake card";
        let card = make_card(card_name, 0);

        let id = add_card(&empty_db, &card).await?;

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        id,
                        roadmap_id,
                        name,
                        description,
                        image_url,
                        slug,
                        timestamp
                    FROM `{R_CARDS_T}`
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get card");
        };

        if let Value::Integer(db_id) = row.get_value(0)? {
            assert_eq!(db_id, id as i64);
        } else {
            panic!("id isn't an integer");
        }

        if let Value::Text(db_roadmap_id) = row.get_value(1)? {
            assert_eq!(db_roadmap_id.as_str(), card_name);
        } else {
            panic!("roadmap_id isn't text");
        }

        if let Value::Text(name) = row.get_value(2)? {
            assert_eq!(name.as_str(), card_name);
        } else {
            panic!("name isn't text");
        }

        if let Value::Text(description) = row.get_value(3)? {
            assert_eq!(description.as_str(), card_name);
        } else {
            panic!("description isn't text");
        }

        if let Value::Text(image) = row.get_value(4)? {
            assert_eq!(image.as_str(), card_name);
        } else {
            panic!("image isn't text");
        }

        if let Value::Text(slug) = row.get_value(5)? {
            assert_eq!(slug.as_str(), card_name);
        } else {
            panic!("slug isn't text");
        }

        if let Value::Text(timestamp) = row.get_value(6)? {
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
    async fn can_assign_card(#[future(awt)] empty_db: Connection) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();

        let parent_activity_id = add_activity(&empty_db).await?;
        let tab_id =
            add_and_assign_tab(&empty_db, &make_tab("fake_tab"), parent_activity_id).await?;

        let card_name = "fake_card";
        let card_id = add_card(&empty_db, &make_card(card_name, 0)).await?;

        let rows_affected = assign_card(
            &empty_db,
            card_id,
            CardAssignmentInfo {
                activity_id: parent_activity_id,
                tab_id,
                section_pos: 0,
                card_pos: 0,
            },
        )
        .await?;

        assert_eq!(rows_affected, 1);

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        tab_id,
                        activity_id,
                        card_id,
                        section_position,
                        card_position,
                        timestamp
                    FROM {R_CARD_ASSIGNS_T}
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get card assignment");
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

        if let Value::Integer(card_db_id) = row.get_value(2)? {
            assert_eq!(card_db_id, card_id as i64);
        } else {
            panic!("card_id isn't an integer");
        }

        if let Value::Integer(section_position) = row.get_value(3)? {
            assert_eq!(section_position, 0);
        } else {
            panic!("section_position isn't an integer");
        }

        if let Value::Integer(card_position) = row.get_value(4)? {
            assert_eq!(card_position, 0);
        } else {
            panic!("card_position isn't an integer");
        }

        if let Value::Text(timestamp) = row.get_value(5)? {
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
    async fn can_not_assign_non_existent_card(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let parent_activity_id = add_activity(&empty_db).await?;
        let tab_id =
            add_and_assign_tab(&empty_db, &make_tab("fake tab"), parent_activity_id).await?;

        let result = assign_card(
            &empty_db,
            0,
            CardAssignmentInfo {
                activity_id: parent_activity_id,
                tab_id,
                section_pos: 0,
                card_pos: 0,
            },
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_assign_to_non_existent_activity_and_tab(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let card_name = "fake_card";
        let card_id = add_card(&empty_db, &make_card(card_name, 0)).await?;

        let result = assign_card(
            &empty_db,
            card_id,
            CardAssignmentInfo {
                activity_id: 0,
                tab_id: 0,
                section_pos: 0,
                card_pos: 0,
            },
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_assign_to_non_existent_tab(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let parent_activity_id = add_activity(&empty_db).await?;

        let card_name = "fake_card";
        let card_id = add_card(&empty_db, &make_card(card_name, 0)).await?;

        let result = assign_card(
            &empty_db,
            card_id,
            CardAssignmentInfo {
                activity_id: parent_activity_id,
                tab_id: 0,
                section_pos: 0,
                card_pos: 0,
            },
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_add_and_assign_card(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let parent_activity_id = add_activity(&empty_db).await?;
        let tab_id =
            add_and_assign_tab(&empty_db, &make_tab("fake_tab"), parent_activity_id).await?;

        let card_name = "fake_card";
        let card_id = add_and_assign_card(
            &empty_db,
            &make_card(card_name, 0),
            CardAssignmentInfo {
                activity_id: parent_activity_id,
                tab_id,
                section_pos: 0,
                card_pos: 0,
            },
        )
        .await?;

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        id,
                        roadmap_id,
                        name,
                        description,
                        image_url,
                        slug,
                        timestamp
                    FROM `{R_CARDS_T}`
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get card");
        };

        if let Value::Integer(db_id) = row.get_value(0)? {
            assert_eq!(db_id, card_id as i64);
        } else {
            panic!("id isn't an integer");
        }

        if let Value::Text(db_roadmap_id) = row.get_value(1)? {
            assert_eq!(db_roadmap_id.as_str(), card_name);
        } else {
            panic!("roadmap_id isn't text");
        }

        if let Value::Text(name) = row.get_value(2)? {
            assert_eq!(name.as_str(), card_name);
        } else {
            panic!("name isn't text");
        }

        if let Value::Text(description) = row.get_value(3)? {
            assert_eq!(description.as_str(), card_name);
        } else {
            panic!("description isn't text");
        }

        if let Value::Text(image) = row.get_value(4)? {
            assert_eq!(image.as_str(), card_name);
        } else {
            panic!("image isn't text");
        }

        if let Value::Text(slug) = row.get_value(5)? {
            assert_eq!(slug.as_str(), card_name);
        } else {
            panic!("slug isn't text");
        }

        if let Value::Text(timestamp) = row.get_value(6)? {
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
                        card_id,
                        section_position,
                        card_position,
                        timestamp
                    FROM {R_CARD_ASSIGNS_T}
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get card assignment");
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

        if let Value::Integer(card_db_id) = row.get_value(2)? {
            assert_eq!(card_db_id, card_id as i64);
        } else {
            panic!("card_id isn't an integer");
        }

        if let Value::Integer(section_position) = row.get_value(3)? {
            assert_eq!(section_position, 0);
        } else {
            panic!("section_position isn't an integer");
        }

        if let Value::Integer(card_position) = row.get_value(4)? {
            assert_eq!(card_position, 0);
        } else {
            panic!("card_position isn't an integer");
        }

        if let Value::Text(timestamp) = row.get_value(5)? {
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
    async fn can_add_and_assign_card_within_transaction(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let parent_activity_id = add_activity(&empty_db).await?;
        let tab_id =
            add_and_assign_tab(&empty_db, &make_tab("fake_tab"), parent_activity_id).await?;

        let tx = empty_db.transaction().await?;

        let card_name = "fake_card";
        let card_id = add_and_assign_card(
            tx.deref(),
            &make_card(card_name, 0),
            CardAssignmentInfo {
                activity_id: parent_activity_id,
                tab_id,
                section_pos: 0,
                card_pos: 0,
            },
        )
        .await?;

        tx.commit().await?;

        let mut rows = empty_db
            .query(
                &format!(
                    "SELECT
                        id,
                        roadmap_id,
                        name,
                        description,
                        image_url,
                        slug,
                        timestamp
                    FROM `{R_CARDS_T}`
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get card");
        };

        if let Value::Integer(db_id) = row.get_value(0)? {
            assert_eq!(db_id, card_id as i64);
        } else {
            panic!("id isn't an integer");
        }

        if let Value::Text(db_roadmap_id) = row.get_value(1)? {
            assert_eq!(db_roadmap_id.as_str(), card_name);
        } else {
            panic!("roadmap_id isn't text");
        }

        if let Value::Text(name) = row.get_value(2)? {
            assert_eq!(name.as_str(), card_name);
        } else {
            panic!("name isn't text");
        }

        if let Value::Text(description) = row.get_value(3)? {
            assert_eq!(description.as_str(), card_name);
        } else {
            panic!("description isn't text");
        }

        if let Value::Text(image) = row.get_value(4)? {
            assert_eq!(image.as_str(), card_name);
        } else {
            panic!("image isn't text");
        }

        if let Value::Text(slug) = row.get_value(5)? {
            assert_eq!(slug.as_str(), card_name);
        } else {
            panic!("slug isn't text");
        }

        if let Value::Text(timestamp) = row.get_value(6)? {
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
                        card_id,
                        section_position,
                        card_position,
                        timestamp
                    FROM {R_CARD_ASSIGNS_T}
                    "
                ),
                params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            panic!("Could not get card assignment");
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

        if let Value::Integer(card_db_id) = row.get_value(2)? {
            assert_eq!(card_db_id, card_id as i64);
        } else {
            panic!("card_id isn't an integer");
        }

        if let Value::Integer(section_position) = row.get_value(3)? {
            assert_eq!(section_position, 0);
        } else {
            panic!("section_position isn't an integer");
        }

        if let Value::Integer(card_position) = row.get_value(4)? {
            assert_eq!(card_position, 0);
        } else {
            panic!("card_position isn't an integer");
        }

        if let Value::Text(timestamp) = row.get_value(5)? {
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
    async fn can_not_add_and_assign_to_non_existent_activity_and_tab(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let card_name = "fake card";
        let card = make_card(card_name, 0);

        let result = add_and_assign_card(
            &empty_db,
            &card,
            CardAssignmentInfo {
                activity_id: 0,
                tab_id: 0,
                section_pos: 0,
                card_pos: 0,
            },
        )
        .await;

        assert!(result.is_err());

        let mut rows = empty_db
            .query(&format!("SELECT * FROM `{R_CARDS_T}`"), params!())
            .await?;

        assert!(rows.next().await?.is_none());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_add_and_assign_to_non_existent_tab(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let parent_activity_id = add_activity(&empty_db).await?;

        let card_name = "fake card";
        let card = make_card(card_name, 0);

        let result = add_and_assign_card(
            &empty_db,
            &card,
            CardAssignmentInfo {
                activity_id: parent_activity_id,
                tab_id: 0,
                section_pos: 0,
                card_pos: 0,
            },
        )
        .await;

        assert!(result.is_err());

        let mut rows = empty_db
            .query(&format!("SELECT * FROM `{R_CARDS_T}`"), params!())
            .await?;

        assert!(rows.next().await?.is_none());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_roadmap_activity_cards(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let parent_activity_id_1 = add_activity(&empty_db).await?;
        let parent_activity_id_2 = add_activity(&empty_db).await?;

        let tab_id_1 =
            add_and_assign_tab(&empty_db, &make_tab("fake_tab_1"), parent_activity_id_1).await?;
        let tab_id_2 =
            add_and_assign_tab(&empty_db, &make_tab("fake_tab_2"), parent_activity_id_2).await?;

        let card_names = ["fake_card_1", "fake_card_2", "fake_card_3", "fake_card_4"];
        let mut card_ids = vec![];

        for (pos, card_name) in card_names[..2].iter().enumerate() {
            let pos = pos as u32;
            card_ids.push(
                add_and_assign_card(
                    &empty_db,
                    &make_card(card_name, pos),
                    CardAssignmentInfo {
                        activity_id: parent_activity_id_1,
                        tab_id: tab_id_1,
                        section_pos: pos,
                        card_pos: pos,
                    },
                )
                .await?,
            );
        }

        for (pos, card_name) in card_names[2..].iter().enumerate() {
            let pos = (pos + 2) as u32;
            card_ids.push(
                add_and_assign_card(
                    &empty_db,
                    &make_card(card_name, pos),
                    CardAssignmentInfo {
                        activity_id: parent_activity_id_2,
                        tab_id: tab_id_2,
                        section_pos: pos,
                        card_pos: pos,
                    },
                )
                .await?,
            );
        }

        let cards = get_roadmap_activity_cards(&empty_db, parent_activity_id_2).await?;

        assert_eq!(cards.len(), card_names[2..].len());
        for card in cards {
            assert!(card_names[2..].contains(&card.name.as_str()));
            assert!(card_names[2..].contains(&card.slug.as_str()));
            assert!(card_names[2..].contains(&card.id.as_str()));
            assert!(card_names[2..].contains(&card.image_url.expect("has image").as_str()));
            assert!(card_names[2..].contains(&card.description.as_str()));
            assert!(
                card_ids[2..].contains(&card.db_id.expect("type from db should have db id set"))
            );
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_get_empty_roadmap_activity_cards(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let cards = get_roadmap_activity_cards(&empty_db, 0).await?;

        assert!(cards.is_empty());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_add_and_assign_tab_cards(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let parent_activity_id = add_activity(&empty_db).await?;

        let tab_id =
            add_and_assign_tab(&empty_db, &make_tab("fake_tab"), parent_activity_id).await?;

        let card_names = ["fake_card_1", "fake_card_2", "fake_card_3", "fake_card_4"];
        let cards = card_names
            .iter()
            .enumerate()
            .map(|(pos, card_name)| make_card(card_name, pos as u32))
            .collect::<Vec<_>>();

        add_and_assign_tab_cards(&empty_db, parent_activity_id, tab_id, &cards).await?;

        let db_cards = get_roadmap_activity_cards(&empty_db, parent_activity_id).await?;

        assert_eq!(db_cards.len(), cards.len());

        for card in db_cards {
            assert!(card_names.contains(&card.name.as_str()));
            assert!(card_names.contains(&card.slug.as_str()));
            assert!(card_names.contains(&card.id.as_str()));
            assert!(card_names.contains(&card.image_url.expect("has image").as_str()));
            assert!(card_names.contains(&card.description.as_str()));
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_add_and_assign_tab_cards_for_non_existent_activity_and_tab(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let card_names = ["fake_card_1", "fake_card_2", "fake_card_3", "fake_card_4"];
        let cards = card_names
            .iter()
            .enumerate()
            .map(|(pos, card_name)| make_card(card_name, pos as u32))
            .collect::<Vec<_>>();

        let result = add_and_assign_tab_cards(&empty_db, 0, 0, &cards).await;
        assert!(result.is_err());

        let db_cards = get_roadmap_activity_cards(&empty_db, 0).await?;
        assert!(db_cards.is_empty());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_add_and_assign_tab_cards_for_non_existent_tab(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let parent_activity_id = add_activity(&empty_db).await?;

        let card_names = ["fake_card_1", "fake_card_2", "fake_card_3", "fake_card_4"];
        let cards = card_names
            .iter()
            .enumerate()
            .map(|(pos, card_name)| make_card(card_name, pos as u32))
            .collect::<Vec<_>>();

        let result = add_and_assign_tab_cards(&empty_db, parent_activity_id, 0, &cards).await;
        assert!(result.is_err());

        let db_cards = get_roadmap_activity_cards(&empty_db, parent_activity_id).await?;
        assert!(db_cards.is_empty());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn can_add_and_assign_cards(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let parent_activity_id = add_activity(&empty_db).await?;

        let tabs = [make_tab("fake_tab_1"), make_tab("fake_tab_2")];

        let tab_id_1 = add_and_assign_tab(&empty_db, &tabs[0], parent_activity_id).await?;
        let tab_id_2 = add_and_assign_tab(&empty_db, &tabs[1], parent_activity_id).await?;

        let card_names = ["fake_card_1", "fake_card_2", "fake_card_3", "fake_card_4"];
        let cards = card_names
            .iter()
            .enumerate()
            .map(|(pos, card_name)| make_card(card_name, pos as u32))
            .collect::<Vec<_>>();

        let mut cards_map = HashMap::new();
        cards_map.insert(tabs[0].id.clone(), cards[..2].to_vec());
        cards_map.insert(tabs[1].id.clone(), cards[2..].to_vec());

        let rmap = Roadmap::with_data(tabs.to_vec(), cards_map);

        let tab_roadmap_to_db_ids = HashMap::from([
            (tabs[0].id.clone(), tab_id_1),
            (tabs[1].id.clone(), tab_id_2),
        ]);

        add_and_assign_cards(&empty_db, &rmap, parent_activity_id, &tab_roadmap_to_db_ids).await?;

        let db_cards = get_roadmap_activity_cards(&empty_db, parent_activity_id).await?;

        assert_eq!(db_cards.len(), cards.len());

        for card in db_cards {
            assert!(card_names.contains(&card.name.as_str()));
            assert!(card_names.contains(&card.slug.as_str()));
            assert!(card_names.contains(&card.id.as_str()));
            assert!(card_names.contains(&card.image_url.expect("has image").as_str()));
            assert!(card_names.contains(&card.description.as_str()));
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_add_and_assign_cards_to_non_saved_activity_and_tabs(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let tabs = [make_tab("fake_tab_1"), make_tab("fake_tab_2")];

        let card_names = ["fake_card_1", "fake_card_2", "fake_card_3", "fake_card_4"];
        let cards = card_names
            .iter()
            .enumerate()
            .map(|(pos, card_name)| make_card(card_name, pos as u32))
            .collect::<Vec<_>>();

        let mut cards_map = HashMap::new();
        cards_map.insert(tabs[0].id.clone(), cards[..2].to_vec());
        cards_map.insert(tabs[1].id.clone(), cards[2..].to_vec());

        let rmap = Roadmap::with_data(tabs.to_vec(), cards_map);

        let tab_roadmap_to_db_ids =
            HashMap::from([(tabs[0].id.clone(), 0), (tabs[1].id.clone(), 1)]);

        let result = add_and_assign_cards(&empty_db, &rmap, 0, &tab_roadmap_to_db_ids).await;
        assert!(result.is_err());

        let db_cards = get_roadmap_activity_cards(&empty_db, 0).await?;

        assert!(db_cards.is_empty());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "Currently fails intentionally. Will be fixed in a future migration"]
    async fn can_not_add_and_assign_cards_to_non_saved_tabs(
        #[future(awt)] empty_db: Connection,
    ) -> Result<(), DatabaseError> {
        let parent_activity_id = add_activity(&empty_db).await?;

        let tabs = [make_tab("fake_tab_1"), make_tab("fake_tab_2")];

        let card_names = ["fake_card_1", "fake_card_2", "fake_card_3", "fake_card_4"];
        let cards = card_names
            .iter()
            .enumerate()
            .map(|(pos, card_name)| make_card(card_name, pos as u32))
            .collect::<Vec<_>>();

        let mut cards_map = HashMap::new();
        cards_map.insert(tabs[0].id.clone(), cards[..2].to_vec());
        cards_map.insert(tabs[1].id.clone(), cards[2..].to_vec());

        let rmap = Roadmap::with_data(tabs.to_vec(), cards_map);

        let tab_roadmap_to_db_ids =
            HashMap::from([(tabs[0].id.clone(), 0), (tabs[1].id.clone(), 1)]);

        let result =
            add_and_assign_cards(&empty_db, &rmap, parent_activity_id, &tab_roadmap_to_db_ids)
                .await;

        assert!(result.is_err());

        let db_cards = get_roadmap_activity_cards(&empty_db, parent_activity_id).await?;

        assert!(db_cards.is_empty());

        Ok(())
    }

    // ------- Util -------
    pub fn make_card(card_title: &str, card_pos: u32) -> RCard {
        RCard {
            id: card_title.to_string(),
            name: card_title.to_string(),
            description: card_title.to_string(),
            image_url: Some(card_title.to_string()),
            slug: card_title.to_string(),
            db_id: None,
            section_position: Some(card_pos),
            card_position: Some(card_pos),
            assign_db_id: None,
            tab_id: None,
        }
    }
}
