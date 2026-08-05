use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    pub page: u32,
    pub total_pages: u32,
    pub items: Vec<NovelSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NovelSummary {
    pub source: String,
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NovelDetail {
    pub source: String,
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub status: Option<String>,
    pub updated_at: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NovelOverview {
    pub detail: NovelDetail,
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Volume {
    pub title: String,
    pub chapters: Vec<ChapterSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChapterSummary {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChapterContent {
    pub source: String,
    pub novel_id: String,
    pub chapter_id: String,
    pub title: String,
    pub nodes: Vec<ChapterNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChapterNode {
    Paragraph { text: String },
    Image { url: String, alt: Option<String> },
}
