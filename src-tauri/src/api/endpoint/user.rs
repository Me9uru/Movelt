use reqwest::header;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use url::Url;

use crate::error::{AppError, Result};

use super::super::connection::{
    decode_envelope, transport, OfficialClient, Session, API_BASE, REFRESH_ACCOUNT,
};

impl OfficialClient {
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
        let value = self
            .http_envelope(
                "/api/user/register",
                json!({
                    "userName": user_name,
                    "email": email,
                    "password": password,
                    "code": code,
                    "inviteCode": invite_code,
                }),
            )
            .await?;
        self.save_login(&value).await?;
        self.hub("GetMyInfo", json!({})).await
    }

    pub(crate) async fn send_register_email(&self, email: String) -> Result<()> {
        let mut url =
            Url::parse(&format!("{API_BASE}/api/user/send_register_email")).map_err(|error| {
                AppError::Internal {
                    detail: format!("构造注册邮件地址失败：{error}"),
                }
            })?;
        url.query_pairs_mut().append_pair("email", &email);
        let response = self
            .http
            .get(url)
            .header("x-id", &self.device_id)
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(transport)?;
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
        let _ = self.delete_credential(REFRESH_ACCOUNT);
        Ok(())
    }
}
