use std::sync::Arc;

use tauri::State;

use crate::{
    api::{
        cache::{get_neighbor_ids, AppCache, NovelChapterKey},
        OfficialClient,
    },
    dto::{
        common::{ListKey, Order, PaginatedList, SearchMode},
        novel::{NovelChapterContent, NovelDetail, NovelSummary},
    },
    error::{AppError, Result},
    mapping::novel::{
        chapter_content, page, reader_detail, summaries as novel_summaries,
        summary as novel_summary,
    },
};

use super::validation::{parse_id, validate_page_size};

#[tauri::command]
/// 获取指定排序方式的小说榜单。
pub(crate) async fn list_novels(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    order: Order,
    page_number: Option<i64>,
    page_size: i64,
) -> Result<Arc<Vec<NovelSummary>>> {
    let page_number = page_number.unwrap_or(1);
    let page_size = validate_page_size(page_size)?;
    let key = ListKey::new(order, page_number, page_size);
    cache
        .load_cache(&cache.novel_list, key, async {
            let response = client.list_novels(page_number, page_size, order).await?;
            Ok(novel_summaries(&response))
        })
        .await
}

#[tauri::command]
/// 获取指定天数范围的小说排行。
pub(crate) async fn rank_novels(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    days: i64,
) -> Result<Arc<Vec<NovelSummary>>> {
    cache
        .load_cache(&cache.novel_rank, days, async {
            Ok(client
                .get_novel_rank(days)
                .await?
                .as_array()
                .map(|items| items.iter().map(novel_summary).collect())
                .unwrap_or_default())
        })
        .await
}

#[tauri::command]
/// 按作品名、作者或标签搜索小说。
pub(crate) async fn search_novels(
    client: State<'_, OfficialClient>,
    query: String,
    page_number: Option<i64>,
    page_size: i64,
    mode: SearchMode,
) -> Result<PaginatedList<NovelSummary>> {
    let page_number = page_number.unwrap_or(1);
    let response = client
        .search_novels(query, page_number, validate_page_size(page_size)?, mode)
        .await?;
    page(&response, page_number)
}

#[tauri::command]
/// 获取小说阅读器概览。
pub(crate) async fn get_reader_overview(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    book_id: String,
) -> Result<NovelDetail> {
    let response = client.get_novel_info(parse_id(&book_id)?).await?;
    let detail = reader_detail(&response)?;
    cache
        .store_cache(
            &cache.novel_chapters,
            book_id.clone(),
            detail
                .chapters
                .iter()
                .map(|chapter| chapter.id.clone())
                .collect(),
        )
        .await;
    Ok(detail)
}

#[tauri::command]
/// 获取小说章节内容并按需预加载后续章节。
pub(crate) async fn get_reader_document(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    book_id: String,
    document_id: String,
    convert: Option<String>,
) -> Result<Arc<NovelChapterContent>> {
    let convert = parse_convert(convert)?;
    let document = load_reader_document(&client, &cache, &book_id, &document_id, convert).await?;
    let chapter_ids = cache
        .load_cache(&cache.novel_chapters, book_id.clone(), async {
            let response = client.get_novel_info(parse_id(&book_id)?).await?;
            Ok(reader_detail(&response)?
                .chapters
                .into_iter()
                .map(|chapter| chapter.id)
                .collect())
        })
        .await?;
    preload_novel_read_ahead(
        client.inner().clone(),
        cache.inner().clone(),
        book_id,
        chapter_ids,
        document_id,
        convert,
    );
    Ok(document)
}

/// 从缓存或官方服务加载小说章节内容。
async fn load_reader_document(
    client: &OfficialClient,
    cache: &AppCache,
    book_id: &str,
    document_id: &str,
    convert: Option<&str>,
) -> Result<Arc<NovelChapterContent>> {
    cache
        .load_cache(
            &cache.novel_pages,
            NovelChapterKey::new(book_id, document_id, convert),
            async {
                let response = client
                    .get_novel_content(parse_id(book_id)?, parse_id(document_id)?, convert)
                    .await?;
                chapter_content(&response, document_id.to_string())
            },
        )
        .await
}

/// 校验繁简转换选项。
fn parse_convert(convert: Option<String>) -> Result<Option<&'static str>> {
    match convert.as_deref() {
        None => Ok(None),
        Some("t2s") => Ok(Some("t2s")),
        Some("s2t") => Ok(Some("s2t")),
        Some(_) => Err(AppError::invalid_input("文字转换选项必须是 t2s 或 s2t")),
    }
}

/// 在后台预加载当前章节之后的小说内容。
fn preload_novel_read_ahead(
    client: OfficialClient,
    cache: AppCache,
    book_id: String,
    chapter_ids: Arc<Vec<String>>,
    current_chapter_id: String,
    convert: Option<&'static str>,
) {
    let neighbors = get_neighbor_ids(&chapter_ids, &current_chapter_id);
    tauri::async_runtime::spawn(async move {
        for chapter_id in neighbors {
            let _ = load_reader_document(&client, &cache, &book_id, &chapter_id, convert).await;
        }
    });
}

#[tauri::command]
/// 保存小说阅读位置。
pub(crate) async fn save_read_position(
    client: State<'_, OfficialClient>,
    book_id: String,
    chapter_id: String,
    xpath: String,
) -> Result<()> {
    client
        .save_novel_position(parse_id(&book_id)?, parse_id(&chapter_id)?, xpath)
        .await
}
