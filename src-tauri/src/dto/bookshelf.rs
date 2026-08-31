use super::{comic::ComicSummary, novel::NovelSummary};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NovelBookshelfEntry {
    pub book: NovelSummary,
    pub added_at: String,
    pub progress: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComicBookshelfEntry {
    pub comic: ComicSummary,
    pub added_at: String,
    pub progress: Option<i64>,
}
