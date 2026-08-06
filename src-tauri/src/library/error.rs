use serde::Serialize;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "code", content = "message", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LibraryError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("local library database error: {0}")]
    Database(String),
}
