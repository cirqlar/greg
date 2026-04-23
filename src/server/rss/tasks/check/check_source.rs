use feed_rs::{model::Feed, parser};
use log::{error, info, warn};
use thiserror::Error;
use time::OffsetDateTime;

use crate::rss::Source;

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

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use time::{ext::NumericalDuration, format_description::well_known};

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn check_source_returns_only_posts_past_time() -> Result<(), CheckError> {
        let mut server = mockito::Server::new_async().await;
        let m1 = server.mock("GET", "/").with_body(format!(
            r##"
                <rss xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0">
                    <channel>
                        <title>Test Source</title>
                        <link/>
                        <description/>
                        <updated>{pub_date_1}</updated>
                        <atom:link href="https://test_source.com" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>Item 1</title>
                            <link>https://test_source.com/item_1</link>
                            <summary>Item 1 summary</summary>
                            <description>Item 1 description</description>
                            <category>Dummy category</category>
                            <guid>https://test_source.com/item_1</guid>
                            <dc:creator>Source creator</dc:creator>
                            <pubDate>{pub_date_1}</pubDate>
                            <image>https://test_source.com/item_1.jpg</image>
                        </item>
                        <item>
                            <title>Item 2</title>
                            <link>https://test_source.com/item_2</link>
                            <summary>Item 2 summary</summary>
                            <description>Item 2 description</description>
                            <category>Dummy category</category>
                            <guid>https://test_source.com/item_2</guid>
                            <dc:creator>Source creator</dc:creator>
                            <pubDate>{pub_date_2}</pubDate>
                            <image>https://test_source.com/item_2.jpg</image>
                        </item>
                    </channel>
                </rss>
            "##, 
            pub_date_1 = (OffsetDateTime::now_utc() - 1.hours()).format(&well_known::Rfc2822).expect("can format datetime"),
            pub_date_2 = (OffsetDateTime::now_utc() - 3.hours()).format(&well_known::Rfc2822).expect("can format datetime")
        )).create_async().await;

        let source = Source {
            id: 0,
            url: server.url(),
            last_checked: (OffsetDateTime::now_utc() - 2.hours()),
            enabled: true,
            failed_count: 0,
        };

        let ret = check_source(&source, reqwest::Client::new()).await?;

        assert!(ret.is_some());

        let CheckReturn(info, posts) = ret.unwrap();

        assert_eq!(info.source_id, source.id);
        assert_eq!(info.source_url, source.url);
        assert_eq!(info.channel_title.as_str(), "Test Source");
        let difference = OffsetDateTime::now_utc() - info.most_recent;
        let compared_to_actual = difference - 1.hours();
        assert!(compared_to_actual.abs() < 5.minutes());

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title.as_str(), "Item 1");

        m1.assert_async().await;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn check_source_returns_none_when_all_posts_are_too_old() -> Result<(), CheckError> {
        let mut server = mockito::Server::new_async().await;
        let m1 = server.mock("GET", "/").with_body(format!(
            r##"
                <rss xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0">
                    <channel>
                        <title>Test Source</title>
                        <link/>
                        <description/>
                        <updated>{pub_date_1}</updated>
                        <atom:link href="https://test_source.com" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>Item 1</title>
                            <link>https://test_source.com/item_1</link>
                            <summary>Item 1 summary</summary>
                            <description>Item 1 description</description>
                            <category>Dummy category</category>
                            <guid>https://test_source.com/item_1</guid>
                            <dc:creator>Source creator</dc:creator>
                            <pubDate>{pub_date_1}</pubDate>
                            <image>https://test_source.com/item_1.jpg</image>
                        </item>
                        <item>
                            <title>Item 2</title>
                            <link>https://test_source.com/item_2</link>
                            <summary>Item 2 summary</summary>
                            <description>Item 2 description</description>
                            <category>Dummy category</category>
                            <guid>https://test_source.com/item_2</guid>
                            <dc:creator>Source creator</dc:creator>
                            <pubDate>{pub_date_2}</pubDate>
                            <image>https://test_source.com/item_2.jpg</image>
                        </item>
                    </channel>
                </rss>
            "##, 
            pub_date_1 = (OffsetDateTime::now_utc() - 125.minutes()).format(&well_known::Rfc2822).expect("can format datetime"),
            pub_date_2 = (OffsetDateTime::now_utc() - 3.hours()).format(&well_known::Rfc2822).expect("can format datetime")
        )).create_async().await;

        let source = Source {
            id: 0,
            url: server.url(),
            last_checked: (OffsetDateTime::now_utc() - 1.hours()),
            enabled: true,
            failed_count: 0,
        };

        let ret = check_source(&source, reqwest::Client::new()).await?;

        assert!(ret.is_none());

        m1.assert_async().await;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn check_source_returns_none_for_disabled_source() -> Result<(), CheckError> {
        let mut server = mockito::Server::new_async().await;
        let m1 = server.mock("GET", "/").with_body(
            r##"
                <rss xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0">
                    <channel>
                        <title>Test Source</title>
                        <link/>
                        <description/>
                        <atom:link href="https://test_source.com" rel="self" type="application/rss+xml"/>
                    </channel>
                </rss>
            "## 
        ).create_async().await.expect(0);

        let source = Source {
            id: 0,
            url: server.url(),
            last_checked: (OffsetDateTime::now_utc() - 2.hours()),
            enabled: false,
            failed_count: 0,
        };

        let ret = check_source(&source, reqwest::Client::new()).await?;

        assert!(ret.is_none());

        m1.assert_async().await;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn check_source_can_handle_multiple_links() -> Result<(), CheckError> {
        let mut server = mockito::Server::new_async().await;
        let m1 = server.mock("GET", "/").with_body(format!(
            r##"
                <rss xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0">
                    <channel>
                        <title>Test Source</title>
                        <link/>
                        <description/>
                        <atom:link href="https://test_source.com" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>Item 1</title>
                            <link>https://test_source.com/item_1</link>
                            <link>https://test_source.com/item_1_alt</link>
                            <summary>Item 1 summary</summary>
                            <description>Item 1 description</description>
                            <category>Dummy category</category>
                            <guid>https://test_source.com/item_1</guid>
                            <dc:creator>Source creator</dc:creator>
                            <pubDate>{pub_date_1}</pubDate>
                            <image>https://test_source.com/item_1.jpg</image>
                        </item>
                    </channel>
                </rss>
            "##, 
            pub_date_1 = (OffsetDateTime::now_utc() - 1.hours()).format(&well_known::Rfc2822).expect("can format datetime")
        )).create_async().await;

        let source = Source {
            id: 0,
            url: server.url(),
            last_checked: (OffsetDateTime::now_utc() - 2.hours()),
            enabled: true,
            failed_count: 0,
        };

        let ret = check_source(&source, reqwest::Client::new()).await?;

        assert!(ret.is_some());

        let CheckReturn(info, posts) = ret.unwrap();

        assert_eq!(info.source_id, source.id);
        assert_eq!(info.source_url, source.url);
        assert_eq!(info.channel_title.as_str(), "Test Source");
        let difference = OffsetDateTime::now_utc() - info.most_recent;
        let compared_to_actual = difference - 1.hours();
        assert!(compared_to_actual.abs() < 5.minutes());

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title.as_str(), "Item 1");
        assert_eq!(posts[0].url.as_str(), "https://test_source.com/item_1");

        m1.assert_async().await;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn check_source_can_handle_no_links() -> Result<(), CheckError> {
        let mut server = mockito::Server::new_async().await;
        let m1 = server.mock("GET", "/").with_body(format!(
            r##"
                <rss xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0">
                    <channel>
                        <title>Test Source</title>
                        <link/>
                        <description/>
                        <atom:link href="https://test_source.com" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>Item 1</title>
                            <summary>Item 1 summary</summary>
                            <description>Item 1 description</description>
                            <category>Dummy category</category>
                            <guid>https://test_source.com/item_1</guid>
                            <dc:creator>Source creator</dc:creator>
                            <pubDate>{pub_date_1}</pubDate>
                            <image>https://test_source.com/item_1.jpg</image>
                        </item>
                    </channel>
                </rss>
            "##, 
            pub_date_1 = (OffsetDateTime::now_utc() - 1.hours()).format(&well_known::Rfc2822).expect("can format datetime")
        )).create_async().await;

        let source = Source {
            id: 0,
            url: server.url(),
            last_checked: (OffsetDateTime::now_utc() - 2.hours()),
            enabled: true,
            failed_count: 0,
        };

        let ret = check_source(&source, reqwest::Client::new()).await?;

        assert!(ret.is_some());

        let CheckReturn(info, posts) = ret.unwrap();

        assert_eq!(info.source_id, source.id);
        assert_eq!(info.source_url, source.url);
        assert_eq!(info.channel_title.as_str(), "Test Source");
        let difference = OffsetDateTime::now_utc() - info.most_recent;
        let compared_to_actual = difference - 1.hours();
        assert!(compared_to_actual.abs() < 5.minutes());

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title.as_str(), "Item 1");
        assert_eq!(posts[0].url.as_str(), "No Url");

        m1.assert_async().await;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn check_source_can_handle_no_summary() -> Result<(), CheckError> {
        let mut server = mockito::Server::new_async().await;
        let m1 = server.mock("GET", "/").with_body(format!(
            r##"
                <rss xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0">
                    <channel>
                        <title>Test Source</title>
                        <link/>
                        <description/>
                        <updated>{pub_date_1}</updated>
                        <atom:link href="https://test_source.com" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>Item 1</title>
                            <link>https://test_source.com/item_1</link>
                            <description>Item 1 description</description>
                            <category>Dummy category</category>
                            <guid>https://test_source.com/item_1</guid>
                            <dc:creator>Source creator</dc:creator>
                            <pubDate>{pub_date_1}</pubDate>
                            <image>https://test_source.com/item_1.jpg</image>
                        </item>
                    </channel>
                </rss>
            "##, 
            pub_date_1 = (OffsetDateTime::now_utc() - 1.hours()).format(&well_known::Rfc2822).expect("can format datetime")
        )).create_async().await;

        let source = Source {
            id: 0,
            url: server.url(),
            last_checked: (OffsetDateTime::now_utc() - 2.hours()),
            enabled: true,
            failed_count: 0,
        };

        let ret = check_source(&source, reqwest::Client::new()).await?;

        assert!(ret.is_some());

        let CheckReturn(info, posts) = ret.unwrap();

        assert_eq!(info.source_id, source.id);
        assert_eq!(info.source_url, source.url);
        assert_eq!(info.channel_title.as_str(), "Test Source");
        let difference = OffsetDateTime::now_utc() - info.most_recent;
        let compared_to_actual = difference - 1.hours();
        assert!(compared_to_actual.abs() < 5.minutes());

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title.as_str(), "Item 1");
        assert_eq!(posts[0].body.as_str(), "Item 1 description");

        m1.assert_async().await;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn check_source_can_handle_malformed_feed() -> Result<(), CheckError> {
        let mut server = mockito::Server::new_async().await;
        let m1 = server.mock("GET", "/").with_body(format!(
            r##"
                <rss xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0">
                    <channel>
                        <title>Test Source</title>
                        <link/>
                        <description/>
                        <updated>{pub_date_1}</updated>
                        <atom:link href="https://test_source.com" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>Item 1</title>
                            <link>https://test_source.com/item_1</link>
            "##, 
            pub_date_1 = (OffsetDateTime::now_utc() - 1.hours()).format(&well_known::Rfc2822).expect("can format datetime")
        )).create_async().await;

        let source = Source {
            id: 0,
            url: server.url(),
            last_checked: (OffsetDateTime::now_utc() - 2.hours()),
            enabled: true,
            failed_count: 0,
        };

        let ret = check_source(&source, reqwest::Client::new()).await;

        assert!(ret.is_err());
        assert!(matches!(ret, Err(CheckError::Parse(_))));

        m1.assert_async().await;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn check_source_can_handle_failed_request() -> Result<(), CheckError> {
        let mut server = mockito::Server::new_async().await;
        let m1 = server
            .mock("GET", "/")
            .with_status(500)
            .create_async()
            .await;

        let source = Source {
            id: 0,
            url: server.url(),
            last_checked: (OffsetDateTime::now_utc() - 2.hours()),
            enabled: true,
            failed_count: 0,
        };

        let ret = check_source(&source, reqwest::Client::new()).await;

        assert!(ret.is_err());
        assert!(matches!(ret, Err(CheckError::Source(_))));

        m1.assert_async().await;

        Ok(())
    }
}
