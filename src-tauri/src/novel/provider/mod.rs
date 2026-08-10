pub(crate) mod local_epub;
pub(super) mod wenku8_api;

use super::{
    domain::{ChapterContent, NovelOverview},
    error::NovelError,
};

#[async_trait::async_trait]
pub(super) trait NovelSource: Send + Sync {
    fn id(&self) -> &'static str;

    async fn overview(&self, novel_id: &str) -> Result<NovelOverview, NovelError>;

    async fn chapter(&self, novel_id: &str, chapter_id: &str)
        -> Result<ChapterContent, NovelError>;

    async fn chapter_with_title(
        &self,
        novel_id: &str,
        chapter_id: &str,
        _title: Option<&str>,
    ) -> Result<ChapterContent, NovelError> {
        self.chapter(novel_id, chapter_id).await
    }
}
