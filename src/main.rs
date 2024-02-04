use std::time::Duration;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use clokwerk::{AsyncScheduler, TimeUnits};
use dotenvy::dotenv;
use greg::db;
use libsql_client::Client;
use tokio::sync::Mutex;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Index")
}

#[derive(serde::Deserialize)]
struct LoginInfo {
    password: String,
}

#[post("/login")]
async fn login(info: web::Json<LoginInfo>) -> impl Responder {
    format!("Your password is {}", info.password)
}

#[get("/sources")]
async fn get_sources(data: web::Data<AppState>) -> impl Responder {
    let db_handle = data.db_handle.lock().await;
    let _result = db_handle.execute("SELECT * FROM sources").await.unwrap();
    HttpResponse::Ok().body("Sources")
}

#[get("/activity")]
async fn get_activity(data: web::Data<AppState>) -> impl Responder {
    let db_handle = data.db_handle.lock().await;
    let _result = db_handle.execute("SELECT * FROM sources").await.unwrap();
    HttpResponse::Ok().body("Activity")
}

#[derive(serde::Deserialize)]
struct Source {
    url: String,
}

#[post("/source/new")]
async fn add_source(source: web::Json<Source>) -> impl Responder {
    format!("New source is {}", source.url)
}

#[post("/recheck")]
async fn recheck() -> impl Responder {
    check_sources().await;
    HttpResponse::Ok().body("Recheck")
}

async fn check_sources_task() {
    check_sources().await
}

async fn check_sources() {
    println!("Checking Sources");
}

struct AppState {
    db_handle: Mutex<Client>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    println!("Hello, world!");
    let client = db::establish_connection().await;
    db::migrate_db(&client).await?;

    let app_state = web::Data::new(AppState {
        db_handle: Mutex::new(client),
    });

    let mut scheduler = AsyncScheduler::new();

    scheduler.every(1.hour()).run(check_sources_task);
    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(login)
            .service(get_sources)
            .service(get_activity)
            .service(add_source)
            .service(recheck)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
