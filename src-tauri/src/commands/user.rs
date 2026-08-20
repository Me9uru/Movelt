use serde_json::Value;
use tauri::State;

use crate::{api::OfficialClient, dto::user::User, error::Result, reader_cache::ReaderCache};

use super::common::{number, optional_string, string};

#[tauri::command]
pub(crate) async fn login(
    client: State<'_, OfficialClient>,
    cache: State<'_, ReaderCache>,
    email: String,
    password: String,
) -> Result<User> {
    let user = user(client.login(email, password).await?)?;
    cache.clear();
    Ok(user)
}
#[tauri::command]
pub(crate) async fn register(
    client: State<'_, OfficialClient>,
    cache: State<'_, ReaderCache>,
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
    cache.clear();
    Ok(user)
}
#[tauri::command]
pub(crate) async fn send_register_email(
    client: State<'_, OfficialClient>,
    email: String,
) -> Result<()> {
    client.send_register_email(email).await
}
#[tauri::command]
pub(crate) async fn restore_user(client: State<'_, OfficialClient>) -> Result<Option<User>> {
    client.restore_user().await?.map(user).transpose()
}
#[tauri::command]
pub(crate) async fn logout(
    client: State<'_, OfficialClient>,
    cache: State<'_, ReaderCache>,
) -> Result<()> {
    client.logout().await?;
    cache.clear();
    Ok(())
}

fn user(value: Value) -> Result<User> {
    Ok(User {
        id: number(&value, "Id"),
        user_name: string(&value, "UserName"),
        avatar: optional_string(&value, "Avatar"),
        email: optional_string(&value, "Email"),
    })
}
