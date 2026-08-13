use serde::Serialize;

/// 应用向前端暴露的统一错误契约。
///
/// 各个领域模块可以构造此错误，但不能相互依赖其实现位置。
#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[serde(tag = "code", content = "message", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum NovelError {
    #[error("invalid novel configuration: {0}")]
    Configuration(String),
    #[error("{0}")]
    InvalidInput(String),
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
    #[error("EPUB I/O error: {0}")]
    EpubIo(String),
    #[error("internal novel source error")]
    Internal,
}

impl NovelError {
    pub(crate) fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub(crate) fn epub_io(error: std::io::Error) -> Self {
        Self::EpubIo(error.to_string())
    }
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "code", content = "message", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum LibraryError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("local library database error: {0}")]
    Database(String),
}
