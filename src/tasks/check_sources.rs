use std::env;

use feed_rs::parser;
use libsql_client::{args, Statement};
use log::{error, info, warn};
use time::OffsetDateTime;
use tokio::{sync::mpsc, task::JoinSet};

use crate::{
    routes::gets::get_sources_inner,
    types::{AppData, FromRow, ISource},
};

enum Message {
    Activity(i32, String),
    Source(i32, OffsetDateTime),
}

const CHECK_BUFFER_IN_MINUTES: i64 = 5;

pub async fn check_sources(data: &AppData) {
    info!("[Check Sources] Starting check");
    let (db_send, mut db_recv) = mpsc::channel(100);

    match get_sources_inner(data, db_send.clone(), &mut db_recv, ISource::from_row).await {
        Ok(sources) => {
            let mut threads = JoinSet::new();
            let client = reqwest::Client::new();
            let start_time = OffsetDateTime::now_utc();

            let (act_send, mut act_recv) = mpsc::channel(100);

            info!("[Check Sources] Check Started at {}", start_time);

            for source in sources {
                let thisclient = client.clone();
                let r_send = act_send.clone();

                info!("[Check Sources] Thread pushed for {}", source.url);
                threads.spawn(async move {
                    let res = thisclient.get(&source.url).send().await;
                    let Ok(res) = res else {
                        error!(
                            "[Check Sources] Network request for {} failed with err {}",
                            &source.url,
                            res.err().unwrap()
                        );
                        return;
                    };

                    if !res.status().is_success() {
                        error!(
                            "[Check Sources] Network request for {} failed with status {}",
                            &source.url,
                            res.status()
                        );
                        return;
                    }

                    let content = res.bytes().await;
                    let Ok(content) = content else {
                        error!(
                            "[Check Sources] Bytes failed for {} with err {}",
                            &source.url,
                            content.err().unwrap()
                        );
                        return;
                    };

                    let channel = parser::parse(&(content)[..]);
                    let Ok(channel) = channel  else {
                        error!(
                            "[Check Sources] Parsing failed for {} with err {}",
                            &source.url,
                            channel.err().unwrap()
                        );
                        return;
                    };

                    if channel.updated.is_some() {
                        match OffsetDateTime::from_unix_timestamp(channel.updated.unwrap().timestamp()) {
                            Ok(upd_time) => {
                                if (upd_time - source.last_checked).whole_minutes() < -CHECK_BUFFER_IN_MINUTES {
                                    info!("[Check Sources] Source {}, hasn't been updated since last_check {}, upd_time {}", &source.url, source.last_checked, upd_time);
                                    return;
                                }
                            },
                            Err(err) => {
                                warn!("[Check Sources] Issue parsing updated for {}, failed with err {}", &source.url, err);
                            },
                        }
                    } else if channel.published.is_some() {
                        match OffsetDateTime::from_unix_timestamp(channel.published.unwrap().timestamp()) {
                            Ok(pub_time) => {
                                if (pub_time - source.last_checked).whole_minutes() < -CHECK_BUFFER_IN_MINUTES {
                                    info!("[Check Sources] Source {} published time {} is before last_check {}, ignoring and checking entries for now", &source.url, pub_time, source.last_checked);
                                } else {
                                    info!("[Check Sources] Source {} published time {} would've passed last_check {} test", &source.url, pub_time, source.last_checked);
                                }
                            },
                            Err(err) => {
                                warn!("[Check Sources] Issue parsing updated for {}, failed with err {}", &source.url, err);
                            },
                        }
                    } else {
                        warn!("[Check Sources] Source at {} has neither published nor updated date", &source.url);
                    }

                    let mut requests = JoinSet::new();
                    let mut most_recent = None;

                    let channel_title = channel.title.unwrap().content;
                    for entry in channel.entries {
                        let content_url: String;
                        if let Some(x) = entry.links.iter().find(|link|
                            link.rel.is_some() && (link.rel.as_ref().unwrap() == "alternate" || link.rel.as_ref().unwrap() == "self")
                            && link.media_type.is_some() && link.media_type.as_ref().unwrap() == "text/html"
                        ) {
                            content_url = x.href.clone();
                        } else if entry.links.len() == 1 {
                            content_url = entry.links[0].href.clone();
                        } else if !entry.links.is_empty() {
                            content_url = entry.links[0].href.clone();
                            warn!("[Check Sources] Using first url for entry {}", content_url);
                        } else if entry.content.is_some() {
                            let opt = entry.content.as_ref().unwrap().src.as_ref();
                            if opt.is_some() {
                                content_url = opt.unwrap().href.clone();
                                warn!("[Check Sources] Using content url for entry {}", content_url);
                            } else {
                                content_url = "No Url".into();
                            }
                        } else {
                            content_url = "No Url".into();
                        }

                        let Ok(pub_time) = OffsetDateTime::from_unix_timestamp(entry.published.unwrap().timestamp()) else {
                            warn!("[Check Sources] Issue parsing published for post at {}", content_url);
                            break;
                        };
                        if (pub_time - source.last_checked).whole_minutes() < -CHECK_BUFFER_IN_MINUTES {
                            warn!("[Check Sources] Last post checked at url {} is {} minutes old", content_url, (start_time - pub_time).whole_minutes());
                            break;
                        }
                        if most_recent.is_none() {
                            most_recent = Some(pub_time);
                        }

                        let r_client = thisclient.clone();
                        let r_channel_title = channel_title.clone();
                        requests.spawn(async move {
                            let mut content_body = "No body";
                            if entry.content.is_some() {
                                content_body = entry.content.as_ref().unwrap().body.as_ref().unwrap();
                            } else if entry.summary.is_some() {
                                content_body = &entry.summary.as_ref().unwrap().content;
                            }

                            let res = r_client.post(env::var("MAIL_URL").expect("MAIL_URL should be set"))
                                .bearer_auth(env::var("MAIL_TOKEN").expect("MAIL_TOKEN should be set"))
                                .header("Content-Type", "application/json")
                                .body(serde_json::json!({
                                    "from": {
                                        "email": env::var("FROM_EMAIL").expect("FROM_EMAIL should be set"),
                                        "name": env::var("FROM_NAME").expect("FROM_NAME should be set")
                                    },
                                    "to": [{
                                        "email": env::var("TO_EMAIL").expect("TO_EMAIL should be set"),
                                        "name": env::var("TO_NAME").expect("TO_NAME should be set")
                                    }],
                                    "subject": format!("{} - {}", entry.title.unwrap().content, r_channel_title),
                                    "text": format!("Source: {}\n\n{}", content_url, content_body),
                                    "html": format!(r#"
                                        <p>Source: <a href="{}">{}</a></p>
                                        <p>{}</p>
                                    "#, content_url, content_url, content_body)
                                }).to_string()).send().await;

                            (res, content_url.to_owned())
                        });
                    }

                    if requests.is_empty() {
                        warn!("[Check Sources] No requests sent for {}", &source.url);
                        return;
                    }

                    match r_send.send(Message::Source(source.id, most_recent.unwrap())).await {
                        Ok(_) => {},
                        Err(err) => {
                            error!("[Check Sources] Failed to send down channel for source {} with err {}", source.id, err);
                        }
                    };

                    while let Some(res) = requests.join_next().await {
                        match res {
                            Ok(success) => match success {
                                (Ok(res), url) => {
                                    let status = res.status();
                                    let body = res.text().await.unwrap_or("Missing body".into());
                                    if status.is_success() {
                                        info!("[Check Sources] succeed for url {} with body {}", &url, body);

                                        match r_send.send(Message::Activity(source.id, url.clone())).await {
                                            Ok(_) => {
                                                info!("[Check Sources] Sent Activity ({}, {})", source.id, url.clone());
                                            },
                                            Err(err) => {
                                                error!("[Check Sources] Failed to send down channel for id {} and url {} with err {}", source.id, &url, err);
                                            },
                                        }
                                    } else {
                                        error!("[Check Status] failed for url {} with status code {} and body {}", url, status, body);
                                    }
                                },
                                (Err(err), url) => {
                                    error!("[Check Sources] Email request for {} failed with err {}", url, err);
                                },
                            },
                            Err(err) => {
                                error!("[Check Sources] unknown email request failed with err {}", err);
                            },
                        }
                    }
                });
            }

            // Drop Sender so receiver closes when all threads terminate
            drop(act_send);

            let mut count = 0;
            while let Some(m) = act_recv.recv().await {
                let stmnt = match m {
                    Message::Activity(id, url) => Statement::with_args(
                        "INSERT INTO activities (source_id, post_url, timestamp) VALUES (?, ?, ?)",
                        args!(id, url, serde_json::to_string(&start_time).unwrap(),),
                    ),
                    Message::Source(id, most_recent) => Statement::with_args(
                        "UPDATE sources SET last_checked = ? WHERE id = ?",
                        args!(serde_json::to_string(&most_recent).unwrap(), id,),
                    ),
                };

                let _ = data.db_channel.send((stmnt, db_send.clone())).await;
                count += 1;
            }

            // Drop Sender so receiver closes when db has fulfilled all requests
            drop(db_send);

            let mut failed = Vec::new();
            while let Some(res) = db_recv.recv().await {
                match res {
                    Ok(_) => {}
                    Err(err) => failed.push(err),
                }
            }

            if !failed.is_empty() {
                error!(
                    "[Check Sources] {} out of {} Activities and Sources failed to add",
                    failed.len(),
                    count
                );
                for (i, err) in failed.into_iter().enumerate() {
                    error!("[Check Sources]\t{}: {}", i, err);
                }
            } else {
                info!(
                    "[Check Sources] Successfully added {} activities and sources",
                    count
                );
            }

            // Don't think this is necessary but will leave in case
            while (threads.join_next().await).is_some() {}

            let now = OffsetDateTime::now_utc();
            info!(
                "[Check Sources] Finished checking sources. Started at {} finished at {} took {}",
                start_time,
                now,
                now - start_time
            );
        }
        Err(failure) => {
            panic!("Couldn't check sources. Err: {}", failure);
        }
    };
}
