use std::io::Read;

use rmpv::Value as MessagePackValue;
use serde_json::{json, Value};
use signalr_client::SignalRClient;

use crate::error::{AppError, Result};

use super::client::{OfficialClient, API_BASE};

const HUB_DOMAIN: &str = "api.lightnovel.life";
const HUB_PATH: &str = "hub/api";

#[derive(Default)]
pub(in crate::api) struct HubSession {
    client: Option<SignalRClient>,
}

impl OfficialClient {
    /// 在现有或新建的 SignalR 连接上执行一次调用。
    pub(in crate::api) async fn hub_once(&self, method: &str, payload: Value) -> Result<Value> {
        let mut hub_session = self.hub_session.lock().await;
        if hub_session.client.is_none() {
            let token = self.token().await?;
            hub_session.client = Some(self.connect_hub(token).await?);
        }

        let client = hub_session
            .client
            .as_mut()
            .ok_or_else(|| AppError::Internal {
                detail: "SignalR 连接未初始化".into(),
            })?;
        let envelope = client
            .invoke_with_args::<MessagePackValue, _>(method.to_owned(), |arguments| {
                arguments.argument(payload.clone());
                arguments.argument(json!({ "UseGzip": true }));
            })
            .await
            .map_err(AppError::transport)?;
        decode_hub_envelope(&envelope)
    }

    /// 建立使用 MessagePack 协议的官方 SignalR Hub 连接。
    async fn connect_hub(&self, token: String) -> Result<SignalRClient> {
        SignalRClient::connect_with(HUB_DOMAIN, HUB_PATH, |configuration| {
            configuration.with_messagepack_protocol();
            if !token.is_empty() {
                configuration.authenticate_bearer(token.clone());
            }
        })
        .await
        .map_err(|error| AppError::transport(format!("连接 {API_BASE}/hub/api 失败：{error}")))
    }

    /// 丢弃当前 SignalR 连接，使下次调用重新连接。
    pub(in crate::api) async fn invalidate_hub(&self) {
        *self.hub_session.lock().await = HubSession::default();
    }
}

/// 校验并解码 SignalR MessagePack 响应包。
fn decode_hub_envelope(value: &MessagePackValue) -> Result<Value> {
    let success = hub_field(value, "Success")
        .and_then(MessagePackValue::as_bool)
        .ok_or_else(|| AppError::protocol("SignalR 响应缺少布尔 Success 字段"))?;
    if !success {
        return Err(AppError::Upstream {
            status: hub_field(value, "Status")
                .and_then(MessagePackValue::as_i64)
                .unwrap_or(500),
            message: hub_field(value, "Msg")
                .and_then(MessagePackValue::as_str)
                .unwrap_or("请求失败")
                .into(),
        });
    }
    let response = hub_field(value, "Response")
        .cloned()
        .unwrap_or(MessagePackValue::Nil);
    if response.is_nil() {
        return Ok(Value::Null);
    }
    if let Some(bytes) = response.as_slice() {
        if bytes.is_empty() {
            return Ok(Value::Null);
        }
        let mut decoder = flate2::read::GzDecoder::new(bytes);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .map_err(|error| AppError::protocol(format!("gzip 解压失败：{error}")))?;
        return serde_json::from_str(&json)
            .map_err(|error| AppError::protocol(format!("gzip JSON 解析失败：{error}")));
    }
    rmpv::ext::from_value(response)
        .map_err(|error| AppError::protocol(format!("SignalR 响应解码失败：{error}")))
}

/// 从 SignalR 响应映射中获取指定字段。
fn hub_field<'a>(value: &'a MessagePackValue, name: &str) -> Option<&'a MessagePackValue> {
    value
        .as_map()?
        .iter()
        .find_map(|(key, value)| (key.as_str() == Some(name)).then_some(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_successful_empty_response_as_null() {
        let envelope = MessagePackValue::Map(vec![
            (
                MessagePackValue::from("Success"),
                MessagePackValue::Boolean(true),
            ),
            (MessagePackValue::from("Response"), MessagePackValue::Nil),
        ]);

        assert_eq!(decode_hub_envelope(&envelope).unwrap(), Value::Null);
    }

    #[test]
    fn decodes_successful_empty_binary_response_as_null() {
        let envelope = MessagePackValue::Map(vec![
            (
                MessagePackValue::from("Success"),
                MessagePackValue::Boolean(true),
            ),
            (
                MessagePackValue::from("Response"),
                MessagePackValue::Binary(Vec::new()),
            ),
        ]);

        assert_eq!(decode_hub_envelope(&envelope).unwrap(), Value::Null);
    }
}
