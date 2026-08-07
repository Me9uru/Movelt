mod archive;
pub(crate) mod commands;
mod content;
mod package;
mod path;
mod service;

use super::NovelSource;
use crate::novel::{
    domain::{ChapterContent, NovelOverview, SearchResult},
    error::NovelError,
};

pub(crate) use service::LocalEpubSource;

const SOURCE_NAME: &str = "本地 EPUB";

#[async_trait::async_trait]
impl NovelSource for LocalEpubSource {
    fn id(&self) -> &'static str {
        Self::SOURCE_ID
    }

    fn name(&self) -> &'static str {
        SOURCE_NAME
    }

    async fn search(&self, _query: &str, _page: u32) -> Result<SearchResult, NovelError> {
        Err(NovelError::invalid_input("本地 EPUB 不支持搜索"))
    }

    async fn overview(&self, novel_id: &str) -> Result<NovelOverview, NovelError> {
        self.overview(novel_id)
    }

    async fn chapter(
        &self,
        novel_id: &str,
        chapter_id: &str,
    ) -> Result<ChapterContent, NovelError> {
        self.chapter(novel_id, chapter_id)
    }
}
