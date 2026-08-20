use std::{
    io::Read,
    sync::Arc,
    time::{Duration, Instant},
};

use futures_util::{SinkExt, StreamExt};
use keyring::Entry;
use reqwest::{header, Client};
use rmpv::Value as MessagePackValue;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;
use uuid::Uuid;

use crate::error::{AppError, Result};

const API_BASE: &str = "https://api.lightnovel.life";
const CREDENTIAL_SERVICE: &str = "com.meguru.movel";
const REFRESH_ACCOUNT: &str = "lightnovel-refresh-token";
const DEVICE_ACCOUNT: &str = "lightnovel-device-id";
// The official Web client intentionally keeps a session token for only 30 seconds
// before exchanging the stored refresh token for a new one. A connected Hub keeps
// its session; refreshes occur only when a new Hub connection is needed.
const SESSION_TOKEN_TTL: Duration = Duration::from_secs(30);

#[derive(Default)]
struct Session {
    token: String,
    expires_at: Option<Instant>,
}

type HubSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Default)]
struct HubSession {
    socket: Option<HubSocket>,
}

#[derive(Clone)]
pub(crate) struct OfficialClient {
    http: Client,
    session: Arc<Mutex<Session>>,
    refresh_lock: Arc<Mutex<()>>,
    hub_session: Arc<Mutex<HubSession>>,
    refresh: Arc<Entry>,
    device: Arc<Entry>,
}

