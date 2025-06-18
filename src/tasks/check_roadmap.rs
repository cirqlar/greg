use std::{collections::HashMap, env};

use libsql::Transaction;
use log::{error, info, warn};
use time::OffsetDateTime;

use crate::{
    db::{R_ACTIVITIES_T, R_CARD_ASSIGNS_T, R_CARDS_T, R_CHANGES_T, R_TAB_ASSIGNS_T, R_TABS_T},
    queries::roadmap::{get_most_recent_roadmap, get_watched_tabs},
    types::{AppData, RCard, RChange, RTab, Roadmap, StringError, WebRoadmap},
    utils::clean_description,
};

const JSON_START_LANDMARK: &str = "window.pbData";
const JSON_END_LANDMARK: &str = "</script>";

pub async fn get_roadmap_json() -> anyhow::Result<String> {
    let client = reqwest::Client::new();

    let res = client
        .get(env::var("VITE_ROADMAP_URL").expect("VITE_ROADMAP_URL exists"))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(StringError(format!("Request code is {}", res.status())).into());
    }

    let mut content = res.text().await?;
    let Some(json_search_start) = content.find(JSON_START_LANDMARK) else {
        return Err(StringError("Didn't find pbData substring".into()).into());
    };
    let Some(json_start) = content[json_search_start..].find('{') else {
        return Err(StringError("Didn't find open bracket".into()).into());
    };
    let Some(json_search_end) = content[json_search_start..].find(JSON_END_LANDMARK) else {
        return Err(StringError("Didn't find </script>".into()).into());
    };
    let Some(json_end) =
        content[json_search_start..=json_search_start + json_search_end].rfind('}')
    else {
        return Err(StringError("Didn't find close_bracket".into()).into());
    };

    content = content[json_search_start + json_start..=json_search_start + json_end].to_owned();
    Ok(content)
}

fn web_to_saved_roadmap(mut roadmap: WebRoadmap, watched_ids: &[String]) -> Roadmap {
    roadmap.portal_cards.sort_by_key(|c| c.id.clone());

    let mut url_saved_roadmap = Roadmap::with_data(roadmap.portal_tabs, HashMap::new());

    for tab_id in watched_ids {
        let tab_assigns = roadmap
            .portal_card_assignments
            .iter()
            .filter(|a| a.portal_tab_id == *tab_id)
            .collect::<Vec<_>>();
        let tab_sections = roadmap
            .portal_sections
            .iter()
            .filter(|s| s.portal_tab_id == *tab_id)
            .collect::<Vec<_>>();

        if tab_assigns.is_empty() {
            continue;
        }

        let watched_cards = url_saved_roadmap.cards.entry(tab_id.clone()).or_default();

        watched_cards.reserve_exact(tab_assigns.len());

        for ass in tab_assigns {
            let section_pos = tab_sections
                .iter()
                .find(|s| s.id == ass.portal_section_id)
                .expect("Has Section")
                .position;
            let card_index = roadmap
                .portal_cards
                .binary_search_by_key(&ass.portal_card_id, |c| c.id.clone())
                .expect("Card exists");
            let mut card = roadmap.portal_cards[card_index].clone();
            card.description = clean_description(card.description);
            card.section_position = Some(section_pos);
            card.card_position = Some(ass.position);
            card.assign_db_id = None;
            card.tab_id = None;

            watched_cards.push(card);
        }

        watched_cards.sort_by_key(|c| c.id.clone());
    }

    url_saved_roadmap
}

