use reqwest::header;
use serde_json::Value;

use crate::error::{AppError, Result};

use super::client::{OfficialClient, API_BASE};

impl OfficialClient {
    /// 发送 HTTP 请求并解包官方响应。
    pub(in crate::api) async fn http_envelope(&self, path: &str, payload: Value) -> Result<Value> {
        let response = self
            .http
            .post(format!("{API_BASE}{path}"))
            .header("x-id", &self.device_id)
            .header(header::ACCEPT, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(transport)?;
        decode_envelope(response).await
    }
}

/// 将底层传输错误转换为项目错误类型。
pub(in crate::api) fn transport<E: std::fmt::Display>(error: E) -> AppError {
    AppError::transport(error.to_string())
}

/// 校验并提取 HTTP 官方响应包中的业务数据。
pub(in crate::api) async fn decode_envelope(response: reqwest::Response) -> Result<Value> {
    let status = response.status().as_u16() as i64;
    let value: Value = response.json().await.map_err(|error| {
        if !(200..300).contains(&status) {
            AppError::Upstream {
                status,
                message: "HTTP 请求被拒绝，且未返回有效错误包".into(),
            }
        } else {
            AppError::protocol(format!("HTTP JSON 解码失败：{error}"))
        }
    })?;
    let success = value
        .get("Success")
        .and_then(Value::as_bool)
        .ok_or_else(|| AppError::protocol("HTTP 响应缺少布尔 Success 字段"))?;
    if !success {
        return Err(AppError::Upstream {
            status: value
                .get("Status")
                .and_then(Value::as_i64)
                .unwrap_or(status),
            message: value
                .get("Msg")
                .and_then(Value::as_str)
                .unwrap_or("请求失败")
                .into(),
        });
    }
    Ok(value.get("Response").cloned().unwrap_or(Value::Null))
}
