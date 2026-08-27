use serde_json::Value;
use tauri::State;

use crate::{
    api::{cache::AppCache, OfficialClient},
    dto::user::{Growth, User},
    error::Result,
};

use super::adapter::{number, optional_string, string};

#[tauri::command]
/// 使用邮箱和密码登录，并清理阅读缓存。
pub(crate) async fn login(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    email: String,
    password: String,
) -> Result<User> {
    let user = user(client.login(email, password).await?)?;
    cache.clear_cache();
    Ok(user)
}

#[tauri::command]
/// 注册用户、保存登录态并清理阅读缓存。
///
/// Tauri 将应用状态与注册字段分别注入此命令；保持现有调用 DTO，避免为
/// 缓存依赖额外扩大前端请求结构。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn register(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    user_name: String,
    email: String,
    password: String,
    code: String,
    invite_code: String,
) -> Result<User> {
    let user = user(
        client
            .register(user_name, email, password, code, invite_code)
            .await?,
    )?;
    cache.clear_cache();
    Ok(user)
}

#[tauri::command]
/// 发送注册验证码邮件。
pub(crate) async fn send_register_email(
    client: State<'_, OfficialClient>,
    email: String,
) -> Result<()> {
    client.send_register_email(email).await
}

#[tauri::command]
/// 恢复当前用户信息。
pub(crate) async fn restore_user(client: State<'_, OfficialClient>) -> Result<Option<User>> {
    client.restore_user().await?.map(user).transpose()
}

#[tauri::command]
/// 更新头像并返回最新的用户资料。
pub(crate) async fn set_avatar(client: State<'_, OfficialClient>, url: String) -> Result<User> {
    validate_avatar_url(&url)?;
    user(client.set_avatar(url).await?)
}

#[tauri::command]
/// 每日签到并返回最新的用户资料。
pub(crate) async fn sign_in(client: State<'_, OfficialClient>) -> Result<User> {
    user(client.sign_in().await?)
}

#[tauri::command]
/// 注销当前会话并清理阅读缓存。
pub(crate) async fn logout(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
) -> Result<()> {
    client.logout().await?;
    cache.clear_cache();
    Ok(())
}

/// 将官方用户数据映射为应用 DTO。
fn user(value: Value) -> Result<User> {
    Ok(User {
        id: number(&value, "Id"),
        user_name: string(&value, "UserName"),
        avatar: optional_string(&value, "Avatar"),
        email: optional_string(&value, "Email"),
        invite_code: optional_string(&value, "InviteCode"),
        user_group: value
            .get("Role")
            .and_then(|role| optional_string(role, "Name")),
        register_at: optional_string(&value, "RegisterAt"),
        growth: growth(value.get("Growth")),
    })
}

fn growth(value: Option<&Value>) -> Option<Growth> {
    let value = value?;
    value.as_object()?;
    Some(Growth {
        exp: number(value, "Exp"),
        coin: number(value, "Coin"),
        level: number(value, "Level"),
        growth_level: number(value, "GrowthLevel"),
        current_level_exp: number(value, "CurrentLevelExp"),
        next_level_exp: optional_number(value, "NextLevelExp"),
        sign_streak: number(value, "SignStreak"),
        today_signed: value
            .get("TodaySigned")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn optional_number(value: &Value, key: &str) -> Option<i64> {
    value
        .get(key)
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
}

fn validate_avatar_url(value: &str) -> Result<()> {
    let url = url::Url::parse(value)
        .map_err(|_| crate::error::AppError::invalid_input("头像地址必须是有效的 HTTPS URL"))?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err(crate::error::AppError::invalid_input(
            "头像地址必须是有效的 HTTPS URL",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{user, validate_avatar_url};

    #[test]
    fn maps_full_user_profile() {
        let mapped = user(json!({
            "Id": 42,
            "UserName": "读者",
            "Email": "reader@example.com",
            "InviteCode": "welcome",
            "Role": { "Name": "普通用户" },
            "RegisterAt": "2025-01-02T03:04:05Z",
            "Growth": {
                "Exp": 10,
                "Coin": 20,
                "Level": 2,
                "GrowthLevel": 1,
                "CurrentLevelExp": 0,
                "NextLevelExp": 100,
                "SignStreak": 3,
                "TodaySigned": true,
            },
        }))
        .expect("profile should map");

        assert_eq!(mapped.user_group.as_deref(), Some("普通用户"));
        assert_eq!(mapped.register_at.as_deref(), Some("2025-01-02T03:04:05Z"));
        assert_eq!(mapped.growth.expect("growth").coin, 20);
    }

    #[test]
    fn accepts_only_https_avatar_urls() {
        assert!(validate_avatar_url("https://example.com/avatar.png").is_ok());
        assert!(validate_avatar_url("http://example.com/avatar.png").is_err());
        assert!(validate_avatar_url("not-a-url").is_err());
    }
}
