use std::sync::Arc;

use tauri::State;
use url::Url;

use crate::{
    api::{
        cache::{get_neighbor_ids, AppCache, NovelChapterKey},
        OfficialClient,
    },
    dto::{
        common::{ListKey, Order, PaginatedList, SearchMode},
        novel::{NovelChapterContent, NovelChapterSummary, NovelDetail, NovelSummary},
    },
    error::{AppError, Result},
};

use super::adapter::{
    array, number, optional_html, optional_string, pagination, parse_id, position, string,
    validate_page_size,
};

/// 将官方小说数据映射为应用摘要。
pub(super) fn novel(value: &serde_json::Value) -> NovelSummary {
    NovelSummary {
        source: "lightnovel".into(),
        id: number(value, "Id").to_string(),
        title: string(value, "Title"),
        cover_url: optional_string(value, "Cover"),
        author: optional_string(value, "Author")
            .or_else(|| optional_string(value, "Arthur"))
            .or_else(|| optional_string(value, "UserName")),
        status: optional_string(value, "LastUpdatedChapter")
            .or_else(|| optional_string(value, "SeriesTitle")),
        updated_at: optional_string(value, "LastUpdatedAt"),
        description: optional_html(value, "Introduction"),
        tags: value
            .pointer("/Extra/classification/tags")
            .and_then(serde_json::Value::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// 将官方小说分页结果映射为发现页 DTO。
fn page(value: serde_json::Value) -> Result<PaginatedList<NovelSummary>> {
    Ok(PaginatedList {
        items: novel_summaries(&value),
        pagination: pagination(&value, 1)?,
    })
}

fn novel_summaries(value: &serde_json::Value) -> Vec<NovelSummary> {
    value
        .get("Data")
        .and_then(serde_json::Value::as_array)
        .map(|items| items.iter().map(novel).collect())
        .unwrap_or_default()
}

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
                .map(|items| items.iter().map(novel).collect())
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
    page(
        client
            .search_novels(
                query,
                page_number.unwrap_or(1),
                validate_page_size(page_size)?,
                mode,
            )
            .await?,
    )
}

#[tauri::command]
/// 获取小说阅读器概览。
pub(crate) async fn get_reader_overview(
    client: State<'_, OfficialClient>,
    cache: State<'_, AppCache>,
    book_id: String,
) -> Result<NovelDetail> {
    let response = client.get_novel_info(parse_id(&book_id)?).await?;
    let book = response
        .get("Book")
        .ok_or_else(|| AppError::protocol("小说详情响应缺少 Book"))?;
    let mut read_position = position(response.get("ReadPosition"));
    let chapters = array(book, "Chapter");
    if let Some(position) = &mut read_position {
        if let Some(index) = chapters
            .iter()
            .position(|chapter| number(chapter, "Id").to_string() == position.chapter_id)
        {
            position.chapter_id = (index + 1).to_string();
        }
    }
    cache
        .store_cache(
            &cache.novel_chapters,
            book_id.clone(),
            (1..=chapters.len())
                .map(|index| index.to_string())
                .collect(),
        )
        .await;
    Ok(NovelDetail {
        summary: novel(book),
        chapters: chapters
            .iter()
            .enumerate()
            .map(|(index, chapter)| NovelChapterSummary {
                id: (index + 1).to_string(),
                title: string(chapter, "Title"),
                sequence: (index + 1) as i64,
            })
            .collect(),
        read_position,
    })
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
            let book = response
                .get("Book")
                .ok_or_else(|| AppError::protocol("小说详情响应缺少 Book"))?;
            Ok((1..=array(book, "Chapter").len())
                .map(|index| index.to_string())
                .collect::<Vec<_>>())
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
                let chapter = response
                    .get("Chapter")
                    .ok_or_else(|| AppError::protocol("小说章节响应缺少 Chapter"))?;
                Ok(NovelChapterContent {
                    chapter_id: document_id.to_string(),
                    server_chapter_id: number(chapter, "Id").to_string(),
                    title: string(chapter, "Title"),
                    html: sanitize_chapter_html(&string(chapter, "Content")),
                    font_url: chapter_font_url(chapter),
                    read_position: position(response.get("ReadPosition")),
                })
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

/// 保留脚注标识，同时清洗不安全的章节 HTML。
fn sanitize_chapter_html(content: &str) -> String {
    let mut sanitizer = ammonia::Builder::default();
    sanitizer.add_generic_attributes(["class", "id"]);
    sanitizer.clean(content).to_string()
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

/// 读取并校验章节字体地址。
fn chapter_font_url(chapter: &serde_json::Value) -> Option<String> {
    let font = optional_string(chapter, "Font")?;
    if font.starts_with('/') {
        return Some(format!("https://api.lightnovel.life{font}"));
    }

    let url = Url::parse(&font).ok()?;
    let trusted_host = matches!(
        url.host_str(),
        Some("api.lightnovel.life" | "cf-api.lightnovel.life" | "img.lightnovel.life")
    );
    (url.scheme() == "https" && trusted_host).then_some(font)
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

#[cfg(test)]
mod tests {
    use super::sanitize_chapter_html;

    #[test]
    fn preserves_official_footnote_markup() {
        let html = sanitize_chapter_html(
            r##"<p>正文<a class="duokan-footnote" href="#note-1"><img class="footnote" src="/note.png"></a></p><div id="note-1">注释</div><script>alert(1)</script>"##,
        );

        assert!(html.contains(r#"class="duokan-footnote""#));
        assert!(html.contains(r#"class="footnote""#));
        assert!(html.contains(r#"id="note-1""#));
        assert!(html.contains(r##"href="#note-1""##));
        assert!(!html.contains("script"));
    }
}
