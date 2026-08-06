mod client;
mod parser;

use super::NovelSource;
use crate::novel::{
    domain::{ChapterContent, NovelOverview, SearchResult},
    error::NovelError,
};
use client::BilinovelClient;

pub struct BilinovelSource {
    client: BilinovelClient,
}

pub(super) const SOURCE_ID: &str = "bilinovel";
const SOURCE_NAME: &str = "哔哩轻小说";

impl BilinovelSource {
    pub fn new() -> Result<Self, NovelError> {
        Ok(Self {
            client: BilinovelClient::new()?,
        })
    }
}

#[async_trait::async_trait]
impl NovelSource for BilinovelSource {
    fn id(&self) -> &'static str {
        SOURCE_ID
    }

    fn name(&self) -> &'static str {
        SOURCE_NAME
    }

    async fn search(&self, query: &str, page: u32) -> Result<SearchResult, NovelError> {
        let novels = self.client.novels().await?;
        Ok(parser::search(&novels, query, page))
    }

    async fn overview(&self, novel_id: &str) -> Result<NovelOverview, NovelError> {
        validate_numeric_id(novel_id)?;
        let novel = self.client.novel(novel_id).await?;
        let detail = parser::detail(&novel);
        let volumes =
            futures::future::try_join_all(novel.volumes.iter().map(|summary| async move {
                let volume = self.client.volume(novel_id, summary.vid).await?;
                Ok::<_, NovelError>(parser::volume(&volume))
            }))
            .await?;
        Ok(NovelOverview { detail, volumes })
    }

    async fn chapter(
        &self,
        novel_id: &str,
        chapter_id: &str,
    ) -> Result<ChapterContent, NovelError> {
        validate_numeric_id(novel_id)?;
        validate_numeric_id(chapter_id)?;
        let chapter = self.client.chapter(novel_id, chapter_id).await?;
        parser::chapter(&chapter)
    }
}

fn validate_numeric_id(id: &str) -> Result<(), NovelError> {
    if !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()) {
        Ok(())
    } else {
        Err(NovelError::invalid_input(
            "novel and chapter ids must contain digits only",
        ))
    }
}
