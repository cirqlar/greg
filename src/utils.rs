use actix_web::{HttpRequest, HttpResponse};
use libsql_client::{args, Client, Statement};
use time::OffsetDateTime;
use tokio::sync::MutexGuard;

use crate::types::{Failure, LOGGED_IN_COOKIE};

pub fn return_password_error() -> HttpResponse {
    HttpResponse::Unauthorized().json(Failure {
        message: "Wrong password".into(),
    })
}

pub async fn is_logged_in(req: &HttpRequest, db_handle: &MutexGuard<'_, Client>) -> bool {
    if req.cookie(LOGGED_IN_COOKIE).is_some() {
        let Ok(mut result) = db_handle
            .execute(Statement::with_args(
                "SELECT * FROM logins WHERE key = ? LIMIT 1",
                args!(req.cookie(LOGGED_IN_COOKIE).unwrap().value()),
            ))
            .await
        else {
            return false;
        };

        if result.rows.is_empty() {
            return false;
        }
        let row = result.rows.remove(0);

        let Ok(timestamp) =
            serde_json::from_str::<OffsetDateTime>(row.try_column("timestamp").unwrap())
        else {
            return false;
        };

        let time_since = OffsetDateTime::now_utc() - timestamp;
        if time_since.whole_days() >= 1 {
            return false;
        }

        true
    } else {
        false
    }
}
