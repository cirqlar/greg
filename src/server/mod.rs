use actix_web::{
    App, HttpServer, Scope,
    dev::HttpServiceFactory,
    middleware::Logger,
    web::{self, scope},
};
use actix_web_httpauth::extractors::basic;
use actix_web_lab::web::spa;
use libsql::Database;

#[cfg(feature = "scheduler")]
use tokio_cron_scheduler::{JobScheduler, JobSchedulerError};

mod db;
mod roadmap;
mod rss;
mod shared;

pub struct AppState {
    pub app_db: Database,
    pub demo_db: Database,
}

pub type AppData = web::Data<AppState>;

fn get_spa_service() -> impl HttpServiceFactory {
    spa()
        .index_file("./dist/index.html")
        .static_resources_mount("/")
        .static_resources_location("./dist")
        .finish()
}

fn get_api_service() -> Scope {
    let mut api_scope = scope("/api");
    api_scope = rss::add_routes(api_scope);
    api_scope = roadmap::add_routes(api_scope);

    api_scope
}

#[cfg(feature = "scheduler")]
pub async fn start_scheduler(app_data: AppData) -> Result<(), JobSchedulerError> {
    let scheduler = JobScheduler::new().await?;

    rss::add_tasks(&scheduler, app_data.clone()).await?;
    roadmap::add_tasks(&scheduler, app_data.clone()).await?;

    Ok(())
}

pub async fn start_server(app_data: AppData) -> anyhow::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_data.clone())
            .app_data(basic::Config::default().realm("Restricted"))
            .service(get_api_service())
            .service(get_spa_service())
    })
    .bind(("0.0.0.0", 10000))?
    .run()
    .await?;

    Ok(())
}
