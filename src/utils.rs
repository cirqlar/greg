use actix_web::HttpResponse;

use crate::types::Failure;

pub fn return_password_error() -> HttpResponse {
    HttpResponse::Unauthorized().json(Failure {
        error: "Wrong password".into(),
    })
}
