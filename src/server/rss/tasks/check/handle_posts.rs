use libsql::Connection;
use log::info;
use thiserror::Error;
use time::OffsetDateTime;

use super::check_source::CheckReturn;
use crate::db::tables::{ACTIVITIES_T, SOURCES_T};
use crate::shared::DatabaseError;

#[cfg(feature = "mail")]
use crate::mail::send_email;

#[derive(Debug, Error)]
pub enum PostsError {
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),
    #[error(transparent)]
    MailEror(#[from] reqwest::Error),
}

pub async fn handle_posts(
    rss_info: CheckReturn,
    _client: reqwest::Client,
    conn: Connection,
) -> Result<(), PostsError> {
    let CheckReturn(rss_info, rss_posts) = rss_info;

    let tx = conn.transaction().await.map_err(DatabaseError::from)?;

    info!(
        "Source at {} has {} new posts",
        rss_info.source_url,
        rss_posts.len()
    );

    let _ = tx
        .execute(
            &format!("UPDATE {SOURCES_T} SET last_checked = ?1, failed_count = ?2 WHERE id = ?3"),
            (
                serde_json::to_string(&rss_info.most_recent).unwrap(),
                0,
                rss_info.source_id,
            ),
        )
        .await
        .map_err(DatabaseError::from)?;

    for post in rss_posts.into_iter().rev() {
        info!("Handling post with title {}", post.title);

        let _ = tx
            .execute(
                &format!(
                    "INSERT INTO {ACTIVITIES_T} 
						(source_id, post_url, timestamp) 
					VALUES 
						(?1, ?2, ?3)
					"
                ),
                (
                    rss_info.source_id,
                    post.url.clone(),
                    serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
                ),
            )
            .await
            .map_err(DatabaseError::from)?;

        #[cfg(feature = "mail")]
        let _ = send_email(
            &format!("{} - {}", post.title, rss_info.channel_title),
            &format!("Source: {}\n\n{}", post.url, post.body),
            &format!(
                r#"
                            <p>Source: <a href="{}">Link</a></p>
                            <p>{}</p>
                        "#,
                post.url, post.body
            ),
        )
        .await?;
    }

    tx.commit().await.map_err(DatabaseError::from)?;

    Ok(())
}
