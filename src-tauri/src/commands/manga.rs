use serde_json::{json, Value};
use tauri::State;

use crate::{
    api::OfficialClient,
    dto::manga::{MangaChapter, MangaDetail, MangaPageBatch, MangaPageList, MangaSummary},
    error::{AppError, Result},
};

use super::common::{
    array, books_for_ids, is_kind, manga, number, optional_html, optional_string, parse_id,
    set_shelf, shelf_items, string,
};

#[tauri::command]
pub(crate) async fn browse_manga(
    client: State<'_, OfficialClient>,
    query: Option<String>,
    page_number: i64,
    browse_type: String,
) -> Result<Vec<MangaSummary>> {
    let search = browse_type == "SEARCH" || browse_type == "TAGS";
    let method = if search {
        "SearchComicSeries"
    } else {
        "GetComicList"
    };
    let payload = if search {
        json!({"KeyWords": query.unwrap_or_default(), "Page": page_number, "Size": 30, "Mode": if browse_type == "TAGS" { "tags" } else { "fuzzy" }})
    } else {
        json!({"Page": page_number, "Size": 30, "Order": if browse_type == "POPULAR" { "view" } else if browse_type == "NEW" { "new" } else { "latest" }})
    };
    Ok(array(&client.hub(method, payload).await?, "Data")
        .iter()
        .map(manga)
        .collect())
}
#[tauri::command]
pub(crate) async fn list_manga_bookshelf(
    client: State<'_, OfficialClient>,
) -> Result<Vec<MangaSummary>> {
    let shelf = client.hub("GetBookShelf", json!({})).await?;
    let ids = shelf_items(&shelf)
        .into_iter()
        .filter(|item| is_kind(item, "COMIC"))
        .map(|item| number(&item, "id"))
        .collect();
    Ok(books_for_ids(&client, ids, Some("Comic"))
        .await?
        .iter()
        .map(|book| MangaSummary {
            id: book.id.clone(),
            title: book.title.clone(),
            thumbnail_url: book.cover_url.clone(),
            author: book.author.clone(),
            unread_count: 0,
            source_name: Some("LightNovelShelf".into()),
        })
        .collect())
}
#[tauri::command]
pub(crate) async fn is_on_manga_bookshelf(
    client: State<'_, OfficialClient>,
    manga_id: String,
) -> Result<bool> {
    let shelf = client.hub("GetBookShelf", json!({})).await?;
    let id = parse_id(&manga_id)?;
    Ok(shelf_items(&shelf)
        .iter()
        .any(|item| number(item, "id") == id && is_kind(item, "COMIC")))
}
#[tauri::command]
pub(crate) async fn set_manga_bookshelf(
    client: State<'_, OfficialClient>,
    manga_id: String,
    present: bool,
) -> Result<()> {
    set_shelf(&client, parse_id(&manga_id)?, "COMIC", present).await
}
#[tauri::command]
pub(crate) async fn get_manga(
    client: State<'_, OfficialClient>,
    manga_id: String,
) -> Result<MangaDetail> {
    let response = client
        .hub("GetComicInfo", json!({"Id": parse_id(&manga_id)?}))
        .await?;
    let book = response
        .get("Book")
        .ok_or_else(|| AppError::InvalidResponse("漫画详情缺失".into()))?;
    let chapters = array(book, "Chapters");
    let chapters = if chapters.is_empty() {
        array(book, "Chapter")
    } else {
        chapters
    };
    Ok(MangaDetail {
        summary: manga(book),
        artist: None,
        description: optional_html(book, "Introduction"),
        genre: book
            .pointer("/Extra/classification/tags")
            .and_then(Value::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        status: optional_string(book, "LastUpdatedChapter").unwrap_or_default(),
        chapters: chapters
            .iter()
            .map(|chapter| MangaChapter {
                id: number(chapter, "Id").to_string(),
                name: string(chapter, "Title"),
                chapter_number: number(chapter, "SortNum"),
                is_read: false,
                last_page_read: 0,
                page_count: number(chapter, "PageCount"),
            })
            .collect(),
    })
}
#[tauri::command]
pub(crate) async fn get_manga_chapter_pages(
    client: State<'_, OfficialClient>,
    chapter_id: String,
) -> Result<MangaPageList> {
    let response = client
        .hub(
            "GetComicContent",
            json!({"Cid": parse_id(&chapter_id)?, "Skip": 0, "Take": 12}),
        )
        .await?;
    let chapter = response
        .get("Chapter")
        .ok_or_else(|| AppError::InvalidResponse("漫画页面缺失".into()))?;
    Ok(MangaPageList {
        chapter_id,
        page_count: number(chapter, "Total"),
        first_page_urls: array(chapter, "Images")
            .iter()
            .filter_map(|image| optional_string(image, "Url"))
            .collect(),
    })
}
#[tauri::command]
pub(crate) async fn get_manga_page_batch(
    client: State<'_, OfficialClient>,
    chapter_id: String,
    page_index: i64,
) -> Result<MangaPageBatch> {
    let start_index = page_index.div_euclid(12) * 12;
    let response = client
        .hub(
            "GetComicContent",
            json!({"Cid": parse_id(&chapter_id)?, "Skip": start_index, "Take": 12}),
        )
        .await?;
    let chapter = response
        .get("Chapter")
        .ok_or_else(|| AppError::InvalidResponse("漫画页面缺失".into()))?;
    Ok(MangaPageBatch {
        start_index,
        page_urls: array(chapter, "Images")
            .iter()
            .filter_map(|image| optional_string(image, "Url"))
            .collect(),
    })
}
