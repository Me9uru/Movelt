use tauri::State;
use url::Url;

use crate::{
    api::OfficialClient,
    dto::novel::{
        ChapterSummary, DiscoveryList, NovelOverview, NovelSummary, ReaderDocument, Volume,
    },
    error::{AppError, Result},
    reader_cache::{neighbor_ids, ReaderCache},
};

use super::adapter::{
    array, number, object, optional_html, optional_string, parse_id, position, string,
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
fn page(value: serde_json::Value) -> Result<DiscoveryList> {
    let raw = object(&value)?;
    let current = raw
        .get("Page")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(1);
    let last = raw
        .get("TotalPages")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(1);
    Ok(DiscoveryList {
        items: raw
            .get("Data")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().map(novel).collect())
            .unwrap_or_default(),
        pagination: crate::dto::novel::Pagination {
            page: current,
            previous: (current > 1).then_some(current - 1),
            next: (current < last).then_some(current + 1),
            first: 1,
            last,
        },
    })
}

#[tauri::command]
/// 获取最新小说列表。
pub(crate) async fn get_latest(
    client: State<'_, OfficialClient>,
    page_number: Option<i64>,
) -> Result<DiscoveryList> {
    page(client.latest_novels(page_number.unwrap_or(1)).await?)
}

#[tauri::command]
/// 获取指定排序方式的小说榜单。
pub(crate) async fn get_ranking(
    client: State<'_, OfficialClient>,
    sort: String,
    page_number: Option<i64>,
) -> Result<DiscoveryList> {
    page(client.ranked_novels(sort, page_number.unwrap_or(1)).await?)
}

#[tauri::command]
/// 获取指定天数范围的小说排行。
pub(crate) async fn get_rank(
    client: State<'_, OfficialClient>,
    days: i64,
) -> Result<Vec<NovelSummary>> {
    Ok(client
        .novel_rank(days)
        .await?
        .as_array()
        .map(|items| items.iter().map(novel).collect())
        .unwrap_or_default())
}

#[tauri::command]
/// 按关键词或标签搜索小说。
pub(crate) async fn search_novels(
    client: State<'_, OfficialClient>,
    query: String,
    page_number: Option<i64>,
    tags: bool,
) -> Result<DiscoveryList> {
    page(
        client
            .search_novels(query, page_number.unwrap_or(1), tags)
            .await?,
    )
}

#[tauri::command]
/// 获取小说阅读器概览。
pub(crate) async fn get_reader_overview(
    client: State<'_, OfficialClient>,
    cache: State<'_, ReaderCache>,
    book_id: String,
) -> Result<NovelOverview> {
    let response = client.novel_info(parse_id(&book_id)?).await?;
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
    let chapter_ids = (1..=chapters.len())
        .map(|index| index.to_string())
        .collect::<Vec<_>>();
    cache.store_novel_chapters(book_id.clone(), chapter_ids);
    Ok(NovelOverview {
        detail: novel(book),
        volumes: vec![Volume {
            title: "章节".into(),
            chapters: chapters
                .iter()
                .enumerate()
                .map(|(index, chapter)| ChapterSummary {
                    id: (index + 1).to_string(),
                    title: string(chapter, "Title"),
                })
                .collect(),
            sections: vec![],
        }],
        read_position,
    })
}

#[tauri::command]
/// 获取小说章节内容并按需预加载后续章节。
pub(crate) async fn get_reader_document(
    client: State<'_, OfficialClient>,
    cache: State<'_, ReaderCache>,
    book_id: String,
    document_id: String,
    convert: Option<String>,
) -> Result<ReaderDocument> {
    let convert = parse_convert(convert)?;
    let document = load_reader_document(&client, &cache, &book_id, &document_id, convert).await?;
    let chapter_ids = cache.novel_chapters(&book_id).unwrap_or_default();
    preload_novel_neighbors(
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
    cache: &ReaderCache,
    book_id: &str,
    document_id: &str,
    convert: Option<&str>,
) -> Result<ReaderDocument> {
    if let Some(document) = cache.novel(book_id, document_id, convert) {
        return Ok(document);
    }
    let response = client
        .novel_content(parse_id(book_id)?, parse_id(document_id)?, convert)
        .await?;
    let chapter = response
        .get("Chapter")
        .ok_or_else(|| AppError::protocol("小说章节响应缺少 Chapter"))?;
    let document = ReaderDocument {
        id: format!("{book_id}:{document_id}"),
        book_id: book_id.to_string(),
        chapter_id: document_id.to_string(),
        server_chapter_id: number(chapter, "Id").to_string(),
        title: string(chapter, "Title"),
        html: sanitize_chapter_html(&string(chapter, "Content")),
        font_url: chapter_font_url(chapter),
        read_position: position(response.get("ReadPosition")),
    };
    cache.store_novel(document.clone(), convert);
    Ok(document)
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
fn preload_novel_neighbors(
    client: OfficialClient,
    cache: ReaderCache,
    book_id: String,
    chapter_ids: Vec<String>,
    current_chapter_id: String,
    convert: Option<&'static str>,
) {
    let neighbors = neighbor_ids(&chapter_ids, &current_chapter_id);
    tauri::async_runtime::spawn(async move {
        for chapter_id in neighbors {
            if cache.novel(&book_id, &chapter_id, convert).is_none() {
                let _ = load_reader_document(&client, &cache, &book_id, &chapter_id, convert).await;
            }
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
