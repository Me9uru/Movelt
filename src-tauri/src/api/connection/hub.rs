use std::{io::Read, time::Duration};

use futures_util::{SinkExt, StreamExt};
use rmpv::Value as MessagePackValue;
use serde_json::{json, Value};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;
use uuid::Uuid;

use crate::error::{AppError, Result};

use super::{
    client::{OfficialClient, API_BASE},
    http::transport,
};

const WEBSOCKET_TIMEOUT: Duration = Duration::from_secs(30);

type HubSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Default)]
pub(in crate::api) struct HubSession {
    socket: Option<HubSocket>,
}

impl OfficialClient {
    /// 在现有或新建的 SignalR 连接上执行一次调用。
    pub(in crate::api) async fn hub_once(&self, method: &str, payload: Value) -> Result<Value> {
        let needs_connection = self.hub_session.lock().await.socket.is_none();
        if needs_connection {
            let token = self.token().await?;
            let mut hub_session = self.hub_session.lock().await;
            if hub_session.socket.is_none() {
                let socket = self.connect_hub(&token).await?;
                hub_session.socket = Some(socket);
            }
        }

        let mut hub_session = self.hub_session.lock().await;
        let socket = hub_session
            .socket
            .as_mut()
            .ok_or_else(|| AppError::Internal {
                detail: "SignalR 连接未初始化".into(),
            })?;
        let result =
            match timeout(WEBSOCKET_TIMEOUT, Self::invoke_hub(socket, method, payload)).await {
                Ok(result) => result,
                Err(_) => Err(AppError::transport("SignalR 调用超时")),
            };
        if matches!(
            result,
            Err(AppError::Transport { .. } | AppError::UpstreamProtocol { .. })
        ) {
            *hub_session = HubSession::default();
        }
        result
    }

