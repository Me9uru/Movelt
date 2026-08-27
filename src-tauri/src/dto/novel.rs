use crate::dto::common::ReadPosition;
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
pub(crate) struct NovelChapterSummary {
    pub id: String,
    pub title: String,
    pub sequence: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NovelDetail {
    #[serde(flatten)]
    pub summary: NovelSummary,
    pub chapters: Vec<NovelChapterSummary>,
    pub read_position: Option<ReadPosition>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NovelChapterContent {
    pub chapter_id: String,
    pub server_chapter_id: String,
    pub title: String,
    pub html: String,
    pub font_url: Option<String>,
    pub read_position: Option<ReadPosition>,
}
