use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NovelSummary {
    pub source: String,
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageInfo {
    pub page: u32,
    pub previous: Option<u32>,
    pub next: Option<u32>,
    pub first: u32,
    pub last: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryList {
    pub items: Vec<NovelSummary>,
    pub pagination: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecommendBlock {
    pub title: String,
    pub items: Vec<NovelSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthStatus {
    pub logged_in: bool,
    pub base_url: String,
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
    #[serde(default)]
    pub tags: Vec<String>,
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
    pub sections: Vec<Volume>,
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
