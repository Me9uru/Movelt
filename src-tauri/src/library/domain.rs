use serde::{Deserialize, Serialize};

use crate::novel::domain::NovelDetail;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReadingProgress {
    pub document_id: String,
    pub document_title: String,
    pub location: f64,
    pub book_location: f64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingProgressInput {
    pub document_id: String,
    pub document_title: String,
    pub location: f64,
    pub book_location: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BookshelfEntry {
    pub book: NovelDetail,
    pub added_at: String,
    pub progress: Option<ReadingProgress>,
}
