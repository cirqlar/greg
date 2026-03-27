use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, post, web};
use log::{error, info, warn};

use crate::server::AppData;
use crate::server::auth::{is_logged_in, return_password_error};
use crate::server::roadmap::queries::tabs;
use crate::server::shared::{Failure, Query, Success};

#[get("/most_recent_tabs")]
pub async fn get_most_recent_tabs(
    data: AppData,
    query: web::Query<Query>,
    req: HttpRequest,
) -> impl Responder {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };
    if query.demo || is_logged_in(&req, db.clone()).await {
        match tabs::get_most_recent_roadmap_tabs(db).await {
            Ok(tabs) => {
                info!("Got roadmap tabs");
                HttpResponse::Ok().json(tabs)
            }
            Err(err) => {
                error!("Failed to get roadmap tabs. err: {err:?}");
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

#[get("/watched_tabs")]
pub async fn get_watched_tabs(
    data: AppData,
    query: web::Query<Query>,
    req: HttpRequest,
) -> impl Responder {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };

    if query.demo || is_logged_in(&req, db.clone()).await {
        match tabs::get_watched_tabs(db).await {
            Ok(watched_tabs) => {
                info!("Got watched tabs");
                HttpResponse::Ok().json(watched_tabs)
            }
            Err(err) => {
                error!("Failed to get watched tabs. err: {err:?}");
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

#[post("/watched_tabs/add/{tab_id}")]
pub async fn add_watched_tab(
    path: web::Path<String>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.app_db.connect().unwrap();

    if is_logged_in(&req, db.clone()).await {
        let tab_id = path.into_inner();

        match tabs::add_watched_tab(db, tab_id.clone()).await {
            Ok(success) => {
                if success == 1 {
                    info!("Inserted watched tab. tab_id: {tab_id}");
                } else {
                    warn!(
                        "Inserted watched tab. tab_id: {tab_id}. Rows affected in insert not 1, is: {success}"
                    );
                }
                (Success {
                    message: "Watched tab added successfully".into(),
                })
                .ok()
            }
            Err(err) => {
                error!("Failed to insert watched tab. tab_id: {tab_id}. err: {err:?}");
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

#[delete("/watched_tabs/{id}")]
pub async fn delete_watched_tab(
    path: web::Path<u32>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.app_db.connect().unwrap();
    let id = path.into_inner();

    if is_logged_in(&req, db.clone()).await {
        match tabs::delete_watched_tab(db, id).await {
            Ok(success) => {
                if success == 1 {
                    info!("Deleted watched tab. id: {id}");
                } else {
                    warn!(
                        "Deleted watched tab. id: {id}. Rows affected in deletion not 1, is: {success}"
                    );
                }
                (Success {
                    message: "Watched tab deleted successfully".into(),
                })
                .ok()
            }
            Err(err) => {
                error!("Failed to delete watched tab. id: {id}. err: {err:?}");
                (Failure {
                    message: format!("Couldn't delete watched tab. Err: {err}"),
                })
                .server_error()
            }
        }
    } else {
        return_password_error()
    }
}
