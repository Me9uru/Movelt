use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::dto::{manga::MangaPageList, novel::ReaderDocument};

const MAX_CACHED_CHAPTERS: usize = 20;

#[derive(Clone, Default)]
pub(crate) struct ReaderCache {
    inner: Arc<Mutex<CacheEntries>>,
}

#[derive(Default)]
struct CacheEntries {
    clock: u64,
    novels: HashMap<String, Cached<ReaderDocument>>,
    novel_chapters: HashMap<String, Vec<String>>,
    manga_pages: HashMap<String, Cached<MangaPageList>>,
}

struct Cached<T> {
    value: T,
    last_used: u64,
}

impl ReaderCache {
    pub(crate) fn clear(&self) {
        *self.inner.lock().expect("reader cache lock poisoned") = CacheEntries::default();
    }

    pub(crate) fn novel(&self, book_id: &str, chapter_id: &str) -> Option<ReaderDocument> {
        let mut entries = self.inner.lock().expect("reader cache lock poisoned");
        entries.clock += 1;
        let clock = entries.clock;
        entries
            .novels
            .get_mut(&novel_key(book_id, chapter_id))
            .map(|entry| {
                entry.last_used = clock;
                entry.value.clone()
            })
    }

    pub(crate) fn store_novel(&self, document: ReaderDocument) {
        let key = novel_key(&document.book_id, &document.chapter_id);
        let mut entries = self.inner.lock().expect("reader cache lock poisoned");
        entries.clock += 1;
        let clock = entries.clock;
        entries.novels.insert(
            key,
            Cached {
                value: document,
                last_used: clock,
            },
        );
        trim(&mut entries.novels);
    }

    pub(crate) fn store_novel_chapters(&self, book_id: String, chapter_ids: Vec<String>) {
        self.inner
            .lock()
            .expect("reader cache lock poisoned")
            .novel_chapters
            .insert(book_id, chapter_ids);
    }

    pub(crate) fn novel_chapters(&self, book_id: &str) -> Option<Vec<String>> {
        self.inner
            .lock()
            .expect("reader cache lock poisoned")
            .novel_chapters
            .get(book_id)
            .cloned()
    }

    pub(crate) fn manga_pages(&self, chapter_id: &str) -> Option<MangaPageList> {
        let mut entries = self.inner.lock().expect("reader cache lock poisoned");
        entries.clock += 1;
        let clock = entries.clock;
        entries.manga_pages.get_mut(chapter_id).map(|entry| {
            entry.last_used = clock;
            entry.value.clone()
        })
    }

    pub(crate) fn store_manga_pages(&self, pages: MangaPageList) {
        let mut entries = self.inner.lock().expect("reader cache lock poisoned");
        entries.clock += 1;
        let clock = entries.clock;
        entries.manga_pages.insert(
            pages.chapter_id.clone(),
            Cached {
                value: pages,
                last_used: clock,
            },
        );
        trim(&mut entries.manga_pages);
    }
}

fn novel_key(book_id: &str, chapter_id: &str) -> String {
    format!("{book_id}:{chapter_id}")
}

fn trim<T>(entries: &mut HashMap<String, Cached<T>>) {
    while entries.len() > MAX_CACHED_CHAPTERS {
        if let Some(key) = entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(key, _)| key.clone())
        {
            entries.remove(&key);
        } else {
            break;
        }
    }
}

pub(crate) fn neighbor_ids(ids: &[String], current_id: &str) -> Vec<String> {
    let Some(index) = ids.iter().position(|id| id == current_id) else {
        return Vec::new();
    };
    let start = index.saturating_sub(2);
    let end = (index + 3).min(ids.len());
    ids[start..end]
        .iter()
        .filter(|id| id.as_str() != current_id)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::neighbor_ids;

    #[test]
    fn returns_two_chapters_on_each_side_when_available() {
        let ids = (1..=7).map(|id| id.to_string()).collect::<Vec<_>>();
        assert_eq!(neighbor_ids(&ids, "4"), ["2", "3", "5", "6"]);
    }

    #[test]
    fn clamps_neighbors_at_catalogue_edges() {
        let ids = (1..=4).map(|id| id.to_string()).collect::<Vec<_>>();
        assert_eq!(neighbor_ids(&ids, "1"), ["2", "3"]);
        assert_eq!(neighbor_ids(&ids, "4"), ["2", "3"]);
    }
}