fn compare_roadmaps(previous: &Roadmap, current: &Roadmap) -> Vec<RChange> {
    info!("Started roadmap comparison");

    let mut all_changes: Vec<RChange> = Vec::new();

    // Check tabs for changes
    for (index, tab) in previous.tabs.iter().enumerate() {
        if !current.tabs.iter().any(|t| t.id == tab.id) {
            all_changes.push(RChange::TabRemoved {
                tab_index: index.try_into().unwrap(),
            });
        } else {
            all_changes.push(RChange::TabUnchanged {
                tab_index: index.try_into().unwrap(),
            });
        }
    }
    for (index, tab) in current.tabs.iter().enumerate() {
        if !previous.tabs.iter().any(|t| t.id == tab.id) {
            all_changes.push(RChange::TabAdded {
                tab_index: index.try_into().unwrap(),
            });
        }
    }
    info!("Finished tab comparison");

    // Check previous cards for removals and modifications
    for (k, cards) in previous.cards.iter() {
        if !current.cards.contains_key(k) {
            // The tab was removed one way or another and will be caught by the tab check
            let tab_index = previous
                .tabs
                .iter()
                .enumerate()
                .find_map(|(index, t)| (t.id == *k).then_some(index))
                .unwrap();

            all_changes.push(RChange::TabCardsNotInCurrent {
                tab_index: tab_index.try_into().unwrap(),
            });
            continue;
        }

        let other_cards = current.cards.get(k).unwrap();
        cards.iter().enumerate().for_each(|(index, card)| {
            if let Ok(c_index) =
                other_cards.binary_search_by(|other_card| other_card.id.cmp(&card.id))
            {
                let _current = &other_cards[c_index];

                let title = card.name != _current.name;
                let description = card.description != _current.description;
                let image = card.image_url != _current.image_url;

                if title || description || image {
                    all_changes.push(RChange::CardModified {
                        tab_id: k.clone(),
                        previous_card_index: index.try_into().unwrap(),
                        current_card_index: c_index.try_into().unwrap(),
                    });
                } else {
                    all_changes.push(RChange::CardUnchanged {
                        tab_id: k.clone(),
                        card_index: index.try_into().unwrap(),
                    });
                }
            } else {
                all_changes.push(RChange::CardRemoved {
                    tab_id: k.clone(),
                    card_index: index.try_into().unwrap(),
                });
            }
        });
    }

    info!("Finished Previous comparison");

    // Check current cards for additions (including whole tab)
    for (k, cards) in current.cards.iter() {
        if !previous.cards.contains_key(k) {
            let tab_index = current
                .tabs
                .iter()
                .enumerate()
                .find_map(|(index, t)| (t.id == *k).then_some(index))
                .unwrap();

            all_changes.push(RChange::TabCardsNotInPrevious {
                tab_index: tab_index.try_into().unwrap(),
            });
            continue;
        }

        let other_cards = previous.cards.get(k).unwrap();
        cards.iter().enumerate().for_each(|(index, card)| {
            if other_cards
                .binary_search_by(|other_card| other_card.id.cmp(&card.id))
                .is_err()
            {
                all_changes.push(RChange::CardAdded {
                    tab_id: k.clone(),
                    card_index: (index).try_into().unwrap(),
                });
            }
        });
    }

    info!("Finished current comparisons");

    info!("Finished comparisons");

    all_changes
}

async fn save_card_tx(db: &Transaction, card: &RCard) -> anyhow::Result<u32> {
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

    let r = result.next().await?.unwrap();

    Ok(r.get(0)?)
}

