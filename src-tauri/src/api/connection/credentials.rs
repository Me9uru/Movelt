use tauri_plugin_movel_credentials::CredentialStore;

use crate::error::{AppError, Result};

/// 原生凭据边界；测试使用内存实现，不访问真实账号。
pub(super) trait Credentials: Send + Sync {
    fn get(&self, account: &str) -> Result<Option<String>>;
    fn set(&self, account: &str, value: &str) -> Result<()>;
    fn delete(&self, account: &str) -> Result<()>;
}

impl Credentials for CredentialStore<tauri::Wry> {
    fn get(&self, account: &str) -> Result<Option<String>> {
        self.get(account)
            .map_err(|error| AppError::Credentials(error.to_string()))
    }

    fn set(&self, account: &str, value: &str) -> Result<()> {
        self.set(account, value)
            .map_err(|error| AppError::Credentials(error.to_string()))
    }

    fn delete(&self, account: &str) -> Result<()> {
        self.delete(account)
            .map_err(|error| AppError::Credentials(error.to_string()))
    }
}
