use actix_web::{HttpRequest, get, web};
use log::{error, info};

use crate::AppData;
use crate::auth::{is_logged_in, return_password_error};
use crate::roadmap::queries::activity;
use crate::roadmap::types::RoadmapActivity;
use crate::shared::{ApiResponse, Failure, PaginationQuery, Success};

#[get("/roadmap_activity")]
pub async fn get_roadmap_activity(
    data: AppData,
    query: web::Query<PaginationQuery>,
    req: HttpRequest,
) -> ApiResponse<Vec<RoadmapActivity>> {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };

    if query.demo || is_logged_in(&req, db.clone()).await? {
        activity::get_roadmap_activity(db, query.count.unwrap_or(35), query.skip.unwrap_or(0))
            .await
            .map(|activities| {
                info!("Got roadmap activity");

                Success::ok(activities)
            })
            .map_err(|err| {
                error!("Failed to get roadmap activity. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}
