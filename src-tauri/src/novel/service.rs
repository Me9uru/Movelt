use std::sync::Arc;

use tokio::sync::Mutex;

use super::provider::wenku8_api::Wenku8ApiSource;
use super::{
    cache::{ChapterCache, ChapterKey, ChapterRequest},
    domain::{ChapterContent, DiscoveryList, HealthStatus, NovelOverview, RecommendBlock},
    error::NovelError,
    provider::NovelSource,
};

pub(crate) struct NovelService {
    providers: Vec<Arc<dyn NovelSource>>,
    discovery_provider: Arc<Wenku8ApiSource>,
    chapter_cache: Arc<Mutex<ChapterCache>>,
}

impl NovelService {
    pub(crate) fn new(base_url: &str) -> Result<Self, NovelError> {
        let discovery_provider = Arc::new(Wenku8ApiSource::new(base_url)?);
        let providers: Vec<Arc<dyn NovelSource>> = vec![discovery_provider.clone()];
        Ok(Self {
            providers,
            discovery_provider,
            chapter_cache: Arc::new(Mutex::new(ChapterCache::default())),
        })
    }

    fn provider(&self, source: &str) -> Result<Arc<dyn NovelSource>, NovelError> {
        self.providers
            .iter()
            .find(|provider| provider.id() == source)
            .cloned()
            .ok_or_else(|| NovelError::unknown_source(source))
    }

    pub(super) async fn health(&self) -> Result<HealthStatus, NovelError> {
        self.discovery_provider.health().await
    }

    pub(super) async fn recommend(&self) -> Result<Vec<RecommendBlock>, NovelError> {
        self.discovery_provider.recommend().await
    }

    pub(super) async fn ranking(&self, sort: &str, page: u32) -> Result<DiscoveryList, NovelError> {
        self.discovery_provider.ranking(sort, page).await
    }

    pub(super) async fn category(
        &self,
        tag: &str,
        sort: &str,
        page: u32,
    ) -> Result<DiscoveryList, NovelError> {
        self.discovery_provider.category(tag, sort, page).await
    }

    pub(super) async fn discovery_search(
        &self,
        query: &str,
        page: u32,
    ) -> Result<DiscoveryList, NovelError> {
        self.discovery_provider
            .search_mode(query, "articlename", page)
            .await
    }

    pub(super) async fn overview(
        &self,
        source: &str,
        novel_id: &str,
    ) -> Result<NovelOverview, NovelError> {
        self.provider(source)?.overview(novel_id).await
    }

    pub(super) async fn cover_data_url(
        &self,
        source: &str,
        novel_id: &str,
    ) -> Result<String, NovelError> {
        self.provider(source)?;
        self.discovery_provider.cover_data_url(novel_id).await
    }

    pub(super) async fn chapter(
        &self,
        source: &str,
        novel_id: &str,
        chapter_id: &str,
        chapter_title: Option<&str>,
    ) -> Result<ChapterContent, NovelError> {
        let provider = self.provider(source)?;
        let key = ChapterKey {
            source: source.into(),
            novel_id: novel_id.into(),
            chapter_id: chapter_id.into(),
        };

        let mut receiver = match self.chapter_cache.lock().await.begin(&key) {
            ChapterRequest::Cached(document) => return Ok(document),
            ChapterRequest::Pending(receiver) => receiver,
            ChapterRequest::Start { receiver, sender } => {
                let cache = Arc::clone(&self.chapter_cache);
                let fetch_key = key.clone();
                let chapter_title = chapter_title.map(str::to_owned);
                tauri::async_runtime::spawn(async move {
                    let result = provider
                        .chapter_with_title(
                            &fetch_key.novel_id,
                            &fetch_key.chapter_id,
                            chapter_title.as_deref(),
                        )
                        .await;
                    cache.lock().await.complete(fetch_key, &result);
                    let _ = sender.send(Some(result));
                });
                receiver
            }
        };

        if receiver.borrow().is_none() {
            receiver.changed().await.map_err(|_| NovelError::Internal)?;
        }
        let result = receiver.borrow().clone().ok_or(NovelError::Internal)?;
        result
    }

    pub(super) async fn prefetch(
        &self,
        source: &str,
        novel_id: &str,
        chapter_ids: &[String],
        chapter_titles: &[String],
    ) -> Result<(), NovelError> {
        self.provider(source)?;
        let results =
            futures::future::join_all(chapter_ids.iter().enumerate().map(|(index, chapter_id)| {
                self.chapter(
                    source,
                    novel_id,
                    chapter_id,
                    chapter_titles.get(index).map(String::as_str),
                )
            }))
            .await;
        if let Some(error) = results.into_iter().find_map(Result::err) {
            return Err(error);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_source() {
        let service = NovelService::new("http://127.0.0.1:8000").unwrap();
        assert!(matches!(
            service.provider("missing"),
            Err(NovelError::UnknownSource(source)) if source == "missing"
        ));
    }

    #[test]
    fn registers_wenku8_api() {
        let service = NovelService::new("http://127.0.0.1:8000").unwrap();
        assert!(service.provider("wenku8_api").is_ok());
    }
}
