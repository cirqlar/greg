use dotenvy::dotenv;

use greg::server::{AppDataError, get_app_data, start_server};

#[cfg(feature = "scheduler")]
use greg::server::start_scheduler;
use thiserror::Error;

#[derive(Debug, Error)]
enum AppError {
    #[error(transparent)]
    Data(#[from] AppDataError),

    #[cfg(feature = "scheduler")]
    #[error("Error setting up schedules")]
    Scheduler(#[from] tokio_cron_scheduler::JobSchedulerError),

    #[error("Error starting server")]
    Server(#[from] actix_web::Error),
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app_data = get_app_data().await?;

    #[cfg(feature = "scheduler")]
    {
        start_scheduler(app_data.clone()).await?;
    }

    start_server(app_data).await?;

    Ok(())
}
