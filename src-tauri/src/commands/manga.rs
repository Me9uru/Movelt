use serde_json::Value;
use tauri::State;

use crate::{
    api::OfficialClient,
    dto::manga::{MangaChapter, MangaDetail, MangaPageBatch, MangaPageList, MangaSummary},
    error::{AppError, Result},
    reader_cache::{neighbor_ids, ReaderCache},
};

use super::{
    adapter::{array, number, optional_html, optional_string, parse_id, position, string},
    bookshelf::{books_for_ids, is_kind, set_shelf, shelf_items},
};

/// 将官方漫画数据映射为应用摘要。
fn manga(value: &Value) -> MangaSummary {
    MangaSummary {
        id: number(value, "Id").to_string(),
        title: string(value, "Title"),
        thumbnail_url: optional_string(value, "Cover"),
        author: optional_string(value, "Author"),
        unread_count: 0,
        source_name: Some("LightNovelShelf".into()),
    }
}

#[tauri::command]
/// 浏览或搜索漫画列表。
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
/// 获取漫画书架中的作品。
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
/// 判断漫画是否已加入书架。
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
/// 设置漫画是否存在于书架中。
pub(crate) async fn set_manga_bookshelf(
    client: State<'_, OfficialClient>,
    manga_id: String,
    present: bool,
) -> Result<()> {
    set_shelf(&client, parse_id(&manga_id)?, "COMIC", present).await
}

#[tauri::command]
/// 获取漫画详情。
pub(crate) async fn get_manga(
    client: State<'_, OfficialClient>,
    cache: State<'_, ReaderCache>,
    manga_id: String,
) -> Result<MangaDetail> {
    let response = client.manga_info(parse_id(&manga_id)?).await?;
    let book = response
        .get("Book")
        .ok_or_else(|| AppError::protocol("漫画详情响应缺少 Book"))?;
    let chapters = array(book, "Chapters");
    let chapters = if chapters.is_empty() {
        array(book, "Chapter")
    } else {
        chapters
    };
    cache.store_manga_chapters(
        manga_id,
        chapters
            .iter()
            .map(|chapter| number(chapter, "Id").to_string())
            .collect(),
    );
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
    Ok(detail)
}

#[tauri::command]
/// 保存漫画章节的阅读页码。
pub(crate) async fn save_manga_read_position(
    client: State<'_, OfficialClient>,
    manga_id: String,
    chapter_id: String,
    page: i64,
) -> Result<()> {
    if page < 1 {
        return Err(AppError::invalid_input("漫画页码必须大于 0"));
    }
    client
        .save_manga_position(parse_id(&manga_id)?, parse_id(&chapter_id)?, page)
        .await
}

#[tauri::command]
/// 获取漫画首批页面，并在后台预加载后续章节。
pub(crate) async fn get_manga_chapter_pages(
    client: State<'_, OfficialClient>,
    cache: State<'_, ReaderCache>,
    manga_id: String,
    chapter_id: String,
) -> Result<MangaPageList> {
    let pages = load_manga_chapter_pages(&client, &cache, &chapter_id).await?;
    if let Some(chapter_ids) = cache.manga_chapters(&manga_id) {
        preload_manga_neighbors(
            client.inner().clone(),
            cache.inner().clone(),
            chapter_ids,
            chapter_id,
        );
    }
    Ok(pages)
}

/// 从缓存或官方服务加载漫画章节页面。
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
        .ok_or_else(|| AppError::protocol("漫画页面响应缺少 Chapter"))?;
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

/// 在后台预加载当前章节之后的漫画页面。
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
/// 获取指定分页批次的漫画页面。
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
        .ok_or_else(|| AppError::protocol("漫画页面响应缺少 Chapter"))?;
    Ok(MangaPageBatch {
        start_index,
        page_urls: array(chapter, "Images")
            .iter()
            .filter_map(|image| optional_string(image, "Url"))
            .collect(),
    })
}
