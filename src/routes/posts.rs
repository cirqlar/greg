use std::env;

use crate::{
    tasks::check_sources::check_sources,
    types::{AddSource, AppState, Failure, FromRow, LoginInfo, Source, Success, LOGGED_IN_COOKIE},
    utils::{is_logged_in, return_password_error},
};
use actix_web::{cookie::Cookie, post, web, HttpRequest, HttpResponse, Responder};
use libsql_client::{args, Statement};
use time::OffsetDateTime;
use tokio::runtime::Handle;
use uuid::Uuid;

#[post("/login")]
async fn login(login_info: web::Json<LoginInfo>, data: web::Data<AppState>) -> impl Responder {
    if env::var("PASSWORD").expect("PASSWORD should be set") == login_info.password {
        let id = Uuid::new_v4();

        let db_handle = data.db_handle.lock().await;
        let Ok(_result) = db_handle
            .execute(Statement::with_args(
                "INSERT INTO logins (timestamp, key) VALUES (?, ?)",
                args!(
                    serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
                    id.to_string(),
                ),
            ))
            .await
        else {
            return HttpResponse::InternalServerError().json(Failure {
                message: "Issue logging in".into(),
            });
        };

        let c = Cookie::build(LOGGED_IN_COOKIE, id.to_string())
            .path("/")
            .secure(true)
            .http_only(true)
            .expires(None)
            .finish();

        HttpResponse::Ok().cookie(c).json(Success {
            message: "Log in Successful".into(),
        })
    } else {
        return_password_error()
    }
}

#[post("/recheck")]
async fn recheck(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let db_handle = data.db_handle.lock().await;

    if is_logged_in(&req, &db_handle).await {
        let rt = Handle::current();
        check_sources(rt, &data);
        HttpResponse::Ok().body("Recheck")
    } else {
        return_password_error()
    }
}

#[post("/source/new")]
pub async fn add_source(
    source: web::Json<AddSource>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let db_handle = data.db_handle.lock().await;

    if is_logged_in(&req, &db_handle).await {
        let result = db_handle
            .execute(Statement::with_args(
                "INSERT INTO sources (url, last_checked) VALUES (?, ?)",
                args!(
                    source.url.clone(),
                    serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
                ),
            ))
            .await;
        match result {
            Ok(mut success) => {
                let source_value = Source::from_row(success.rows.remove(0));
                HttpResponse::Ok().json(source_value)
            }
            Err(failure) => HttpResponse::InternalServerError().json(Failure {
                message: format!("Couldn't add source. Err: {}", failure),
            }),
        }
    } else {
        return_password_error()
    }
}
