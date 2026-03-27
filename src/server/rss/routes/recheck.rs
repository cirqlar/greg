use actix_web::{HttpRequest, Responder, post};
use log::{error, info};

use crate::AppData;
use crate::auth::{is_logged_in, return_password_error};
use crate::rss::tasks::check::check_rss;
use crate::shared::{Failure, Success};

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
