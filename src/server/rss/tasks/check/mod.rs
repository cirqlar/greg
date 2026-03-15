use log::{error, info, warn};
use thiserror::Error;
use tokio::task::JoinSet;

mod check_source;
mod handle_check_failure;
mod handle_posts;

pub use check_source::get_source;

use crate::server::{AppData, rss::queries::sources::get_sources};

#[derive(Error, Debug)]
pub enum SubTaskError {
    #[error(transparent)]
    Check(#[from] check_source::CheckError),
    #[error(transparent)]
    Posts(#[from] handle_posts::PostsError),
}

#[derive(Error, Debug)]
pub enum CheckRssError {
    #[error(transparent)]
    Database(#[from] crate::server::shared::DatabaseError),
    #[error("{} sources failed", .0.len())]
    Source(Vec<SubTaskError>),
}

pub async fn check_rss(data: &AppData) -> Result<(), CheckRssError> {
    let start_time = time::OffsetDateTime::now_utc();
    info!("Started checking sources at {start_time}");

    if !cfg!(feature = "mail") {
        warn!("Will not send emails as feature is not enabled");
    }

    let sources = get_sources(data.app_db.connect().unwrap()).await?;

    let mut threads = JoinSet::new();
    let client = reqwest::Client::new();
    let conn = data.app_db.connect().unwrap();

    for source in sources {
        let s_client = client.clone();
        let s_conn = conn.clone();

        threads.spawn(async move {
            let res = check_source::check_source(&source, s_client.clone()).await;
            if let Err(e) = res {
                error!("Checking source at {} failed. err: {:?}", &source.url, e);

                let _ = handle_check_failure::handle_check_failure(
                    &source,
                    &e,
                    s_conn,
                    s_client.clone(),
                )
                .await;
                return Err(SubTaskError::from(e));
            }

            let Some(activity) = res? else {
                return Ok(());
            };

            let res = handle_posts::handle_posts(activity, s_client, s_conn).await;
            if let Err(e) = res {
                error!(
                    "Handling posts for source at {} failed. err: {:?}",
                    &source.url, e
                );

                return Err(SubTaskError::from(e));
            }

            Ok(())
        });
    }

    let threads = threads.join_all().await;

    let end_time = time::OffsetDateTime::now_utc();
    info!(
        "Finished checking sources at {} took {}",
        end_time,
        end_time - start_time
    );

    let sub_task_errors = threads
        .into_iter()
        .filter_map(|r| r.err())
        .collect::<Vec<_>>();

    if sub_task_errors.is_empty() {
        Ok(())
    } else {
        Err(CheckRssError::Source(sub_task_errors))
    }
}
