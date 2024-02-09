use std::env;

use feed_rs::parser;
use libsql_client::{args, Statement};
use log::{error, info, warn};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};
use tokio::{sync::mpsc, task::JoinSet};

use crate::{routes::gets::get_sources_inner, types::AppData};

pub async fn check_sources(data: &AppData) {
    info!("[Check Sources] Starting check");
    let (db_send, mut db_recv) = mpsc::channel(100);

    match get_sources_inner(data, db_send.clone(), &mut db_recv).await {
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
                        match OffsetDateTime::parse(&channel.updated.unwrap().to_rfc2822(), &Rfc2822) {
                            Ok(pubtime) => {
                                if (start_time - pubtime).whole_minutes() > 90 {
                                    return;
                                }
                            },
                            Err(err) => {
                                warn!("[Check Sources] Issue parsing updated for {}, failed with err {}", &source.url, err);
                            },
                        }
                    } else {
                        warn!("[Check Sources] Source at {} doesn't have a last published date", &source.url);
                    }

                    let mut requests = JoinSet::new();

                    let channel_title = channel.title.unwrap().content;
                    for entry in channel.entries.iter() {
                        let Ok(pubtime) = OffsetDateTime::parse(&entry.published.unwrap().to_rfc2822(), &Rfc2822) else {
                            warn!("[Check Sources] Issue parsing updated for post at {:?}", entry.links.first());
                            break;
                        };
                        if (start_time - pubtime).whole_minutes() > 60 {
                            warn!("[Check Sources] Last post checked at url {:?} is {} minutes old", entry.links.first(), (start_time - pubtime).whole_minutes());
                            break;
                        }

                        let r_client = thisclient.clone();
                        let r_entry = entry.clone();
                        let r_channel_title = channel_title.clone();
                        requests.spawn(async move {
                            let mut content_body = "No body";
                            let content_url: &str;
                            if r_entry.content.is_some() {
                                content_body = r_entry.content.as_ref().unwrap().body.as_ref().unwrap();
                            } else if r_entry.summary.is_some() {
                                content_body = &r_entry.summary.as_ref().unwrap().content;
                            }

                            if let Some(x) = r_entry.links.iter().find(|link| 
                                link.rel.is_some() && (link.rel.as_ref().unwrap() == "alternate" || link.rel.as_ref().unwrap() == "self")
                                && link.media_type.is_some() && link.media_type.as_ref().unwrap() == "text/html"
                            ) {
                                content_url = &x.href;
                            } else if r_entry.links.len() == 1 {
                                content_url = &r_entry.links[0].href;
                            } else if !r_entry.links.is_empty() {
                                content_url = &r_entry.links[0].href;
                                warn!("[Check Sources] Using first url for entry {}", content_url);
                            } else if r_entry.content.is_some() {
                                let opt = r_entry.content.as_ref().unwrap().src.as_ref();
                                if opt.is_some() {
                                    content_url = &opt.unwrap().href;
                                    warn!("[Check Sources] Using content url for entry {}", content_url);
                                } else {
                                    content_url = "No Url";
                                }
                            } else {
                                content_url = "No Url";
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
                                    "subject": format!("{} - {}", r_entry.title.unwrap().content, r_channel_title),
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

                    while let Some(res) = requests.join_next().await {
                        match res {
                            Ok(success) => match success {
                                (Ok(res), url) => {
                                    let status = res.status();
                                    let body = res.text().await.unwrap_or("Missing body".into());
                                    if status.is_success() {
                                        info!("[Check Sources] succeed for url {} with body {}", &url, body);

                                        match r_send.send((source.id, url.clone())).await {
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
            while let Some((source_id, url)) = act_recv.recv().await {
                let _ = data.db_channel.send((
                    Statement::with_args(
                        "INSERT INTO activities (source_id, post_url, timestamp) VALUES (?, ?, ?)",
                        args!(
                            source_id,
                            url.clone(),
                            serde_json::to_string(&start_time).unwrap(),
                        ),
                    ),
                    db_send.clone()
                )).await;
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
                    "[Check Sources] {} out of {} Activities failed to add",
                    failed.len(),
                    count
                );
                for (i, err) in failed.into_iter().enumerate() {
                    error!("[Check Sources]\t{}: {}", i, err);
                }
            } else {
                info!("[Check Sources] Successfully added {} activities", count);
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
