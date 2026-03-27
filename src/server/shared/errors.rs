use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database request failed")]
    ClientError(#[from] libsql::Error),

    #[error("Parsing database return failed")]
    ParseError(#[from] serde::de::value::Error),

    #[error("Parsing database return failed")]
    JsonParse(#[from] serde_json::Error),
}
