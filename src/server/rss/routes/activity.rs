use actix_web::{HttpRequest, delete, get, web};
use log::{error, info, warn};

use crate::AppData;
use crate::auth::{is_logged_in, return_password_error};
use crate::rss::Activity;
use crate::rss::queries::activity;
use crate::shared::{ApiResponse, PaginationQuery};
use crate::shared::{Failure, Success};

#[get("/activity")]
pub async fn get_activity(
    data: AppData,
    query: web::Query<PaginationQuery>,
    req: HttpRequest,
) -> ApiResponse<Vec<Activity>> {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };

    if query.demo || is_logged_in(&req, db.clone()).await? {
        activity::get_activity(db, query.count.unwrap_or(35), query.skip.unwrap_or(0))
            .await
            .map(|activity| {
                info!("Got activity");

                Success::ok(activity)
            })
            .map_err(|err| {
                error!("Failed to get activity. err: {err:?}");

                Failure::server_error(err)
            })
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
) -> ApiResponse<Vec<Activity>> {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };
    let source_id = path.into_inner();

    if query.demo || is_logged_in(&req, db.clone()).await? {
        activity::get_source_activity(
            db,
            query.count.unwrap_or(35),
            query.skip.unwrap_or(0),
            source_id,
        )
        .await
        .map(|activity| {
            info!("Got activity for {source_id}");

            Success::ok(activity)
        })
        .map_err(|err| {
            error!("Failed to get activity for source with id {source_id}. err: {err:?}");

            Failure::server_error(err)
        })
    } else {
        return_password_error()
    }
}

#[delete("/activity")]
pub async fn clear_all_activities(data: AppData, req: HttpRequest) -> ApiResponse {
    let db = data.app_db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await? {
        activity::delete_all_activity(db)
            .await
            .map(|_| {
                info!("Deleted activity");

                Success::ok_message("Activities deleted successfully".into())
            })
            .map_err(|err| {
                error!("Failed to delete activity. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}

#[delete("/activity/{num}")]
pub async fn clear_activities(
    path: web::Path<u32>,
    data: AppData,
    req: HttpRequest,
) -> ApiResponse {
    let db = data.app_db.connect().unwrap();
    let num = path.into_inner();

    if is_logged_in(&req, db.clone()).await? {
        activity::delete_activity(db, num)
            .await
            .map(|success| {
                if success == (num as u64) {
                    info!("Deleted activity. count: {num}");
                } else {
                    warn!("Deleted activity. Rows affected not {num}, is: {success}");
                }

                Success::ok_message(format!("Activity deleted. count: {success}"))
            })
            .map_err(|err| {
                error!("Failed to delete activity. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}
