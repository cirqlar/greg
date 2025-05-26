use actix_web::{HttpRequest, HttpResponse, Responder, delete, web};
use libsql::params;
use log::{error, info};

use crate::{
    db::{ACTIVITIES_T, R_WATCHED_TABS_T, SOURCES_T},
    types::{AppData, Failure, Success},
    utils::{is_logged_in, return_password_error},
};

#[delete("/source/{id}")]
pub async fn delete_source(
    path: web::Path<i32>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.db.connect().unwrap();
    let id = path.into_inner();

    if is_logged_in(&req, db.clone()).await {
        let result = db
            .execute(&format!("DELETE FROM {SOURCES_T} WHERE id = ?1"), [id])
            .await;

        match result {
            Ok(success) => {
                if success == 1 {
                    info!("[Delete Source] Deleted source successfully");
                    HttpResponse::Ok().json(Success {
                        message: "Source deleted successfully".into(),
                    })
                } else {
                    error!(
                        "[Delete Source] Rows affected in deletion not 1, is: {}",
                        success
                    );
                    HttpResponse::InternalServerError().json(Failure {
                        message: "Unexpected issue deleting source".into(),
                    })
                }
            }
            Err(err) => {
                error!("[Delete Source] Deleting source failed with err: {}", err);
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't delete source. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Delete Source] Failed due to auth error");
        return_password_error()
    }
}

#[delete("/activity")]
pub async fn clear_all_activities(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await {
        let result = db
            .execute(&format!("DELETE FROM {ACTIVITIES_T}"), params!())
            .await;

        match result {
            Ok(_) => {
                info!("[Delete All Activities] Deleted activities successfully");
                HttpResponse::Ok().json(Success {
                    message: "Activities deleted successfully".into(),
                })
            }
            Err(err) => {
                error!(
                    "[Delete All Activities] Deleting activities failed with err: {}",
                    err
                );
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't delete all activities. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Delete All Activities] Failed due to auth error");
        return_password_error()
    }
}

#[delete("/activity/{num}")]
pub async fn clear_activities(
    path: web::Path<i32>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.db.connect().unwrap();
    let num = path.into_inner();

    if is_logged_in(&req, db.clone()).await {
        let result = db
            .execute(
                &format!(
                    "DELETE FROM {ACTIVITIES_T} 
                    WHERE id IN (
                        SELECT id 
                        FROM {ACTIVITIES_T} 
                        ORDER BY id ASC 
                        LIMIT ?1
                    )
                    "
                ),
                [num],
            )
            .await;

        match result {
            Ok(success) => {
                if success == (num as u64) {
                    info!("[Delete Activities] Deleted activities successfully");
                    HttpResponse::Ok().json(Success {
                        message: "Activities deleted successfully".into(),
                    })
                } else {
                    error!(
                        "[Delete Activities] Rows affected in deletion not {}, is: {}",
                        num, success
                    );
                    HttpResponse::InternalServerError().json(Failure {
                        message: "Unexpected issue adding source".into(),
                    })
                }
            }
            Err(err) => {
                error!(
                    "[Delete Activities] Deleting activities failed with err: {}",
                    err
                );
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't delete activities. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Delete Activities] Failed due to auth error");
        return_password_error()
    }
}

#[delete("/watched_tabs/{id}")]
pub async fn delete_watched_tab(
    path: web::Path<i32>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.db.connect().unwrap();
    let id = path.into_inner();

    if is_logged_in(&req, db.clone()).await {
        let result = db
            .execute(
                &format!("DELETE FROM {R_WATCHED_TABS_T} WHERE id = ?1"),
                [id],
            )
            .await;

        match result {
            Ok(success) => {
                if success == 1 {
                    info!("[Delete Watched Tab] Deleted watched tab successfully");
                    HttpResponse::Ok().json(Success {
                        message: "Watched tab deleted successfully".into(),
                    })
                } else {
                    error!(
                        "[Delete Watched Tab] Rows affected in deletion not 1, is: {}",
                        success
                    );
                    HttpResponse::InternalServerError().json(Failure {
                        message: "Unexpected issue deleting watched tabs".into(),
                    })
                }
            }
            Err(err) => {
                error!(
                    "[Delete Watched Tab] Deleting source failed with err: {}",
                    err
                );
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't delete watched tab. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Delete Watched Tab] Failed due to auth error");
        return_password_error()
    }
}
