use actix_web::{HttpRequest, post};
use log::{error, info};

use crate::AppData;
use crate::auth::{is_logged_in, return_password_error};
use crate::roadmap::tasks::check::check_roadmap;
use crate::shared::{ApiResponse, Failure, Success};

#[post("/recheck_roadmap")]
pub async fn recheck_roadmap(data: AppData, req: HttpRequest) -> ApiResponse {
    let db = data.app_db.connect().unwrap();
    if is_logged_in(&req, db).await? {
        check_roadmap(&data)
            .await
            .map(|_| {
                info!("Rechecked Roadmap");

                Success::ok_message("Rechecked Roadmap Successfully".into())
            })
            .map_err(|err| {
                error!("Failed to recheck roadmap. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}