async fn save_tab_tx(db: &Transaction, tab: &RTab) -> anyhow::Result<u32> {
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

async fn new_roadmap_tx(db: &Transaction) -> anyhow::Result<u32> {
    let mut result = db
        .query(
            &format!(
                "INSERT INTO {R_ACTIVITIES_T} 
                    (timestamp) 
                VALUES
                    (?1)
                RETURNING id
                "
            ),
            [serde_json::to_string(&OffsetDateTime::now_utc()).unwrap()],
        )
        .await?;

    let r = result.next().await?.unwrap();

    Ok(r.get(0)?)
}

/// Save card assignment
/// * `assign_ids` - activity, tab, card, section_pos, card_pos
async fn save_card_assignment_tx(
    db: &Transaction,
    card_id: u32,
    assign_info: &[u32; 4],
) -> anyhow::Result<()> {
    let _result = db
        .execute(
            &format!(
                "INSERT INTO {R_CARD_ASSIGNS_T} 
                    (activity_id, tab_id, card_id, section_position, card_position, timestamp) 
                VALUES 
                    (?1,?2,?3,?4,?5,?6)
                "
            ),
            (
                assign_info[0],
                assign_info[1],
                card_id,
                assign_info[2],
                assign_info[3],
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    Ok(())
}

async fn save_card_and_assignment(
    db: &Transaction,
    card: &RCard,
    assign_info: &[u32; 4],
) -> anyhow::Result<u32> {
    let card_id = save_card_tx(db, card).await?;
    save_card_assignment_tx(db, card_id, assign_info).await?;
    Ok(card_id)
}

/// Save tab assignment
/// * `assign_ids` - activity, tab, card, section_pos, card_pos
async fn save_tab_assignment_tx(
    db: &Transaction,
    activity_id: u32,
    tab_db_id: u32,
) -> anyhow::Result<()> {
    let _result = db
        .execute(
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
        .await?;

    Ok(())
}

async fn save_tab_and_assignment(
    db: &Transaction,
    tab: &RTab,
    activity_id: u32,
) -> anyhow::Result<u32> {
    let tab_id = save_tab_tx(db, tab)
        .await
        .map_err(|e| StringError(format!("Failed to save tab {}", e)))?;

    save_tab_assignment_tx(db, activity_id, tab_id)
        .await
        .map_err(|e| StringError(format!("Failed to save assignment {}", e)))?;

    Ok(tab_id)
}

/// Save change
/// * `change_info` - previous_card, current_card, tab
async fn save_change_tx(
    db: &Transaction,
    change_type: &str,
    activity: u32,
    change_info: &[Option<u32>; 3],
) -> anyhow::Result<()> {
    let _result = db
        .execute(
            &format!(
                "INSERT INTO {R_CHANGES_T} 
                    (type, activity_id, previous_card_id, current_card_id, tab_id, timestamp) 
                VALUES 
                    (?1,?2,?3,?4,?5,?6)
                "
            ),
            (
                change_type,
                activity,
                change_info[0],
                change_info[1],
                change_info[2],
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    Ok(())
}

const _MAX_GOING_REQUESTS: usize = 10;

async fn save_all_cards_sync_tx(
    db: &Transaction,
    roadmap: &Roadmap,
    roadmap_id: u32,
    tab_ids: &HashMap<String, u32>,
) -> anyhow::Result<()> {
    info!("Saving all cards");
    for k in roadmap.cards.keys() {
        for card in roadmap.cards.get(k).unwrap() {
            let _ = save_card_and_assignment(
                db,
                card,
                &[
                    roadmap_id,
                    *tab_ids.get(k).unwrap(),
                    card.section_position.unwrap(),
                    card.card_position.unwrap(),
                ],
            )
            .await?;
        }
    }
    info!("Finished Saving Cards");
    Ok(())
}

async fn rollback_tx(tx: Transaction) {
    if let Err(e) = tx.rollback().await {
        error!("[Check Roadmap] Failed to rollback {}", e);
    };
}

pub async fn check_roadmap(data: &AppData) {
    info!("[Check Roadmap] Starting check");
    if !cfg!(feature = "mail") {
        warn!("[Check Roadmap] will not send emails as feature is not enabled");
    }

    let start_time = OffsetDateTime::now_utc();

    let db = data.db.connect().unwrap();
    // Get Watched Tabs
    let watched_tabs_result = get_watched_tabs(db.clone()).await;
    let Ok(watched_tabs) = watched_tabs_result else {
        error!(
            "[Check Roadmap] Failed to get watched tabs. Err: {}",
            watched_tabs_result.unwrap_err()
        );
        return;
    };
    let watched_tabs = watched_tabs
        .into_iter()
        .map(|rwt| rwt.tab_id)
        .collect::<Vec<_>>();

    // Get roadmap
    let roadmap_result = get_roadmap_json().await;
    let Ok(roadmap_string) = roadmap_result else {
        error!(
            "[Check Roadmap] Request to/parsing of roadmap page failed. Err: {}",
            roadmap_result.unwrap_err()
        );
        return;
    };
    let roadmap_result: Result<WebRoadmap, _> = serde_json::from_str(&roadmap_string);
    let Ok(roadmap) = roadmap_result else {
        error!(
            "[Check Roadmap] Failed to deserialize web roadmap. Err: {}",
            roadmap_result.unwrap_err()
        );
        return;
    };
    let roadmap = web_to_saved_roadmap(roadmap, &watched_tabs);

    // Get previous roadmap
    let previous_roadmap_result = get_most_recent_roadmap(db.clone()).await;
    let Ok(previous_roadmap) = previous_roadmap_result else {
        error!(
            "[Check Roadmap] Failed to get previous roadmap. Err: {}",
            previous_roadmap_result.unwrap_err()
        );
        return;
    };

    if let Some(previous_roadmap) = previous_roadmap {
        let changes = compare_roadmaps(&previous_roadmap, &roadmap);
        let should_notify = changes.iter().any(|c| {
            matches!(
                c,
                RChange::CardAdded { .. }
                    | RChange::CardModified { .. }
                    | RChange::CardRemoved { .. }
                    | RChange::TabAdded { .. }
                    | RChange::TabRemoved { .. }
            )
        });
        let should_save = should_notify
            || changes.iter().any(|c| {
                matches!(
                    c,
                    RChange::TabCardsNotInCurrent { .. } | RChange::TabCardsNotInPrevious { .. }
                )
            });

        if !should_save {
            info!("[Check Roadmap] No Changes to save detected.");
            let now = OffsetDateTime::now_utc();
            info!(
                "[Check Roadmap] Finished checking . Started at {} finished at {} took {}",
                start_time,
                now,
                now - start_time
            );
            return;
        }

        // Save Roadmap
        let db = data.db.connect();
        let Ok(db) = db else {
            error!("[Check Roadmap] DB failed to connect {}", db.unwrap_err());
            return;
        };

        let tx = db.transaction().await;
        let Ok(tx) = tx else {
            error!(
                "[Check Roadmap] Failed to create transaction {}",
                tx.err().unwrap()
            );
            return;
        };

        let roadmap_result = new_roadmap_tx(&tx).await;
        let Ok(roadmap_id) = roadmap_result else {
            error!(
                "[Check Roadmap] Failed to save roadmap to db err: {}",
                roadmap_result.unwrap_err()
            );
            rollback_tx(tx).await;
            return;
        };

        let mut tab_ids: HashMap<String, u32> = previous_roadmap
            .tabs
            .iter()
            .map(|t| (t.id.clone(), t.db_id.unwrap()))
            .collect();

        let first_non_tab_index = changes
            .iter()
            .enumerate()
            .find_map(|(index, ch)| {
                (!matches!(
                    ch,
                    RChange::TabAdded { .. }
                        | RChange::TabRemoved { .. }
                        | RChange::TabUnchanged { .. }
                ))
                .then_some(index)
            })
            .unwrap_or(changes.len());

        let (tab_changes, card_changes) = changes.split_at(first_non_tab_index);

        for tab_change in tab_changes {
            match tab_change {
                RChange::TabAdded { tab_index } => {
                    let tab = &roadmap.tabs[*tab_index as usize];

                    // Add tab
                    let tab_result = save_tab_and_assignment(&tx, tab, roadmap_id).await;
                    let Ok(tab_id) = tab_result else {
                        error!(
                            "[Check Roadmap] Failed to save tab to db err: {}",
                            tab_result.unwrap_err()
                        );
                        rollback_tx(tx).await;
                        return;
                    };

                    tab_ids.insert(tab.id.clone(), tab_id);

                    let change_result = save_change_tx(
                        &tx,
                        "tab_added",
                        roadmap_id,
                        &[Option::<u32>::None, Option::<u32>::None, Some(tab_id)],
                    )
                    .await;
                    let Ok(_) = change_result else {
                        error!(
                            "[Check Roadmap] Failed to save tab change to db err: {}",
                            change_result.unwrap_err()
                        );
                        rollback_tx(tx).await;
                        return;
                    };
                }
                RChange::TabRemoved { tab_index } => {
                    let tab_id = previous_roadmap.tabs[*tab_index as usize].db_id.unwrap();
                    let change_result = save_change_tx(
                        &tx,
                        "tab_removed",
                        roadmap_id,
                        &[Option::<u32>::None, Option::<u32>::None, Some(tab_id)],
                    )
                    .await;
                    let Ok(_) = change_result else {
                        error!(
                            "[Save Change] unable to save tab change. err: {}",
                            change_result.unwrap_err()
                        );
                        rollback_tx(tx).await;
                        return;
                    };
                }
                RChange::TabUnchanged { tab_index } => {
                    let tab_id = previous_roadmap.tabs[*tab_index as usize].db_id.unwrap();

                    let assign_result = save_tab_assignment_tx(&tx, roadmap_id, tab_id).await;
                    let Ok(_) = assign_result else {
                        error!(
                            "[Check Roadmap] Failed to save tab assignment to db err: {}",
                            assign_result.unwrap_err()
                        );
                        rollback_tx(tx).await;
                        return;
                    };
                }
                _ => {
                    error!("[Check Roadmap] Found non tab change in tab changelist");
                }
            };
        }

        for card_change in card_changes {
            let mut previous_card_id = None;
            let mut current_card_id = None;
            let mut change_type = "";

            match card_change {
                RChange::CardUnchanged { tab_id, card_index } => {
                    let card = &previous_roadmap.cards.get(tab_id).unwrap()[*card_index as usize];

                    let assign_result = save_card_assignment_tx(
                        &tx,
                        card.db_id.unwrap(),
                        &[
                            roadmap_id,
                            *tab_ids.get(tab_id).unwrap(),
                            card.section_position.unwrap(),
                            card.card_position.unwrap(),
                        ],
                    )
                    .await;
                    let Ok(_) = assign_result else {
                        error!(
                            "[Check Roadmap] Failed to save card assignment to db err: {}",
                            assign_result.unwrap_err()
                        );
                        rollback_tx(tx).await;
                        return;
                    };
                }
                RChange::CardAdded { tab_id, card_index } => {
                    change_type = "card_added";
                    let card = &roadmap.cards.get(tab_id).unwrap()[*card_index as usize];

                    let card_result = save_card_and_assignment(
                        &tx,
                        card,
                        &[
                            roadmap_id,
                            *tab_ids.get(tab_id).unwrap(),
                            card.section_position.unwrap(),
                            card.card_position.unwrap(),
                        ],
                    )
                    .await;
                    let Ok(card_id) = card_result else {
                        error!(
                            "[Check Roadmap] Failed to save card to db err: {}",
                            card_result.unwrap_err()
                        );
                        rollback_tx(tx).await;
                        return;
                    };

                    current_card_id = Some(card_id);
                }
                RChange::CardRemoved { tab_id, card_index } => {
                    change_type = "card_removed";
                    let card = &previous_roadmap.cards.get(tab_id).unwrap()[*card_index as usize];
                    previous_card_id = Some(card.db_id.unwrap());
                }
                RChange::CardModified {
                    tab_id,
                    previous_card_index,
                    current_card_index,
                } => {
                    change_type = "card_modified";
                    let card =
                        &previous_roadmap.cards.get(tab_id).unwrap()[*previous_card_index as usize];
                    previous_card_id = Some(card.db_id.unwrap());

                    let card = &roadmap.cards.get(tab_id).unwrap()[*current_card_index as usize];
                    let card_result = save_card_and_assignment(
                        &tx,
                        card,
                        &[
                            roadmap_id,
                            *tab_ids.get(tab_id).unwrap(),
                            card.section_position.unwrap(),
                            card.card_position.unwrap(),
                        ],
                    )
                    .await;
                    let Ok(card_id) = card_result else {
                        error!(
                            "[Check Roadmap] Failed to save card to db err: {}",
                            card_result.unwrap_err()
                        );
                        rollback_tx(tx).await;
                        return;
                    };

                    current_card_id = Some(card_id);
                }
                RChange::TabAdded { .. }
                | RChange::TabRemoved { .. }
                | RChange::TabUnchanged { .. } => {
                    error!("[Check Roadmap] Found non tab change in card changelist");
                    continue;
                }
                RChange::TabCardsNotInCurrent { .. } => {
                    continue;
                }
                RChange::TabCardsNotInPrevious { .. } => {
                    // Save all cards
                    if let Err(e) =
                        save_all_cards_sync_tx(&tx, &roadmap, roadmap_id, &tab_ids).await
                    {
                        error!("[Check Roadmap] Failed to save cards. err: {}", e);
                        rollback_tx(tx).await;
                        return;
                    };

                    continue;
                }
            };

            // Save change
            if previous_card_id.is_some() || current_card_id.is_some() {
                let change_result = save_change_tx(
                    &tx,
                    change_type,
                    roadmap_id,
                    &[previous_card_id, current_card_id, Option::<u32>::None],
                )
                .await;
                let Ok(_) = change_result else {
                    error!(
                        "[Save Change] unable to save card change. err: {}",
                        change_result.unwrap_err()
                    );
                    rollback_tx(tx).await;
                    return;
                };
            }
        }

        // Finish
        if let Err(e) = tx.commit().await {
            error!("[Check Roadmap] Failed to commit {}", e);
            return;
        };

        #[cfg(feature = "mail")]
        if should_notify {
            // send email that there are changes
            let count = changes
                .iter()
                .filter(|c| {
                    matches!(
                        c,
                        RChange::CardAdded { .. }
                            | RChange::CardModified { .. }
                            | RChange::CardRemoved { .. }
                            | RChange::TabAdded { .. }
                            | RChange::TabRemoved { .. }
                    )
                })
                .count();
            let base_url = env::var("VITE_BASE_URL").unwrap_or("Missing base url".into());
            let res = crate::queries::mail::send_email(
                &format!("{} new changes on roadmap", count),
                &format!("{}/roadmap/{}", base_url, roadmap_id),
                &format!(
                    r#"<a href="{}/roadmap/{}">View changes</a>"#,
                    base_url, roadmap_id
                ),
            )
            .await;

            match res {
                Ok(success) => {
                    let status = success.status();
                    let body = success.text().await.unwrap_or("Missing Body".into());
                    if !status.is_success() {
                        error!(
                            "[Check Roadmap] Change notification email request failed with status {} and body {}",
                            status, body
                        );
                    }
                }
                Err(failure) => {
                    error!(
                        "[Check Roadmap] Change notification email failed to send with error: {}",
                        failure
                    )
                }
            }
        }
    } else {
        let db = data.db.connect();
        let Ok(db) = db else {
            error!("[Check Roadmap] DB failed to connect {}", db.unwrap_err());
            return;
        };

        let tx = db.transaction().await;
        let Ok(tx) = tx else {
            error!(
                "[Check Roadmap] Failed to create transaction {}",
                tx.err().unwrap()
            );
            return;
        };

        // save roadmap
        let roadmap_result = new_roadmap_tx(&tx).await;
        let Ok(roadmap_id) = roadmap_result else {
            error!(
                "[Check Roadmap] Failed to save roadmap to db err: {}",
                roadmap_result.unwrap_err()
            );
            rollback_tx(tx).await;
            return;
        };

        let mut tab_ids: HashMap<String, u32> = HashMap::new();

        for tab in roadmap.tabs.iter() {
            let tab_result = save_tab_and_assignment(&tx, tab, roadmap_id).await;
            let Ok(tab_id) = tab_result else {
                error!(
                    "[Check Roadmap] Failed to save tab to db err: {}",
                    tab_result.unwrap_err()
                );
                rollback_tx(tx).await;
                return;
            };

            tab_ids.insert(tab.id.clone(), tab_id);
        }

        if let Err(e) = save_all_cards_sync_tx(&tx, &roadmap, roadmap_id, &tab_ids).await {
            error!("[Check Roadmap] Failed to save cards. err: {}", e);
            rollback_tx(tx).await;
            return;
        };
        if let Err(e) = tx.commit().await {
            error!("[Check Roadmap] Failed to commit {}", e);
        };
    }

    let now = OffsetDateTime::now_utc();
    info!(
        "[Check Roadmap] Finished checking . Started at {} finished at {} took {}",
        start_time,
        now,
        now - start_time
    );
}
