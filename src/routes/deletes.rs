use actix_web::{HttpRequest, HttpResponse, Responder, delete, web};
use libsql_client::{Statement, args};
use log::{error, info};
use tokio::sync::mpsc;

use crate::{
    types::{AppData, Failure, Success},
    utils::{is_logged_in, return_password_error},
};

#[delete("/source/{id}")]
pub async fn delete_source(
    path: web::Path<i32>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let id = path.into_inner();
    let (send, mut recv) = mpsc::channel(100);

    if is_logged_in(&req, &data, send.clone(), &mut recv).await {
        let _ = data
            .db_channel
            .send((
                Statement::with_args("DELETE FROM sources WHERE id = ?", args!(id)),
                send,
            ))
            .await;

        match recv.recv().await {
            Some(Ok(success)) => {
                if success.rows_affected == 1 {
                    info!("[Delete Source] Deleted source successfully");
                    HttpResponse::Ok().json(Success {
                        message: "Source deleted successfully".into(),
                    })
                } else {
                    error!(
                        "[Delete Source] Rows affected in deletion not 1, is: {}",
                        success.rows_affected
                    );
                    HttpResponse::InternalServerError().json(Failure {
                        message: "Unexpected issue adding source".into(),
                    })
                }
            }
            Some(Err(err)) => {
                error!("[Delete Source] Deleting source failed with err: {}", err);
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't delete source. Err: {}", err),
                })
            }
            None => unreachable!(),
        }
    } else {
        error!("[Delete Source] Failed due to auth error");
        return_password_error()
    }
}

#[delete("/activity")]
pub async fn clear_all_activities(data: AppData, req: HttpRequest) -> impl Responder {
    let (send, mut recv) = mpsc::channel(100);

    if is_logged_in(&req, &data, send.clone(), &mut recv).await {
        let _ = data
            .db_channel
            .send((Statement::new("DELETE FROM activities"), send))
            .await;

        match recv.recv().await {
            Some(Ok(_)) => {
                info!("[Delete All Activities] Deleted activities successfully");
                HttpResponse::Ok().json(Success {
                    message: "Activities deleted successfully".into(),
                })
            }
            Some(Err(err)) => {
                error!(
                    "[Delete All Activities] Deleting activities failed with err: {}",
                    err
                );
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't delete all activities. Err: {}", err),
                })
            }
            None => unreachable!(),
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
    let num = path.into_inner();
    let (send, mut recv) = mpsc::channel(100);

    if is_logged_in(&req, &data, send.clone(), &mut recv).await {
        let _ = data
            .db_channel
            .send((Statement::with_args(
				"DELETE FROM activities WHERE id IN (SELECT id FROM activities ORDER BY id ASC LIMIT ?)", 
				args!(num)
			), send))
            .await;

        match recv.recv().await {
            Some(Ok(success)) => {
                if success.rows_affected == (num as u64) {
                    info!("[Delete Activities] Deleted activities successfully");
                    HttpResponse::Ok().json(Success {
                        message: "Activities deleted successfully".into(),
                    })
                } else {
                    error!(
                        "[Delete Activities] Rows affected in deletion not {}, is: {}",
                        num, success.rows_affected
                    );
                    HttpResponse::InternalServerError().json(Failure {
                        message: "Unexpected issue adding source".into(),
                    })
                }
            }
            Some(Err(err)) => {
                error!(
                    "[Delete Activities] Deleting activities failed with err: {}",
                    err
                );
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't delete activities. Err: {}", err),
                })
            }
            None => unreachable!(),
        }
    } else {
        error!("[Delete Activities] Failed due to auth error");
        return_password_error()
    }
}
