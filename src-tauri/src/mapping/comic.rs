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
pub(crate) fn series_detail(
    value: &Value,
    series_title: String,
    books: Vec<ComicBook>,
) -> Result<ComicSeriesDetail> {
    let (series, _) = super::book::detail(value, "Comic")?;
    if books.is_empty() {
        return Err(AppError::protocol("漫画系列不含可阅读分卷"));
    }

    Ok(ComicSeriesDetail {
        summary: ComicSummary {
            id: series_title,
            title: optional_string(value, "SeriesTitle").unwrap_or_else(|| string(series, "Title")),
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
        books,
    })
}

/// 将官方漫画分卷详情响应映射为应用 DTO。
pub(crate) fn book_detail(value: &Value) -> Result<ComicBook> {
    let (book, mut chapters) = super::book::detail(value, "Comic")?;
    chapters.sort_by_key(|chapter| chapter.sort_num);
    Ok(ComicBook {
        id: number(book, "Id").to_string(),
        title: string(book, "Title"),
        read_position: read_position(value.get("ReadPosition")),
        chapters: chapters
            .into_iter()
            .map(|chapter| ComicChapterSummary {
                id: chapter.id.to_string(),
                title: chapter.title,
                sequence: chapter.sort_num,
                page_count: chapter.page_count,
            })
            .collect(),
    })
}

/// Series 只提供分卷摘要，不包含目录；单卷作品仍需保留当前 Book。
pub(crate) fn series_book_ids(value: &Value) -> Result<Vec<i64>> {
    let (book, _) = super::book::detail(value, "Comic")?;
    let current_id = positive_book_id(book)?;
    let series = value
        .get("Series")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::protocol("详情响应缺少 Series 数组"))?;
    let mut ids = Vec::new();
    for item in series {
        let id = positive_book_id(item)?;
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    if !ids.contains(&current_id) {
        ids.insert(0, current_id);
    }
    Ok(ids)
}

pub(crate) fn positive_book_id(value: &Value) -> Result<i64> {
    value
        .get("Id")
        .and_then(Value::as_i64)
        .filter(|id| *id > 0)
        .ok_or_else(|| AppError::protocol("书籍响应缺少有效 Id"))
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

fn page_urls(chapter: &Value) -> Vec<String> {
    array(chapter, "Images")
        .iter()
        .filter_map(|image| image.as_str().map(str::to_owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        book_detail, bookshelf_summary, page_batch, series_book_ids, series_detail, summary,
    };

    fn unified_detail() -> serde_json::Value {
        json!({
            "Book": {"Id": 42, "Type": "Comic", "Title": "第一卷", "Author": "作者",
                "Chapters": [{"Id": 301, "SortNum": 3, "Title": "第三话", "PageCount": 20}]},
            "SeriesTitle": "漫画系列", "Series": [{"Id": 42}, {"Id": 43}, {"Id": 43}],
            "ReadPosition": {"ChapterId": 301, "Position": "7"}
        })
    }

    #[test]
    fn maps_unified_comic_detail_and_series() {
        let value = unified_detail();
        assert_eq!(series_book_ids(&value).unwrap(), [42, 43]);
        let book = book_detail(&value).unwrap();
        assert_eq!(book.id, "42");
        assert_eq!(book.chapters[0].id, "301");
        assert_eq!(book.chapters[0].sequence, 3);
        assert_eq!(book.chapters[0].page_count, 20);
        assert_eq!(book.read_position.as_ref().unwrap().position, "7");
        let detail = series_detail(&value, "入口系列名".into(), vec![book]).unwrap();
        assert_eq!(detail.summary.id, "入口系列名");
        assert_eq!(detail.summary.title, "漫画系列");
        assert_eq!(detail.books.len(), 1);
    }

    #[test]
    fn retains_singleton_book_and_rejects_invalid_series_ids() {
        let mut value = unified_detail();
        value["Series"] = json!([]);
        assert_eq!(series_book_ids(&value).unwrap(), [42]);
        value["Series"] = json!([{"Id": 0}]);
        assert!(series_book_ids(&value).is_err());
        value["Book"]["Type"] = json!("Novel");
        assert!(book_detail(&value).is_err());
    }

    #[test]
    fn maps_series_summary_by_title() {
        let value = json!({"Id": 42, "Title": "Series title"});

        let mapped = summary(&value);

        assert_eq!(mapped.id, "Series title");
        assert_eq!(mapped.book_id, None);
    }

    #[test]
    fn maps_bookshelf_summary_with_book_id() {
        let value = json!({"Id": 42, "Title": "Series title", "Progress": 100});

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
