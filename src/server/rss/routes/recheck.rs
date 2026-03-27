use actix_web::{HttpRequest, Responder, post};
use log::{error, info};

use crate::server::AppData;
use crate::server::auth::{is_logged_in, return_password_error};
use crate::server::rss::tasks::check::check_rss;
use crate::server::shared::{Failure, Success};

#[post("/recheck")]
pub async fn recheck(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.app_db.connect().unwrap();
    if is_logged_in(&req, db).await {
        match check_rss(&data).await {
            Ok(_) => {
                info!("Rechecked Sources");

                (Success {
                    message: "Rechecked Sources Successfully".into(),
                })
                .ok()
            }
            Err(err) => {
                error!("Failed to recheck sources. err: {err:?}");

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
