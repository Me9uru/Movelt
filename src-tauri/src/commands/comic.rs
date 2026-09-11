use std::sync::Arc;

use futures_util::{stream, StreamExt, TryStreamExt};
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

use super::validation::{parse_id, validate_page_number, validate_page_size};
use crate::mapping::value::array;

const SERIES_DETAIL_CONCURRENCY: usize = 4;

#[tauri::command]
/// 浏览漫画列表。
pub(crate) async fn list_comics(
    client: State<'_, OfficialClient>,
    page_number: i64,
    page_size: i64,
    order: Order,
) -> Result<Arc<Vec<ComicSummary>>> {
    let client = client.scoped().await;
    let cache = client.cache();
    let page_size = validate_page_size(page_size)?;
    let page_number = validate_page_number(page_number)?;
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
    let client = client.scoped().await;
    let page_number = validate_page_number(page_number)?;
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
    series_title: String,
) -> Result<ComicSeriesDetail> {
    let client = client.scoped().await;
    let cache = client.cache();
    if series_title.trim().is_empty() {
        return Err(AppError::invalid_input("漫画系列标题不能为空"));
    }
    let list = client.find_comic_series_book(&series_title).await?;
    let first = array(&list, "Data")
        .first()
        .ok_or_else(|| AppError::protocol("漫画系列不含可阅读分卷"))?;
    let first_id = positive_book_id(first)?;
    let response = client.get_comic_info(first_id).await?;
    let books = load_series_books(&client, first_id, &response).await?;
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

async fn load_series_books(
    client: &OfficialClient,
    first_id: i64,
    response: &serde_json::Value,
) -> Result<Vec<ComicBook>> {
    // 并行获取目录，同时保留官方系列顺序并复用已经获取的代表卷。
    stream::iter(series_book_ids(response)?)
        .map(|id| async move {
            if id == first_id {
                book_detail(response)
            } else {
                book_detail(&client.get_comic_info(id).await?)
            }
        })
        .buffered(SERIES_DETAIL_CONCURRENCY)
        .try_collect()
        .await
}

#[tauri::command]
/// 获取单个漫画分卷详情与章节目录。
pub(crate) async fn get_comic_book(
    client: State<'_, OfficialClient>,
    book_id: String,
) -> Result<ComicBook> {
    let client = client.scoped().await;
    let cache = client.cache();
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
    let client = client.scoped().await;
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
    comic_id: String,
    chapter_id: String,
    page_index: i64,
) -> Result<Arc<ComicChapterPageBatch>> {
    let client = client.scoped().await;
    let cache = client.cache();
    if page_index < 0 || page_index.checked_add(24).is_none() {
        return Err(AppError::invalid_input("漫画页码超出有效范围"));
    }
    let start_index = page_index.div_euclid(12) * 12;
    let pages = load_comic_chapter_pages_at(&client, cache, &chapter_id, start_index).await?;
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
        client.clone(),
        cache.clone(),
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

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex,
        },
        time::Duration,
    };

    use serde_json::{json, Value};

    use super::{load_series_books, SERIES_DETAIL_CONCURRENCY};
    use crate::{api::mock_client, error::AppError};

    fn detail(id: i64) -> Value {
        json!({
            "Book": { "Id": id, "Type": "Comic", "Title": "卷", "Chapters": [] },
            "Series": (1..=10).map(|id| json!({ "Id": id })).collect::<Vec<_>>()
        })
    }

    #[tokio::test(start_paused = true)]
    async fn loads_series_concurrently_in_order_without_refetching_representative() {
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let calls = Arc::new(Mutex::new(Vec::new()));
        let client = mock_client({
            let active = active.clone();
            let peak = peak.clone();
            let calls = calls.clone();
            move |method, payload| {
                let active = active.clone();
                let peak = peak.clone();
                let calls = calls.clone();
                async move {
                    assert_eq!(method, "GetBookInfo");
                    let id = payload["Id"].as_i64().unwrap();
                    calls.lock().unwrap().push(id);
                    let count = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(count, Ordering::SeqCst);
                    // 后面的卷先完成，返回目录仍须保持官方顺序。
                    tokio::time::sleep(Duration::from_millis((11 - id) as u64 * 10)).await;
                    active.fetch_sub(1, Ordering::SeqCst);
                    Ok(detail(id))
                }
            }
        })
        .await;
        let books = load_series_books(&client, 1, &detail(1)).await.unwrap();
        assert_eq!(
            books.iter().map(|book| book.id.clone()).collect::<Vec<_>>(),
            (1..=10).map(|id| id.to_string()).collect::<Vec<_>>()
        );
        assert_eq!(peak.load(Ordering::SeqCst), SERIES_DETAIL_CONCURRENCY);
        let mut calls = calls.lock().unwrap().clone();
        calls.sort_unstable();
        assert_eq!(calls, (2..=10).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn propagates_volume_errors_instead_of_returning_partial_series() {
        let client = mock_client(|_, _| async { Err(AppError::protocol("invalid volume")) }).await;
        assert!(matches!(
            load_series_books(&client, 1, &detail(1)).await,
            Err(AppError::UpstreamProtocol { .. })
        ));
    }
}
