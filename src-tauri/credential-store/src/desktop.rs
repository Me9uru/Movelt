use std::marker::PhantomData;

use keyring::Entry;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::{Error, Result};

const SERVICE: &str = "com.meguru.movel";

pub struct CredentialStore<R: Runtime>(PhantomData<fn() -> R>);

impl<R: Runtime> Clone for CredentialStore<R> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

pub fn init<R: Runtime, C: serde::de::DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> Result<CredentialStore<R>> {
    Ok(CredentialStore(PhantomData))
}

impl<R: Runtime> CredentialStore<R> {
    pub fn get(&self, account: &str) -> Result<Option<String>> {
        match entry(account)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(Error::Storage(error.to_string())),
        }
    }

    pub fn set(&self, account: &str, value: &str) -> Result<()> {
        entry(account)?
            .set_password(value)
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub fn delete(&self, account: &str) -> Result<()> {
        match entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(Error::Storage(error.to_string())),
        }
    }
}

fn entry(account: &str) -> Result<Entry> {
    Entry::new(SERVICE, account).map_err(|error| Error::Storage(error.to_string()))
}
