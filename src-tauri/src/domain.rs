use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NovelSummary {
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
pub(crate) struct NovelDetail {
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
pub(crate) struct NovelOverview {
    pub detail: NovelDetail,
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Volume {
    pub title: String,
    pub chapters: Vec<ChapterSummary>,
    pub sections: Vec<Volume>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ChapterSummary {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ChapterContent {
    pub source: String,
    pub novel_id: String,
    pub chapter_id: String,
    pub title: String,
    pub nodes: Vec<ChapterNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ChapterNode {
    Paragraph { text: String },
    Image { url: String, alt: Option<String> },
}
