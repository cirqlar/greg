use std::env;

use feed_rs::parser;
use libsql::Connection;
use log::{error, info, warn};
use time::OffsetDateTime;
use tokio::task::JoinSet;
// use tokio::sync::mpsc;

use crate::{
    db::{ACTIVITIES_T, SOURCES_T},
    queries::sources::get_sources,
    types::{AppData, Source},
};

// enum Message {
//     Activity(u32, String),
//     Source(u32, OffsetDateTime),
// }

const CHECK_BUFFER_IN_MINUTES: i64 = 5;

enum SourceActivity {
    Disabled {
        source_url: String,
    },
    Failed {
        source_id: u32,
        source_url: String,
        new_failed_count: u32,
        reason: String,
    },
    Unchanged {
        source_url: String,
    },
    Changed {
        source_id: u32,
        source_url: String,
        channel_title: String,
        most_recent: OffsetDateTime,
        posts: Vec<SourceEntry>,
    },
}

struct SourceEntry {
    title: String,
    url: String,
    body: String,
}

async fn check_source(source: Source, client: reqwest::Client) -> SourceActivity {
    if !source.enabled {
        info!("[Check Sources] Skipping disabled source {}", source.url);
        return SourceActivity::Disabled {
            source_url: source.url,
        };
    }

    let res = client.get(&source.url).send().await;
    let Ok(res) = res else {
        let err = format!(
            "Network request for {} failed with err {}",
            &source.url,
            res.expect_err("must be an error")
        );
        error!("[Check Sources] {}", err);
        return SourceActivity::Failed {
            source_id: source.id,
            source_url: source.url,
            new_failed_count: source.failed_count + 1,
            reason: err,
        };
    };

    if !res.status().is_success() {
        let err = format!(
            "Network request for {} failed with status {}",
            &source.url,
            res.status()
        );
        error!("[Check Sources] {}", err);
        return SourceActivity::Failed {
            source_id: source.id,
            source_url: source.url,
            new_failed_count: source.failed_count + 1,
            reason: err,
        };
    }

    let content = res.bytes().await;
    let Ok(content) = content else {
        let err = format!(
            "Bytes failed for {} with err {}",
            &source.url,
            content.expect_err("must be an error")
        );
        error!("[Check Sources] {}", err);
        return SourceActivity::Failed {
            source_id: source.id,
            source_url: source.url,
            new_failed_count: source.failed_count + 1,
            reason: err,
        };
    };

    let channel = parser::parse(&(content)[..]);
    let Ok(channel) = channel else {
        let err = format!(
            "Parsing failed for {} with err {}",
            &source.url,
            channel.expect_err("must be an error")
        );
        error!("[Check Sources] {}", err);
        return SourceActivity::Failed {
            source_id: source.id,
            source_url: source.url,
            new_failed_count: source.failed_count + 1,
            reason: err,
        };
    };

    if let Some(a) = channel.updated
        && let Ok(upd_time) = OffsetDateTime::from_unix_timestamp(a.timestamp())
    {
        if (upd_time - source.last_checked).whole_minutes() < -CHECK_BUFFER_IN_MINUTES {
            info!(
                "[Check Sources] Source {}, hasn't been updated since last_check {}, upd_time {}",
                &source.url, source.last_checked, upd_time
            );
            return SourceActivity::Unchanged {
                source_url: source.url,
            };
        }
    // The only way return from second condition fails is a logic bug in one of the dependencies so don't bother
    //     Err(err) => {
    //         warn!(
    //             "[Check Sources] Issue parsing updated for {}, failed with err {}",
    //             &source.url, err
    //         );
    //     }
    // }
    // } else if channel.published.is_some() {
    //     match OffsetDateTime::from_unix_timestamp(channel.published.unwrap().timestamp()) {
    //         Ok(pub_time) => {
    //             if (pub_time - source.last_checked).whole_minutes() < -CHECK_BUFFER_IN_MINUTES {
    //                 info!("[Check Sources] Source {} published time {} is before last_check {}, ignoring and checking entries for now", &source.url, pub_time, source.last_checked);
    //             } else {
    //                 info!("[Check Sources] Source {} published time {} would've passed last_check {} test", &source.url, pub_time, source.last_checked);
    //             }
    //         },
    //         Err(err) => {
    //             warn!("[Check Sources] Issue parsing updated for {}, failed with err {}", &source.url, err);
    //         },
    //     }
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
            warn!("[Check Sources] Using first url for entry {}", content_url);
        } else if let Some(ref content) = entry.content
            && let Some(ref url) = content.src
        {
            content_url = url.href.clone();
            warn!(
                "[Check Sources] Using content url for entry {}",
                content_url
            );
        } else {
            content_url = "No Url".into();
        }

        let pub_time = if let Some(ref pub_) = entry.published
            && let Ok(pub_time) = OffsetDateTime::from_unix_timestamp(pub_.timestamp())
        {
            pub_time
        } else {
            error!(
                "[Check Sources] Issue parsing published for post at {}",
                content_url
            );
            break;
        };

        if pub_time <= source.last_checked {
            warn!(
                "[Check Sources] Last post checked at url {} was published {}",
                content_url, pub_time
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

        let content_body = if let Some(ref summary) = entry.summary {
            summary.content.clone()
        } else if let Some(ref content) = entry.content
            && let Some(ref body) = content.body
        {
            body.clone()
        } else {
            "No body".into()
        };

        entries.push(SourceEntry {
            title: content_title,
            url: content_url,
            body: content_body,
        });
    }

    if entries.is_empty() {
        SourceActivity::Unchanged {
            source_url: source.url,
        }
    } else {
        SourceActivity::Changed {
            source_id: source.id,
            source_url: source.url,
            channel_title: channel
                .title
                .map_or_else(|| "Missing Channel Title".into(), |t| t.content),
            most_recent: most_recent.unwrap_or_else(OffsetDateTime::now_utc),
            posts: entries,
        }
    }
}

async fn handle_activity(activity: SourceActivity, client: reqwest::Client, conn: Connection) {
    match activity {
        SourceActivity::Disabled { source_url } => {
            info!(
                "[Check Sources]:[Handle Activity] Source at {} remains disabled",
                source_url
            );
        }
        SourceActivity::Failed {
            source_id,
            source_url,
            new_failed_count,
            reason,
        } => {
            let new_enabled: u32 = if new_failed_count
                >= env::var("SOURCE_DISABLE_AFTER").map_or(10, |v| v.parse().unwrap_or(10))
            {
                0
            } else {
                1
            };

            let res = conn
                .execute(
                    &format!(
                        "UPDATE {SOURCES_T} SET failed_count = ?1, enabled = ?2 WHERE id = ?3"
                    ),
                    (new_failed_count, new_enabled, source_id),
                )
                .await;

            if let Err(err) = res {
                error!(
                    "[Check Sources]:[Handle Activity] failed to update failed source at {} for reason: {}",
                    source_url, err
                );
            } else if new_enabled == 0 {
                error!(
                    "[Check Sources]:[Handle Activity] Disabling source at {} for reason: {}",
                    source_url, reason
                );

                #[cfg(feature = "mail")]
                let res = crate::queries::mail::send_email_with_cient(
                    client,
                    "Source disabled",
                    &format!("The source at {} has been disabled after failing too much. The error is {}", source_url, reason),
                    &format!(r#"
                        <p>Url: {u}</p>
                        <p>Link: <a href="{u}">Link</a></p>
                        <p>Reason:</p><pre>{r}</pre>
                    "#, u = source_url, r = reason)).await;
                #[cfg(feature = "mail")]
                if let Err(err) = res {
                    error!(
                        "[Check Sources]:[Handle Activity] failed to send disabled email for source at {} for reason: {}",
                        source_url, err
                    );
                }
            } else {
                error!(
                    "[Check Sources]:[Handle Activity] Source at {} has failed {} times",
                    source_url, new_failed_count
                );
            }
        }
        SourceActivity::Unchanged { source_url } => {
            info!(
                "[Check Sources]:[Handle Activity] Source at {} has no new posts",
                source_url
            );
        }
        SourceActivity::Changed {
            source_id,
            source_url,
            channel_title,
            most_recent,
            posts,
        } => {
            let res = conn
                .execute(
                    &format!(
                        "UPDATE {SOURCES_T} SET last_checked = ?1, failed_count = ?2 WHERE id = ?3"
                    ),
                    (serde_json::to_string(&most_recent).unwrap(), 0, source_id),
                )
                .await;

            if let Err(err) = res {
                error!(
                    "[Check Sources]:[Handle Activity] failed to update source at url {} for reason {}",
                    source_url, err
                );
            }

            for post in posts {
                let res = conn
                    .execute(
                        &format!(
                            "INSERT INTO {ACTIVITIES_T} 
                                        (source_id, post_url, timestamp) 
                                    VALUES 
                                        (?1, ?2, ?3)
                                    "
                        ),
                        (
                            source_id,
                            post.url.clone(),
                            serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
                        ),
                    )
                    .await;

                if let Err(err) = res {
                    error!(
                        "[Check Sources]:[Handle Activity] failed to insert activity at url {} for reason {}",
                        post.url, err
                    );
                }

                #[cfg(feature = "mail")]
                let res = crate::queries::mail::send_email(
                    &format!("{} - {}", post.title, channel_title),
                    &format!("Source: {}\n\n{}", post.url, post.body),
                    &format!(
                        r#"
                            <p>Source: <a href="{}">Link</a></p>
                            <p>{}</p>
                        "#,
                        post.url, post.body
                    ),
                )
                .await;

                #[cfg(feature = "mail")]
                if let Err(err) = res {
                    error!(
                        "[Check Sources]:[Handle Activity] failed to send email for activity at url {} for reason {}",
                        post.url, err
                    );
                }
            }
        }
    }
}

pub async fn check_sources(data: &AppData) {
    let start_time = OffsetDateTime::now_utc();
    info!("[Check Sources] Starting check {}", start_time);

    if !cfg!(feature = "mail") {
        warn!("[Check Sources] will not send emails as feature is not enabled");
    }

    let sources_res = get_sources(data.db.connect().unwrap()).await;
    let Ok(sources) = sources_res else {
        panic!(
            "Couldn't check sources. Err: {}",
            sources_res.err().unwrap()
        );
    };

    let mut threads = JoinSet::new();
    let client = reqwest::Client::new();
    let conn = data.db.connect().unwrap();

    for source in sources {
        let s_client = client.clone();
        let s_conn = conn.clone();

        threads.spawn(async move {
            let activity = check_source(source, s_client.clone()).await;
            handle_activity(activity, s_client, s_conn).await
        });
    }

    let _ = threads.join_all().await;

    let now = OffsetDateTime::now_utc();
    info!(
        "[Check Sources] Finished checking sources. Started at {} finished at {} took {}",
        start_time,
        now,
        now - start_time
    );

    // match get_sources(data.db.connect().unwrap()).await {
    //     Ok(sources) => {
    //         let mut threads = JoinSet::new();
    //         let client = reqwest::Client::new();
    //         let start_time = OffsetDateTime::now_utc();

    //         let (act_send, mut act_recv) = mpsc::channel(100);

    //         info!("[Check Sources] Check Started at {}", start_time);

    //         for source in sources {
    //             if !source.enabled {
    //                 info!("[Check Sources] Skipping disabled source {}", source.url);
    //                 continue;
    //             }

    //             let thisclient = client.clone();
    //             let r_send = act_send.clone();

    //             info!("[Check Sources] Thread pushed for {}", source.url);
    //             threads.spawn(async move {
    //                 let res = thisclient.get(&source.url).send().await;
    //                 let Ok(res) = res else {
    //                     error!(
    //                         "[Check Sources] Network request for {} failed with err {}",
    //                         &source.url,
    //                         res.err().unwrap()
    //                     );
    //                     return;
    //                 };

    //                 if !res.status().is_success() {
    //                     error!(
    //                         "[Check Sources] Network request for {} failed with status {}",
    //                         &source.url,
    //                         res.status()
    //                     );
    //                     return;
    //                 }

    //                 let content = res.bytes().await;
    //                 let Ok(content) = content else {
    //                     error!(
    //                         "[Check Sources] Bytes failed for {} with err {}",
    //                         &source.url,
    //                         content.err().unwrap()
    //                     );
    //                     return;
    //                 };

    //                 let channel = parser::parse(&(content)[..]);
    //                 let Ok(channel) = channel  else {
    //                     error!(
    //                         "[Check Sources] Parsing failed for {} with err {}",
    //                         &source.url,
    //                         channel.err().unwrap()
    //                     );
    //                     return;
    //                 };

    //                 if channel.updated.is_some() {
    //                     match OffsetDateTime::from_unix_timestamp(channel.updated.unwrap().timestamp()) {
    //                         Ok(upd_time) => {
    //                             if (upd_time - source.last_checked).whole_minutes() < -CHECK_BUFFER_IN_MINUTES {
    //                                 info!("[Check Sources] Source {}, hasn't been updated since last_check {}, upd_time {}", &source.url, source.last_checked, upd_time);
    //                                 return;
    //                             }
    //                         },
    //                         Err(err) => {
    //                             warn!("[Check Sources] Issue parsing updated for {}, failed with err {}", &source.url, err);
    //                         },
    //                     }
    //                 } else if channel.published.is_some() {
    //                     match OffsetDateTime::from_unix_timestamp(channel.published.unwrap().timestamp()) {
    //                         Ok(pub_time) => {
    //                             if (pub_time - source.last_checked).whole_minutes() < -CHECK_BUFFER_IN_MINUTES {
    //                                 info!("[Check Sources] Source {} published time {} is before last_check {}, ignoring and checking entries for now", &source.url, pub_time, source.last_checked);
    //                             } else {
    //                                 info!("[Check Sources] Source {} published time {} would've passed last_check {} test", &source.url, pub_time, source.last_checked);
    //                             }
    //                         },
    //                         Err(err) => {
    //                             warn!("[Check Sources] Issue parsing updated for {}, failed with err {}", &source.url, err);
    //                         },
    //                     }
    //                 } else {
    //                     warn!("[Check Sources] Source at {} has neither published nor updated date", &source.url);
    //                 }

    //                 let mut requests = JoinSet::new();
    //                 let mut most_recent = None;

    //                 #[cfg(feature = "mail")]
    //                 let channel_title = channel.title.unwrap().content;
    //                 for entry in channel.entries {
    //                     let content_url: String;
    //                     if let Some(x) = entry.links.iter().find(|link|
    //                         link.rel.is_some() && (link.rel.as_ref().unwrap() == "alternate" || link.rel.as_ref().unwrap() == "self")
    //                         && link.media_type.is_some() && link.media_type.as_ref().unwrap() == "text/html"
    //                     ) {
    //                         content_url = x.href.clone();
    //                     } else if entry.links.len() == 1 {
    //                         content_url = entry.links[0].href.clone();
    //                     } else if !entry.links.is_empty() {
    //                         content_url = entry.links[0].href.clone();
    //                         warn!("[Check Sources] Using first url for entry {}", content_url);
    //                     } else if entry.content.is_some() {
    //                         let opt = entry.content.as_ref().unwrap().src.as_ref();
    //                         if opt.is_some() {
    //                             content_url = opt.unwrap().href.clone();
    //                             warn!("[Check Sources] Using content url for entry {}", content_url);
    //                         } else {
    //                             content_url = "No Url".into();
    //                         }
    //                     } else {
    //                         content_url = "No Url".into();
    //                     }

    //                     let Ok(pub_time) = OffsetDateTime::from_unix_timestamp(entry.published.unwrap().timestamp()) else {
    //                         warn!("[Check Sources] Issue parsing published for post at {}", content_url);
    //                         break;
    //                     };
    //                     if pub_time <= source.last_checked {
    //                         warn!("[Check Sources] Last post checked at url {} is {} minutes old", content_url, (start_time - pub_time).whole_minutes());
    //                         break;
    //                     }
    //                     if most_recent.is_none() {
    //                         most_recent = Some(pub_time);
    //                     }

    //                     #[cfg(feature = "mail")]
    //                     let r_channel_title = channel_title.clone();
    //                     requests.spawn(async move {
    //                         #[cfg(feature = "mail")]
    //                         {
    //                             let mut content_body = "No body";
    //                             if entry.content.is_some() {
    //                                 content_body = entry.content.as_ref().unwrap().body.as_ref().unwrap();
    //                             } else if entry.summary.is_some() {
    //                                 content_body = &entry.summary.as_ref().unwrap().content;
    //                             }

    //                             let res = crate::queries::mail::send_email(
    //                                 &format!("{} - {}", entry.title.unwrap().content, r_channel_title),
    //                                 &format!("Source: {}\n\n{}", content_url, content_body),
    //                                 &format!(r#"
    //                                         <p>Source: <a href="{}">{}</a></p>
    //                                         <p>{}</p>
    //                                     "#, content_url, content_url, content_body)
    //                             ).await;

    //                             return (res, content_url.to_owned());
    //                         };

    //                         #[cfg(not(feature = "mail"))]
    //                         (content_url.to_owned())
    //                     });
    //                 }

    //                 if requests.is_empty() {
    //                     warn!("[Check Sources] No requests sent for {}", &source.url);
    //                     return;
    //                 }

    //                 match r_send.send(Message::Source(source.id, most_recent.unwrap())).await {
    //                     Ok(_) => {},
    //                     Err(err) => {
    //                         error!("[Check Sources] Failed to send down channel for source {} with err {}", source.id, err);
    //                     }
    //                 };

    //                 while let Some(res) = requests.join_next().await {
    //                     match res {
    //                         Ok(success) => {
    //                             #[cfg(feature = "mail")]
    //                             match success {
    //                                 (Ok(res), url) => {
    //                                     let status = res.status();
    //                                     let body = res.text().await.unwrap_or("Missing body".into());
    //                                     if status.is_success() {
    //                                         info!("[Check Sources] succeed for url {} with body {}", &url, body);

    //                                         match r_send.send(Message::Activity(source.id, url.clone())).await {
    //                                             Ok(_) => {
    //                                                 info!("[Check Sources] Sent Activity ({}, {})", source.id, url.clone());
    //                                             },
    //                                             Err(err) => {
    //                                                 error!("[Check Sources] Failed to send down channel for id {} and url {} with err {}", source.id, &url, err);
    //                                             },
    //                                         }
    //                                     } else {
    //                                         error!("[Check Status] failed for url {} with status code {} and body {}", url, status, body);
    //                                     }
    //                                 },
    //                                 (Err(err), url) => {
    //                                     error!("[Check Sources] Email request for {} failed with err {}", url, err);
    //                                 },
    //                             }
    //                             #[cfg(not(feature = "mail"))]
    //                             match r_send.send(Message::Activity(source.id, success.clone())).await {
    //                                 Ok(_) => {
    //                                     info!("[Check Sources] Sent Activity ({}, {})", source.id, success.clone());
    //                                 },
    //                                 Err(err) => {
    //                                     error!("[Check Sources] Failed to send down channel for id {} and url {} with err {}", source.id, &success, err);
    //                                 },
    //                             }
    //                         },
    //                         Err(err) => {
    //                             error!("[Check Sources] unknown email request failed with err {}", err);
    //                         },
    //                     }
    //                 }
    //             });
    //         }

    //         // Drop Sender so receiver closes when all threads terminate
    //         drop(act_send);

    //         let mut count = 0;
    //         let mut failed = Vec::new();
    //         let db = data.db.connect().unwrap();
    //         while let Some(m) = act_recv.recv().await {
    //             match m {
    //                 Message::Activity(id, url) => {
    //                     let result = db
    //                         .execute(
    //                             &format!(
    //                                 "INSERT INTO {ACTIVITIES_T}
    //                                     (source_id, post_url, timestamp)
    //                                 VALUES
    //                                     (?1, ?2, ?3)
    //                                 "
    //                             ),
    //                             (id, url, serde_json::to_string(&start_time).unwrap()),
    //                         )
    //                         .await;

    //                     if let Err(e) = result {
    //                         failed.push(e);
    //                     }
    //                 }
    //                 Message::Source(id, most_recent) => {
    //                     let result = db
    //                         .execute(
    //                             &format!("UPDATE {SOURCES_T} SET last_checked = ?1 WHERE id = ?2"),
    //                             (serde_json::to_string(&most_recent).unwrap(), id),
    //                         )
    //                         .await;

    //                     if let Err(e) = result {
    //                         failed.push(e);
    //                     }
    //                 }
    //             };

    //             count += 1;
    //         }

    //         if !failed.is_empty() {
    //             error!(
    //                 "[Check Sources] {} out of {} Activities and Sources failed to add",
    //                 failed.len(),
    //                 count
    //             );
    //             for (i, err) in failed.into_iter().enumerate() {
    //                 error!("[Check Sources]\t{}: {}", i, err);
    //             }
    //         } else {
    //             info!(
    //                 "[Check Sources] Successfully added {} activities and sources",
    //                 count
    //             );
    //         }

    //         // Don't think this is necessary but will leave in case
    //         while (threads.join_next().await).is_some() {}

    //         let now = OffsetDateTime::now_utc();
    //         info!(
    //             "[Check Sources] Finished checking sources. Started at {} finished at {} took {}",
    //             start_time,
    //             now,
    //             now - start_time
    //         );
    //     }
    //     Err(failure) => {
    //         panic!("Couldn't check sources. Err: {}", failure);
    //     }
    // };
}
