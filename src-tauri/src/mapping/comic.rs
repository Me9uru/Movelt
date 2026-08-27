use serde_json::Value;

use crate::{
    dto::comic::{
        ComicBook, ComicChapterPageBatch, ComicChapterSummary, ComicSeriesDetail, ComicSummary,
    },
    error::{AppError, Result},
    mapping::{
        common::read_position,
        value::{array, number, optional_html, optional_string, string},
    },
};

/// 将官方漫画系列数据映射为应用摘要。
pub(crate) fn summary(value: &Value) -> ComicSummary {
    ComicSummary {
        // 漫画详情接口需要系列标题而不是分卷数字 ID。
        id: string(value, "Title"),
        book_id: None,
        title: string(value, "Title"),
        cover_url: optional_string(value, "Cover"),
        author: optional_string(value, "Author"),
    }
}

/// 将官方漫画书架批量查询结果映射为应用摘要。
pub(crate) fn bookshelf_summary(value: &Value) -> ComicSummary {
    ComicSummary {
        id: string(value, "Title"),
        book_id: Some(number(value, "Id").to_string()),
        title: string(value, "Title"),
        cover_url: optional_string(value, "Cover"),
        author: optional_string(value, "Author"),
    }
}

/// 将官方漫画系列详情响应映射为应用 DTO。
pub(crate) fn series_detail(value: &Value, series_title: String) -> Result<ComicSeriesDetail> {
    let series = value
        .get("Series")
        .ok_or_else(|| AppError::protocol("漫画系列响应缺少 Series"))?;
    let books = array(value, "Books");
    if books.is_empty() {
        return Err(AppError::protocol("漫画系列不含可阅读分卷"));
    }

    Ok(ComicSeriesDetail {
        summary: ComicSummary {
            id: series_title,
            ..summary(series)
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
        books: books
            .iter()
            .map(|book| book_summary(book, book.get("ReadPosition")))
            .collect(),
    })
}

/// 将官方漫画分卷详情响应映射为应用 DTO。
pub(crate) fn book_detail(value: &Value) -> Result<ComicBook> {
    let book = value
        .get("Book")
        .ok_or_else(|| AppError::protocol("漫画分卷响应缺少 Book"))?;
    Ok(book_summary(book, value.get("ReadPosition")))
}

/// 将官方漫画章节页面响应映射为应用 DTO。
pub(crate) fn page_batch(
    chapter_id: String,
    value: &Value,
    start_index: i64,
) -> Result<ComicChapterPageBatch> {
    let chapter = value
        .get("Chapter")
        .ok_or_else(|| AppError::protocol("漫画页面响应缺少 Chapter"))?;
    Ok(ComicChapterPageBatch {
        chapter_id,
        start_index,
        page_urls: page_urls(chapter),
        page_count: number(chapter, "Total"),
        read_position: read_position(value.get("ReadPosition")),
    })
}

fn book_summary(value: &Value, position: Option<&Value>) -> ComicBook {
    ComicBook {
        id: number(value, "Id").to_string(),
        title: string(value, "Title"),
        read_position: read_position(position),
        chapters: chapter_values(value).iter().map(chapter_summary).collect(),
    }
}

fn chapter_values(book: &Value) -> &[Value] {
    let chapters = array(book, "Chapters");
    if chapters.is_empty() {
        array(book, "Chapter")
    } else {
        chapters
    }
}

fn chapter_summary(chapter: &Value) -> ComicChapterSummary {
    ComicChapterSummary {
        id: number(chapter, "Id").to_string(),
        title: string(chapter, "Title"),
        sequence: number(chapter, "SortNum"),
        page_count: number(chapter, "PageCount"),
    }
}

fn page_urls(chapter: &Value) -> Vec<String> {
    array(chapter, "Images")
        .iter()
        .filter_map(|image| image.as_str().map(str::to_owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{bookshelf_summary, page_batch, summary};

    #[test]
    fn maps_series_summary_by_title() {
        let value = json!({"Id": 42, "Title": "Series title"});

        let mapped = summary(&value);

        assert_eq!(mapped.id, "Series title");
        assert_eq!(mapped.book_id, None);
    }

    #[test]
    fn maps_bookshelf_summary_with_book_id() {
        let value = json!({"Id": 42, "Title": "Series title"});

        let mapped = bookshelf_summary(&value);

        assert_eq!(mapped.id, "Series title");
        assert_eq!(mapped.book_id.as_deref(), Some("42"));
    }

    #[test]
    fn maps_current_string_image_urls() {
        let value = json!({"Chapter": {"Images": ["https://images.example/1.webp"], "Total": 1}});

        let mapped = page_batch("42".into(), &value, 0).expect("page batch");

        assert_eq!(mapped.page_urls, ["https://images.example/1.webp"]);
    }
}
