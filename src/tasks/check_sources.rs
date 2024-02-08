use actix_web::web;
use feed_rs::parser;
use time::{format_description::well_known::Rfc2822, OffsetDateTime};
use tokio::runtime::Handle;

use crate::{routes::gets::get_sources_inner, types::AppState};

pub fn check_sources(rt: Handle, app_data: &web::Data<AppState>) {
    rt.block_on(async {
        let db_handle = app_data.db_handle.lock().await;

        match get_sources_inner(&db_handle).await {
            Ok(sources) => {
                let mut threads = Vec::with_capacity(sources.len());
                let client = reqwest::Client::new();
                let start_time = OffsetDateTime::now_utc();

                for source in sources {
                    let thisclient = client.clone();

                    threads.push(tokio::spawn(async move {
                        let Ok(res) = thisclient.get(source.url).send().await else {
                            return;
                        };
                        let Ok(content) = res.bytes().await else {
                            return;
                        };
                        let Ok(channel) = parser::parse(&(content)[..]) else {
                            return;
                        };
                        let Ok(pubtime) =
                            OffsetDateTime::parse(&channel.updated.unwrap().to_rfc2822(), &Rfc2822)
                        else {
                            return;
                        };

                        if (start_time - pubtime).whole_minutes() < 90 {
                            return;
                        }

                        todo!()
                    }));
                }
            }
            Err(failure) => {
                panic!("Couldn't check sources. Err: {}", failure);
            }
        };

        println!("Checking Sources");
    })
}
