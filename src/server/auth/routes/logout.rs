use actix_web::delete;

use crate::auth::make_auth_cookie;
use crate::shared::{ApiResponse, Success};

#[delete("/logout")]
pub async fn logout() -> ApiResponse {
    let mut c = make_auth_cookie("".into());

    c.make_removal();

    Ok(Success::ok_message("Successfully logged out".into()).with_cookie(c))
}
