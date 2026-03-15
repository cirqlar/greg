use actix_web::{Scope, web};
use log::{error, info};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

use crate::server::AppData;

mod queries;
mod routes;
mod tasks;
mod types;
mod utils;

pub(super) fn add_routes(scope: Scope) -> Scope {
    scope
        // Tabs
        .service(routes::tabs::get_most_recent_tabs)
        .service(routes::tabs::get_watched_tabs)
        .service(routes::tabs::add_watched_tab)
        .service(routes::tabs::delete_watched_tab)
        // Activity
        .service(routes::activity::get_roadmap_activity)
        // Changes
        .service(routes::changes::get_changes)
        // Recheck
        .service(routes::recheck::recheck_roadmap)
}

pub(super) async fn add_tasks(
    scheduler: &JobScheduler,
    app_data: AppData,
) -> Result<(), JobSchedulerError> {
    scheduler
        .add(Job::new_async("every 3 hours", move |_uuid, _l| {
            let sched_data = web::Data::clone(&app_data);
            Box::pin(async move {
                let our_data = web::Data::clone(&sched_data);

                if let Err(err) = tasks::check::check_roadmap(&our_data).await {
                    error!("check_roadmap task errored. err: {err:?}");
                };
            })
        })?)
        .await?;

    info!("Added check_roadmap task");

    Ok(())
}
