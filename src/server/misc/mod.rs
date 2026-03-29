use actix_web::{Scope, web::scope};

mod routes;

pub(super) fn get_routes() -> Scope {
    scope("").service(routes::keep_alive::keep_alive)
}
