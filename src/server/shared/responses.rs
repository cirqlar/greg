use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct Success {
    pub message: String,
}

impl Success {
    pub fn ok(self) -> HttpResponse {
        HttpResponse::Ok().json(self)
    }
}

#[derive(Serialize)]
pub struct Failure {
    pub message: String,
}

impl Failure {
    pub fn server_error(self) -> HttpResponse {
        HttpResponse::InternalServerError().json(self)
    }

    pub fn bad_request(self) -> HttpResponse {
        HttpResponse::BadRequest().json(self)
    }
}
