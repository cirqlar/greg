use std::thread;

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
        deletes::{clear_activities, clear_all_activities, delete_source},
        gets::{check_logged_in, get_activity, get_sources, keep_alive},
        posts::{add_source, login, recheck, trigger_check},
    },
    types::{AppState, DbMesssage},
};
use log::info;
use tokio::sync::mpsc::{self};

#[cfg(feature = "scheduler")]
use greg::tasks::check_sources::check_sources;
#[cfg(feature = "scheduler")]
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

    let (send, mut recv) = mpsc::channel::<DbMesssage>(100);

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let db_client = db::establish_connection().await;
            while let Some((stmnt, send)) = recv.recv().await {
                let res = db_client.execute(stmnt).await;
                let _ = send.send(res).await;
            }
        });
    });

    let app_data = web::Data::new(AppState {
        db_channel: send.clone(),
    });

    #[cfg(feature = "scheduler")]
    {
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
        info!("Initialized Scheduler");
        scheduler.start().await?;
        info!("Scheduler Started");
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
                    .service(trigger_check)
                    .service(keep_alive),
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
