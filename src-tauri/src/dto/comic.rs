use crate::dto::common::ReadPosition;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComicSummary {
    pub id: String,
    /// 官方书架中的数值分卷 ID；仅书架项目具备该值。
    pub book_id: Option<String>,
    pub title: String,
    pub cover_url: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComicChapterSummary {
    pub id: String,
    pub title: String,
    pub sequence: i64,
    pub page_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComicBook {
    pub id: String,
    pub title: String,
    pub read_position: Option<ReadPosition>,
    pub chapters: Vec<ComicChapterSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComicSeriesDetail {
    #[serde(flatten)]
    pub summary: ComicSummary,
    pub description: Option<String>,
    pub genre: Vec<String>,
    pub status: String,
    pub books: Vec<ComicBook>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComicChapterPageBatch {
    pub chapter_id: String,
    pub start_index: i64,
    pub page_count: i64,
    pub page_urls: Vec<String>,
    pub read_position: Option<ReadPosition>,
}
