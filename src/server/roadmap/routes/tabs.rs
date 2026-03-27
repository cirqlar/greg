use actix_web::{HttpRequest, delete, get, post, web};
use log::{error, info, warn};

use crate::AppData;
use crate::auth::{is_logged_in, return_password_error};
use crate::roadmap::queries::tabs;
use crate::roadmap::types::{RTab, RoadmapWatchedTab};
use crate::shared::{ApiResponse, Failure, Query, Success};

#[get("/most_recent_tabs")]
pub async fn get_most_recent_tabs(
    data: AppData,
    query: web::Query<Query>,
    req: HttpRequest,
) -> ApiResponse<Vec<RTab>> {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };
    if query.demo || is_logged_in(&req, db.clone()).await? {
        tabs::get_most_recent_roadmap_tabs(db)
            .await
            .map(|tabs| {
                info!("Got roadmap tabs");

                Success::ok(tabs)
            })
            .map_err(|err| {
                error!("Failed to get roadmap tabs. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}

#[get("/watched_tabs")]
pub async fn get_watched_tabs(
    data: AppData,
    query: web::Query<Query>,
    req: HttpRequest,
) -> ApiResponse<Vec<RoadmapWatchedTab>> {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };

    if query.demo || is_logged_in(&req, db.clone()).await? {
        tabs::get_watched_tabs(db)
            .await
            .map(|watched_tabs| {
                info!("Got watched tabs");

                Success::ok(watched_tabs)
            })
            .map_err(|err| {
                error!("Failed to get watched tabs. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}

#[post("/watched_tabs/add/{tab_id}")]
pub async fn add_watched_tab(
    path: web::Path<String>,
    data: AppData,
    req: HttpRequest,
) -> ApiResponse {
    let db = data.app_db.connect().unwrap();

    if is_logged_in(&req, db.clone()).await? {
        let tab_id = path.into_inner();

        tabs::add_watched_tab(db, tab_id.clone()).await
            .map(|success| {
                if success == 1 {
                    info!("Inserted watched tab. tab_id: {tab_id}");
                } else {
                    warn!(
                        "Inserted watched tab. tab_id: {tab_id}. Rows affected in insert not 1, is: {success}"
                    );
                }

                Success::ok_message("Watched tab added successfully".into())
            })
            .map_err(|err| {
                error!("Failed to insert watched tab. tab_id: {tab_id}. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}

#[delete("/watched_tabs/{id}")]
pub async fn delete_watched_tab(
    path: web::Path<u32>,
    data: AppData,
    req: HttpRequest,
) -> ApiResponse {
    let db = data.app_db.connect().unwrap();
    let id = path.into_inner();

    if is_logged_in(&req, db.clone()).await? {
        tabs::delete_watched_tab(db, id).await
            .map(|success| {
                if success == 1 {
                    info!("Deleted watched tab. id: {id}");
                } else {
                    warn!(
                        "Deleted watched tab. id: {id}. Rows affected in deletion not 1, is: {success}"
                    );
                }

                Success::ok_message("Watched tab deleted successfully".into())
            })
            .map_err(|err| {
                error!("Failed to delete watched tab. id: {id}. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}
