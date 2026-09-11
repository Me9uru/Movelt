use std::{sync::Arc, time::Instant};

use tokio::sync::Mutex;

use crate::{
    api::cache::AppCache,
    error::{AppError, Result},
};

use super::hub::HubSession;

/// 每次身份切换创建新的上下文。旧请求继续持有旧连接和旧缓存。
#[derive(Default)]
pub(super) struct SessionContext {
    pub cache: AppCache,
    pub hub: Mutex<HubSession>,
    pub connecting: Mutex<()>,
    pub bookshelf: Mutex<()>,
}

pub(super) struct Session {
    pub context: Arc<SessionContext>,
    pub token: String,
    pub expires_at: Option<Instant>,
    pub refresh_token: Option<String>,
    pub restore_from_store: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            context: Arc::default(),
            token: String::new(),
            expires_at: None,
            refresh_token: None,
            restore_from_store: true,
        }
    }
}

impl Session {
    pub fn check(&self, context: &Arc<SessionContext>) -> Result<()> {
        if Arc::ptr_eq(&self.context, context) {
            Ok(())
        } else {
            Err(AppError::SessionChanged)
        }
    }

    pub fn invalidate_access_token(&mut self) {
        self.token.clear();
        self.expires_at = None;
    }

    pub fn reset(&mut self) {
        *self = Self {
            restore_from_store: false,
            ..Self::default()
        };
    }
}
