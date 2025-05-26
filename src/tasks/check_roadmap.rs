use std::{collections::HashMap, env, sync::Arc};

use libsql::Connection;
use log::{error, info, warn};
use time::OffsetDateTime;
use tokio::task::JoinSet;

use crate::{
    db::{R_ACTIVITIES_T, R_CARD_ASSIGNS_T, R_CARDS_T, R_CHANGES_T, R_TAB_ASSIGNS_T, R_TABS_T},
    queries::roadmap::{get_most_recent_roadmap, get_watched_tabs},
    types::{AppData, RCard, RChange, RTab, Roadmap, StringError, WebRoadmap},
};

pub async fn get_roadmap_json() -> anyhow::Result<String> {
    let client = reqwest::Client::new();

    let res = client
        .get(env::var("ROADMAP_URL").expect("ROADMAP_URL exists"))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(StringError(format!("Request code is {}", res.status())).into());
    }

    let mut content = res.text().await?;
    let Some(pb_start) = content.find("window.pbData") else {
        return Err(StringError("Didn't find pbData substring".into()).into());
    };
    let Some(open_bracket) = content[pb_start..].find('{') else {
        return Err(StringError("Didn't find open bracket".into()).into());
    };
    let Some(end_script) = content[pb_start..].find("</script>") else {
        return Err(StringError("Didn't find </script>".into()).into());
    };
    let Some(close_bracket) = content[pb_start..=pb_start + end_script].rfind('}') else {
        return Err(StringError("Didn't find close_bracket".into()).into());
    };

    content = content[pb_start + open_bracket..=pb_start + close_bracket].to_owned();
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

fn compare_roadmaps_sync(previous: &Roadmap, current: &Roadmap) -> (Vec<RChange>, bool, bool) {
    info!("Started comparison");

    let mut all_changes: Vec<RChange> = Vec::new();
    let mut should_notify = false;
    let mut should_save = false;

    // Check tabs for changes
    for (index, tab) in previous.tabs.iter().enumerate() {
        if !current.tabs.iter().any(|t| t.id == tab.id) {
            should_notify = true;
            should_save = true;

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
            should_notify = true;
            should_save = true;

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

            should_save = true;
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
                    should_notify = true;
                    should_save = true;
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
                should_notify = true;
                should_save = true;
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

            should_save = true;
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
                should_notify = true;
                should_save = true;
                all_changes.push(RChange::CardAdded {
                    tab_id: k.clone(),
                    card_index: (index).try_into().unwrap(),
                });
            }
        });
    }

    info!("Finished current comparisons");

    info!("Finished comparisons");
    (all_changes, should_notify, should_save)
}