    /// 协商并建立使用 MessagePack 协议的 SignalR WebSocket 连接。
    async fn connect_hub(&self, token: &str) -> Result<HubSocket> {
        let negotiate_url = format!("{API_BASE}/hub/api/negotiate?negotiateVersion=1");
        let mut request = self
            .http
            .post(negotiate_url)
            .header("x-id", &self.device_id);
        if !token.is_empty() {
            request = request.bearer_auth(token);
        }
        let response = request.send().await.map_err(transport)?;
        let status = response.status().as_u16() as i64;
        if !response.status().is_success() {
            return Err(AppError::Upstream {
                status,
                message: "SignalR 协商请求失败".into(),
            });
        }
        let negotiation: Value = response.json().await.map_err(|error| {
            AppError::protocol(format!("SignalR negotiate JSON 解码失败：{error}"))
        })?;
        let connection_token = negotiation
            .get("connectionToken")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::protocol("SignalR negotiate 未返回 connectionToken"))?;
        let mut url =
            Url::parse(&format!("{API_BASE}/hub/api")).map_err(|e| AppError::Internal {
                detail: format!("构造 SignalR 地址失败：{e}"),
            })?;
        url.set_scheme("wss").map_err(|_| AppError::Internal {
            detail: "无法建立 WebSocket 地址".into(),
        })?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("id", connection_token);
            if !token.is_empty() {
                query.append_pair("access_token", token);
            }
        }
        let (mut socket, _) = timeout(WEBSOCKET_TIMEOUT, connect_async(url.as_str()))
            .await
            .map_err(|_| AppError::transport("SignalR WebSocket 连接超时"))?
            .map_err(websocket_error)?;
        timeout(
            WEBSOCKET_TIMEOUT,
            socket.send(Message::Text(
                "{\"protocol\":\"messagepack\",\"version\":1}\u{1e}".into(),
            )),
        )
        .await
        .map_err(|_| AppError::transport("发送 SignalR 握手请求超时"))?
        .map_err(transport)?;
        let handshake = timeout(WEBSOCKET_TIMEOUT, socket.next())
            .await
            .map_err(|_| AppError::transport("等待 SignalR 握手响应超时"))?
            .ok_or_else(|| AppError::transport("SignalR 连接意外关闭"))?
            .map_err(transport)?;
        let handshake_text = handshake
            .into_text()
            .map_err(|error| AppError::protocol(format!("SignalR 握手不是文本帧：{error}")))?;
        if !handshake_text.starts_with("{}") {
            return Err(AppError::protocol(format!(
                "SignalR 握手失败：{handshake_text}"
            )));
        }
        Ok(socket)
    }

    /// 丢弃当前 SignalR 连接，使下次调用重新连接。
    pub(in crate::api) async fn invalidate_hub(&self) {
        *self.hub_session.lock().await = HubSession::default();
    }

    /// 发送 SignalR 调用帧并读取对应的完成消息。
    async fn invoke_hub(socket: &mut HubSocket, method: &str, payload: Value) -> Result<Value> {
        let invocation_id = Uuid::new_v4().to_string();
        let frame = rmp_serde::to_vec(&(
            1_u8,
            Value::Object(Default::default()),
            invocation_id.clone(),
            method,
            vec![payload, json!({ "UseGzip": true })],
            Vec::<String>::new(),
        ))
        .map_err(|e| AppError::Internal {
            detail: format!("编码 SignalR 调用失败：{e}"),
        })?;
        timeout(
            WEBSOCKET_TIMEOUT,
            socket.send(Message::Binary(with_length_prefix(frame).into())),
        )
        .await
        .map_err(|_| AppError::transport("发送 SignalR 调用请求超时"))?
        .map_err(transport)?;
        while let Some(message) = timeout(WEBSOCKET_TIMEOUT, socket.next())
            .await
            .map_err(|_| AppError::transport("等待 SignalR 调用响应超时"))?
        {
            let message = message.map_err(transport)?;
            if let Message::Binary(bytes) = message {
                for frame in split_binary_frames(&bytes)? {
                    let data: Vec<MessagePackValue> =
                        rmp_serde::from_slice(frame).map_err(|e| {
                            AppError::protocol(format!("SignalR MessagePack 解码失败：{e}"))
                        })?;
                    if data.first().and_then(MessagePackValue::as_i64) != Some(3)
                        || data.get(2).and_then(MessagePackValue::as_str) != Some(&invocation_id)
                    {
                        continue;
                    }
                    if let Some(error) = data.get(3).and_then(MessagePackValue::as_str) {
                        return Err(AppError::Upstream {
                            status: 500,
                            message: error.into(),
                        });
                    }
                    return decode_hub_envelope(
                        data.get(4)
                            .ok_or_else(|| AppError::protocol("SignalR 完成消息缺少结果"))?,
                    );
                }
            }
        }
        Err(AppError::transport("SignalR 连接在返回结果前关闭"))
    }
}

/// 将 WebSocket 错误映射为传输或上游错误。
fn websocket_error(error: tokio_tungstenite::tungstenite::Error) -> AppError {
    match error {
        tokio_tungstenite::tungstenite::Error::Http(response) => AppError::Upstream {
            status: response.status().as_u16() as i64,
            message: "SignalR WebSocket 连接被拒绝".into(),
        },
        error => transport(error),
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
    if let Some(bytes) = response.as_slice() {
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

/// 为 SignalR 二进制帧添加变长长度前缀。
fn with_length_prefix(frame: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::new();
    let mut length = frame.len();
    while length >= 0x80 {
        out.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    out.push(length as u8);
    out.extend(frame);
    out
}

/// 按长度前缀拆分 SignalR 二进制消息中的帧。
fn split_binary_frames(data: &[u8]) -> Result<Vec<&[u8]>> {
    let mut frames = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let mut length = 0usize;
        let mut shift = 0;
        loop {
            let byte = *data
                .get(offset)
                .ok_or_else(|| AppError::protocol("不完整的 SignalR 帧"))?;
            offset += 1;
            length |= ((byte & 0x7f) as usize) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 28 {
                return Err(AppError::protocol("非法 SignalR 帧长度"));
            }
        }
        let end = offset
            .checked_add(length)
            .filter(|end| *end <= data.len())
            .ok_or_else(|| AppError::protocol("不完整的 SignalR 数据"))?;
        frames.push(&data[offset..end]);
        offset = end;
    }
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_frames_round_trip() {
        let data = with_length_prefix(vec![1, 2, 3]);
        let frames = split_binary_frames(&data).unwrap();
        assert_eq!(frames, vec![&[1, 2, 3]]);
    }
}