impl OfficialClient {
    pub(crate) fn new() -> Result<Self> {
        let http = Client::builder()
            .gzip(true)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Network(e.to_string()))?;
        let refresh = Entry::new(CREDENTIAL_SERVICE, REFRESH_ACCOUNT)
            .map_err(|e| AppError::Credentials(e.to_string()))?;
        let device = Entry::new(CREDENTIAL_SERVICE, DEVICE_ACCOUNT)
            .map_err(|e| AppError::Credentials(e.to_string()))?;
        Ok(Self {
            http,
            session: Arc::new(Mutex::new(Session::default())),
            refresh_lock: Arc::new(Mutex::new(())),
            hub_session: Arc::new(Mutex::new(HubSession::default())),
            refresh: Arc::new(refresh),
            device: Arc::new(device),
        })
    }

    pub(crate) async fn login(&self, email: String, password: String) -> Result<Value> {
        let password = format!("{:x}", Sha256::digest(password.as_bytes()));
        let value = self
            .http_envelope(
                "/api/user/login",
                json!({ "email": email, "password": password }),
            )
            .await?;
        self.save_login(&value).await?;
        self.hub("GetMyInfo", json!({})).await
    }

    pub(crate) async fn register(
        &self,
        user_name: String,
        email: String,
        password: String,
        code: String,
        invite_code: String,
    ) -> Result<Value> {
        let password = format!("{:x}", Sha256::digest(password.as_bytes()));
        let value = self.http_envelope("/api/user/register", json!({ "userName": user_name, "email": email, "password": password, "code": code, "inviteCode": invite_code })).await?;
        self.save_login(&value).await?;
        self.hub("GetMyInfo", json!({})).await
    }

    pub(crate) async fn send_register_email(&self, email: String) -> Result<()> {
        let mut url = Url::parse(&format!("{API_BASE}/api/user/send_register_email"))
            .map_err(|e| AppError::InvalidResponse(e.to_string()))?;
        url.query_pairs_mut().append_pair("email", &email);
        let response = self
            .http
            .get(url)
            .header("x-id", self.device_id()?)
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(network)?;
        decode_envelope(response).await.map(|_| ())
    }

    pub(crate) async fn restore_user(&self) -> Result<Option<Value>> {
        if self.token().await?.is_empty() {
            return Ok(None);
        }
        match self.hub("GetMyInfo", json!({})).await {
            Ok(user) => Ok(Some(user)),
            Err(AppError::AuthenticationExpired) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub(crate) async fn logout(&self) -> Result<()> {
        *self.session.lock().await = Session::default();
        self.invalidate_hub().await;
        match self.refresh.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Credentials(e.to_string())),
        }
    }

    pub(crate) async fn hub(&self, method: &str, payload: Value) -> Result<Value> {
        match self.hub_once(method, payload.clone()).await {
            Err(AppError::Network(_)) => {
                self.invalidate_hub().await;
                self.hub_once(method, payload).await
            }
            Err(error) if is_authentication_failure(&error) => {
                self.invalidate_access_token().await;
                self.invalidate_hub().await;
                if self.token().await?.is_empty() {
                    return Err(AppError::AuthenticationExpired);
                }

                match self.hub_once(method, payload).await {
                    Err(error) if is_authentication_failure(&error) => {
                        self.clear_credentials().await;
                        Err(AppError::AuthenticationExpired)
                    }
                    result => result,
                }
            }
            result => result,
        }
    }

    async fn hub_once(&self, method: &str, payload: Value) -> Result<Value> {
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
            .ok_or_else(|| AppError::InvalidResponse("SignalR 连接未初始化".into()))?;
        let result = Self::invoke_hub(socket, method, payload).await;
        if matches!(
            result,
            Err(AppError::Network(_) | AppError::InvalidResponse(_))
        ) {
            *hub_session = HubSession::default();
        }
        result
    }

    async fn connect_hub(&self, token: &str) -> Result<HubSocket> {
        let negotiate_url = format!("{API_BASE}/hub/api/negotiate?negotiateVersion=1");
        let mut request = self
            .http
            .post(negotiate_url)
            .header("x-id", self.device_id()?);
        if !token.is_empty() {
            request = request.bearer_auth(token);
        }
        let response = request.send().await.map_err(network)?;
        let status = response.status().as_u16() as i64;
        if !response.status().is_success() {
            return Err(AppError::Upstream {
                status,
                message: "SignalR 协商请求失败".into(),
            });
        }
        let negotiation: Value = response.json().await.map_err(network)?;
        let connection_token = negotiation
            .get("connectionToken")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::InvalidResponse("SignalR negotiate 未返回 connectionToken".into())
            })?;
        let mut url = Url::parse(&format!("{API_BASE}/hub/api"))
            .map_err(|e| AppError::InvalidResponse(e.to_string()))?;
        url.set_scheme("wss")
            .map_err(|_| AppError::InvalidResponse("无法建立 WebSocket 地址".into()))?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("id", connection_token);
            if !token.is_empty() {
                query.append_pair("access_token", token);
            }
        }
        let (mut socket, _) = connect_async(url.as_str()).await.map_err(websocket_error)?;
        socket
            .send(Message::Text(
                "{\"protocol\":\"messagepack\",\"version\":1}\u{1e}".into(),
            ))
            .await
            .map_err(network)?;
        let handshake = socket
            .next()
            .await
            .ok_or_else(|| AppError::Network("SignalR 连接意外关闭".into()))?
            .map_err(network)?;
        let handshake_text = handshake.into_text().map_err(network)?;
        if !handshake_text.starts_with("{}") {
            return Err(AppError::InvalidResponse(format!(
                "SignalR 握手失败：{handshake_text}"
            )));
        }
        Ok(socket)
    }

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
        .map_err(|e| AppError::InvalidResponse(e.to_string()))?;
        socket
            .send(Message::Binary(with_length_prefix(frame).into()))
            .await
            .map_err(network)?;
        while let Some(message) = socket.next().await {
            let message = message.map_err(network)?;
            if let Message::Binary(bytes) = message {
                for frame in split_binary_frames(&bytes)? {
                    let data: Vec<MessagePackValue> = rmp_serde::from_slice(frame)
                        .map_err(|e| AppError::InvalidResponse(e.to_string()))?;
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
                    return decode_hub_envelope(data.get(4).ok_or_else(|| {
                        AppError::InvalidResponse("SignalR 完成消息缺少结果".into())
                    })?);
                }
            }
        }
        Err(AppError::Network("SignalR 连接在返回结果前关闭".into()))
    }

    async fn save_login(&self, value: &Value) -> Result<()> {
        let token = value
            .get("Token")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::InvalidResponse("登录响应缺少 Token".into()))?;
        let refresh = value
            .get("RefreshToken")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::InvalidResponse("登录响应缺少 RefreshToken".into()))?;
        self.refresh
            .set_password(refresh)
            .map_err(|e| AppError::Credentials(e.to_string()))?;
        *self.session.lock().await = Session {
            token: token.into(),
            expires_at: Some(Instant::now() + SESSION_TOKEN_TTL),
        };
        self.invalidate_hub().await;
        Ok(())
    }

    async fn token(&self) -> Result<String> {
        if let Some(token) = self.cached_token().await {
            return Ok(token);
        }
        let _refresh_lock = self.refresh_lock.lock().await;
        if let Some(token) = self.cached_token().await {
            return Ok(token);
        }
        let refresh = match self.refresh.get_password() {
            Ok(token) => token,
            Err(keyring::Error::NoEntry) => return Ok(String::new()),
            Err(e) => return Err(AppError::Credentials(e.to_string())),
        };
        let response = self
            .http_envelope("/api/user/refresh_token", json!({ "token": refresh }))
            .await;
        match response {
            Ok(value) => {
                let token = value
                    .as_str()
                    .ok_or_else(|| AppError::InvalidResponse("刷新响应不是 Token 字符串".into()))?
                    .to_owned();
                *self.session.lock().await = Session {
                    token: token.clone(),
                    expires_at: Some(Instant::now() + SESSION_TOKEN_TTL),
                };
                Ok(token)
            }
            Err(error) if is_authentication_failure(&error) => {
                self.clear_credentials().await;
                Err(AppError::AuthenticationExpired)
            }
            Err(error) => Err(error),
        }
    }

    async fn invalidate_access_token(&self) {
        *self.session.lock().await = Session::default();
    }

    async fn cached_token(&self) -> Option<String> {
        let session = self.session.lock().await;
        session
            .expires_at
            .filter(|expires| *expires > Instant::now())
            .map(|_| session.token.clone())
    }

    async fn invalidate_hub(&self) {
        *self.hub_session.lock().await = HubSession::default();
    }

    async fn clear_credentials(&self) {
        self.invalidate_access_token().await;
        self.invalidate_hub().await;
        if let Err(error) = self.refresh.delete_credential() {
            if !matches!(error, keyring::Error::NoEntry) {
                // A failure to remove a now-invalid credential must not hide the
                // authentication result from the UI.
            }
        }
    }

    async fn http_envelope(&self, path: &str, payload: Value) -> Result<Value> {
        let response = self
            .http
            .post(format!("{API_BASE}{path}"))
            .header("x-id", self.device_id()?)
            .header(header::ACCEPT, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(network)?;
        decode_envelope(response).await
    }

    fn device_id(&self) -> Result<String> {
        match self.device.get_password() {
            Ok(value) => Ok(value),
            Err(keyring::Error::NoEntry) => {
                let value = Uuid::new_v4().to_string();
                self.device
                    .set_password(&value)
                    .map_err(|e| AppError::Credentials(e.to_string()))?;
                Ok(value)
            }
            Err(e) => Err(AppError::Credentials(e.to_string())),
        }
    }
}

