use serde_json::Value;

use crate::mapping::value::array;

/// 读取官方书架中的原始条目。
pub(crate) fn items(value: &Value) -> Vec<Value> {
    array(value, "data").to_vec()
}

/// 判断书架条目是否属于指定作品类型。
pub(crate) fn is_kind(item: &Value, kind: &str) -> bool {
    item.get("type")
        .and_then(Value::as_str)
        .is_some_and(|ty| ty.eq_ignore_ascii_case(kind))
        || item.get("type").and_then(Value::as_i64) == Some(if kind == "BOOK" { 0 } else { 1 })
}
