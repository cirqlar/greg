use std::collections::HashMap;
use std::env;

use libsql::Transaction;
use log::{error, info, warn};
use thiserror::Error;
use time::OffsetDateTime;

use crate::AppData;
use crate::roadmap::queries::roadmap;
use crate::roadmap::types::RChange;
use crate::shared::DatabaseError;

#[cfg(feature = "mail")]
use crate::mail::send_email;

mod changes;
mod compare;
mod db;
mod new_roadmap;
mod web;

use changes::SaveOrNotify;

#[derive(Debug, Error)]
pub enum CheckRoadmapError {
    #[error("failed to fetch current roadmap")]
    Web(#[from] web::WebError),
    #[error("failed to fetch previous roadmap")]
    Prev(DatabaseError),
    #[error("failed to fetch connect to database")]
    Conn(libsql::Error),
    #[error("failed to begin/commit transaction")]
    Trans(libsql::Error),
    #[error("failed to save new roadmap")]
    NewRoadmap(DatabaseError),
    #[error("failed to save changes")]
    Changes(#[from] changes::SaveChangesError),
    #[error("failed to send email about changes")]
    Mail(#[from] reqwest::Error),
}

async fn make_transation(data: &AppData) -> Result<Transaction, CheckRoadmapError> {
    let tx = data
        .app_db
        .connect()
        .map_err(CheckRoadmapError::Conn)?
        .transaction()
        .await
        .map_err(CheckRoadmapError::Trans)?;

    Ok(tx)
}

pub async fn check_roadmap(data: &AppData) -> Result<(), CheckRoadmapError> {
    let start_time = OffsetDateTime::now_utc();
    info!("Started checking roadmap at {start_time}");

    if !cfg!(feature = "mail") {
        warn!("Will not send emails as feature is not enabled");
    }

    let db = data.app_db.connect().map_err(CheckRoadmapError::Conn)?;
    let current_roadmap = web::get_web_roadmap(db.clone()).await?;

    if let Some(previous_roadmap) = roadmap::get_most_recent_roadmap(db.clone())
        .await
        .map_err(CheckRoadmapError::Prev)?
    {
        let changes = compare::compare_roadmaps(&previous_roadmap, &current_roadmap);
        let notify_count = changes.iter().filter(|c| c.should_notify()).count();
        let should_save = notify_count > 0 || changes.iter().any(|c| c.should_save());

        if !should_save {
            info!("No Changes to save.");
            let end_time = OffsetDateTime::now_utc();
            info!(
                "Finished checking roadmap at {} took {}",
                end_time,
                end_time - start_time
            );
            return Ok(());
        }

        let tx = make_transation(data).await?;
        let new_roadmap_id = db::roadmap::new_activity_tx(&tx)
            .await
            .map_err(changes::SaveChangesError::DatabaseError)?;

        let mut tab_ids: HashMap<String, u32> = previous_roadmap
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

        changes::handle_tab_changes(
            &tx,
            &previous_roadmap,
            &current_roadmap,
            new_roadmap_id,
            tab_changes,
            &mut tab_ids,
        )
        .await?;

        changes::handle_card_changes(
            &tx,
            &previous_roadmap,
            &current_roadmap,
            new_roadmap_id,
            card_changes,
            &tab_ids,
        )
        .await?;

        tx.commit().await.map_err(CheckRoadmapError::Trans)?;

        // TODO: Should failing email rollback db?
        // It currently does for rss but in that case the email is **the** point
        // if the email fails here might be better to have it.
        // Or maybe crash the program?
        #[cfg(feature = "mail")]
        if notify_count > 0 {
            // send email that there are changes

            let base_url = env::var("VITE_BASE_URL").unwrap_or("Missing base url".into());
            let res = send_email(
                &format!("{notify_count} new changes on roadmap"),
                &format!("{base_url}/roadmap/{new_roadmap_id}"),
                &format!(r#"<a href="{base_url}/roadmap/{new_roadmap_id}">View changes</a>"#),
            )
            .await?;

            // TODO: should return failed request as error.
            if !res.status().is_success() {
                let status = res.status();
                let body = res.text().await.unwrap_or("Missing Body".into());
                if !status.is_success() {
                    error!(
                        "Change notification email request failed with status {status} and body {body}"
                    );
                }
            }
        }
    } else {
        let tx = make_transation(data).await?;

        new_roadmap::save_new_roadmap(&tx, current_roadmap)
            .await
            .map_err(CheckRoadmapError::NewRoadmap)?;

        tx.commit().await.map_err(CheckRoadmapError::Trans)?;
    }

    let end_time = time::OffsetDateTime::now_utc();
    info!(
        "Finished checking roadmap at {} took {}",
        end_time,
        end_time - start_time
    );
    Ok(())
}
