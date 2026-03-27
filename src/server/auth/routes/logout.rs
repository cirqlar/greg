use actix_web::{HttpResponse, Responder, delete};

use crate::server::auth::make_auth_cookie;
use crate::server::shared::Success;

#[delete("/logout")]
pub async fn logout() -> impl Responder {
    let mut c = make_auth_cookie("");

    c.make_removal();

    HttpResponse::Ok().cookie(c).json(Success {
        message: "Successfully logged out".into(),
    })
}
