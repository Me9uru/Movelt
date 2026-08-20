use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::{Error, Result};

const PLUGIN_IDENTIFIER: &str = "com.meguru.movel.credentials";

#[derive(Serialize)]
struct AccountRequest<'a> {
    account: &'a str,
}

#[derive(Serialize)]
struct SetRequest<'a> {
    account: &'a str,
    value: &'a str,
}

#[derive(Deserialize)]
struct GetResponse {
    value: Option<String>,
}

pub struct CredentialStore<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Clone for CredentialStore<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub fn init<R: Runtime, C: serde::de::DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<CredentialStore<R>> {
    let handle = api
        .register_android_plugin(PLUGIN_IDENTIFIER, "CredentialStorePlugin")
        .map_err(|error| Error::Storage(error.to_string()))?;
    Ok(CredentialStore(handle))
}

impl<R: Runtime> CredentialStore<R> {
    pub fn get(&self, account: &str) -> Result<Option<String>> {
        let response: GetResponse = self
            .0
            .run_mobile_plugin("get", AccountRequest { account })
            .map_err(|error| Error::Storage(error.to_string()))?;
        Ok(response.value)
    }

    pub fn set(&self, account: &str, value: &str) -> Result<()> {
        self.0
            .run_mobile_plugin::<()>("set", SetRequest { account, value })
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub fn delete(&self, account: &str) -> Result<()> {
        self.0
            .run_mobile_plugin::<()>("delete", AccountRequest { account })
            .map_err(|error| Error::Storage(error.to_string()))
    }
}
