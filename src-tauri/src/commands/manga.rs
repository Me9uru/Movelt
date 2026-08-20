use serde_json::Value;
use tauri::State;

use crate::{
    api::OfficialClient,
    dto::manga::{MangaChapter, MangaDetail, MangaPageBatch, MangaPageList, MangaSummary},
    error::{AppError, Result},
    reader_cache::{neighbor_ids, ReaderCache},
};

use super::common::{
    array, books_for_ids, is_kind, manga, number, optional_html, optional_string, parse_id,
    position, set_shelf, shelf_items, string,
};

#[tauri::command]
pub(crate) async fn browse_manga(
    client: State<'_, OfficialClient>,
    query: Option<String>,
    page_number: i64,
    browse_type: String,
) -> Result<Vec<MangaSummary>> {
    Ok(array(
        &client.manga_list(query, page_number, &browse_type).await?,
        "Data",
    )
    .iter()
    .map(manga)
    .collect())
}
#[tauri::command]
pub(crate) async fn list_manga_bookshelf(
    client: State<'_, OfficialClient>,
) -> Result<Vec<MangaSummary>> {
    let shelf = client.bookshelf().await?;
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
    let shelf = client.bookshelf().await?;
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
    cache: State<'_, ReaderCache>,
    manga_id: String,
    current_chapter_id: Option<String>,
) -> Result<MangaDetail> {
    let response = client.manga_info(parse_id(&manga_id)?).await?;
    let book = response
        .get("Book")
        .ok_or_else(|| AppError::InvalidResponse("漫画详情缺失".into()))?;
    let chapters = array(book, "Chapters");
    let chapters = if chapters.is_empty() {
        array(book, "Chapter")
    } else {
        chapters
    };
    let detail = MangaDetail {
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
        read_position: position(response.get("ReadPosition")),
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
    };
    let current_chapter_id = current_chapter_id
        .or_else(|| {
            detail
                .read_position
                .as_ref()
                .map(|position| position.chapter_id.clone())
        })
        .or_else(|| detail.chapters.first().map(|chapter| chapter.id.clone()));
    if let Some(current_chapter_id) = current_chapter_id {
        preload_manga_neighbors(
            client.inner().clone(),
            cache.inner().clone(),
            detail
                .chapters
                .iter()
                .map(|chapter| chapter.id.clone())
                .collect(),
            current_chapter_id,
        );
    }
    Ok(detail)
}
#[tauri::command]
pub(crate) async fn save_manga_read_position(
    client: State<'_, OfficialClient>,
    manga_id: String,
    chapter_id: String,
    page: i64,
) -> Result<()> {
    if page < 1 {
        return Err(AppError::InvalidResponse("无效的漫画页码".into()));
    }
    client
        .save_manga_position(parse_id(&manga_id)?, parse_id(&chapter_id)?, page)
        .await
}
#[tauri::command]
pub(crate) async fn get_manga_chapter_pages(
    client: State<'_, OfficialClient>,
    cache: State<'_, ReaderCache>,
    chapter_id: String,
) -> Result<MangaPageList> {
    if let Some(pages) = cache.manga_pages(&chapter_id) {
        return Ok(pages);
    }
    let response = client.manga_content(parse_id(&chapter_id)?, 0).await?;
    let chapter = response
        .get("Chapter")
        .ok_or_else(|| AppError::InvalidResponse("漫画页面缺失".into()))?;
    let pages = MangaPageList {
        chapter_id,
        page_count: number(chapter, "Total"),
        first_page_urls: array(chapter, "Images")
            .iter()
            .filter_map(|image| optional_string(image, "Url"))
            .collect(),
        read_position: position(response.get("ReadPosition")),
    };
    cache.store_manga_pages(pages.clone());
    Ok(pages)
}

async fn load_manga_chapter_pages(
    client: &OfficialClient,
    cache: &ReaderCache,
    chapter_id: &str,
) -> Result<MangaPageList> {
    if let Some(pages) = cache.manga_pages(chapter_id) {
        return Ok(pages);
    }
    let response = client.manga_content(parse_id(chapter_id)?, 0).await?;
    let chapter = response
        .get("Chapter")
        .ok_or_else(|| AppError::InvalidResponse("漫画页面缺失".into()))?;
    let pages = MangaPageList {
        chapter_id: chapter_id.into(),
        page_count: number(chapter, "Total"),
        first_page_urls: array(chapter, "Images")
            .iter()
            .filter_map(|image| optional_string(image, "Url"))
            .collect(),
        read_position: position(response.get("ReadPosition")),
    };
    cache.store_manga_pages(pages.clone());
    Ok(pages)
}

fn preload_manga_neighbors(
    client: OfficialClient,
    cache: ReaderCache,
    chapter_ids: Vec<String>,
    current_chapter_id: String,
) {
    let neighbors = neighbor_ids(&chapter_ids, &current_chapter_id);
    tauri::async_runtime::spawn(async move {
        for chapter_id in neighbors {
            if cache.manga_pages(&chapter_id).is_none() {
                let _ = load_manga_chapter_pages(&client, &cache, &chapter_id).await;
            }
        }
    });
}
#[tauri::command]
pub(crate) async fn get_manga_page_batch(
    client: State<'_, OfficialClient>,
    chapter_id: String,
    page_index: i64,
) -> Result<MangaPageBatch> {
    let start_index = page_index.div_euclid(12) * 12;
    let response = client
        .manga_content(parse_id(&chapter_id)?, start_index)
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
