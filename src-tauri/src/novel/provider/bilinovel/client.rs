use crate::novel::error::NovelError;
use reqwest::{header, Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{RwLock, Semaphore};
use url::Url;

const DEFAULT_API_BASE_URL: &str = "https://lnovel.animes.garden/";
const USER_AGENT: &str = "NovelTauri/0.1 (+personal, rate-limited reader)";
const INDEX_TTL: Duration = Duration::from_secs(600);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_CONCURRENCY: usize = 4;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiNovel {
    pub nid: u64,
    pub name: String,
    #[serde(default)]
    pub authors: Vec<ApiAuthor>,
    pub description: Option<String>,
    pub cover: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub volumes: Vec<ApiVolumeSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiAuthor {
    pub name: String,
    pub position: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiVolumeSummary {
    pub vid: u64,
}

#[derive(Debug, Deserialize)]
pub struct ApiVolume {
    pub name: String,
    #[serde(default)]
    pub chapters: Vec<ApiChapterSummary>,
}

#[derive(Debug, Deserialize)]
pub struct ApiChapterSummary {
    pub cid: u64,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiChapter {
    pub nid: u64,
    pub cid: u64,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope<T> {
    ok: bool,
    data: Option<T>,
}

struct CachedIndex {
    fetched_at: Instant,
    novels: Arc<Vec<ApiNovel>>,
}

pub struct BilinovelClient {
    base_url: Url,
    http: Client,
    limiter: Arc<Semaphore>,
    index: RwLock<Option<CachedIndex>>,
}

impl BilinovelClient {
    pub fn new() -> Result<Self, NovelError> {
        let base_url = env::var("BILINOVEL_API_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_API_BASE_URL.into())
            .parse()
            .map_err(|error: url::ParseError| NovelError::Upstream(error.to_string()))?;
        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()
            .map_err(|error| NovelError::Upstream(error.to_string()))?;

        Ok(Self {
            base_url,
            http,
            limiter: Arc::new(Semaphore::new(MAX_CONCURRENCY)),
            index: RwLock::new(None),
        })
    }

    pub async fn novels(&self) -> Result<Arc<Vec<ApiNovel>>, NovelError> {
        {
            let cached = self.index.read().await;
            if let Some(index) = cached.as_ref() {
                if index.fetched_at.elapsed() < INDEX_TTL {
                    return Ok(Arc::clone(&index.novels));
                }
            }
        }

        let novels: Vec<ApiNovel> = self.get("bili/novels").await?;
        let novels = Arc::new(novels);
        *self.index.write().await = Some(CachedIndex {
            fetched_at: Instant::now(),
            novels: Arc::clone(&novels),
        });
        Ok(novels)
    }

    pub async fn novel(&self, novel_id: &str) -> Result<ApiNovel, NovelError> {
        self.get(&format!("bili/novel/{novel_id}")).await
    }

    pub async fn volume(&self, novel_id: &str, volume_id: u64) -> Result<ApiVolume, NovelError> {
        self.get(&format!("bili/novel/{novel_id}/vol/{volume_id}"))
            .await
    }

    pub async fn chapter(
        &self,
        novel_id: &str,
        chapter_id: &str,
    ) -> Result<ApiChapter, NovelError> {
        self.get(&format!("bili/novel/{novel_id}/chapter/{chapter_id}"))
            .await
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, NovelError> {
        let url = self.base_url.join(path).map_err(|_| NovelError::Internal)?;
        let _permit = self
            .limiter
            .acquire()
            .await
            .map_err(|_| NovelError::Internal)?;
        let response = self
            .http
            .get(url)
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| NovelError::Upstream(error.to_string()))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(NovelError::NotFound);
        }
        if !response.status().is_success() {
            return Err(NovelError::Upstream(format!(
                "upstream returned HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|error| NovelError::Upstream(error.to_string()))?;
        let envelope: ApiEnvelope<T> =
            serde_json::from_slice(&bytes).map_err(|error| NovelError::Parse(error.to_string()))?;
        if !envelope.ok {
            return Err(NovelError::Upstream("upstream reported failure".into()));
        }
        envelope
            .data
            .ok_or_else(|| NovelError::Parse("upstream response omitted data".into()))
    }
}
