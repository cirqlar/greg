use crate::{
    types::{
        Activity, AppData, DbReturnReciever, DbReturnSender, Failure, FromRow, Source,
        LOGGED_IN_COOKIE,
    },
    utils::{is_logged_in, return_password_error},
};

use actix_web::{cookie::Cookie, get, HttpRequest, HttpResponse, Responder};
use libsql_client::{Row, Statement};
use log::{error, info};
use serde_json::json;
use tokio::sync::mpsc;

#[get("/check-logged-in")]
pub async fn check_logged_in(data: AppData, req: HttpRequest) -> impl Responder {
    let (send, mut recv) = mpsc::channel(100);

    let logged_in = is_logged_in(&req, &data, send, &mut recv).await;
    let mut res = HttpResponse::Ok();
    if !logged_in {
        let mut c = Cookie::build(LOGGED_IN_COOKIE, "").finish();
        c.make_removal();
        res.cookie(c);
    }

    res.json(logged_in)
}

#[get("/keep_alive")]
pub async fn keep_alive() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "kept_alive": true,
    }))
}

pub async fn get_sources_inner<T>(
    data: &AppData,
    send: DbReturnSender,
    recv: &mut DbReturnReciever,
    transform: fn(Row) -> anyhow::Result<T>,
) -> anyhow::Result<Vec<T>> {
    let _ = data
        .db_channel
        .send((Statement::from("SELECT * FROM sources"), send))
        .await;
    let result = recv.recv().await.unwrap()?;

    let sources = result
        .rows
        .into_iter()
        .filter_map(|row| {
            let ret = transform(row);

            if ret.is_err() {
                error!(
                    "[Get Sources Inner] Failed to parse row with err: {}",
                    ret.err().unwrap()
                );
                None
            } else {
                ret.ok()
            }
        })
        .collect::<Vec<_>>();

    Ok(sources)
}

#[get("/sources")]
pub async fn get_sources(data: AppData, req: HttpRequest) -> impl Responder {
    let (send, mut recv) = mpsc::channel(100);

    if is_logged_in(&req, &data, send.clone(), &mut recv).await {
        info!("[Get Sources] Getting sources from db");
        match get_sources_inner(&data, send.clone(), &mut recv, Source::from_row).await {
            Ok(sources) => {
                info!("[Get Sources] Got sources successfully");
                HttpResponse::Ok().json(sources)
            }
            Err(err) => {
                error!("[Get Sources] Getting sources failed with err: {}", err);
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't get sources. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Get Sources] Failed due to auth error");
        return_password_error()
    }
}

#[get("/activity")]
pub async fn get_activity(data: AppData, req: HttpRequest) -> impl Responder {
    let (send, mut recv) = mpsc::channel(100);

    if is_logged_in(&req, &data, send.clone(), &mut recv).await {
        info!("[Get Activity] Getting activities from db");

        let _ = data
            .db_channel
            .send((
                Statement::from(
                    "SELECT 
                        activities.id, 
                        activities.post_url, 
                        activities.timestamp, 
                        sources.url 
                    FROM activities 
                    INNER JOIN sources ON activities.source_id = sources.id
                    ORDER BY activities.id DESC
                    LIMIT 35",
                ),
                send.clone(),
            ))
            .await;
        let Some(result) = recv.recv().await else {
            unreachable!();
        };
        match result {
            Ok(success) => {
                let activities = success
                    .rows
                    .into_iter()
                    .filter_map(|row| {
                        let ret = Activity::from_row(row);

                        if ret.is_err() {
                            error!(
                                "[Get Activity] Failed to parse row with err: {}",
                                ret.err().unwrap()
                            );
                            None
                        } else {
                            ret.ok()
                        }
                    })
                    .collect::<Vec<_>>();

                info!("[Get Activity] Got activities successfully");
                HttpResponse::Ok().json(activities)
            }
            Err(err) => {
                error!("[Get Activity] Getting sources failed with err: {}", err);
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't get activities. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Get Activity] Failed due to auth error");
        return_password_error()
    }
}
