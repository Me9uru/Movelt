use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::dto::{manga::MangaPageList, novel::ReaderDocument};

const MAX_CACHED_CHAPTERS: usize = 5;

#[derive(Clone, Default)]
pub(crate) struct ReaderCache {
    inner: Arc<Mutex<CacheEntries>>,
}

#[derive(Default)]
struct CacheEntries {
    clock: u64,
    novels: HashMap<String, Cached<ReaderDocument>>,
    novel_chapters: HashMap<String, Vec<String>>,
    manga_chapters: HashMap<String, Vec<String>>,
    manga_pages: HashMap<String, Cached<MangaPageList>>,
}

struct Cached<T> {
    value: T,
    last_used: u64,
}

impl ReaderCache {
    /// 清空全部阅读器缓存。
    pub(crate) fn clear(&self) {
        *self.inner.lock().expect("reader cache lock poisoned") = CacheEntries::default();
    }

    /// 获取指定转换版本的小说章节缓存，并更新其最近访问时间。
    pub(crate) fn novel(
        &self,
        book_id: &str,
        chapter_id: &str,
        convert: Option<&str>,
    ) -> Option<ReaderDocument> {
        let mut entries = self.inner.lock().expect("reader cache lock poisoned");
        entries.clock += 1;
        let clock = entries.clock;
        entries
            .novels
            .get_mut(&novel_key(book_id, chapter_id, convert))
            .map(|entry| {
                entry.last_used = clock;
                entry.value.clone()
            })
    }

    /// 写入指定转换版本的小说章节缓存，并按最近最少使用策略淘汰旧项。
    pub(crate) fn store_novel(&self, document: ReaderDocument, convert: Option<&str>) {
        let key = novel_key(&document.book_id, &document.chapter_id, convert);
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

    /// 缓存小说的有序章节目录。
    pub(crate) fn store_novel_chapters(&self, book_id: String, chapter_ids: Vec<String>) {
        self.inner
            .lock()
            .expect("reader cache lock poisoned")
            .novel_chapters
            .insert(book_id, chapter_ids);
    }

    /// 获取小说的有序章节目录。
    pub(crate) fn novel_chapters(&self, book_id: &str) -> Option<Vec<String>> {
        self.inner
            .lock()
            .expect("reader cache lock poisoned")
            .novel_chapters
            .get(book_id)
            .cloned()
    }

    /// 缓存漫画的有序章节目录。
    pub(crate) fn store_manga_chapters(&self, manga_id: String, chapter_ids: Vec<String>) {
        self.inner
            .lock()
            .expect("reader cache lock poisoned")
            .manga_chapters
            .insert(manga_id, chapter_ids);
    }

    /// 获取漫画的有序章节目录。
    pub(crate) fn manga_chapters(&self, manga_id: &str) -> Option<Vec<String>> {
        self.inner
            .lock()
            .expect("reader cache lock poisoned")
            .manga_chapters
            .get(manga_id)
            .cloned()
    }

    /// 获取漫画章节页面缓存，并更新其最近访问时间。
    pub(crate) fn manga_pages(&self, chapter_id: &str) -> Option<MangaPageList> {
        let mut entries = self.inner.lock().expect("reader cache lock poisoned");
        entries.clock += 1;
        let clock = entries.clock;
        entries.manga_pages.get_mut(chapter_id).map(|entry| {
            entry.last_used = clock;
            entry.value.clone()
        })
    }

    /// 写入漫画章节页面缓存，并按最近最少使用策略淘汰旧项。
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

/// 生成包含文字转换模式的小说章节缓存键。
fn novel_key(book_id: &str, chapter_id: &str, convert: Option<&str>) -> String {
    format!("{book_id}:{chapter_id}:{}", convert.unwrap_or("original"))
}

/// 将缓存条目裁剪至容量上限。
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

/// 返回当前章节之后最多两章的 ID。
pub(crate) fn neighbor_ids(ids: &[String], current_id: &str) -> Vec<String> {
    let Some(index) = ids.iter().position(|id| id == current_id) else {
        return Vec::new();
    };
    let end = (index + 3).min(ids.len());
    ids[index + 1..end].to_vec()
}

#[cfg(test)]
mod tests {
    use super::{neighbor_ids, novel_key};

    #[test]
    fn returns_two_following_chapters_when_available() {
        let ids = (1..=7).map(|id| id.to_string()).collect::<Vec<_>>();
        assert_eq!(neighbor_ids(&ids, "4"), ["5", "6"]);
    }

    #[test]
    fn clamps_neighbors_at_catalogue_edges() {
        let ids = (1..=4).map(|id| id.to_string()).collect::<Vec<_>>();
        assert_eq!(neighbor_ids(&ids, "1"), ["2", "3"]);
        assert!(neighbor_ids(&ids, "4").is_empty());
    }

    #[test]
    fn separates_chapter_cache_keys_by_conversion_mode() {
        assert_ne!(novel_key("1", "2", None), novel_key("1", "2", Some("t2s")));
    }
}
