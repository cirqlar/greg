use feed_rs::{model::Feed, parser};
use log::{error, info, warn};
use thiserror::Error;
use time::OffsetDateTime;

use crate::server::rss::Source;

#[derive(Error, Debug)]
pub enum CheckError {
    #[error("client failed to get source")]
    Client(reqwest::Error),
    #[error("request to source failed")]
    Source(reqwest::Response),
    #[error("bytes failed")]
    Bytes(reqwest::Error),
    #[error("Failed to parse source")]
    Parse(#[from] feed_rs::parser::ParseFeedError),
}

pub struct RssPost {
    pub title: String,
    pub url: String,
    pub body: String,
}

pub struct RssInfo {
    pub source_id: u32,
    pub source_url: String,
    pub channel_title: String,
    pub most_recent: OffsetDateTime,
}

pub struct CheckReturn(pub RssInfo, pub Vec<RssPost>);

const CHECK_BUFFER_IN_MINUTES: i64 = 5;

pub async fn get_source(url: &str, client: reqwest::Client) -> Result<Feed, CheckError> {
    let res = client.get(url).send().await.map_err(CheckError::Client)?;

    if !res.status().is_success() {
        return Err(CheckError::Source(res));
    }

    let content = res.bytes().await.map_err(CheckError::Bytes)?;

    let channel = parser::parse(&(content)[..])?;

    Ok(channel)
}

pub async fn check_source(
    source: &Source,
    client: reqwest::Client,
) -> Result<Option<CheckReturn>, CheckError> {
    if !source.enabled {
        info!("Skipping disabled source {}", source.url);
        return Ok(None);
    }

    let channel = get_source(&source.url, client).await?;

    if let Some(a) = channel.updated
        && let Ok(upd_time) = OffsetDateTime::from_unix_timestamp(a.timestamp())
    {
        if (upd_time - source.last_checked).whole_minutes() < -CHECK_BUFFER_IN_MINUTES {
            info!(
                "Source {}, hasn't been updated since last_check {}, upd_time {}",
                &source.url, source.last_checked, upd_time
            );
            return Ok(None);
        }
    } else {
        warn!(
            "[Check Sources] Source at {} has no updated date",
            &source.url
        );
    }

    let mut most_recent = None;
    let mut entries = Vec::new();

    for entry in channel.entries {
        let content_url: String;
        if let Some(x) = entry.links.iter().find(|link| {
            if let Some(ref rel) = link.rel
                && let Some(_) = (rel == "alternate" || rel == "self").then_some(())
                && let Some(ref med_t) = link.media_type
                && let Some(_) = (med_t == "text/html").then_some(())
            {
                true
            } else {
                false
            }
        }) {
            content_url = x.href.clone();
        } else if entry.links.len() == 1 {
            content_url = entry.links[0].href.clone();
        } else if !entry.links.is_empty() {
            content_url = entry.links[0].href.clone();
            warn!("[Check Sources] Using first url for entry {content_url}");
        } else if let Some(ref content) = entry.content
            && let Some(ref url) = content.src
        {
            content_url = url.href.clone();
            warn!("[Check Sources] Using content url for entry {content_url}");
        } else {
            content_url = "No Url".into();
        }

        let pub_time = if let Some(ref pub_) = entry.published
            && let Ok(pub_time) = OffsetDateTime::from_unix_timestamp(pub_.timestamp())
        {
            pub_time
        } else {
            error!("[Check Sources] Issue parsing published for post at {content_url}");
            break;
        };

        if pub_time <= source.last_checked {
            warn!(
                "[Check Sources] Last post checked at url {content_url} was published {pub_time}"
            );
            break;
        }
        if most_recent.is_none() {
            most_recent = Some(pub_time);
        }

        let content_title = entry
            .title
            .as_ref()
            .map_or_else(|| "Missing Content Title".into(), |t| t.content.clone());

        let content_body = if let Some(ref summary) = entry.summary
            && let Some(_) = (!summary.content.trim().is_empty()).then_some(())
        {
            summary.content.clone()
        } else if let Some(ref content) = entry.content
            && let Some(ref body) = content.body
            && let Some(_) = (!body.trim().is_empty()).then_some(())
        {
            body.clone()
        } else {
            "No body".into()
        };

        entries.push(RssPost {
            title: content_title,
            url: content_url,
            body: content_body,
        });
    }

    if entries.is_empty() {
        warn!("Source at {} was checked but had no new posts", &source.url);
        Ok(None)
    } else {
        let info = RssInfo {
            source_id: source.id,
            source_url: source.url.clone(),
            channel_title: channel
                .title
                .map_or_else(|| "Missing Channel Title".into(), |t| t.content),
            most_recent: most_recent.unwrap_or_else(OffsetDateTime::now_utc),
        };

        Ok(Some(CheckReturn(info, entries)))
    }
}
