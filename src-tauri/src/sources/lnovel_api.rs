use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::{SinkExt, StreamExt};
use reqwest::Client;
use rmpv::Value;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::error::NovelError;

pub(crate) const PAGE_BATCH_SIZE: usize = 12;

#[derive(Clone)]
pub(crate) struct LnovelApiSource {
    http: Client,
    base_url: Url,
    hub: Arc<Mutex<Option<HubConnection>>>,
    page_cache: Arc<Mutex<HashMap<String, PageCache>>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaSummary {
    pub id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub author: Option<String>,
    pub unread_count: i32,
    pub source_name: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaDetail {
    pub id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub author: Option<String>,
    pub artist: Option<String>,
    pub description: Option<String>,
    pub genre: Vec<String>,
    pub status: String,
    pub source_name: Option<String>,
    pub chapters: Vec<MangaChapter>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaChapter {
    pub id: String,
    pub name: String,
    pub chapter_number: f32,
    pub is_read: bool,
    pub last_page_read: i32,
    pub page_count: i32,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaPageList {
    pub chapter_id: String,
    pub page_count: usize,
    pub first_page_urls: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaPageBatch {
    pub start_index: usize,
    pub page_urls: Vec<String>,
}

impl LnovelApiSource {
    pub(crate) fn new(base_url: &str) -> Result<Self, NovelError> {
        let base_url = parse_base_url(base_url)?;
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(8))
            .timeout(Duration::from_secs(45))
            .build()
            .map_err(|error| NovelError::configuration(error.to_string()))?;
        Ok(Self {
            http,
            base_url,
            hub: Arc::default(),
            page_cache: Arc::default(),
        })
    }

    pub(crate) async fn browse(
        &self,
        query: Option<&str>,
        page: i32,
        browse_type: &str,
    ) -> Result<Vec<MangaSummary>, NovelError> {
        validate_page(page)?;
        let response: ComicList = match browse_type {
            "SEARCH" => {
                let keywords = query
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| NovelError::invalid_input("search query must not be empty"))?;
                self.invoke(
                    "SearchComicSeries",
                    json!({ "KeyWords": keywords, "Page": page, "Size": 30 }),
                )
                .await?
            }
            "POPULAR" => {
                self.invoke(
                    "GetComicList",
                    json!({ "Page": page, "Size": 30, "Order": "view" }),
                )
                .await?
            }
            "LATEST" => {
                self.invoke(
                    "GetComicList",
                    json!({ "Page": page, "Size": 30, "Order": "latest" }),
                )
                .await?
            }
            _ => return Err(NovelError::invalid_input("unsupported manga browse type")),
        };
        Ok(response
            .data
            .into_iter()
            .map(ComicListItem::summary)
            .collect())
    }

    pub(crate) async fn manga(&self, manga_id: &str) -> Result<MangaDetail, NovelError> {
        let id = parse_id(manga_id, "manga")?;
        let response: ComicInfo = self.invoke("GetComicInfo", json!({ "Id": id })).await?;
        Ok(response.book.detail())
    }

    pub(crate) async fn chapter_pages(
        &self,
        manga_id: &str,
        chapter_id: &str,
    ) -> Result<MangaPageList, NovelError> {
        let _ = parse_id(manga_id, "manga")?;
        let chapter_id_number = parse_id(chapter_id, "chapter")?;
        let response: ComicContent = self
            .invoke(
                "GetComicContent",
                json!({ "Cid": chapter_id_number, "Skip": 0, "Take": PAGE_BATCH_SIZE }),
            )
            .await?;
        let first_page_urls = response
            .chapter
            .images
            .into_iter()
            .filter_map(|image| (!image.url.trim().is_empty()).then_some(image.url))
            .collect::<Vec<_>>();
        if first_page_urls.is_empty() {
            return Err(NovelError::Upstream(
                "lnovelApi returned no chapter images".into(),
            ));
        }
        let page_count = response.chapter.total.max(first_page_urls.len());
        self.page_cache.lock().await.insert(
            chapter_id.into(),
            PageCache::new(page_count, &first_page_urls),
        );
        Ok(MangaPageList {
            chapter_id: chapter_id.into(),
            page_count,
            first_page_urls,
        })
    }

    pub(crate) async fn page_batch(
        &self,
        chapter_id: &str,
        page_index: usize,
    ) -> Result<MangaPageBatch, NovelError> {
        let start_index = page_index / PAGE_BATCH_SIZE * PAGE_BATCH_SIZE;
        if let Some(batch) = self
            .page_cache
            .lock()
            .await
            .get(chapter_id)
            .and_then(|cache| cache.batch(start_index))
        {
            return Ok(batch);
        }
        let chapter_id_number = parse_id(chapter_id, "chapter")?;
        let response: ComicContent = self
            .invoke(
                "GetComicContent",
                json!({ "Cid": chapter_id_number, "Skip": start_index, "Take": PAGE_BATCH_SIZE }),
            )
            .await?;
        let page_urls = response
            .chapter
            .images
            .into_iter()
            .filter_map(|image| (!image.url.trim().is_empty()).then_some(image.url))
            .collect::<Vec<_>>();
        if page_urls.is_empty() {
            return Err(NovelError::Upstream(
                "lnovelApi returned no chapter images for the requested page batch".into(),
            ));
        }
        let mut cache = self.page_cache.lock().await;
        let entry = cache.entry(chapter_id.into()).or_insert_with(|| {
            PageCache::new(
                response.chapter.total.max(start_index + page_urls.len()),
                &[],
            )
        });
        entry.insert(start_index, &page_urls);
        Ok(MangaPageBatch {
            start_index,
            page_urls,
        })
    }

    async fn invoke<T: DeserializeOwned>(
        &self,
        target: &str,
        argument: serde_json::Value,
    ) -> Result<T, NovelError> {
        let mut hub = self.hub.lock().await;
        if hub.is_none() {
            *hub = Some(self.connect_hub().await?);
        }
        let result = hub
            .as_mut()
            .expect("SignalR connection is initialized")
            .invoke(target, argument)
            .await;
        if result.is_err() {
            *hub = None;
        }
        result
    }

    async fn connect_hub(&self) -> Result<HubConnection, NovelError> {
        let token: NegotiateResponse = self
            .http
            .post(self.url("hub/api/negotiate?negotiateVersion=1")?)
            .send()
            .await
            .map_err(upstream_error)?
            .error_for_status()
            .map_err(upstream_error)?
            .json()
            .await
            .map_err(|error| NovelError::Parse(error.to_string()))?;
        let mut ws_url = self.url("hub/api")?;
        ws_url
            .set_scheme(if ws_url.scheme() == "https" {
                "wss"
            } else {
                "ws"
            })
            .map_err(|_| NovelError::configuration("invalid lnovelApi WebSocket scheme"))?;
        ws_url
            .query_pairs_mut()
            .append_pair("id", &token.connection_token);
        let (mut socket, _) = connect_async(ws_url.as_str())
            .await
            .map_err(upstream_error)?;
        socket
            .send(Message::Text(
                "{\"protocol\":\"messagepack\",\"version\":1}\u{1e}".into(),
            ))
            .await
            .map_err(upstream_error)?;
        match socket.next().await {
            Some(Ok(Message::Text(handshake))) if handshake.contains("{}") => {}
            Some(Ok(message)) => {
                return Err(NovelError::Parse(format!(
                    "unexpected SignalR handshake response: {message:?}"
                )))
            }
            Some(Err(error)) => return Err(upstream_error(error)),
            None => {
                return Err(NovelError::Upstream(
                    "lnovelApi closed during SignalR handshake".into(),
                ))
            }
        }
        Ok(HubConnection { socket, next_id: 1 })
    }

    fn url(&self, path: &str) -> Result<Url, NovelError> {
        self.base_url
            .join(path)
            .map_err(|error| NovelError::invalid_input(error.to_string()))
    }
}

struct HubConnection {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl HubConnection {
    async fn invoke<T: DeserializeOwned>(
        &mut self,
        target: &str,
        argument: serde_json::Value,
    ) -> Result<T, NovelError> {
        let invocation_id = self.next_id.to_string();
        self.next_id += 1;
        let payload = rmp_serde::to_vec(&(
            1_u8,
            HashMap::<String, String>::new(),
            &invocation_id,
            target,
            vec![argument, json!({ "UseGzip": false })],
            Vec::<String>::new(),
        ))
        .map_err(|error| NovelError::Parse(error.to_string()))?;
        self.socket
            .send(Message::Binary(frame(payload)))
            .await
            .map_err(upstream_error)?;
        while let Some(message) = self.socket.next().await {
            let bytes = match message.map_err(upstream_error)? {
                Message::Binary(bytes) => bytes,
                Message::Close(_) => {
                    return Err(NovelError::Upstream(
                        "lnovelApi closed the SignalR connection".into(),
                    ))
                }
                _ => continue,
            };
            let value = decode_frame(&bytes)?;
            let array = value
                .as_array()
                .ok_or_else(|| NovelError::Parse("invalid SignalR response".into()))?;
            if array.first().and_then(Value::as_i64) != Some(3)
                || array.get(2).and_then(Value::as_str) != Some(invocation_id.as_str())
            {
                continue;
            }
            match array.get(3).and_then(Value::as_i64) {
                Some(1) => {
                    return Err(NovelError::Upstream(
                        array
                            .get(4)
                            .and_then(Value::as_str)
                            .unwrap_or("lnovelApi request failed")
                            .into(),
                    ))
                }
                Some(2) => {
                    return Err(NovelError::Parse(
                        "lnovelApi returned an empty response".into(),
                    ))
                }
                Some(3) => {
                    let response = array.get(4).ok_or_else(|| {
                        NovelError::Parse("lnovelApi response is missing data".into())
                    })?;
                    let envelope: UpstreamResult = rmpv::ext::from_value(response.clone())
                        .map_err(|error| NovelError::Parse(error.to_string()))?;
                    if !envelope.success {
                        return Err(NovelError::Upstream(envelope.msg));
                    }
                    return rmpv::ext::from_value(envelope.response)
                        .map_err(|error| NovelError::Parse(error.to_string()));
                }
                _ => {
                    return Err(NovelError::Parse(
                        "unknown SignalR completion response".into(),
                    ))
                }
            }
        }
        Err(NovelError::Upstream(
            "lnovelApi closed before returning a response".into(),
        ))
    }
}

struct PageCache {
    page_count: usize,
    urls: Vec<Option<String>>,
}
impl PageCache {
    fn new(page_count: usize, first_page_urls: &[String]) -> Self {
        let mut cache = Self {
            page_count,
            urls: vec![None; page_count],
        };
        cache.insert(0, first_page_urls);
        cache
    }
    fn insert(&mut self, start_index: usize, page_urls: &[String]) {
        if self.urls.len() < self.page_count {
            self.urls.resize(self.page_count, None);
        }
        for (index, url) in page_urls.iter().enumerate() {
            if let Some(slot) = self.urls.get_mut(start_index + index) {
                *slot = Some(url.clone());
            }
        }
    }
    fn batch(&self, start_index: usize) -> Option<MangaPageBatch> {
        if start_index >= self.page_count {
            return None;
        }
        let end = (start_index + PAGE_BATCH_SIZE).min(self.page_count);
        let page_urls = self
            .urls
            .get(start_index..end)?
            .iter()
            .cloned()
            .collect::<Option<Vec<_>>>()?;
        Some(MangaPageBatch {
            start_index,
            page_urls,
        })
    }
}

fn parse_base_url(value: &str) -> Result<Url, NovelError> {
    let url = Url::parse(&format!("{}/", value.trim_end_matches('/')))
        .map_err(|error| NovelError::configuration(error.to_string()))?;
    if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() {
        Ok(url)
    } else {
        Err(NovelError::configuration(
            "lnovelApi URL must be an absolute http(s) URL",
        ))
    }
}
fn validate_page(page: i32) -> Result<(), NovelError> {
    if (1..=1000).contains(&page) {
        Ok(())
    } else {
        Err(NovelError::invalid_input(
            "manga page must be between 1 and 1000",
        ))
    }
}
fn parse_id(value: &str, label: &str) -> Result<i64, NovelError> {
    value
        .parse::<i64>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| NovelError::invalid_input(format!("invalid manga {label} id")))
}
fn upstream_error(error: impl std::fmt::Display) -> NovelError {
    NovelError::Upstream(format!("lnovelApi: {error}"))
}
fn frame(mut payload: Vec<u8>) -> Vec<u8> {
    let mut prefix = Vec::new();
    let mut length = payload.len();
    loop {
        let mut value = (length & 0x7f) as u8;
        length >>= 7;
        if length != 0 {
            value |= 0x80;
        }
        prefix.push(value);
        if length == 0 {
            break;
        }
    }
    prefix.append(&mut payload);
    prefix
}
fn decode_frame(bytes: &[u8]) -> Result<Value, NovelError> {
    let mut offset = 0;
    let mut length = 0_usize;
    let mut shift = 0;
    loop {
        let byte = *bytes
            .get(offset)
            .ok_or_else(|| NovelError::Parse("truncated SignalR frame".into()))?;
        offset += 1;
        length |= usize::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 28 {
            return Err(NovelError::Parse("invalid SignalR frame length".into()));
        }
    }
    let payload = bytes
        .get(offset..offset + length)
        .ok_or_else(|| NovelError::Parse("truncated SignalR payload".into()))?;
    rmpv::decode::read_value(&mut std::io::Cursor::new(payload))
        .map_err(|error| NovelError::Parse(error.to_string()))
}

#[derive(Deserialize)]
struct NegotiateResponse {
    #[serde(rename = "connectionToken")]
    connection_token: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UpstreamResult {
    success: bool,
    msg: String,
    response: Value,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ComicList {
    data: Vec<ComicListItem>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ComicListItem {
    id: i64,
    title: String,
    #[serde(default)]
    cover: String,
}
impl ComicListItem {
    fn summary(self) -> MangaSummary {
        MangaSummary {
            id: self.id.to_string(),
            title: self.title,
            thumbnail_url: (!self.cover.is_empty()).then_some(self.cover),
            author: None,
            // lnovelApi's `Count` is a series/book count, not a local unread counter.
            unread_count: 0,
            source_name: Some("lnovelApi".into()),
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ComicInfo {
    book: ComicBook,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ComicBook {
    id: i64,
    title: String,
    #[serde(default)]
    cover: String,
    author: Option<String>,
    introduction: Option<String>,
    #[serde(default)]
    last_updated_chapter: String,
    #[serde(default)]
    extra: serde_json::Value,
    #[serde(default)]
    chapters: Vec<ComicChapter>,
}
impl ComicBook {
    fn detail(self) -> MangaDetail {
        let genre = self
            .extra
            .get("classification")
            .and_then(|value| value.get("tags"))
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|tag| tag.as_str().map(str::to_owned))
            .collect();
        MangaDetail {
            id: self.id.to_string(),
            title: self.title,
            thumbnail_url: (!self.cover.is_empty()).then_some(self.cover),
            author: self.author,
            artist: None,
            description: self.introduction.filter(|text| !text.is_empty()),
            genre,
            status: self.last_updated_chapter,
            source_name: Some("lnovelApi".into()),
            chapters: self
                .chapters
                .into_iter()
                .map(ComicChapter::into_manga)
                .collect(),
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ComicChapter {
    id: i64,
    sort_num: f32,
    title: String,
    #[serde(default)]
    page_count: i32,
}
impl ComicChapter {
    fn into_manga(self) -> MangaChapter {
        MangaChapter {
            id: self.id.to_string(),
            name: self.title,
            chapter_number: self.sort_num,
            is_read: false,
            last_page_read: 0,
            page_count: self.page_count,
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ComicContent {
    chapter: ContentChapter,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContentChapter {
    total: usize,
    images: Vec<ComicImage>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ComicImage {
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_messagepack_payload() {
        assert_eq!(frame(vec![1, 2]), vec![2, 1, 2]);
    }
    #[test]
    fn rejects_non_numeric_ids() {
        assert!(parse_id("abc", "chapter").is_err());
    }
    #[test]
    fn maps_list_items() {
        assert_eq!(
            ComicListItem {
                id: 7,
                title: "x".into(),
                cover: String::new(),
            }
            .summary()
            .id,
            "7"
        );
    }

    #[test]
    #[ignore = "requires a running lnovelApi instance on localhost:8000"]
    fn fetches_a_manga_list_from_lnovel_api() {
        let source = LnovelApiSource::new("http://127.0.0.1:8000").unwrap();
        let manga = tauri::async_runtime::block_on(source.browse(None, 1, "LATEST")).unwrap();
        assert!(!manga.is_empty());
    }
}
