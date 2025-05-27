use actix_web::{
    App, HttpServer,
    middleware::Logger,
    web::{self, scope},
};
use actix_web_httpauth::extractors::basic;
use actix_web_lab::web::spa;
use dotenvy::dotenv;
use greg::{
    db,
    routes::{
        deletes::{clear_activities, clear_all_activities, delete_source, delete_watched_tab},
        gets::{
            check_logged_in, get_activity, get_changes, get_most_recent_tabs, get_roadmap_activity,
            get_sources, get_watched_tabs, keep_alive,
        },
        posts::{add_source, add_watched_tab, login, recheck, recheck_roadmap},
    },
    types::AppState,
};
use log::info;

#[cfg(feature = "scheduler")]
use greg::tasks::check_roadmap::check_roadmap;
#[cfg(feature = "scheduler")]
use greg::tasks::check_sources::check_sources;
#[cfg(feature = "scheduler")]
use tokio_cron_scheduler::{Job, JobScheduler};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let db = db::get_database().await;
    info!("Connecting to Database");
    let conn = db.connect().unwrap().clone();
    info!("Connected to Database. Migrating");
    db::migrate_db(conn).await?;
    info!("Migrated Database");

    let app_data = web::Data::new(AppState { db });

    // check_roadmap(&app_data).await;
    // return Ok(());

    #[cfg(feature = "scheduler")]
    {
        // Sources
        let tmp_data = app_data.clone();

        let scheduler = JobScheduler::new().await?;
        scheduler
            .add(Job::new_async("0 0 */3 * * *", move |_uuid, _l| {
                let sched_data = web::Data::clone(&tmp_data);
                Box::pin(async move {
                    let our_data = web::Data::clone(&sched_data);
                    check_sources(&our_data).await;
                })
            })?)
            .await?;
        info!("Initialized Sources Scheduler");
        scheduler.start().await?;
        info!("Sources Scheduler Started");

        // Roadmap
        let tmp_data = app_data.clone();

        let scheduler = JobScheduler::new().await?;
        scheduler
            .add(Job::new_async("0 0 4 * * *", move |_uuid, _l| {
                let sched_data = web::Data::clone(&tmp_data);
                Box::pin(async move {
                    let our_data = web::Data::clone(&sched_data);
                    check_roadmap(&our_data).await;
                })
            })?)
            .await?;
        info!("Initialized Roadmap Scheduler");
        scheduler.start().await?;
        info!("Roadmap Scheduler Started");
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_data.clone())
            .app_data(basic::Config::default().realm("Restricted"))
            .service(
                scope("/api")
                    .service(get_sources)
                    .service(get_activity)
                    .service(add_source)
                    .service(recheck)
                    .service(login)
                    .service(check_logged_in)
                    .service(delete_source)
                    .service(clear_all_activities)
                    .service(clear_activities)
                    .service(keep_alive)
                    .service(get_roadmap_activity)
                    .service(get_most_recent_tabs)
                    .service(get_watched_tabs)
                    .service(recheck_roadmap)
                    .service(add_watched_tab)
                    .service(delete_watched_tab)
                    .service(get_changes),
            )
            .service(
                spa()
                    .index_file("./dist/index.html")
                    .static_resources_mount("/")
                    .static_resources_location("./dist")
                    .finish(),
            )
    })
    .bind(("0.0.0.0", 10000))?
    .run()
    .await?;

    Ok(())
}