async fn save_card(db: Connection, card: &RCard) -> anyhow::Result<u32> {
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

async fn save_tab(db: Connection, tab: &RTab) -> anyhow::Result<u32> {
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

async fn new_roadmap(db: Connection) -> anyhow::Result<u32> {
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
async fn save_card_assignment(db: Connection, assign_info: &[u32; 5]) -> anyhow::Result<()> {
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
                assign_info[2],
                assign_info[3],
                assign_info[4],
                serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
            ),
        )
        .await?;

    Ok(())
}

/// Save tab assignment
/// * `assign_ids` - activity, tab, card, section_pos, card_pos
async fn save_tab_assignment(
    db: Connection,
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

/// Save change
/// * `change_info` - previous_card, current_card, tab
async fn save_change(
    db: Connection,
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

/// Faster but not by much (at ten)
/// Hangs sometimes, not sure why
/// So disabled for now
async fn _save_all_cards(
    db: Connection,
    roadmap: Arc<Roadmap>,
    roadmap_id: u32,
    tab_ids: Arc<HashMap<String, u32>>,
) {
    info!("Saving all cards");

    for k in roadmap.cards.keys() {
        let card_count = roadmap.cards.get(k).unwrap().len();
        let mut index_iter = 0..card_count;

        let mut count = 0;
        loop {
            let mut set = JoinSet::new();
            let mut finished = false;

            while count < _MAX_GOING_REQUESTS && !finished {
                if let Some(index) = index_iter.next() {
                    let ndb = db.clone();
                    let key = k.clone();
                    let nroadmap = roadmap.clone();
                    let ntab_ids = tab_ids.clone();

                    set.spawn(async move {
                        let c = &nroadmap.cards.get(&key).unwrap()[index];
                        let card_result = save_card(ndb.clone(), c).await;
                        let Ok(card_id) = card_result else {
                            return Err(StringError(format!(
                                "[Check Roadmap] Failed to save card to db err: {}",
                                card_result.unwrap_err()
                            )));
                        };

                        let assign_result = save_card_assignment(
                            ndb.clone(),
                            &[
                                roadmap_id,
                                *ntab_ids.get(&key).unwrap(),
                                card_id,
                                c.section_position.unwrap(),
                                c.card_position.unwrap(),
                            ],
                        )
                        .await;
                        let Ok(_) = assign_result else {
                            return Err(StringError(format!(
                                "[Check Roadmap] Failed to save card assignment to db err: {}",
                                assign_result.unwrap_err()
                            )));
                        };

                        Ok(())
                    });
                } else {
                    finished = true;
                }

                count += 1;
            }

            while let Some(Ok(r)) = set.join_next().await {
                match r {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{}", e.0);
                    }
                }
            }

            if finished {
                break;
            }
        }
    }

    info!("Finished Saving Cards");
}

async fn save_all_cards_sync(
    db: Connection,
    roadmap: &Roadmap,
    roadmap_id: u32,
    tab_ids: &HashMap<String, u32>,
) {
    info!("Saving all cards");
    for k in roadmap.cards.keys() {
        for card in roadmap.cards.get(k).unwrap() {
            let card_result = save_card(db.clone(), card).await;
            let Ok(card_id) = card_result else {
                error!(
                    "[Check Roadmap] Failed to save card to db err: {}",
                    card_result.unwrap_err()
                );
                return;
            };

            let assign_result = save_card_assignment(
                db.clone(),
                &[
                    roadmap_id,
                    *tab_ids.get(k).unwrap(),
                    card_id,
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
                return;
            };
        }
    }
    info!("Finished Saving Cards")
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
        let (changes, _should_notify, should_save) =
            compare_roadmaps_sync(&previous_roadmap, &roadmap);

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
        let db = data.db.connect().unwrap();
        let roadmap_result = new_roadmap(db.clone()).await;
        let Ok(roadmap_id) = roadmap_result else {
            error!(
                "[Check Roadmap] Failed to save roadmap to db err: {}",
                roadmap_result.unwrap_err()
            );
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
                    let tab_result = save_tab(db.clone(), tab).await;
                    let Ok(tab_id) = tab_result else {
                        error!(
                            "[Check Roadmap] Failed to save tab to db err: {}",
                            tab_result.unwrap_err()
                        );
                        return;
                    };

                    tab_ids.insert(tab.id.clone(), tab_id);

                    let assign_result = save_tab_assignment(db.clone(), roadmap_id, tab_id).await;
                    let Ok(_) = assign_result else {
                        error!(
                            "[Check Roadmap] Failed to save tab assignment to db err: {}",
                            assign_result.unwrap_err()
                        );
                        return;
                    };

                    let change_result = save_change(
                        db.clone(),
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
                        return;
                    };
                }
                RChange::TabRemoved { tab_index } => {
                    let tab_id = previous_roadmap.tabs[*tab_index as usize].db_id.unwrap();
                    let change_result = save_change(
                        db.clone(),
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
                        return;
                    };
                }
                RChange::TabUnchanged { tab_index } => {
                    let tab_id = previous_roadmap.tabs[*tab_index as usize].db_id.unwrap();

                    let assign_result = save_tab_assignment(db.clone(), roadmap_id, tab_id).await;
                    let Ok(_) = assign_result else {
                        error!(
                            "[Check Roadmap] Failed to save tab assignment to db err: {}",
                            assign_result.unwrap_err()
                        );
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

                    let assign_result = save_card_assignment(
                        db.clone(),
                        &[
                            roadmap_id,
                            *tab_ids.get(tab_id).unwrap(),
                            card.db_id.unwrap(),
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
                        return;
                    };
                }
                RChange::CardAdded { tab_id, card_index } => {
                    change_type = "card_added";
                    let card = &roadmap.cards.get(tab_id).unwrap()[*card_index as usize];

                    let card_result = save_card(db.clone(), card).await;
                    let Ok(card_id) = card_result else {
                        error!(
                            "[Check Roadmap] Failed to save card to db err: {}",
                            card_result.unwrap_err()
                        );
                        return;
                    };

                    let assign_result = save_card_assignment(
                        db.clone(),
                        &[
                            roadmap_id,
                            *tab_ids.get(tab_id).unwrap(),
                            card_id,
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
                    let card_result = save_card(db.clone(), card).await;
                    let Ok(card_id) = card_result else {
                        error!(
                            "[Check Roadmap] Failed to save card to db err: {}",
                            card_result.unwrap_err()
                        );
                        return;
                    };
                    let assign_result = save_card_assignment(
                        db.clone(),
                        &[
                            roadmap_id,
                            *tab_ids.get(tab_id).unwrap(),
                            card_id,
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
                        return;
                    };
                    current_card_id = Some(card_id);
                }
                RChange::TabAdded { .. }
                | RChange::TabRemoved { .. }
                | RChange::TabUnchanged { .. } => {
                    error!("[Check Roadmap] Found non tab change in tab changelist");
                    continue;
                }
                RChange::TabCardsNotInCurrent { .. } => {
                    continue;
                }
                RChange::TabCardsNotInPrevious { .. } => {
                    // Save all cards
                    save_all_cards_sync(db.clone(), &roadmap, roadmap_id, &tab_ids).await;
                    continue;
                }
            };

            // Save change
            if previous_card_id.is_some() || current_card_id.is_some() {
                let change_result = save_change(
                    db.clone(),
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
                    return;
                };
            }
        }

        #[cfg(feature = "mail")]
        if _should_notify {
            // send email that there are changes
            let res = crate::queries::mail::send_email(
                "New changes on roadmap",
                "TODO: Add url to changes",
                "TODO: Add url to changes",
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
        let db = data.db.connect().unwrap();

        // save roadmap
        let roadmap_result = new_roadmap(db.clone()).await;
        let Ok(roadmap_id) = roadmap_result else {
            error!(
                "[Check Roadmap] Failed to save roadmap to db err: {}",
                roadmap_result.unwrap_err()
            );
            return;
        };

        let mut tab_ids: HashMap<String, u32> = HashMap::new();

        for tab in roadmap.tabs.iter() {
            let tab_result = save_tab(db.clone(), tab).await;
            let Ok(tab_id) = tab_result else {
                error!(
                    "[Check Roadmap] Failed to save tab to db err: {}",
                    tab_result.unwrap_err()
                );
                return;
            };

            tab_ids.insert(tab.id.clone(), tab_id);

            let assign_result = save_tab_assignment(db.clone(), roadmap_id, tab_id).await;
            let Ok(_) = assign_result else {
                error!(
                    "[Check Roadmap] Failed to save tab assignment to db err: {}",
                    assign_result.unwrap_err()
                );
                return;
            };
        }

        save_all_cards_sync(db.clone(), &roadmap, roadmap_id, &tab_ids).await;

        // let roadmap = Arc::new(roadmap);
        // let tab_ids = Arc::new(tab_ids);

        // save_all_cards(db.clone(), roadmap, roadmap_id, tab_ids).await;
        // info!("Saving all cards");
        // for k in roadmap.cards.keys() {
        //     for card in roadmap.cards.get(k).unwrap() {
        //         let card_result = save_card(db.clone(), card).await;
        //         let Ok(card_id) = card_result else {
        //             error!(
        //                 "[Check Roadmap] Failed to save card to db err: {}",
        //                 card_result.unwrap_err()
        //             );
        //             return;
        //         };

        //         let assign_result = save_card_assignment(
        //             db.clone(),
        //             &[
        //                 roadmap_id,
        //                 *tab_ids.get(k).unwrap(),
        //                 card_id,
        //                 card.section_position.unwrap(),
        //                 card.card_position.unwrap(),
        //             ],
        //         )
        //         .await;
        //         let Ok(_) = assign_result else {
        //             error!(
        //                 "[Check Roadmap] Failed to save card assignment to db err: {}",
        //                 assign_result.unwrap_err()
        //             );
        //             return;
        //         };
        //     }
        // }
        // info!("Finished Saving Cards")
    }

    let now = OffsetDateTime::now_utc();
    info!(
        "[Check Roadmap] Finished checking . Started at {} finished at {} took {}",
        start_time,
        now,
        now - start_time
    );
}
