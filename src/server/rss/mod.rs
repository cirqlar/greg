use actix_web::{Scope, web};
use log::{error, info};

#[cfg(feature = "scheduler")]
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

use crate::server::AppData;

mod queries;
mod routes;
mod tasks;
mod types;

use types::{Activity, Source};

pub(super) fn add_routes(scope: Scope) -> Scope {
    scope
        // Sources
        .service(routes::sources::get_sources)
        .service(routes::sources::add_source)
        .service(routes::sources::enable_source)
        .service(routes::sources::delete_source)
        // Activity
        .service(routes::activity::get_activity)
        .service(routes::activity::get_source_activity)
        .service(routes::activity::clear_all_activities)
        .service(routes::activity::clear_activities)
        // Recheck
        .service(routes::recheck::recheck)
}

#[cfg(feature = "scheduler")]
pub(super) async fn add_tasks(
    scheduler: &JobScheduler,
    app_data: AppData,
) -> Result<(), JobSchedulerError> {
    scheduler
        .add(Job::new_async("every 3 hours", move |_uuid, _l| {
            let sched_data = web::Data::clone(&app_data);
            Box::pin(async move {
                let our_data = web::Data::clone(&sched_data);

                if let Err(err) = tasks::check::check_rss(&our_data).await {
                    error!("Schedule errored. err: {err:?}");
                };
            })
        })?)
        .await?;

    info!("Added check_rss task");

    Ok(())
}
