use serde_json::Value;

use crate::mapping::value::array;

fn field<'a>(value: &'a Value, lower: &str, upper: &str) -> Option<&'a Value> {
    value.get(lower).or_else(|| value.get(upper))
}

/// 读取官方书架中的原始条目。
pub(crate) fn items(value: &Value) -> Vec<Value> {
    if let Some(items) = value.as_array() {
        return items.clone();
    }
    let items = array(value, "data");
    if items.is_empty() {
        array(value, "Data").to_vec()
    } else {
        items.to_vec()
    }
}

/// 读取书架条目的作品 ID，兼容新旧字段大小写和数字字符串。
pub(crate) fn item_id(item: &Value) -> i64 {
    field(item, "id", "Id")
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
        .unwrap_or_default()
}

/// 读取书架条目的更新时间。
pub(crate) fn item_updated_at(item: &Value) -> Option<String> {
    field(item, "updateAt", "UpdateAt")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// 读取书架结构版本。
pub(crate) fn version(value: &Value) -> Option<&str> {
    field(value, "ver", "Ver").and_then(Value::as_str)
}

/// 判断书架条目是否属于指定作品类型。
pub(crate) fn is_kind(item: &Value, kind: &str) -> bool {
    field(item, "type", "Type")
        .and_then(Value::as_str)
        .is_some_and(|ty| ty.eq_ignore_ascii_case(kind))
        || field(item, "type", "Type").and_then(Value::as_i64)
            == match kind {
                "BOOK" => Some(0),
                "FOLDER" => Some(1),
                _ => None,
            }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{is_kind, item_id, item_updated_at, items, version};

    #[test]
    fn reads_bookshelf_items_from_both_documented_and_current_casing() {
        let documented = json!({"data": [{"id": 1, "type": "BOOK"}]});
        let current = json!({
            "Data": [{"Id": "2", "Type": "Book", "UpdateAt": "2026-09-03T00:00:00Z"}],
            "Ver": "20220211"
        });

        assert_eq!(items(&documented)[0]["id"], 1);
        let current_item = &items(&current)[0];
        assert_eq!(item_id(current_item), 2);
        assert!(is_kind(current_item, "BOOK"));
        assert_eq!(
            item_updated_at(current_item).as_deref(),
            Some("2026-09-03T00:00:00Z")
        );
        assert_eq!(version(&current), Some("20220211"));
    }

    #[test]
    fn reads_legacy_root_array_and_numeric_item_types() {
        let shelf = json!([{"Id": 3, "Type": 0}]);
        let item = &items(&shelf)[0];

        assert_eq!(item_id(item), 3);
        assert!(is_kind(item, "BOOK"));
        assert!(!is_kind(item, "FOLDER"));
    }
}
