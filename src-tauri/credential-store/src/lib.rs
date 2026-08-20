use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(target_os = "android")]
mod android;
#[cfg(not(target_os = "android"))]
mod desktop;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("credential storage failure: {0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(target_os = "android")]
pub use android::CredentialStore;
#[cfg(not(target_os = "android"))]
pub use desktop::CredentialStore;

pub trait CredentialStoreExt<R: Runtime> {
    fn credential_store(&self) -> &CredentialStore<R>;
}

impl<R: Runtime, T: Manager<R>> CredentialStoreExt<R> for T {
    fn credential_store(&self) -> &CredentialStore<R> {
        self.state::<CredentialStore<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("movel-credentials")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let store = android::init(app, api)?;
            #[cfg(not(target_os = "android"))]
            let store = desktop::init(app, api)?;
            app.manage(store);
            Ok(())
        })
        .build()
}
