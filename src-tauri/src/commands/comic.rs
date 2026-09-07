use std::sync::Arc;

use tauri::State;

use crate::{
    api::{
        cache::{AppCache, ComicPageBatchKey},
        OfficialClient,
    },
    dto::{
        comic::{ComicBook, ComicChapterPageBatch, ComicSeriesDetail, ComicSummary},
        common::{ListKey, Order, PaginatedList, SearchMode},
    },
    error::{AppError, Result},
    mapping::{
        comic::{
            book_detail, page_batch, positive_book_id, series_book_ids, series_detail,
            summary as comic_summary,
        },
        common::pagination,
    },
};

use super::validation::{parse_id, validate_page_size};
use crate::mapping::value::array;

#[tauri::command]
/// 浏览漫画列表。
pub(crate) async fn list_comics(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    page_number: i64,
    page_size: i64,
    order: Order,
) -> Result<Arc<Vec<ComicSummary>>> {
    let page_size = validate_page_size(page_size)?;
    let key = ListKey::new(order, page_number, page_size);
    cache
        .load_cache(&cache.comic_list, key, async {
            Ok(array(
                &client.list_comics(page_number, page_size, order).await?,
                "Data",
            )
            .iter()
            .map(comic_summary)
            .collect())
        })
        .await
}

#[tauri::command]
/// 搜索漫画并保留官方分页信息。
pub(crate) async fn search_comics(
    client: State<'_, OfficialClient>,
    query: String,
    page_number: i64,
    page_size: i64,
    mode: SearchMode,
) -> Result<PaginatedList<ComicSummary>> {
    let response = client
        .search_comics(query, page_number, validate_page_size(page_size)?, mode)
        .await?;
    Ok(PaginatedList {
        items: array(&response, "Data").iter().map(comic_summary).collect(),
        pagination: pagination(&response, page_number)?,
    })
}

#[tauri::command]
/// 获取漫画系列详情及其全部可阅读分卷。
pub(crate) async fn get_comic_series(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    series_title: String,
) -> Result<ComicSeriesDetail> {
    if series_title.trim().is_empty() {
        return Err(AppError::invalid_input("漫画系列标题不能为空"));
    }
    let list = client.find_comic_series_book(&series_title).await?;
    let first = array(&list, "Data")
        .first()
        .ok_or_else(|| AppError::protocol("漫画系列不含可阅读分卷"))?;
    let first_id = positive_book_id(first)?;
    let response = client.get_comic_info(first_id).await?;
    let mut books = Vec::new();
    for id in series_book_ids(&response)? {
        let book = if id == first_id {
            book_detail(&response)?
        } else {
            book_detail(&client.get_comic_info(id).await?)?
        };
        books.push(book);
    }
    let detail = series_detail(&response, series_title, books)?;
    for comic_book in &detail.books {
        cache
            .store_cache(
                &cache.comic_chapters,
                comic_book.id.clone(),
                comic_book
                    .chapters
                    .iter()
                    .map(|chapter| chapter.id.clone())
                    .collect(),
            )
            .await;
    }
    Ok(detail)
}

#[tauri::command]
/// 获取单个漫画分卷详情与章节目录。
pub(crate) async fn get_comic_book(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    book_id: String,
) -> Result<ComicBook> {
    let response = client.get_comic_info(parse_id(&book_id)?).await?;
    let comic_book = book_detail(&response)?;
    cache
        .store_cache(
            &cache.comic_chapters,
            comic_book.id.clone(),
            comic_book
                .chapters
                .iter()
                .map(|chapter| chapter.id.clone())
                .collect(),
        )
        .await;
    Ok(comic_book)
}

#[tauri::command]
/// 保存漫画章节的阅读页码。
pub(crate) async fn save_comic_read_position(
    client: State<'_, OfficialClient>,
    comic_id: String,
    chapter_id: String,
    page: i64,
) -> Result<()> {
    if page < 1 {
        return Err(AppError::invalid_input("漫画页码必须大于 0"));
    }
    client
        .save_comic_position(parse_id(&comic_id)?, parse_id(&chapter_id)?, page)
        .await
}

#[tauri::command]
/// 获取指定页面所在批次，并在后台预加载后续阅读内容。
pub(crate) async fn get_comic_chapter_pages(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    comic_id: String,
    chapter_id: String,
    page_index: i64,
) -> Result<Arc<ComicChapterPageBatch>> {
    if page_index < 0 {
        return Err(AppError::invalid_input("漫画页码不能小于 0"));
    }
    let start_index = page_index.div_euclid(12) * 12;
    let pages = load_comic_chapter_pages_at(&client, &cache, &chapter_id, start_index).await?;
    let next_chapter_id = if start_index + 12 >= pages.page_count {
        cache
            .comic_chapters
            .get(&comic_id)
            .await
            .and_then(|chapter_ids| {
                chapter_ids
                    .iter()
                    .position(|id| id == &chapter_id)
                    .and_then(|index| chapter_ids.get(index + 1))
                    .cloned()
            })
    } else {
        None
    };
    preload_comic_read_ahead(
        client.inner().clone(),
        cache.inner().clone(),
        chapter_id.clone(),
        start_index,
        pages.page_count,
        next_chapter_id,
    );
    Ok(pages)
}

async fn load_comic_chapter_pages_at(
    client: &OfficialClient,
    cache: &AppCache,
    chapter_id: &str,
    start_index: i64,
) -> Result<Arc<ComicChapterPageBatch>> {
    cache
        .load_cache(
            &cache.comic_pages,
            ComicPageBatchKey::new(chapter_id.into(), start_index),
            async {
                let response = client
                    .get_comic_content(parse_id(chapter_id)?, start_index)
                    .await?;
                page_batch(chapter_id.into(), &response, start_index)
            },
        )
        .await
}

/// 在后台预加载当前章节后两批页面；当前批为末批时改为预加载下一章首批。
fn preload_comic_read_ahead(
    client: OfficialClient,
    cache: AppCache,
    chapter_id: String,
    start_index: i64,
    page_count: i64,
    next_chapter_id: Option<String>,
) {
    tauri::async_runtime::spawn(async move {
        let next_start_index = start_index + 12;
        if next_start_index >= page_count {
            if let Some(next_chapter_id) = next_chapter_id {
                let _ = load_comic_chapter_pages_at(&client, &cache, &next_chapter_id, 0).await;
            }
            return;
        }

        for start_index in [next_start_index, next_start_index + 12] {
            if start_index >= page_count {
                break;
            }
            let _ = load_comic_chapter_pages_at(&client, &cache, &chapter_id, start_index).await;
        }
    });
}
