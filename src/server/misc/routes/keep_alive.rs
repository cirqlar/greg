use actix_web::get;

use crate::shared::{ApiResponse, Success};

#[get("/keep_alive")]
pub async fn keep_alive() -> ApiResponse {
    Ok(Success::ok_message("kept alive".into()))
}