fn network<E: std::fmt::Display>(error: E) -> AppError {
    AppError::Network(error.to_string())
}

fn websocket_error(error: tokio_tungstenite::tungstenite::Error) -> AppError {
    match error {
        tokio_tungstenite::tungstenite::Error::Http(response) => AppError::Upstream {
            status: response.status().as_u16() as i64,
            message: "SignalR WebSocket 连接被拒绝".into(),
        },
        error => network(error),
    }
}

fn is_authentication_failure(error: &AppError) -> bool {
    match error {
        AppError::Upstream { status, message } => {
            matches!(*status, 401 | 403)
                || [
                    "unauthorized",
                    "forbidden",
                    "authentication",
                    "authorization",
                    "token",
                    "jwt",
                    "bearer",
                    "未登录",
                    "未授权",
                    "登录",
                    "认证",
                ]
                .iter()
                .any(|marker| message.to_lowercase().contains(&marker.to_lowercase()))
        }
        AppError::AuthenticationExpired => true,
        _ => false,
    }
}

async fn decode_envelope(response: reqwest::Response) -> Result<Value> {
    let status = response.status().as_u16() as i64;
    let value: Value = response.json().await.map_err(|error| {
        if matches!(status, 401 | 403) {
            AppError::Upstream {
                status,
                message: "认证请求被拒绝".into(),
            }
        } else {
            network(error)
        }
    })?;
    if !value
        .get("Success")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
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

fn decode_hub_envelope(value: &MessagePackValue) -> Result<Value> {
    if !hub_field(value, "Success")
        .and_then(MessagePackValue::as_bool)
        .unwrap_or(false)
    {
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
            .map_err(|error| AppError::InvalidResponse(format!("gzip 解压失败：{error}")))?;
        return serde_json::from_str(&json)
            .map_err(|error| AppError::InvalidResponse(format!("gzip JSON 解析失败：{error}")));
    }
    rmpv::ext::from_value(response).map_err(|error| AppError::InvalidResponse(error.to_string()))
}

fn hub_field<'a>(value: &'a MessagePackValue, name: &str) -> Option<&'a MessagePackValue> {
    value
        .as_map()?
        .iter()
        .find_map(|(key, value)| (key.as_str() == Some(name)).then_some(value))
}

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

fn split_binary_frames(data: &[u8]) -> Result<Vec<&[u8]>> {
    let mut frames = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let mut length = 0usize;
        let mut shift = 0;
        loop {
            let byte = *data
                .get(offset)
                .ok_or_else(|| AppError::InvalidResponse("不完整的 SignalR 帧".into()))?;
            offset += 1;
            length |= ((byte & 0x7f) as usize) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 28 {
                return Err(AppError::InvalidResponse("非法 SignalR 帧长度".into()));
            }
        }
        let end = offset
            .checked_add(length)
            .filter(|end| *end <= data.len())
            .ok_or_else(|| AppError::InvalidResponse("不完整的 SignalR 数据".into()))?;
        frames.push(&data[offset..end]);
        offset = end;
    }
    Ok(frames)
}

#[cfg(test)]
mod authentication_tests {
    use super::*;

    #[test]
    fn session_token_ttl_matches_official_web_client() {
        assert_eq!(SESSION_TOKEN_TTL, Duration::from_secs(30));
    }

    #[test]
    fn authentication_failure_recognizes_signalr_authorization_messages() {
        let error = AppError::Upstream {
            status: 500,
            message: "未授权访问".into(),
        };

        assert!(is_authentication_failure(&error));
    }
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
