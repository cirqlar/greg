use std::env;

use crate::{
    tasks::check_sources::check_sources,
    types::{AddSource, AppState, Failure, LoginInfo, Success, LOGGED_IN_COOKIE},
    utils::{is_logged_in, return_password_error},
};
use actix_web::{cookie::Cookie, post, web, HttpRequest, HttpResponse, Responder};
use feed_rs::parser;
use libsql_client::{args, Statement};
use log::{error, info};
use time::OffsetDateTime;
use tokio::sync::mpsc;
use url::Url;
use uuid::Uuid;

#[post("/login")]
pub async fn login(login_info: web::Json<LoginInfo>, data: web::Data<AppState>) -> impl Responder {
    let password = match env::var("PASSWORD") {
        Ok(x) => x,
        Err(err) => {
            error!(
                "[Login] PASSWORD is not set. Env get failed with err: {}",
                err
            );
            return return_password_error();
        }
    };

    if password == login_info.password {
        let id = Uuid::new_v4();

        info!("[Login] Inserting login key");

        let (send, mut recv) = mpsc::channel(100);
        let _ = data
            .db_channel
            .send((
                Statement::with_args(
                    "INSERT INTO logins (timestamp, key) VALUES (?, ?)",
                    args!(
                        serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
                        id.to_string(),
                    ),
                ),
                send,
            ))
            .await;
        match recv.recv().await {
            Some(Ok(_x)) => {
                info!("[Login] Insert successful");
            }
            Some(Err(err)) => {
                error!("[Login] Inserting login key failed with err: {}", err);
                return HttpResponse::InternalServerError().json(Failure {
                    message: "Issue logging in".into(),
                });
            }
            None => unreachable!(),
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
        error!(
            "[Login] Login failed with wrong password: {}",
            login_info.password
        );
        return_password_error()
    }
}

#[post("/recheck")]
pub async fn recheck(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let (send, mut recv) = mpsc::channel(100);

    if is_logged_in(&req, &data, send.clone(), &mut recv).await {
        check_sources(&data).await;
        HttpResponse::Ok().body("Recheck")
    } else {
        return_password_error()
    }
}

async fn test_source(url: &str) -> Option<HttpResponse> {
    let _url = match Url::parse(url) {
        Ok(x) => x,
        Err(err) => {
            error!("[Add Source] Failed with error: {} for url: {}", err, url);
            return Some(HttpResponse::InternalServerError().json(Failure {
                message: format!("Couldn't add source. Err: {}", err),
            }));
        }
    };
    let res = match reqwest::get(url).await {
        Ok(x) => x,
        Err(err) => {
            error!(
                "[Add Source] Failed due to network error: {} for url: {}",
                err, url
            );
            return Some(HttpResponse::InternalServerError().json(Failure {
                message: format!("Couldn't add source. Err: {}", err),
            }));
        }
    };
    let _chan = match parser::parse(&(res.bytes().await.unwrap())[..]) {
        Ok(x) => x,
        Err(err) => {
            error!(
                "[Add Source] Failed due to result parse error: {} for url: {}",
                err, url
            );
            return Some(HttpResponse::InternalServerError().json(Failure {
                message: format!("Couldn't add source. Err: {}", err),
            }));
        }
    };

    None
}

#[post("/source/new")]
pub async fn add_source(
    source: web::Json<AddSource>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let (send, mut recv) = mpsc::channel(100);

    if is_logged_in(&req, &data, send.clone(), &mut recv).await {
        if let Some(ret) = test_source(&source.url).await {
            return ret;
        }

        info!("[Add Source] Inserting source to db");
        let _ = data
            .db_channel
            .send((
                Statement::with_args(
                    "INSERT INTO sources (url, last_checked) VALUES (?, ?)",
                    args!(
                        source.url.clone(),
                        serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
                    ),
                ),
                send,
            ))
            .await;
        match recv.recv().await {
            Some(Ok(success)) => {
                if success.rows_affected >= 1 {
                    info!("[Add Source] Inserting source successful");
                    HttpResponse::Ok().json(Success {
                        message: "Source added successfully".into(),
                    })
                } else {
                    error!(
                        "[Add Source] Rows affected in insert not 1, is: {}",
                        success.rows_affected
                    );
                    HttpResponse::InternalServerError().json(Failure {
                        message: "Unexpected issue adding source".into(),
                    })
                }
            }
            Some(Err(err)) => {
                error!("[Add Source] Inserting source failed with err: {}", err);
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't add source. Err: {}", err),
                })
            }
            None => unreachable!(),
        }
    } else {
        error!("[Add Source] Failed due to auth error");
        return_password_error()
    }
}
