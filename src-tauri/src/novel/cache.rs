use std::collections::{HashMap, VecDeque};

use tokio::sync::watch;

use super::{domain::ChapterContent, error::NovelError};

const MAX_CACHED_CHAPTERS: usize = 8;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(super) struct ChapterKey {
    pub(super) source: String,
    pub(super) novel_id: String,
    pub(super) chapter_id: String,
}

type ChapterResult = Result<ChapterContent, NovelError>;
type PendingChapter = watch::Receiver<Option<ChapterResult>>;

pub(super) enum ChapterRequest {
    Cached(ChapterContent),
    Pending(PendingChapter),
    Start {
        receiver: PendingChapter,
        sender: watch::Sender<Option<ChapterResult>>,
    },
}

#[derive(Default)]
pub(super) struct ChapterCache {
    documents: HashMap<ChapterKey, ChapterContent>,
    order: VecDeque<ChapterKey>,
    pending: HashMap<ChapterKey, PendingChapter>,
}

impl ChapterCache {
    pub(super) fn begin(&mut self, key: &ChapterKey) -> ChapterRequest {
        if let Some(document) = self.documents.get(key).cloned() {
            self.touch(key);
            return ChapterRequest::Cached(document);
        }
        if let Some(receiver) = self.pending.get(key) {
            return ChapterRequest::Pending(receiver.clone());
        }

        let (sender, receiver) = watch::channel(None);
        self.pending.insert(key.clone(), receiver.clone());
        ChapterRequest::Start { receiver, sender }
    }

    pub(super) fn complete(&mut self, key: ChapterKey, result: &ChapterResult) {
        self.pending.remove(&key);
        if let Ok(document) = result {
            self.insert(key, document.clone());
        }
    }

    fn insert(&mut self, key: ChapterKey, document: ChapterContent) {
        self.documents.insert(key.clone(), document);
        self.touch(&key);
        while self.documents.len() > MAX_CACHED_CHAPTERS {
            if let Some(oldest) = self.order.pop_front() {
                self.documents.remove(&oldest);
            }
        }
    }

    fn touch(&mut self, key: &ChapterKey) {
        if let Some(index) = self.order.iter().position(|candidate| candidate == key) {
            self.order.remove(index);
        }
        self.order.push_back(key.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chapter_key(chapter_id: usize) -> ChapterKey {
        ChapterKey {
            source: "test".into(),
            novel_id: "book".into(),
            chapter_id: chapter_id.to_string(),
        }
    }

    fn chapter(chapter_id: usize) -> ChapterContent {
        ChapterContent {
            source: "test".into(),
            novel_id: "book".into(),
            chapter_id: chapter_id.to_string(),
            title: format!("Chapter {chapter_id}"),
            nodes: Vec::new(),
        }
    }

    #[test]
    fn evicts_the_least_recently_used_document() {
        let mut cache = ChapterCache::default();
        for chapter_id in 0..MAX_CACHED_CHAPTERS {
            cache.insert(chapter_key(chapter_id), chapter(chapter_id));
        }

        assert!(matches!(
            cache.begin(&chapter_key(0)),
            ChapterRequest::Cached(_)
        ));
        cache.insert(
            chapter_key(MAX_CACHED_CHAPTERS),
            chapter(MAX_CACHED_CHAPTERS),
        );

        assert!(cache.documents.contains_key(&chapter_key(0)));
        assert!(!cache.documents.contains_key(&chapter_key(1)));
        assert_eq!(cache.documents.len(), MAX_CACHED_CHAPTERS);
    }
}
