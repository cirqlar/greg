use actix_web::{HttpRequest, post};
use log::{error, info};

use crate::AppData;
use crate::auth::{is_logged_in, return_password_error};
use crate::rss::tasks::check::check_rss;
use crate::shared::{ApiResponse, Failure, Success};

#[post("/recheck")]
pub async fn recheck(data: AppData, req: HttpRequest) -> ApiResponse {
    let db = data.app_db.connect().unwrap();

    if is_logged_in(&req, db).await? {
        check_rss(&data)
            .await
            .map(|_| {
                info!("Rechecked Sources");

                Success::ok_message("Rechecked Sources Successfully".into())
            })
            .map_err(|err| {
                error!("Failed to recheck sources. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}
