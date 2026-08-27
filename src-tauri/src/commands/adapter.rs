use serde_json::Value;

use crate::{
    dto::common::{Pagination, ReadPosition},
    error::{AppError, Result},
};

/// 读取 JSON 对象，不符合预期时返回协议错误。
pub(super) fn object(value: &Value) -> Result<&serde_json::Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| AppError::protocol("命令映射时预期对象响应"))
}

/// 读取字符串字段，缺失时返回空字符串。
pub(super) fn string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// 读取非空字符串字段。
pub(super) fn optional_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// 清洗并读取非空 HTML 字段。
pub(super) fn optional_html(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ammonia::clean)
        .filter(|value| !value.is_empty())
}

/// 读取整数字段，兼容官方接口返回的数字字符串，缺失时返回零。
pub(super) fn number(value: &Value, key: &str) -> i64 {
    value
        .get(key)
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
        .unwrap_or_default()
}

/// 读取数组字段，缺失时返回空切片。
pub(super) fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// 将官方阅读位置映射为应用 DTO。
pub(super) fn position(value: Option<&Value>) -> Option<ReadPosition> {
    let value = value?.as_object()?;
    Some(ReadPosition {
        chapter_id: value.get("ChapterId")?.as_i64()?.to_string(),
        position: value
            .get("Position")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
    })
}

/// 校验并解析作品或章节 ID。
pub(super) fn parse_id(value: &str) -> Result<i64> {
    value
        .parse()
        .map_err(|_| AppError::invalid_input("作品 ID 必须是数字"))
}

/// 校验分页大小，避免将空值或负数透传给官方接口。
pub(super) fn validate_page_size(value: i64) -> Result<i64> {
    if value > 0 {
        Ok(value)
    } else {
        Err(AppError::invalid_input("分页大小必须大于 0"))
    }
}

/// 从官方分页响应中构建应用分页信息。
pub(super) fn pagination(value: &Value, default_page: i64) -> Result<Pagination> {
    let raw = object(value)?;
    let page = raw
        .get("Page")
        .and_then(Value::as_i64)
        .unwrap_or(default_page);
    let last = raw.get("TotalPages").and_then(Value::as_i64).unwrap_or(1);

    Ok(Pagination {
        page,
        previous: (page > 1).then_some(page - 1),
        next: (page < last).then_some(page + 1),
        first: 1,
        last,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{number, optional_html, pagination, position, validate_page_size};

    #[test]
    fn reads_numeric_strings_as_numbers() {
        let value = json!({"Id": "42"});

        assert_eq!(number(&value, "Id"), 42);
    }

    #[test]
    fn sanitizes_introduction_html_without_losing_paragraphs() {
        let value = json!({
            "Introduction": "<p>第一段</p><p onclick=\"alert(1)\">第二段</p><script>alert(1)</script>",
        });

        let introduction = optional_html(&value, "Introduction").expect("introduction");

        assert!(introduction.contains("<p>第一段</p>"));
        assert!(introduction.contains("<p>第二段</p>"));
        assert!(!introduction.contains("script"));
        assert!(!introduction.contains("onclick"));
    }

    #[test]
    fn maps_comic_read_position() {
        let value = json!({"ChapterId": 42, "Position": "17"});

        let mapped = position(Some(&value)).expect("read position should map");

        assert_eq!(mapped.chapter_id, "42");
        assert_eq!(mapped.position, "17");
    }

    #[test]
    fn rejects_non_positive_page_sizes() {
        assert_eq!(validate_page_size(6).expect("positive page size"), 6);
        assert!(validate_page_size(0).is_err());
    }

    #[test]
    fn builds_pagination_from_official_response() {
        let value = json!({"Page": 2, "TotalPages": 3});

        let mapped = pagination(&value, 1).expect("pagination");

        assert_eq!(mapped.page, 2);
        assert_eq!(mapped.previous, Some(1));
        assert_eq!(mapped.next, Some(3));
        assert_eq!(mapped.last, 3);
    }
}
