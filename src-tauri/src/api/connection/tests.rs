use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use serde_json::{json, Value};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
};

use crate::error::{AppError, Result};

use super::{
    client::{OfficialClient, REFRESH_ACCOUNT},
    credentials::Credentials,
};

pub(super) type TestHub = Arc<
    dyn Fn(
            String,
            Value,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
>;

pub(crate) async fn mock_client<F, Fut>(hub: F) -> OfficialClient
where
    F: Fn(String, Value) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Value>> + Send + 'static,
{
    let mut root = client(Arc::default());
    root.test_hub = Some(Arc::new(move |method, payload| {
        Box::pin(hub(method, payload))
    }));
    root.scoped().await
}

#[derive(Default)]
struct MemoryCredentials {
    values: Mutex<HashMap<String, String>>,
    fail_set: AtomicBool,
    fail_delete: AtomicBool,
}

impl Credentials for MemoryCredentials {
    fn get(&self, account: &str) -> Result<Option<String>> {
        Ok(self.values.lock().unwrap().get(account).cloned())
    }

    fn set(&self, account: &str, value: &str) -> Result<()> {
        if self.fail_set.load(Ordering::SeqCst) {
            return Err(AppError::Credentials("test storage unavailable".into()));
        }
        self.values
            .lock()
            .unwrap()
            .insert(account.into(), value.into());
        Ok(())
    }

    fn delete(&self, account: &str) -> Result<()> {
        if self.fail_delete.load(Ordering::SeqCst) {
            return Err(AppError::Credentials("test deletion failed".into()));
        }
        self.values.lock().unwrap().remove(account);
        Ok(())
    }
}

fn client(credentials: Arc<MemoryCredentials>) -> OfficialClient {
    OfficialClient::with_credentials(
        reqwest::Client::builder().no_proxy().build().unwrap(),
        credentials,
        "test-device".into(),
    )
}

async fn login(root: &OfficialClient, account: &str) -> OfficialClient {
    root.scoped()
        .await
        .save_login(&json!({
            "Token": format!("access-{account}"), "RefreshToken": format!("refresh-{account}")
        }))
        .await
        .unwrap()
}

/// 只监听随机本地端口；测试决定何时让在途 HTTP 请求完成。
async fn delayed_http(
    root: &mut OfficialClient,
) -> (oneshot::Receiver<Value>, oneshot::Sender<Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    root.http_base = format!("http://{}", listener.local_addr().unwrap());
    let (request_tx, request_rx) = oneshot::channel();
    let (response_tx, response_rx) = oneshot::channel::<Value>();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut bytes = Vec::new();
        let (offset, size) = loop {
            let mut buffer = [0; 1024];
            let read = stream.read(&mut buffer).await.unwrap();
            assert_ne!(read, 0);
            bytes.extend_from_slice(&buffer[..read]);
            if let Some(index) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..index]);
                let size: usize = headers
                    .lines()
                    .find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        key.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse().unwrap())
                    })
                    .unwrap();
                break (index + 4, size);
            }
        };
        while bytes.len() < offset + size {
            let mut buffer = [0; 1024];
            let read = stream.read(&mut buffer).await.unwrap();
            assert_ne!(read, 0);
            bytes.extend_from_slice(&buffer[..read]);
        }
        request_tx
            .send(serde_json::from_slice(&bytes[offset..offset + size]).unwrap())
            .unwrap();
        let body = response_rx.await.unwrap().to_string();
        stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
    });
    (request_rx, response_tx)
}

#[tokio::test]
async fn expired_refresh_during_hub_creation_does_not_deadlock() {
    let credentials = Arc::new(MemoryCredentials::default());
    credentials.set(REFRESH_ACCOUNT, "expired").unwrap();
    let mut root = client(credentials.clone());
    let (request, reply) = delayed_http(&mut root).await;
    let scoped = root.scoped().await;
    let call = tokio::spawn(async move { scoped.hub("GetMyInfo", json!({})).await });
    request.await.unwrap();
    reply
        .send(json!({"Success": false, "Status": 401, "Msg": "expired"}))
        .unwrap();
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), call)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(result, Err(AppError::AuthenticationExpired)));
    assert!(root.scoped().await.token().await.unwrap().is_empty());
    assert_eq!(credentials.get(REFRESH_ACCOUNT).unwrap(), None);
}

#[tokio::test]
async fn late_refresh_cannot_restore_logged_out_session() {
    let credentials = Arc::new(MemoryCredentials::default());
    let mut root = client(credentials.clone());
    let (request, reply) = delayed_http(&mut root).await;
    let old = login(&root, "old").await;
    old.invalidate_access_token().await.unwrap();
    let refresh = tokio::spawn(async move { old.token().await });
    request.await.unwrap();
    root.scoped().await.logout().await.unwrap();
    reply
        .send(json!({"Success": true, "Response": "late-access"}))
        .unwrap();
    assert!(matches!(
        refresh.await.unwrap(),
        Err(AppError::SessionChanged)
    ));
    assert!(root.scoped().await.token().await.unwrap().is_empty());
    assert_eq!(credentials.get(REFRESH_ACCOUNT).unwrap(), None);
}

#[tokio::test]
async fn late_refresh_success_or_failure_cannot_change_new_account() {
    for response in [
        json!({"Success": true, "Response": "late-access"}),
        json!({"Success": false, "Status": 401}),
    ] {
        let credentials = Arc::new(MemoryCredentials::default());
        let mut root = client(credentials.clone());
        let (request, reply) = delayed_http(&mut root).await;
        let old = login(&root, "old").await;
        old.invalidate_access_token().await.unwrap();
        let refresh = tokio::spawn(async move { old.token().await });
        request.await.unwrap();
        let current = login(&root, "new").await;
        reply.send(response).unwrap();
        assert!(matches!(
            refresh.await.unwrap(),
            Err(AppError::SessionChanged)
        ));
        assert_eq!(current.token().await.unwrap(), "access-new");
        assert_eq!(
            credentials.get(REFRESH_ACCOUNT).unwrap().as_deref(),
            Some("refresh-new")
        );
    }
}

