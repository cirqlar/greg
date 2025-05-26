use crate::{
    queries::{
        roadmap::{self, get_most_recent_roadmap_tabs, get_roadmap_activities},
        sources,
    },
    types::{AppData, Failure, LOGGED_IN_COOKIE},
    utils::{is_logged_in, return_password_error},
};

use actix_web::{HttpRequest, HttpResponse, Responder, cookie::Cookie, get};
use log::{error, info};
use serde_json::json;

#[get("/check-logged-in")]
pub async fn check_logged_in(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.db.connect().unwrap();
    let logged_in = is_logged_in(&req, db).await;
    let mut res = HttpResponse::Ok();
    if !logged_in {
        let mut c = Cookie::build(LOGGED_IN_COOKIE, "").finish();
        c.make_removal();
        res.cookie(c);
    }

    res.json(logged_in)
}

#[get("/keep_alive")]
pub async fn keep_alive() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "kept_alive": true,
    }))
}

#[get("/sources")]
pub async fn get_sources(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await {
        info!("[Get Sources] Getting sources from db");
        match sources::get_sources(db).await {
            Ok(sources) => {
                info!("[Get Sources] Got sources successfully");
                HttpResponse::Ok().json(sources)
            }
            Err(err) => {
                error!("[Get Sources] Getting sources failed with err: {}", err);
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't get sources. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Get Sources] Failed due to auth error");
        return_password_error()
    }
}

#[get("/activity")]
pub async fn get_activity(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await {
        info!("[Get Activity] Getting activities from db");
        match sources::get_activity(db, 35, 0).await {
            Ok(activities) => {
                info!("[Get Activity] Got activities successfully");
                HttpResponse::Ok().json(activities)
            }
            Err(err) => {
                error!("[Get Activity] Getting activities failed with err: {}", err);
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't get activities. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Get Activity] Failed due to auth error");
        return_password_error()
    }
}

#[get("/roadmap_activity")]
pub async fn get_roadmap_activity(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await {
        info!("[Get Roadmap Activity] Getting activities from db");

        match get_roadmap_activities(db, 35, 0).await {
            Ok(activities) => {
                info!("[Get Roadmap Activity] Got activities successfully");
                HttpResponse::Ok().json(activities)
            }
            Err(err) => {
                error!(
                    "[Get Roadmap Activity] Getting roadmap activities failed with err: {}",
                    err
                );
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't get roadmap activities. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Get Roadmap Activity] Failed due to auth error");
        return_password_error()
    }
}

#[get("/most_recent_tabs")]
pub async fn get_most_recent_tabs(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await {
        info!("[Get Roadmap Tabs] Getting most recent tabs from db");

        match get_most_recent_roadmap_tabs(db).await {
            Ok(tabs) => {
                info!("[Get Roadmap Tabs] Got tabs successfully");
                HttpResponse::Ok().json(tabs)
            }
            Err(err) => {
                error!(
                    "[Get Roadmap Tabs] Getting roadmap tabs failed with err: {}",
                    err
                );
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't get roadmap tabs. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Get Roadmap Tabs] Failed due to auth error");
        return_password_error()
    }
}

#[get("/watched_tabs")]
pub async fn get_watched_tabs(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.db.connect().unwrap();
    if is_logged_in(&req, db.clone()).await {
        info!("[Get Watched Tabs] Getting watched tabs from db");

        match roadmap::get_watched_tabs(db).await {
            Ok(watched_tabs) => {
                info!("[Get Watched Tabs] Got watched tabs successfully");
                HttpResponse::Ok().json(watched_tabs)
            }
            Err(err) => {
                error!(
                    "[Get Watched Tabs] Getting watched tabs failed with err: {}",
                    err
                );
                HttpResponse::InternalServerError().json(Failure {
                    message: format!("Couldn't get watched tabs. Err: {}", err),
                })
            }
        }
    } else {
        error!("[Get Watched Tabs] Failed due to auth error");
        return_password_error()
    }
}
