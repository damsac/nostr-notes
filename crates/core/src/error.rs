#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
