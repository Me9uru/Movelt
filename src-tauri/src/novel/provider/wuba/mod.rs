mod client;
mod parser;

use super::NovelSource;
use crate::novel::{
    domain::{ChapterContent, NovelOverview, SearchResult},
    error::NovelError,
};
use client::WubaClient;

pub(super) const SOURCE_ID: &str = "wuba58";
const SOURCE_NAME: &str = "五八书阁";

pub(super) struct WubaSource {
    client: WubaClient,
}

impl WubaSource {
    pub(super) fn new() -> Result<Self, NovelError> {
        Ok(Self {
            client: WubaClient::new()?,
        })
    }
}

#[async_trait::async_trait]
impl NovelSource for WubaSource {
    fn id(&self) -> &'static str {
        SOURCE_ID
    }

    fn name(&self) -> &'static str {
        SOURCE_NAME
    }

    async fn search(&self, query: &str, _page: u32) -> Result<SearchResult, NovelError> {
        let html = self.client.search(query).await?;
        parser::search(&html, self.client.base_url())
    }

    async fn overview(&self, novel_id: &str) -> Result<NovelOverview, NovelError> {
        validate_numeric_id(novel_id, "novel")?;
        let html = self.client.detail(novel_id).await?;
        parser::overview(&html, novel_id, self.client.base_url())
    }

    async fn chapter(
        &self,
        novel_id: &str,
        chapter_id: &str,
    ) -> Result<ChapterContent, NovelError> {
        validate_numeric_id(novel_id, "novel")?;
        validate_numeric_id(chapter_id, "chapter")?;
        let pages = self.client.chapter_pages(novel_id, chapter_id).await?;
        parser::chapter(&pages, novel_id, chapter_id)
    }
}

fn validate_numeric_id(id: &str, kind: &str) -> Result<(), NovelError> {
    if !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()) {
        Ok(())
    } else {
        Err(NovelError::invalid_input(format!(
            "Wuba {kind} id must contain digits only"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_numeric_remote_ids() {
        assert!(validate_numeric_id("50919", "novel").is_ok());
        assert!(validate_numeric_id("../50919", "novel").is_err());
    }
}
