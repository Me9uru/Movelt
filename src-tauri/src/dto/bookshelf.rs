use super::novel::NovelSummary;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BookshelfEntry {
    pub book: NovelSummary,
    pub added_at: String,
    pub progress: Option<()>,
}
