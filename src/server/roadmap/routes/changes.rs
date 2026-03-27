use actix_web::{HttpRequest, get, web};
use log::{error, info};

use crate::AppData;
use crate::auth::{is_logged_in, return_password_error};
use crate::roadmap::queries::changes;
use crate::roadmap::types::RDBChange;
use crate::shared::{ApiResponse, Failure, Query, Success};

#[get("/roadmap_activity/{id}")]
pub async fn get_changes(
    data: AppData,
    path: web::Path<u32>,
    query: web::Query<Query>,
    req: HttpRequest,
) -> ApiResponse<Vec<RDBChange>> {
    let activity_id = path.into_inner();

    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };

    if query.demo || is_logged_in(&req, db.clone()).await? {
        changes::get_roadmap_changes(db, activity_id)
            .await
            .map(|changes| {
                info!("Got roadmap changes. activity_id: {activity_id}");

                Success::ok(changes)
            })
            .map_err(|err| {
                error!("Failed to get roadmap changes. activity_id: {activity_id}. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}
