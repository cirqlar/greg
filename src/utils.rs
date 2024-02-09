use actix_web::{cookie::Cookie, web, HttpRequest, HttpResponse};
use libsql_client::{args, Statement};
use log::{error, info};
use time::OffsetDateTime;

use crate::types::{AppState, DbReturnReciever, DbReturnSender, Failure, LOGGED_IN_COOKIE};

pub fn return_password_error() -> HttpResponse {
    let mut c = Cookie::build(LOGGED_IN_COOKIE, "").finish();
    c.make_removal();
    HttpResponse::Unauthorized().cookie(c).json(Failure {
        message: "Wrong password".into(),
    })
}

pub async fn is_logged_in(
    req: &HttpRequest,
    data: &web::Data<AppState>,
    send: DbReturnSender,
    recv: &mut DbReturnReciever,
) -> bool {
    match req.cookie(LOGGED_IN_COOKIE) {
        None => {
            info!("[LoggedInCheck] No Loggged in cookie set");
            false
        }
        Some(res) => {
            let _ = data
                .db_channel
                .send((
                    Statement::with_args(
                        "SELECT * FROM logins WHERE key = ? LIMIT 1",
                        args!(res.value()),
                    ),
                    send,
                ))
                .await;
            match recv.recv().await {
                Some(Err(err)) => {
                    info!(
                        "[LoggedInCheck] Searching key in db failed with error: {}",
                        err
                    );
                    false
                }
                Some(Ok(res)) => {
                    if res.rows.is_empty() {
                        info!("[LoggedInCheck] Key not in db");
                        return false;
                    }

                    let timestamp = res.rows[0].try_column("timestamp").unwrap();
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
                None => unreachable!(),
            }
        }
    }
}
