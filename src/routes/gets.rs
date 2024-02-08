use crate::{
    types::{Activity, AppState, Failure, FromRow, Source},
    utils::{is_logged_in, return_password_error},
};

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use libsql_client::Client;

pub async fn get_sources_inner(db_handle: &Client) -> Result<Vec<Source>, anyhow::Error> {
    let result = db_handle.execute("SELECT * FROM sources").await?;

    let sources = result
        .rows
        .into_iter()
        .map(Source::from_row)
        .collect::<Vec<_>>();

    Ok(sources)
}

#[get("/sources")]
async fn get_sources(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let db_handle = data.db_handle.lock().await;

    if is_logged_in(&req, &db_handle).await {
        match get_sources_inner(&db_handle).await {
            Ok(sources) => HttpResponse::Ok().json(sources),
            Err(failure) => HttpResponse::InternalServerError().json(Failure {
                message: format!("Couldn't get sources. Err: {}", failure),
            }),
        }
    } else {
        return_password_error()
    }
}

#[get("/activity")]
async fn get_activity(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let db_handle = data.db_handle.lock().await;

    if is_logged_in(&req, &db_handle).await {
        let result = db_handle.execute("SELECT * FROM activities").await;
        match result {
            Ok(success) => {
                let activities = success
                    .rows
                    .into_iter()
                    .map(Activity::from_row)
                    .collect::<Vec<_>>();

                HttpResponse::Ok().json(activities)
            }
            Err(failure) => HttpResponse::InternalServerError().json(Failure {
                message: format!("Couldn't get activities. Err: {}", failure),
            }),
        }
    } else {
        return_password_error()
    }
}
