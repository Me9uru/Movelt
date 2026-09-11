use serde_json::Value;
use url::Url;

use crate::{
    dto::{
        common::PaginatedList,
        novel::{NovelChapterContent, NovelChapterSummary, NovelDetail, NovelSummary},
    },
    error::{AppError, Result},
    mapping::{
        common::{pagination, read_position},
        value::{
            array, number, optional_html, optional_string, required_id, required_string, string,
        },
    },
};

/// 将官方小说数据映射为应用摘要。
pub(crate) fn summary(value: &Value) -> NovelSummary {
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
            .and_then(Value::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// 将官方小说分页结果映射为发现页 DTO。
pub(crate) fn page(value: &Value, default_page: i64) -> Result<PaginatedList<NovelSummary>> {
    Ok(PaginatedList {
        items: summaries(value),
        pagination: pagination(value, default_page)?,
    })
}

/// 将官方小说列表映射为应用摘要。
pub(crate) fn summaries(value: &Value) -> Vec<NovelSummary> {
    array(value, "Data").iter().map(summary).collect()
}

/// 将官方小说详情响应映射为阅读器概览。
pub(crate) fn reader_detail(value: &Value) -> Result<NovelDetail> {
    let (book, mut chapters) = super::book::detail(value, "Novel")?;
    chapters.sort_by_key(|chapter| chapter.sort_num);
    let mut read_position = read_position(value.get("ReadPosition"));
    if let Some(position) = &mut read_position {
        if let Some(chapter) = chapters
            .iter()
            .find(|chapter| chapter.id.to_string() == position.chapter_id)
        {
            position.chapter_id = chapter.sort_num.to_string();
        }
    }

    Ok(NovelDetail {
        summary: summary(book),
        chapters: chapters
            .iter()
            .map(|chapter| NovelChapterSummary {
                id: chapter.sort_num.to_string(),
                title: chapter.title.clone(),
                sequence: chapter.sort_num,
            })
            .collect(),
        read_position,
    })
}

/// 将官方小说章节响应映射为阅读内容。
pub(crate) fn chapter_content(value: &Value, document_id: String) -> Result<NovelChapterContent> {
    let chapter = value
        .get("Chapter")
        .ok_or_else(|| AppError::protocol("小说章节响应缺少 Chapter"))?;
    Ok(NovelChapterContent {
        chapter_id: document_id,
        server_chapter_id: required_id(chapter, "Id")?.to_string(),
        title: required_string(chapter, "Title")?,
        html: sanitize_chapter_html(&required_string(chapter, "Content")?),
        font_url: chapter_font_url(chapter),
        read_position: read_position(value.get("ReadPosition")),
    })
}

/// 保留脚注标识，同时清洗不安全的章节 HTML。
pub(crate) fn sanitize_chapter_html(content: &str) -> String {
    let mut sanitizer = ammonia::Builder::default();
    sanitizer.add_generic_attributes(["class", "id"]);
    sanitizer.clean(content).to_string()
}

fn chapter_font_url(chapter: &Value) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{chapter_content, reader_detail, sanitize_chapter_html, summary};

    #[test]
    fn rejects_malformed_content_but_accepts_an_empty_chapter() {
        for chapter in [
            json!(null),
            json!({}),
            json!({"Id": 1, "Title": "一", "Content": null}),
        ] {
            assert!(chapter_content(&json!({"Chapter": chapter}), "1".into()).is_err());
        }
        let content = chapter_content(
            &json!({"Chapter": {"Id": 1, "Title": "一", "Content": ""}}),
            "1".into(),
        )
        .unwrap();
        assert!(content.html.is_empty());
    }

    #[test]
    fn maps_unified_chapters_and_resume_position_by_sort_num() {
        let value = json!({
            "Book": {"Id": 42, "Type": "Novel", "Title": "小说", "Author": "作者",
                "Chapters": [
                    {"Id": 102, "SortNum": 5, "Title": "后章", "PageCount": 0},
                    {"Id": 101, "SortNum": 2, "Title": "前章", "PageCount": 0}
                ]},
            "SeriesTitle": "系列", "Series": [],
            "ReadPosition": {"ChapterId": 102, "Position": "/p[3]"}
        });
        let detail = reader_detail(&value).unwrap();
        assert_eq!(
            detail
                .chapters
                .iter()
                .map(|c| c.id.as_str())
                .collect::<Vec<_>>(),
            ["2", "5"]
        );
        assert_eq!(detail.chapters[1].sequence, 5);
        assert_eq!(detail.read_position.unwrap().chapter_id, "5");
        assert_eq!(detail.summary.author.as_deref(), Some("作者"));
    }

    #[test]
    fn rejects_wrong_domain_and_missing_new_catalogue() {
        assert!(reader_detail(&json!({"Book": {"Type": "Comic", "Chapters": []}})).is_err());
        assert!(reader_detail(&json!({"Book": {"Type": "Novel", "Chapter": []}})).is_err());
        assert!(
            reader_detail(&json!({"Book": {"Type": "Novel", "Chapters": [
                {"Id": 1, "SortNum": 0, "Title": "错误", "PageCount": 0}
            ]}}))
            .is_err()
        );
    }

    #[test]
    fn maps_summary_with_official_field_fallbacks() {
        let value = json!({
            "Id": 42,
            "Title": "Novel title",
            "Arthur": "Fallback author",
            "SeriesTitle": "Series title",
            "Introduction": "<p>Introduction</p>",
            "Extra": {"classification": {"tags": ["Fantasy", "Adventure"]}},
        });

        let mapped = summary(&value);

        assert_eq!(mapped.id, "42");
        assert_eq!(mapped.author.as_deref(), Some("Fallback author"));
        assert_eq!(mapped.status.as_deref(), Some("Series title"));
        assert_eq!(mapped.tags, ["Fantasy", "Adventure"]);
    }

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
