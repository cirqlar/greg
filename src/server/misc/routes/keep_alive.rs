use actix_web::{Responder, get};

use crate::shared::Success;

#[get("/keep_alive")]
pub async fn keep_alive() -> impl Responder {
    (Success {
        message: "kept alive".into(),
    })
    .ok()
}
