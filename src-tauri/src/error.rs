use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("网络请求失败：{0}")]
    Network(String),
    #[error("服务返回错误（{status}）：{message}")]
    Upstream { status: i64, message: String },
    #[error("数据格式错误：{0}")]
    InvalidResponse(String),
    #[error("认证信息不可用：{0}")]
    Credentials(String),
    #[error("登录已失效，请重新登录")]
    AuthenticationExpired,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct ErrorResponse<'a> {
            code: &'a str,
            message: String,
        }

        let code = match self {
            Self::Network(_) => "NETWORK_ERROR",
            Self::Upstream { .. } => "UPSTREAM_ERROR",
            Self::InvalidResponse(_) => "INVALID_RESPONSE",
            Self::Credentials(_) => "CREDENTIALS_ERROR",
            Self::AuthenticationExpired => "AUTHENTICATION_EXPIRED",
        };
        ErrorResponse {
            code,
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub(crate) type Result<T> = std::result::Result<T, AppError>;
