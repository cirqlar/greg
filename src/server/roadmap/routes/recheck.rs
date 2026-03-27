use actix_web::{HttpRequest, Responder, post};
use log::{error, info};

use crate::server::AppData;
use crate::server::auth::{is_logged_in, return_password_error};
use crate::server::roadmap::tasks::check::check_roadmap;
use crate::server::shared::{Failure, Success};

#[post("/recheck_roadmap")]
pub async fn recheck_roadmap(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.app_db.connect().unwrap();
    if is_logged_in(&req, db).await {
        match check_roadmap(&data).await {
            Ok(_) => {
                info!("Rechecked Roadmap");

                (Success {
                    message: "Rechecked Roadmap Successfully".into(),
                })
                .ok()
            }
            Err(err) => {
                error!("Failed to recheck roadmap. err: {err:?}");

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
