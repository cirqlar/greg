use std::env;

use libsql::Connection;
use log::error;
use thiserror::Error;

use super::check_source::CheckError;
use crate::db::tables::SOURCES_T;
use crate::rss::Source;
use crate::shared::DatabaseError;

#[cfg(feature = "mail")]
use crate::mail::send_email_with_cient;

#[derive(Debug, Error)]
pub enum HandleFailureError {
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),
    #[error(transparent)]
    MailError(#[from] reqwest::Error),
}

pub async fn handle_check_failure(
    source: &Source,
    check_error: &CheckError,
    conn: Connection,
    client: reqwest::Client,
) -> Result<(), HandleFailureError> {
    let failed_count = source.failed_count + 1;
    let enabled = if failed_count
        >= env::var("SOURCE_DISABLE_AFTER").map_or(10, |v| v.parse().unwrap_or(10))
    {
        0
    } else {
        1
    };

    let _ = conn
        .execute(
            &format!("UPDATE {SOURCES_T} SET failed_count = ?1, enabled = ?2 WHERE id = ?3"),
            (failed_count, enabled, source.id),
        )
        .await
        .map_err(DatabaseError::from)?;

    if enabled == 0 {
        error!(
            "Disabling ource at {} because has failed {} times. Error: {:?}",
            source.url, failed_count, check_error
        );

        #[cfg(feature = "mail")]
		let _ = send_email_with_cient(
			client,
			"Source disabled",
			&format!(
				"The source at {source_url} has been disabled after failing too much. The error is {reason}", 
				source_url = source.url, reason = check_error
			),
			&format!(r#"
				<p>Url: {source_url}</p>
				<p>Link: <a href="{source_url}">Link</a></p>
				<p>Reason:</p><pre>{reason}</pre>
			"#, source_url = source.url, reason = check_error),
		).await?;

        Ok(())
    } else {
        error!(
            "[Check Sources]:[Handle Activity] Source at {source_url} has failed {failed_count} times",
            source_url = source.url
        );
        Ok(())
    }
}
