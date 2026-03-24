use std::{pin::Pin, sync::Arc};

use libsql::Transaction;

use crate::server::shared::DatabaseError;

// migration_import_start
// migration_import_end

type MigrationFunction = Box<
    dyn Fn(
        Arc<Transaction>,
    ) -> Pin<Box<dyn Future<Output = Result<(), DatabaseError>> + Send + 'static>>,
>;

struct Migration {
    name: &'static str,
    run: MigrationFunction,
}

fn get_migrations() -> Vec<Migration> {
    vec![
        // migration_list_start
        // migration_list_end
    ]
}
