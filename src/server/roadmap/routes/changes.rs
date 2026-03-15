use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use log::{error, info};

use crate::server::AppData;
use crate::server::roadmap::queries::changes;
use crate::server::shared::auth::{is_logged_in, return_password_error};
use crate::server::shared::{Failure, Query};

#[get("/roadmap_activity/{id}")]
pub async fn get_changes(
    data: AppData,
    path: web::Path<u32>,
    query: web::Query<Query>,
    req: HttpRequest,
) -> impl Responder {
    let activity_id = path.into_inner();

    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };

    if query.demo || is_logged_in(&req, db.clone()).await {
        match changes::get_roadmap_changes(db, activity_id).await {
            Ok(changes) => {
                info!("Got roadmap changes. activity_id: {activity_id}");
                HttpResponse::Ok().json(changes)
            }
            Err(err) => {
                error!("Failed to get roadmap changes. activity_id: {activity_id}. err: {err:?}");
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
