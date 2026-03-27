use actix_web::Scope;

mod queries;
mod routes;
mod util;

pub use util::{base_is_logged_in, is_logged_in, make_auth_cookie, return_password_error};

pub(super) fn add_routes(scope: Scope) -> Scope {
    scope
        // Login
        .service(routes::login::login)
        .service(routes::login::check_logged_in)
        // Logout
        .service(routes::logout::logout)
}
