use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReadPosition {
    pub chapter_id: String,
    pub position: String,
}

/// 官方列表响应中的分页信息，供小说与漫画搜索共同使用。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Pagination {
    pub page: i64,
    pub previous: Option<i64>,
    pub next: Option<i64>,
    pub first: i64,
    pub last: i64,
}

/// 带分页信息的应用列表响应。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PaginatedList<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
}

/// 小说与漫画列表共用的官方排序方式。
#[derive(Debug, Clone, Copy, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Order {
    Latest,
    View,
    New,
}

impl Order {
    pub(crate) fn hub_order(self) -> &'static str {
        match self {
            Self::Latest => "latest",
            Self::View => "view",
            Self::New => "new",
        }
    }
}

/// 小说与漫画搜索共用的请求模式。
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SearchMode {
    Fuzzy,
    Exact,
    Title,
    Author,
    Name,
    Tags,
}

impl SearchMode {
    pub(crate) fn hub_mode(self) -> &'static str {
        match self {
            Self::Fuzzy => "fuzzy",
            Self::Exact => "exact",
            Self::Title => "title",
            Self::Author => "author",
            Self::Name => "name",
            Self::Tags => "tags",
        }
    }
}

/// 列表缓存键；排序、页码和单页数量都会影响官方响应。
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(crate) struct ListKey {
    order: Order,
    page_number: i64,
    page_size: i64,
}

impl ListKey {
    pub(crate) fn new(order: Order, page_number: i64, page_size: i64) -> Self {
        Self {
            order,
            page_number,
            page_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Order, SearchMode};

    #[test]
    fn accepts_only_documented_order_values() {
        assert!(serde_json::from_str::<Order>(r#""view""#).is_ok());
        assert!(serde_json::from_str::<Order>(r#""hot""#).is_err());
    }

    #[test]
    fn accepts_only_documented_search_values() {
        assert!(serde_json::from_str::<SearchMode>(r#""exact""#).is_ok());
        assert!(serde_json::from_str::<SearchMode>(r#""featured""#).is_err());
    }
}
