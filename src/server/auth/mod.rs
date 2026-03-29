use actix_web::{Scope, web::scope};

mod queries;
mod routes;
mod util;

pub use util::{base_is_logged_in, is_logged_in, make_auth_cookie, return_password_error};

pub(super) fn get_routes() -> Scope {
    scope("")
        // Login
        .service(routes::login::login)
        .service(routes::login::check_logged_in)
        // Logout
        .service(routes::logout::logout)
}
