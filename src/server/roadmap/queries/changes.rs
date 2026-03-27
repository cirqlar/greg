use libsql::{Connection, de};

use crate::server::db::tables::{R_CARD_ASSIGNS_T, R_CARDS_T, R_CHANGES_T, R_TABS_T};
use crate::server::roadmap::types::RDBChange;
use crate::server::shared::DatabaseError;

pub async fn get_roadmap_changes(
    db: Connection,
    activity_id: u32,
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
            [activity_id],
        )
        .await?;

    let mut changes = Vec::new();
    while let Some(r) = result.next().await? {
        let c = de::from_row(&r)?;
        changes.push(c);
    }

    Ok(changes)
}
