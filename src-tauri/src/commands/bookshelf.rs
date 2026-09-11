use std::sync::Arc;

use chrono::Utc;
use serde_json::{json, Value};
use tauri::State;

use crate::{
    api::OfficialClient,
    dto::bookshelf::{ComicBookshelfEntry, NovelBookshelfEntry},
    error::Result,
    mapping::{
        bookshelf::{
            is_kind, item_id as shelf_item_id, item_updated_at, items as shelf_items,
            version as shelf_version,
        },
        comic::bookshelf_summary as comic_bookshelf_summary,
        novel::summary as novel_summary,
        value::{array, number, optional_number},
    },
};

use super::validation::parse_id;

fn is_comic_book(value: &Value) -> bool {
    value
        .get("Type")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind.eq_ignore_ascii_case("Comic"))
}

/// 按官方接口的批量上限查询书架中的作品。
pub(super) async fn book_values_for_ids(
    client: &OfficialClient,
    ids: Vec<i64>,
    ty: Option<&str>,
) -> Result<Vec<Value>> {
    let mut books = Vec::new();
    for ids in ids.chunks(24) {
        let response = client.get_books_by_ids(ids, ty).await?;
        let items = response
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_else(|| array(&response, "Data"));
        books.extend(items.iter().cloned());
    }
    Ok(books)
}

/// 向官方书架添加或移除指定类型的作品。
pub(super) async fn set_shelf(
    client: &OfficialClient,
    id: i64,
    kind: &str,
    present: bool,
) -> Result<()> {
    let _shelf_guard = client.lock_bookshelf().await?;
    let shelf = client.get_bookshelf().await?;
    let mut items = shelf_items(&shelf)?;
    let exists = items
        .iter()
        .any(|item| shelf_item_id(item) == id && is_kind(item, kind));
    if present && !exists {
        items.insert(0, json!({"id": id, "type": kind, "parents": [], "index": 0, "updateAt": Utc::now().to_rfc3339()}));
    }
    if !present {
        items.retain(|item| shelf_item_id(item) != id || !is_kind(item, kind));
    }
    client
        .save_bookshelf(items, shelf_version(&shelf).unwrap_or("20220211"))
        .await
}

#[tauri::command]
/// 获取小说书架。
pub(crate) async fn list_novel_bookshelf(
    client: State<'_, OfficialClient>,
) -> Result<Arc<Vec<NovelBookshelfEntry>>> {
    let client = client.scoped().await;
    let cache = client.cache();
    cache
        .load_cache(&cache.novel_bookshelf, (), load_novel_bookshelf(&client))
        .await
}

async fn load_novel_bookshelf(client: &OfficialClient) -> Result<Vec<NovelBookshelfEntry>> {
    let shelf = client.get_bookshelf().await?;
    let items: Vec<_> = shelf_items(&shelf)?
        .into_iter()
        .filter(|item| is_kind(item, "BOOK"))
        .collect();
    let books =
        book_values_for_ids(client, items.iter().map(shelf_item_id).collect(), None).await?;
    Ok(items
        .into_iter()
        .filter_map(|item| {
            books
                .iter()
                .find(|book| number(book, "Id") == shelf_item_id(&item) && !is_comic_book(book))
                .map(|book_value| NovelBookshelfEntry {
                    added_at: item_updated_at(&item).unwrap_or_default(),
                    book: novel_summary(book_value),
                    progress: optional_number(book_value, "Progress"),
                })
        })
        .collect())
}

#[tauri::command]
/// 判断小说是否已加入书架。
pub(crate) async fn is_on_novel_bookshelf(
    client: State<'_, OfficialClient>,
    book_id: String,
) -> Result<bool> {
    let client = client.scoped().await;
    let cache = client.cache();
    let book_id = parse_id(&book_id)?.to_string();
    let books = cache
        .load_cache(&cache.novel_bookshelf, (), load_novel_bookshelf(&client))
        .await?;
    Ok(books.iter().any(|entry| entry.book.id == book_id))
}

#[tauri::command]
/// 设置小说是否存在于书架中。
pub(crate) async fn set_novel_bookshelf(
    client: State<'_, OfficialClient>,
    book_id: String,
    present: bool,
) -> Result<()> {
    let client = client.scoped().await;
    let cache = client.cache();
    set_shelf(&client, parse_id(&book_id)?, "BOOK", present).await?;
    cache.invalidate_bookshelves();
    Ok(())
}

#[tauri::command]
/// 获取漫画书架中的作品。
pub(crate) async fn list_comic_bookshelf(
    client: State<'_, OfficialClient>,
) -> Result<Arc<Vec<ComicBookshelfEntry>>> {
    let client = client.scoped().await;
    let cache = client.cache();
    cache
        .load_cache(&cache.comic_bookshelf, (), load_comic_bookshelf(&client))
        .await
}

async fn load_comic_bookshelf(client: &OfficialClient) -> Result<Vec<ComicBookshelfEntry>> {
    let shelf = client.get_bookshelf().await?;
    let items: Vec<_> = shelf_items(&shelf)?
        .into_iter()
        .filter(|item| is_kind(item, "BOOK"))
        .collect();
    let comics = book_values_for_ids(
        client,
        items.iter().map(shelf_item_id).collect(),
        Some("Comic"),
    )
    .await?;
    Ok(items
        .into_iter()
        .filter_map(|item| {
            comics
                .iter()
                .find(|comic| number(comic, "Id") == shelf_item_id(&item))
                .map(|comic_value| ComicBookshelfEntry {
                    comic: comic_bookshelf_summary(comic_value),
                    added_at: item_updated_at(&item).unwrap_or_default(),
                    progress: optional_number(comic_value, "Progress"),
                })
        })
        .collect())
}

