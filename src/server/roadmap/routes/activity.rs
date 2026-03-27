use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use log::{error, info};

use crate::server::AppData;
use crate::server::auth::{is_logged_in, return_password_error};
use crate::server::roadmap::queries::activity;
use crate::server::shared::{Failure, PaginationQuery};

#[get("/roadmap_activity")]
pub async fn get_roadmap_activity(
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
        match activity::get_roadmap_activity(db, query.count.unwrap_or(35), query.skip.unwrap_or(0))
            .await
        {
            Ok(activities) => {
                info!("Got roadmap activity");
                HttpResponse::Ok().json(activities)
            }
            Err(err) => {
                error!("Failed to get roadmap activity. err: {err:?}");
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
