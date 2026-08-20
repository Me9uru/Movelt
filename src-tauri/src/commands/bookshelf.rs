use tauri::State;

use crate::{api::OfficialClient, dto::bookshelf::BookshelfEntry, error::Result};

use super::common::{books_for_ids, is_kind, number, optional_string, set_shelf, shelf_items};

#[tauri::command]
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
pub(crate) async fn set_novel_bookshelf(
    client: State<'_, OfficialClient>,
    book_id: String,
    present: bool,
) -> Result<()> {
    set_shelf(&client, super::common::parse_id(&book_id)?, "BOOK", present).await
}
