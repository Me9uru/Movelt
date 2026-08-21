use chrono::Utc;
use serde_json::{json, Value};
use tauri::State;

use crate::{
    api::OfficialClient,
    dto::{bookshelf::BookshelfEntry, novel::NovelSummary},
    error::Result,
};

use super::{
    adapter::{array, number, optional_string, parse_id},
    novel::novel,
};

/// 读取官方书架中的原始条目。
pub(super) fn shelf_items(value: &Value) -> Vec<Value> {
    array(value, "data").to_vec()
}

/// 判断书架条目是否属于指定作品类型。
pub(super) fn is_kind(item: &Value, kind: &str) -> bool {
    item.get("type")
        .and_then(Value::as_str)
        .is_some_and(|ty| ty.eq_ignore_ascii_case(kind))
        || item.get("type").and_then(Value::as_i64) == Some(if kind == "BOOK" { 0 } else { 1 })
}

/// 按官方接口的批量上限查询书架中的作品。
pub(super) async fn books_for_ids(
    client: &OfficialClient,
    ids: Vec<i64>,
    ty: Option<&str>,
) -> Result<Vec<NovelSummary>> {
    let mut books = Vec::new();
    for ids in ids.chunks(24) {
        let response = client.books_by_ids(ids, ty).await?;
        let items = response
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_else(|| array(&response, "Data"));
        books.extend(items.iter().map(novel));
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
    let shelf = client.bookshelf().await?;
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
pub(crate) async fn list_bookshelf(
    client: State<'_, OfficialClient>,
    query: Option<String>,
) -> Result<Vec<BookshelfEntry>> {
    let shelf = client.bookshelf().await?;
    let items: Vec<_> = shelf_items(&shelf)
        .into_iter()
        .filter(|item| is_kind(item, "BOOK"))
        .collect();
    let books = books_for_ids(
        &client,
        items.iter().map(|item| number(item, "id")).collect(),
        None,
    )
    .await?;
    let query = query.unwrap_or_default().to_lowercase();
    Ok(items
        .into_iter()
        .filter_map(|item| {
            books
                .iter()
                .find(|book| book.id == number(&item, "id").to_string())
                .cloned()
                .map(|book| BookshelfEntry {
                    added_at: optional_string(&item, "updateAt").unwrap_or_default(),
                    book,
                    progress: None,
                })
        })
        .filter(|entry| query.is_empty() || entry.book.title.to_lowercase().contains(&query))
        .collect())
}

#[tauri::command]
/// 设置小说是否存在于书架中。
pub(crate) async fn set_novel_bookshelf(
    client: State<'_, OfficialClient>,
    book_id: String,
    present: bool,
) -> Result<()> {
    set_shelf(&client, parse_id(&book_id)?, "BOOK", present).await
}
