use serde::Serialize;
use super::novel::NovelSummary;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookshelfEntry {
    pub book: NovelSummary,
    pub added_at: String,
    pub progress: Option<()>,
}
