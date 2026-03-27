use actix_web::{
    App, HttpServer, Scope,
    dev::HttpServiceFactory,
    middleware::Logger,
    web::{self, ServiceConfig, scope},
};
use actix_web_httpauth::extractors::basic;
use actix_web_lab::web::spa;
use dotenvy::dotenv;
use libsql::Database;
use thiserror::Error;

#[cfg(feature = "scheduler")]
use tokio_cron_scheduler::{JobScheduler, JobSchedulerError};

pub(crate) mod auth;
pub(crate) mod db;
pub(crate) mod mail;
pub(crate) mod misc;
pub(crate) mod roadmap;
pub(crate) mod rss;
pub(crate) mod shared;

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
    api_scope = auth::add_routes(api_scope);
    api_scope = misc::add_routes(api_scope);

    api_scope
}

#[derive(Debug, Error)]
pub enum AppDataError {
    #[error(transparent)]
    Get(#[from] db::GetDatabaseError),
    #[error(transparent)]
    Connect(#[from] shared::DatabaseError),
    #[error(transparent)]
    Migration(#[from] db::ApplyMigrationError),
}

pub async fn get_app_data() -> Result<AppData, AppDataError> {
    let app_db = db::get_database().await?;
    db::apply_migrations(app_db.connect().map_err(shared::DatabaseError::from)?).await?;

    let demo_db = db::get_demo_database().await?;
    db::apply_migrations(demo_db.connect().map_err(shared::DatabaseError::from)?).await?;

    Ok(web::Data::new(AppState { app_db, demo_db }))
}

#[cfg(feature = "scheduler")]
pub async fn start_scheduler(app_data: AppData) -> Result<(), JobSchedulerError> {
    let scheduler = JobScheduler::new().await?;

    rss::add_tasks(&scheduler, app_data.clone()).await?;
    roadmap::add_tasks(&scheduler, app_data.clone()).await?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Data(#[from] AppDataError),

    #[cfg(feature = "scheduler")]
    #[error("Error setting up schedules")]
    Scheduler(#[from] tokio_cron_scheduler::JobSchedulerError),

    #[error("Error starting server")]
    Server(#[from] std::io::Error),
}

pub fn config_app(app_data: AppData) -> Box<dyn Fn(&mut ServiceConfig)> {
    Box::new(move |cfg| {
        cfg.app_data(app_data.clone())
            .app_data(basic::Config::default().realm("Restricted"))
            .service(get_api_service())
            .service(get_spa_service());
    })
}

pub async fn run_app() -> Result<(), AppError> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app_data = get_app_data().await?;

    #[cfg(feature = "scheduler")]
    {
        start_scheduler(app_data.clone()).await?;
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(config_app(app_data.clone()))
    })
    .bind(("0.0.0.0", 10000))?
    .run()
    .await?;

    Ok(())
}
