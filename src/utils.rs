use actix_web::{HttpRequest, HttpResponse};
use libsql_client::{args, Client, Statement};
use log::{error, info};
use time::OffsetDateTime;
use tokio::sync::MutexGuard;

use crate::types::{Failure, LOGGED_IN_COOKIE};

pub fn return_password_error() -> HttpResponse {
    HttpResponse::Unauthorized().json(Failure {
        message: "Wrong password".into(),
    })
}

pub async fn is_logged_in(req: &HttpRequest, db_handle: &MutexGuard<'_, Client>) -> bool {
    match req.cookie(LOGGED_IN_COOKIE) {
        None => {
            info!("[LoggedInCheck] No Loggged in cookie set");
            false
        }
        Some(res) => match db_handle
            .execute(Statement::with_args(
                "SELECT * FROM logins WHERE key = ? LIMIT 1",
                args!(res.value()),
            ))
            .await
        {
            Err(err) => {
                info!(
                    "[LoggedInCheck] Searching key in db failed with error: {}",
                    err
                );
                false
            }
            Ok(res) => {
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
        },
    }
}
