use serde::Serialize;

use super::novel::ReadPosition;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaSummary {
    pub id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub author: Option<String>,
    pub unread_count: i64,
    pub source_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaChapter {
    pub id: String,
    pub name: String,
    pub chapter_number: i64,
    pub is_read: bool,
    pub last_page_read: i64,
    pub page_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaDetail {
    #[serde(flatten)]
    pub summary: MangaSummary,
    pub artist: Option<String>,
    pub description: Option<String>,
    pub genre: Vec<String>,
    pub status: String,
    pub read_position: Option<ReadPosition>,
    pub chapters: Vec<MangaChapter>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaPageList {
    pub chapter_id: String,
    pub page_count: i64,
    pub first_page_urls: Vec<String>,
    pub read_position: Option<ReadPosition>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MangaPageBatch {
    pub start_index: i64,
    pub page_urls: Vec<String>,
}