#[tauri::command]
/// 判断漫画是否已加入书架。
pub(crate) async fn is_on_comic_bookshelf(
    client: State<'_, OfficialClient>,
    comic_id: String,
) -> Result<bool> {
    let client = client.scoped().await;
    let cache = client.cache();
    let comic_id = parse_id(&comic_id)?.to_string();
    let comics = cache
        .load_cache(&cache.comic_bookshelf, (), load_comic_bookshelf(&client))
        .await?;
    Ok(comics
        .iter()
        .any(|entry| entry.comic.book_id.as_deref() == Some(&comic_id)))
}

#[tauri::command]
/// 设置漫画是否存在于书架中。
pub(crate) async fn set_comic_bookshelf(
    client: State<'_, OfficialClient>,
    comic_id: String,
    present: bool,
) -> Result<()> {
    let client = client.scoped().await;
    let cache = client.cache();
    set_shelf(&client, parse_id(&comic_id)?, "BOOK", present).await?;
    cache.invalidate_bookshelves();
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use tokio::sync::{Mutex, Notify};

    use super::{is_comic_book, set_shelf};
    use crate::{api::mock_client, error::AppError};

    #[test]
    fn separates_comics_from_novels_using_official_book_type() {
        assert!(is_comic_book(&json!({"Id": 1, "Type": "Comic"})));
        assert!(!is_comic_book(&json!({"Id": 2, "Type": "Novel"})));
        assert!(!is_comic_book(&json!({"Id": 3})));
    }

    #[tokio::test]
    async fn serializes_the_entire_bookshelf_update_and_preserves_folders() {
        let folder =
            json!({"type": "FOLDER", "id": "folder", "title": "收藏", "parents": [], "index": 0});
        let state = Arc::new(Mutex::new(
            json!({"data": [folder.clone()], "ver": "20220211"}),
        ));
        let reads = Arc::new(AtomicUsize::new(0));
        let read_started = Arc::new(Notify::new());
        let resume = Arc::new(Notify::new());
        let client = mock_client({
            let state = state.clone();
            let reads = reads.clone();
            let read_started = read_started.clone();
            let resume = resume.clone();
            move |method, payload| {
                let state = state.clone();
                let reads = reads.clone();
                let read_started = read_started.clone();
                let resume = resume.clone();
                async move {
                    match method.as_str() {
                        "GetBookShelf" => {
                            let snapshot = state.lock().await.clone();
                            if reads.fetch_add(1, Ordering::SeqCst) == 0 {
                                read_started.notify_one();
                                resume.notified().await;
                            }
                            Ok(snapshot)
                        }
                        "SaveBookShelf" => {
                            *state.lock().await = payload;
                            Ok(serde_json::Value::Null)
                        }
                        _ => panic!("unexpected method"),
                    }
                }
            }
        })
        .await;
        let first = tokio::spawn({
            let client = client.clone();
            async move { set_shelf(&client, 1, "BOOK", true).await }
        });
        read_started.notified().await;
        let second = set_shelf(&client, 2, "BOOK", true);
        tokio::pin!(second);
        // 主动轮询第二次更新，它必须停在事务锁，不能先读到旧书架。
        tokio::select! {
            biased;
            result = &mut second => panic!("second update bypassed transaction lock: {result:?}"),
            _ = std::future::ready(()) => {}
        }
        assert_eq!(reads.load(Ordering::SeqCst), 1);
        resume.notify_one();
        first.await.unwrap().unwrap();
        second.await.unwrap();
        let shelf = state.lock().await;
        let items = shelf["data"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert!(items.contains(&folder));
        assert!(items.iter().any(|item| item["id"] == 1));
        assert!(items.iter().any(|item| item["id"] == 2));
    }

    #[tokio::test]
    async fn malformed_shelf_is_never_saved() {
        let saves = Arc::new(AtomicUsize::new(0));
        let client = mock_client({
            let saves = saves.clone();
            move |method, _| {
                if method == "SaveBookShelf" {
                    saves.fetch_add(1, Ordering::SeqCst);
                }
                async { Ok(json!({"data": null})) }
            }
        })
        .await;
        assert!(matches!(
            set_shelf(&client, 1, "BOOK", true).await,
            Err(AppError::UpstreamProtocol { .. })
        ));
        assert_eq!(saves.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn does_not_replay_a_write_when_its_response_is_lost() {
        let saves = Arc::new(AtomicUsize::new(0));
        let client = mock_client({
            let saves = saves.clone();
            move |method, _| {
                let result = if method == "SaveBookShelf" {
                    saves.fetch_add(1, Ordering::SeqCst);
                    Err(AppError::transport("response lost"))
                } else {
                    Ok(json!({"data": []}))
                };
                async { result }
            }
        })
        .await;
        assert!(set_shelf(&client, 1, "BOOK", true).await.is_err());
        assert_eq!(saves.load(Ordering::SeqCst), 1);
    }
}
