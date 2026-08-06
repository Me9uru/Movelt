use super::parser;
use crate::novel::error::NovelError;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, RequestBuilder, Response, StatusCode,
};
use std::{env, sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use url::Url;

const DEFAULT_BASE_URL: &str = "http://m.5859ycdh.com/";
const USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_CONCURRENCY: usize = 4;
const MAX_HTML_BYTES: usize = 5 * 1024 * 1024;
const MAX_CHAPTER_PAGES: u32 = 50;

pub(super) struct WubaClient {
    base_url: Url,
    http: Client,
    limiter: Arc<Semaphore>,
}

impl WubaClient {
    pub(super) fn new() -> Result<Self, NovelError> {
        let base_url = env::var("WUBA_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.into())
            .parse::<Url>()
            .map_err(|_| NovelError::Internal)?;
        validate_base_url(&base_url)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static("zh-CN,zh;q=0.9"),
        );
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("text/html,application/xhtml+xml"),
        );
        let redirect_base = base_url.clone();
        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                let url = attempt.url();
                let same_origin = url.scheme() == redirect_base.scheme()
                    && url.host_str() == redirect_base.host_str()
                    && url.port_or_known_default() == redirect_base.port_or_known_default();
                if attempt.previous().len() >= 3 || !same_origin {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            }))
            .build()
            .map_err(|_| NovelError::Internal)?;

        Ok(Self {
            base_url,
            http,
            limiter: Arc::new(Semaphore::new(MAX_CONCURRENCY)),
        })
    }

    pub(super) async fn search(&self, query: &str) -> Result<String, NovelError> {
        let url = self.join("search/")?;
        let request = self.http.post(url.clone()).form(&[("q", query)]);
        self.fetch_html(url, request).await
    }

    pub(super) async fn detail(&self, novel_id: &str) -> Result<String, NovelError> {
        let url = self.join(&format!("wuba/{novel_id}/"))?;
        let request = self.http.get(url.clone());
        self.fetch_html(url, request).await
    }

    pub(super) async fn chapter_pages(
        &self,
        novel_id: &str,
        chapter_id: &str,
    ) -> Result<Vec<String>, NovelError> {
        let first = self.chapter_page(novel_id, chapter_id, 1).await?;
        let page_count = parser::chapter_page_count(&first, chapter_id)?;
        if page_count > MAX_CHAPTER_PAGES {
            return Err(NovelError::Parse("chapter has too many pages".into()));
        }

        let mut pages = Vec::with_capacity(page_count as usize);
        pages.push(first);
        for page in 2..=page_count {
            pages.push(self.chapter_page(novel_id, chapter_id, page).await?);
        }
        Ok(pages)
    }

    async fn chapter_page(
        &self,
        novel_id: &str,
        chapter_id: &str,
        page: u32,
    ) -> Result<String, NovelError> {
        let filename = if page == 1 {
            format!("{chapter_id}.html")
        } else {
            format!("{chapter_id}-{page}.html")
        };
        let url = self.join(&format!("wubashu/{novel_id}/{filename}"))?;
        let request = self.http.get(url.clone());
        self.fetch_html(url, request).await
    }

    pub(super) fn base_url(&self) -> &Url {
        &self.base_url
    }

    fn join(&self, path: &str) -> Result<Url, NovelError> {
        let url = self.base_url.join(path).map_err(|_| NovelError::Internal)?;
        self.validate_url(&url)?;
        Ok(url)
    }

    async fn fetch_html(
        &self,
        expected_url: Url,
        request: RequestBuilder,
    ) -> Result<String, NovelError> {
        self.validate_url(&expected_url)?;
        let _permit = self
            .limiter
            .acquire()
            .await
            .map_err(|_| NovelError::Internal)?;
        let mut response = send_with_retry(request).await?;
        self.validate_url(response.url())?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(NovelError::NotFound);
        }
        if !response.status().is_success() {
            return Err(NovelError::Upstream(format!(
                "upstream returned HTTP {}",
                response.status()
            )));
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_HTML_BYTES as u64)
        {
            return Err(NovelError::Parse("upstream page is too large".into()));
        }

        read_limited_utf8(&mut response).await
    }

    fn validate_url(&self, url: &Url) -> Result<(), NovelError> {
        if url.scheme() != self.base_url.scheme()
            || url.host_str() != self.base_url.host_str()
            || url.port_or_known_default() != self.base_url.port_or_known_default()
        {
            return Err(NovelError::Upstream(
                "upstream redirected outside the configured source".into(),
            ));
        }
        Ok(())
    }
}

async fn send_with_retry(request: RequestBuilder) -> Result<Response, NovelError> {
    let retry = request.try_clone();
    match request.send().await {
        Ok(response) => Ok(response),
        Err(error) if error.is_connect() || error.is_timeout() => {
            let retry = retry.ok_or(NovelError::Internal)?;
            retry
                .send()
                .await
                .map_err(|error| NovelError::Upstream(error.to_string()))
        }
        Err(error) => Err(NovelError::Upstream(error.to_string())),
    }
}

async fn read_limited_utf8(response: &mut Response) -> Result<String, NovelError> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| NovelError::Upstream(error.to_string()))?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_HTML_BYTES {
            return Err(NovelError::Parse("upstream page is too large".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes)
        .map_err(|error| NovelError::Parse(format!("page is not UTF-8: {error}")))
}

fn validate_base_url(url: &Url) -> Result<(), NovelError> {
    if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() {
        Ok(())
    } else {
        Err(NovelError::Internal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires live Wuba access"]
    fn searches_live_site() {
        let client = WubaClient::new().unwrap();
        let html = tauri::async_runtime::block_on(client.search("蛊真人")).unwrap();
        let result = parser::search(&html, client.base_url()).unwrap();
        assert!(result.items.iter().any(|item| item.title == "蛊真人"));

        let detail_html = tauri::async_runtime::block_on(client.detail("50919")).unwrap();
        let overview = parser::overview(&detail_html, "50919", client.base_url()).unwrap();
        assert_eq!(overview.detail.title, "蛊真人");
        assert!(overview.volumes[0].chapters.len() > 2_000);

        let pages =
            tauri::async_runtime::block_on(client.chapter_pages("50919", "37262160")).unwrap();
        assert!(pages.len() > 1);
        let chapter = parser::chapter(&pages, "50919", "37262160").unwrap();
        assert_eq!(chapter.title, "第一节：纵身亡魔心仍不悔");
        assert!(chapter.nodes.len() > 10);
    }
}
