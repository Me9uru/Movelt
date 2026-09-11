use std::{future::Future, io::Read, time::Duration};

use rmpv::Value as MessagePackValue;
use serde_json::{json, Value};
use signalr_client::SignalRClient;

use crate::error::{AppError, Result};

use super::client::{OfficialClient, API_BASE};

const HUB_DOMAIN: &str = "api.lightnovel.life";
const HUB_PATH: &str = "hub/api";
const HUB_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Default)]
pub(in crate::api) struct HubSession {
    client: Option<SignalRClient>,
    revision: u64,
}

impl Drop for HubSession {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            tauri::async_runtime::spawn_blocking(move || drop(client));
        }
    }
}

impl OfficialClient {
    /// 在现有或新建的 SignalR 连接上执行一次调用。
    pub(in crate::api) async fn hub_once(&self, method: &str, payload: Value) -> Result<Value> {
        #[cfg(test)]
        if let Some(hub) = &self.test_hub {
            self.ensure_current().await?;
            let result = with_timeout(hub(method.to_owned(), payload)).await;
            self.ensure_current().await?;
            return result;
        }
        let (mut client, revision) = with_timeout(self.connected_client()).await?;
        self.ensure_current().await?;
        let result = with_timeout(async {
            client
                .invoke_with_args::<MessagePackValue, _>(method.to_owned(), |arguments| {
                    arguments.argument(payload.clone());
                    arguments.argument(json!({ "UseGzip": true }));
                })
                .await
                .map_err(AppError::transport)
        })
        .await;
        let current = self.ensure_current().await;
        if result.as_ref().is_err_and(|error| error.is_transport()) {
            self.invalidate_hub_revision(Some(revision)).await;
        }
        // SignalR 的 Drop 内部会同步等待断开，放到阻塞线程以免占住运行时。
        tauri::async_runtime::spawn_blocking(move || drop(client));
        current?;
        decode_hub_envelope(&result?)
    }

    async fn connected_client(&self) -> Result<(SignalRClient, u64)> {
        self.ensure_current().await?;
        if let Some(client) = self.cached_hub().await {
            return Ok(client);
        }
        let _connecting = self.context.connecting.lock().await;
        if let Some(client) = self.cached_hub().await {
            return Ok(client);
        }
        // 刷新和连接都不持有 Hub 锁；认证失败可以安全地重置会话。
        let token = self.token().await?;
        let client = self.connect_hub(token).await?;
        if let Err(error) = self.ensure_current().await {
            tauri::async_runtime::spawn_blocking(move || drop(client));
            return Err(error);
        }
        let mut hub = self.context.hub.lock().await;
        hub.revision += 1;
        let result = (client.clone(), hub.revision);
        hub.client = Some(client);
        Ok(result)
    }

    async fn cached_hub(&self) -> Option<(SignalRClient, u64)> {
        let hub = self.context.hub.lock().await;
        hub.client
            .as_ref()
            .map(|client| (client.clone(), hub.revision))
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
        self.invalidate_hub_revision(None).await;
    }

    async fn invalidate_hub_revision(&self, revision: Option<u64>) {
        let removed = {
            let mut hub = self.context.hub.lock().await;
            if revision.is_some_and(|revision| revision != hub.revision) {
                return;
            }
            hub.client.take()
        };
        tauri::async_runtime::spawn_blocking(move || drop(removed));
    }
}

async fn with_timeout<T>(future: impl Future<Output = Result<T>>) -> Result<T> {
    tokio::time::timeout(HUB_TIMEOUT, future)
        .await
        .map_err(|_| AppError::transport("SignalR 请求超时"))?
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
    fn decodes_gzip_payload_and_rejects_corruption() {
        use std::io::Write;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(br#"{"Id":42}"#).unwrap();
        let envelope = |bytes| {
            MessagePackValue::Map(vec![
                ("Success".into(), true.into()),
                ("Response".into(), MessagePackValue::Binary(bytes)),
            ])
        };
        assert_eq!(
            decode_hub_envelope(&envelope(encoder.finish().unwrap())).unwrap(),
            json!({"Id": 42})
        );
        assert!(matches!(
            decode_hub_envelope(&envelope(vec![1, 2, 3])),
            Err(AppError::UpstreamProtocol { .. })
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn bounds_an_unresponsive_operation() {
        let started = tokio::time::Instant::now();
        let result = with_timeout(std::future::pending::<Result<()>>()).await;
        assert!(matches!(result, Err(AppError::Transport { .. })));
        assert_eq!(started.elapsed(), HUB_TIMEOUT);
        assert!(with_timeout(async { Ok(()) }).await.is_ok());
    }

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
