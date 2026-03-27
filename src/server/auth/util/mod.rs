use actix_web::{HttpRequest, cookie::Cookie};
use libsql::Connection;
use log::error;
use time::OffsetDateTime;

use super::queries::login::get_key_timestamp;
use crate::shared::{ApiResponse, DatabaseError, Failure};

pub const LOGGED_IN_COOKIE: &str = "logged_in";

pub fn make_auth_cookie(value: String) -> Cookie<'static> {
    Cookie::build(LOGGED_IN_COOKIE, value)
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(None)
        .finish()
}

pub fn return_password_error<T: serde::Serialize>() -> ApiResponse<T> {
    let mut c = make_auth_cookie("".into());

    c.make_removal();

    error!("Unauthorized request");

    Err(Failure::unauthorized_message("Wrong password".into()).with_cookie(c))
}

pub async fn base_is_logged_in(req: &HttpRequest, db: Connection) -> Result<bool, DatabaseError> {
    match req.cookie(LOGGED_IN_COOKIE) {
        None => Ok(false),
        Some(res) => match get_key_timestamp(db, res.value()).await? {
            None => Ok(false),
            Some(timestamp) => Ok((OffsetDateTime::now_utc() - timestamp).whole_hours() < 1),
        },
    }
}

pub async fn is_logged_in(req: &HttpRequest, db: Connection) -> Result<bool, Failure> {
    base_is_logged_in(req, db).await.map_err(|err| {
        error!("Failed to check if logged in. err: {err:?}");
        Failure::server_error(err)
    })
}
