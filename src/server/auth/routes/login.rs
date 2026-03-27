use std::env;

use actix_web::{HttpRequest, get};
use actix_web::{post, web};
use log::{error, info, warn};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppData;
use crate::auth::queries::login::save_login_id;
use crate::auth::{base_is_logged_in, make_auth_cookie, return_password_error};
use crate::shared::{ApiResponse, Failure, Success};

#[get("/check-logged-in")]
pub async fn check_logged_in(data: AppData, req: HttpRequest) -> ApiResponse<bool> {
    let db = data.app_db.connect().unwrap();

    base_is_logged_in(&req, db)
        .await
        .map(|logged_in| {
            let mut res = Success::ok(logged_in);

            if !logged_in {
                let mut c = make_auth_cookie("".into());
                c.make_removal();

                res = res.with_cookie(c);
            }

            res
        })
        .map_err(|err| {
            error!("Failed to check logged in state. err: {err:?}");

            let mut c = make_auth_cookie("".into());
            c.make_removal();

            Failure::server_error(err).with_cookie(c)
        })
}

#[derive(Deserialize)]
struct LoginInfo {
    password: String,
}

#[post("/login")]
pub async fn login(login_info: web::Json<LoginInfo>, data: AppData) -> ApiResponse {
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

        save_login_id(db, &id)
            .await
            .map(|x| {
                if x == 1 {
                    info!("Inserted login key.");
                } else {
                    warn!("Inserted login key. Rows affected in insert not 1, is {x}");
                }

                let c = make_auth_cookie(id.to_string());

                Success::ok_message("Log in Successful".into()).with_cookie(c)
            })
            .map_err(|err| {
                error!("Failed to insert login key. err: {err:?}");

                Failure::server_error(err)
            })
    } else {
        return_password_error()
    }
}
