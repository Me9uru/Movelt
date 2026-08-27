use std::sync::Arc;

use serde_json::Value;
use tauri::State;

use crate::{
    api::{
        cache::{AppCache, ComicPageBatchKey},
        OfficialClient,
    },
    dto::{
        comic::{
            ComicBook, ComicChapterPageBatch, ComicChapterSummary, ComicSeriesDetail, ComicSummary,
        },
        common::{ListKey, Order, PaginatedList, SearchMode},
    },
    error::{AppError, Result},
};

use super::{
    adapter::{
        array, number, optional_html, optional_string, pagination, parse_id, position, string,
        validate_page_size,
    },
    bookshelf::{books_for_ids, is_kind, set_shelf, shelf_items},
};

/// 将官方漫画数据映射为应用摘要。
fn comic(value: &Value) -> ComicSummary {
    ComicSummary {
        // 漫画列表按系列聚合。详情接口需要的是系列标题而不是分卷数字 ID。
        id: string(value, "Title"),
        book_id: None,
        title: string(value, "Title"),
        cover_url: optional_string(value, "Cover"),
        author: optional_string(value, "Author"),
    }
}

fn title_matches_query(title: &str, query: &str) -> bool {
    query.is_empty() || title.to_lowercase().contains(query)
}

/// 从官方漫画图片数组中提取 URL。
///
/// 当前官方接口返回 `string[]`，旧响应则可能为 `{ Url: string }[]`；
/// 保留两种形式的兼容性，避免将有效页面静默过滤掉。
fn comic_page_urls(chapter: &Value) -> Vec<String> {
    array(chapter, "Images")
        .iter()
        .filter_map(|image| {
            image
                .as_str()
                .map(str::to_owned)
                .or_else(|| optional_string(image, "Url"))
        })
        .collect()
}

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
            .map(comic)
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
        items: array(&response, "Data").iter().map(comic).collect(),
        pagination: pagination(&response, page_number)?,
    })
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
    Ok(books_for_ids(client, ids, Some("Comic"))
        .await?
        .iter()
        .map(|book| ComicSummary {
            // 漫画详情接口按系列标题查询，书架入口也必须使用该值。
            id: book.title.clone(),
            book_id: Some(book.id.clone()),
            title: book.title.clone(),
            cover_url: book.cover_url.clone(),
            author: book.author.clone(),
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

#[tauri::command]
/// 获取漫画系列详情及其全部可阅读分卷。
pub(crate) async fn get_comic_series(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    series_title: String,
) -> Result<ComicSeriesDetail> {
    let response = client.get_comic_series_info(&series_title).await?;
    let series = response
        .get("Series")
        .ok_or_else(|| AppError::protocol("漫画系列响应缺少 Series"))?;
    let books = array(&response, "Books");
    if books.is_empty() {
        return Err(AppError::protocol("漫画系列不含可阅读分卷"));
    }
    let mut comic_books = Vec::with_capacity(books.len());
    for book in books {
        let chapter_values = comic_chapter_values(book);
        let comic_book = ComicBook {
            id: number(book, "Id").to_string(),
            title: string(book, "Title"),
            read_position: position(book.get("ReadPosition")),
            chapters: chapter_values.iter().map(comic_chapter_summary).collect(),
        };
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
        comic_books.push(comic_book);
    }
    Ok(ComicSeriesDetail {
        summary: ComicSummary {
            id: series_title,
            book_id: None,
            title: string(series, "Title"),
            cover_url: optional_string(series, "Cover"),
            author: optional_string(series, "Author"),
        },
        description: optional_html(series, "Introduction"),
        genre: series
            .pointer("/Extra/classification/tags")
            .and_then(Value::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        status: optional_string(series, "LastUpdatedChapter").unwrap_or_default(),
        books: comic_books,
    })
}

#[tauri::command]
/// 获取单个漫画分卷详情与章节目录。
pub(crate) async fn get_comic_book(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    book_id: String,
) -> Result<ComicBook> {
    let response = client.get_comic_info(parse_id(&book_id)?).await?;
    let book = response
        .get("Book")
        .ok_or_else(|| AppError::protocol("漫画分卷响应缺少 Book"))?;
    let chapter_values = comic_chapter_values(book);
    let comic_book = ComicBook {
        id: number(book, "Id").to_string(),
        title: string(book, "Title"),
        read_position: position(response.get("ReadPosition")),
        chapters: chapter_values.iter().map(comic_chapter_summary).collect(),
    };
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

/// 返回官方漫画分卷中的章节数组，兼容两种响应字段名。
fn comic_chapter_values(book: &Value) -> &[Value] {
    let chapters = array(book, "Chapters");
    if chapters.is_empty() {
        array(book, "Chapter")
    } else {
        chapters
    }
}

/// 将官方漫画章节映射为应用章节摘要。
fn comic_chapter_summary(chapter: &Value) -> ComicChapterSummary {
    ComicChapterSummary {
        id: number(chapter, "Id").to_string(),
        title: string(chapter, "Title"),
        sequence: number(chapter, "SortNum"),
        page_count: number(chapter, "PageCount"),
    }
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
                comic_page_batch(chapter_id.into(), response, start_index)
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

fn comic_page_batch(
    chapter_id: String,
    response: Value,
    start_index: i64,
) -> Result<ComicChapterPageBatch> {
    let chapter = response
        .get("Chapter")
        .ok_or_else(|| AppError::protocol("漫画页面响应缺少 Chapter"))?;
    Ok(ComicChapterPageBatch {
        chapter_id,
        start_index,
        page_urls: comic_page_urls(chapter),
        page_count: number(chapter, "Total"),
        read_position: position(response.get("ReadPosition")),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{comic_page_urls, title_matches_query};

    #[test]
    fn maps_current_string_image_urls() {
        let chapter = json!({"Images": ["https://images.example/1.webp"]});

        assert_eq!(comic_page_urls(&chapter), ["https://images.example/1.webp"]);
    }

    #[test]
    fn keeps_legacy_object_image_urls_compatible() {
        let chapter = json!({"Images": [{"Url": "https://images.example/1.webp"}]});

        assert_eq!(comic_page_urls(&chapter), ["https://images.example/1.webp"]);
    }

    #[test]
    fn filters_comic_bookshelf_titles_case_insensitively() {
        assert!(title_matches_query("My Favorite Comic", "favorite"));
        assert!(!title_matches_query("My Favorite Comic", "author"));
        assert!(title_matches_query("My Favorite Comic", ""));
    }
}
