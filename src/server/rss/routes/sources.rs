use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, post, web};
use log::{error, info, warn};
use serde::Deserialize;

use crate::server::AppData;
use crate::server::rss::queries::sources;
use crate::server::rss::tasks::check::get_source;
use crate::server::shared::Query;
use crate::server::shared::auth::{is_logged_in, return_password_error};
use crate::server::shared::{Failure, Success};

#[get("/sources")]
pub async fn get_sources(
    data: AppData,
    query: web::Query<Query>,
    req: HttpRequest,
) -> impl Responder {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };

    if query.demo || is_logged_in(&req, db.clone()).await {
        match sources::get_sources(db).await {
            Ok(sources) => {
                info!("Got sources");
                HttpResponse::Ok().json(sources)
            }
            Err(err) => {
                error!("Failed to get sources. err: {err:?}");
                (Failure {
                    message: format!("{err}"),
                })
                .server_error()
            }
        }
    } else {
        return_password_error()
    }
}

#[derive(Deserialize)]
pub struct AddSource {
    pub url: String,
}

#[post("/source/new")]
pub async fn add_source(
    source: web::Json<AddSource>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.app_db.connect().unwrap();

    if is_logged_in(&req, db.clone()).await {
        if let Err(err) = get_source(&source.url, reqwest::Client::new()).await {
            error!("Failed to verify source. err: {err:?}");
            return (Failure {
                message: format!("{err}"),
            })
            .bad_request();
        }

        match sources::add_source(db, source.url.clone()).await {
            Ok(success) => {
                if success == 1 {
                    info!("Inserted source. url: {}", source.url);
                } else {
                    warn!(
                        "Inserted source. url: {}. Rows affected in insert not 1, is: {success}",
                        source.url
                    );
                }

                (Success {
                    message: "Source added successfully".into(),
                })
                .ok()
            }
            Err(err) => {
                error!("Failed to insert source. url: {} err: {err:?}", source.url);
                (Failure {
                    message: format!("{err}"),
                })
                .server_error()
            }
        }
    } else {
        return_password_error()
    }
}

#[post("/source/{id}/enable/{enabled}")]
pub async fn enable_source(
    path: web::Path<(u32, bool)>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let (source_id, enabled) = path.into_inner();

    let db = data.app_db.connect().unwrap();

    if is_logged_in(&req, db.clone()).await {
        match sources::enable_source(db, source_id, enabled).await {
            Ok(success) => {
                if success == 1 {
                    info!(
                        "{} source. id: {source_id}",
                        if enabled { "Enabled" } else { "Disabled" }
                    );
                } else {
                    warn!(
                        "{} source. id: {source_id}. Rows affected in update not 1, is: {success}",
                        if enabled { "Enabled" } else { "Disabled" }
                    );
                }

                (Success {
                    message: "Source updated successfully".into(),
                })
                .ok()
            }
            Err(err) => {
                error!(
                    "Failed to {} source. id: {source_id}. err: {err:?}",
                    if enabled { "enable" } else { "disable" }
                );

                (Failure {
                    message: format!("{err}"),
                })
                .server_error()
            }
        }
    } else {
        return_password_error()
    }
}

#[delete("/source/{id}")]
pub async fn delete_source(
    path: web::Path<u32>,
    data: AppData,
    req: HttpRequest,
) -> impl Responder {
    let db = data.app_db.connect().unwrap();
    let source_id = path.into_inner();

    if is_logged_in(&req, db.clone()).await {
        match sources::delete_source(db, source_id).await {
            Ok(success) => {
                if success == 1 {
                    info!("Deleted source. id: {source_id}");
                } else {
                    warn!(
                        "Deleted source. id: {source_id}. Rows affected in deletion not 1, is: {success}"
                    );
                }

                (Success {
                    message: "Source deleted successfully".into(),
                })
                .ok()
            }
            Err(err) => {
                error!("Failed to delete source. id: {source_id}. err: {err:?}");

                (Failure {
                    message: format!("{err}"),
                })
                .server_error()
            }
        }
    } else {
        return_password_error()
    }
}
