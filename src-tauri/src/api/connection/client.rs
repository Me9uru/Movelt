use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{AppError, Result};
use tauri_plugin_movel_credentials::CredentialStore;

use super::hub::HubSession;

pub(in crate::api) const API_BASE: &str = "https://api.lightnovel.life";
pub(in crate::api) const REFRESH_ACCOUNT: &str = "lightnovel-refresh-token";
const DEVICE_ACCOUNT: &str = "lightnovel-device-id";
const SESSION_TOKEN_TTL: Duration = Duration::from_secs(30);

#[derive(Default)]
pub(in crate::api) struct Session {
    token: String,
    expires_at: Option<Instant>,
    refresh_token: Option<String>,
}

#[derive(Clone)]
pub(crate) struct OfficialClient {
    pub(in crate::api) http: Client,
    pub(in crate::api) session: Arc<Mutex<Session>>,
    refresh_lock: Arc<Mutex<()>>,
    pub(in crate::api) hub_session: Arc<Mutex<HubSession>>,
    credentials: CredentialStore<tauri::Wry>,
    pub(in crate::api) device_id: String,
}

impl OfficialClient {
    /// 创建官方 API 客户端，并恢复或生成设备标识。
    pub(crate) fn new(credentials: CredentialStore<tauri::Wry>) -> Result<Self> {
        let http = Client::builder()
            .gzip(true)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Internal {
                detail: format!("创建 HTTP 客户端失败：{e}"),
            })?;
        let device_id = credentials
            .get(DEVICE_ACCOUNT)
            .ok()
            .flatten()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let _ = credentials.set(DEVICE_ACCOUNT, &device_id);
        Ok(Self {
            http,
            session: Arc::new(Mutex::new(Session::default())),
            refresh_lock: Arc::new(Mutex::new(())),
            hub_session: Arc::new(Mutex::new(HubSession::default())),
            credentials,
            device_id,
        })
    }

    /// 调用 SignalR 方法；遇到连接或认证问题时自动恢复后重试。
    pub(in crate::api) async fn hub(&self, method: &str, payload: Value) -> Result<Value> {
        match self.hub_once(method, payload.clone()).await {
            Err(error) if error.is_transport() => {
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

    /// 保存登录令牌及刷新凭据，并重置现有 Hub 连接。
    pub(in crate::api) async fn save_login(&self, value: &Value) -> Result<()> {
        let token = value
            .get("Token")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::protocol("登录响应缺少 Token"))?;
        let refresh = value
            .get("RefreshToken")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::protocol("登录响应缺少 RefreshToken"))?;
        *self.session.lock().await = Session {
            token: token.into(),
            expires_at: Some(Instant::now() + SESSION_TOKEN_TTL),
            refresh_token: Some(refresh.into()),
        };
        // 密钥环不可用时仅保留内存会话。
        let _ = self.set_credential(REFRESH_ACCOUNT, refresh);
        self.invalidate_hub().await;
        Ok(())
    }

    /// 返回有效访问令牌，必要时使用刷新令牌换取新令牌。
    pub(in crate::api) async fn token(&self) -> Result<String> {
        if let Some(token) = self.cached_token().await {
            return Ok(token);
        }
        let _refresh_lock = self.refresh_lock.lock().await;
        if let Some(token) = self.cached_token().await {
            return Ok(token);
        }
        let refresh = self
            .session
            .lock()
            .await
            .refresh_token
            .clone()
            .or_else(|| self.get_credential(REFRESH_ACCOUNT).ok().flatten());
        let Some(refresh) = refresh else {
            return Ok(String::new());
        };
        let refresh_for_session = refresh.clone();
        let response = self
            .http_envelope("/api/user/refresh_token", json!({ "token": refresh }))
            .await;
        match response {
            Ok(value) => {
                let token = value
                    .as_str()
                    .ok_or_else(|| AppError::protocol("刷新响应不是 Token 字符串"))?
                    .to_owned();
                *self.session.lock().await = Session {
                    token: token.clone(),
                    expires_at: Some(Instant::now() + SESSION_TOKEN_TTL),
                    refresh_token: Some(refresh_for_session),
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

    /// 清空内存中的访问令牌。
    async fn invalidate_access_token(&self) {
        *self.session.lock().await = Session::default();
    }

    /// 读取尚未过期的内存访问令牌。
    async fn cached_token(&self) -> Option<String> {
        let session = self.session.lock().await;
        session
            .expires_at
            .filter(|expires| *expires > Instant::now())
            .map(|_| session.token.clone())
    }

    /// 清除内存会话、Hub 连接和持久化刷新凭据。
    async fn clear_credentials(&self) {
        self.invalidate_access_token().await;
        self.invalidate_hub().await;
        // 删除失效凭据失败不应掩盖认证结果。
        let _ = self.delete_credential(REFRESH_ACCOUNT);
    }

    /// 从系统凭据存储读取指定账户的值。
    fn get_credential(&self, account: &str) -> Result<Option<String>> {
        self.credentials
            .get(account)
            .map_err(|error| AppError::Credentials(error.to_string()))
    }

    /// 向系统凭据存储写入指定账户的值。
    fn set_credential(&self, account: &str, value: &str) -> Result<()> {
        self.credentials
            .set(account, value)
            .map_err(|error| AppError::Credentials(error.to_string()))
    }

    /// 从系统凭据存储删除指定账户的值。
    pub(in crate::api) fn delete_credential(&self, account: &str) -> Result<()> {
        self.credentials
            .delete(account)
            .map_err(|error| AppError::Credentials(error.to_string()))
    }
}

/// 判断错误是否表示认证已失效。
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

#[cfg(test)]
mod authentication_test {
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
