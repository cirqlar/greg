use std::pin::Pin;
use std::sync::Arc;

use libsql::Transaction;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::server::shared::DatabaseError;
use crate::server::shared::timestamp::deserialize_timestamp;

pub(super) type MigrationFunction = Box<
    dyn Fn(
        Arc<Transaction>,
    ) -> Pin<Box<dyn Future<Output = Result<(), DatabaseError>> + Send + 'static>>,
>;

pub(super) struct Migration {
    pub(super) name: &'static str,
    pub(super) run: MigrationFunction,
}

#[derive(Debug, Deserialize)]
pub(super) struct DbMigration {
    pub(super) id: u32,
    pub(super) name: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub(super) timestamp: OffsetDateTime,
}
