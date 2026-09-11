use serde::Deserialize;
use serde_json::Value;

use crate::error::{AppError, Result};

/// 官方统一详情中的章节；与前端使用的领域 DTO 分离。
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Chapter {
    pub id: i64,
    pub sort_num: i64,
    pub title: String,
    pub page_count: i64,
}

pub(super) fn detail<'a>(value: &'a Value, kind: &str) -> Result<(&'a Value, Vec<Chapter>)> {
    let book = value
        .get("Book")
        .filter(|book| book.is_object())
        .ok_or_else(|| AppError::protocol("详情响应缺少 Book 对象"))?;
    if book.get("Type").and_then(Value::as_str) != Some(kind) {
        return Err(AppError::protocol("详情 Book.Type 与请求的作品类型不符"));
    }
    super::value::required_id(book, "Id")?;
    super::value::required_string(book, "Title")?;
    let chapters: Vec<Chapter> = serde_json::from_value(
        book.get("Chapters")
            .cloned()
            .ok_or_else(|| AppError::protocol("详情响应缺少 Book.Chapters"))?,
    )
    .map_err(|_| AppError::protocol("Book.Chapters 格式无效"))?;
    if chapters
        .iter()
        .any(|chapter| chapter.id <= 0 || chapter.sort_num <= 0)
    {
        return Err(AppError::protocol("章节 Id 和 SortNum 必须大于 0"));
    }
    Ok((book, chapters))
}
