use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    api::cache::AppCache,
    error::{AppError, Result},
};
use tauri_plugin_movel_credentials::CredentialStore;

use super::{
    credentials::Credentials,
    session::{Session, SessionContext},
};

pub(in crate::api) const API_BASE: &str = "https://api.lightnovel.life";
pub(in crate::api) const REFRESH_ACCOUNT: &str = "lightnovel-refresh-token";
const DEVICE_ACCOUNT: &str = "lightnovel-device-id";
const SESSION_TOKEN_TTL: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub(crate) struct OfficialClient {
    pub(in crate::api) http: Client,
    pub(super) session: Arc<Mutex<Session>>,
    refresh_lock: Arc<Mutex<()>>,
    pub(super) context: Arc<SessionContext>,
    credentials: Arc<dyn Credentials>,
    pub(in crate::api) device_id: String,
    pub(super) http_base: String,
    #[cfg(test)]
    pub(super) test_hub: Option<super::tests::TestHub>,
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
        Ok(Self::with_credentials(
            http,
            Arc::new(credentials),
            device_id,
        ))
    }

    pub(super) fn with_credentials(
        http: Client,
        credentials: Arc<dyn Credentials>,
        device_id: String,
    ) -> Self {
        let session = Session::default();
        Self {
            http,
            // 管理态只用于创建快照，不持有任何账号的连接或缓存。
            context: Arc::default(),
            session: Arc::new(Mutex::new(session)),
            refresh_lock: Arc::new(Mutex::new(())),
            credentials,
            device_id,
            http_base: API_BASE.into(),
            #[cfg(test)]
            test_hub: None,
        }
    }

    /// 命令入口捕获身份；后续调用和预加载必须沿用这个快照。
    pub(crate) async fn scoped(&self) -> Self {
        let mut client = self.clone();
        client.context = self.session.lock().await.context.clone();
        client
    }

    pub(crate) fn cache(&self) -> &AppCache {
        &self.context.cache
    }

    pub(crate) async fn lock_bookshelf(&self) -> Result<tokio::sync::MutexGuard<'_, ()>> {
        let guard = self.context.bookshelf.lock().await;
        self.ensure_current().await?;
        Ok(guard)
    }

    pub(super) async fn ensure_current(&self) -> Result<()> {
        self.session.lock().await.check(&self.context)
    }

    /// 调用 SignalR 方法；遇到连接或认证问题时自动恢复后重试。
    pub(in crate::api) async fn hub(&self, method: &str, payload: Value) -> Result<Value> {
        // Keep transport state out of every caller's inline future. Tauri constructs
        // command futures on Android's small JavaBridge stack before spawning them.
        match Box::pin(self.hub_once(method, payload.clone())).await {
            // 写请求丢失响应时可能已在服务器提交，不能盲目重放。
            Err(error)
                if error.is_transport()
                    && (method.starts_with("Get") || method == "SearchComicSeries") =>
            {
                Box::pin(self.hub_once(method, payload)).await
            }
            Err(error)
                if matches!(error, AppError::Upstream { .. })
                    && is_authentication_failure(&error) =>
            {
                self.invalidate_access_token().await?;
                self.invalidate_hub().await;
                if self.token().await?.is_empty() {
                    return Err(AppError::AuthenticationExpired);
                }

                match Box::pin(self.hub_once(method, payload)).await {
                    Err(error) if is_authentication_failure(&error) => {
                        self.clear_credentials().await?;
                        Err(AppError::AuthenticationExpired)
                    }
                    result => result,
                }
            }
            result => result,
        }
    }

    /// 保存登录令牌及刷新凭据，并重置现有 Hub 连接。
    pub(in crate::api) async fn save_login(&self, value: &Value) -> Result<Self> {
        let token = value
            .get("Token")
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| AppError::protocol("登录响应缺少 Token"))?;
        let refresh = value
            .get("RefreshToken")
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| AppError::protocol("登录响应缺少 RefreshToken"))?;
        let mut session = self.session.lock().await;
        session.check(&self.context)?;
        session.reset();
        session.token = token.into();
        session.expires_at = Some(Instant::now() + SESSION_TOKEN_TTL);
        session.refresh_token = Some(refresh.into());
        // 密钥环不可用时仅保留内存会话。
        let _ = self.credentials.set(REFRESH_ACCOUNT, refresh);
        let mut client = self.clone();
        client.context = session.context.clone();
        Ok(client)
    }

    /// 返回有效访问令牌，必要时使用刷新令牌换取新令牌。
    pub(in crate::api) async fn token(&self) -> Result<String> {
        if let Some(token) = self.cached_token().await? {
            return Ok(token);
        }
        let _refresh_lock = self.refresh_lock.lock().await;
        if let Some(token) = self.cached_token().await? {
            return Ok(token);
        }
        let refresh = {
            let session = self.session.lock().await;
            session.check(&self.context)?;
            match &session.refresh_token {
                Some(refresh) => Some(refresh.clone()),
                None if session.restore_from_store => self.credentials.get(REFRESH_ACCOUNT)?,
                None => None,
            }
        };
        let Some(refresh) = refresh else {
            return Ok(String::new());
        };
        let refresh_for_session = refresh.clone();
        let response = self
            .http_envelope("/api/user/refresh_token", json!({ "token": refresh }))
            .await;
        self.ensure_current().await?;
        match response {
            Ok(value) => {
                let token = value
                    .as_str()
                    .filter(|token| !token.is_empty())
                    .ok_or_else(|| AppError::protocol("刷新响应不是 Token 字符串"))?
                    .to_owned();
                let mut session = self.session.lock().await;
                session.check(&self.context)?;
                session.token = token.clone();
                session.expires_at = Some(Instant::now() + SESSION_TOKEN_TTL);
                session.refresh_token = Some(refresh_for_session);
                Ok(token)
            }
            Err(error) if is_authentication_failure(&error) => {
                self.clear_credentials().await?;
                Err(AppError::AuthenticationExpired)
            }
            Err(error) => Err(error),
        }
    }

    /// 清空内存中的访问令牌。
    pub(super) async fn invalidate_access_token(&self) -> Result<()> {
        let mut session = self.session.lock().await;
        session.check(&self.context)?;
        session.invalidate_access_token();
        Ok(())
    }

    /// 读取尚未过期的内存访问令牌。
    async fn cached_token(&self) -> Result<Option<String>> {
        let session = self.session.lock().await;
        session.check(&self.context)?;
        Ok(session
            .expires_at
            .filter(|expires| *expires > Instant::now())
            .map(|_| session.token.clone()))
    }

    /// 清除内存会话、Hub 连接和持久化刷新凭据。
    pub(in crate::api) async fn clear_credentials(&self) -> Result<()> {
        let mut session = self.session.lock().await;
        session.check(&self.context)?;
        // 即使系统删除失败，也立即隔离旧连接、缓存并禁止再次恢复旧凭据。
        session.reset();
        self.credentials.delete(REFRESH_ACCOUNT)
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
