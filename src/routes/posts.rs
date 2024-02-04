use std::env;

use crate::{
    db::ToSerdeJsonValue,
    tasks::check_sources::check_sources,
    types::{AddSource, AppState, Failure, LoginInfo, Source, Success, LOGGED_IN, LOGGED_IN_VALUE},
    utils::return_password_error,
};
use actix_web::{cookie::Cookie, post, web, HttpRequest, HttpResponse, Responder};
use libsql_client::{args, Statement};
use serde::{de::value::MapDeserializer, Deserialize};
use time::OffsetDateTime;

#[post("/login")]
async fn login(login_info: web::Json<LoginInfo>) -> impl Responder {
    if env::var("PASSWORD").expect("PASSWORD should be set") == login_info.password {
        let c = Cookie::build(LOGGED_IN, LOGGED_IN_VALUE)
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
async fn recheck() -> impl Responder {
    check_sources().await;
    HttpResponse::Ok().body("Recheck")
}

#[post("/source/new")]
pub async fn add_source(
    source: web::Json<AddSource>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if req.cookie(LOGGED_IN).is_some() && req.cookie(LOGGED_IN).unwrap().value() == LOGGED_IN_VALUE
    {
        let db_handle = data.db_handle.lock().await;
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
                let source_map = success.rows.remove(0).value_map;
                let Ok(source_value) = Source::deserialize(MapDeserializer::new(
                    source_map
                        .into_iter()
                        .map(|(key, val)| (key, val.convert())),
                )) else {
                    return HttpResponse::InternalServerError().json(Failure {
                        error: "Error returning source".into(),
                    });
                };
                HttpResponse::Ok().json(source_value)
            }
            Err(failure) => HttpResponse::InternalServerError().json(Failure {
                error: format!("Couldn't add source. Err: {}", failure),
            }),
        }
    } else {
        return_password_error()
    }
}
