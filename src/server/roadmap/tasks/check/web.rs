use std::{collections::HashMap, env};

use libsql::Connection;
use thiserror::Error;

use crate::roadmap::queries::tabs;
use crate::roadmap::types::{Roadmap, WebRoadmap};
use crate::roadmap::utils::clean_description;

const JSON_START_LANDMARK: &str = "window.pbData";
const JSON_END_LANDMARK: &str = "</script>";

#[derive(Debug, Error)]
pub enum WebError {
    #[error("fialed to get roadmap page")]
    Get(#[from] reqwest::Error),
    #[error("{0}")]
    Data(String),
    #[error("fialed to parse roadmap json")]
    Json(#[from] serde_json::Error),
    #[error("fialed to get watched tabs")]
    WatchedTabs(#[from] crate::shared::DatabaseError),
}

async fn get_roadmap_json() -> Result<String, WebError> {
    let client = reqwest::Client::new();

    let res = client
        .get(env::var("VITE_ROADMAP_URL").expect("VITE_ROADMAP_URL exists"))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(WebError::Data(format!(
            "Response from request is not success. status: {}",
            res.status()
        )));
    }

    let mut content = res.text().await?;
    let Some(json_search_start) = content.find(JSON_START_LANDMARK) else {
        return Err(WebError::Data(format!(
            "Didn't find JSON_START_LANDMARK. landmark: {}",
            JSON_START_LANDMARK
        )));
    };

    let Some(json_start) = content[json_search_start..].find('{') else {
        return Err(WebError::Data(format!(
            "Didn't find open bracket after JSON_START_LANDMARK. landmark: {}",
            JSON_START_LANDMARK
        )));
    };

    let Some(json_search_end) = content[json_search_start..].find(JSON_END_LANDMARK) else {
        return Err(WebError::Data(format!(
            "Didn't find JSON_END_LANDMARK. landmar: {}",
            JSON_END_LANDMARK
        )));
    };

    let Some(json_end) =
        content[json_search_start..=json_search_start + json_search_end].rfind('}')
    else {
        return Err(WebError::Data(format!(
            "Didn't find close bracket before JSON_END_LANDMARK. landmark: {}",
            JSON_END_LANDMARK
        )));
    };

    content = content[json_search_start + json_start..=json_search_start + json_end].to_owned();
    Ok(content)
}

async fn get_web_roadmap_from_json(json: &str) -> Result<WebRoadmap, WebError> {
    serde_json::from_str(json).map_err(|e| e.into())
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

pub async fn get_web_roadmap(db: Connection) -> Result<Roadmap, WebError> {
    let roadmap_json = get_roadmap_json().await?;
    let web_roadmap = get_web_roadmap_from_json(&roadmap_json).await?;
    let watched_ids = tabs::get_watched_tabs(db)
        .await?
        .into_iter()
        .map(|t| t.tab_id)
        .collect::<Vec<_>>();

    Ok(web_to_saved_roadmap(web_roadmap, &watched_ids))
}
