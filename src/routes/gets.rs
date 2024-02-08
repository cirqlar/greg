use crate::{
    types::{Activity, AppState, Failure, FromRow, Source},
    utils::{is_logged_in, return_password_error},
};

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use libsql_client::Client;
use log::{error, info};

pub async fn get_sources_inner(db_handle: &Client) -> anyhow::Result<Vec<Source>> {
    let result = db_handle.execute("SELECT * FROM sources").await?;

    let sources = result
        .rows
        .into_iter()
        .filter_map(|row| {
            let ret = Source::from_row(row);

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
async fn get_sources(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let db_handle = data.db_handle.lock().await;

    if is_logged_in(&req, &db_handle).await {
        info!("[Get Sources] Getting sources from db");
        match get_sources_inner(&db_handle).await {
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
async fn get_activity(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let db_handle = data.db_handle.lock().await;

    if is_logged_in(&req, &db_handle).await {
        info!("[Get Activity] Getting activities from db");
        let result = db_handle.execute("SELECT * FROM activities").await;
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
