use reqwest::header;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use url::Url;

use crate::error::{AppError, Result};

use super::super::connection::{decode_envelope, transport, OfficialClient, API_BASE};

impl OfficialClient {
    pub(crate) async fn login(&self, email: String, password: String) -> Result<Value> {
        let password = format!("{:x}", Sha256::digest(password.as_bytes()));
        let value = self
            .http_envelope(
                "/api/user/login",
                json!({ "email": email, "password": password }),
            )
            .await?;
        self.save_login(&value)
            .await?
            .hub("GetMyInfo", json!({}))
            .await
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
        self.save_login(&value)
            .await?
            .hub("GetMyInfo", json!({}))
            .await
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
        match self.token().await {
            Ok(token) if token.is_empty() => return Ok(None),
            Err(AppError::AuthenticationExpired) => return Ok(None),
            Err(error) => return Err(error),
            Ok(_) => {}
        }
        match self.hub("GetMyInfo", json!({})).await {
            Ok(user) => Ok(Some(user)),
            Err(AppError::AuthenticationExpired) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub(crate) async fn set_avatar(&self, url: String) -> Result<Value> {
        self.hub("SetAvatar", json!({ "Url": url })).await?;
        self.hub("GetMyInfo", json!({})).await
    }

    pub(crate) async fn sign_in(&self) -> Result<Value> {
        self.hub("SignIn", json!({})).await?;
        self.hub("GetMyInfo", json!({})).await
    }

    pub(crate) async fn logout(&self) -> Result<()> {
        self.clear_credentials().await
    }
}
