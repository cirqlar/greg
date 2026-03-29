use std::ops::Deref;

use libsql::Connection;
use log::info;
use thiserror::Error;

use super::check_source::CheckReturn;
use crate::rss::queries::activity::add_activity;
use crate::rss::queries::sources::update_source_last_checked;
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

    let _ =
        update_source_last_checked(tx.deref(), rss_info.source_id, rss_info.most_recent).await?;

    for post in rss_posts.into_iter().rev() {
        info!("Handling post with title {}", post.title);

        let _ = add_activity(tx.deref(), rss_info.source_id, &post.url).await?;

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
