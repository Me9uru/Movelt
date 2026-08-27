use serde_json::Value;

/// 读取字符串字段，缺失时返回空字符串。
pub(crate) fn string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// 读取非空字符串字段。
pub(crate) fn optional_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// 清洗并读取非空 HTML 字段。
pub(crate) fn optional_html(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ammonia::clean)
        .filter(|value| !value.is_empty())
}

/// 读取整数字段，兼容官方接口返回的数字字符串，缺失时返回零。
pub(crate) fn number(value: &Value, key: &str) -> i64 {
    value
        .get(key)
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
        .unwrap_or_default()
}

/// 读取可选整数，兼容官方接口返回的数字字符串。
pub(crate) fn optional_number(value: &Value, key: &str) -> Option<i64> {
    value
        .get(key)
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
}

/// 读取数组字段，缺失时返回空切片。
pub(crate) fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{number, optional_html};

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
}
