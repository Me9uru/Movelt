use chrono::Utc;
use serde_json::{json, Value};

use crate::{
    api::OfficialClient,
    dto::{
        manga::MangaSummary,
        novel::{DiscoveryList, NovelSummary, Pagination, ReadPosition},
    },
    error::{AppError, Result},
};

pub(super) fn object(value: &Value) -> Result<&serde_json::Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| AppError::InvalidResponse("预期对象响应".into()))
}
pub(super) fn string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}
pub(super) fn optional_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}
/// Sanitizes rich text supplied by LightNovelShelf before it reaches `v-html`.
pub(super) fn optional_html(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ammonia::clean)
        .filter(|value| !value.is_empty())
}
pub(super) fn number(value: &Value, key: &str) -> i64 {
    value.get(key).and_then(Value::as_i64).unwrap_or_default()
}
pub(super) fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}
pub(super) fn novel(value: &Value) -> NovelSummary {
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
pub(super) fn page(value: Value) -> Result<DiscoveryList> {
    let raw = object(&value)?;
    let current = raw.get("Page").and_then(Value::as_i64).unwrap_or(1);
    let last = raw.get("TotalPages").and_then(Value::as_i64).unwrap_or(1);
    Ok(DiscoveryList {
        items: raw
            .get("Data")
            .and_then(Value::as_array)
            .map(|items| items.iter().map(novel).collect())
            .unwrap_or_default(),
        pagination: Pagination {
            page: current,
            previous: (current > 1).then_some(current - 1),
            next: (current < last).then_some(current + 1),
            first: 1,
            last,
        },
    })
}
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
pub(super) fn manga(value: &Value) -> MangaSummary {
    MangaSummary {
        id: number(value, "Id").to_string(),
        title: string(value, "Title"),
        thumbnail_url: optional_string(value, "Cover"),
        author: optional_string(value, "Author"),
        unread_count: 0,
        source_name: Some("LightNovelShelf".into()),
    }
}
pub(super) fn shelf_items(value: &Value) -> Vec<Value> {
    array(value, "data").to_vec()
}
pub(super) fn is_kind(item: &Value, kind: &str) -> bool {
    item.get("type")
        .and_then(Value::as_str)
        .is_some_and(|ty| ty.eq_ignore_ascii_case(kind))
        || item.get("type").and_then(Value::as_i64) == Some(if kind == "BOOK" { 0 } else { 1 })
}
pub(super) async fn books_for_ids(
    client: &OfficialClient,
    ids: Vec<i64>,
    ty: Option<&str>,
) -> Result<Vec<NovelSummary>> {
    let mut books = Vec::new();
    for ids in ids.chunks(24) {
        let mut payload = json!({"Ids": ids});
        if let Some(ty) = ty {
            payload["Type"] = Value::String(ty.into());
        }
        let response = client.hub("GetBookListByIds", payload).await?;
        let items = response
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_else(|| array(&response, "Data"));
        books.extend(items.iter().map(novel));
    }
    Ok(books)
}
pub(super) async fn set_shelf(
    client: &OfficialClient,
    id: i64,
    kind: &str,
    present: bool,
) -> Result<()> {
    let shelf = client.hub("GetBookShelf", json!({})).await?;
    let mut items = shelf_items(&shelf);
    let exists = items
        .iter()
        .any(|item| number(item, "id") == id && is_kind(item, kind));
    if present && !exists {
        items.insert(0, json!({"id": id, "type": kind, "parents": [], "index": 0, "updateAt": Utc::now().to_rfc3339()}));
    }
    if !present {
        items.retain(|item| number(item, "id") != id || !is_kind(item, kind));
    }
    client.hub("SaveBookShelf", json!({"data": items, "ver": shelf.get("ver").and_then(Value::as_str).unwrap_or("20220211")})).await.map(|_| ())
}
pub(super) fn parse_id(value: &str) -> Result<i64> {
    value
        .parse()
        .map_err(|_| AppError::InvalidResponse("无效的作品 ID".into()))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::optional_html;

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
