use std::sync::Arc;

use chrono::Utc;
use serde_json::{json, Value};
use tauri::State;

use crate::{
    api::{cache::AppCache, OfficialClient},
    dto::{bookshelf::BookshelfEntry, comic::ComicSummary},
    error::Result,
    mapping::{
        bookshelf::{is_kind, items as shelf_items},
        comic::bookshelf_summary as comic_bookshelf_summary,
        novel::summary as novel_summary,
        value::{array, number, optional_string},
    },
};

use super::validation::parse_id;

fn title_matches_query(title: &str, query: &str) -> bool {
    query.is_empty() || title.to_lowercase().contains(query)
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
        .any(|item| number(item, "id") == id && is_kind(item, kind));
    if present && !exists {
        items.insert(0, json!({"id": id, "type": kind, "parents": [], "index": 0, "updateAt": Utc::now().to_rfc3339()}));
    }
    if !present {
        items.retain(|item| number(item, "id") != id || !is_kind(item, kind));
    }
    client
        .save_bookshelf(
            items,
            shelf
                .get("ver")
                .and_then(Value::as_str)
                .unwrap_or("20220211"),
        )
        .await
}

#[tauri::command]
/// 获取小说书架，并支持按标题筛选。
pub(crate) async fn list_novel_bookshelf(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    query: Option<String>,
) -> Result<Arc<Vec<BookshelfEntry>>> {
    let query = query.unwrap_or_default().to_lowercase();
    let books = cache
        .load_cache(&cache.novel_bookshelf, (), load_novel_bookshelf(&client))
        .await?;
    if query.is_empty() {
        return Ok(books);
    }

    Ok(Arc::new(
        books
            .iter()
            .filter(|entry| title_matches_query(&entry.book.title, &query))
            .cloned()
            .collect(),
    ))
}

async fn load_novel_bookshelf(client: &OfficialClient) -> Result<Vec<BookshelfEntry>> {
    let shelf = client.get_bookshelf().await?;
    let items: Vec<_> = shelf_items(&shelf)
        .into_iter()
        .filter(|item| is_kind(item, "BOOK"))
        .collect();
    let books = book_values_for_ids(
        client,
        items.iter().map(|item| number(item, "id")).collect(),
        None,
    )
    .await?;
    Ok(items
        .into_iter()
        .filter_map(|item| {
            books
                .iter()
                .find(|book| number(book, "Id") == number(&item, "id"))
                .map(novel_summary)
                .map(|book| BookshelfEntry {
                    added_at: optional_string(&item, "updateAt").unwrap_or_default(),
                    book,
                    progress: None,
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
    query: Option<String>,
) -> Result<Arc<Vec<ComicSummary>>> {
    let query = query.unwrap_or_default().to_lowercase();
    let comics = cache
        .load_cache(&cache.comic_bookshelf, (), load_comic_bookshelf(&client))
        .await?;
    if query.is_empty() {
        return Ok(comics);
    }

    Ok(Arc::new(
        comics
            .iter()
            .filter(|comic| title_matches_query(&comic.title, &query))
            .cloned()
            .collect(),
    ))
}

async fn load_comic_bookshelf(client: &OfficialClient) -> Result<Vec<ComicSummary>> {
    let shelf = client.get_bookshelf().await?;
    let ids = shelf_items(&shelf)
        .into_iter()
        .filter(|item| is_kind(item, "COMIC"))
        .map(|item| number(&item, "id"))
        .collect();
    Ok(book_values_for_ids(client, ids, Some("Comic"))
        .await?
        .iter()
        .map(comic_bookshelf_summary)
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
        .any(|comic| comic.book_id.as_deref() == Some(&comic_id)))
}

#[tauri::command]
/// 设置漫画是否存在于书架中。
pub(crate) async fn set_comic_bookshelf(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    comic_id: String,
    present: bool,
) -> Result<()> {
    set_shelf(&client, parse_id(&comic_id)?, "COMIC", present).await?;
    cache.invalidate_bookshelves();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::title_matches_query;

    #[test]
    fn filters_comic_bookshelf_titles_case_insensitively() {
        assert!(title_matches_query("My Favorite Comic", "favorite"));
        assert!(!title_matches_query("My Favorite Comic", "author"));
        assert!(title_matches_query("My Favorite Comic", ""));
    }
}
