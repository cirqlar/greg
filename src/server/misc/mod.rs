use actix_web::Scope;

mod routes;

pub(super) fn add_routes(scope: Scope) -> Scope {
    scope.service(routes::keep_alive::keep_alive)
}
