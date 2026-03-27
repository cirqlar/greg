use std::env::{VarError, var};

use libsql::{Builder, Database, OpenFlags};
use thiserror::Error;

use crate::shared::DatabaseError;

#[derive(Debug, Error)]
pub enum GetDatabaseError {
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error("Error retreiving environment variable")]
    Var(#[from] VarError),
}

async fn get_local_database() -> Result<Database, GetDatabaseError> {
    let db = Builder::new_local(var("LOCAL_DB_URL")?)
        .flags(OpenFlags::default())
        .build()
        .await
        .map_err(DatabaseError::from)?;

    Ok(db)
}

async fn get_remote_database() -> Result<Database, GetDatabaseError> {
    let database_url = var("DATABASE_URL")?;
    let auth_key = var("DATABASE_AUTH_KEY")?;

    let db = Builder::new_remote(database_url, auth_key)
        .build()
        .await
        .map_err(DatabaseError::from)?;

    Ok(db)
}

pub async fn get_database() -> Result<Database, GetDatabaseError> {
    if var("USE_LOCAL").unwrap_or("false".into()) == "false" {
        get_remote_database().await
    } else {
        get_local_database().await
    }
}

pub async fn get_demo_database() -> Result<Database, GetDatabaseError> {
    let db = Builder::new_local("db/demo.db")
        .flags(OpenFlags::default())
        .build()
        .await
        .map_err(DatabaseError::from)?;

    Ok(db)
}
