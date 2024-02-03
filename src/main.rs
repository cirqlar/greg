use std::time::Duration;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use clokwerk::{AsyncScheduler, TimeUnits};

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
async fn get_sources() -> impl Responder {
    HttpResponse::Ok().body("Sources")
}

#[get("/activity")]
async fn get_activity() -> impl Responder {
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

async fn check_sources() {
    println!("Checking Sources");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    let mut scheduler = AsyncScheduler::new();

    scheduler.every(1.minute()).run(check_sources);
    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(login)
            .service(get_sources)
            .service(get_activity)
            .service(add_source)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
