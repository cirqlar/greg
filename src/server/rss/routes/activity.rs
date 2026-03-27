use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, web};
use log::{error, info, warn};

use crate::server::AppData;
use crate::server::auth::{is_logged_in, return_password_error};
use crate::server::rss::queries::activity;
use crate::server::shared::PaginationQuery;
use crate::server::shared::{Failure, Success};

#[get("/activity")]
pub async fn get_activity(
    data: AppData,
    query: web::Query<PaginationQuery>,
    req: HttpRequest,
) -> impl Responder {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };

    if query.demo || is_logged_in(&req, db.clone()).await {
        match activity::get_activity(db, query.count.unwrap_or(35), query.skip.unwrap_or(0)).await {
            Ok(activities) => {
                info!("Got activity");
                HttpResponse::Ok().json(activities)
            }
            Err(err) => {
                error!("Failed to get activity. err: {err:?}");
                (Failure {
                    message: format!("{err}"),
                })
                .server_error()
            }
        }
    } else {
        return_password_error()
    }
}

#[get("/activity/{source_id}")]
pub async fn get_source_activity(
    data: AppData,
    path: web::Path<u32>,
    query: web::Query<PaginationQuery>,
    req: HttpRequest,
) -> impl Responder {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };
    let source_id = path.into_inner();

    if query.demo || is_logged_in(&req, db.clone()).await {
        match activity::get_source_activity(
            db,
            query.count.unwrap_or(35),
            query.skip.unwrap_or(0),
            source_id,
        )
        .await
        {
            Ok(activities) => {
                info!("Got activity for {source_id}");
                HttpResponse::Ok().json(activities)
            }
            Err(err) => {
                error!("Failed to get activity for source with id {source_id}. err: {err:?}");
                (Failure {
                    message: format!("{err}"),
                })
                .server_error()
            }
        }
    } else {
        return_password_error()
    }
}

#[delete("/activity")]
pub async fn clear_all_activities(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.app_db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await {
        match activity::delete_all_activity(db).await {
            Ok(_) => {
                info!("Deleted activity");
                (Success {
                    message: "Activities deleted successfully".into(),
                })
                .ok()
            }
            Err(err) => {
                error!("Failed to delete activity. err: {err:?}");
                (Failure {
                    message: format!("{err}"),
                })
                .server_error()
            }
        }
    } else {
        return_password_error()
    }
}

#[delete("/activity/{num}")]
pub async fn clear_activities(
    path: web::Path<u32>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.app_db.connect().unwrap();
    let num = path.into_inner();

    if is_logged_in(&req, db.clone()).await {
        match activity::delete_activity(db, num).await {
            Ok(success) => {
                if success == (num as u64) {
                    info!("Deleted activity. count: {num}");
                } else {
                    warn!("Deleted activity. Rows affected not {num}, is: {success}");
                }

                (Success {
                    message: format!("Activity deleted. count: {success}"),
                })
                .ok()
            }
            Err(err) => {
                error!("Failed to delete activity. err: {err:?}");
                (Failure {
                    message: format!("{err}"),
                })
                .server_error()
            }
        }
    } else {
        return_password_error()
    }
}
