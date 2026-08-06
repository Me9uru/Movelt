mod bilinovel;
mod wuba;

use std::sync::Arc;

use super::{
    domain::{ChapterContent, NovelOverview, SearchResult},
    error::NovelError,
};
use bilinovel::BilinovelSource;
use wuba::WubaSource;

#[async_trait::async_trait]
pub(super) trait NovelSource: Send + Sync {
    fn id(&self) -> &'static str;

    fn name(&self) -> &'static str;

    async fn search(&self, query: &str, page: u32) -> Result<SearchResult, NovelError>;

    async fn overview(&self, novel_id: &str) -> Result<NovelOverview, NovelError>;

    async fn chapter(&self, novel_id: &str, chapter_id: &str)
        -> Result<ChapterContent, NovelError>;
}

pub(super) fn built_in() -> Result<Vec<Arc<dyn NovelSource>>, NovelError> {
    Ok(vec![
        Arc::new(WubaSource::new()?),
        Arc::new(BilinovelSource::new()?),
    ])
}
