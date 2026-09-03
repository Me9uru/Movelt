use std::sync::Arc;

use chrono::Utc;
use serde_json::{json, Value};
use tauri::State;

use crate::{
    api::{cache::AppCache, OfficialClient},
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
    let shelf = client.get_bookshelf().await?;
    let mut items = shelf_items(&shelf);
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
    cache: State<'_, AppCache>,
) -> Result<Arc<Vec<NovelBookshelfEntry>>> {
    cache
        .load_cache(&cache.novel_bookshelf, (), load_novel_bookshelf(&client))
        .await
}

async fn load_novel_bookshelf(client: &OfficialClient) -> Result<Vec<NovelBookshelfEntry>> {
    let shelf = client.get_bookshelf().await?;
    let items: Vec<_> = shelf_items(&shelf)
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
    cache: State<'_, AppCache>,
    book_id: String,
) -> Result<bool> {
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
    cache: State<'_, AppCache>,
    book_id: String,
    present: bool,
) -> Result<()> {
    set_shelf(&client, parse_id(&book_id)?, "BOOK", present).await?;
    cache.invalidate_bookshelves();
    Ok(())
}

#[tauri::command]
/// 获取漫画书架中的作品。
pub(crate) async fn list_comic_bookshelf(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
) -> Result<Arc<Vec<ComicBookshelfEntry>>> {
    cache
        .load_cache(&cache.comic_bookshelf, (), load_comic_bookshelf(&client))
        .await
}

async fn load_comic_bookshelf(client: &OfficialClient) -> Result<Vec<ComicBookshelfEntry>> {
    let shelf = client.get_bookshelf().await?;
    let items: Vec<_> = shelf_items(&shelf)
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
    cache: State<'_, AppCache>,
    comic_id: String,
) -> Result<bool> {
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
    cache: State<'_, AppCache>,
    comic_id: String,
    present: bool,
) -> Result<()> {
    set_shelf(&client, parse_id(&comic_id)?, "BOOK", present).await?;
    cache.invalidate_bookshelves();
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::is_comic_book;

    #[test]
    fn separates_comics_from_novels_using_official_book_type() {
        assert!(is_comic_book(&json!({"Id": 1, "Type": "Comic"})));
        assert!(!is_comic_book(&json!({"Id": 2, "Type": "Novel"})));
        assert!(!is_comic_book(&json!({"Id": 3})));
    }
}
