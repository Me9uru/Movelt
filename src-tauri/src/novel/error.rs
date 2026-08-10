use serde::Serialize;

#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[serde(tag = "code", content = "message", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NovelError {
    #[error("invalid novel configuration: {0}")]
    Configuration(String),
    #[error("{0}")]
    InvalidInput(String),
    #[error("unknown novel source: {0}")]
    UnknownSource(String),
    #[error("resource not found")]
    NotFound,
    #[error("upstream request failed: {0}")]
    Upstream(String),
    #[error("Wenku8 API is not logged in; fix its credentials and restart it")]
    NotLoggedIn,
    #[error("Wenku8 API rate limit was reached: {0}")]
    RateLimited(String),
    #[error("upstream response could not be parsed: {0}")]
    Parse(String),
    #[error("internal novel source error")]
    Internal,
}

impl NovelError {
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn unknown_source(source: impl Into<String>) -> Self {
        Self::UnknownSource(source.into())
    }
}
