use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct NovelSummary {
    pub source: String,
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub author: Option<String>,
    pub status: Option<String>,
    pub updated_at: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Pagination {
    pub page: i64,
    pub previous: Option<i64>,
    pub next: Option<i64>,
    pub first: i64,
    pub last: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiscoveryList {
    pub items: Vec<NovelSummary>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChapterSummary {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Volume {
    pub title: String,
    pub chapters: Vec<ChapterSummary>,
    pub sections: Vec<Volume>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReadPosition {
    pub chapter_id: String,
    pub position: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NovelOverview {
    pub detail: NovelSummary,
    pub volumes: Vec<Volume>,
    pub read_position: Option<ReadPosition>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReaderDocument {
    pub id: String,
    pub book_id: String,
    pub chapter_id: String,
    pub server_chapter_id: String,
    pub title: String,
    pub html: String,
    pub font_url: Option<String>,
    pub read_position: Option<ReadPosition>,
}
