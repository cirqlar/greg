use std::env;

use crate::{
    db::{LOGINS_T, R_WATCHED_TABS_T, SOURCES_T},
    tasks::{check_roadmap::check_roadmap, check_sources::check_sources},
    types::{AddSource, AppData, Failure, LOGGED_IN_COOKIE, LoginInfo, Success},
    utils::{is_logged_in, return_password_error},
};
use actix_web::{HttpRequest, HttpResponse, Responder, cookie::Cookie, post, web};
use feed_rs::parser;
use log::{error, info};
use time::{OffsetDateTime, ext::NumericalDuration};
use url::Url;
use uuid::Uuid;

#[post("/login")]
pub async fn login(login_info: web::Json<LoginInfo>, data: AppData) -> impl Responder {
    let password = match env::var("PASSWORD") {
        Ok(x) => x,
        Err(err) => {
            error!(
                "[Login] PASSWORD is not set. Env get failed with err: {err}"
            );
            return return_password_error();
        }
    };

    if password == login_info.password {
        let id = Uuid::new_v4();

        info!("[Login] Inserting login key");
        let db = data.db.connect().unwrap();

        let result = db
            .execute(
                &format!("INSERT INTO {LOGINS_T} (timestamp, key) VALUES (?1, ?2)"),
                [
                    serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
                    id.to_string(),
                ],
            )
            .await;

        match result {
            Ok(_x) => {
                info!("[Login] Insert successful");
            }
            Err(err) => {
                error!("[Login] Inserting login key failed with err: {err}");
                return HttpResponse::InternalServerError().json(Failure {
                    message: "Issue logging in".into(),
                });
            }
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
pub async fn recheck(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db).await {
        check_sources(&data).await;
        HttpResponse::Ok().json(Success {
            message: "Rechecked Sources Successfully".into(),
        })
    } else {
        return_password_error()
    }
}

#[post("/recheck_roadmap")]
pub async fn recheck_roadmap(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db).await {
        check_roadmap(&data).await;
        HttpResponse::Ok().json(Success {
            message: "Rechecked Roadmap Successfully".into(),
        })
    } else {
        return_password_error()
    }
}

async fn test_source(url: &str) -> Option<HttpResponse> {
    let _url = match Url::parse(url) {
        Ok(x) => x,
        Err(err) => {
            error!("[Add Source] Failed with error: {err} for url: {url}");
            return Some(HttpResponse::InternalServerError().json(Failure {
                message: format!("Couldn't add source. Err: {err}"),
            }));
        }
    };
    let res = match reqwest::get(url).await {
        Ok(x) => x,
        Err(err) => {
            error!(
                "[Add Source] Failed due to network error: {err} for url: {url}"
            );
            return Some(HttpResponse::InternalServerError().json(Failure {
                message: format!("Couldn't add source. Err: {err}"),
            }));
        }
    };
    let _chan = match parser::parse(&(res.bytes().await.unwrap())[..]) {
        Ok(x) => x,
        Err(err) => {
            error!(
                "[Add Source] Failed due to result parse error: {err} for url: {url}"
            );
            return Some(HttpResponse::InternalServerError().json(Failure {
                message: format!("Couldn't add source. Err: {err}"),
            }));
        }
    };

    None
}

#[post("/source/new")]
pub async fn add_source(
    source: web::Json<AddSource>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await {
        if let Some(ret) = test_source(&source.url).await {
            return ret;
        }

        info!("[Add Source] Inserting source to db");
        let result = db
            .execute(
                &format!("INSERT INTO {SOURCES_T} (url, last_checked) VALUES (?1, ?2)"),
                [
                    source.url.clone(),
                    serde_json::to_string(&(OffsetDateTime::now_utc() - 1.hours())).unwrap(),
                ],
            )
            .await;

        match result {
            Ok(success) => {
                if success >= 1 {
                    info!("[Add Source] Inserting source successful");
                    HttpResponse::Ok().json(Success {
                        message: "Source added successfully".into(),
                    })
                } else {
                    error!(
                        "[Add Source] Rows affected in insert not 1, is: {success}"
                    );
                    HttpResponse::InternalServerError().json(Failure {
                        message: "Unexpected issue adding source".into(),
                    })
                }
            }
            Err(err) => {
                error!("[Add Source] Inserting source failed with err: {err}");
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't add source. Err: {err}"),
                })
            }
        }
    } else {
        error!("[Add Source] Failed due to auth error");
        return_password_error()
    }
}

#[post("/source/{id}/enable/{enabled}")]
pub async fn enable_source(
    path: web::Path<(u32, bool)>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let (source_id, new_enabled) = path.into_inner();

    let db = data.db.connect().unwrap();

    if is_logged_in(&req, db.clone()).await {
        info!(
            "[Update Source] {} source",
            if new_enabled { "Enabling" } else { "Disabling" }
        );

        let result = db
            .execute(
                &format!("UPDATE {SOURCES_T} SET failed_count = ?1, enabled = ?2 WHERE id = ?3"),
                (0, if new_enabled { 1 } else { 0 }, source_id),
            )
            .await;

        match result {
            Ok(success) => {
                if success >= 1 {
                    info!("[Update Source] Updated source successfully");
                    HttpResponse::Ok().json(Success {
                        message: "Source updated successfully".into(),
                    })
                } else {
                    error!(
                        "[Update Source] Rows affected in insert not 1, is: {success}"
                    );
                    HttpResponse::InternalServerError().json(Failure {
                        message: "Unexpected issue updating source".into(),
                    })
                }
            }
            Err(err) => {
                error!("[Update Source] Updating source failed with err: {err}");
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't update source. Err: {err}"),
                })
            }
        }
    } else {
        error!("[Update Source] Failed due to auth error");
        return_password_error()
    }
}

#[post("/watched_tabs/add/{tab_id}")]
pub async fn add_watched_tab(
    path: web::Path<String>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await {
        let tab_id = path.into_inner();

        info!("[Add Watched Tab] Inserting tab to db");
        let result = db
            .execute(
                &format!(
                    "INSERT INTO {R_WATCHED_TABS_T} (tab_roadmap_id, timestamp) VALUES (?1, ?2)"
                ),
                [
                    tab_id,
                    serde_json::to_string(&OffsetDateTime::now_utc()).unwrap(),
                ],
            )
            .await;

        match result {
            Ok(success) => {
                if success >= 1 {
                    info!("[Add Watched Tab] Inserting watched tab successful");
                    HttpResponse::Ok().json(Success {
                        message: "Watched tab added successfully".into(),
                    })
                } else {
                    error!(
                        "[Add Watched Tab] Rows affected in insert not 1, is: {success}"
                    );
                    HttpResponse::InternalServerError().json(Failure {
                        message: "Unexpected issue adding watched tab".into(),
                    })
                }
            }
            Err(err) => {
                error!(
                    "[Add Watched Tab] Inserting watched tab failed with err: {err}"
                );
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't add Watched Tab. Err: {err}"),
                })
            }
        }
    } else {
        error!("[Add Watched Tab] Failed due to auth error");
        return_password_error()
    }
}
