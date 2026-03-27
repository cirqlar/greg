use actix_web::{HttpRequest, delete, get, post, web};
use log::{error, info, warn};
use serde::Deserialize;

use crate::AppData;
use crate::auth::{is_logged_in, return_password_error};
use crate::rss::Source;
use crate::rss::queries::sources;
use crate::rss::tasks::check::get_source;
use crate::shared::{ApiResponse, Query};
use crate::shared::{Failure, Success};

#[get("/sources")]
pub async fn get_sources(
    data: AppData,
    query: web::Query<Query>,
    req: HttpRequest,
) -> ApiResponse<Vec<Source>> {
    let db = if query.demo {
        data.demo_db.connect().unwrap()
    } else {
        data.app_db.connect().unwrap()
    };

    if query.demo || is_logged_in(&req, db.clone()).await? {
        sources::get_sources(db)
            .await
            .map(|sources| {
                info!("Got sources");

                Success::ok(sources)
            })
            .map_err(|err| {
                error!("Failed to get sources. err: {err:?}");

                Failure::server_error(err)
            })
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
) -> ApiResponse {
    let db = data.app_db.connect().unwrap();

    if is_logged_in(&req, db.clone()).await? {
        if let Err(err) = get_source(&source.url, reqwest::Client::new()).await {
            error!("Failed to verify source. err: {err:?}");

            return Err(Failure::bad_request(err));
        }

        sources::add_source(db, source.url.clone())
            .await
            .map(|success| {
                if success == 1 {
                    info!("Inserted source. url: {}", source.url);
                } else {
                    warn!(
                        "Inserted source. url: {}. Rows affected in insert not 1, is: {success}",
                        source.url
                    );
                }

                Success::ok_message("Source added successfully".into())
            })
            .map_err(|err| {
                error!("Failed to insert source. url: {} err: {err:?}", source.url);

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}

#[post("/source/{id}/enable/{enabled}")]
pub async fn enable_source(
    path: web::Path<(u32, bool)>,
    data: AppData,
    req: HttpRequest,
) -> ApiResponse {
    let (source_id, enabled) = path.into_inner();

    let db = data.app_db.connect().unwrap();

    if is_logged_in(&req, db.clone()).await? {
        sources::enable_source(db, source_id, enabled)
            .await
            .map(|success| {
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

                Success::ok_message("Source updated successfully".into())
            })
            .map_err(|err| {
                error!(
                    "Failed to {} source. id: {source_id}. err: {err:?}",
                    if enabled { "enable" } else { "disable" }
                );

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}

#[delete("/source/{id}")]
pub async fn delete_source(path: web::Path<u32>, data: AppData, req: HttpRequest) -> ApiResponse {
    let db = data.app_db.connect().unwrap();
    let source_id = path.into_inner();

    if is_logged_in(&req, db.clone()).await? {
        sources::delete_source(db, source_id).await
        .map(|success| {
            if success == 1 {
                    info!("Deleted source. id: {source_id}");
                } else {
                    warn!(
                        "Deleted source. id: {source_id}. Rows affected in deletion not 1, is: {success}"
                    );
                }

                Success::ok_message("Source deleted successfully".into())
        })
        .map_err(|err| {
            error!("Failed to delete source. id: {source_id}. err: {err:?}");

            Failure::server_error(err)
        })
    } else {
        return_password_error()
    }
}
