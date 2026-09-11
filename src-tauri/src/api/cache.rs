use std::{future::Future, sync::Arc, time::Duration};

use moka::future::Cache;

use crate::{
    dto::{
        bookshelf::{ComicBookshelfEntry, NovelBookshelfEntry},
        comic::{ComicChapterPageBatch, ComicSummary},
        common::ListKey,
        novel::{NovelChapterContent, NovelSummary},
    },
    error::Result,
};

const MAX_CACHED_CHAPTERS: u64 = 5;
const MAX_CACHED_CATALOGUES: u64 = 100;
const MAX_CACHED_COMIC_PAGES: u64 = 100;

/// 仅存活于应用进程内的官方 API 数据缓存。
///
/// 列表、书架和阅读器响应均在 Rust 后端复用，登出或身份切换时统一清除。
#[derive(Clone)]
pub(crate) struct AppCache {
    pub(crate) novel_list: Cache<ListKey, Arc<Vec<NovelSummary>>>,
    pub(crate) novel_rank: Cache<i64, Arc<Vec<NovelSummary>>>,
    pub(crate) comic_list: Cache<ListKey, Arc<Vec<ComicSummary>>>,
    pub(crate) novel_bookshelf: Cache<(), Arc<Vec<NovelBookshelfEntry>>>,
    pub(crate) comic_bookshelf: Cache<(), Arc<Vec<ComicBookshelfEntry>>>,
    pub(crate) novel_pages: Cache<NovelChapterKey, Arc<NovelChapterContent>>,
    pub(crate) novel_chapters: Cache<String, Arc<Vec<String>>>,
    pub(crate) comic_pages: Cache<ComicPageBatchKey, Arc<ComicChapterPageBatch>>,
    pub(crate) comic_chapters: Cache<String, Arc<Vec<String>>>,
}

impl Default for AppCache {
    fn default() -> Self {
        Self {
            novel_list: create_cache(Duration::from_secs(60 * 60), 100),
            novel_rank: create_cache(Duration::from_secs(60 * 60), 100),
            comic_list: create_cache(Duration::from_secs(60 * 60), 100),
            novel_bookshelf: create_cache(Duration::from_secs(24 * 60 * 60), 100),
            comic_bookshelf: create_cache(Duration::from_secs(24 * 60 * 60), 100),
            novel_pages: create_cache(Duration::from_secs(24 * 60 * 60), MAX_CACHED_CHAPTERS),
            novel_chapters: create_cache(Duration::from_secs(30 * 60), MAX_CACHED_CATALOGUES),
            comic_pages: create_cache(Duration::from_secs(24 * 60 * 60), MAX_CACHED_COMIC_PAGES),
            comic_chapters: create_cache(Duration::from_secs(30 * 60), MAX_CACHED_CATALOGUES),
        }
    }
}

impl AppCache {
    /// 读取缓存；未命中时仅执行一次异步加载，并在成功后写入。
    pub(crate) async fn load_cache<K, V, F>(
        &self,
        cache: &Cache<K, Arc<V>>,
        key: K,
        init: F,
    ) -> Result<Arc<V>>
    where
        K: Send + Sync + 'static + std::hash::Hash + Eq,
        V: Send + Sync + 'static,
        F: Future<Output = Result<V>>,
    {
        // Moka's coalescing state must not inflate every command future on the
        // Android IPC thread. Allocation/polling happens after the command is spawned.
        Box::pin(cache.try_get_with(key, async { Box::pin(init).await.map(Arc::new) }))
            .await
            .map_err(|error| (*error).clone())
    }

    /// 将已获取的官方数据写入指定缓存。
    pub(crate) async fn store_cache<K, V>(&self, cache: &Cache<K, Arc<V>>, key: K, value: V)
    where
        K: Send + Sync + 'static + std::hash::Hash + Eq,
        V: Send + Sync + 'static,
    {
        cache.insert(key, Arc::new(value)).await;
    }

    pub(crate) fn invalidate_bookshelves(&self) {
        self.novel_bookshelf.invalidate_all();
        self.comic_bookshelf.invalidate_all();
    }
}

/// 小说章节内容的缓存键，包含影响响应内容的全部请求参数。
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(crate) struct NovelChapterKey {
    book_id: String,
    chapter_id: String,
    convert: Option<String>,
}

impl NovelChapterKey {
    pub(crate) fn new(book_id: &str, chapter_id: &str, convert: Option<&str>) -> Self {
        Self {
            book_id: book_id.into(),
            chapter_id: chapter_id.into(),
            convert: convert.map(str::to_owned),
        }
    }
}

/// 漫画图片批次的缓存键，覆盖 `GetComicContent` 的可变请求参数。
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(crate) struct ComicPageBatchKey {
    /// CID全局唯一
    chapter_id: String,
    start_index: i64,
}

impl ComicPageBatchKey {
    pub(crate) fn new(chapter_id: String, start_index: i64) -> Self {
        Self {
            chapter_id,
            start_index,
        }
    }
}

fn create_cache<K, V>(ttl: Duration, max_capacity: u64) -> Cache<K, V>
where
    K: Send + Sync + 'static + std::hash::Hash + Eq,
    V: Clone + Send + Sync + 'static,
{
    Cache::builder()
        .time_to_live(ttl)
        .max_capacity(max_capacity)
        .build()
}

/// 返回当前章节之后最多两章的 ID。
pub(crate) fn get_neighbor_ids(ids: &[String], current_id: &str) -> Vec<String> {
    let Some(index) = ids.iter().position(|id| id == current_id) else {
        return Vec::new();
    };
    let end = (index + 3).min(ids.len());
    ids[index + 1..end].to_vec()
}

#[cfg(test)]
mod tests {
    use super::{get_neighbor_ids, NovelChapterKey};

    #[test]
    fn returns_two_following_chapters_when_available() {
        let ids = (1..=7).map(|id| id.to_string()).collect::<Vec<_>>();
        assert_eq!(get_neighbor_ids(&ids, "4"), ["5", "6"]);
    }

    #[test]
    fn clamps_neighbors_at_catalogue_edges() {
        let ids = (1..=4).map(|id| id.to_string()).collect::<Vec<_>>();
        assert_eq!(get_neighbor_ids(&ids, "1"), ["2", "3"]);
        assert!(get_neighbor_ids(&ids, "4").is_empty());
    }

    #[test]
    fn separates_chapter_cache_keys_by_conversion_mode() {
        assert_ne!(
            NovelChapterKey::new("1", "2", None),
            NovelChapterKey::new("1", "2", Some("t2s"))
        );
    }
}
