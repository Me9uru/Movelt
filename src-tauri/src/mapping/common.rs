use serde_json::Value;

use crate::{
    dto::common::{Pagination, ReadPosition},
    error::{AppError, Result},
};

/// 将官方阅读位置映射为应用 DTO。
pub(crate) fn read_position(value: Option<&Value>) -> Option<ReadPosition> {
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

/// 从官方分页响应中构建应用分页信息。
pub(crate) fn pagination(value: &Value, default_page: i64) -> Result<Pagination> {
    let raw = value
        .as_object()
        .ok_or_else(|| AppError::protocol("命令映射时预期对象响应"))?;
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

    use super::{pagination, read_position};

    #[test]
    fn maps_read_position() {
        let value = json!({"ChapterId": 42, "Position": "17"});

        let mapped = read_position(Some(&value)).expect("read position should map");

        assert_eq!(mapped.chapter_id, "42");
        assert_eq!(mapped.position, "17");
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
