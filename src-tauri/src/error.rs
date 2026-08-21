use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("网络连接失败，请检查网络后重试")]
    Transport { detail: String },
    #[error("服务返回错误（{status}）：{message}")]
    Upstream { status: i64, message: String },
    #[error("服务返回的数据格式异常，请稍后重试")]
    UpstreamProtocol { detail: String },
    #[error("请求参数无效：{message}")]
    InvalidInput { message: String },
    #[error("认证信息不可用：{0}")]
    Credentials(String),
    #[error("登录已失效，请重新登录")]
    AuthenticationExpired,
    #[error("应用内部错误，请稍后重试")]
    Internal { detail: String },
}

impl AppError {
    pub(crate) fn transport(detail: impl Into<String>) -> Self {
        Self::Transport {
            detail: detail.into(),
        }
    }

    pub(crate) fn protocol(detail: impl Into<String>) -> Self {
        Self::UpstreamProtocol {
            detail: detail.into(),
        }
    }

    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    pub(crate) fn is_transport(&self) -> bool {
        matches!(self, Self::Transport { .. })
    }
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
            Self::Transport { .. } => "NETWORK_ERROR",
            Self::Upstream { .. } => "UPSTREAM_ERROR",
            Self::UpstreamProtocol { .. } => "UPSTREAM_PROTOCOL_ERROR",
            Self::InvalidInput { .. } => "INVALID_INPUT",
            Self::Credentials(_) => "CREDENTIALS_ERROR",
            Self::AuthenticationExpired => "AUTHENTICATION_EXPIRED",
            Self::Internal { .. } => "INTERNAL_ERROR",
        };
        ErrorResponse {
            code,
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub(crate) type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::AppError;

    #[test]
    fn serializes_transport_as_a_retryable_network_category() {
        let value = serde_json::to_value(AppError::transport("DNS lookup failed"))
            .expect("error should serialize");

        assert_eq!(value["code"], "NETWORK_ERROR");
        assert_eq!(value["message"], "网络连接失败，请检查网络后重试");
        assert_ne!(
            value,
            json!({ "code": "NETWORK_ERROR", "message": "DNS lookup failed" })
        );
    }

    #[test]
    fn keeps_input_and_upstream_protocol_failures_distinct() {
        let input = serde_json::to_value(AppError::invalid_input("作品 ID 必须是数字"))
            .expect("error should serialize");
        let protocol = serde_json::to_value(AppError::protocol("Response missing Book"))
            .expect("error should serialize");

        assert_eq!(input["code"], "INVALID_INPUT");
        assert_eq!(protocol["code"], "UPSTREAM_PROTOCOL_ERROR");
    }
}
