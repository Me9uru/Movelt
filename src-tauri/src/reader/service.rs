use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use super::cache::{ChapterCache, ChapterKey, ChapterRequest};
use crate::domain::{ChapterContent, ChapterSummary, NovelOverview, Volume};
use crate::error::NovelError;
use crate::library::local_epub::LocalEpubSource;
use crate::sources::wenku8_api::Wenku8ApiSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SourceId {
    Wenku8Api,
    LocalEpub,
}

impl SourceId {
    pub(super) fn parse(source: &str) -> Result<Self, NovelError> {
        match source.trim() {
            crate::sources::wenku8_api::SOURCE_ID => Ok(Self::Wenku8Api),
            LocalEpubSource::SOURCE_ID => Ok(Self::LocalEpub),
            source => Err(NovelError::invalid_input(format!(
                "unsupported reader source: {source}"
            ))),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ReaderService {
    wenku8_api: Arc<Wenku8ApiSource>,
    local_epub: Arc<LocalEpubSource>,
    chapter_cache: Arc<Mutex<ChapterCache>>,
    chapter_catalogues: Arc<Mutex<HashMap<String, Vec<ChapterSummary>>>>,
}

impl ReaderService {
    pub(crate) fn new(wenku8_api: Arc<Wenku8ApiSource>, local_epub: Arc<LocalEpubSource>) -> Self {
        Self {
            wenku8_api,
            local_epub,
            chapter_cache: Arc::new(Mutex::new(ChapterCache::default())),
            chapter_catalogues: Arc::default(),
        }
    }

    pub(super) async fn overview(
        &self,
        source: SourceId,
        novel_id: &str,
    ) -> Result<NovelOverview, NovelError> {
        match source {
            SourceId::Wenku8Api => {
                let overview = self.wenku8_api.overview(novel_id).await?;
                self.cache_chapter_catalogue(novel_id, &overview).await;
                Ok(overview)
            }
            SourceId::LocalEpub => self.local_epub.overview(novel_id),
        }
    }

    pub(super) async fn cover_data_url(
        &self,
        source: SourceId,
        novel_id: &str,
    ) -> Result<String, NovelError> {
        match source {
            SourceId::Wenku8Api => self.wenku8_api.cover_data_url(novel_id).await,
            SourceId::LocalEpub => {
                let overview = self.local_epub.overview(novel_id)?;
                let cover_path = overview.detail.cover_url.ok_or(NovelError::NotFound)?;
                self.local_epub.asset_data_url(novel_id, &cover_path)
            }
        }
    }

    pub(super) async fn chapter(
        &self,
        source: SourceId,
        novel_id: &str,
        chapter_id: &str,
        chapter_title: Option<&str>,
    ) -> Result<ChapterContent, NovelError> {
        let chapter = self
            .load_chapter(source, novel_id, chapter_id, chapter_title)
            .await?;
        self.schedule_prefetch(source, novel_id, chapter_id);
        Ok(chapter)
    }

    async fn load_chapter(
        &self,
        source: SourceId,
        novel_id: &str,
        chapter_id: &str,
        chapter_title: Option<&str>,
    ) -> Result<ChapterContent, NovelError> {
        if source == SourceId::LocalEpub {
            return self.local_epub.chapter(novel_id, chapter_id);
        }
        let wenku8_api = Arc::clone(&self.wenku8_api);
        let key = ChapterKey {
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
                    let result = wenku8_api
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

    fn schedule_prefetch(&self, source: SourceId, novel_id: &str, chapter_id: &str) {
        if source == SourceId::LocalEpub {
            return;
        }
        let service = self.clone();
        let novel_id = novel_id.to_owned();
        let chapter_id = chapter_id.to_owned();
        tauri::async_runtime::spawn(async move {
            let chapters = match service.following_chapters(&novel_id, &chapter_id).await {
                Ok(chapters) => chapters,
                Err(_) => return,
            };
            let chapter_ids = chapters
                .iter()
                .map(|chapter| chapter.id.clone())
                .collect::<Vec<_>>();
            let chapter_titles = chapters
                .iter()
                .map(|chapter| chapter.title.clone())
                .collect::<Vec<_>>();
            let _ = service
                .prefetch(source, &novel_id, &chapter_ids, &chapter_titles)
                .await;
        });
    }

    async fn following_chapters(
        &self,
        novel_id: &str,
        chapter_id: &str,
    ) -> Result<Vec<ChapterSummary>, NovelError> {
        let chapters = match self.chapter_catalogues.lock().await.get(novel_id).cloned() {
            Some(chapters) => chapters,
            None => {
                let overview = self.wenku8_api.overview(novel_id).await?;
                let chapters = flatten_chapters(&overview.volumes);
                self.chapter_catalogues
                    .lock()
                    .await
                    .insert(novel_id.to_owned(), chapters.clone());
                chapters
            }
        };
        let Some(index) = chapters.iter().position(|chapter| chapter.id == chapter_id) else {
            return Ok(Vec::new());
        };
        Ok(chapters.into_iter().skip(index + 1).take(2).collect())
    }

    async fn cache_chapter_catalogue(&self, novel_id: &str, overview: &NovelOverview) {
        self.chapter_catalogues
            .lock()
            .await
            .insert(novel_id.to_owned(), flatten_chapters(&overview.volumes));
    }

    async fn prefetch(
        &self,
        source: SourceId,
        novel_id: &str,
        chapter_ids: &[String],
        chapter_titles: &[String],
    ) -> Result<(), NovelError> {
        if source == SourceId::LocalEpub {
            return Ok(());
        }
        let results =
            futures::future::join_all(chapter_ids.iter().enumerate().map(|(index, chapter_id)| {
                self.load_chapter(
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

fn flatten_chapters(volumes: &[Volume]) -> Vec<ChapterSummary> {
    let mut chapters = Vec::new();
    for volume in volumes {
        chapters.extend(volume.chapters.iter().cloned());
        chapters.extend(flatten_chapters(&volume.sections));
    }
    chapters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_source_ids() {
        assert_eq!(SourceId::parse("wenku8_api").unwrap(), SourceId::Wenku8Api);
        assert_eq!(SourceId::parse("local_epub").unwrap(), SourceId::LocalEpub);
    }

    #[test]
    fn rejects_unsupported_source_ids() {
        assert!(matches!(
            SourceId::parse("missing"),
            Err(NovelError::InvalidInput(message)) if message == "unsupported reader source: missing"
        ));
    }

    #[test]
    fn flattens_chapters_in_catalogue_order() {
        let volumes = vec![Volume {
            title: "卷一".into(),
            chapters: vec![ChapterSummary {
                id: "1".into(),
                title: "第一章".into(),
            }],
            sections: vec![Volume {
                title: "子卷".into(),
                chapters: vec![ChapterSummary {
                    id: "2".into(),
                    title: "第二章".into(),
                }],
                sections: Vec::new(),
            }],
        }];

        assert_eq!(
            flatten_chapters(&volumes)
                .into_iter()
                .map(|chapter| chapter.id)
                .collect::<Vec<_>>(),
            ["1", "2"]
        );
    }
}
