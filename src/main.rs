use std::time::Duration;

use actix_web::{
    web::{self, scope},
    App, HttpServer,
};
use actix_web_lab::web::spa;
use clokwerk::{AsyncScheduler, TimeUnits};
use dotenvy::dotenv;
use greg::{
    db::{self},
    routes::{
        gets::{get_activity, get_sources},
        posts::{add_source, recheck},
    },
    tasks::check_sources::check_sources,
    types::AppState,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let client = db::establish_connection().await;
    db::migrate_db(&client).await?;

    let app_state = web::Data::new(AppState {
        db_handle: Mutex::new(client),
    });

    let mut scheduler = AsyncScheduler::new();
    scheduler.every(1.hour()).run(check_sources);
    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(
                scope("/api")
                    .service(get_sources)
                    .service(get_activity)
                    .service(add_source)
                    .service(recheck),
            )
            .service(
                spa()
                    .index_file("./dist/index.html")
                    .static_resources_mount("/")
                    .static_resources_location("./dist")
                    .finish(),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
