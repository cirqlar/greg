use actix_web::{HttpRequest, HttpResponse, cookie::Cookie};
use libsql::Connection;
use log::{error, info};
use time::OffsetDateTime;

use crate::{
    db::LOGINS_T,
    types::{Failure, LOGGED_IN_COOKIE},
};

pub fn return_password_error() -> HttpResponse {
    let mut c = Cookie::build(LOGGED_IN_COOKIE, "").finish();
    c.make_removal();
    HttpResponse::Unauthorized().cookie(c).json(Failure {
        message: "Wrong password".into(),
    })
}

pub async fn is_logged_in(req: &HttpRequest, db: Connection) -> bool {
    match req.cookie(LOGGED_IN_COOKIE) {
        None => {
            info!("[LoggedInCheck] No Loggged in cookie set");
            false
        }
        Some(res) => {
            let result = db
                .query(
                    &format!("SELECT timestamp FROM {LOGINS_T} WHERE key = ?1 LIMIT 1"),
                    [res.value()],
                )
                .await;

            match result {
                Err(err) => {
                    info!(
                        "[LoggedInCheck] Searching key in db failed with error: {}",
                        err
                    );
                    false
                }
                Ok(mut res) => {
                    let res = res.next().await;
                    let Ok(rows) = res else {
                        info!(
                            "[LoggedInCheck] Searching key in db failed with error: {}",
                            res.unwrap_err()
                        );
                        return false;
                    };
                    let Some(row) = rows else {
                        info!("[LoggedInCheck] Key not in db");
                        return false;
                    };

                    let timestamp = row.get_str(0).unwrap();
                    match serde_json::from_str::<OffsetDateTime>(timestamp) {
                        Err(err) => {
                            error!(
                                "[LoggedInCheck] Timestamp: {} couldn't be parsed with err: {}",
                                timestamp, err
                            );
                            false
                        }
                        Ok(timestamp) => (OffsetDateTime::now_utc() - timestamp).whole_hours() < 1,
                    }
                }
            }
        }
    }
}
