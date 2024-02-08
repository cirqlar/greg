use actix_web::{
    middleware::Logger,
    web::{self, scope},
    App, HttpServer,
};
use actix_web_lab::web::spa;
use dotenvy::dotenv;
use greg::{
    db,
    routes::{
        gets::{get_activity, get_sources},
        posts::{add_source, login, recheck},
    },
    tasks::check_sources::check_sources,
    types::AppState,
};
use log::info;
use tokio::{runtime::Handle, sync::Mutex};
use tokio_cron_scheduler::{Job, JobScheduler};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Connecting to Database");
    let client = db::establish_connection().await;
    info!("Connected to Database");
    db::migrate_db(&client).await?;
    info!("Migrated Database");

    let app_state = web::Data::new(AppState {
        db_handle: Mutex::new(client),
    });
    let data = app_state.clone();

    let scheduler = JobScheduler::new().await?;

    let rt = Handle::current();
    let ourdata = data.clone();
    scheduler
        .add(Job::new("0 0 * * * *", move |_uuid, _l| {
            check_sources(rt.clone(), &ourdata);
        })?)
        .await?;
    info!("Initialized Scheduler");
    scheduler.start().await?;
    info!("Scheduler Started");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .service(
                scope("/api")
                    .service(get_sources)
                    .service(get_activity)
                    .service(add_source)
                    .service(recheck)
                    .service(login),
            )
            .service(
                spa()
                    .index_file("./dist/index.html")
                    .static_resources_mount("/")
                    .static_resources_location("./dist")
                    .finish(),
            )
    })
    .bind(("127.0.0.1", 10000))?
    .run()
    .await?;

    Ok(())
}