#[tokio::test]
async fn late_login_cannot_undo_logout() {
    let mut root = client(Arc::default());
    let (request, reply) = delayed_http(&mut root).await;
    let scoped = root.scoped().await;
    let pending =
        tokio::spawn(async move { scoped.login("test@example.com".into(), "test".into()).await });
    request.await.unwrap();
    root.scoped().await.logout().await.unwrap();
    reply
        .send(
            json!({"Success": true, "Response": {"Token": "late", "RefreshToken": "late-refresh"}}),
        )
        .unwrap();
    assert!(matches!(
        pending.await.unwrap(),
        Err(AppError::SessionChanged)
    ));
    assert!(root.scoped().await.token().await.unwrap().is_empty());
}

#[tokio::test]
async fn memory_only_login_retains_refresh_token_after_access_invalidation() {
    let credentials = Arc::new(MemoryCredentials::default());
    credentials.fail_set.store(true, Ordering::SeqCst);
    let mut root = client(credentials);
    let (request, reply) = delayed_http(&mut root).await;
    let scoped = login(&root, "memory").await;
    scoped.invalidate_access_token().await.unwrap();
    let refresh = tokio::spawn(async move { scoped.token().await });
    assert_eq!(request.await.unwrap(), json!({"token": "refresh-memory"}));
    reply
        .send(json!({"Success": true, "Response": "refreshed"}))
        .unwrap();
    assert_eq!(refresh.await.unwrap().unwrap(), "refreshed");
}

#[tokio::test]
async fn failed_credential_deletion_is_reported_and_cannot_restore_session() {
    let credentials = Arc::new(MemoryCredentials::default());
    let root = client(credentials.clone());
    let old = login(&root, "old").await;
    credentials.fail_delete.store(true, Ordering::SeqCst);
    assert!(matches!(old.logout().await, Err(AppError::Credentials(_))));
    assert!(matches!(old.token().await, Err(AppError::SessionChanged)));
    assert!(root.scoped().await.token().await.unwrap().is_empty());
    assert!(credentials.get(REFRESH_ACCOUNT).unwrap().is_some());
    credentials.fail_delete.store(false, Ordering::SeqCst);
    root.scoped().await.logout().await.unwrap();
    assert_eq!(credentials.get(REFRESH_ACCOUNT).unwrap(), None);
}

#[tokio::test]
async fn in_flight_cache_load_cannot_fill_new_account_cache() {
    let root = client(Arc::default());
    let old = login(&root, "old").await;
    let (started_tx, started_rx) = oneshot::channel();
    let (finish_tx, finish_rx) = oneshot::channel();
    let pending = tokio::spawn(async move {
        let cache = old.cache();
        cache
            .load_cache(&cache.novel_chapters, "book".into(), async {
                started_tx.send(()).unwrap();
                finish_rx.await.unwrap();
                Ok(vec!["old".into()])
            })
            .await
            .unwrap()
    });
    started_rx.await.unwrap();
    // save_login 本身就更换缓存，不依赖后续 GetMyInfo 是否成功。
    let current = login(&root, "new").await;
    let cache = current.cache();
    let result = cache
        .load_cache(&cache.novel_chapters, "book".into(), async {
            Ok(vec!["new".into()])
        })
        .await
        .unwrap();
    assert_eq!(*result, ["new"]);
    finish_tx.send(()).unwrap();
    assert_eq!(*pending.await.unwrap(), ["old"]);
    assert_eq!(*cache.novel_chapters.get("book").await.unwrap(), ["new"]);
}

#[tokio::test]
async fn failed_profile_fetch_after_login_still_replaces_old_cache() {
    let mut root =
        mock_client(|_, _| async { Err(AppError::transport("profile unavailable")) }).await;
    let old = login(&root, "old").await;
    old.cache()
        .store_cache(
            &old.cache().novel_chapters,
            "book".into(),
            vec!["old".into()],
        )
        .await;
    let (request, reply) = delayed_http(&mut root).await;
    let scoped = root.scoped().await;
    let pending =
        tokio::spawn(async move { scoped.login("test@example.com".into(), "test".into()).await });
    request.await.unwrap();
    reply.send(json!({"Success": true, "Response": {"Token": "new-access", "RefreshToken": "new-refresh"}})).unwrap();
    assert!(matches!(
        pending.await.unwrap(),
        Err(AppError::Transport { .. })
    ));
    let current = root.scoped().await;
    assert_eq!(current.token().await.unwrap(), "new-access");
    assert!(current.cache().novel_chapters.get("book").await.is_none());
    assert!(matches!(old.token().await, Err(AppError::SessionChanged)));
}

#[tokio::test(start_paused = true)]
async fn unresponsive_hub_call_does_not_hold_up_another_call_or_logout() {
    let client = mock_client(|method, _| async move {
        if method == "Slow" {
            std::future::pending::<Result<Value>>().await
        } else {
            Ok(json!("ready"))
        }
    })
    .await;
    let slow = client.hub_once("Slow", json!({}));
    tokio::pin!(slow);
    tokio::select! {
        biased;
        _ = &mut slow => panic!("slow call unexpectedly returned"),
        _ = std::future::ready(()) => {}
    }
    assert_eq!(
        client.hub_once("Fast", json!({})).await.unwrap(),
        json!("ready")
    );
    client.logout().await.unwrap();
    assert!(matches!(slow.await, Err(AppError::SessionChanged)));
}
