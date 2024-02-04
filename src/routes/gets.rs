use crate::{
    db::ToSerdeJsonValue,
    types::{Activity, AppState, Failure, Source, LOGGED_IN, LOGGED_IN_VALUE},
    utils::return_password_error,
};

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::{de::value::MapDeserializer, Deserialize};

#[get("/sources")]
async fn get_sources(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    if req.cookie(LOGGED_IN).is_some() && req.cookie(LOGGED_IN).unwrap().value() == LOGGED_IN_VALUE
    {
        let db_handle = data.db_handle.lock().await;
        let result = db_handle.execute("SELECT * FROM sources").await;
        match result {
            Ok(success) => {
                let sources = success
                    .rows
                    .into_iter()
                    .filter_map(|row| {
                        Source::deserialize(MapDeserializer::new(
                            row.value_map
                                .into_iter()
                                .map(|(key, value)| (key, value.convert())),
                        ))
                        .ok()
                    })
                    .collect::<Vec<_>>();

                HttpResponse::Ok().json(sources)
            }
            Err(failure) => HttpResponse::InternalServerError().json(Failure {
                error: format!("Couldn't get sources. Err: {}", failure),
            }),
        }
    } else {
        return_password_error()
    }
}

#[get("/activity")]
async fn get_activity(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    if req.cookie(LOGGED_IN).is_some() && req.cookie(LOGGED_IN).unwrap().value() == LOGGED_IN_VALUE
    {
        let db_handle = data.db_handle.lock().await;
        let result = db_handle.execute("SELECT * FROM activities").await;
        match result {
            Ok(success) => {
                let activities = success
                    .rows
                    .into_iter()
                    .filter_map(|row| {
                        Activity::deserialize(MapDeserializer::new(
                            row.value_map
                                .into_iter()
                                .map(|(key, value)| (key, value.convert())),
                        ))
                        .ok()
                    })
                    .collect::<Vec<_>>();

                HttpResponse::Ok().json(activities)
            }
            Err(failure) => HttpResponse::InternalServerError().json(Failure {
                error: format!("Couldn't get activities. Err: {}", failure),
            }),
        }
    } else {
        return_password_error()
    }
}
