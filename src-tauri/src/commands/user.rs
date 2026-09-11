use tauri::State;

use crate::{api::OfficialClient, dto::user::User, error::Result, mapping::user::user};

#[tauri::command]
/// 使用邮箱和密码登录，并清理阅读缓存。
pub(crate) async fn login(
    client: State<'_, OfficialClient>,
    email: String,
    password: String,
) -> Result<User> {
    let client = client.scoped().await;
    user(client.login(email, password).await?)
}

#[tauri::command]
/// 注册用户、保存登录态并清理阅读缓存。
pub(crate) async fn register(
    client: State<'_, OfficialClient>,
    user_name: String,
    email: String,
    password: String,
    code: String,
    invite_code: String,
) -> Result<User> {
    let client = client.scoped().await;
    user(
        client
            .register(user_name, email, password, code, invite_code)
            .await?,
    )
}

#[tauri::command]
/// 发送注册验证码邮件。
pub(crate) async fn send_register_email(
    client: State<'_, OfficialClient>,
    email: String,
) -> Result<()> {
    let client = client.scoped().await;
    client.send_register_email(email).await
}

#[tauri::command]
/// 恢复当前用户信息。
pub(crate) async fn restore_user(client: State<'_, OfficialClient>) -> Result<Option<User>> {
    let client = client.scoped().await;
    client.restore_user().await?.map(user).transpose()
}

#[tauri::command]
/// 更新头像并返回最新的用户资料。
pub(crate) async fn set_avatar(client: State<'_, OfficialClient>, url: String) -> Result<User> {
    let client = client.scoped().await;
    validate_avatar_url(&url)?;
    user(client.set_avatar(url).await?)
}

#[tauri::command]
/// 每日签到并返回最新的用户资料。
pub(crate) async fn sign_in(client: State<'_, OfficialClient>) -> Result<User> {
    let client = client.scoped().await;
    user(client.sign_in().await?)
}

#[tauri::command]
/// 注销当前会话并清理阅读缓存。
pub(crate) async fn logout(client: State<'_, OfficialClient>) -> Result<()> {
    let client = client.scoped().await;
    client.logout().await
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
    use super::validate_avatar_url;

    #[test]
    fn accepts_only_https_avatar_urls() {
        assert!(validate_avatar_url("https://example.com/avatar.png").is_ok());
        assert!(validate_avatar_url("http://example.com/avatar.png").is_err());
        assert!(validate_avatar_url("not-a-url").is_err());
    }
}
