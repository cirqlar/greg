use std::env;

use actix_web::{HttpRequest, HttpResponse, Responder, get};
use actix_web::{post, web};
use log::{error, info, warn};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppData;
use crate::auth::queries::login::save_login_id;
use crate::auth::{is_logged_in, make_auth_cookie, return_password_error};
use crate::shared::{Failure, Success};

#[get("/check-logged-in")]
pub async fn check_logged_in(data: AppData, req: HttpRequest) -> impl Responder {
    let db = data.app_db.connect().unwrap();
    let logged_in = is_logged_in(&req, db).await;
    let mut res = HttpResponse::Ok();
    if !logged_in {
        let mut c = make_auth_cookie("");
        c.make_removal();
        res.cookie(c);
    }

    res.json(logged_in)
}

#[derive(Deserialize)]
struct LoginInfo {
    password: String,
}

#[post("/login")]
pub async fn login(login_info: web::Json<LoginInfo>, data: AppData) -> impl Responder {
    let password = match env::var("PASSWORD") {
        Ok(x) => x,
        Err(err) => {
            error!("Failed to get PASSWORD env variable. err: {err:?}");
            return return_password_error();
        }
    };

    if password == login_info.password {
        let id = Uuid::new_v4();
        let db = data.app_db.connect().unwrap();

        match save_login_id(db, &id).await {
            Ok(x) => {
                if x == 1 {
                    info!("Inserted login key.");
                } else {
                    warn!("Inserted login key. Rows affected in insert not 1, is {x}");
                }

                let string_id = id.to_string();
                let c = make_auth_cookie(&string_id);

                HttpResponse::Ok().cookie(c).json(Success {
                    message: "Log in Successful".into(),
                })
            }
            Err(err) => {
                error!("Failed to insert login key. err: {err:?}");
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
