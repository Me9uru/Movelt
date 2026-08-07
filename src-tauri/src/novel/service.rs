use std::sync::Arc;

use tokio::sync::Mutex;

use super::{
    cache::{ChapterCache, ChapterKey, ChapterRequest},
    domain::{ChapterContent, NovelOverview, NovelSourceInfo, SearchResult},
    error::NovelError,
    provider::{self, NovelSource},
};

pub(crate) struct NovelService {
    providers: Vec<Arc<dyn NovelSource>>,
    chapter_cache: Arc<Mutex<ChapterCache>>,
}

impl NovelService {
    pub(crate) fn new(local_epub_root: std::path::PathBuf) -> Result<Self, NovelError> {
        Ok(Self {
            providers: provider::built_in(local_epub_root)?,
            chapter_cache: Arc::new(Mutex::new(ChapterCache::default())),
        })
    }

    pub(super) fn sources(&self) -> Vec<NovelSourceInfo> {
        self.providers
            .iter()
            .map(|provider| NovelSourceInfo {
                id: provider.id().into(),
                name: provider.name().into(),
            })
            .collect()
    }

    fn provider(&self, source: &str) -> Result<Arc<dyn NovelSource>, NovelError> {
        self.providers
            .iter()
            .find(|provider| provider.id() == source)
            .cloned()
            .ok_or_else(|| NovelError::unknown_source(source))
    }

    pub(super) async fn search(
        &self,
        source: &str,
        query: &str,
        page: u32,
    ) -> Result<SearchResult, NovelError> {
        self.provider(source)?.search(query, page).await
    }

    pub(super) async fn overview(
        &self,
        source: &str,
        novel_id: &str,
    ) -> Result<NovelOverview, NovelError> {
        self.provider(source)?.overview(novel_id).await
    }

    pub(super) async fn chapter(
        &self,
        source: &str,
        novel_id: &str,
        chapter_id: &str,
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
                tauri::async_runtime::spawn(async move {
                    let result = provider
                        .chapter(&fetch_key.novel_id, &fetch_key.chapter_id)
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
    ) -> Result<(), NovelError> {
        self.provider(source)?;
        let results = futures::future::join_all(
            chapter_ids
                .iter()
                .map(|chapter_id| self.chapter(source, novel_id, chapter_id)),
        )
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
        let service = NovelService::new(std::env::temp_dir().join("movel-test-epub")).unwrap();
        assert!(matches!(
            service.provider("missing"),
            Err(NovelError::UnknownSource(source)) if source == "missing"
        ));
    }
}
