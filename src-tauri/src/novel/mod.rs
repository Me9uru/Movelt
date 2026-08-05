mod bilinovel;
mod domain;
mod error;

use bilinovel::BilinovelSource;
pub use domain::{ChapterContent, NovelDetail, NovelOverview, SearchResult};
pub use error::NovelError;
use tauri::State;

pub struct NovelState {
    source: BilinovelSource,
}

impl NovelState {
    pub fn new() -> Result<Self, NovelError> {
        Ok(Self {
            source: BilinovelSource::new()?,
        })
    }
}

#[tauri::command]
pub async fn search_novels(
    state: State<'_, NovelState>,
    query: String,
    page: Option<u32>,
) -> Result<SearchResult, NovelError> {
    let query = query.trim();
    if query.is_empty() || query.chars().count() > 100 {
        return Err(NovelError::invalid_input(
            "query must contain between 1 and 100 characters",
        ));
    }
    let page = page.unwrap_or(1);
    if page == 0 || page > 1_000 {
        return Err(NovelError::invalid_input("page must be between 1 and 1000"));
    }
    state.source.search(query, page).await
}

#[tauri::command]
pub async fn get_novel_overview(
    state: State<'_, NovelState>,
    novel_id: String,
) -> Result<NovelOverview, NovelError> {
    state.source.overview(&novel_id).await
}

#[tauri::command]
pub async fn get_chapter(
    state: State<'_, NovelState>,
    novel_id: String,
    chapter_id: String,
) -> Result<ChapterContent, NovelError> {
    state.source.chapter(&novel_id, &chapter_id).await
}
